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
    /// Portrait overrides of `items` (sidecar file `<name>.portrait`; empty for older elements).
    pub overrides: Vec<crate::docext::ElementOverride>,
}

/// Cleans elements that come from a file (library, community): huge font sizes, NaN, control characters,
/// parent cycles... They go through the same `Doc::sanitize` as a loaded design, inside a temporary document.
pub fn sanitize_items(items: &[Element]) -> Vec<Element> {
    let mut d = Doc::empty(1920.0, 1080.0);
    d.elements = items.to_vec();
    d.next_id = 1;
    d.sanitize();
    d.elements
}

/// Writes `data` to `path` without ever leaving a half-written file: a temporary file in the same folder
/// is written, flushed and renamed over the destination (an atomic replace on the same volume).
pub fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let dir = match path.parent() {
        Some(d) if !d.as_os_str().is_empty() => d.to_path_buf(),
        _ => PathBuf::from("."),
    };
    let name = tmp_name_part(path);
    static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let tmp = dir.join(format!(".{}.{}-{}.tmp", name, std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)));
    let res = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(data)?;
        f.sync_all()?;
        drop(f);
        // An antivirus scanner or a sync client (OneDrive) may hold the destination for a moment: retry the rename.
        let mut last = Ok(());
        for attempt in 0..RENAME_ATTEMPTS {
            last = std::fs::rename(&tmp, path);
            if last.is_ok() {
                break;
            }
            if attempt + 1 < RENAME_ATTEMPTS {
                std::thread::sleep(std::time::Duration::from_millis(60 * (attempt as u64 + 1)));
            }
        }
        last
    })();
    if res.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    res
}

const RENAME_ATTEMPTS: u32 = 3;
/// Longest file name (in characters) used inside the temporary file's name (the full name could exceed the OS limit).
const TMP_NAME_MAX: usize = 60;

/// The part of the destination's file name that goes into the temporary file name (cut to [`TMP_NAME_MAX`] characters).
fn tmp_name_part(path: &Path) -> String {
    let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| "file".into());
    name.chars().take(TMP_NAME_MAX).collect()
}

/// Deletes the leftover temporary files (`.<name>.<pid>-<n>.tmp`) that [`write_atomic`] left next to `path` after a crash,
/// when they are older than `min_age`. Returns how many were removed.
pub fn clean_tmp_orphans(path: &Path, min_age: std::time::Duration) -> usize {
    let dir = match path.parent() {
        Some(d) if !d.as_os_str().is_empty() => d.to_path_buf(),
        _ => PathBuf::from("."),
    };
    let prefix = format!(".{}.", tmp_name_part(path));
    let Ok(rd) = std::fs::read_dir(&dir) else { return 0 };
    let mut n = 0;
    for f in rd.filter_map(|e| e.ok()) {
        let name = f.file_name().to_string_lossy().to_string();
        if name.starts_with(&prefix) && name.ends_with(".tmp") && f.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|d| d >= min_age) && std::fs::remove_file(f.path()).is_ok() {
            n += 1;
        }
    }
    n
}

/// Leftover temporary files older than this are deleted when a project is opened or saved.
pub const TMP_ORPHAN_AGE: std::time::Duration = std::time::Duration::from_secs(3600);

/// Largest project file the editor reads (a bigger one is refused before it is parsed).
pub const MAX_PROJECT_BYTES: u64 = 64 * 1024 * 1024;

