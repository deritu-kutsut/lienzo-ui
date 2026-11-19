//! The user's library: uploaded fonts, images, saved elements and templates.
//! Everything lives in a folder in the user's data directory so it is kept
//! between sessions and shared by all designs.

use crate::model::{Doc, Element, Kind};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct LibImage {
    pub file: String,
    pub w: u32,
    pub h: u32,
}

#[derive(Clone, Debug)]
pub struct LibElement {
    pub name: String,
    pub path: PathBuf,
    pub items: Vec<Element>,
}

#[derive(Clone, Debug)]
pub struct LibTemplate {
    pub name: String,
    pub path: PathBuf,
    pub doc: Doc,
}

#[derive(Default)]
pub struct Library {
    pub images: Vec<LibImage>,
    pub elements: Vec<LibElement>,
    pub templates: Vec<LibTemplate>,
}

pub const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "bmp", "gif", "webp"];
pub const FONT_EXTS: &[&str] = &["ttf", "otf"];

pub fn root_dir() -> PathBuf {
    let base = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
    };
    base.unwrap_or_else(|| PathBuf::from(".")).join("EditorGUI").join("biblioteca")
}

pub fn images_dir() -> PathBuf {
    root_dir().join("imagenes")
}
pub fn fonts_dir() -> PathBuf {
    root_dir().join("fuentes")
}
fn elements_dir() -> PathBuf {
    root_dir().join("elements")
}
fn templates_dir() -> PathBuf {
    root_dir().join("plantillas")
}

/// Folder of the open design: images with a relative path are looked up there (portable designs,
/// no personal machine paths).
pub static DESIGN_FOLDER: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/// The Unity project folder (the one that contains `Assets`): images imported from Unity are
/// stored as `unity:Assets/...[#sub-sprite]` and resolved against it.
pub static UNITY_PROJECT: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

pub fn set_unity_project(dir: &str) {
    if let Ok(mut g) = UNITY_PROJECT.lock() {
        let p = Path::new(dir);
        *g = if dir.trim().is_empty() || !p.join("Assets").is_dir() { None } else { Some(p.to_path_buf()) };
    }
}

/// The Unity project folder that contains `file` (walks up looking for an `Assets` folder next to a `ProjectSettings` one).
pub fn find_unity_project(file: &Path) -> Option<PathBuf> {
    let mut cur = file.parent();
    while let Some(d) = cur {
        if d.join("Assets").is_dir() && d.join("ProjectSettings").is_dir() {
            return Some(d.to_path_buf());
        }
        cur = d.parent();
    }
    None
}

pub fn set_design_folder(file: &Path) {
    if let Ok(mut g) = DESIGN_FOLDER.lock() {
        *g = file.parent().map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()));
    }
}

/// Path of an element's image: absolute, relative to the open design's folder
/// (e.g. «pixel/sword.png») or a file in the user's library.
/// Text of a JSON file without the UTF-8 byte order mark that some editors add (serde_json rejects it).
pub fn strip_bom(s: &str) -> &str {
    s.strip_prefix('\u{feff}').unwrap_or(s)
}

/// True for UNC / device paths (`\\server\share`, `//server/share`, `\\?\`) and for alternate data streams
/// (`a.png:stream`): reading a UNC path can leak the user's NTLM hash to a remote host.
pub fn is_unc_or_stream(name: &str) -> bool {
    let n = name.trim_start();
    n.starts_with("\\\\") || n.starts_with("//") || n.char_indices().any(|(i, c)| c == ':' && i != 1)
}

pub fn resolve_image(name: &str) -> PathBuf {
    if is_unc_or_stream(name) {
        // refused: a path that does not exist (the caller reports a missing image)
        return images_dir().join("__blocked__");
    }
    if let Some(rest) = name.strip_prefix("unity:") {
        // «unity:Assets/UI/frame.png#sub-sprite»: a file inside the Unity project (never outside it).
        let rel = rest.split('#').next().unwrap_or("");
        let safe = !rel.is_empty() && !rel.contains("..") && !Path::new(rel).is_absolute() && rel.starts_with("Assets");
        if let (true, Some(root)) = (safe, UNITY_PROJECT.lock().ok().and_then(|g| g.clone())) {
            return root.join(rel);
        }
        return images_dir().join("__blocked__");
    }
    let p = Path::new(name);
    if p.is_absolute() {
        return p.to_path_buf();
    }
    if let Some(base) = DESIGN_FOLDER.lock().ok().and_then(|g| g.clone()) {
        let candidate = base.join(name);
        if candidate.exists() {
            return candidate;
        }
    }
    images_dir().join(name)
}

