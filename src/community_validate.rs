//! Validation of what the community downloads and uploads. These are the same rules the server applies
//! (`supabase/functions/_shared/validate.ts`): the app NEVER trusts what it downloads and re-checks it.
//! Data formats only; never executables, installers, system scripts, archives or SVG.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const KINDS: &[&str] = &["plugin", "image", "font", "template", "element", "theme"];

pub fn byte_limit(kind: &str) -> usize {
    match kind {
        "image" => 2 * 1024 * 1024,
        "font" => 5 * 1024 * 1024,
        "template" => 512 * 1024,
        "theme" => 32 * 1024,
        _ => 256 * 1024, // element, plugin
    }
}

const FORBIDDEN_EXTS: &[&str] = &[
    "exe", "dll", "bat", "cmd", "com", "scr", "msi", "ps1", "psm1", "vbs", "vbe", "js", "jse", "mjs", "cjs", "wsf", "jar", "apk", "dmg", "app", "sh", "bash", "lnk", "reg", "hta", "cpl", "so", "dylib", "py", "rb", "php", "zip",
    "rar", "7z", "gz", "tar", "iso", "svg", "html", "htm",
];

/// Extensions a plugin exporter may write. Anything else (and anything in `FORBIDDEN_EXTS`) is refused.
pub const EXPORT_EXTENSIONS: &[&str] = &["txt", "csv", "json", "xml", "md", "cs", "uxml", "uss", "tsv"];

/// An exporter `extension`: lowercase `[a-z0-9]{1,5}`, in the allowlist and not forbidden.
pub fn valid_export_extension(e: &str) -> bool {
    (1..=5).contains(&e.len()) && e.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit()) && EXPORT_EXTENSIONS.contains(&e) && !FORBIDDEN_EXTS.contains(&e)
}

/// Invisible / direction-changing characters that let a name look like another one (RLO, zero-width, BOM, ALM...).
pub fn spoofing_char(c: char) -> bool {
    matches!(c, '\u{200E}' | '\u{200F}' | '\u{061C}' | '\u{200B}'..='\u{200D}' | '\u{FEFF}' | '\u{2060}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

/// A name or title shown to other people: no control characters, no spoofing characters.
pub fn valid_display_text(s: &str) -> bool {
    !s.chars().any(|c| c.is_control() || spoofing_char(c) || c == '\u{2028}' || c == '\u{2029}')
}

/// Text from the server for printing in a terminal: control characters (ESC, BEL, CR, C1...) and spoofing characters removed.
pub fn terminal_safe(s: &str) -> String {
    s.chars().filter(|c| !c.is_control() && !spoofing_char(*c) && *c != '\u{2028}' && *c != '\u{2029}').collect()
}

/// `terminal_safe` applied to every string of a JSON value (in place).
pub fn terminal_safe_json(v: &mut serde_json::Value) {
    match v {
        serde_json::Value::String(s) => *s = terminal_safe(s),
        serde_json::Value::Array(a) => a.iter_mut().for_each(terminal_safe_json),
        serde_json::Value::Object(o) => o.values_mut().for_each(terminal_safe_json),
        _ => {}
    }
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{:02x}", b)).collect()
}

fn ext(name: &str) -> String {
    name.rsplit_once('.').map(|(_, e)| e.to_lowercase()).unwrap_or_default()
}

/// Actual format from the first bytes (not from the extension).
pub fn detect(b: &[u8]) -> Option<&'static str> {
    let starts = |f: &[u8], d: usize| b.len() >= d + f.len() && &b[d..d + f.len()] == f;
    if starts(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a], 0) {
        Some("png")
    } else if starts(&[0xff, 0xd8, 0xff], 0) {
        Some("jpeg")
    } else if starts(b"GIF8", 0) {
        Some("gif")
    } else if starts(b"RIFF", 0) && starts(b"WEBP", 8) {
        Some("webp")
    } else if starts(&[0, 1, 0, 0], 0) || starts(b"true", 0) {
        Some("ttf")
    } else if starts(b"OTTO", 0) {
        Some("otf")
    } else if starts(b"MZ", 0) {
        Some("executable-windows")
    } else if starts(&[0x7f, b'E', b'L', b'F'], 0) {
        Some("executable-linux")
    } else if starts(&[0x50, 0x4b, 0x03, 0x04], 0) {
        Some("zip")
    } else if starts(&[0xcf, 0xfa, 0xed, 0xfe], 0) || starts(&[0xca, 0xfe, 0xba, 0xbe], 0) {
        Some("executable-mac")
    } else if starts(b"#!", 0) {
        Some("script-shebang")
    } else {
        None
    }
}

/// Old rule, used on free text inside designs (it may legitimately contain `:`).
fn dangerous_path_basic(s: &str) -> bool {
    let b = s.as_bytes();
    s.contains('\0') || s.starts_with('/') || s.starts_with('\\') || (b.len() >= 2 && b[1] == b':') || s.split(['/', '\\']).any(|p| p == "..")
}

const RESERVED: &[&str] = &["con", "prn", "aux", "nul", "conin$", "conout$", "clock$"];

/// `com1`..`com9` / `lpt1`..`lpt9`, also with the superscript digits Windows treats as the same devices (COM1 with a superscript 1, 2 or 3).
fn reserved_numbered(stem: &str) -> bool {
    let mut chars = stem.chars();
    let prefix: String = chars.by_ref().take(3).collect();
    if prefix != "com" && prefix != "lpt" {
        return false;
    }
    let rest: Vec<char> = chars.collect();
    rest.len() == 1 && matches!(rest[0], '1'..='9' | '\u{00B9}' | '\u{00B2}' | '\u{00B3}')
}

