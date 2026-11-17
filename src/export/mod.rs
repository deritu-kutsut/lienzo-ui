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

/// PascalCase C# identifier ("Menú principal" → "MenuPrincipal").
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
        out = "Diseno".into();
    }
    if out.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        out = format!("Ui{}", out);
    }
    // Sirve de nombre de archivo y de clase: un título larguísimo no debe pasar del límite de rutas de Windows.
    out.chars().take(64).collect()
}

/// camelCase C# identifier.
#[cfg_attr(not(test), allow(dead_code))]
pub fn ident(s: &str) -> String {
    let p = class_name(s);
    let mut c = p.chars();
    match c.next() {
        Some(f) => f.to_ascii_lowercase().to_string() + c.as_str(),
        None => "elemento".into(),
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

/// C# string literal.
pub fn cs_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '\\' => o.push_str("\\\\"),
            '"' => o.push_str("\\\""),
            '\n' => o.push_str("\\n"),
            '\r' => {}
            '\t' => o.push_str("\\t"),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

/// XML attribute value.
pub fn xml_attr(s: &str) -> String {
    s.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;").replace('>', "&gt;").replace('\n', "&#10;")
}

pub fn f(v: f32) -> String {
    let r = (v * 100.0).round() / 100.0;
    if r == r.trunc() { format!("{}", r as i64) } else { format!("{}", r) }
}

/// C# float literal.
pub fn cf(v: f32) -> String {
    format!("{}f", f(v))
}

/// Unique GameObject names among siblings (Unity's `Transform.Find` uses names; '/' is not allowed).
pub fn unity_names(doc: &Doc) -> HashMap<u64, String> {
    let mut out = HashMap::new();
    let mut used: HashMap<Option<u64>, HashSet<String>> = HashMap::new();
    for e in &doc.elements {
        let base = {
            let n = e.name.replace('/', "-").trim().to_string();
            if n.is_empty() { e.kind.label().to_string() } else { n }
        };
        let set = used.entry(doc.parent_of(e.id)).or_default();
        let mut name = base.clone();
        let mut k = 2;
        while set.contains(&name) {
            name = format!("{} ({})", base, k);
            k += 1;
        }
        set.insert(name.clone());
        out.insert(e.id, name);
    }
    out
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
    // Con ancho o alto negativos o no numéricos (archivos ajenos) `clamp` entraría en pánico.
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
        if !e.font.is_empty() && e.font != "Monoespaciada" && !v.contains(&e.font) {
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

/// Write everything into `<project>/Assets/EditorGUI`. Returns the written paths (relative to the project).
pub fn write_unity_package(doc: &Doc, project: &Path) -> Result<Vec<String>, String> {
    let root = find_unity_root(project).ok_or("esa carpeta no parece un proyecto de Unity (no tiene carpeta Assets)")?;
    let base = root.join(UNITY_ROOT);
    let cls = class_name(&doc.title);
    // Las imágenes se leen antes de escribir nada: si falta una, el proyecto de Unity no queda a medias.
    let mut pictures = Vec::new();
    let mut done = HashSet::new();
    for e in &doc.elements {
        if !e.image.is_empty() && done.insert(e.image.clone()) {
            let src = crate::library::resolve_image(&e.image);
            let b = std::fs::read(&src).map_err(|err| format!("no se pudo leer la imagen {}: {}", src.display(), err))?;
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
    put(format!("{}/Scripts/{}ViewBase.cs", UNITY_ROOT, cls), view::base_script(doc).as_bytes())?;
    let user_view = format!("{}/Scripts/{}View.cs", UNITY_ROOT, cls);
    if !root.join(&user_view).exists() {
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
        assert_eq!(class_name("Menú principal"), "MenuPrincipal");
        assert_eq!(class_name("3 vidas"), "Ui3Vidas");
        assert_eq!(ident("Botón Jugar"), "botonJugar");
        assert_eq!(cs_str("a\"b\nc"), "\"a\\\"b\\nc\"");
    }

    #[test]
    fn names_and_paths() {
        let d = model::template(3, 1920.0, 1080.0);
        let names = unity_names(&d);
        let slider = d.elements.iter().find(|e| e.name == "Slider Brillo").unwrap();
        assert_eq!(unity_path(&d, &names, slider.id), "Tarjeta/Opciones/Viewport/Content/Slider Brillo");
    }

    #[test]
    fn package_is_written() {
        let dir = std::env::temp_dir().join(format!("editorgui-unity-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("Assets")).unwrap();
        std::fs::create_dir_all(dir.join("ProjectSettings")).unwrap();
        let d = model::template(0, 1920.0, 1080.0);
        let files = write_unity_package(&d, &dir.join("Assets")).unwrap();
        assert!(files.iter().any(|f| f.ends_with("Designs/MenuPrincipal.json")));
        assert!(files.iter().any(|f| f.ends_with("Scripts/MenuPrincipalView.cs")));
        // the user's View script is never overwritten
        let again = write_unity_package(&d, &dir).unwrap();
        assert!(!again.iter().any(|f| f.ends_with("Scripts/MenuPrincipalView.cs")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod dump {
    /// `EDITORGUI_DUMP=<carpeta> cargo test dump_templates -- --ignored` writes the
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
mod robustez_tests {
    use super::*;

    #[test]
    fn radio_con_tamanos_raros_no_entra_en_panico() {
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
    fn nombre_de_clase_largo_se_acorta() {
        assert!(class_name(&"x".repeat(500)).chars().count() <= 64);
        assert_eq!(class_name("Menú principal"), "MenuPrincipal");
    }
}
