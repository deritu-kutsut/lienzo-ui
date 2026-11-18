//! Validation of what the community downloads and uploads. These are the same rules the server applies
//! (`supabase/functions/_compartido/validar.ts`): the app NEVER trusts what it downloads and re-checks it.
//! Data formats only; never executables, installers, system scripts, archives or SVG.

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const TIPOS: &[&str] = &["plugin", "imagen", "fuente", "plantilla", "elemento", "tema"];

pub fn limite_bytes(tipo: &str) -> usize {
    match tipo {
        "imagen" => 2 * 1024 * 1024,
        "fuente" => 5 * 1024 * 1024,
        "plantilla" => 512 * 1024,
        "tema" => 32 * 1024,
        _ => 256 * 1024, // element, plugin
    }
}

const EXT_PROHIBIDAS: &[&str] = &[
    "exe", "dll", "bat", "cmd", "com", "scr", "msi", "ps1", "psm1", "vbs", "vbe", "js", "jse", "mjs", "cjs", "wsf", "jar", "apk", "dmg", "app", "sh", "bash", "lnk", "reg", "hta", "cpl", "so", "dylib", "py", "rb", "php", "zip",
    "rar", "7z", "gz", "tar", "iso", "svg", "html", "htm",
];

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|b| format!("{:02x}", b)).collect()
}

fn ext(nombre: &str) -> String {
    nombre.rsplit_once('.').map(|(_, e)| e.to_lowercase()).unwrap_or_default()
}

/// Actual format from the first bytes (not from the extension).
pub fn detectar(b: &[u8]) -> Option<&'static str> {
    let empieza = |f: &[u8], d: usize| b.len() >= d + f.len() && &b[d..d + f.len()] == f;
    if empieza(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a], 0) {
        Some("png")
    } else if empieza(&[0xff, 0xd8, 0xff], 0) {
        Some("jpeg")
    } else if empieza(b"GIF8", 0) {
        Some("gif")
    } else if empieza(b"RIFF", 0) && empieza(b"WEBP", 8) {
        Some("webp")
    } else if empieza(&[0, 1, 0, 0], 0) || empieza(b"true", 0) {
        Some("ttf")
    } else if empieza(b"OTTO", 0) {
        Some("otf")
    } else if empieza(b"MZ", 0) {
        Some("ejecutable-windows")
    } else if empieza(&[0x7f, b'E', b'L', b'F'], 0) {
        Some("ejecutable-linux")
    } else if empieza(&[0x50, 0x4b, 0x03, 0x04], 0) {
        Some("zip")
    } else if empieza(&[0xcf, 0xfa, 0xed, 0xfe], 0) || empieza(&[0xca, 0xfe, 0xba, 0xbe], 0) {
        Some("ejecutable-mac")
    } else if empieza(b"#!", 0) {
        Some("script-shebang")
    } else {
        None
    }
}

fn ruta_peligrosa(s: &str) -> bool {
    let b = s.as_bytes();
    s.contains('\0') || s.starts_with('/') || s.starts_with('\\') || (b.len() >= 2 && b[1] == b':') || s.split(['/', '\\']).any(|p| p == "..")
}

fn textos(v: &serde_json::Value, acc: &mut Vec<String>, prof: u32) {
    if prof > 40 {
        return;
    }
    match v {
        serde_json::Value::String(s) => acc.push(s.clone()),
        serde_json::Value::Array(a) => a.iter().for_each(|x| textos(x, acc, prof + 1)),
        serde_json::Value::Object(o) => o.values().for_each(|x| textos(x, acc, prof + 1)),
        _ => {}
    }
}

fn url_externa(s: &str) -> bool {
    let l = s.to_lowercase();
    ["http:", "https:", "ftp:", "file:", "data:", "javascript:"].iter().any(|p| l.contains(p))
}