/// Stricter rule for file paths inside a package (they are written to disk on Windows):
/// no NTFS streams (`:`), no trailing dots or spaces, no reserved device names (`con`, `nul.txt`, `com1`...).
fn dangerous_path(s: &str) -> bool {
    if dangerous_path_basic(s) || s.contains(':') || s.is_empty() || s.chars().any(|c| c.is_control()) {
        return true;
    }
    s.split(['/', '\\']).any(|part| {
        if part.is_empty() || part.ends_with('.') || part.ends_with(' ') || part.starts_with(' ') {
            return true;
        }
        let stem = part.split('.').next().unwrap_or("").trim_end().to_lowercase();
        RESERVED.contains(&stem.as_str()) || reserved_numbered(&stem)
    })
}

/// Characters that break generated C#: control characters and line separators (U+2028, U+2029, U+0085, CR).
fn bad_text_char(c: char) -> bool {
    (c.is_control() && c != '\n' && c != '\t') || c == '\u{2028}' || c == '\u{2029}'
}

/// How deep a JSON document may be nested. `serde_json` already refuses text deeper than 128 levels when it PARSES, but values built in memory
/// (plugin output, scripts) are not bounded by that: anything deeper than this is refused instead of being silently left unscanned.
const MAX_JSON_DEPTH: usize = 1000;

/// Every string VALUE of a JSON document (iterative: no recursion, so a hostile nesting cannot overflow the stack), or an error when the
/// document is nested deeper than `MAX_JSON_DEPTH`.
fn texts(v: &serde_json::Value, acc: &mut Vec<String>) -> Result<(), String> {
    let mut stack: Vec<(&serde_json::Value, usize)> = vec![(v, 0)];
    while let Some((v, depth)) = stack.pop() {
        match v {
            serde_json::Value::String(s) => acc.push(s.clone()),
            serde_json::Value::Array(a) => {
                if depth >= MAX_JSON_DEPTH {
                    return Err("The document is nested too deeply.".into());
                }
                stack.extend(a.iter().rev().map(|x| (x, depth + 1)));
            }
            serde_json::Value::Object(o) => {
                if depth >= MAX_JSON_DEPTH {
                    return Err("The document is nested too deeply.".into());
                }
                stack.extend(o.values().map(|x| (x, depth + 1)));
            }
            _ => {}
        }
    }
    Ok(())
}

fn external_url(s: &str) -> bool {
    let l = s.to_lowercase();
    ["http:", "https:", "ftp:", "file:", "data:", "javascript:"].iter().any(|p| l.contains(p))
}

/// Longest `on_click` script a downloaded design or a plugin may carry.
const MAX_ON_CLICK: usize = 4096;

/// An `on_click` script of an element is CODE that runs when the design is tested: besides the text rules (no control characters),
/// it gets the same scan as a plugin script (no eval, import, links, processes) and a length limit.
fn check_on_click(code: &str) -> Result<(), String> {
    if code.len() > MAX_ON_CLICK {
        return Err("An on-click script is too long.".into());
    }
    if code.chars().any(|c| spoofing_char(c) || c == '\u{2028}' || c == '\u{2029}') {
        return Err("An on-click script has invisible characters.".into());
    }
    if suspicious(code) {
        return Err("An on-click script contains forbidden instructions (eval, import, network, processes).".into());
    }
    Ok(())
}

/// The optional element fields (`unity_name`, `slice`, `ppu`, `sprite`, `image_mode`, `preserve_aspect`) are allowed,
/// but with the right type and range.
pub fn validate_element_fields(e: &serde_json::Value) -> Result<(), String> {
    use serde_json::Value;
    let bad = |what: &str| Err(format!("Element field “{}” is not valid.", what));
    let Some(o) = e.as_object() else { return Ok(()) };
    if let Some(v) = o.get("on_click") {
        // the code itself is checked in `validate_design` / `check_plugin_output` (it may already be in the user's own design)
        if !matches!(v, Value::Null | Value::String(_)) {
            return bad("on_click");
        }
    }
    if let Some(v) = o.get("unity_name").filter(|v| !v.is_null()) {
        if !v.as_str().is_some_and(crate::docext::valid_unity_name) {
            return bad("unity_name");
        }
    }
    if let Some(v) = o.get("sprite").filter(|v| !v.is_null()) {
        if !v.as_str().is_some_and(|s| s.chars().count() <= 256 && !s.trim().is_empty()) {
            return bad("sprite");
        }
    }
    if let Some(v) = o.get("slice").filter(|v| !v.is_null()) {
        let ok = v.as_array().is_some_and(|a| a.len() == 4 && a.iter().all(|x| x.as_f64().is_some_and(|f| f.is_finite() && (0.0..=100_000.0).contains(&f))));
        if !ok {
            return bad("slice");
        }
    }
    if let Some(v) = o.get("ppu").filter(|v| !v.is_null()) {
        if !v.as_f64().is_some_and(|f| (0.01..=10_000.0).contains(&f)) {
            return bad("ppu");
        }
    }
    if let Some(v) = o.get("image_mode").filter(|v| !v.is_null()) {
        if !matches!(v.as_str(), Some("Simple" | "Sliced" | "Tiled" | "Filled")) {
            return bad("image_mode");
        }
    }
    if let Some(v) = o.get("preserve_aspect") {
        if !matches!(v, Value::Bool(_)) {
            return bad("preserve_aspect");
        }
    }
    Ok(())
}

/// Optional Doc blocks (`canvas`, `unity`, `portrait`) of a template.
pub fn validate_doc_fields(json: &serde_json::Value) -> Result<(), String> {
    use crate::docext::{CanvasSettings, PortraitLayout, UnitySettings};
    let Some(o) = json.as_object() else { return Ok(()) };
    if let Some(v) = o.get("canvas") {
        serde_json::from_value::<CanvasSettings>(v.clone()).map_err(|_| "The design's “canvas” block is not valid.")?;
    }
    if let Some(v) = o.get("unity") {
        let u: UnitySettings = serde_json::from_value(v.clone()).map_err(|_| "The design's “unity” block is not valid.")?;
        if u.scripts_dir.as_deref().is_some_and(|s| !crate::docext::valid_scripts_dir(s)) || u.assembly.as_deref().is_some_and(|s| !crate::docext::valid_assembly(s)) {
            return Err("The design's “unity” block has an invalid folder or assembly name.".into());
        }
    }
    if let Some(v) = o.get("portrait").filter(|v| !v.is_null()) {
        let p: PortraitLayout = serde_json::from_value(v.clone()).map_err(|_| "The design's “portrait” block is not valid.")?;
        if p.overrides.len() > 2000 {
            return Err("Too many portrait overrides.".into());
        }
    }
    Ok(())
}

