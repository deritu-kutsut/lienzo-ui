//! The built-in asset pack: backgrounds, frames, buttons, icons and pixel art that ship inside the executable.
//!
//! `assets/pack/manifest.json` lists every item; `build.rs` turns it into a table of `include_bytes!`
//! (`PACK_FILES`). Using an item copies its file into the user's library images folder as
//! `pack_<id>.<ext>` (skipped when an identical file is already there) and creates an Image element
//! that points at it, so the exporter treats it like any other library image. See docs/CONTENT.md.

use crate::model::{Element, Kind};
use eframe::egui;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{LazyLock, Mutex};

include!(concat!(env!("OUT_DIR"), "/pack_table.rs"));

const MANIFEST: &str = include_str!("../assets/pack/manifest.json");

/// (category id, English label). The order is the order of the chips in the Images tab.
pub const CATEGORIES: &[(&str, &str)] = &[("backgrounds", "Backgrounds"), ("frames", "Frames"), ("buttons", "Buttons"), ("icons", "Icons"), ("pixel", "Pixel art")];

#[derive(Clone, Debug, Deserialize)]
pub struct PackItem {
    pub id: String,
    pub category: String,
    /// English display name.
    pub name: String,
    /// Path inside `assets/pack/` (e.g. `frames/wood.png`).
    pub file: String,
    /// 9-slice border `[left, bottom, right, top]` in source-image pixels.
    #[serde(default)]
    pub slice: Option<[f32; 4]>,
    #[serde(default)]
    pub ppu: Option<f32>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub pixel: bool,
    /// Size of the sprite before it was scaled up with nearest neighbour (pixel art only).
    #[serde(default)]
    pub native: Option<[u32; 2]>,
}

static ITEMS: LazyLock<Vec<PackItem>> = LazyLock::new(|| serde_json::from_str(MANIFEST).expect("assets/pack/manifest.json is valid"));

pub fn items() -> &'static [PackItem] {
    &ITEMS
}

pub fn item(id: &str) -> Option<&'static PackItem> {
    ITEMS.iter().find(|i| i.id == id)
}

/// The embedded file of an item (None for an unknown id).
pub fn bytes(id: &str) -> Option<&'static [u8]> {
    item(id).and_then(|i| bytes_of_file(&i.file))
}

pub fn bytes_of_file(file: &str) -> Option<&'static [u8]> {
    PACK_FILES.iter().find(|(f, _)| *f == file).map(|(_, b)| *b)
}

/// Translated label of a category id.
pub fn category_label(cat: &str) -> String {
    CATEGORIES.iter().find(|(c, _)| *c == cat).map(|(_, l)| crate::i18n::t(l)).unwrap_or_else(|| cat.to_string())
}

fn ext_of(file: &str) -> &str {
    file.rsplit('.').next().unwrap_or("png")
}

/// Name of the library file an item is copied to.
pub fn library_name(it: &PackItem) -> String {
    format!("pack_{}.{}", it.id, ext_of(&it.file))
}

/// Pixel size of the embedded image (reads the header only).
pub fn pixel_size(it: &PackItem) -> Option<(u32, u32)> {
    let b = bytes_of_file(&it.file)?;
    image::ImageReader::new(std::io::Cursor::new(b)).with_guessed_format().ok()?.into_dimensions().ok()
}

/// True when `path` holds exactly these bytes (compares the size first, so a huge user file is never read).
fn same_bytes(path: &Path, data: &[u8]) -> bool {
    std::fs::metadata(path).is_ok_and(|m| m.len() == data.len() as u64) && std::fs::read(path).map(|old| old == data).unwrap_or(false)
}

/// Fallback name when a user's own file already uses `pack_<id>.<ext>`: `pack_<id>_<first 8 hex of the sha-256>.<ext>`.
fn alternate_name(it: &PackItem, data: &[u8]) -> String {
    let h = crate::community_validate::sha256_hex(data);
    format!("pack_{}_{}.{}", it.id, &h[..8], ext_of(&it.file))
}