/// Reads a project / design file as text, refusing files bigger than [`MAX_PROJECT_BYTES`].
pub fn read_project_text(path: &Path) -> Result<String, String> {
    let len = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    if len > MAX_PROJECT_BYTES {
        return Err(format!("the file is too large ({} MB, the limit is {} MB)", len / (1024 * 1024), MAX_PROJECT_BYTES / (1024 * 1024)));
    }
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

fn portrait_sidecar(path: &Path) -> PathBuf {
    path.with_extension("portrait")
}

fn load_overrides(path: &Path, items: &[Element]) -> Vec<crate::docext::ElementOverride> {
    let Some(text) = std::fs::read_to_string(portrait_sidecar(path)).ok() else { return Vec::new() };
    let Ok(mut ovs) = serde_json::from_str::<Vec<crate::docext::ElementOverride>>(strip_bom(&text)) else { return Vec::new() };
    let ids: std::collections::HashSet<u64> = items.iter().map(|e| e.id).collect();
    let mut seen = std::collections::HashSet::new();
    ovs.retain(|o| ids.contains(&o.id) && seen.insert(o.id));
    ovs.truncate(items.len());
    for o in ovs.iter_mut() {
        o.sanitize();
    }
    ovs
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
    base.unwrap_or_else(fallback_base).join("EditorGUI").join("biblioteca")
}

static DATA_FALLBACK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Where data goes when the OS gives no application-data folder (no APPDATA / HOME): next to the executable, or the
/// temp folder. Never a path relative to the working directory (which could be anywhere, even a read-only place).
fn fallback_base() -> PathBuf {
    DATA_FALLBACK.store(true, std::sync::atomic::Ordering::Relaxed);
    fallback_dir()
}

fn fallback_dir() -> PathBuf {
    std::env::current_exe().ok().and_then(|e| e.parent().map(|p| p.to_path_buf())).filter(|p| p.is_absolute()).unwrap_or_else(std::env::temp_dir)
}

/// True once the data folder had to fall back to the executable's folder or the temp folder (the editor shows a notice).
pub fn data_dir_is_fallback() -> bool {
    let _ = root_dir();
    DATA_FALLBACK.load(std::sync::atomic::Ordering::Relaxed)
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

/// The folder that holds `file`: a relative name without a folder (`x.editorgui.json`) lives in the current directory.
pub fn folder_of(file: &Path) -> Option<PathBuf> {
    let p = match file.parent() {
        Some(p) if !p.as_os_str().is_empty() => p.to_path_buf(),
        Some(_) => std::env::current_dir().ok()?,
        None => return None,
    };
    Some(std::fs::canonicalize(&p).unwrap_or(p))
}

pub fn set_design_folder(file: &Path) {
    if let Ok(mut g) = DESIGN_FOLDER.lock() {
        *g = folder_of(file);
    }
    crate::render::clear_texture_cache();
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

/// Whether designs opened right now come from a source the user trusts (a file the user opened from
/// disk, a design built in the editor): only then may an image reference be an absolute path.
/// SITES THAT MUST CALL `set_trusted_source(false)` before loading a design (owned by other code):
/// plugin command output, community templates / downloaded designs, MCP-supplied designs.
/// The file-open paths (app open dialog, `--open`) call `set_trusted_source(true)`.
/// Default is `true` so existing local workflows keep working.
static TRUSTED_SOURCE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

pub fn set_trusted_source(trusted: bool) {
    TRUSTED_SOURCE.store(trusted, std::sync::atomic::Ordering::Relaxed);
}

pub fn trusted_source() -> bool {
    TRUSTED_SOURCE.load(std::sync::atomic::Ordering::Relaxed)
}

/// The path returned for a refused image reference (it never exists, so callers report a missing image).
pub fn blocked_path() -> PathBuf {
    images_dir().join("__blocked__")
}

const RESERVED_DEVICES: &[&str] = &["CON", "PRN", "AUX", "NUL", "CLOCK$", "CONIN$", "CONOUT$"];

/// A path component that is safe on every platform: no stream/drive `:`, no control characters, no
/// reserved Windows device names (`CON`, `NUL.png`, `COM1`...), no trailing dot or space.
fn safe_component(c: &str) -> bool {
    if c.is_empty() || c == "." || c == ".." || c.contains(':') || c.contains('\\') || c.chars().any(|ch| ch.is_control() || matches!(ch, '<' | '>' | '"' | '|' | '?' | '*')) {
        return false;
    }
    if c.ends_with('.') || c.ends_with(' ') {
        return false;
    }
    let stem = c.split('.').next().unwrap_or("").trim_end().to_ascii_uppercase();
    if RESERVED_DEVICES.contains(&stem.as_str()) {
        return false;
    }
    let digits = |p: &str| stem.strip_prefix(p).map(|d| d.len() == 1 && d != "0" && d.chars().all(|x| x.is_ascii_digit())).unwrap_or(false);
    !(digits("COM") || digits("LPT"))
}

/// `Assets/...` or `Packages/...` (case-sensitive, as Unity) made only of safe components.
pub fn valid_unity_rel(rel: &str) -> bool {
    let mut parts = rel.split('/');
    matches!(parts.next(), Some("Assets") | Some("Packages")) && {
        let mut n = 0;
        parts.all(|c| {
            n += 1;
            safe_component(c)
        }) && n > 0
    }
}

/// A bare file name (no separators, no `..`, no stream/drive, no device name).
pub fn valid_image_file_name(n: &str) -> bool {
    !n.contains('/') && !n.contains('#') && safe_component(n)
}

/// Is `p` (canonicalized) inside `base` (canonicalized)?
fn inside(p: &Path, base: &Path) -> bool {
    match (std::fs::canonicalize(p), std::fs::canonicalize(base)) {
        (Ok(p), Ok(b)) => p.starts_with(&b),
        _ => false,
    }
}

/// Relative name made only of normal components (`./` allowed), nothing rooted, no `:`.
fn relative_name_ok(name: &str) -> bool {
    use std::path::Component;
    if name.is_empty() || name.contains(':') || name.chars().any(|c| c.is_control()) {
        return false;
    }
    let p = Path::new(name);
    let mut any = false;
    for c in p.components() {
        match c {
            Component::Normal(s) => {
                any = true;
                if !s.to_str().map(safe_component).unwrap_or(false) {
                    return false;
                }
            }
            Component::CurDir => {}
            _ => return false,
        }
    }
    any
}

/// Path of an element's image. Handled in this order:
/// - `unity:Assets/...[#sub]` / `unity:Packages/...`: validated by path components and resolved
///   against the Unity project root; the real (canonical) file must stay inside it (junctions).
/// - UNC / device paths and alternate streams: refused.
/// - absolute paths: only when the design comes from a trusted local source (`set_trusted_source`),
///   or when they already point inside the library images folder or the open design's folder.
/// - relative names: no `..`, nothing rooted; resolved in the open design's folder or the library
///   images folder and the canonical file must stay inside it.
/// Anything refused returns `blocked_path()`.
pub fn resolve_image(name: &str) -> PathBuf {
    if let Some(rest) = name.strip_prefix("unity:") {
        let rel = rest.split('#').next().unwrap_or("");
        if !valid_unity_rel(rel) {
            return blocked_path();
        }
        let Some(root) = UNITY_PROJECT.lock().ok().and_then(|g| g.clone()) else { return blocked_path() };
        let joined = root.join(rel);
        if joined.exists() && !inside(&joined, &root) {
            return blocked_path();
        }
        return joined;
    }
    if is_unc_or_stream(name) {
        return blocked_path();
    }
    let design = DESIGN_FOLDER.lock().ok().and_then(|g| g.clone());
    let p = Path::new(name);
    if p.is_absolute() {
        if trusted_source() || inside(p, &images_dir()) || design.as_ref().map(|d| inside(p, d)).unwrap_or(false) {
            return p.to_path_buf();
        }
        return blocked_path();
    }
    if !relative_name_ok(name) {
        return blocked_path();
    }
    if let Some(base) = design {
        let candidate = base.join(name);
        if candidate.exists() {
            return if inside(&candidate, &base) { candidate } else { blocked_path() };
        }
    }
    let lib = images_dir();
    let candidate = lib.join(name);
    // A built-in pack image (`pack_<id>.<ext>`) that is not in the library yet is copied there the first time it is needed.
    if name.starts_with("pack_") && !candidate.exists() {
        crate::pack::ensure_named(&lib, name);
    }
    if candidate.exists() && !inside(&candidate, &lib) {
        return blocked_path();
    }
    candidate
}

fn ext_of(p: &Path) -> String {
    p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase()
}

/// CON, PRN, AUX, NUL, COM1-9, LPT1-9 (also with a superscript digit), compared case-insensitively.
fn is_reserved_device_name(stem: &str) -> bool {
    let base = stem.split('.').next().unwrap_or("").trim_end_matches([' ', '.']).to_ascii_lowercase();
    if matches!(base.as_str(), "con" | "prn" | "aux" | "nul" | "conin$" | "conout$") {
        return true;
    }
    let digits = ['1', '2', '3', '4', '5', '6', '7', '8', '9', '¹', '²', '³'];
    for pre in ["com", "lpt"] {
        if let Some(rest) = base.strip_prefix(pre) {
            let mut it = rest.chars();
            if let (Some(c), None) = (it.next(), it.next()) {
                if digits.contains(&c) {
                    return true;
                }
            }
        }
    }
    false
}

/// Make a file name safe and unique inside `dir`.
fn unique_name(dir: &Path, stem: &str, ext: &str) -> String {
    let clean: String = stem.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' }).collect();
    // Windows drops trailing dots / spaces and reserves device names (CON, NUL, COM1...) with any extension.
    let clean = clean.trim().trim_end_matches(['.', ' ']).to_string();
    let clean = if clean.is_empty() { "archivo".to_string() } else { clean };
    let clean = if is_reserved_device_name(&clean) { format!("{}_", clean) } else { clean };
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
            // `pack_*` files are copies of the built-in pack (see pack.rs): they belong to the pack tiles, not to "Your images"
            // (only those that really match the built-in manifest: a user's own `pack_cat.png` stays visible)
            let is_pack_copy = crate::pack::is_pack_copy(&p);
            if IMAGE_EXTS.contains(&ext_of(&p).as_str()) && !is_pack_copy {
                if let Some((w, h)) = image_size(&p) {
                    self.images.push(LibImage { file: p.file_name().unwrap().to_string_lossy().into(), w, h });
                }
            }
        }
        for p in list(&elements_dir()) {
            if ext_of(&p) == "json" {
                if let Some(items) = std::fs::read_to_string(&p).ok().and_then(|s| serde_json::from_str::<Vec<Element>>(strip_bom(&s)).ok()) {
                    let items = sanitize_items(&items);
                    let overrides = load_overrides(&p, &items);
                    self.elements.push(LibElement { name: stem(&p), path: p, items, overrides });
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
        self.save_element_with(name, items, &[])
    }

    /// [`Library::save_element`] that also stores the portrait overrides of the elements (geometry relative to the
    /// group, like the elements themselves).
    pub fn save_element_with(&mut self, name: &str, items: &[Element], overrides: &[crate::docext::ElementOverride]) -> Result<String, String> {
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
        let side = portrait_sidecar(&dir.join(&file));
        let ovs: Vec<crate::docext::ElementOverride> = overrides
            .iter()
            .filter(|o| ids.contains(&o.id))
            .cloned()
            .map(|mut o| {
                o.x = o.x.map(|v| v - min_x);
                o.y = o.y.map(|v| v - min_y);
                o
            })
            .collect();
        if ovs.is_empty() {
            let _ = std::fs::remove_file(&side);
        } else {
            let _ = std::fs::write(&side, serde_json::to_string_pretty(&ovs).map_err(|e| e.to_string())?);
        }
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
        if ext_of(path) == "json" {
            let _ = std::fs::remove_file(portrait_sidecar(path));
        }
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
    /// Autosave / crash recovery turned off (it is on by default).
    pub no_autosave: bool,
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
            let _ = write_atomic(&p, s.as_bytes());
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
        // elements saved with portrait overrides keep them (relative to the group); a hostile file is cleaned on load
        let mut a = Element::new(1, Kind::Panel);
        a.x = 100.0;
        a.y = 50.0;
        a.w = 20.0;
        a.h = 20.0;
        let mut o = crate::docext::ElementOverride::new(1);
        o.x = Some(300.0);
        o.y = Some(80.0);
        let f = lib.save_element_with("con", &[a.clone()], &[o]).unwrap();
        assert_ne!(f.to_lowercase(), "con.json");
        let saved = lib.elements.iter().find(|e| e.path.file_name().unwrap().to_string_lossy() == f).unwrap();
        assert_eq!(saved.overrides.len(), 1);
        assert_eq!((saved.overrides[0].x, saved.overrides[0].y), (Some(200.0), Some(30.0)));
        let path = saved.path.clone();
        let mut evil = a.clone();
        evil.font_size = 1.0e9;
        std::fs::write(&path, serde_json::to_string(&vec![evil]).unwrap()).unwrap();
        lib.refresh();
        let again = lib.elements.iter().find(|e| e.path == path).unwrap();
        assert!(again.items[0].font_size <= 1000.0);
        lib.remove(&path);
        assert!(!path.with_extension("portrait").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    static GLOBALS: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn temp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("lienzo-resolve-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn unity_and_relative_references_are_contained() {
        let _g = GLOBALS.lock().unwrap_or_else(|e| e.into_inner());
        let base = temp_dir("a");
        let proj = base.join("proj");
        std::fs::create_dir_all(proj.join("Assets/UI")).unwrap();
        std::fs::create_dir_all(proj.join("ProjectSettings")).unwrap();
        std::fs::write(proj.join("Assets/UI/a.png"), b"x").unwrap();
        std::fs::write(base.join("secret.txt"), b"s").unwrap();
        set_unity_project(proj.to_str().unwrap());
        let blocked = blocked_path();
        // valid
        assert_eq!(resolve_image("unity:Assets/UI/a.png"), proj.join("Assets/UI/a.png"));
        assert_eq!(resolve_image("unity:Assets/UI/a.png#sprite_1"), proj.join("Assets/UI/a.png"));
        assert_eq!(resolve_image("unity:Packages/com.x/a.png"), proj.join("Packages/com.x/a.png"));
        // blocked
        for bad in [
            "unity:Assets/../secret.txt",
            "unity:AssetsEvil/a.png",
            "unity:assets/UI/a.png",
            "unity:C:foo",
            "unity:Assets/C:foo/a.png",
            "unity:Assets/UI/a.png:stream",
            "unity:/Assets/a.png",
            "unity:Assets//a.png",
            "unity:Assets\\UI\\a.png",
            "unity://server/share/a.png",
            "unity:\\\\server\\share\\a.png",
            "unity:Assets/NUL.png",
            "unity:Assets/com1/a.png",
            "unity:Assets",
            "unity:",
            "unity:Assets/",
            "\\\\server\\share\\a.png",
            "//server/share/a.png",
            "a.png:stream",
        ] {
            assert_eq!(resolve_image(bad), blocked, "{bad}");
        }
        // relative names
        for bad in ["..\\..\\secret.txt", "../secret.txt", "a/../../b.png", "C:foo.png", "/etc/passwd", "\\rooted.png", "sub/../x.png"] {
            assert_eq!(resolve_image(bad), blocked, "{bad}");
        }
        assert_eq!(resolve_image("sword.png"), images_dir().join("sword.png"));
        // design folder
        std::fs::create_dir_all(base.join("design/pixel")).unwrap();
        std::fs::write(base.join("design/pixel/s.png"), b"x").unwrap();
        set_design_folder(&base.join("design/x.json"));
        assert_ne!(resolve_image("pixel/s.png"), blocked);
        assert!(resolve_image("pixel/s.png").exists());
        // absolute: allowed only when trusted (or already inside the design folder)
        let abs = base.join("secret.txt");
        set_trusted_source(true);
        assert_eq!(resolve_image(abs.to_str().unwrap()), abs);
        set_trusted_source(false);
        assert_eq!(resolve_image(abs.to_str().unwrap()), blocked);
        let inside_design = base.join("design/pixel/s.png");
        assert_eq!(resolve_image(inside_design.to_str().unwrap()), inside_design);
        set_trusted_source(true);
        if let Ok(mut g) = DESIGN_FOLDER.lock() {
            *g = None;
        }
        set_unity_project("");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn junction_escape_is_blocked() {
        if !cfg!(windows) {
            return;
        }
        let _g = GLOBALS.lock().unwrap_or_else(|e| e.into_inner());
        let base = temp_dir("j");
        let proj = base.join("proj");
        std::fs::create_dir_all(proj.join("Assets")).unwrap();
        let outside = base.join("outside");
        std::fs::create_dir_all(&outside).unwrap();
        std::fs::write(outside.join("s.png"), b"x").unwrap();
        let link = proj.join("Assets").join("link");
        let ok = std::process::Command::new("cmd").args(["/c", "mklink", "/J"]).arg(&link).arg(&outside).output().map(|o| o.status.success()).unwrap_or(false);
        if !ok {
            eprintln!("mklink /J not available: test skipped");
            let _ = std::fs::remove_dir_all(&base);
            return;
        }
        set_unity_project(proj.to_str().unwrap());
        assert_eq!(resolve_image("unity:Assets/link/s.png"), blocked_path());
        // remove the junction itself first so remove_dir_all never follows it
        let _ = std::process::Command::new("cmd").args(["/c", "rmdir"]).arg(&link).output();
        set_unity_project("");
        let _ = std::fs::remove_dir_all(&base);
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

    #[test]
    fn elements_from_files_are_sanitized_before_they_reach_the_canvas() {
        let mut bad = Element::new(1, Kind::Text);
        bad.font_size = 1.0e9;
        bad.w = f32::NAN;
        bad.h = 1.0e12;
        bad.x = f32::INFINITY;
        bad.text = "a\u{0}b\u{2028}c".into();
        bad.parent = Some(2);
        let mut b2 = Element::new(2, Kind::Panel);
        b2.parent = Some(1); // parent cycle
        let out = sanitize_items(&[bad, b2]);
        assert_eq!(out.len(), 2);
        for e in &out {
            assert!(e.font_size <= 1000.0 && e.w.is_finite() && e.h <= crate::model::COORD_LIMIT && e.x.is_finite(), "{:?}", e);
        }
        assert_eq!(out[0].text, "abc");
        assert!(out.iter().filter(|e| e.parent.is_none()).count() >= 1, "the cycle is broken");
        // tiles clamp too (a hostile element that skipped the loader)
        let mut huge = Element::new(1, Kind::Text);
        huge.w = 0.0;
        huge.font_size = f32::NAN;
        let ctx = eframe::egui::Context::default();
        let mut out = ctx.run_ui(Default::default(), |ui| {
            let r = eframe::egui::Rect::from_min_size(eframe::egui::pos2(0.0, 0.0), eframe::egui::vec2(80.0, 60.0));
            crate::render::paint_element_thumb(ui.painter(), &huge, r);
        });
        out.textures_delta.clear();
    }

    #[test]
    fn generated_names_avoid_windows_device_names_and_trailing_dots() {
        let d = std::env::temp_dir();
        for (stem, bad) in [("con", true), ("NUL", true), ("com1", true), ("Lpt9", true), ("aux", true), ("com10", false), ("console", false), ("a", false)] {
            let n = unique_name(&d, stem, "json");
            assert!(!is_reserved_device_name(&n), "{} -> {}", stem, n);
            if bad {
                assert_ne!(n.to_lowercase(), format!("{}.json", stem.to_lowercase()));
            }
        }
        for s in ["name...", "name. . ", "name ", "name.", "..."] {
            let n = unique_name(&d, s, "png");
            let stem = n.trim_end_matches(".png");
            assert!(!stem.ends_with('.') && !stem.ends_with(' ') && !stem.is_empty(), "{:?} -> {:?}", s, n);
        }
        assert_eq!(unique_name(&d, "  ", "png"), "archivo.png");
    }

    #[test]
    fn write_atomic_handles_long_names_and_cleans_old_orphans() {
        let dir = std::env::temp_dir().join(format!("lienzo_wa_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // a file name close to the 255-character limit: the temporary name is cut, so the write still works
        let long = dir.join(format!("{}.json", "n".repeat(245)));
        write_atomic(&long, b"{}").unwrap();
        assert_eq!(std::fs::read(&long).unwrap(), b"{}");
        write_atomic(&long, b"{\"v\":2}").unwrap();
        assert_eq!(std::fs::read(&long).unwrap(), b"{\"v\":2}");
        assert!(std::fs::read_dir(&dir).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().ends_with(".tmp")));
        // orphaned temporaries of THIS file older than the age are deleted; recent ones and other files' are not
        let target = dir.join("p.editorgui.json");
        let old = dir.join(".p.editorgui.json.99-1.tmp");
        let young = dir.join(".p.editorgui.json.99-2.tmp");
        let other = dir.join(".q.editorgui.json.99-1.tmp");
        for f in [&old, &young, &other] {
            std::fs::write(f, "x").unwrap();
        }
        let two_hours_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(7200);
        std::fs::OpenOptions::new().write(true).open(&old).unwrap().set_modified(two_hours_ago).unwrap();
        assert_eq!(clean_tmp_orphans(&target, TMP_ORPHAN_AGE), 1);
        assert!(!old.exists() && young.exists() && other.exists());
        // a relative name without a folder works too (the current directory)
        assert_eq!(clean_tmp_orphans(Path::new("no-such-file-xyz.json"), TMP_ORPHAN_AGE), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn project_files_over_the_size_cap_are_refused_before_parsing() {
        let dir = std::env::temp_dir().join(format!("lienzo_cap_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let ok = dir.join("ok.json");
        std::fs::write(&ok, "{}").unwrap();
        assert_eq!(read_project_text(&ok).unwrap(), "{}");
        let big = dir.join("big.json");
        let f = std::fs::File::create(&big).unwrap();
        f.set_len(MAX_PROJECT_BYTES + 1).unwrap();
        drop(f);
        assert!(read_project_text(&big).unwrap_err().contains("too large"));
        assert!(read_project_text(&dir.join("missing.json")).is_err());
        assert!(fallback_dir().is_absolute(), "the fallback data folder is never relative to the working directory");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_atomic_replaces_without_leftovers() {
        let dir = temp_dir("atomic");
        let f = dir.join("p.json");
        write_atomic(&f, b"one").unwrap();
        write_atomic(&f, b"two two").unwrap();
        assert_eq!(std::fs::read_to_string(&f).unwrap(), "two two");
        let names: Vec<String> = std::fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect();
        assert_eq!(names, vec!["p.json".to_string()], "no temporary file is left behind");
        // a destination that cannot be written fails without a leftover
        assert!(write_atomic(&dir.join("missing").join("p.json"), b"x").is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn portrait_overrides_sidecar_round_trip() {
        let dir = temp_dir("side");
        let el = dir.join("pieza.json");
        let mut a = Element::new(1, Kind::Panel);
        a.w = 10.0;
        let mut b = Element::new(2, Kind::Panel);
        b.parent = Some(1);
        let items = vec![a, b];
        std::fs::write(&el, serde_json::to_string(&items).unwrap()).unwrap();
        // nothing next to it: older library elements have no overrides
        assert!(load_overrides(&el, &items).is_empty());
        let mut o = crate::docext::ElementOverride::new(2);
        o.x = Some(7.0);
        let stray = crate::docext::ElementOverride::new(99);
        let dup = crate::docext::ElementOverride::new(2);
        std::fs::write(portrait_sidecar(&el), serde_json::to_string(&vec![o, stray, dup]).unwrap()).unwrap();
        let got = load_overrides(&el, &items);
        assert_eq!(got.len(), 1, "unknown ids and duplicates are dropped");
        assert_eq!((got[0].id, got[0].x), (2, Some(7.0)));
        assert_ne!(portrait_sidecar(&el).extension().unwrap(), "json", "the sidecar is not listed as an element");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn texture_cache_follows_the_file_on_disk() {
        let _g = GLOBALS.lock().unwrap_or_else(|e| e.into_inner());
        let dir = temp_dir("tex");
        set_design_folder(&dir.join("d.json"));
        let ctx = eframe::egui::Context::default();
        // Other tests may open designs at the same time (that resets the folder and the cache): poll instead of assuming.
        let folder = dir.join("d.json");
        let wait = |size: Option<[usize; 2]>| {
            for _ in 0..40 {
                let want = std::fs::canonicalize(&dir).unwrap();
                if DESIGN_FOLDER.lock().unwrap().as_ref() != Some(&want) {
                    set_design_folder(&folder);
                }
                std::thread::sleep(std::time::Duration::from_millis(300));
                if crate::render::texture(&ctx, "late.png").map(|t| t.size()) == size {
                    return true;
                }
            }
            false
        };
        // missing: nothing, but it is looked for again
        assert!(wait(None));
        image::RgbaImage::from_pixel(8, 4, image::Rgba([1, 2, 3, 255])).save(dir.join("late.png")).unwrap();
        assert!(wait(Some([8, 4])), "the file appeared");
        // replaced by another size: reloaded
        image::RgbaImage::from_pixel(16, 6, image::Rgba([1, 2, 3, 255])).save(dir.join("late.png")).unwrap();
        assert!(wait(Some([16, 6])), "the changed file was reloaded");
        *DESIGN_FOLDER.lock().unwrap() = None;
        crate::render::clear_texture_cache();
        let _ = std::fs::remove_dir_all(&dir);
    }
}