fn ext_of(p: &Path) -> String {
    p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase()
}

/// Make a file name safe and unique inside `dir`.
fn unique_name(dir: &Path, stem: &str, ext: &str) -> String {
    let clean: String = stem.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' }).collect();
    let clean = if clean.trim().is_empty() { "archivo".to_string() } else { clean.trim().to_string() };
    let mut name = format!("{}.{}", clean, ext);
    let mut n = 2;
    while dir.join(&name).exists() {
        name = format!("{} ({}).{}", clean, n, ext);
        n += 1;
    }
    name
}

/// Copy a file into a library folder and return its new file name.
fn import_file(src: &Path, dir: &Path) -> Result<String, String> {
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("archivo");
    let name = unique_name(dir, stem, &ext_of(src));
    std::fs::copy(src, dir.join(&name)).map_err(|e| e.to_string())?;
    Ok(name)
}

pub fn image_size(path: &Path) -> Option<(u32, u32)> {
    image::image_dimensions(path).ok()
}

impl Library {
    pub fn load() -> Self {
        let mut lib = Library::default();
        lib.refresh();
        lib
    }

    pub fn refresh(&mut self) {
        self.images.clear();
        self.elements.clear();
        self.templates.clear();
        for p in list(&images_dir()) {
            if IMAGE_EXTS.contains(&ext_of(&p).as_str()) {
                if let Some((w, h)) = image_size(&p) {
                    self.images.push(LibImage { file: p.file_name().unwrap().to_string_lossy().into(), w, h });
                }
            }
        }
        for p in list(&elements_dir()) {
            if ext_of(&p) == "json" {
                if let Some(items) = std::fs::read_to_string(&p).ok().and_then(|s| serde_json::from_str::<Vec<Element>>(strip_bom(&s)).ok()) {
                    self.elements.push(LibElement { name: stem(&p), path: p, items });
                }
            }
        }
        for p in list(&templates_dir()) {
            if ext_of(&p) == "json" {
                if let Some(doc) = std::fs::read_to_string(&p).ok().and_then(|s| Doc::from_json_str(&s).ok()) {
                    self.templates.push(LibTemplate { name: stem(&p), path: p, doc });
                }
            }
        }
    }

    /// Copy an image into the library. Returns its file name.
    pub fn add_image(&mut self, src: &Path) -> Result<String, String> {
        if !IMAGE_EXTS.contains(&ext_of(src).as_str()) {
            return Err("unsupported image format".into());
        }
        image_size(src).ok_or("could not read the image")?;
        let name = import_file(src, &images_dir())?;
        self.refresh();
        Ok(name)
    }

    /// Upload an element: an image file becomes an image element, a .json file is a saved element.
    pub fn add_element_file(&mut self, src: &Path) -> Result<String, String> {
        let ext = ext_of(src);
        if ext == "json" {
            let text = std::fs::read_to_string(src).map_err(|e| e.to_string())?;
            let text = strip_bom(&text).to_string();
            let items: Vec<Element> = serde_json::from_str::<Vec<Element>>(&text)
                .or_else(|_| serde_json::from_str::<Element>(&text).map(|e| vec![e]))
                .map_err(|_| "the file does not contain elements from this editor".to_string())?;
            return self.save_element(&stem(src), &items);
        }
        let file = self.add_image(src)?;
        let (w, h) = image_size(&images_dir().join(&file)).unwrap_or((256, 256));
        let el = image_element(&file, w, h, 300.0);
        self.save_element(&stem(src), &[el])
    }