/// Copies the item into `dir` as `pack_<id>.<ext>`; an identical file already there is left alone. A different file
/// with that name (the user's own) is never overwritten: the copy goes to an alternate name, and if even that name is
/// taken by something else the call fails with a clear error. Returns the file name actually used.
pub fn extract_to(dir: &Path, it: &PackItem) -> Result<String, String> {
    let data = bytes_of_file(&it.file).ok_or_else(|| format!("pack file {} is not embedded", it.file))?;
    let mut name = library_name(it);
    let mut target = dir.join(&name);
    if same_bytes(&target, data) {
        return Ok(name);
    }
    if target.exists() {
        name = alternate_name(it, data);
        target = dir.join(&name);
        if same_bytes(&target, data) {
            return Ok(name);
        }
        if target.exists() {
            return Err(format!("{} already exists in the library with different content; rename or remove it", name));
        }
    }
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    if let Err(e) = crate::library::write_atomic(&target, data) {
        // Another thread (or the app itself) may have just written the same file: that is fine.
        if !same_bytes(&target, data) {
            return Err(e.to_string());
        }
    }
    Ok(name)
}

/// True when the library file `path` is an untouched copy of a built-in pack image (`pack_<id>.<ext>` or the
/// alternate name) whose content matches the manifest. A user's own file that merely starts with `pack_` is not.
pub fn is_pack_copy(path: &Path) -> bool {
    let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) else { return false };
    let Some(rest) = name.strip_prefix("pack_") else { return false };
    let Some((stem, ext)) = rest.rsplit_once('.') else { return false };
    let candidates = [Some(stem), stem.rsplit_once('_').filter(|(_, h)| h.len() == 8 && h.bytes().all(|b| b.is_ascii_hexdigit())).map(|(id, _)| id)];
    candidates.into_iter().flatten().any(|id| item(id).is_some_and(|it| ext_of(&it.file) == ext && bytes_of_file(&it.file).is_some_and(|d| same_bytes(path, d))))
}

/// Makes sure the library file named `pack_<id>.<ext>` exists in `dir` (used when a design refers to a pack image by name).
/// Returns false when the name is not a pack file.
pub fn ensure_named(dir: &Path, file_name: &str) -> bool {
    let Some(rest) = file_name.strip_prefix("pack_") else { return false };
    let Some((id, ext)) = rest.rsplit_once('.') else { return false };
    match item(id) {
        Some(it) if ext_of(&it.file) == ext => extract_to(dir, it).is_ok(),
        _ => false,
    }
}

/// Copies the item into the user's library images folder. Returns the file name to put in `Element::image`.
pub fn extract(it: &PackItem) -> Result<String, String> {
    extract_to(&crate::library::images_dir(), it)
}

/// The Image element that uses an item (`file` = the library file name, `page` = the design size, to fit backgrounds).
pub fn element_for(it: &PackItem, file: &str, page: (f32, f32)) -> Element {
    let (pw, ph) = pixel_size(it).map(|(w, h)| (w as f32, h as f32)).unwrap_or((256.0, 256.0));
    let mut e = Element::new(0, Kind::Image);
    e.image = file.to_string();
    e.name = it.name.clone();
    e.radius = 0.0;
    let (w, h) = match it.category.as_str() {
        "backgrounds" => {
            // cover the page, keeping the aspect ratio
            let s = (page.0 / pw).max(page.1 / ph).max(0.01);
            e.raycast = false;
            (pw * s, ph * s)
        }
        "frames" => fit(pw, ph, 480.0),
        "buttons" => fit(pw, ph, 400.0),
        "icons" => fit(pw, ph, 128.0),
        // pixel art: 1 native pixel = 8 page units whatever the export scale (ppu 100 = x8 sprites, ppu 50 = x4 UI)
        _ => {
            let s = 100.0 / it.ppu.unwrap_or(100.0).max(1.0);
            (pw * s, ph * s)
        }
    };
    e.w = w.max(8.0);
    e.h = h.max(8.0);
    e.slice = it.slice;
    e.ppu = it.ppu;
    e
}