fn validar_diseno(json: serde_json::Value, es_plantilla: bool) -> Result<(), String> {
    let json = if !es_plantilla && (json.is_array() || json.is_object()) {
        // a library element is a list of elements (or a single one)
        let lista = if json.is_array() { json } else { serde_json::Value::Array(vec![json]) };
        serde_json::json!({ "elements": lista })
    } else {
        json
    };
    let elementos = json.get("elements").and_then(|e| e.as_array()).ok_or("Falta la lista «elements» del diseño.")?;
    if elementos.len() > if es_plantilla { 2000 } else { 500 } {
        return Err("Demasiados elementos.".into());
    }
    let mut t = Vec::new();
    textos(&serde_json::Value::Array(elementos.clone()), &mut t, 0);
    for s in &t {
        if url_externa(s) {
            return Err("Los diseños no pueden llevar enlaces ni rutas externas.".into());
        }
        if ruta_peligrosa(s) && (s.contains('/') || s.contains('\\')) {
            return Err("Los diseños no pueden llevar rutas absolutas o con «..».".into());
        }
    }
    Ok(())
}

/// A plugin travels as a single JSON: { manifiesto, archivos: { "ruta.rhai": "código" } }.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Paquete {
    pub manifiesto: serde_json::Value,
    pub archivos: BTreeMap<String, String>,
}

fn validar_plugin(json: serde_json::Value) -> Result<Paquete, String> {
    let p: Paquete = serde_json::from_value(json).map_err(|_| "Un plugin necesita «manifiesto» y «archivos».")?;
    let nombre = p.manifiesto.get("nombre").and_then(|n| n.as_str()).unwrap_or("");
    if nombre.chars().count() < 3 || nombre.chars().count() > 60 {
        return Err("Manifiesto: nombre inválido.".into());
    }
    if p.archivos.is_empty() || p.archivos.len() > 20 {
        return Err("Un plugin lleva entre 1 y 20 archivos.".into());
    }
    for (ruta, contenido) in &p.archivos {
        if ruta_peligrosa(ruta) || ruta.len() > 80 {
            return Err(format!("Ruta no permitida: {}", ruta));
        }
        let e = ext(ruta);
        if e != "rhai" && e != "json" {
            return Err(format!("Solo se admiten .rhai y .json ({}).", ruta));
        }
        if e == "rhai" && sospechoso(contenido) {
            return Err(format!("{}: contiene instrucciones no permitidas (eval, import, red, procesos).", ruta));
        }
    }
    for clave in ["comandos", "exportadores"] {
        for x in p.manifiesto.get(clave).and_then(|v| v.as_array()).into_iter().flatten() {
            if let Some(s) = x.get("script").and_then(|s| s.as_str()) {
                if !p.archivos.contains_key(s) {
                    return Err(format!("El manifiesto usa {}, que no está en el paquete.", s));
                }
            }
        }
    }
    Ok(p)
}

fn sospechoso(codigo: &str) -> bool {
    let l = codigo.to_lowercase();
    if l.contains("http://") || l.contains("https://") || l.contains("std::") || l.contains("os::") || l.contains(".exe") {
        return true;
    }
    // whole words: eval, import "…", Engine, process, Command, spawn
    let es_palabra = |w: &str| {
        let mut desde = 0;
        while let Some(i) = l[desde..].find(w) {
            let a = desde + i;
            let antes = l[..a].chars().next_back().is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
            let despues = l[a + w.len()..].chars().next().is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
            if antes && despues {
                return true;
            }
            desde = a + w.len();
        }
        false
    };
    es_palabra("eval") || es_palabra("engine") || es_palabra("process") || es_palabra("command") || es_palabra("spawn") || l.contains("import \"") || l.contains("import '")
}

