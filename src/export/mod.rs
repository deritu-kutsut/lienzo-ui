//! Export to Unity: uGUI (JSON + editor importer that builds a prefab), UI Toolkit
//! (UXML + USS), IMGUI (an `OnGUI` MonoBehaviour) and a "View" script with
//! references to the named elements.

pub mod imgui;
pub mod ugui;
pub mod uitk;
pub mod view;

#[cfg(test)]
mod ext_tests;

use crate::model::{Doc, Element, Frame, Kind, rect_transform_in, RectTransform};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Unity editor script (goes in Assets/EditorGUI/Editor/).
pub const IMPORTER_CS: &str = include_str!("EditorGuiImporter.cs");

/// Runtime helpers written next to the generated scripts when a design uses the safe area / a portrait layout.
pub const SAFE_AREA_CS: &str = include_str!("LienzoSafeArea.cs");
pub const ORIENTATION_CS: &str = include_str!("LienzoOrientationLayout.cs");

/// Folder inside the Unity project.
pub const UNITY_ROOT: &str = "Assets/EditorGUI";

/// Default folder of the generated scripts (`unity.scripts_dir` overrides it).
pub const DEFAULT_SCRIPTS_DIR: &str = "Assets/EditorGUI/Scripts";

/// Names of the GameObjects the importer generates itself. They start with `__` so they cannot collide
/// with a name the user chose (the old Spanish names `Borde`, `Texto`, `Fondo del lienzo`, `Tick` could).
pub const GEN_SAFE_AREA: &str = "__safe_area";
pub const GEN_NAMES: [&str; 5] = ["__border", "__text", "__tick", "__canvas_background", GEN_SAFE_AREA];

/// ASCII-only version of a name (accents removed).
pub fn ascii_fold(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'Á' | 'À' | 'Ä' | 'Â' => 'A',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'É' | 'È' | 'Ë' | 'Ê' => 'E',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'Í' | 'Ì' | 'Ï' | 'Î' => 'I',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'Ó' | 'Ò' | 'Ö' | 'Ô' => 'O',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'Ú' | 'Ù' | 'Ü' | 'Û' => 'U',
            'ñ' => 'n',
            'Ñ' => 'N',
            'ç' => 'c',
            'Ç' => 'C',
            c => c,
        })
        .collect()
}

fn words(s: &str) -> Vec<String> {
    ascii_fold(s).split(|c: char| !c.is_ascii_alphanumeric()).filter(|w| !w.is_empty()).map(|w| w.to_string()).collect()
}

/// PascalCase C# identifier ("Main menu" → "MainMenu").
pub fn class_name(s: &str) -> String {
    let mut out: String = words(s)
        .iter()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
                None => String::new(),
            }
        })
        .collect();
    if out.is_empty() {
        out = "Design".into();
    }
    if out.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        out = format!("Ui{}", out);
    }
    // Used as file and class name: a very long title must not exceed the Windows path limit.
    let out: String = out.chars().take(64).collect();
    // CON, PRN, AUX, NUL, COM1… and LPT1… cannot be file names on Windows.
    let up = out.to_ascii_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL"].contains(&up.as_str())
        || ((up.starts_with("COM") || up.starts_with("LPT")) && up.len() == 4 && up.as_bytes()[3].is_ascii_digit());
    if reserved { format!("{}Ui", out) } else { out }
}

/// camelCase C# identifier.
#[cfg_attr(not(test), allow(dead_code))]
pub fn ident(s: &str) -> String {
    let p = class_name(s);
    let mut c = p.chars();
    match c.next() {
        Some(f) => f.to_ascii_lowercase().to_string() + c.as_str(),
        None => "element".into(),
    }
}

/// Field name that does not clash with C# keywords or MonoBehaviour members.
pub fn safe_field(s: &str) -> String {
    const BAD: &[&str] = &[
        "abstract", "as", "base", "bool", "break", "byte", "case", "catch", "char", "checked", "class", "const", "continue", "decimal", "default", "delegate", "do", "double", "else", "enum", "event",
        "explicit", "extern", "false", "finally", "fixed", "float", "for", "foreach", "goto", "if", "implicit", "in", "int", "interface", "internal", "is", "lock", "long", "namespace", "new", "null",
        "object", "operator", "out", "override", "params", "private", "protected", "public", "readonly", "ref", "return", "sbyte", "sealed", "short", "sizeof", "stackalloc", "static", "string",
        "struct", "switch", "this", "throw", "true", "try", "typeof", "uint", "ulong", "unchecked", "unsafe", "ushort", "using", "virtual", "void", "volatile", "while", "name", "tag", "transform",
        "gameObject", "enabled", "hideFlags", "useGUILayout", "runInEditMode", "destroyCancellationToken", "didStart", "didAwake",
    ];
    if BAD.contains(&s) { format!("{}Ui", s) } else { s.to_string() }
}

/// True for every character that C# (or XML/USS tooling) treats as a line break.
pub fn is_line_break(c: char) -> bool {
    matches!(c, '\n' | '\r' | '\u{0B}' | '\u{0C}' | '\u{85}' | '\u{2028}' | '\u{2029}')
}

/// Text that is safe inside a one-line comment (`//`, `///`): every line break, control
/// character and bidi override becomes a space, so user text can never end the comment.
pub fn sanitize_line(s: &str) -> String {
    s.chars()
        .map(|c| if is_line_break(c) || c.is_control() || matches!(c, '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}') { ' ' } else { c })
        .collect()
}

/// Splits text into lines using every kind of line break (not just `\n`), each one made comment-safe.
pub fn comment_lines(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut prev_cr = false;
    for c in s.chars() {
        if is_line_break(c) {
            if !(c == '\n' && prev_cr) {
                out.push(sanitize_line(&cur));
                cur.clear();
            }
            prev_cr = c == '\r';
        } else {
            prev_cr = false;
            cur.push(c);
        }
    }
    out.push(sanitize_line(&cur));
    if out.last().is_some_and(|l| l.is_empty()) {
        out.pop();
    }
    out
}