fn validate_design(json: serde_json::Value, is_template: bool) -> Result<(), String> {
    let json = if !is_template && (json.is_array() || json.is_object()) {
        // a library element is a list of elements (or a single one)
        let list = if json.is_array() { json } else { serde_json::Value::Array(vec![json]) };
        serde_json::json!({ "elements": list })
    } else {
        json
    };
    let elements = json.get("elements").and_then(|e| e.as_array()).ok_or("The design's “elements” list is missing.")?;
    if elements.len() > if is_template { 2000 } else { 500 } {
        return Err("Too many elements.".into());
    }
    for e in elements {
        validate_element_fields(e)?;
        if let Some(code) = e.get("on_click").and_then(|v| v.as_str()) {
            check_on_click(code)?;
        }
    }
    validate_doc_fields(&json)?;
    let mut t = Vec::new();
    texts(&serde_json::Value::Array(elements.clone()), &mut t)?;
    for s in &t {
        if external_url(s) {
            return Err("Designs cannot contain external links or paths.".into());
        }
        if s.chars().any(bad_text_char) {
            return Err("Designs cannot contain control characters or line separators.".into());
        }
        if dangerous_path_basic(s) && (s.contains('/') || s.contains('\\')) {
            return Err("Designs cannot contain absolute paths or paths with “..”.".into());
        }
    }
    // Image references are file paths: the strict rule applies (this also refuses drive-relative `C:foo.png`, which has no slash and so slips
    // through the text rule above, and NTFS streams / UNC paths).
    for e in elements {
        if let Some(r) = e.get("image").and_then(|v| v.as_str()).filter(|r| !r.is_empty()) {
            if dangerous_path(r) || crate::library::is_unc_or_stream(r) {
                return Err("Designs cannot contain absolute paths or paths with “..”.".into());
            }
        }
    }
    Ok(())
}

/// A plugin travels as a single JSON: { manifest, files: { "path.rhai": "code" } }.
/// Old community items (uploaded by app versions 1.0 - 1.3.3) use the Spanish keys `manifiesto` and `archivos`:
/// they are still accepted when reading; new packages are written with the English keys.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Package {
    #[serde(alias = "manifiesto")]
    pub manifest: serde_json::Value,
    #[serde(alias = "archivos")]
    pub files: BTreeMap<String, String>,
}

fn validate_plugin(json: serde_json::Value) -> Result<Package, String> {
    let p: Package = serde_json::from_value(json).map_err(|_| "A plugin needs a manifest and its files.")?;
    let name = crate::plugins::manifest_key(&p.manifest, "name", "nombre").and_then(|n| n.as_str()).unwrap_or("");
    if name.chars().count() < 3 || name.chars().count() > 60 {
        return Err("Manifest: invalid name.".into());
    }
    if !valid_display_text(name) {
        return Err("Manifest: the name has control or invisible characters.".into());
    }
    let mut manifest_texts = Vec::new();
    texts(&p.manifest, &mut manifest_texts)?;
    if manifest_texts.iter().any(|s| s.chars().any(|c| bad_text_char(c) || spoofing_char(c))) {
        return Err("Manifest: it has control or invisible characters.".into());
    }
    if p.files.is_empty() || p.files.len() > 20 {
        return Err("A plugin has between 1 and 20 files.".into());
    }
    // Case-insensitive duplicates ("a.rhai" and "A.RHAI") would collide on Windows and macOS.
    let mut seen = std::collections::HashSet::new();
    for path in p.files.keys() {
        if !seen.insert(path.to_lowercase()) {
            return Err(format!("Duplicate file (names differ only in capitals): {}", path));
        }
    }
    // Files the manifest runs as scripts (any extension) and the ones it offers as templates.
    let mut scripts = std::collections::HashSet::new();
    let mut templates = std::collections::HashSet::new();
    for (new, old) in [("commands", "comandos"), ("exporters", "exportadores")] {
        for x in crate::plugins::manifest_key(&p.manifest, new, old).and_then(|v| v.as_array()).into_iter().flatten() {
            if let Some(s) = x.get("script").and_then(|s| s.as_str()) {
                scripts.insert(s.to_string());
            }
            if new == "exporters" {
                let e = x.get("extension").and_then(|e| e.as_str()).unwrap_or("");
                if !valid_export_extension(e) {
                    return Err(format!("Exporter extension not allowed: “{}” (allowed: {}).", terminal_safe(e).chars().take(12).collect::<String>(), EXPORT_EXTENSIONS.join(", ")));
                }
            }
        }
    }
    for x in crate::plugins::manifest_key(&p.manifest, "templates", "plantillas").and_then(|v| v.as_array()).into_iter().flatten() {
        if let Some(s) = x.as_str() {
            templates.insert(s.to_string());
        }
    }
    for (path, content) in &p.files {
        if dangerous_path(path) || path.len() > 80 {
            return Err(format!("Path not allowed: {}", path));
        }
        let e = ext(path);
        if e != "rhai" && e != "json" {
            return Err(format!("Only .rhai and .json are allowed ({}).", path));
        }
        if path.eq_ignore_ascii_case("plugin.json") {
            // The manifest travels in `manifest`. Packages made by older versions repeated it here:
            // that identical copy is tolerated (and never written); anything else could replace the validated manifest.
            let same = serde_json::from_str::<serde_json::Value>(content).map(|v| v == p.manifest).unwrap_or(false);
            if !same {
                return Err("plugin.json must not be inside the package files.".into());
            }
            continue;
        }
        let is_script = scripts.contains(path.as_str());
        // every script is scanned, whatever its extension (a manifest can point a command to a .json file)
        if (e != "json" || is_script) && suspicious(content) {
            return Err(format!("{}: contains forbidden instructions (eval, import, network, processes).", path));
        }
        if e == "json" {
            match serde_json::from_str::<serde_json::Value>(content) {
                Ok(v) => {
                    // A .json that is a template or looks like a design is ALWAYS validated as a design,
                    // even when the manifest also lists it as a command script.
                    if templates.contains(path.as_str()) || v.get("elements").is_some() {
                        validate_design(v, true).map_err(|er| format!("{}: {}", path, er))?;
                    } else if !is_script {
                        // any other data file: same text rules as a design
                        let mut t = Vec::new();
                        texts(&v, &mut t).map_err(|er| format!("{}: {}", path, er))?;
                        if t.iter().any(|s| external_url(s) || s.chars().any(bad_text_char)) {
                            return Err(format!("{}: contains external links or control characters.", path));
                        }
                    }
                }
                Err(_) if is_script && !templates.contains(path.as_str()) => {} // rhai code in a .json file: scanned above
                Err(_) => return Err(format!("{}: not valid JSON.", path)),
            }
        }
    }
    for s in scripts.iter().chain(templates.iter()) {
        if !p.files.contains_key(s) {
            return Err(format!("The manifest uses {}, which is not in the package.", s));
        }
    }
    Ok(p)
}