    /// Save elements (e.g. the current selection) as a reusable library element.
    pub fn save_element(&mut self, name: &str, items: &[Element]) -> Result<String, String> {
        if items.is_empty() {
            return Err("nothing is selected".into());
        }
        let ids: Vec<u64> = items.iter().map(|e| e.id).collect();
        // normalise: top-left of the group at (0,0)
        let min_x = items.iter().map(|e| e.aabb()[0]).fold(f32::MAX, f32::min);
        let min_y = items.iter().map(|e| e.aabb()[1]).fold(f32::MAX, f32::min);
        let items: Vec<Element> = items
            .iter()
            .cloned()
            .map(|mut e| {
                e.x -= min_x;
                e.y -= min_y;
                e.locked = false;
                if e.parent.map(|p| !ids.contains(&p)).unwrap_or(false) {
                    e.parent = None;
                }
                e
            })
            .collect();
        let dir = elements_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let file = unique_name(&dir, name, "json");
        let json = serde_json::to_string_pretty(&items).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(&file), json).map_err(|e| e.to_string())?;
        self.refresh();
        Ok(file)
    }

    pub fn save_template(&mut self, name: &str, doc: &Doc) -> Result<String, String> {
        let dir = templates_dir();
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        let file = unique_name(&dir, name, "json");
        let mut d = doc.clone();
        d.title = name.to_string();
        let json = serde_json::to_string_pretty(&d).map_err(|e| e.to_string())?;
        std::fs::write(dir.join(&file), json).map_err(|e| e.to_string())?;
        self.refresh();
        Ok(file)
    }

    /// Upload a template: a project/template .json saved by this editor.
    pub fn add_template_file(&mut self, src: &Path) -> Result<String, String> {
        let text = std::fs::read_to_string(src).map_err(|e| e.to_string())?;
        let text = strip_bom(&text);
        let doc = Doc::from_json_str(text).map_err(|_| "the file is not a design from this editor".to_string())?;
        self.save_template(&stem(src), &doc)
    }

    pub fn remove(&mut self, path: &Path) {
        let _ = std::fs::remove_file(path);
        self.refresh();
    }
}

/// Copy a font file into the library. Returns the font's display name (file stem), bytes and library path.
pub fn add_font_file(src: &Path) -> Result<(String, Vec<u8>, PathBuf), String> {
    if !FONT_EXTS.contains(&ext_of(src).as_str()) {
        return Err("use a .ttf or .otf font file".into());
    }
    let bytes = std::fs::read(src).map_err(|e| e.to_string())?;
    let dir = fonts_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut dest = dir.join(src.file_name().ok_or("invalid name")?);
    // Another font with the same file name: stored separately (previously the new one was used this session and the old one after a restart).
    if dest.exists() && std::fs::read(&dest).map(|b| b != bytes).unwrap_or(true) {
        dest = dir.join(unique_name(&dir, &stem(src), &ext_of(src)));
    }
    if !dest.exists() {
        std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
    }
    Ok((stem(&dest), bytes, dest))
}

/// Fonts previously uploaded to the library: (name, path, bytes).
pub fn library_fonts() -> Vec<(String, PathBuf, Vec<u8>)> {
    list(&fonts_dir())
        .into_iter()
        .filter(|p| FONT_EXTS.contains(&ext_of(p).as_str()))
        .filter_map(|p| std::fs::read(&p).ok().map(|b| (stem(&p), p.clone(), b)))
        .collect()
}

/// Editor preferences kept between sessions (`ajustes.json`; files written by older versions, with Spanish keys, still load through the aliases).
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Last Unity project the design was exported to.
    pub unity_project: String,
    pub last_project_file: String,
    /// The tutorial has already been shown (it does not open by itself on startup).
    #[serde(alias = "tutorial_visto")]
    pub tutorial_seen: bool,
    /// Accessibility: interface language («es», «en»…), size (1.0 = 100 %), high contrast and reduced motion.
    #[serde(alias = "idioma")]
    pub language: String,
    #[serde(alias = "escala_ui")]
    pub ui_scale: f32,
    #[serde(alias = "alto_contraste")]
    pub high_contrast: bool,
    #[serde(alias = "reducir_movimiento")]
    pub reduce_motion: bool,
    /// Updates: do not check on startup, and the version the user chose to skip.
    #[serde(alias = "no_buscar_actualizaciones")]
    pub skip_update_check: bool,
    #[serde(alias = "version_omitida")]
    pub skipped_version: String,
    /// Interface theme (None = the default one).
    #[serde(alias = "tema")]
    pub theme: Option<crate::theme::Theme>,
    /// Automated session (screenshots, tests): nothing is written to disk.
    #[serde(skip)]
    pub read_only: bool,
}