/// C# string literal. Escapes line breaks (including U+2028, U+2029, U+0085 and `\r`) and control characters.
pub fn cs_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => o.push_str("\\\\"),
            '"' => o.push_str("\\\""),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if c.is_control() || matches!(c, '\u{2028}' | '\u{2029}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}') => {
                let mut u = [0u16; 2];
                for unit in c.encode_utf16(&mut u) {
                    o.push_str(&format!("\\u{:04X}", unit));
                }
            }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// Text for an XML comment: no `--` (invalid inside comments), no line breaks, and it cannot end with `-`.
pub fn xml_comment(s: &str) -> String {
    let mut t = sanitize_line(s);
    while t.contains("--") {
        t = t.replace("--", "- ");
    }
    t.trim_end_matches('-').to_string()
}

/// Text for a CSS/USS `/* */` comment: `*/` removed until none is left (`**//` would otherwise rebuild one).
pub fn css_comment(s: &str) -> String {
    let mut t = sanitize_line(s);
    while t.contains("*/") {
        t = t.replace("*/", "");
    }
    t
}

/// Piece of a quoted USS `url("...")`: no quotes, backslashes, parentheses or line breaks.
pub fn css_url_part(s: &str) -> String {
    s.chars().filter(|c| !matches!(c, '"' | '\\' | '(' | ')') && !is_line_break(*c) && !c.is_control()).collect()
}

/// XML attribute value.
pub fn xml_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;").replace('\n', "&#10;").replace('\r', "&#13;").replace('\t', "&#9;")
}

pub fn f(v: f32) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r == r.trunc() { format!("{}", r as i64) } else { format!("{}", r) }
}

/// C# float literal.
pub fn cf(v: f32) -> String {
    format!("{}f", f(v))
}

//// Name an element would get before de-duplication: its valid `unity_name` verbatim, else its display name
/// (never one of the names the importer generates for its own helper objects: those get a trailing `_`).
pub fn base_unity_name(e: &Element) -> String {
    if let Some(u) = e.unity_name.as_deref().filter(|u| crate::docext::valid_unity_name(u)) {
        return u.to_string();
    }
    let n = e.name.replace('/', "-").trim().to_string();
    if n.is_empty() {
        e.kind.label().to_string()
    } else if crate::docext::is_generated_name(&n) {
        format!("{}_", n)
    } else {
        n
    }
}

/// Unique GameObject names among siblings (Unity's `Transform.Find` uses names; '/' is not allowed).
/// A valid `unity_name` is used verbatim; it is only suffixed when it collides with a sibling that
/// claimed the same name first (see [`unity_name_collisions`]). Explicit names are assigned before
/// automatic ones, so a generated name never takes an explicit name away.
pub fn unity_names(doc: &Doc) -> HashMap<u64, String> {
    unity_names_report(doc).0
}

#[allow(dead_code)] // used by the exporters built on top of the model
/// Ids of elements with an explicit `unity_name` that had to be renamed because a sibling already uses it.
pub fn unity_name_collisions(doc: &Doc) -> Vec<u64> {
    unity_names_report(doc).1
}

fn unity_names_report(doc: &Doc) -> (HashMap<u64, String>, Vec<u64>) {
    let mut out = HashMap::with_capacity(doc.elements.len());
    let mut clashes = Vec::new();
    let idx = doc.index_map();
    let par = doc.parent_indices(&idx);
    let mut used: HashMap<Option<usize>, HashSet<String>> = HashMap::new();
    // next suffix to try per (parent, base name): names already taken stay taken, so the search never restarts at 2
    let mut next_k: HashMap<(Option<usize>, String), usize> = HashMap::new();
    let explicit = |e: &Element| e.unity_name.as_deref().is_some_and(crate::docext::valid_unity_name);
    for pass in [true, false] {
        for (i, e) in doc.elements.iter().enumerate().filter(|(_, e)| explicit(e) == pass) {
            let base = base_unity_name(e);
            let key = par[i];
            let set = used.entry(key).or_default();
            let mut name = base.clone();
            if set.contains(&name) {
                let k = next_k.entry((key, base.clone())).or_insert(2);
                loop {
                    name = format!("{} ({})", base, *k);
                    *k += 1;
                    if !set.contains(&name) {
                        break;
                    }
                }
            }
            if pass && name != base {
                clashes.push(e.id);
            }
            set.insert(name.clone());
            out.insert(e.id, name);
        }
    }
    (out, clashes)
}

/// Path from the Canvas (`transform.Find` style). Children of a ScrollView live in "Viewport/Content".
pub fn unity_path(doc: &Doc, names: &HashMap<u64, String>, id: u64) -> String {
    let mut parts = vec![names.get(&id).cloned().unwrap_or_default()];
    let mut cur = id;
    let mut guard = 0usize;
    while let Some(p) = doc.parent_of(cur) {
        guard += 1;
        if guard > doc.elements.len() {
            break;
        }
        if doc.get(p).map(|e| e.kind == Kind::ScrollView).unwrap_or(false) {
            parts.push("Viewport/Content".into());
        }
        parts.push(names.get(&p).cloned().unwrap_or_default());
        cur = p;
    }
    parts.reverse();
    parts.join("/")
}