/// Validates a file according to its type. `Ok(())` = safe to install or upload.
pub fn validar(tipo: &str, nombre: &str, bytes: &[u8]) -> Result<(), String> {
    if !TIPOS.contains(&tipo) {
        return Err("Tipo de contenido desconocido.".into());
    }
    if bytes.is_empty() {
        return Err("Archivo vacío.".into());
    }
    if bytes.len() > limite_bytes(tipo) {
        return Err(format!("Supera el límite de {} KB para {}.", limite_bytes(tipo) / 1024, tipo));
    }
    if EXT_PROHIBIDAS.contains(&ext(nombre).as_str()) {
        return Err("Tipo de archivo prohibido.".into());
    }
    let real = detectar(bytes);
    if let Some(r) = real {
        if r.starts_with("ejecutable") || r == "zip" || r.starts_with("script") {
            return Err(format!("Contenido peligroso detectado ({}).", r));
        }
    }
    match tipo {
        "imagen" => {
            let r = real.filter(|r| ["png", "jpeg", "gif", "webp"].contains(r)).ok_or("No es una imagen PNG, JPEG, GIF ni WebP válida.")?;
            let (w, h) = match r {
                "png" if bytes.len() >= 24 => (u32::from_be_bytes(bytes[16..20].try_into().unwrap()), u32::from_be_bytes(bytes[20..24].try_into().unwrap())),
                "gif" if bytes.len() >= 10 => (u16::from_le_bytes([bytes[6], bytes[7]]) as u32, u16::from_le_bytes([bytes[8], bytes[9]]) as u32),
                _ => (1, 1),
            };
            if w == 0 || h == 0 || w > 4096 || h > 4096 {
                return Err("Dimensiones fuera de rango (máximo 4096×4096).".into());
            }
            Ok(())
        }
        "fuente" => match real {
            Some("ttf") | Some("otf") => Ok(()),
            _ => Err("Solo se admiten fuentes TTF u OTF.".into()),
        },
        _ => {
            if real.is_some() {
                return Err("Se esperaba un JSON de texto.".into());
            }
            let texto = std::str::from_utf8(bytes).map_err(|_| "El archivo no es JSON UTF-8 válido.")?;
            let json: serde_json::Value = serde_json::from_str(texto).map_err(|_| "El archivo no es JSON válido.")?;
            if tipo == "tema" {
                // Only the known theme fields: #RRGGBB colors, corners, light base and a font name.
                let t: crate::theme::Tema = serde_json::from_value(json).map_err(|_| "El archivo no es un tema de Lienzo UI.")?;
                t.validar()
            } else if tipo == "plugin" {
                validar_plugin(json).map(|_| ())
            } else {
                validar_diseno(json, tipo == "plantilla")
            }
        }
    }
}