/// Checks the design a plugin command returns (untrusted output). `input` is the design the command received:
/// strings and image references that were already there are kept; NEW ones must pass the same rules as a
/// downloaded design (no control characters, external links, absolute/UNC/drive paths or `..`).
pub fn check_plugin_output(input: &serde_json::Value, output: &serde_json::Value) -> Result<(), String> {
    let elements = output.get("elements").and_then(|e| e.as_array()).ok_or("The returned design has no “elements” list.")?;
    if elements.len() > 5000 {
        return Err("The returned design has too many elements.".into());
    }
    for e in elements {
        validate_element_fields(e)?;
    }
    validate_doc_fields(output)?;
    let mut old = Vec::new();
    texts(input, &mut old)?;
    let known: std::collections::HashSet<String> = old.into_iter().collect();
    let mut new = Vec::new();
    texts(&serde_json::Value::Array(elements.clone()), &mut new)?;
    for s in new.iter().filter(|s| !known.contains(*s)) {
        if external_url(s) {
            return Err("The returned design contains external links.".into());
        }
        if s.chars().any(bad_text_char) {
            return Err("The returned design contains control characters or line separators.".into());
        }
        if dangerous_path_basic(s) && (s.contains('/') || s.contains('\\')) {
            return Err("The returned design contains absolute paths or paths with “..”.".into());
        }
    }
    // A NEW on_click script (not one the design already had) gets the same scan as a downloaded design.
    for e in elements {
        if let Some(code) = e.get("on_click").and_then(|v| v.as_str()).filter(|c| !c.is_empty() && !known.contains(*c)) {
            check_on_click(code)?;
        }
    }
    for e in elements {
        for key in ["image"] {
            let Some(r) = e.get(key).and_then(|v| v.as_str()).filter(|r| !r.is_empty() && !known.contains(*r)) else { continue };
            if dangerous_path(r) || crate::library::is_unc_or_stream(r) {
                return Err(format!("The returned design has an image reference that is not allowed: {}", terminal_safe(r).chars().take(40).collect::<String>()));
            }
        }
    }
    Ok(())
}

fn suspicious(code: &str) -> bool {
    let l = code.to_lowercase();
    if l.contains("http://") || l.contains("https://") || l.contains("std::") || l.contains("os::") || l.contains(".exe") {
        return true;
    }
    // whole words: eval, import "…", Engine, process, Command, spawn
    let is_word = |w: &str| {
        let mut from = 0;
        while let Some(i) = l[from..].find(w) {
            let a = from + i;
            let before = l[..a].chars().next_back().is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
            let after = l[a + w.len()..].chars().next().is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
            if before && after {
                return true;
            }
            from = a + w.len();
        }
        false
    };
    is_word("eval") || is_word("engine") || is_word("process") || is_word("command") || is_word("spawn") || l.contains("import \"") || l.contains("import '")
}

/// Validates a file according to its type. `Ok(())` = safe to install or upload.
pub fn validate(kind: &str, name: &str, bytes: &[u8]) -> Result<(), String> {
    if !KINDS.contains(&kind) {
        return Err("Unknown content type.".into());
    }
    if bytes.is_empty() {
        return Err("Empty file.".into());
    }
    if bytes.len() > byte_limit(kind) {
        return Err(format!("Exceeds the limit of {} KB for {}.", byte_limit(kind) / 1024, kind));
    }
    if FORBIDDEN_EXTS.contains(&ext(name).as_str()) {
        return Err("File type not allowed.".into());
    }
    let real = detect(bytes);
    if let Some(r) = real {
        if r.starts_with("executable") || r == "zip" || r.starts_with("script") {
            return Err(format!("Dangerous content detected ({}).", r));
        }
    }
    match kind {
        "image" => {
            let r = real.filter(|r| ["png", "jpeg", "gif", "webp"].contains(r)).ok_or("Not a valid PNG, JPEG, GIF or WebP image.")?;
            let (w, h) = match r {
                "png" if bytes.len() >= 24 => (u32::from_be_bytes(bytes[16..20].try_into().unwrap()), u32::from_be_bytes(bytes[20..24].try_into().unwrap())),
                "gif" if bytes.len() >= 10 => (u16::from_le_bytes([bytes[6], bytes[7]]) as u32, u16::from_le_bytes([bytes[8], bytes[9]]) as u32),
                _ => (1, 1),
            };
            if w == 0 || h == 0 || w > 4096 || h > 4096 {
                return Err("Dimensions out of range (maximum 4096×4096).".into());
            }
            Ok(())
        }
        "font" => match real {
            Some("ttf") | Some("otf") => Ok(()),
            _ => Err("Only TTF or OTF fonts are allowed.".into()),
        },
        _ => {
            if real.is_some() {
                return Err("A text JSON was expected.".into());
            }
            let text = std::str::from_utf8(bytes).map_err(|_| "The file is not valid UTF-8 JSON.")?;
            let json: serde_json::Value = serde_json::from_str(text).map_err(|_| "The file is not valid JSON.")?;
            if kind == "theme" {
                // Only the known theme fields: #RRGGBB colors, corners, light base and a font name.
                let t: crate::theme::Theme = serde_json::from_value(json).map_err(|_| "The file is not a Lienzo UI theme.")?;
                t.validate()
            } else if kind == "plugin" {
                validate_plugin(json).map(|_| ())
            } else {
                validate_design(json, kind == "template")
            }
        }
    }
}