fn settings_path() -> PathBuf {
    root_dir().parent().map(|p| p.join("ajustes.json")).unwrap_or_else(|| PathBuf::from("ajustes.json"))
}

impl Settings {
    pub fn load() -> Self {
        let mut s: Self = std::fs::read_to_string(settings_path()).ok().and_then(|s| serde_json::from_str(strip_bom(&s)).ok()).unwrap_or_default();
        if let Some(t) = &mut s.theme {
            t.name = crate::theme::canonical_name(&t.name).to_string(); // old Spanish names of the bundled themes
        }
        s
    }
    pub fn save(&self) {
        if self.read_only {
            return;
        }
        let p = settings_path();
        if let Some(d) = p.parent() {
            let _ = std::fs::create_dir_all(d);
        }
        if let Ok(s) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(p, s);
        }
    }
}

pub fn image_element(file: &str, w: u32, h: u32, max_side: f32) -> Element {
    let mut e = Element::new(0, Kind::Image);
    let s = max_side / (w.max(h).max(1) as f32);
    e.w = (w as f32 * s).max(8.0);
    e.h = (h as f32 * s).max(8.0);
    e.radius = 0.0;
    e.image = file.to_string();
    e.name = Path::new(file).file_stem().and_then(|s| s.to_str()).unwrap_or("Image").to_string();
    e
}

fn stem(p: &Path) -> String {
    p.file_stem().and_then(|s| s.to_str()).unwrap_or("unnamed").to_string()
}

fn list(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir).map(|rd| rd.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_file()).collect()).unwrap_or_default();
    v.sort();
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_round_trip() {
        let dir = std::env::temp_dir().join(format!("editorgui-test-{}", std::process::id()));
        // keep the test away from the real library
        unsafe {
            std::env::set_var("APPDATA", &dir);
            std::env::set_var("XDG_DATA_HOME", &dir);
            std::env::set_var("HOME", &dir);
        }
        let mut lib = Library::load();
        // image upload
        let png = dir.join("foto.png");
        std::fs::create_dir_all(&dir).unwrap();
        image::RgbaImage::from_pixel(40, 20, image::Rgba([255, 0, 0, 255])).save(&png).unwrap();
        let name = lib.add_image(&png).unwrap();
        assert_eq!(lib.images.len(), 1);
        assert_eq!((lib.images[0].w, lib.images[0].h), (40, 20));
        // uploading an image as an element creates a saved element
        lib.add_element_file(&png).unwrap();
        assert_eq!(lib.elements.len(), 1);
        let el_img = lib.elements[0].items[0].image.clone();
        assert_eq!(lib.images.len(), 2);
        assert_ne!(el_img, name);
        assert!(lib.images.iter().any(|i| i.file == el_img));
        // templates
        let doc = crate::model::template(1, 1920.0, 1080.0);
        lib.save_template("Mi HUD", &doc).unwrap();
        assert_eq!(lib.templates.len(), 1);
        assert_eq!(lib.templates[0].doc.elements.len(), doc.elements.len());
        let tpl_path = lib.templates[0].path.clone();
        lib.add_template_file(&tpl_path).unwrap();
        assert_eq!(lib.templates.len(), 2);
        lib.remove(&tpl_path);
        assert_eq!(lib.templates.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn old_spanish_settings_keys_still_load() {
        let old = r#"{ "unity_project": "C:/u", "tutorial_visto": true, "idioma": "en", "escala_ui": 1.5, "alto_contraste": true,
                       "reducir_movimiento": true, "no_buscar_actualizaciones": true, "version_omitida": "1.2.0",
                       "tema": { "nombre": "Mi tema", "claro": true } }"#;
        let s: Settings = serde_json::from_str(old).expect("settings with Spanish keys must load");
        assert_eq!((s.tutorial_seen, s.language.as_str(), s.ui_scale, s.high_contrast, s.reduce_motion), (true, "en", 1.5, true, true));
        assert_eq!((s.skip_update_check, s.skipped_version.as_str()), (true, "1.2.0"));
        let t = s.theme.expect("the old `tema` key");
        assert_eq!((t.name.as_str(), t.light), ("Mi tema", true));
        // what is saved now uses the English keys
        let out = serde_json::to_string(&Settings::default()).unwrap();
        assert!(out.contains("\"language\"") && !out.contains("idioma"));
    }
}