/// Reads an already-validated downloaded plugin.
pub fn leer_paquete(bytes: &[u8]) -> Result<Paquete, String> {
    validar("plugin", "plugin.json", bytes)?;
    let json: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    validar_plugin(json)
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
    fn imagen_valida_y_enorme() {
        assert!(validar("imagen", "a.png", &png(16, 16)).is_ok());
        assert!(validar("imagen", "a.png", &png(9000, 10)).is_err());
    }

    #[test]
    fn ejecutable_disfrazado_de_imagen() {
        let mut exe = vec![0u8; 64];
        exe[..2].copy_from_slice(b"MZ");
        assert!(validar("imagen", "foto.png", &exe).unwrap_err().contains("peligroso"));
        for n in ["x.exe", "x.dll", "x.js", "x.svg", "x.zip", "x.ps1"] {
            assert!(validar("imagen", n, &png(8, 8)).is_err(), "{}", n);
        }
    }

    #[test]
    fn zip_con_nombre_json() {
        let mut z = vec![0u8; 32];
        z[..4].copy_from_slice(&[0x50, 0x4b, 3, 4]);
        assert!(validar("plantilla", "a.json", &z).is_err());
    }

    #[test]
    fn fuentes() {
        let mut ttf = vec![0u8; 40];
        ttf[..4].copy_from_slice(&[0, 1, 0, 0]);
        assert!(validar("fuente", "f.ttf", &ttf).is_ok());
        assert!(validar("fuente", "f.ttf", &png(8, 8)).is_err());
    }

    #[test]
    fn plantillas_y_elementos() {
        let ok = br#"{"elements":[{"kind":"Text","text":"Hola mundo"}]}"#;
        assert!(validar("plantilla", "t.json", ok).is_ok());
        assert!(validar("plantilla", "t.json", br#"{"elements":[{"text":"visita https://malo.example"}]}"#).is_err());
        assert!(validar("plantilla", "t.json", br#"{"elements":[{"image":"C:/Users/yo/foto.png"}]}"#).is_err());
        assert!(validar("plantilla", "t.json", br#"{"elements":[{"image":"../../secreto.png"}]}"#).is_err());
        assert!(validar("plantilla", "t.json", b"{no").is_err());
        assert!(validar("elemento", "e.json", br#"[{"kind":"Text","text":"Hola"}]"#).is_ok());
        assert!(validar("elemento", "e.json", br#"{"kind":"Text","text":"Hola"}"#).is_ok());
        assert!(validar("elemento", "e.json", br#"[{"text":"https://malo.example"}]"#).is_err());
    }

    fn plugin(archivos: &str) -> Vec<u8> {
        format!(r#"{{"manifiesto":{{"nombre":"Ajustar","comandos":[{{"nombre":"x","script":"a.rhai"}}]}},"archivos":{}}}"#, archivos).into_bytes()
    }

    #[test]
    fn plugins() {
        assert!(validar("plugin", "p.json", &plugin(r#"{"a.rhai":"fn ejecutar(d){ d }"}"#)).is_ok());
        for malo in [
            r#"{"../a.rhai":"x"}"#,
            r#"{"/etc/a.rhai":"x"}"#,
            r#"{"a.rhai":"x","m.exe":"MZ"}"#,
            r#"{"b.rhai":"x"}"#,
            r#"{"a.rhai":"eval(\"x\")"}"#,
            r#"{"a.rhai":"import \"mod\" as m;"}"#,
            r#"{"a.rhai":"let u = \"https://malo.example\";"}"#,
            r#"{"a.rhai":"Command::new"}"#,
        ] {
            assert!(validar("plugin", "p.json", &plugin(malo)).is_err(), "{}", malo);
        }
        // "evaluar" or "processing" are not "eval" or "process"
        assert!(validar("plugin", "p.json", &plugin(r#"{"a.rhai":"let evaluar = 1; let processing = 2; fn ejecutar(d){ d }"}"#)).is_ok());
    }

    #[test]
    fn los_plugins_de_ejemplo_se_pueden_empaquetar() {
        // the upload format must accept what the repository already ships
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("plugins/ajustar-cuadricula");
        let mut archivos = BTreeMap::new();
        for f in ["plugin.json", "ajustar.rhai"] {
            archivos.insert(f.to_string(), std::fs::read_to_string(base.join(f)).unwrap());
        }
        let m: serde_json::Value = serde_json::from_str(&archivos["plugin.json"]).unwrap();
        let p = Paquete { manifiesto: m, archivos };
        assert!(validar("plugin", "p.json", &serde_json::to_vec(&p).unwrap()).is_ok());
    }

    #[test]
    fn hash_conocido() {
        assert_eq!(sha256_hex(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}

/// Auxiliary images (icon and preview of a content item, profile avatar and banner): PNG only, with limits.
/// Same rules as the server's `validarPng`. Returns (width, height).
pub fn validar_png(clase: &str, bytes: &[u8]) -> Result<(u32, u32), String> {
    let (max_bytes, max_w, max_h) = match clase {
        "icono" => (256 * 1024, 512, 512),
        "avatar" => (512 * 1024, 512, 512),
        "vista" => (1024 * 1024, 1920, 1080),
        "banner" => (1536 * 1024, 1920, 640),
        _ => return Err("Tipo de imagen desconocido.".into()),
    };
    if bytes.is_empty() || bytes.len() > max_bytes {
        return Err(format!("La imagen debe pesar como máximo {} KB.", max_bytes / 1024));
    }
    if detectar(bytes) != Some("png") || bytes.len() < 24 {
        return Err("La imagen debe ser PNG.".into());
    }
    let w = u32::from_be_bytes(bytes[16..20].try_into().unwrap());
    let h = u32::from_be_bytes(bytes[20..24].try_into().unwrap());
    if w == 0 || h == 0 || w > max_w || h > max_h {
        return Err(format!("La imagen debe medir como máximo {}×{} px.", max_w, max_h));
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
    fn imagenes_auxiliares() {
        assert!(validar_png("icono", &png(256, 256)).is_ok());
        assert!(validar_png("icono", &png(1024, 1024)).is_err());
        assert!(validar_png("banner", &png(1500, 500)).is_ok());
        assert!(validar_png("banner", &png(1500, 900)).is_err());
        assert!(validar_png("avatar", &[0xff, 0xd8, 0xff, 0, 0, 0]).is_err());
    }
}
