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
/// Shared, bounded output of a script (see [`OutLog`]).
pub type Log = Rc<RefCell<OutLog>>;

/// Most lines and bytes a script may print (`print`, `debug`, `message`, plugin `log`). Past either limit the
/// script is stopped: without this cap a loop of `print` calls could allocate until the process dies.
pub const MAX_LOG_LINES: usize = 200;
pub const MAX_LOG_BYTES: usize = 64 * 1024;
/// A single message is cut to this many characters (cutting is not an error; the totals above are).
const MAX_LINE_CHARS: usize = 1000;

/// Bounded list of messages produced by a script. `overflow` turns true on the first message that does not fit,
/// and `harden` stops the script at its next operation.
#[derive(Default)]
pub struct OutLog {
    pub lines: Vec<String>,
    bytes: usize,
    pub overflow: bool,
}

impl OutLog {
    pub fn new_shared() -> Log {
        Rc::new(RefCell::new(OutLog::default()))
    }

    /// Adds a message; flags the overflow when a limit is exceeded.
    pub fn push(&mut self, s: impl Into<String>) {
        self.try_push(s);
    }

    /// Like `push`, returns false when the message did not fit.
    pub fn try_push(&mut self, s: impl Into<String>) -> bool {
        if self.overflow {
            return false;
        }
        let mut s: String = s.into();
        if s.chars().count() > MAX_LINE_CHARS {
            s = s.chars().take(MAX_LINE_CHARS).collect::<String>() + "…";
        }
        if self.lines.len() >= MAX_LOG_LINES || self.bytes + s.len() > MAX_LOG_BYTES {
            self.overflow = true;
            return false;
        }
        self.bytes += s.len();
        self.lines.push(s);
        true
    }

    /// Adds a line ignoring the limits (the app's own error reports; at most a handful per run).
    pub fn push_forced(&mut self, s: String) {
        self.lines.push(s);
    }
}

/// Joins messages into one status-bar string of at most `max_chars` characters.
pub fn bounded_join(msgs: &[String], sep: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for (i, m) in msgs.iter().enumerate() {
        if i > 0 {
            out.push_str(sep);
        }
        out.push_str(m);
        if out.chars().count() > max_chars {
            let mut cut: String = out.chars().take(max_chars).collect();
            cut.push('…');
            return cut;
        }
    }
    out
}

fn with_el(doc: &Shared, log: &Log, name: &str, f: impl FnOnce(&mut Element)) {
    let mut d = doc.borrow_mut();
    match d.elements.iter_mut().find(|e| e.name == name) {
        Some(e) => f(e),
        None => log.borrow_mut().push(format!("⚠ There is no element named “{}”", name)),
    }
}

/// Hard memory ceiling for a script run: a global allocation counter that only counts on the thread that holds a [`mem_budget::Guard`]
/// (the thread running a Rhai script). `on_progress` reads it and stops the script when the net growth passes the limit. Rhai's own size limits
/// (strings, arrays, maps) already bound each value; this is the backstop for everything else (many variables, deep calls, long garbage chains).
/// It is NOT a sandbox for a single huge allocation: the check runs between script operations, so one operation can still overshoot by its own size
/// (which Rhai's size limits cap). No job object / OS limit is involved, so it works the same on every platform.
pub mod mem_budget {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;

    /// Net bytes a script run may hold above where it started.
    pub const SCRIPT_MEMORY_LIMIT: i64 = 256 * 1024 * 1024;

    // `const` initializers and no destructors: the slots need no lazy set-up, so touching them from inside the allocator can never allocate.
    thread_local! {
        static ON: Cell<bool> = const { Cell::new(false) };
        static LIVE: Cell<i64> = const { Cell::new(0) };
        static CEIL: Cell<i64> = const { Cell::new(i64::MAX) };
    }

    /// Accounts `delta` bytes (positive: allocated, negative: freed) on the current thread when a guard is active.
    #[inline]
    pub fn note(delta: i64) {
        let _ = ON.try_with(|on| {
            if on.get() {
                let _ = LIVE.try_with(|l| l.set(l.get().saturating_add(delta)));
            }
        });
    }

    /// True while a guard is active on this thread and the run holds more than its limit.
    pub fn exceeded() -> bool {
        ON.try_with(|on| on.get()).unwrap_or(false) && LIVE.try_with(|l| l.get()).unwrap_or(0) > CEIL.try_with(|c| c.get()).unwrap_or(i64::MAX)
    }