/// [`unity_path`] of every element at once (iterative, O(total path length)).
pub fn unity_paths(doc: &Doc, names: &HashMap<u64, String>) -> HashMap<u64, String> {
    let idx = doc.index_map();
    let par = doc.parent_indices(&idx);
    let n = doc.elements.len();
    let mut out: Vec<Option<String>> = vec![None; n];
    for s in 0..n {
        let mut chain = Vec::new();
        let mut cur = Some(s);
        while let Some(c) = cur {
            if out[c].is_some() || chain.len() > n {
                break;
            }
            chain.push(c);
            cur = par[c];
        }
        for &c in chain.iter().rev() {
            let el = &doc.elements[c];
            let nm = names.get(&el.id).cloned().unwrap_or_default();
            let path = match par[c] {
                Some(p) => {
                    let pp = out[p].clone().unwrap_or_default();
                    if doc.elements[p].kind == Kind::ScrollView { format!("{}/Viewport/Content/{}", pp, nm) } else { format!("{}/{}", pp, nm) }
                }
                None => nm,
            };
            out[c] = Some(path);
        }
    }
    doc.elements.iter().zip(out).map(|(e, p)| (e.id, p.unwrap_or_default())).collect()
}

/// Grows `bottom` (a ScrollView's content height) so that `child` fits.
fn grow_content(sf: &Frame, bottom: &mut f32, child: &Element) {
    let b = child.aabb();
    for corner in [[b[0], b[1]], [b[2], b[3]]] {
        let l = sf.to_local(corner);
        *bottom = bottom.max(l[1] + sf.h / 2.0 + 20.0);
    }
}

/// Height of a ScrollView's content: tall enough for every child (at least the view).
pub fn content_height(doc: &Doc, sv: &Element) -> f32 {
    let f = sv.frame();
    let mut bottom = f.h;
    for e in doc.elements.iter().filter(|e| e.parent == Some(sv.id) && e.id != sv.id) {
        grow_content(&f, &mut bottom, e);
    }
    bottom.ceil()
}

/// Lookup tables built once per export: id -> position, children, ScrollView content heights. Makes every
/// per-element question O(1) (the plain `Doc` helpers scan the element list).
pub struct ExportCtx<'a> {
    pub doc: &'a Doc,
    idx: HashMap<u64, usize>,
    kids: HashMap<Option<u64>, Vec<u64>>,
    heights: HashMap<u64, f32>,
}

impl<'a> ExportCtx<'a> {
    pub fn new(doc: &'a Doc) -> Self {
        let idx = doc.index_map();
        let mut bottoms: HashMap<u64, f32> = HashMap::new();
        for e in doc.elements.iter().filter(|e| e.kind == Kind::ScrollView) {
            bottoms.entry(e.id).or_insert(e.frame().h);
        }
        for e in &doc.elements {
            let Some(p) = e.parent.and_then(|p| idx.get(&p)).map(|&i| &doc.elements[i]) else { continue };
            if p.kind == Kind::ScrollView && p.id != e.id {
                if let Some(b) = bottoms.get_mut(&p.id) {
                    grow_content(&p.frame(), b, e);
                }
            }
        }
        let heights = bottoms.into_iter().map(|(k, v)| (k, v.ceil())).collect();
        ExportCtx { doc, idx, kids: doc.child_map(), heights }
    }

    pub fn get(&self, id: u64) -> Option<&Element> {
        self.idx.get(&id).map(|&i| &self.doc.elements[i])
    }

    pub fn children(&self, parent: Option<u64>) -> &[u64] {
        self.kids.get(&parent).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// The parent element of `e` (None for roots).
    pub fn parent(&self, e: &Element) -> Option<&Element> {
        e.parent.filter(|p| *p != e.id).and_then(|p| self.get(p))
    }

    pub fn content_height(&self, sv: &Element) -> f32 {
        self.heights.get(&sv.id).copied().unwrap_or_else(|| sv.frame().h.ceil())
    }

    pub fn parent_frame(&self, e: &Element) -> Frame {
        match self.parent(e) {
            Some(p) if p.kind == Kind::ScrollView => scroll_content_frame(p, self.content_height(p)),
            Some(p) => p.frame(),
            None => self.doc.page_frame(),
        }
    }

    pub fn rt(&self, e: &Element) -> RectTransform {
        rect_transform_in(e, &self.parent_frame(e))
    }
}

fn scroll_content_frame(p: &Element, ch: f32) -> Frame {
    let pf = p.frame();
    let c = pf.to_world([0.0, -pf.h / 2.0 + ch / 2.0]);
    Frame { cx: c[0], cy: c[1], w: pf.w, h: ch, rot: pf.rot }
}

/// The box Unity measures an element's RectTransform against: its parent, or the
/// parent's Content when the parent is a ScrollView (top-aligned, full width).
pub fn export_parent_frame(doc: &Doc, e: &Element) -> Frame {
    match doc.parent_of(e.id).and_then(|p| doc.get(p)) {
        Some(p) if p.kind == Kind::ScrollView => scroll_content_frame(p, content_height(doc, p)),
        Some(p) => p.frame(),
        None => doc.page_frame(),
    }
}

pub fn export_rt(doc: &Doc, e: &Element) -> RectTransform {
    rect_transform_in(e, &export_parent_frame(doc, e))
}

// Corner radius usable for an element (never more than half its short side).
pub fn radius_of(e: &Element) -> f32 {
    let max = e.w.min(e.h) / 2.0;
    // With negative or non-numeric width/height (foreign files) `clamp` would panic.
    if max.is_finite() && max > 0.0 && e.radius.is_finite() { e.radius.clamp(0.0, max) } else { 0.0 }
}

/// File name an image gets inside Assets/EditorGUI/Sprites.
pub fn image_file_name(image: &str) -> String {
    Path::new(image).file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| image.to_string())
}

