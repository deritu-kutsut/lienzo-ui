//! "On click" code: a small, sandboxed script (Rhai) run when an element is
//! clicked in test mode. Functions act on elements by their layer name.

use crate::model::{Doc, Element, Rgba};
use rhai::{Dynamic, Engine, Scope};
use std::{cell::RefCell, rc::Rc};

pub const HELP: &str = "Functions (the names are those in the layer list; “me” is the clicked element):
  show(\"Name\")      hide(\"Name\")      toggle(\"Name\")
  set_text(\"Name\", \"Hello\")   read_text(\"Name\")
  set_color(\"Name\", \"#FF8800\")   set_text_color(\"Name\", \"#FFFFFF\")
  move_by(\"Name\", dx, dy)   set_position(\"Name\", x, y)   rotate(\"Name\", degrees)
  set_value(\"Name\", 0.5)   read_value(\"Name\")   set_opacity(\"Name\", 0.5)
  message(\"text\")  — also usable: let, if, for, while, +, etc.";

/// Old Spanish names of the script functions and the clicked-element constant (`yo`). Scripts saved in designs or
/// plugins with those names keep working: every function is registered under both names.
pub const LEGACY_NAMES: &[(&str, &str)] = &[
    ("message", "mensaje"),
    ("show", "mostrar"),
    ("hide", "ocultar"),
    ("toggle", "alternar"),
    ("set_text", "texto"),
    ("read_text", "leer_texto"),
    ("set_color", "color"),
    ("set_text_color", "color_texto"),
    ("move_by", "mover"),
    ("set_position", "posicion"),
    ("rotate", "girar"),
    ("set_value", "valor"),
    ("read_value", "leer_valor"),
    ("set_opacity", "transparencia"),
];

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
    // Each function is registered under its English name and under the old Spanish one (see `LEGACY_NAMES`).
    macro_rules! reg {
        ($name:literal, |$d:ident, $l:ident $(, $a:ident : $t:ty)*| $body:expr) => {{
            let $d = shared.clone();
            let $l = out.clone();
            let f = move |$($a: $t),*| $body;
            engine.register_fn($name, f.clone());
            if let Some((_, old)) = LEGACY_NAMES.iter().find(|(new, _)| *new == $name) {
                engine.register_fn(*old, f);
            }
        }};
    }
    reg!("message", |_d, l, s: Dynamic| l.borrow_mut().push(s.to_string()));
    reg!("show", |d, l, n: &str| with_el(&d, &l, n, |e| e.visible = true));
    reg!("hide", |d, l, n: &str| with_el(&d, &l, n, |e| e.visible = false));
    reg!("toggle", |d, l, n: &str| with_el(&d, &l, n, |e| e.visible = !e.visible));
    reg!("set_text", |d, l, n: &str, v: Dynamic| with_el(&d, &l, n, |e| e.text = v.to_string()));
    reg!("read_text", |d, l, n: &str| match d.borrow().elements.iter().find(|e| e.name == n) {
        Some(e) => e.text.clone(),
        None => {
            l.borrow_mut().push(format!("⚠ There is no element named “{}”", n));
            String::new()
        }
    });
    reg!("read_value", |d, l, n: &str| match d.borrow().elements.iter().find(|e| e.name == n) {
        Some(e) => e.value as f64,
        None => {
            l.borrow_mut().push(format!("⚠ There is no element named “{}”", n));
            0.0
        }
    });
    // Slider and bars: 0 to 1. Dropdown: option number (0, 1, 2…).
    reg!("set_value", |d, l, n: &str, v: Dynamic| with_el(&d, &l, n, |e| {
        e.value = if e.kind == crate::model::Kind::Dropdown {
            num(&v).round().clamp(0.0, e.options.len().saturating_sub(1) as f32)
        } else {
            num(&v).clamp(0.0, 1.0)
        }
    }));
    reg!("set_opacity", |d, l, n: &str, v: Dynamic| with_el(&d, &l, n, |e| e.opacity = num(&v).clamp(0.0, 1.0)));
    reg!("move_by", |d, l, n: &str, dx: Dynamic, dy: Dynamic| with_frame(&d, &l, n, |f| {
        f.cx += num(&dx);
        f.cy += num(&dy);
    }));
    // posicion: top-left corner, like x/y in the inspector.
    reg!("set_position", |d, l, n: &str, x: Dynamic, y: Dynamic| with_frame(&d, &l, n, |f| {
        f.cx = num(&x) + f.w / 2.0;
        f.cy = num(&y) + f.h / 2.0;
    }));
    reg!("rotate", |d, l, n: &str, g: Dynamic| with_frame(&d, &l, n, |f| f.rot = num(&g) % 360.0));
    reg!("set_color", |d, l, n: &str, c: &str| match parse_hex(c) {
        Some(rgba) => with_el(&d, &l, n, |e| e.fill = rgba),
        None => l.borrow_mut().push(format!("⚠ Invalid color: {}", c)),
    });
    reg!("set_text_color", |d, l, n: &str, c: &str| match parse_hex(c) {
        Some(rgba) => with_el(&d, &l, n, |e| e.text_color = rgba),
        None => l.borrow_mut().push(format!("⚠ Invalid color: {}", c)),
    });

    let mut scope = Scope::new();
    scope.push_constant("me", me.clone());
    scope.push_constant("yo", me.clone()); // old name
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
        run(&mut doc, id, "set_text(\"Subtitle\", \"Hello \" + 3);\nhide(\"Moon\");\nset_color(me, \"#46BE78\");\nmove_by(me, 10, 5.5);\nmessage(read_text(\"Subtitle\"));\nshow(\"DoesNotExist\");", &mut log);
        let get = |n: &str| doc.elements.iter().find(|e| e.name == n).unwrap().clone();
        assert_eq!(get("Subtitle").text, "Hello 3");
        assert!(!get("Moon").visible);
        assert_eq!(get("Play button").fill, [0x46, 0xBE, 0x78, 255]);
        assert_eq!(log[0], "Hello 3");
        assert!(log[1].contains("DoesNotExist"), "{:?}", log);
    }

    #[test]
    fn move_carries_children_and_value_respects_dropdowns() {
        let mut doc = model::template(2, 1920.0, 1080.0);
        let get = |d: &Doc, n: &str| d.elements.iter().find(|e| e.name == n).unwrap().clone();
        let (v0, t0) = (get(&doc, "Window"), get(&doc, "Window title"));
        let mut log = Vec::new();
        run(&mut doc, v0.id, "move_by(\"Window\", 100, -20);\nmove_by(\"Window\", 1.0/0.0, 0);", &mut log);
        let (v1, t1) = (get(&doc, "Window"), get(&doc, "Window title"));
        assert!((v1.x - v0.x - 100.0).abs() < 0.01 && (v1.y - v0.y + 20.0).abs() < 0.01, "{:?}", log);
        assert!((t1.x - t0.x - 100.0).abs() < 0.01 && (t1.y - t0.y + 20.0).abs() < 0.01, "the child follows the parent");
        assert!(doc.elements.iter().all(|e| e.x.is_finite() && e.y.is_finite()));

        let mut doc = model::template(3, 1920.0, 1080.0);
        let id = get(&doc, "Quality").id;
        run(&mut doc, id, "set_value(\"Quality\", 3); set_value(\"Slider Brightness\", 7);", &mut log);
        assert_eq!(get(&doc, "Quality").value, 3.0);
        assert_eq!(get(&doc, "Slider Brightness").value, 1.0);
        run(&mut doc, id, "set_value(\"Quality\", 99);", &mut log);
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

    #[test]
    fn old_spanish_names_still_work() {
        // Scripts saved in old designs and plugins use the Spanish function names and `yo`.
        let mut doc = model::template(2, 1920.0, 1080.0);
        let get = |d: &Doc, n: &str| d.elements.iter().find(|e| e.name == n).unwrap().clone();
        let w0 = get(&doc, "Window");
        let mut log = Vec::new();
        run(
            &mut doc,
            w0.id,
            "ocultar(\"Dim\"); mostrar(\"Dim\"); alternar(\"Dim\"); texto(\"Window title\", \"Hola\"); mensaje(leer_texto(\"Window title\"));\
             mover(\"Window\", 10, 0); posicion(\"Window\", 100, 100); girar(\"Window\", 90); valor(\"Window\", 1); transparencia(\"Window\", 0.5);\
             color(yo, \"#46BE78\"); color_texto(\"Window title\", \"#FFFFFF\"); mensaje(yo + leer_valor(\"Window\"));",
            &mut log,
        );
        assert!(!get(&doc, "Dim").visible, "alternar hides it again: {:?}", log);
        assert_eq!(get(&doc, "Window title").text, "Hola");
        assert_eq!(get(&doc, "Window").fill, [0x46, 0xBE, 0x78, 255]);
        assert_eq!(get(&doc, "Window").opacity, 0.5);
        assert_eq!(log[0], "Hola");
        assert!(log.iter().all(|l| !l.starts_with('⚠')), "{:?}", log);
        // the new names also read the old constant
        run(&mut doc, w0.id, "message(yo + me);", &mut log);
        assert!(log.iter().all(|l| !l.starts_with('⚠')), "{:?}", log);
    }

    #[test]
    fn bundled_sample_designs_load_and_their_click_code_runs() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(root.join("disenos")).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.to_string_lossy().ends_with(".editorgui.json")).collect();
        files.push(root.join("plugins/extra-templates/shop.editorgui.json"));
        assert!(files.len() >= 7, "{:?}", files);
        for f in files {
            let mut doc: Doc = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap_or_else(|e| panic!("{}: {}", f.display(), e));
            doc.migrate_groups();
            let with_code: Vec<(u64, String)> = doc.elements.iter().filter(|e| !e.on_click.is_empty()).map(|e| (e.id, e.on_click.clone())).collect();
            for (id, code) in with_code {
                let mut log = Vec::new();
                run(&mut doc, id, &code, &mut log);
                assert!(log.iter().all(|l| !l.starts_with('⚠')), "{}: {:?}", f.display(), log);
            }
            // images referenced by the sample exist next to it
            for e in &doc.elements {
                if !e.image.is_empty() {
                    assert!(f.parent().unwrap().join(&e.image).exists(), "{}: missing image {}", f.display(), e.image);
                }
            }
        }
    }
}