    /// Active for as long as it lives (one script run). Not re-entrant: a new guard starts a new count.
    pub struct Guard;

    impl Guard {
        pub fn begin(limit: i64) -> Guard {
            LIVE.with(|l| l.set(0));
            CEIL.with(|c| c.set(limit));
            ON.with(|o| o.set(true));
            Guard
        }
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            ON.with(|o| o.set(false));
        }
    }

    /// The program's allocator: `System`, plus the accounting above (one thread-local read when no script is running).
    pub struct BudgetAlloc;

    unsafe impl GlobalAlloc for BudgetAlloc {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            note(l.size() as i64);
            unsafe { System.alloc(l) }
        }
        unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
            note(l.size() as i64);
            unsafe { System.alloc_zeroed(l) }
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            note(-(l.size() as i64));
            unsafe { System.dealloc(p, l) }
        }
        unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
            note(new as i64 - l.size() as i64);
            unsafe { System.realloc(p, l, new) }
        }
    }
}

#[cfg(not(test))]
#[global_allocator]
static BUDGET_ALLOC: mem_budget::BudgetAlloc = mem_budget::BudgetAlloc;

/// Sandbox limits for a Rhai engine: no `import` (no file or network module resolver), bounded memory
/// (strings, arrays/blobs, maps), bounded recursion and expression depth, bounded operations and a
/// wall-clock deadline (`on_progress`) so no script can hang or exhaust memory.
#[allow(dead_code)] // kept for callers that do not collect output
pub fn harden(engine: &mut Engine, max_ops: u64, max_string: usize, max_items: usize, deadline_ms: u64) {
    harden_with_log(engine, max_ops, max_string, max_items, deadline_ms, None);
}

/// Like [`harden`], and also stops the script as soon as `log` overflows (too much output).
/// Rhai counts nested arrays/maps/strings in its size limits, so `a.push(a.clone())` style growth is cut by `max_items`/`max_string`.
pub fn harden_with_log(engine: &mut Engine, max_ops: u64, max_string: usize, max_items: usize, deadline_ms: u64, log: Option<Log>) {
    engine.set_module_resolver(rhai::module_resolvers::DummyModuleResolver::new());
    engine.set_max_operations(max_ops);
    engine.set_max_string_size(max_string);
    engine.set_max_array_size(max_items); // also limits BLOBs
    engine.set_max_map_size(max_items);
    engine.set_max_call_levels(32);
    engine.set_max_expr_depths(64, 32);
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(deadline_ms);
    engine.on_progress(move |_| {
        if std::time::Instant::now() > deadline {
            Some(Dynamic::from("time limit exceeded"))
        } else if mem_budget::exceeded() {
            Some(Dynamic::from("memory limit exceeded"))
        } else if log.as_ref().is_some_and(|l| l.try_borrow().map(|l| l.overflow).unwrap_or(false)) {
            Some(Dynamic::from("too much output (message limit exceeded)"))
        } else {
            None
        }
    });
}

/// Stack of the thread that runs a Rhai script and post-processes what it returns. Dropping, cloning or serializing a
/// deeply nested value (`let a=[]; for i in 0..50000 { a=[a]; }`) recurses once per level and would overflow the
/// 8 MB (Windows: 1 MB) UI-thread stack and kill the process. Only address space is reserved; pages are committed when used.
pub const SCRIPT_STACK_BYTES: usize = 768 * 1024 * 1024;

/// Deepest nesting of arrays/maps a script may return as data (a design is about 6 levels deep).
pub const MAX_RETURN_DEPTH: usize = 64;

/// Runs `f` on a dedicated thread with a big stack and waits for it. A panic inside becomes an `Err` (it never reaches the UI thread).
/// Scoped, so `f` may borrow from the caller. Everything that touches Rhai values must happen INSIDE `f` (they are not `Send`).
pub fn on_big_stack<T: Send>(f: impl FnOnce() -> T + Send) -> Result<T, String> {
    std::thread::scope(|s| {
        let handle = std::thread::Builder::new().name("lienzo-script".into()).stack_size(SCRIPT_STACK_BYTES).spawn_scoped(s, f).map_err(|e| format!("could not start the script thread: {}", e))?;
        handle.join().map_err(|_| "the script stopped unexpectedly (internal error)".to_string())
    })
}

