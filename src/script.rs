//! "Al hacer clic" code: a small, sandboxed script (Rhai) run when an element is
//! clicked in test mode. Functions act on elements by their layer name.

use crate::model::{Doc, Element, Rgba};
use rhai::{Dynamic, Engine, Scope};
use std::{cell::RefCell, rc::Rc};

pub const HELP: &str = "Functions (the names are those in the layer list; “yo” is the clicked element):
  mostrar(\"Name\")      ocultar(\"Name\")      alternar(\"Name\")
  texto(\"Name\", \"Hello\")   leer_texto(\"Name\")
  color(\"Name\", \"#FF8800\")   color_texto(\"Name\", \"#FFFFFF\")
  mover(\"Name\", dx, dy)   posicion(\"Name\", x, y)   girar(\"Name\", degrees)
  valor(\"Name\", 0.5)   leer_valor(\"Name\")   transparencia(\"Name\", 0.5)
  mensaje(\"text\")  — also usable: let, if, for, while, +, etc.";

/// Number from a script argument; NaN or infinity count as 0 (they must not reach the coordinates).
fn num(d: &Dynamic) -> f32 {
    let v = d.as_float().map(|f| f as f32).or_else(|_| d.as_int().map(|i| i as f32)).unwrap_or(0.0);
    if v.is_finite() { v.clamp(-1.0e6, 1.0e6) } else { 0.0 }
}

/// Changes an element's frame by its name; children follow according to their anchors (as when moving it in the editor).
fn with_frame(doc: &Shared, log: &Log, name: &str, f: impl FnOnce(&mut crate::model::Frame)) {
    let mut d = doc.borrow_mut();
    match d.elements.iter().find(|e| e.name == name).map(|e| (e.id, e.frame())) {
        Some((id, mut fr)) => {
            f(&mut fr);
            d.set_frame_keep_children(id, &fr);
        }
        None => log.borrow_mut().push(format!("⚠ There is no element named “{}”", name)),
    }
}

fn parse_hex(s: &str) -> Option<Rgba> {
    let s = s.trim().trim_start_matches('#');
    let v = |i: usize| u8::from_str_radix(s.get(i..i + 2)?, 16).ok();
    match s.len() {
        6 => Some([v(0)?, v(2)?, v(4)?, 255]),
        8 => Some([v(0)?, v(2)?, v(4)?, v(6)?]),
        _ => None,
    }
}

type Shared = Rc<RefCell<Doc>>;
type Log = Rc<RefCell<Vec<String>>>;

fn with_el(doc: &Shared, log: &Log, name: &str, f: impl FnOnce(&mut Element)) {
    let mut d = doc.borrow_mut();
    match d.elements.iter_mut().find(|e| e.name == name) {
        Some(e) => f(e),
        None => log.borrow_mut().push(format!("⚠ There is no element named “{}”", name)),
    }
}

/// Run `code` as if element `id` was clicked. Changes are applied to `doc`;
/// messages and errors are appended to `log`.
pub fn run(doc: &mut Doc, id: u64, code: &str, log: &mut Vec<String>) {
    let me = doc.get(id).map(|e| e.name.clone()).unwrap_or_default();
    let shared: Shared = Rc::new(RefCell::new(doc.clone()));
    let out: Log = Rc::new(RefCell::new(Vec::new()));
    let mut engine = Engine::new();
    engine.set_max_operations(500_000);
    engine.set_max_string_size(100_000);
    {
        let o = out.clone();
        engine.on_print(move |s| o.borrow_mut().push(s.to_string()));
    }
    {
        let o = out.clone();
        engine.on_debug(move |s, _, _| o.borrow_mut().push(s.to_string()));
    }
    macro_rules! reg {
        ($name:literal, |$d:ident, $l:ident $(, $a:ident : $t:ty)*| $body:expr) => {{
            let $d = shared.clone();
            let $l = out.clone();
            engine.register_fn($name, move |$($a: $t),*| $body);
        }};
    }
    reg!("mensaje", |_d, l, s: Dynamic| l.borrow_mut().push(s.to_string()));
    reg!("mostrar", |d, l, n: &str| with_el(&d, &l, n, |e| e.visible = true));
    reg!("ocultar", |d, l, n: &str| with_el(&d, &l, n, |e| e.visible = false));
    reg!("alternar", |d, l, n: &str| with_el(&d, &l, n, |e| e.visible = !e.visible));
    reg!("texto", |d, l, n: &str, v: Dynamic| with_el(&d, &l, n, |e| e.text = v.to_string()));
    reg!("leer_texto", |d, l, n: &str| match d.borrow().elements.iter().find(|e| e.name == n) {
        Some(e) => e.text.clone(),
        None => {
            l.borrow_mut().push(format!("⚠ There is no element named “{}”", n));
            String::new()
        }
    });
    reg!("leer_valor", |d, l, n: &str| match d.borrow().elements.iter().find(|e| e.name == n) {
        Some(e) => e.value as f64,
        None => {
            l.borrow_mut().push(format!("⚠ There is no element named “{}”", n));
            0.0
        }
    });
    // Slider and bars: 0 to 1. Dropdown: option number (0, 1, 2…).
    reg!("valor", |d, l, n: &str, v: Dynamic| with_el(&d, &l, n, |e| {
        e.value = if e.kind == crate::model::Kind::Dropdown {
            num(&v).round().clamp(0.0, e.options.len().saturating_sub(1) as f32)
        } else {
            num(&v).clamp(0.0, 1.0)
        }
    }));
    reg!("transparencia", |d, l, n: &str, v: Dynamic| with_el(&d, &l, n, |e| e.opacity = num(&v).clamp(0.0, 1.0)));
    reg!("mover", |d, l, n: &str, dx: Dynamic, dy: Dynamic| with_frame(&d, &l, n, |f| {
        f.cx += num(&dx);
        f.cy += num(&dy);
    }));
    // posicion: top-left corner, like x/y in the inspector.
    reg!("posicion", |d, l, n: &str, x: Dynamic, y: Dynamic| with_frame(&d, &l, n, |f| {
        f.cx = num(&x) + f.w / 2.0;
        f.cy = num(&y) + f.h / 2.0;
    }));
    reg!("girar", |d, l, n: &str, g: Dynamic| with_frame(&d, &l, n, |f| f.rot = num(&g) % 360.0));
    reg!("color", |d, l, n: &str, c: &str| match parse_hex(c) {
        Some(rgba) => with_el(&d, &l, n, |e| e.fill = rgba),
        None => l.borrow_mut().push(format!("⚠ Invalid color: {}", c)),
    });
    reg!("color_texto", |d, l, n: &str, c: &str| match parse_hex(c) {
        Some(rgba) => with_el(&d, &l, n, |e| e.text_color = rgba),
        None => l.borrow_mut().push(format!("⚠ Invalid color: {}", c)),
    });

    let mut scope = Scope::new();
    scope.push_constant("yo", me.clone());
    let result = engine.run_with_scope(&mut scope, code);
    drop(engine);
    if let Err(err) = result {
        out.borrow_mut().push(format!("⚠ Error in the code of “{}”: {}", me, err));
    }
    *doc = shared.borrow().clone();
    log.extend(out.borrow().iter().cloned());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;

    #[test]
    fn click_code_changes_elements() {
        let mut doc = model::template(0, 1920.0, 1080.0);
        let id = doc.elements.iter().find(|e| e.name == "Play button").unwrap().id;
        let mut log = Vec::new();
        run(&mut doc, id, "texto(\"Subtitle\", \"Hello \" + 3);\nocultar(\"Moon\");\ncolor(yo, \"#46BE78\");\nmover(yo, 10, 5.5);\nmensaje(leer_texto(\"Subtitle\"));\nmostrar(\"NoExiste\");", &mut log);
        let get = |n: &str| doc.elements.iter().find(|e| e.name == n).unwrap().clone();
        assert_eq!(get("Subtitle").text, "Hello 3");
        assert!(!get("Moon").visible);
        assert_eq!(get("Play button").fill, [0x46, 0xBE, 0x78, 255]);
        assert_eq!(log[0], "Hello 3");
        assert!(log[1].contains("NoExiste"), "{:?}", log);
    }

    #[test]
    fn mover_lleva_a_los_hijos_y_valor_respeta_desplegables() {
        let mut doc = model::template(2, 1920.0, 1080.0);
        let get = |d: &Doc, n: &str| d.elements.iter().find(|e| e.name == n).unwrap().clone();
        let (v0, t0) = (get(&doc, "Window"), get(&doc, "Window title"));
        let mut log = Vec::new();
        run(&mut doc, v0.id, "mover(\"Window\", 100, -20);\nmover(\"Window\", 1.0/0.0, 0);", &mut log);
        let (v1, t1) = (get(&doc, "Window"), get(&doc, "Window title"));
        assert!((v1.x - v0.x - 100.0).abs() < 0.01 && (v1.y - v0.y + 20.0).abs() < 0.01, "{:?}", log);
        assert!((t1.x - t0.x - 100.0).abs() < 0.01 && (t1.y - t0.y + 20.0).abs() < 0.01, "the child follows the parent");
        assert!(doc.elements.iter().all(|e| e.x.is_finite() && e.y.is_finite()));

        let mut doc = model::template(3, 1920.0, 1080.0);
        let id = get(&doc, "Quality").id;
        run(&mut doc, id, "valor(\"Quality\", 3); valor(\"Slider Brightness\", 7);", &mut log);
        assert_eq!(get(&doc, "Quality").value, 3.0);
        assert_eq!(get(&doc, "Slider Brightness").value, 1.0);
        run(&mut doc, id, "valor(\"Quality\", 99);", &mut log);
        assert_eq!(get(&doc, "Quality").value, 3.0, "does not go past the last option");
    }

    #[test]
    fn errors_and_endless_loops_are_reported() {
        let mut doc = model::template(0, 1920.0, 1080.0);
        let mut log = Vec::new();
        run(&mut doc, 1, "loop { }", &mut log);
        run(&mut doc, 1, "this is not code (", &mut log);
        assert_eq!(log.len(), 2);
        assert!(log.iter().all(|l| l.starts_with('⚠')));
    }
}