fn fit(w: f32, h: f32, max_side: f32) -> (f32, f32) {
    let s = max_side / w.max(h).max(1.0);
    (w * s, h * s)
}

/// Search by name, tags or id (case-insensitive; the translated name also counts).
pub fn matches(it: &PackItem, q: &str) -> bool {
    let q = q.trim().to_lowercase();
    q.is_empty() || it.name.to_lowercase().contains(&q) || crate::i18n::t(&it.name).to_lowercase().contains(&q) || it.tags.iter().any(|t| t.to_lowercase().contains(&q)) || it.id.to_lowercase().contains(&q)
}

// ------------------------------------------------------------------ thumbnails (lazy, cached)

static THUMBS: LazyLock<Mutex<HashMap<String, egui::TextureHandle>>> = LazyLock::new(Default::default);

/// Cached thumbnail of an item. Decoding happens the first time only; `budget` limits how many new decodes one frame may do
/// (returns None while the rest wait for the next frames; the caller keeps asking and repaints).
pub fn thumbnail(ctx: &egui::Context, it: &PackItem, budget: &mut u32) -> Option<egui::TextureHandle> {
    let mut g = THUMBS.lock().ok()?;
    if let Some(t) = g.get(&it.id) {
        return Some(t.clone());
    }
    if *budget == 0 {
        ctx.request_repaint();
        return None;
    }
    *budget -= 1;
    let img = image::load_from_memory(bytes_of_file(&it.file)?).ok()?;
    let img = if img.width().max(img.height()) > 192 { img.resize(192, 192, image::imageops::FilterType::Triangle) } else { img };
    let rgba = img.to_rgba8();
    let ci = egui::ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw());
    let opts = if it.pixel { egui::TextureOptions::NEAREST } else { egui::TextureOptions::LINEAR };
    let tex = ctx.load_texture(format!("pack:{}", it.id), ci, opts);
    g.insert(it.id.clone(), tex.clone());
    Some(tex)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn temp(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("lienzo-pack-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// (width, height, extension) every file of a category must have.
    fn spec(cat: &str) -> Option<(u32, u32, &'static str)> {
        match cat {
            "backgrounds" => Some((1280, 720, "jpg")),
            "frames" => Some((512, 512, "png")),
            "buttons" => Some((512, 192, "png")),
            "icons" => Some((256, 256, "png")),
            _ => None,
        }
    }

    #[test]
    fn extract_never_overwrites_a_different_user_file() {
        let d = temp("noclobber");
        let it = &items()[0];
        let name = library_name(it);
        // a user's file with the same name: kept, the pack copy goes to an alternate name
        std::fs::write(d.join(&name), b"my own artwork").unwrap();
        let got = extract_to(&d, it).unwrap();
        assert_ne!(got, name);
        assert!(got.starts_with(&format!("pack_{}_", it.id)));
        assert_eq!(std::fs::read(d.join(&name)).unwrap(), b"my own artwork");
        assert_eq!(std::fs::read(d.join(&got)).unwrap(), bytes_of_file(&it.file).unwrap());
        // idempotent
        assert_eq!(extract_to(&d, it).unwrap(), got);
        // the alternate name is also taken by something else: a clear error, nothing overwritten
        std::fs::write(d.join(&got), b"squatter").unwrap();
        let e = extract_to(&d, it).unwrap_err();
        assert!(e.contains("already exists"), "{}", e);
        assert_eq!(std::fs::read(d.join(&got)).unwrap(), b"squatter");
        // an identical file is reused under the normal name
        let d2 = temp("same");
        assert_eq!(extract_to(&d2, it).unwrap(), name);
        assert_eq!(extract_to(&d2, it).unwrap(), name);
        // only the copies that match the manifest are recognised as pack files
        assert!(is_pack_copy(&d2.join(&name)));
        assert!(!is_pack_copy(&d.join(&name)), "a user's file called pack_<id>.png is not a pack copy");
        assert!(!is_pack_copy(&d.join("pack_nope.png")));
        let mut fake = ::std::fs::File::create(d2.join("pack_unknown.png")).unwrap();
        std::io::Write::write_all(&mut fake, b"x").unwrap();
        assert!(!is_pack_copy(&d2.join("pack_unknown.png")));
        let _ = std::fs::remove_dir_all(&d);
        let _ = std::fs::remove_dir_all(&d2);
    }

    #[test]
    fn manifest_is_consistent() {
        let mut ids = HashSet::new();
        let mut files = HashSet::new();
        assert!(items().len() >= 90, "the pack has {} items", items().len());
        for it in items() {
            assert!(ids.insert(it.id.as_str()), "duplicate id {}", it.id);
            assert!(files.insert(it.file.as_str()), "duplicate file {}", it.file);
            assert!(CATEGORIES.iter().any(|(c, _)| *c == it.category), "{}: unknown category {}", it.id, it.category);
            assert!(it.file.starts_with(&format!("{}/", it.category)), "{}: file {} is not in its category folder", it.id, it.file);
            assert!(!it.name.is_empty() && !it.tags.is_empty(), "{}: needs a name and tags", it.id);
            assert!(it.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'), "{}: id characters", it.id);
            // the file is in the folder, in the embedded table, and decodes
            let on_disk = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/pack").join(&it.file);
            assert!(on_disk.is_file(), "{} does not exist", it.file);
            let b = bytes(&it.id).unwrap_or_else(|| panic!("{} is not embedded", it.id));
            let img = image::load_from_memory(b).unwrap_or_else(|e| panic!("{} does not decode: {}", it.file, e));
            assert_eq!(pixel_size(it), Some((img.width(), img.height())));
            if let Some((w, h, ext)) = spec(&it.category) {
                assert_eq!((img.width(), img.height()), (w, h), "{}: wrong size", it.id);
                assert_eq!(ext_of(&it.file), ext, "{}: wrong format", it.id);
            } else {
                assert_eq!(ext_of(&it.file), "png");
                assert!(it.pixel, "{}: pixel category without pixel:true", it.id);
                let n = it.native.unwrap_or_else(|| panic!("{}: pixel art needs native:[w,h]", it.id));
                assert_eq!(img.width() % n[0], 0, "{}: not an integer scale", it.id);
                assert_eq!(img.width() / n[0], img.height() / n[1], "{}: scale differs per axis", it.id);
            }
            if matches!(it.category.as_str(), "frames" | "buttons") || (it.pixel && it.slice.is_some()) {
                assert!(it.slice.is_some(), "{}: frames and buttons stretch, they need a slice", it.id);
            }
            if let Some(s) = it.slice {
                let (w, h) = (img.width() as f32, img.height() as f32);
                assert!(s.iter().all(|v| *v >= 0.0), "{}: negative slice", it.id);
                assert!(s[0] + s[2] <= w && s[1] + s[3] <= h, "{}: slice {:?} does not fit {}x{}", it.id, s, w, h);
            }
            if let Some(p) = it.ppu {
                assert!((1.0..=10_000.0).contains(&p), "{}: ppu", it.id);
            }
            // transparent categories really have transparency
            if matches!(it.category.as_str(), "frames" | "buttons" | "icons") || (it.pixel && !it.tags.iter().any(|t| t == "fill")) {
                assert!(img.to_rgba8().pixels().any(|p| p[3] == 0), "{}: expected transparent pixels", it.id);
            }
        }
        for (cat, _) in CATEGORIES {
            assert!(items().iter().any(|i| i.category == *cat), "category {} is empty", cat);
        }
        assert!(items().iter().filter(|i| i.category == "pixel").count() >= 48);
    }

    #[test]
    fn the_exact_files_of_the_brief_exist() {
        let want = [
            ("backgrounds", "jpg", vec!["forest", "space", "dungeon", "neon_city", "desert", "ocean", "snow", "sunset", "cave", "candy"]),
            ("frames", "png", vec!["wood", "stone", "gold", "scifi", "parchment", "glass", "leather", "neon"]),
            ("buttons", "png", vec!["wood", "stone", "gold", "scifi", "glass", "candy", "neon", "leaf"]),
            ("icons", "png", vec!["heart", "mana", "coin", "gem", "star", "key", "sword", "shield", "potion", "scroll", "bomb", "crown", "lock", "clock", "map", "chest"]),
        ];
        for (cat, ext, names) in want {
            for n in names {
                let f = format!("{}/{}.{}", cat, n, ext);
                assert!(items().iter().any(|i| i.file == f), "missing manifest entry for {}", f);
            }
        }
    }

    #[test]
    fn the_build_table_lists_every_manifest_file() {
        let manifest_files: HashSet<&str> = items().iter().map(|i| i.file.as_str()).collect();
        let table: HashSet<&str> = PACK_FILES.iter().map(|(f, _)| *f).collect();
        assert_eq!(manifest_files, table);
        assert!(PACK_FILES.iter().all(|(_, b)| !b.is_empty()));
    }

    #[test]
    fn using_an_item_extracts_the_file_once_and_builds_a_valid_element() {
        let dir = temp("extract");
        for it in items() {
            let name = extract_to(&dir, it).unwrap();
            assert_eq!(name, library_name(it));
            let again = extract_to(&dir, it).unwrap();
            assert_eq!(again, name);
            assert_eq!(std::fs::read(dir.join(&name)).unwrap(), bytes(&it.id).unwrap());
            let e = element_for(it, &name, (1920.0, 1080.0));
            assert_eq!(e.kind, Kind::Image);
            assert_eq!(e.image, name);
            assert!(e.w >= 8.0 && e.h >= 8.0 && e.w.is_finite() && e.h.is_finite());
            assert_eq!(e.slice, it.slice);
            assert_eq!(crate::library::image_size(&dir.join(&name)), pixel_size(it));
        }
        // a different file with the same name is the user's: it is kept and the pack file gets another name
        let it = &items()[0];
        let name = library_name(it);
        std::fs::write(dir.join(&name), b"not an image").unwrap();
        let alt = extract_to(&dir, it).unwrap();
        assert_ne!(alt, name);
        assert_eq!(std::fs::read(dir.join(&name)).unwrap(), b"not an image");
        assert_eq!(std::fs::read(dir.join(&alt)).unwrap(), bytes(&it.id).unwrap());
        // named lookup (designs that mention pack_<id>.<ext>)
        let d2 = temp("named");
        assert!(ensure_named(&d2, "pack_px_coin.png"));
        assert!(d2.join("pack_px_coin.png").is_file());
        assert!(!ensure_named(&d2, "pack_nothing.png"));
        assert!(!ensure_named(&d2, "pack_px_coin.jpg"));
        assert!(!ensure_named(&d2, "photo.png"));
        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&d2);
    }

    #[test]
    fn backgrounds_cover_the_page_and_pixel_sprites_get_whole_scales() {
        let bg = item("bg_forest").unwrap();
        let e = element_for(bg, "pack_bg_forest.jpg", (1920.0, 1080.0));
        assert!((e.w - 1920.0).abs() < 0.5 && (e.h - 1080.0).abs() < 0.5);
        let e = element_for(bg, "pack_bg_forest.jpg", (1080.0, 1920.0));
        assert!(e.w >= 1080.0 && e.h >= 1920.0 - 0.5);
        let heart = item("px_heart_full").unwrap();
        let e = element_for(heart, "x.png", (1920.0, 1080.0));
        assert_eq!((e.w, e.h), (128.0, 128.0)); // 16 native pixels x 8
        let panel = item("px_panel_stone").unwrap();
        let e = element_for(panel, "x.png", (1920.0, 1080.0));
        assert_eq!((e.w, e.h), (384.0, 384.0)); // 48 native pixels x 8 (x4 file, ppu 50)
        assert_eq!(e.slice, Some([24.0, 24.0, 24.0, 24.0]));
    }

    #[test]
    fn search_finds_names_and_tags() {
        let wood = item("frame_wood").unwrap();
        assert!(matches(wood, "wood") && matches(wood, "RUSTIC") && matches(wood, "") && !matches(wood, "dragon"));
    }
}