/// Converts a value returned by a script to JSON, refusing anything nested deeper than [`MAX_RETURN_DEPTH`] or that is not plain data
/// (the recursion is bounded by that depth, so a hostile nesting cannot overflow the stack here).
pub fn dynamic_to_json(d: &Dynamic) -> Result<serde_json::Value, String> {
    fn go(d: &Dynamic, depth: usize) -> Result<serde_json::Value, String> {
        use serde_json::Value;
        if d.is_array() || d.is_map() {
            if depth >= MAX_RETURN_DEPTH {
                return Err(format!("the returned data is nested too deeply (more than {} levels)", MAX_RETURN_DEPTH));
            }
            if let Some(a) = d.read_lock::<rhai::Array>() {
                return a.iter().map(|x| go(x, depth + 1)).collect::<Result<Vec<_>, _>>().map(Value::Array);
            }
            if let Some(m) = d.read_lock::<rhai::Map>() {
                return m.iter().map(|(k, v)| go(v, depth + 1).map(|v| (k.to_string(), v))).collect::<Result<serde_json::Map<_, _>, _>>().map(Value::Object);
            }
            return Err("unsupported value".into());
        }
        if d.is_unit() {
            Ok(Value::Null)
        } else if let Ok(b) = d.as_bool() {
            Ok(Value::Bool(b))
        } else if let Ok(i) = d.as_int() {
            Ok(Value::from(i))
        } else if let Ok(f) = d.as_float() {
            Ok(serde_json::Number::from_f64(f).map(Value::Number).unwrap_or(Value::Null))
        } else if let Ok(c) = d.as_char() {
            Ok(Value::String(c.to_string()))
        } else if d.is_string() {
            Ok(Value::String(d.clone().into_string().unwrap_or_default()))
        } else {
            Err(format!("unsupported value of type {}", d.type_name()))
        }
    }
    go(d, 0)
}

/// Run `code` as if element `id` was clicked. Changes are applied to `doc`;
/// messages and errors are appended to `log`. The script runs on its own big-stack thread (see [`on_big_stack`]).
pub fn run(doc: &mut Doc, id: u64, code: &str, log: &mut Vec<String>) {
    let mut out_lines = Vec::new();
    let mut work = doc.clone();
    match on_big_stack(move || {
        run_inner(&mut work, id, code, &mut out_lines);
        (work, out_lines)
    }) {
        Ok((d, lines)) => {
            *doc = d;
            log.extend(lines);
        }
        Err(e) => log.push(format!("⚠ Error in the code of an element: {}", e)),
    }
}