/// Reads an already-validated downloaded plugin.
pub fn read_package(bytes: &[u8]) -> Result<Package, String> {
    validate("plugin", "plugin.json", bytes)?;
    let json: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    validate_plugin(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(w: u32, h: u32) -> Vec<u8> {
        let mut b = vec![0u8; 33];
        b[..8].copy_from_slice(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
        b[16..20].copy_from_slice(&w.to_be_bytes());
        b[20..24].copy_from_slice(&h.to_be_bytes());
        b
    }

    #[test]
    fn valid_and_huge_image() {
        assert!(validate("image", "a.png", &png(16, 16)).is_ok());
        assert!(validate("image", "a.png", &png(9000, 10)).is_err());
    }

    #[test]
    fn executable_disguised_as_image() {
        let mut exe = vec![0u8; 64];
        exe[..2].copy_from_slice(b"MZ");
        assert!(validate("image", "foto.png", &exe).unwrap_err().contains("Dangerous"));
        for n in ["x.exe", "x.dll", "x.js", "x.svg", "x.zip", "x.ps1"] {
            assert!(validate("image", n, &png(8, 8)).is_err(), "{}", n);
        }
    }

    #[test]
    fn zip_named_json() {
        let mut z = vec![0u8; 32];
        z[..4].copy_from_slice(&[0x50, 0x4b, 3, 4]);
        assert!(validate("template", "a.json", &z).is_err());
    }

    #[test]
    fn fonts() {
        let mut ttf = vec![0u8; 40];
        ttf[..4].copy_from_slice(&[0, 1, 0, 0]);
        assert!(validate("font", "f.ttf", &ttf).is_ok());
        assert!(validate("font", "f.ttf", &png(8, 8)).is_err());
    }

    #[test]
    fn templates_and_elements() {
        let ok = br#"{"elements":[{"kind":"Text","text":"Hola mundo"}]}"#;
        assert!(validate("template", "t.json", ok).is_ok());
        assert!(validate("template", "t.json", br#"{"elements":[{"text":"visita https://malo.example"}]}"#).is_err());
        assert!(validate("template", "t.json", br#"{"elements":[{"image":"C:/Users/yo/foto.png"}]}"#).is_err());
        assert!(validate("template", "t.json", br#"{"elements":[{"image":"../../secreto.png"}]}"#).is_err());
        assert!(validate("template", "t.json", b"{no").is_err());
        assert!(validate("element", "e.json", br#"[{"kind":"Text","text":"Hola"}]"#).is_ok());
        assert!(validate("element", "e.json", br#"{"kind":"Text","text":"Hola"}"#).is_ok());
        assert!(validate("element", "e.json", br#"[{"text":"https://malo.example"}]"#).is_err());
    }

    fn plugin(files: &str) -> Vec<u8> {
        format!(r#"{{"manifest":{{"name":"Align","commands":[{{"name":"x","script":"a.rhai"}}]}},"files":{}}}"#, files).into_bytes()
    }

    #[test]
    fn plugins() {
        assert!(validate("plugin", "p.json", &plugin(r#"{"a.rhai":"fn run(d){ d }"}"#)).is_ok());
        for bad in [
            r#"{"../a.rhai":"x"}"#,
            r#"{"/etc/a.rhai":"x"}"#,
            r#"{"a.rhai":"x","m.exe":"MZ"}"#,
            r#"{"b.rhai":"x"}"#,
            r#"{"a.rhai":"eval(\"x\")"}"#,
            r#"{"a.rhai":"import \"mod\" as m;"}"#,
            r#"{"a.rhai":"let u = \"https://malo.example\";"}"#,
            r#"{"a.rhai":"Command::new"}"#,
        ] {
            assert!(validate("plugin", "p.json", &plugin(bad)).is_err(), "{}", bad);
        }
        // "evaluate" or "processing" are not "eval" or "process"
        assert!(validate("plugin", "p.json", &plugin(r#"{"a.rhai":"let evaluate = 1; let processing = 2; fn run(d){ d }"}"#)).is_ok());
    }

    #[test]
    fn bundled_plugins_can_be_packed() {
        // the upload format must accept what the repository already ships
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("plugins/snap-to-grid");
        let mut files = BTreeMap::new();
        for f in ["plugin.json", "snap.rhai"] {
            files.insert(f.to_string(), std::fs::read_to_string(base.join(f)).unwrap());
        }
        let m: serde_json::Value = serde_json::from_str(&files["plugin.json"]).unwrap();
        let p = Package { manifest: m, files };
        assert!(validate("plugin", "p.json", &serde_json::to_vec(&p).unwrap()).is_ok());
    }

    #[test]
    fn known_hash() {
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }

    #[test]
    fn package_reads_old_and_new_keys_and_writes_english_keys() {
        let old = br#"{"manifiesto":{"nombre":"Plug","comandos":[{"nombre":"x","script":"a.rhai"}]},"archivos":{"a.rhai":"fn run(d){ d }"}}"#;
        let new = br#"{"manifest":{"name":"Plug","commands":[{"name":"x","script":"a.rhai"}]},"files":{"a.rhai":"fn run(d){ d }"}}"#;
        assert!(read_package(old).is_ok());
        assert!(read_package(new).is_ok());
        let out = serde_json::to_string(&read_package(new).unwrap()).unwrap();
        assert!(out.contains("\"manifest\"") && out.contains("\"files\"") && !out.contains("manifiesto"), "new packages use the English keys: {}", out);
    }
}

/// Auxiliary images (icon and preview of a content item, profile avatar and banner): PNG only, with limits.
/// Same rules as the server's `validatePng`. Returns (width, height).
pub fn validate_png(class: &str, bytes: &[u8]) -> Result<(u32, u32), String> {
    let (max_bytes, max_w, max_h) = match class {
        "icon" => (256 * 1024, 512, 512),
        "avatar" => (512 * 1024, 512, 512),
        "preview" => (1024 * 1024, 1920, 1080),
        "banner" => (1536 * 1024, 1920, 640),
        _ => return Err("Unknown image type.".into()),
    };
    if bytes.is_empty() || bytes.len() > max_bytes {
        return Err(format!("The image must weigh at most {} KB.", max_bytes / 1024));
    }
    if detect(bytes) != Some("png") || bytes.len() < 24 {
        return Err("The image must be PNG.".into());
    }
    let w = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    if w == 0 || h == 0 || w > max_w || h > max_h {
        return Err(format!("The image must measure at most {}×{} px.", max_w, max_h));
    }
    Ok((w, h))
}

#[cfg(test)]
mod tests_png {
    use super::*;
    fn png(w: u32, h: u32) -> Vec<u8> {
        let mut b = vec![0u8; 33];
        b[..8].copy_from_slice(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);
        b[16..20].copy_from_slice(&w.to_be_bytes());
        b[20..24].copy_from_slice(&h.to_be_bytes());
        b
    }
    #[test]
    fn helper_images() {
        assert!(validate_png("icon", &png(256, 256)).is_ok());
        assert!(validate_png("icon", &png(1024, 1024)).is_err());
        assert!(validate_png("banner", &png(1500, 500)).is_ok());
        assert!(validate_png("banner", &png(1500, 900)).is_err());
        assert!(validate_png("avatar", &[0xff, 0xd8, 0xff, 0, 0, 0]).is_err());
    }
}

#[cfg(test)]
mod tests_hardening {
    use super::*;

    fn pkg(manifest: &str, files: serde_json::Value) -> Vec<u8> {
        format!(r#"{{"manifest":{},"files":{}}}"#, manifest, files).into_bytes()
    }
    const M: &str = r#"{"name":"Align","commands":[{"name":"x","script":"a.rhai"}]}"#;

    #[test]
    fn dangerous_paths() {
        for bad in ["a.rhai:evil", "a.rhai:stream:$DATA", "a.rhai.", "a.rhai ", "con.rhai", "CON", "nul.json", "aux", "com1.rhai", "LPT9.json", "d/prn.rhai", "d/lpt1", "c:\\x", "/x", "..\\x", "d\\..\\x"] {
            assert!(dangerous_path(bad), "{}", bad);
        }
        for ok in ["a.rhai", "sub/dir/b.json", "console.rhai", "com10.rhai", "com0.rhai", "snap.rhai"] {
            assert!(!dangerous_path(ok), "{}", ok);
        }
        let b = pkg(M, serde_json::json!({ "a.rhai": "fn run(d){ d }", "x.rhai:s": "1" }));
        assert!(validate("plugin", "p.json", &b).is_err());
    }

    #[test]
    fn plugin_json_inside_files_is_rejected_unless_it_is_the_same_legacy_copy() {
        let evil = pkg(M, serde_json::json!({ "a.rhai": "fn run(d){ d }", "plugin.json": r#"{"name":"Other","commands":[{"name":"y","script":"b.rhai"}]}"# }));
        assert!(validate("plugin", "p.json", &evil).unwrap_err().contains("plugin.json"));
        let legacy = pkg(M, serde_json::json!({ "a.rhai": "fn run(d){ d }", "plugin.json": M }));
        assert!(validate("plugin", "p.json", &legacy).is_ok());
    }

    #[test]
    fn template_files_are_validated_as_designs() {
        let m = r#"{"name":"Tpl","templates":["t.json"],"commands":[{"name":"x","script":"a.rhai"}]}"#;
        let ok = pkg(m, serde_json::json!({ "a.rhai": "fn run(d){ d }", "t.json": r#"{"elements":[{"kind":"Text","text":"Hi"}]}"# }));
        assert!(validate("plugin", "p.json", &ok).is_ok());
        for bad in [r#"{"elements":[{"text":"https://evil.example"}]}"#, r#"{"elements":[{"image":"C:/Users/x/a.png"}]}"#, r#"{"elements":[{"on_click":"a\u2028b"}]}"#, "{no"] {
            let b = pkg(m, serde_json::json!({ "a.rhai": "fn run(d){ d }", "t.json": bad }));
            assert!(validate("plugin", "p.json", &b).is_err(), "{}", bad);
        }
    }

    #[test]
    fn every_script_is_scanned_whatever_its_extension() {
        let m = r#"{"name":"Sneaky","commands":[{"name":"x","script":"data.json"}]}"#;
        let b = pkg(m, serde_json::json!({ "data.json": "eval(\"1\")" }));
        assert!(validate("plugin", "p.json", &b).is_err());
    }

    #[test]
    fn designs_reject_line_separators_and_control_chars() {
        for bad in ["a\u{2028}b", "a\u{2029}b", "a\u{85}b", "a\rb", "a\u{0}b"] {
            let j = serde_json::json!({ "elements": [{ "kind": "Button", "name": bad }] }).to_string();
            assert!(validate("template", "t.json", j.as_bytes()).is_err(), "{:?}", bad);
            let j = serde_json::json!({ "elements": [{ "kind": "Button", "on_click": bad }] }).to_string();
            assert!(validate("template", "t.json", j.as_bytes()).is_err(), "{:?}", bad);
        }
        let ok = serde_json::json!({ "elements": [{ "kind": "Button", "name": "Time: 5/10", "on_click": "a();\nb();\tc();" }] }).to_string();
        assert!(validate("template", "t.json", ok.as_bytes()).is_ok());
    }
}

#[cfg(test)]
mod tests_format_fields {
    use super::*;
    use serde_json::json;

    #[test]
    fn new_element_and_doc_fields_are_allowed_but_validated() {
        let el = |extra: serde_json::Value| {
            let mut e = json!({"id":1,"name":"a","kind":"Image","x":0,"y":0,"w":1,"h":1});
            e.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone());
            json!({"elements":[e]})
        };
        let good = json!({"unity_name":"Exact","slice":[1,2,3,4],"ppu":100,"sprite":"s0","image_mode":"Sliced","preserve_aspect":true});
        assert!(validate_design(el(good), true).is_ok());
        for bad in [json!({"unity_name":"a/b"}), json!({"unity_name":5}), json!({"slice":[1,2,3]}), json!({"slice":[-1,0,0,0]}), json!({"ppu":0}), json!({"ppu":1e9}), json!({"image_mode":"Nope"}), json!({"preserve_aspect":"yes"})] {
            assert!(validate_design(el(bad.clone()), true).is_err(), "{}", bad);
        }
        let mut d = el(json!({}));
        d["unity"] = json!({"scripts_dir":"Assets/Game"});
        d["canvas"] = json!({"render_mode":"WorldSpace","sort_order":3});
        d["portrait"] = json!({"width":1080,"height":1920,"overrides":[{"id":1,"x":2}]});
        assert!(validate_design(d.clone(), true).is_ok());
        d["unity"] = json!({"scripts_dir":"../Evil"});
        assert!(validate_design(d.clone(), true).is_err());
        d["unity"] = json!({});
        d["canvas"] = json!({"render_mode":"Bogus"});
        assert!(validate_design(d, true).is_err());
    }
}

#[cfg(test)]
mod tests_audit {
    use super::*;
    use serde_json::json;

    fn pkg(manifest: serde_json::Value, files: serde_json::Value) -> Vec<u8> {
        json!({ "manifest": manifest, "files": files }).to_string().into_bytes()
    }

    #[test]
    fn exporter_extension_allowlist() {
        for ok in ["txt", "csv", "json", "xml", "md", "cs", "uxml", "uss", "tsv"] {
            let m = json!({"name":"Exporter","exporters":[{"name":"x","extension":ok,"script":"a.rhai"}]});
            assert!(validate("plugin", "p.json", &pkg(m, json!({"a.rhai":"fn export_text(d){ \"\" }"}))).is_ok(), "{}", ok);
        }
        for bad in ["exe", "bat", "ps1", "js", "svg", "html", "dll", "TXT", ".txt", "", "txt ", "toolong", "t\u{0}xt", "cmd", "abc", "tx t"] {
            let m = json!({"name":"Exporter","exporters":[{"name":"x","extension":bad,"script":"a.rhai"}]});
            assert!(validate("plugin", "p.json", &pkg(m, json!({"a.rhai":"fn export_text(d){ \"\" }"}))).is_err(), "{:?}", bad);
        }
        // a missing extension is not allowed either
        let m = json!({"name":"Exporter","exporters":[{"name":"x","script":"a.rhai"}]});
        assert!(validate("plugin", "p.json", &pkg(m, json!({"a.rhai":"fn export_text(d){ \"\" }"}))).is_err());
    }

    #[test]
    fn json_design_is_validated_even_if_it_is_also_a_command_script() {
        let m = json!({"name":"Sneaky","commands":[{"name":"x","script":"t.json"}]});
        let evil = r#"{"elements":[{"text":"https://evil.example"}]}"#;
        assert!(validate("plugin", "p.json", &pkg(m.clone(), json!({"t.json": evil}))).is_err());
        let m2 = json!({"name":"Sneaky","commands":[{"name":"x","script":"a.rhai"}]});
        let evil2 = r#"{"elements":[{"image":"C:/Users/x/a.png"}]}"#;
        assert!(validate("plugin", "p.json", &pkg(m2, json!({"a.rhai":"fn run(d){d}","other.json": evil2}))).is_err(), "a design-looking json is validated even if not listed");
        let m3 = json!({"name":"Sneaky","templates":["t.json"],"commands":[{"name":"x","script":"t.json"}]});
        assert!(validate("plugin", "p.json", &pkg(m3, json!({"t.json": evil}))).is_err());
    }

    #[test]
    fn drive_relative_and_stream_image_references_are_rejected() {
        for bad in ["C:foo.png", "c:foo.png", "Z:a/b.png", "icons/a.png:stream", "\\\\srv\\share\\a.png", "..", "a/../b.png", "con.png", "icons/", "a.png."] {
            let j = json!({"elements":[{"kind":"Image","image":bad}]}).to_string();
            assert!(validate("template", "t.json", j.as_bytes()).is_err(), "template {:?}", bad);
            let m = json!({"name":"Img","templates":["t.json"],"commands":[{"name":"x","script":"a.rhai"}]});
            assert!(validate("plugin", "p.json", &pkg(m, json!({"a.rhai":"fn run(d){d}","t.json": j}))).is_err(), "plugin template {:?}", bad);
            assert!(check_plugin_output(&json!({"elements":[]}), &json!({"elements":[{"image":bad}]})).is_err(), "plugin output {:?}", bad);
        }
        for ok in ["icons/a.png", "a.png", "My Icons/btn 1.png"] {
            let j = json!({"elements":[{"kind":"Image","image":ok}]}).to_string();
            assert!(validate("template", "t.json", j.as_bytes()).is_ok(), "{:?}", ok);
        }
    }

    #[test]
    fn u2028_in_on_click_is_rejected_everywhere() {
        let j = json!({"elements":[{"kind":"Button","on_click":"a();\u{2028}b();"}]}).to_string();
        assert!(validate("template", "t.json", j.as_bytes()).is_err());
        let m = json!({"name":"Lines","templates":["t.json"],"commands":[{"name":"x","script":"a.rhai"}]});
        assert!(validate("plugin", "p.json", &pkg(m, json!({"a.rhai":"fn run(d){d}","t.json": j.clone()}))).is_err());
        assert!(check_plugin_output(&json!({"elements":[]}), &json!({"elements":[{"on_click":"x\u{2028}y"}]})).is_err());
        // text that was already in the input is kept, a NEW one is not
        assert!(check_plugin_output(&json!({"elements":[{"on_click":"ok"}]}), &json!({"elements":[{"on_click":"ok"},{"on_click":"p\u{2029}q"}]})).is_err());
    }

    #[test]
    fn deeply_nested_documents_are_refused_not_left_unscanned() {
        // Strings nested below 40 levels used to be skipped: a link could hide there.
        let mut v = json!("https://evil.example");
        for _ in 0..100 {
            v = json!([v]);
        }
        let mut t = Vec::new();
        assert!(texts(&v, &mut t).is_ok());
        assert_eq!(t, vec!["https://evil.example".to_string()], "strings at any normal depth are scanned");
        let out = json!({"elements":[{"kind":"Text","text": v}]});
        assert!(check_plugin_output(&json!({"elements":[]}), &out).is_err());
        // far deeper than serde_json can even parse: refused (the walk is iterative, so no stack overflow either)
        let mut deep = json!("x");
        for _ in 0..(MAX_JSON_DEPTH + 10) {
            deep = json!([deep]);
        }
        let mut t = Vec::new();
        assert!(texts(&deep, &mut t).is_err());
        let r = check_plugin_output(&json!({"elements":[]}), &json!({"elements":[{"kind":"Text","extra": deep}]}));
        assert!(r.unwrap_err().contains("nested too deeply"));
    }

    #[test]
    fn case_insensitive_duplicate_files_are_rejected() {
        let m = json!({"name":"Dupes","commands":[{"name":"x","script":"a.rhai"}]});
        let b = pkg(m, json!({"a.rhai":"fn run(d){d}","A.rhai":"fn run(d){d}"}));
        assert!(validate("plugin", "p.json", &b).unwrap_err().contains("Duplicate"));
    }

    #[test]
    fn spoofing_characters_are_rejected_in_names() {
        for c in ['\u{200E}', '\u{200F}', '\u{061C}', '\u{200B}', '\u{200C}', '\u{200D}', '\u{FEFF}', '\u{202E}'] {
            let m = json!({"name": format!("Align{}x", c), "commands":[{"name":"x","script":"a.rhai"}]});
            assert!(validate("plugin", "p.json", &pkg(m, json!({"a.rhai":"fn run(d){d}"}))).is_err(), "{:?}", c);
            assert!(!valid_display_text(&format!("a{}b", c)));
        }
        assert!(valid_display_text("Plugin ñandú 日本語 – ok"));
    }

    #[test]
    fn terminal_text_loses_control_characters() {
        assert_eq!(terminal_safe("ok\u{1b}[31mred\u{7}\r\n\u{9b}x\u{202E}y"), "ok[31mredxy");
        let mut v = json!({"a":["x\u{1b}[2J", {"b":"\u{7}b"}], "n": 5});
        terminal_safe_json(&mut v);
        assert_eq!(v, json!({"a":["x[2J", {"b":"b"}], "n": 5}));
    }

    #[test]
    fn plugin_output_is_checked() {
        let input = json!({"elements":[{"id":1,"text":"Hello","image":"C:/mine/a.png"}]});
        let ok = json!({"elements":[{"id":1,"text":"Hello","image":"C:/mine/a.png"},{"id":2,"text":"New","image":"icons/b.png"}]});
        assert!(check_plugin_output(&input, &ok).is_ok(), "existing absolute refs are kept, safe new ones are fine");
        for bad in [
            json!({"elements":[{"image":"..\\..\\x.png"}]}),
            json!({"elements":[{"image":"C:/Users/x/y.png"}]}),
            json!({"elements":[{"image":"\\\\server\\share\\a.png"}]}),
            json!({"elements":[{"image":"//server/share/a.png"}]}),
            json!({"elements":[{"text":"visit https://evil.example"}]}),
            json!({"elements":[{"text":"a\u{1b}b"}]}),
            json!({"elements":[{"slice":[1,2]}]}),
        ] {
            assert!(check_plugin_output(&input, &bad).is_err(), "{}", bad);
        }
    }
}

#[cfg(test)]
mod tests_round_two {
    use super::*;
    use serde_json::json;

    #[test]
    fn superscript_device_names_are_reserved() {
        for bad in ["COM\u{00B9}", "com\u{00B2}.rhai", "LPT\u{00B3}", "d/LPT\u{00B9}.json", "Com1", "lpt9.txt"] {
            assert!(dangerous_path(bad), "{:?} must be refused", bad);
        }
        for ok in ["com10", "com0.rhai", "lpt", "comx", "COM\u{00B9}\u{00B9}", "console.rhai"] {
            assert!(!dangerous_path(ok), "{:?} is a normal name", ok);
        }
    }

    #[test]
    fn on_click_code_is_scanned_like_a_plugin_script() {
        let design = |code: &str| json!({"elements":[{"id":1,"name":"B","kind":"Button","on_click": code}]});
        assert!(validate_design(design("set_text(\"B\", \"hi\"); hide(\"B\");"), true).is_ok());
        for bad in ["eval(\"x\")", "import \"m\" as m;", "let u = \"https://evil.example\";", "spawn(1)", "a\u{202e}b", "x".repeat(MAX_ON_CLICK + 1).as_str()] {
            assert!(validate_design(design(bad), true).is_err(), "{:?}", &bad[..bad.len().min(30)]);
        }
        // a plugin may not ADD forbidden code, but a script that was already there is left alone
        let input = design("process(1)");
        assert!(check_plugin_output(&input, &input).is_ok());
        assert!(check_plugin_output(&design("hide(\"B\")"), &design("eval(\"x\")")).is_err());
        assert!(check_plugin_output(&design("hide(\"B\")"), &json!({"elements":[{"id":1,"on_click": 5}]})).is_err());
    }
}