/// `unity:<asset path>[#sub-sprite]` or a plain path under `Assets/` (or `Packages/`): a picture that already
/// lives in the Unity project. It is referenced as it is, never copied or overwritten.
pub fn unity_image_ref(image: &str) -> Option<(String, Option<String>)> {
    let t = image.trim().replace('\\', "/");
    let body = t.strip_prefix("unity:").unwrap_or(&t);
    let (path, sub) = match body.split_once('#') {
        Some((p, s)) => (p.to_string(), Some(s.to_string()).filter(|s| !s.is_empty())),
        None => (body.to_string(), None),
    };
    let ok = (path.starts_with("Assets/") || path.starts_with("Packages/")) && !path.split('/').any(|p| p == "..") && !path.chars().any(|c| c.is_control());
    if ok { Some((path, sub)) } else { None }
}

/// Sub-sprite the element uses: its `sprite`, or the `#name` of a `unity:` reference.
pub fn element_sprite(e: &Element) -> Option<String> {
    e.sprite.clone().or_else(|| unity_image_ref(&e.image).and_then(|(_, s)| s))
}

/// Asset path inside the Unity project of an element's picture: the referenced asset for a `unity:` image,
/// else the copy that the export writes into `Assets/EditorGUI/Sprites`.
pub fn image_asset_path(image: &str) -> String {
    match unity_image_ref(image) {
        Some((p, _)) => p,
        None => format!("{}/Sprites/{}", UNITY_ROOT, image_file_name(image)),
    }
}

/// Fonts used by the design that are not the default one.
pub fn used_fonts(doc: &Doc) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    for e in &doc.elements {
        if !e.font.is_empty() && e.font != "Monospace" && !v.contains(&e.font) {
            v.push(e.font.clone());
        }
    }
    v
}

/// Accept the project folder, its Assets folder, or any folder inside Assets.
pub fn find_unity_root(dir: &Path) -> Option<PathBuf> {
    let mut cur = Some(dir);
    while let Some(d) = cur {
        if d.join("Assets").is_dir() && (d.join("ProjectSettings").is_dir() || d.join("Packages").is_dir()) {
            return Some(d.to_path_buf());
        }
        cur = d.parent();
    }
    if dir.join("Assets").is_dir() {
        return Some(dir.to_path_buf());
    }
    None
}

/// Text of a `.cs` file as Unity reads it: UTF-8 (with or without BOM) or UTF-16 with a BOM; invalid bytes are replaced.
pub fn decode_cs(bytes: &[u8]) -> String {
    if let Some(rest) = bytes.strip_prefix(&[0xFF, 0xFE]) {
        let u: Vec<u16> = rest.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        return String::from_utf16_lossy(&u);
    }
    if let Some(rest) = bytes.strip_prefix(&[0xFE, 0xFF]) {
        let u: Vec<u16> = rest.chunks_exact(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect();
        return String::from_utf16_lossy(&u);
    }
    let b = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    String::from_utf8_lossy(b).into_owned()
}

/// Every `.cs` under `<project>/Assets` (relative paths with `/`) whose text satisfies `pred` (bounded scan).
/// Files are read as bytes and decoded leniently (see [`decode_cs`]): a Latin-1 or UTF-16 file is still scanned.
fn scan_cs(root: &Path, mut pred: impl FnMut(&str) -> bool) -> Vec<String> {
    let mut stack = vec![root.join("Assets")];
    let mut found = Vec::new();
    let (mut files, mut dirs) = (0usize, 0usize);
    while let Some(d) = stack.pop() {
        dirs += 1;
        if dirs > 20_000 {
            break;
        }
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for ent in rd.flatten() {
            let p = ent.path();
            let Ok(ft) = ent.file_type() else { continue };
            if ft.is_dir() {
                stack.push(p);
            } else if ft.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("cs")) {
                files += 1;
                if files > 50_000 {
                    return found;
                }
                if ent.metadata().map(|m| m.len() > 2_000_000).unwrap_or(true) {
                    continue;
                }
                if let Ok(bytes) = std::fs::read(&p) {
                    if pred(&decode_cs(&bytes)) {
                        if let Ok(r) = p.strip_prefix(root) {
                            found.push(r.to_string_lossy().replace('\\', "/"));
                        }
                    }
                }
            }
        }
    }
    found.sort();
    found
}

/// True if some `.cs` under `<project>/Assets` already declares `class {cls}View` (bounded textual scan).
fn view_class_exists(root: &Path, cls: &str) -> bool {
    let needle = format!("{}View", cls);
    !scan_cs(root, |t| declares_class(t, &needle)).is_empty()
}

// Project folder (relative, starts with `Assets`) of the `.asmdef` named `assembly`, or of an `.asmref` that
/// points to it (a folder with an `.asmref` compiles into that assembly too).
pub fn find_asmdef_folder(root: &Path, assembly: &str) -> Option<String> {
    let mut stack = vec![root.join("Assets")];
    let mut dirs = 0usize;
    let mut via_ref = None;
    while let Some(d) = stack.pop() {
        dirs += 1;
        if dirs > 20_000 {
            break;
        }
        let Ok(rd) = std::fs::read_dir(&d) else { continue };
        for ent in rd.flatten() {
            let p = ent.path();
            let Ok(ft) = ent.file_type() else { continue };
            if ft.is_dir() {
                stack.push(p);
                continue;
            }
            let ext = p.extension().and_then(|x| x.to_str()).unwrap_or("").to_ascii_lowercase();
            if (ext == "asmdef" || ext == "asmref") && ft.is_file() {
                let Ok(t) = std::fs::read_to_string(&p) else { continue };
                let Ok(v) = serde_json::from_str::<serde_json::Value>(t.trim_start_matches('\u{feff}')) else { continue };
                let key = if ext == "asmdef" { "name" } else { "reference" };
                if v[key].as_str() == Some(assembly) {
                    let rel = p.parent().and_then(|d| d.strip_prefix(root).ok()).map(|r| r.to_string_lossy().replace('\\', "/"));
                    if ext == "asmdef" {
                        return rel;
                    }
                    via_ref = via_ref.or(rel);
                }
            }
        }
    }
    via_ref
}

