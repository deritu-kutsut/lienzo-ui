//! Export to Unity: uGUI (JSON + editor importer that builds a prefab), UI Toolkit
//! (UXML + USS), IMGUI (an `OnGUI` MonoBehaviour) and a "View" script with
//! references to the named elements.

pub mod imgui;
pub mod ugui;
pub mod uitk;
pub mod view;

use crate::model::{Doc, Element, Frame, Kind, rect_transform_in, RectTransform};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Unity editor script (goes in Assets/EditorGUI/Editor/).
pub const IMPORTER_CS: &str = include_str!("EditorGuiImporter.cs");

/// Folder inside the Unity project.
pub const UNITY_ROOT: &str = "Assets/EditorGUI";

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

/// Name an element would get before de-duplication: its valid `unity_name` verbatim, else its display name.
pub fn base_unity_name(e: &Element) -> String {
    if let Some(u) = e.unity_name.as_deref().filter(|u| crate::docext::valid_unity_name(u)) {
        return u.to_string();
    }
    let n = e.name.replace('/', "-").trim().to_string();
    if n.is_empty() { e.kind.label().to_string() } else { n }
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
    let mut out = HashMap::new();
    let mut clashes = Vec::new();
    let mut used: HashMap<Option<u64>, HashSet<String>> = HashMap::new();
    let explicit = |e: &Element| e.unity_name.as_deref().is_some_and(crate::docext::valid_unity_name);
    for pass in [true, false] {
        for e in doc.elements.iter().filter(|e| explicit(e) == pass) {
            let base = base_unity_name(e);
            let set = used.entry(doc.parent_of(e.id)).or_default();
            let mut name = base.clone();
            let mut k = 2;
            while set.contains(&name) {
                name = format!("{} ({})", base, k);
                k += 1;
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
    while let Some(p) = doc.parent_of(cur) {
        if doc.get(p).map(|e| e.kind == Kind::ScrollView).unwrap_or(false) {
            parts.push("Viewport/Content".into());
        }
        parts.push(names.get(&p).cloned().unwrap_or_default());
        cur = p;
    }
    parts.reverse();
    parts.join("/")
}

/// Height of a ScrollView's content: tall enough for every child (at least the view).
pub fn content_height(doc: &Doc, sv: &Element) -> f32 {
    let f = sv.frame();
    let mut bottom = f.h;
    for c in doc.children(Some(sv.id)) {
        if let Some(e) = doc.get(c) {
            let b = e.aabb();
            for corner in [[b[0], b[1]], [b[2], b[3]]] {
                let l = f.to_local(corner);
                bottom = bottom.max(l[1] + f.h / 2.0 + 20.0);
            }
        }
    }
    bottom.ceil()
}

/// The box Unity measures an element's RectTransform against: its parent, or the
/// parent's Content when the parent is a ScrollView (top-aligned, full width).
pub fn export_parent_frame(doc: &Doc, e: &Element) -> Frame {
    match doc.parent_of(e.id).and_then(|p| doc.get(p)) {
        Some(p) if p.kind == Kind::ScrollView => {
            let pf = p.frame();
            let ch = content_height(doc, p);
            let c = pf.to_world([0.0, -pf.h / 2.0 + ch / 2.0]);
            Frame { cx: c[0], cy: c[1], w: pf.w, h: ch, rot: pf.rot }
        }
        Some(p) => p.frame(),
        None => doc.page_frame(),
    }
}

pub fn export_rt(doc: &Doc, e: &Element) -> RectTransform {
    rect_transform_in(e, &export_parent_frame(doc, e))
}

/// Corner radius usable for an element (never more than half its short side).
pub fn radius_of(e: &Element) -> f32 {
    let max = e.w.min(e.h) / 2.0;
    // With negative or non-numeric width/height (foreign files) `clamp` would panic.
    if max.is_finite() && max > 0.0 && e.radius.is_finite() { e.radius.clamp(0.0, max) } else { 0.0 }
}

/// File name an image gets inside Assets/EditorGUI/Sprites.
pub fn image_file_name(image: &str) -> String {
    Path::new(image).file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| image.to_string())
}

pub fn image_asset_path(image: &str) -> String {
    format!("{}/Sprites/{}", UNITY_ROOT, image_file_name(image))
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

/// True if some `.cs` under `<project>/Assets` already declares `class {cls}View` (bounded textual scan).
fn view_class_exists(root: &Path, cls: &str) -> bool {
    let needle = format!("{}View", cls);
    let mut stack = vec![root.join("Assets")];
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
                    return false;
                }
                if ent.metadata().map(|m| m.len() > 2_000_000).unwrap_or(true) {
                    continue;
                }
                if let Ok(t) = std::fs::read_to_string(&p) {
                    if declares_class(&t, &needle) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// `class <name>` as whole words (so `MainMenuViewBase` does not count as `MainMenuView`).
fn declares_class(text: &str, name: &str) -> bool {
    let pat = format!("class {}", name);
    let mut from = 0;
    while let Some(i) = text[from..].find(&pat) {
        let a = from + i;
        let after = text[a + pat.len()..].chars().next();
        let before = text[..a].chars().next_back();
        if after.is_none_or(|c| !(c.is_alphanumeric() || c == '_')) && before.is_none_or(|c| c.is_whitespace()) {
            return true;
        }
        from = a + pat.len();
    }
    false
}

/// Write everything into `<project>/Assets/EditorGUI`. Returns the written paths (relative to the project).
pub fn write_unity_package(doc: &Doc, project: &Path) -> Result<Vec<String>, String> {
    let root = find_unity_root(project).ok_or("that folder does not look like a Unity project (it has no Assets folder)")?;
    let base = root.join(UNITY_ROOT);
    let cls = class_name(&doc.title);
    // Images are read before writing anything: if one is missing, the Unity project is not left half-written.
    let mut pictures = Vec::new();
    let mut done = HashSet::new();
    for e in &doc.elements {
        if !e.image.is_empty() && done.insert(e.image.clone()) {
            let src = crate::library::resolve_image(&e.image);
            let b = std::fs::read(&src).map_err(|err| format!("could not read the image {}: {}", src.display(), err))?;
            pictures.push((image_asset_path(&e.image), b));
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
    put(format!("{}/Editor/EditorGuiImporter.cs", UNITY_ROOT), IMPORTER_CS.as_bytes())?;
    for (rel, b) in &pictures {
        put(rel.clone(), b)?;
    }
    // fonts (Unity turns them into TextMeshPro font assets)
    for name in used_fonts(doc) {
        if let Some(src) = crate::render::font_path(&name) {
            let ext = src.extension().and_then(|x| x.to_str()).unwrap_or("ttf").to_lowercase();
            if let Ok(b) = std::fs::read(&src) {
                put(format!("{}/Fonts/{}.{}", UNITY_ROOT, name, ext), &b)?;
            }
        }
    }
    put(format!("{}/UI Toolkit/{}.uss", UNITY_ROOT, cls), uitk::uss(doc).as_bytes())?;
    put(format!("{}/UI Toolkit/{}.uxml", UNITY_ROOT, cls), uitk::uxml(doc).as_bytes())?;
    put(format!("{}/Scripts/{}OnGUI.cs", UNITY_ROOT, cls), imgui::script(doc).as_bytes())?;
    let base_existed = root.join(format!("{}/Scripts/{}ViewBase.cs", UNITY_ROOT, cls)).exists();
    put(format!("{}/Scripts/{}ViewBase.cs", UNITY_ROOT, cls), view::base_script(doc).as_bytes())?;
    let user_view = format!("{}/Scripts/{}View.cs", UNITY_ROOT, cls);
    // The View is user-owned: write the stub only if the user has no `{cls}View` class anywhere in
    // the project (they may have moved or renamed the file) and the base class was not already exported.
    if !base_existed && !view_class_exists(&root, &cls) {
        put(user_view, view::user_script(doc).as_bytes())?;
    }
    // the design last: the importer rebuilds the prefab when this file changes
    put(format!("{}/Designs/{}.json", UNITY_ROOT, cls), ugui::json(doc, true).as_bytes())?;
    let _ = base;
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