fn run_inner(doc: &mut Doc, id: u64, code: &str, log: &mut Vec<String>) {
    let me = doc.get(id).map(|e| e.name.clone()).unwrap_or_default();
    let shared: Shared = Rc::new(RefCell::new(doc.clone()));
    let out: Log = OutLog::new_shared();
    let mut engine = Engine::new();
    harden_with_log(&mut engine, 500_000, 64 * 1024, 10_000, 2000, Some(out.clone()));
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

    let _memory = mem_budget::Guard::begin(mem_budget::SCRIPT_MEMORY_LIMIT);
    let mut scope = Scope::new();
    scope.push_constant("me", me.clone());
    scope.push_constant("yo", me.clone()); // old name
    let result = engine.run_with_scope(&mut scope, code);
    drop(engine);
    {
        let mut o = out.borrow_mut();
        match result {
            Err(err) => o.push_forced(format!("⚠ Error in the code of “{}”: {}", me, err)),
            Ok(()) if o.overflow => o.push_forced(format!("⚠ Error in the code of “{}”: too much output (message limit exceeded)", me)),
            Ok(()) => {}
        }
    }
    *doc = shared.borrow().clone();
    log.extend(out.borrow().lines.iter().cloned());
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
    fn deeply_nested_data_does_not_overflow_the_stack() {
        // PoC: 50000 levels of nesting dropped at the end of the script used to abort the whole process.
        let mut doc = model::template(0, 1920.0, 1080.0);
        let mut log = Vec::new();
        run(&mut doc, 1, "let a=[]; for i in 0..50000 { a=[a]; }", &mut log);
        run(&mut doc, 1, "let a=#{}; for i in 0..100000 { a=#{x: a}; } message(\"done\");", &mut log);
        assert!(log.iter().all(|l| !l.contains("internal error")), "{:?}", log);
    }

    #[test]
    fn returned_data_nesting_is_bounded() {
        let e = Engine::new();
        let deep: Dynamic = e.eval("let a=[]; for i in 0..200 { a=[a]; } a").unwrap();
        assert!(dynamic_to_json(&deep).is_err());
        let ok: Dynamic = e.eval("#{a: [1, 2.5, \"x\", true, ()], b: #{c: 'z'}}").unwrap();
        let v = dynamic_to_json(&ok).unwrap();
        assert_eq!(v["a"][1], 2.5);
        assert_eq!(v["b"]["c"], "z");
    }

    #[test]
    fn a_panic_in_the_script_thread_is_an_error_not_a_crash() {
        assert!(on_big_stack(|| -> u8 { panic!("boom") }).is_err());
        assert_eq!(on_big_stack(|| 7).unwrap(), 7);
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
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(root.join("designs")).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.to_string_lossy().ends_with(".editorgui.json")).collect();
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

#[cfg(test)]
mod sandbox_tests {
    use super::*;
    use crate::model;
    use std::time::{Duration, Instant};

    fn err_of(code: &str) -> String {
        let mut doc = model::template(0, 1920.0, 1080.0);
        let id = doc.elements[0].id;
        let mut log = Vec::new();
        let t = Instant::now();
        run(&mut doc, id, code, &mut log);
        assert!(t.elapsed() < Duration::from_secs(10), "too slow: {:?} for {}", t.elapsed(), code);
        log.iter().find(|l| l.contains("Error in the code")).cloned().unwrap_or_else(|| panic!("no error for {}: {:?}", code, log))
    }

    #[test]
    fn hostile_scripts_fail_fast() {
        for code in [
            "let a = []; a.pad(300000000, 0);",
            "let b = blob(1000000000);",
            "let m = #{}; for i in 0..100000 { m[i.to_string()] = i; }",
            "let s = \"x\"; s.pad(100000000, 'a');",
            "import \"C:\\x\\m\" as m;",
            "import \"\\\\server\\share\\m\" as m;",
            "loop { }",
            "fn f(n) { f(n + 1) } f(0);",
        ] {
            let e = err_of(code);
            assert!(!e.is_empty());
        }
    }

    #[test]
    fn normal_scripts_still_work() {
        let mut doc = model::template(0, 1920.0, 1080.0);
        let id = doc.elements[0].id;
        let mut log = Vec::new();
        run(&mut doc, id, "let a = []; for i in 0..1000 { a.push(i); } message(a.len());", &mut log);
        assert_eq!(log, vec!["1000".to_string()]);
    }
}

/// Test-only allocation counter (per thread), to assert peak memory without measuring the process RSS.
#[cfg(test)]
pub mod alloc_probe {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::cell::Cell;

    pub struct Counting;

    thread_local! {
        static CUR: Cell<i64> = const { Cell::new(0) };
        static PEAK: Cell<i64> = const { Cell::new(0) };
    }

    unsafe impl GlobalAlloc for Counting {
        unsafe fn alloc(&self, l: Layout) -> *mut u8 {
            super::mem_budget::note(l.size() as i64);
            let _ = CUR.try_with(|c| {
                c.set(c.get() + l.size() as i64);
                let _ = PEAK.try_with(|p| p.set(p.get().max(c.get())));
            });
            unsafe { System.alloc(l) }
        }
        unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
            super::mem_budget::note(-(l.size() as i64));
            let _ = CUR.try_with(|c| c.set(c.get() - l.size() as i64));
            unsafe { System.dealloc(p, l) }
        }
        unsafe fn realloc(&self, p: *mut u8, l: Layout, new: usize) -> *mut u8 {
            super::mem_budget::note(new as i64 - l.size() as i64);
            let _ = CUR.try_with(|c| {
                c.set(c.get() + new as i64 - l.size() as i64);
                let _ = PEAK.try_with(|p| p.set(p.get().max(c.get())));
            });
            unsafe { System.realloc(p, l, new) }
        }
    }

    /// Runs `f` and returns the most memory (bytes) this thread held above the starting point.
    pub fn peak_of<R>(f: impl FnOnce() -> R) -> (R, i64) {
        let start = CUR.with(|c| c.get());
        PEAK.with(|p| p.set(start));
        let r = f();
        (r, PEAK.with(|p| p.get()) - start)
    }
}

#[cfg(test)]
#[global_allocator]
static COUNTING: alloc_probe::Counting = alloc_probe::Counting;

#[cfg(test)]
mod flood_tests {
    use super::*;
    use crate::model;
    use std::time::{Duration, Instant};

    const MB: i64 = 1024 * 1024;

