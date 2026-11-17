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
    root_dir().join("elementos")
}
fn templates_dir() -> PathBuf {
    root_dir().join("plantillas")
}

/// Carpeta del diseño abierto: las imágenes con ruta relativa se buscan ahí (diseños portables,
/// sin rutas personales del equipo).
pub static CARPETA_DISENO: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

pub fn set_carpeta_diseno(archivo: &Path) {
    if let Ok(mut g) = CARPETA_DISENO.lock() {
        *g = archivo.parent().map(|p| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf()));
    }
}

/// Ruta de la imagen de un elemento: absoluta, relativa a la carpeta del diseño abierto
/// (por ejemplo «pixel/espada.png») o un archivo de la biblioteca del usuario.
pub fn resolve_image(name: &str) -> PathBuf {
    let p = Path::new(name);
    if p.is_absolute() {
        return p.to_path_buf();
    }
    if let Some(base) = CARPETA_DISENO.lock().ok().and_then(|g| g.clone()) {
        let candidata = base.join(name);
        if candidata.exists() {
            return candidata;
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
                if let Some(items) = std::fs::read_to_string(&p).ok().and_then(|s| serde_json::from_str::<Vec<Element>>(&s).ok()) {
                    self.elements.push(LibElement { name: stem(&p), path: p, items });
                }
            }
        }
        for p in list(&templates_dir()) {
            if ext_of(&p) == "json" {
                if let Some(mut doc) = std::fs::read_to_string(&p).ok().and_then(|s| serde_json::from_str::<Doc>(&s).ok()) {
                    doc.migrate_groups();
                    self.templates.push(LibTemplate { name: stem(&p), path: p, doc });
                }
            }
        }
    }

    /// Copy an image into the library. Returns its file name.
    pub fn add_image(&mut self, src: &Path) -> Result<String, String> {
        if !IMAGE_EXTS.contains(&ext_of(src).as_str()) {
            return Err("formato de imagen no compatible".into());
        }
        image_size(src).ok_or("no se pudo leer la imagen")?;
        let name = import_file(src, &images_dir())?;
        self.refresh();
        Ok(name)
    }

    /// Upload an element: an image file becomes an image element, a .json file is a saved element.
    pub fn add_element_file(&mut self, src: &Path) -> Result<String, String> {
        let ext = ext_of(src);
        if ext == "json" {
            let text = std::fs::read_to_string(src).map_err(|e| e.to_string())?;
            let items: Vec<Element> = serde_json::from_str::<Vec<Element>>(&text)
                .or_else(|_| serde_json::from_str::<Element>(&text).map(|e| vec![e]))
                .map_err(|_| "el archivo no contiene elementos de este editor".to_string())?;
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
            return Err("no hay nada seleccionado".into());
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
        let doc: Doc = serde_json::from_str(&text).map_err(|_| "el archivo no es un diseño de este editor".to_string())?;
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
        return Err("usa un archivo de fuente .ttf o .otf".into());
    }
    let bytes = std::fs::read(src).map_err(|e| e.to_string())?;
    let dir = fonts_dir();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let dest = dir.join(src.file_name().ok_or("nombre no válido")?);
    if !dest.exists() {
        std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
    }
    Ok((stem(src), bytes, dest))
}

/// Fonts previously uploaded to the library: (name, path, bytes).
pub fn library_fonts() -> Vec<(String, PathBuf, Vec<u8>)> {
    list(&fonts_dir())
        .into_iter()
        .filter(|p| FONT_EXTS.contains(&ext_of(p).as_str()))
        .filter_map(|p| std::fs::read(&p).ok().map(|b| (stem(&p), p.clone(), b)))
        .collect()
}

/// Editor preferences kept between sessions.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Last Unity project the design was exported to.
    pub unity_project: String,
    pub last_project_file: String,
    /// El tutorial ya se mostró (no se abre solo al iniciar).
    pub tutorial_visto: bool,
    /// Accesibilidad: idioma de la interfaz («es», «en»…), tamaño (1.0 = 100 %), alto contraste y movimiento reducido.
    pub idioma: String,
    pub escala_ui: f32,
    pub alto_contraste: bool,
    pub reducir_movimiento: bool,
    /// Tema de la interfaz (None = el predeterminado).
    pub tema: Option<crate::theme::Tema>,
    /// Sesión automática (capturas, pruebas): no se escribe nada en disco.
    #[serde(skip)]
    pub solo_lectura: bool,
}

fn settings_path() -> PathBuf {
    root_dir().parent().map(|p| p.join("ajustes.json")).unwrap_or_else(|| PathBuf::from("ajustes.json"))
}

impl Settings {
    pub fn load() -> Self {
        std::fs::read_to_string(settings_path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
    }
    pub fn save(&self) {
        if self.solo_lectura {
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
    e.name = Path::new(file).file_stem().and_then(|s| s.to_str()).unwrap_or("Imagen").to_string();
    e
}

fn stem(p: &Path) -> String {
    p.file_stem().and_then(|s| s.to_str()).unwrap_or("sin nombre").to_string()
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
}