/// Where the generated scripts of `doc` go and a note when the request could not be followed exactly.
///
/// * `unity.scripts_dir` (default `Assets/EditorGUI/Scripts`).
/// * With `unity.assembly`: if an `.asmdef` of that name is found in the project, the scripts must live under its
///   folder to compile into it: `scripts_dir` is kept when it is already inside that folder, otherwise
///   `<asmdef folder>/LienzoUI` is used. If no such asmdef exists the scripts stay in `scripts_dir` (Assembly-CSharp).
pub fn resolve_scripts_dir(doc: &Doc, root: &Path) -> (String, Option<String>) {
    let want = doc.unity.scripts_dir.clone();
    if let Some(asm) = &doc.unity.assembly {
        return match find_asmdef_folder(root, asm) {
            Some(folder) => {
                let inside = |w: &str| w == folder || w.starts_with(&format!("{}/", folder));
                match want {
                    Some(w) if inside(&w) => (w, None),
                    Some(w) => (format!("{}/LienzoUI", folder), Some(format!("scripts_dir {} is outside the folder of assembly {} ({}): the scripts were put in {}/LienzoUI", w, asm, folder, folder))),
                    None => (format!("{}/LienzoUI", folder), None),
                }
            }
            None => (
                want.unwrap_or_else(|| DEFAULT_SCRIPTS_DIR.to_string()),
                Some(format!("assembly {} was not found (no .asmdef with that name in Assets): the scripts compile into Assembly-CSharp", asm)),
            ),
        };
    }
    (want.unwrap_or_else(|| DEFAULT_SCRIPTS_DIR.to_string()), None)
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Positions of the `class` keyword of every declaration `class <name>` (whole words, any whitespace - spaces, tabs,
/// line breaks - between `class` and the name; `MainMenuViewBase` is not `MainMenuView`; `subclass X` is not a declaration).
fn class_decls(text: &str, name: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut from = 0;
    while let Some(i) = text[from..].find("class") {
        let a = from + i;
        from = a + 5;
        if text[..a].chars().next_back().is_some_and(is_ident_char) {
            continue;
        }
        let rest = &text[a + 5..];
        let trimmed = rest.trim_start_matches(|c: char| c.is_whitespace());
        if trimmed.len() == rest.len() {
            continue; // `class` must be followed by whitespace
        }
        if let Some(after) = trimmed.strip_prefix(name) {
            if after.chars().next().is_none_or(|c| !is_ident_char(c)) {
                out.push(a);
            }
        }
    }
    out
}

/// `class <name>` as whole words.
fn declares_class(text: &str, name: &str) -> bool {
    !class_decls(text, name).is_empty()
}

/// `abstract class <name>` (the generated ViewBase): `abstract` is the word right before `class`, or only separated
/// from it by other modifiers such as `partial`.
fn declares_abstract_class(text: &str, name: &str) -> bool {
    class_decls(text, name).into_iter().any(|a| {
        let before = &text[..a];
        let words: Vec<&str> = before.split(|c: char| c.is_whitespace() || matches!(c, '{' | '}' | ';' | ']')).filter(|w| !w.is_empty()).collect();
        words.iter().rev().take(4).take_while(|w| matches!(**w, "abstract" | "partial" | "public" | "internal" | "sealed" | "static")).any(|w| *w == "abstract")
    })
}

#[cfg(test)]
pub fn declares_abstract_class_for_tests(text: &str, name: &str) -> bool {
    declares_abstract_class(text, name)
}

/// Markers that a generated (overwritable) C# file carries in its first line (the Spanish one is from older versions).
const GENERATED_MARKS: [&str; 2] = ["Generated by Lienzo UI", "Generado por Lienzo UI"];

/// True when the first line of `text` carries a generated marker.
fn has_generated_mark(text: &str) -> bool {
    text.lines().next().is_some_and(|l| GENERATED_MARKS.iter().any(|m| l.contains(m)))
}

/// True if `<root>/<rel>` does not exist, or exists and carries the generated marker in its first line.
fn overwritable(root: &Path, rel: &str) -> bool {
    match std::fs::read(root.join(rel)) {
        Ok(b) => has_generated_mark(&decode_cs(&b[..b.len().min(4096)])),
        Err(e) => e.kind() == std::io::ErrorKind::NotFound,
    }
}

/// Case-insensitive, separator-insensitive form of a project-relative path (for comparing paths on Windows).
fn norm_rel(p: &str) -> String {
    p.replace('\\', "/").trim_matches('/').to_lowercase()
}

/// Write the selected outputs into `<project>/Assets/EditorGUI` (scripts into the resolved scripts folder).
/// Returns the written paths (relative to the project).
///
/// Only what `doc.unity.generate` asks for is written: `prefab` (importer + design JSON + the runtime helpers the
/// prefab needs), `uitk` (UXML + USS), `imgui` (`*OnGUI.cs`, only when `imgui_enabled()`), `view_scripts`
/// (`*ViewBase.cs` and the user's `*View.cs` stub). Pictures and fonts are copied for prefab / UI Toolkit.
///
/// A generated C# file is only overwritten when its first line carries the generated marker; a file without it is
/// the user's own and is skipped with a warning. The ViewBase is updated where it already lives in the project.
pub fn write_unity_package(doc: &Doc, project: &Path) -> Result<Vec<String>, String> {
    let root = find_unity_root(project).ok_or("that folder does not look like a Unity project (it has no Assets folder)")?;
    let gn = doc.unity.generate.clone();
    let cls = class_name(&doc.title);
    let (scripts, note) = resolve_scripts_dir(doc, &root);
    if let Some(n) = &note {
        eprintln!("[Lienzo UI] {}", n);
    }
    // Images are read before writing anything: if one is missing, the Unity project is not left half-written.
    // Pictures that already live in the Unity project (`unity:` / `Assets/...`) are referenced, never copied.
    // Only real pictures are copied: a known image extension AND png / jpeg / gif / webp magic bytes, from a path
    // that `library::resolve_image` accepted (anything else is skipped with a warning).
    let mut pictures = Vec::new();
    let mut done = HashSet::new();
    if gn.prefab || gn.uitk {
        for e in &doc.elements {
            if !e.image.is_empty() && unity_image_ref(&e.image).is_none() && done.insert(e.image.clone()) {
                let src = crate::library::resolve_image(&e.image);
                if src == crate::library::blocked_path() {
                    eprintln!("[Lienzo UI] the image {:?} is outside the allowed folders: it was not copied", e.image);
                    continue;
                }
                let ext = src.extension().and_then(|x| x.to_str()).unwrap_or("").to_ascii_lowercase();
                if !crate::library::IMAGE_EXTS.contains(&ext.as_str()) {
                    eprintln!("[Lienzo UI] the file {:?} is not a picture (extension .{}): it was not copied", e.image, ext);
                    continue;
                }
                let b = std::fs::read(&src).map_err(|err| format!("could not read the image {}: {}", src.display(), err))?;
                if !matches!(crate::community_validate::detect(&b), Some("png" | "jpeg" | "gif" | "webp")) {
                    eprintln!("[Lienzo UI] the file {:?} is not a png / jpeg / gif / webp picture: it was not copied", e.image);
                    continue;
                }
                pictures.push((image_asset_path(&e.image), b));
            }
        }
    }
    let mut written = Vec::new();
    let mut put = |rel: String, bytes: &[u8]| -> Result<(), String> {
        let p = root.join(&rel);
        if let Some(d) = p.parent() {
            std::fs::create_dir_all(d).map_err(|e| format!("{}: {}", d.display(), e))?;
        }
        std::fs::write(&p, bytes).map_err(|e| format!("{}: {}", p.display(), e))?;
        written.push(rel);
        Ok(())
    };
    // generated C# files: never overwrite a file that does not carry the generated marker
    let put_script = |rel: String, bytes: &[u8], put: &mut dyn FnMut(String, &[u8]) -> Result<(), String>| -> Result<(), String> {
        if !overwritable(&root, &rel) {
            eprintln!("[Lienzo UI] {} exists and is not a generated file (no \"{}\" in its first line): it was left as it is", rel, GENERATED_MARKS[0]);
            return Ok(());
        }
        put(rel, bytes)
    };
    if gn.prefab {
        put(format!("{}/Editor/EditorGuiImporter.cs", UNITY_ROOT), IMPORTER_CS.as_bytes())?;
    }
    for (rel, b) in &pictures {
        put(rel.clone(), b)?;
    }
    // fonts (Unity turns them into TextMeshPro font assets, unless a TMP asset with that name already exists)
    if gn.prefab || gn.uitk {
        for name in used_fonts(doc) {
            if let Some(src) = crate::render::font_path(&name) {
                let ext = src.extension().and_then(|x| x.to_str()).unwrap_or("ttf").to_lowercase();
                if let Ok(b) = std::fs::read(&src) {
                    put(format!("{}/Fonts/{}.{}", UNITY_ROOT, name, ext), &b)?;
                }
            }
        }
    }
    if gn.uitk {
        put(format!("{}/UI Toolkit/{}.uss", UNITY_ROOT, cls), uitk::uss(doc).as_bytes())?;
        put(format!("{}/UI Toolkit/{}.uxml", UNITY_ROOT, cls), uitk::uxml(doc).as_bytes())?;
    }
    if gn.imgui_enabled() {
        let target = format!("{}/{}OnGUI.cs", scripts, cls);
        put_script(target.clone(), imgui::script(doc).as_bytes(), &mut put)?;
        // A generated copy left in another folder (the scripts folder changed) would declare the class twice:
        // it is removed only because the new copy now exists, and only when it carries the generated marker.
        if overwritable(&root, &target) && root.join(&target).exists() {
            let ongui = format!("{}OnGUI", cls);
            for old in scan_cs(&root, |t| declares_class(t, &ongui) && has_generated_mark(t)) {
                if norm_rel(&old) != norm_rel(&target) {
                    let _ = std::fs::remove_file(root.join(&old));
                    let _ = std::fs::remove_file(root.join(format!("{}.meta", old)));
                }
            }
        }
    }
    if gn.view_scripts {
        // Where does a `{cls}ViewBase` already live? A generated copy is updated in place (it may have been moved
        // to another folder / assembly, or left from an export with another scripts folder); it is never deleted.
        // A copy without the generated marker belongs to the user: nothing is written (a second class with the
        // same name would not compile).
        let target = format!("{}/{}ViewBase.cs", scripts, cls);
        let base_name = format!("{}ViewBase", cls);
        let existing = scan_cs(&root, |t| declares_abstract_class(t, &base_name));
        let mut generated: Vec<String> = Vec::new();
        let mut user_owned: Vec<String> = Vec::new();
        for rel in existing {
            if overwritable(&root, &rel) { generated.push(rel) } else { user_owned.push(rel) }
        }
        let base_code = view::base_script(doc);
        let base_existed;
        if let Some(own) = user_owned.first() {
            eprintln!("[Lienzo UI] {} declares {} without the generated marker: the ViewBase was not written", own, base_name);
            base_existed = true;
        } else {
            // Where to write: the resolved scripts folder when a copy is already there; else the copy that lives
            // elsewhere (moved by the user, or left by an export with another scripts folder) is updated in place.
            // The one exception: a generated copy inside Lienzo's own folder (Assets/EditorGUI) is a leftover of the
            // default location (e.g. the assembly / scripts folder was set afterwards): the new copy is written first
            // and only then the leftover is removed, so there is never a moment without a copy.
            let in_target = generated.iter().find(|g| norm_rel(g) == norm_rel(&target)).cloned();
            let own_folder = format!("{}/", norm_rel(UNITY_ROOT));
            let leftovers: Vec<String> = if in_target.is_none() { generated.iter().filter(|g| norm_rel(g).starts_with(&own_folder)).cloned().collect() } else { Vec::new() };
            let at = in_target.clone().or_else(|| generated.iter().find(|g| !leftovers.contains(g)).cloned());
            base_existed = !generated.is_empty();
            let dest = at.unwrap_or(target.clone());
            if norm_rel(&dest) != norm_rel(&target) {
                eprintln!("[Lienzo UI] the ViewBase was updated where it already is ({}), not in {}", dest, scripts);
            }
            for extra in generated.iter().filter(|g| norm_rel(g) != norm_rel(&dest) && !leftovers.contains(g)) {
                eprintln!("[Lienzo UI] {} also declares {}: delete one of the two copies", extra, base_name);
            }
            put(dest.clone(), base_code.as_bytes())?;
            for old in &leftovers {
                if norm_rel(old) != norm_rel(&dest) {
                    let _ = std::fs::remove_file(root.join(old));
                    let _ = std::fs::remove_file(root.join(format!("{}.meta", old)));
                }
            }
        }
        let user_view = format!("{}/{}View.cs", scripts, cls);
        // The View is user-owned: write the stub only if the user has no `{cls}View` class anywhere in
        // the project (they may have moved or renamed the file) and the base class was not already exported.
        if !base_existed && !view_class_exists(&root, &cls) {
            put(user_view, view::user_script(doc).as_bytes())?;
        }
    }
    if gn.prefab {
        // Runtime helpers (shared by every design, compiled together with the scripts: the configured
        // assembly or Assembly-CSharp). Written once; a copy declared elsewhere in the project is respected.
        let mut helpers: Vec<(&str, &str)> = Vec::new();
        if doc.canvas.safe_area {
            helpers.push(("LienzoSafeArea", SAFE_AREA_CS));
        }
        if doc.portrait.as_ref().is_some_and(|p| !p.overrides.is_empty()) {
            helpers.push(("LienzoOrientationLayout", ORIENTATION_CS));
        }
        for (name, code) in helpers {
            let target = format!("{}/Runtime/{}.cs", scripts, name);
            if root.join(&target).exists() || scan_cs(&root, |t| declares_class(t, name)).is_empty() {
                put_script(target, code.as_bytes(), &mut put)?;
            }
        }
        // the design last: the importer rebuilds the prefab when this file changes
        put(format!("{}/Designs/{}.json", UNITY_ROOT, cls), ugui::json(doc, true).as_bytes())?;
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model;

    #[test]
    fn identifiers() {
        assert_eq!(class_name("Main menu"), "MainMenu");
        assert_eq!(class_name("3 vidas"), "Ui3Vidas");
        assert_eq!(ident("Play button"), "playButton");
        assert_eq!(cs_str("a\"b\nc"), "\"a\\\"b\\nc\"");
    }

    #[test]
    fn names_and_paths() {
        let d = model::template(3, 1920.0, 1080.0);
        let names = unity_names(&d);
        let slider = d.elements.iter().find(|e| e.name == "Slider Brightness").unwrap();
        assert_eq!(unity_path(&d, &names, slider.id), "Card/Options/Viewport/Content/Slider Brightness");
    }

    #[test]
    fn package_is_written() {
        let dir = std::env::temp_dir().join(format!("editorgui-unity-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("Assets")).unwrap();
        std::fs::create_dir_all(dir.join("ProjectSettings")).unwrap();
        let d = model::template(0, 1920.0, 1080.0);
        let files = write_unity_package(&d, &dir.join("Assets")).unwrap();
        assert!(files.iter().any(|f| f.ends_with("Designs/MainMenu.json")));
        assert!(files.iter().any(|f| f.ends_with("Scripts/MainMenuView.cs")));
        // the user's View script is never overwritten
        let again = write_unity_package(&d, &dir).unwrap();
        assert!(!again.iter().any(|f| f.ends_with("Scripts/MainMenuView.cs")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod dump {
    /// `EDITORGUI_DUMP=<folder> cargo test dump_templates -- --ignored` writes the
    /// Unity package of every template (used to compile the generated C#).
    #[test]
    #[ignore]
    fn dump_templates() {
        let dir = std::path::PathBuf::from(std::env::var("EDITORGUI_DUMP").expect("EDITORGUI_DUMP"));
        std::fs::create_dir_all(dir.join("Assets")).unwrap();
        std::fs::create_dir_all(dir.join("ProjectSettings")).unwrap();
        for i in 0..crate::model::TEMPLATES.len() {
            let d = crate::model::template(i, 1920.0, 1080.0);
            super::write_unity_package(&d, &dir).unwrap();
        }
    }
}

#[cfg(test)]
mod robustness_tests {
    use super::*;

    #[test]
    fn radius_with_odd_sizes_does_not_panic() {
        let mut e = Element::new(1, crate::model::Kind::Panel);
        e.radius = 10.0;
        for (w, h) in [(-5.0, 10.0), (10.0, -5.0), (f32::NAN, 4.0), (0.0, 0.0), (f32::INFINITY, f32::INFINITY)] {
            e.w = w;
            e.h = h;
            assert!(radius_of(&e) >= 0.0);
        }
        e.w = 20.0;
        e.h = 8.0;
        assert_eq!(radius_of(&e), 4.0);
    }

    #[test]
    fn long_kind_name_is_shortened() {
        assert!(class_name(&"x".repeat(500)).chars().count() <= 64);
        assert_eq!(class_name("Main menu"), "MainMenu");
    }
}

#[cfg(test)]
mod hostile_tests {
    use super::*;
    use crate::model::{Doc, Element, Kind};

    const SEPS: [char; 3] = ['\u{2028}', '\u{2029}', '\u{85}'];

    /// A text that tries to end a comment / string with every kind of line break and then run `Injected()`.
    fn evil(tag: &str) -> String {
        format!("{tag}\u{2028}Injected1();\u{2029}Injected2();\u{85}Injected3();\rInjected4();\r\nInjected5(); \"; Injected6(); //")
    }

    fn hostile_doc() -> Doc {
        let mut d = Doc::empty(1920.0, 1080.0);
        d.title = evil("T");
        for (i, k) in [Kind::Button, Kind::Text, Kind::Toggle, Kind::InputField, Kind::Dropdown, Kind::Slider, Kind::Image, Kind::Panel].into_iter().enumerate() {
            let mut e = Element::new(i as u64 + 1, k);
            e.name = evil("N");
            e.text = evil("X");
            e.on_click = evil("C");
            e.options = vec![evil("O"), evil("P")];
            e.image = if k == Kind::Image { format!("{}.png", evil("I")) } else { String::new() };
            d.elements.push(e);
        }
        d.next_id = 100;
        d
    }

    /// `needle` is inside a string literal or after a `//` in this line.
    fn harmless(line: &str, needle: &str) -> bool {
        let at = line.find(needle).unwrap();
        let before = &line[..at];
        let mut in_str = false;
        let mut esc = false;
        let mut prev = ' ';
        for c in before.chars() {
            if in_str {
                if esc { esc = false } else if c == '\\' { esc = true } else if c == '"' { in_str = false }
            } else if c == '"' {
                in_str = true;
            } else if c == '/' && prev == '/' {
                return true;
            }
            prev = c;
        }
        in_str
    }

    fn check(name: &str, cs: &str) {
        assert!(!cs.contains('\r'), "{name}: raw \r");
        for s in SEPS {
            assert!(!cs.contains(s), "{name}: raw U+{:04X}", s as u32);
        }
        for line in cs.lines() {
            for n in 1..=6 {
                let needle = format!("Injected{n}();");
                if line.contains(&needle) {
                    assert!(harmless(line, &needle), "{name}: injected statement: {line}");
                }
            }
        }
    }

    #[test]
    fn hostile_strings_cannot_inject_csharp() {
        let mut d = hostile_doc();
        d.sanitize(); // what loading does: separators are removed already
        let raw = hostile_doc(); // and the exporters must be safe even without sanitize
        for doc in [&d, &raw] {
            check("imgui", &imgui::script(doc));
            check("view_base", &view::base_script(doc));
            check("view_user", &view::user_script(doc));
        }
    }

    #[test]
    fn uss_uxml_comments_stay_closed() {
        let mut d = hostile_doc();
        d.title = "a-- b */ **// -".into();
        d.elements[0].name = "x**//y */ z\u{2028}".into();
        let uxml = uitk::uxml(&d);
        for l in uxml.lines().filter(|l| l.contains("<!--")) {
            let inner = &l[l.find("<!--").unwrap() + 4..l.rfind("-->").unwrap()];
            assert!(!inner.contains("--") && !inner.ends_with('-'), "{l}");
        }
        let uss = uitk::uss(&d);
        for l in uss.lines().filter(|l| l.starts_with("/*")) {
            let inner = &l[2..l.rfind("*/").unwrap()];
            assert!(!inner.contains("*/"), "{l}");
        }
    }

    #[test]
    fn cs_str_escapes_line_breaks() {
        let s = cs_str("a\r\u{2028}\u{2029}\u{85}\u{0}b");
        assert_eq!(s, "\"a\\r\\u2028\\u2029\\u0085\\u0000b\"");
        assert_eq!(sanitize_line("a\r\nb\u{2028}c"), "a  b c");
        assert_eq!(comment_lines("a\r\nb\rc\u{2028}d\n"), vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn model_sanitize_removes_separators() {
        let mut d = hostile_doc();
        d.sanitize();
        let all = |t: &str| !t.contains(|c| c == '\r' || SEPS.contains(&c));
        assert!(all(&d.title));
        for e in &d.elements {
            assert!(all(&e.name) && all(&e.text) && all(&e.on_click) && all(&e.font) && all(&e.image));
            assert!(e.options.iter().all(|o| all(o)));
        }
    }

    #[test]
    fn view_stub_not_duplicated_when_moved_or_base_existed() {
        let dir = std::env::temp_dir().join(format!("editorgui-view-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Assets/Game/UI")).unwrap();
        std::fs::create_dir_all(dir.join("ProjectSettings")).unwrap();
        let d = crate::model::template(0, 1920.0, 1080.0);
        let cls = class_name(&d.title);
        // the user already has the View somewhere else
        std::fs::write(dir.join(format!("Assets/Game/UI/{cls}View.cs")), format!("public class {cls}View : {cls}ViewBase {{ }}")).unwrap();
        let files = write_unity_package(&d, &dir).unwrap();
        assert!(!files.iter().any(|f| f.ends_with(&format!("Scripts/{cls}View.cs"))), "{files:?}");
        assert!(!dir.join(format!("{UNITY_ROOT}/Scripts/{cls}View.cs")).exists());
        // ViewBase exists from a previous export and the user deleted/moved the View: treated as user-owned
        std::fs::remove_file(dir.join(format!("Assets/Game/UI/{cls}View.cs"))).unwrap();
        let again = write_unity_package(&d, &dir).unwrap();
        assert!(!again.iter().any(|f| f.ends_with(&format!("Scripts/{cls}View.cs"))));
        // a class that merely starts with the same name is not the View
        assert!(!declares_class(&format!("class {cls}ViewBase : X"), &format!("{cls}View")));
        assert!(declares_class(&format!("public sealed class {cls}View : X"), &format!("{cls}View")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