    fn run_measured(code: &str) -> (Vec<String>, i64, Duration) {
        let mut doc = model::template(0, 1920.0, 1080.0);
        let id = doc.elements[0].id;
        let mut log = Vec::new();
        let t = Instant::now();
        let (_, peak) = alloc_probe::peak_of(|| run(&mut doc, id, code, &mut log));
        (log, peak, t.elapsed())
    }

    #[test]
    fn print_flood_is_cut_by_message_count_and_bytes() {
        for code in ["loop { print(\"x\"); }", "loop { message(\"hello\"); }", "loop { debug(\"d\"); }", "loop { show(\"Nope\"); }", "let s = \"y\".repeat(60000); loop { print(s); }"] {
            let (log, peak, t) = run_measured(code);
            assert!(log.len() <= MAX_LOG_LINES + 1, "{}: {} lines", code, log.len());
            assert!(log.iter().map(|l| l.len()).sum::<usize>() <= MAX_LOG_BYTES + 400, "{}", code);
            assert!(log.last().is_some_and(|l| l.contains("Error in the code")), "{}: {:?}", code, log.last());
            assert!(peak < 8 * MB, "{}: peak {} bytes", code, peak);
            assert!(t < Duration::from_secs(5), "{}: {:?}", code, t);
        }
    }

    #[test]
    fn nested_clone_growth_and_huge_strings_fail_fast_with_small_peak() {
        for code in [
            "let a = []; loop { a.push(a.clone()); }",
            "let a = [0]; loop { a += a; }",
            "let m = #{}; loop { m.x = m.clone(); }",
            "let s = \"abc\"; loop { s += s; }",
            "let s = \"abc\"; loop { s = s + s; }",
            "let a = []; let s = \"x\".repeat(60000); loop { a.push(s); }",
            "let a = []; let s = \"x\".repeat(60000); for i in 0..9000 { a.push(s + i); }",
        ] {
            let (log, peak, t) = run_measured(code);
            assert!(log.iter().any(|l| l.contains("Error in the code")), "{}: {:?}", code, log);
            assert!(peak < 64 * MB, "{}: peak {} bytes", code, peak);
            assert!(t < Duration::from_secs(5), "{}: {:?}", code, t);
        }
    }

    #[test]
    fn memory_budget_stops_a_script_that_holds_too_much() {
        // 9000 integers held in a list are far below Rhai's own limits but above a 50 KB budget.
        let mut engine = Engine::new();
        harden_with_log(&mut engine, 5_000_000, 64 * 1024, 10_000, 5000, None);
        let guard = mem_budget::Guard::begin(50 * 1024);
        let r = engine.run("let a = []; for i in 0..9000 { a.push(i); }");
        drop(guard);
        match *r.unwrap_err() {
            rhai::EvalAltResult::ErrorTerminated(token, _) => assert_eq!(token.into_string().unwrap(), "memory limit exceeded"),
            other => panic!("expected the memory limit, got {:?}", other),
        }
        // after the guard is gone nothing is counted or stopped
        assert!(!mem_budget::exceeded());
        assert!(engine.run("let a = []; for i in 0..9000 { a.push(i); }").is_ok());
        // a normal script under the real limit is untouched
        let _g = mem_budget::Guard::begin(mem_budget::SCRIPT_MEMORY_LIMIT);
        assert!(engine.run("let a = []; for i in 0..9000 { a.push(i); }").is_ok());
        assert!(!mem_budget::exceeded());
    }

    #[test]
    fn memory_budget_counts_net_growth_not_garbage() {
        let mut engine = Engine::new();
        harden_with_log(&mut engine, 5_000_000, 64 * 1024, 10_000, 5000, None);
        let _g = mem_budget::Guard::begin(2 * 1024 * 1024);
        // thousands of short-lived 60 KB strings (hundreds of MB of churn) never hold more than a couple of them at a time
        let r = engine.run("for i in 0..3000 { let s = \"x\"; for j in 0..15 { s += s; } }");
        assert!(r.is_ok(), "{:?}", r);
    }

    #[test]
    fn bounded_join_cuts_long_status_strings() {
        let msgs: Vec<String> = (0..500).map(|i| format!("message number {}", i)).collect();
        let s = bounded_join(&msgs, " · ", 300);
        assert!(s.chars().count() <= 301, "{}", s.len());
        assert_eq!(bounded_join(&["a".into(), "b".into()], "-", 300), "a-b");
    }
}
