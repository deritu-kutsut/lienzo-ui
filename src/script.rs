//! "Al hacer clic" code: a small, sandboxed script (Rhai) run when an element is
//! clicked in test mode. Functions act on elements by their layer name.

use crate::model::{Doc, Element, Rgba};
use rhai::{Dynamic, Engine, Scope};
use std::{cell::RefCell, rc::Rc};

pub const HELP: &str = "Funciones (los nombres son los de la lista de capas; «yo» es el elemento clicado):
  mostrar(\"Nombre\")      ocultar(\"Nombre\")      alternar(\"Nombre\")
  texto(\"Nombre\", \"Hola\")   leer_texto(\"Nombre\")
  color(\"Nombre\", \"#FF8800\")   color_texto(\"Nombre\", \"#FFFFFF\")
  mover(\"Nombre\", dx, dy)   posicion(\"Nombre\", x, y)   girar(\"Nombre\", grados)
  valor(\"Nombre\", 0.5)   leer_valor(\"Nombre\")   transparencia(\"Nombre\", 0.5)
  mensaje(\"texto\")  — también sirven let, if, for, while, +, etc.";

fn num(d: &Dynamic) -> f32 {
    d.as_float().map(|f| f as f32).or_else(|_| d.as_int().map(|i| i as f32)).unwrap_or(0.0)
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
        None => log.borrow_mut().push(format!("⚠ No existe un elemento llamado «{}»", name)),
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
    reg!("leer_texto", |d, _l, n: &str| d.borrow().elements.iter().find(|e| e.name == n).map(|e| e.text.clone()).unwrap_or_default());
    reg!("leer_valor", |d, _l, n: &str| d.borrow().elements.iter().find(|e| e.name == n).map(|e| e.value as f64).unwrap_or(0.0));
    reg!("valor", |d, l, n: &str, v: Dynamic| with_el(&d, &l, n, |e| e.value = num(&v).clamp(0.0, 1.0)));
    reg!("transparencia", |d, l, n: &str, v: Dynamic| with_el(&d, &l, n, |e| e.opacity = num(&v).clamp(0.0, 1.0)));
    reg!("mover", |d, l, n: &str, dx: Dynamic, dy: Dynamic| with_el(&d, &l, n, |e| {
        e.x += num(&dx);
        e.y += num(&dy);
    }));
    reg!("posicion", |d, l, n: &str, x: Dynamic, y: Dynamic| with_el(&d, &l, n, |e| {
        e.x = num(&x);
        e.y = num(&y);
    }));
    reg!("girar", |d, l, n: &str, g: Dynamic| with_el(&d, &l, n, |e| e.rot = num(&g)));
    reg!("color", |d, l, n: &str, c: &str| match parse_hex(c) {
        Some(rgba) => with_el(&d, &l, n, |e| e.fill = rgba),
        None => l.borrow_mut().push(format!("⚠ Color no válido: {}", c)),
    });
    reg!("color_texto", |d, l, n: &str, c: &str| match parse_hex(c) {
        Some(rgba) => with_el(&d, &l, n, |e| e.text_color = rgba),
        None => l.borrow_mut().push(format!("⚠ Color no válido: {}", c)),
    });

    let mut scope = Scope::new();
    scope.push_constant("yo", me.clone());
    let result = engine.run_with_scope(&mut scope, code);
    drop(engine);
    if let Err(err) = result {
        out.borrow_mut().push(format!("⚠ Error en el código de «{}»: {}", me, err));
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
        let id = doc.elements.iter().find(|e| e.name == "Botón Jugar").unwrap().id;
        let mut log = Vec::new();
        run(&mut doc, id, "texto(\"Subtítulo\", \"Hola \" + 3);\nocultar(\"Luna\");\ncolor(yo, \"#46BE78\");\nmover(yo, 10, 5.5);\nmensaje(leer_texto(\"Subtítulo\"));\nmostrar(\"NoExiste\");", &mut log);
        let get = |n: &str| doc.elements.iter().find(|e| e.name == n).unwrap().clone();
        assert_eq!(get("Subtítulo").text, "Hola 3");
        assert!(!get("Luna").visible);
        assert_eq!(get("Botón Jugar").fill, [0x46, 0xBE, 0x78, 255]);
        assert_eq!(log[0], "Hola 3");
        assert!(log[1].contains("NoExiste"), "{:?}", log);
    }

    #[test]
    fn errors_and_endless_loops_are_reported() {
        let mut doc = model::template(0, 1920.0, 1080.0);
        let mut log = Vec::new();
        run(&mut doc, 1, "loop { }", &mut log);
        run(&mut doc, 1, "esto no es código (", &mut log);
        assert_eq!(log.len(), 2);
        assert!(log.iter().all(|l| l.starts_with('⚠')));
    }
}
