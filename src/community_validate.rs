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

fn dangerous_path(s: &str) -> bool {
    let b = s.as_bytes();
    s.contains('\0') || s.starts_with('/') || s.starts_with('\\') || (b.len() >= 2 && b[1] == b':') || s.split(['/', '\\']).any(|p| p == "..")
}

fn texts(v: &serde_json::Value, acc: &mut Vec<String>, depth: u32) {
    if depth > 40 {
        return;
    }
    match v {
        serde_json::Value::String(s) => acc.push(s.clone()),
        serde_json::Value::Array(a) => a.iter().for_each(|x| texts(x, acc, depth + 1)),
        serde_json::Value::Object(o) => o.values().for_each(|x| texts(x, acc, depth + 1)),
        _ => {}
    }
}

fn external_url(s: &str) -> bool {
    let l = s.to_lowercase();
    ["http:", "https:", "ftp:", "file:", "data:", "javascript:"].iter().any(|p| l.contains(p))
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
    let mut t = Vec::new();
    texts(&serde_json::Value::Array(elements.clone()), &mut t, 0);
    for s in &t {
        if external_url(s) {
            return Err("Designs cannot contain external links or paths.".into());
        }
        if dangerous_path(s) && (s.contains('/') || s.contains('\\')) {
            return Err("Designs cannot contain absolute paths or paths with “..”.".into());
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
    if p.files.is_empty() || p.files.len() > 20 {
        return Err("A plugin has between 1 and 20 files.".into());
    }
    for (path, content) in &p.files {
        if dangerous_path(path) || path.len() > 80 {
            return Err(format!("Path not allowed: {}", path));
        }
        let e = ext(path);
        if e != "rhai" && e != "json" {
            return Err(format!("Only .rhai and .json are allowed ({}).", path));
        }
        if e == "rhai" && suspicious(content) {
            return Err(format!("{}: contains forbidden instructions (eval, import, network, processes).", path));
        }
    }
    for (new, old) in [("commands", "comandos"), ("exporters", "exportadores")] {
        for x in crate::plugins::manifest_key(&p.manifest, new, old).and_then(|v| v.as_array()).into_iter().flatten() {
            if let Some(s) = x.get("script").and_then(|s| s.as_str()) {
                if !p.files.contains_key(s) {
                    return Err(format!("The manifest uses {}, which is not in the package.", s));
                }
            }
        }
    }
    Ok(p)
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
