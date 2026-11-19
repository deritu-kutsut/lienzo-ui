//! Painting of page elements (shared by the canvas, thumbnails and drag ghosts).

use crate::model::{Element, Kind, Rgba};
use eframe::egui::{
    self, Align, Color32, FontId, Mesh, Painter, Pos2, Shape, Stroke, Vec2,
    epaint::TextShape,
    text::{LayoutJob, TextFormat},
};
use std::sync::Arc;

/// Maps page coordinates to screen coordinates.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub origin: Pos2,
    pub zoom: f32,
}

impl View {
    pub fn to_screen(&self, x: f32, y: f32) -> Pos2 {
        Pos2::new(self.origin.x + x * self.zoom, self.origin.y + y * self.zoom)
    }
    pub fn to_page(&self, p: Pos2) -> [f32; 2] {
        [(p.x - self.origin.x) / self.zoom, (p.y - self.origin.y) / self.zoom]
    }
}

pub fn col(c: Rgba, opacity: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c[0], c[1], c[2], (c[3] as f32 * opacity.clamp(0.0, 1.0)) as u8)
}

fn lerp_col(a: Rgba, b: Rgba, t: f32, opacity: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    col([l(a[0], b[0]), l(a[1], b[1]), l(a[2], b[2]), l(a[3], b[3])], opacity)
}

/// Element-local frame: local coordinates are page units relative to the box centre.
#[derive(Clone, Copy)]
pub struct Xf {
    pub c: Pos2,
    pub sin: f32,
    pub cos: f32,
    pub z: f32,
}

impl Xf {
    pub fn of(e: &Element, v: &View) -> Self {
        let [cx, cy] = e.center();
        let (sin, cos) = e.rot.to_radians().sin_cos();
        Xf { c: v.to_screen(cx, cy), sin, cos, z: v.zoom }
    }
    pub fn map(&self, lx: f32, ly: f32) -> Pos2 {
        let x = lx * self.z;
        let y = ly * self.z;
        Pos2::new(self.c.x + x * self.cos - y * self.sin, self.c.y + x * self.sin + y * self.cos)
    }
    pub fn dir(&self, lx: f32, ly: f32) -> Vec2 {
        Vec2::new(lx * self.cos - ly * self.sin, lx * self.sin + ly * self.cos)
    }
}

/// Outline of a rounded rectangle centred at the origin, clockwise.
pub fn rounded_rect_pts(hw: f32, hh: f32, r: f32) -> Vec<[f32; 2]> {
    let r = r.max(0.0).min(hw).min(hh);
    if r < 0.5 {
        return vec![[-hw, -hh], [hw, -hh], [hw, hh], [-hw, hh]];
    }
    let segs = 8;
    let mut pts = Vec::with_capacity(4 * (segs + 1));
    let corners = [
        ([hw - r, -hh + r], -90.0f32),
        ([hw - r, hh - r], 0.0),
        ([-hw + r, hh - r], 90.0),
        ([-hw + r, -hh + r], 180.0),
    ];
    for (c, start) in corners {
        for i in 0..=segs {
            let a = (start + 90.0 * i as f32 / segs as f32).to_radians();
            pts.push([c[0] + r * a.cos(), c[1] + r * a.sin()]);
        }
    }
    pts
}

pub fn ellipse_pts(hw: f32, hh: f32) -> Vec<[f32; 2]> {
    (0..64)
        .map(|i| {
            let a = i as f32 / 64.0 * std::f32::consts::TAU;
            [hw * a.cos(), hh * a.sin()]
        })
        .collect()
}

fn fill_poly(p: &Painter, xf: &Xf, pts: &[[f32; 2]], fill: Color32, stroke: Stroke) {
    let screen: Vec<Pos2> = pts.iter().map(|q| xf.map(q[0], q[1])).collect();
    if fill.a() > 0 {
        p.add(Shape::convex_polygon(screen.clone(), fill, Stroke::NONE));
    }
    if stroke.width > 0.0 && stroke.color.a() > 0 {
        p.add(Shape::closed_line(screen, stroke));
    }
}

fn gradient_poly(p: &Painter, xf: &Xf, pts: &[[f32; 2]], hw: f32, hh: f32, a: Rgba, b: Rgba, opacity: f32) {
    let t_of = |q: [f32; 2]| ((q[0] / hw.max(1e-3) + q[1] / hh.max(1e-3)) * 0.25 + 0.5).clamp(0.0, 1.0);
    let mut mesh = Mesh::default();
    mesh.colored_vertex(xf.map(0.0, 0.0), lerp_col(a, b, 0.5, opacity));
    for q in pts {
        mesh.colored_vertex(xf.map(q[0], q[1]), lerp_col(a, b, t_of(*q), opacity));
    }
    let n = pts.len() as u32;
    for i in 0..n {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
    }
    p.add(Shape::mesh(mesh));
}

// ---------------------------------------------------------------- fonts

struct FontState {
    defs: egui::FontDefinitions,
    available: Vec<String>,
    pending: Vec<String>,
    paths: std::collections::HashMap<String, std::path::PathBuf>,
    /// Fallback font for the interface language (Chinese, Japanese, Korean), at the end of the lists.
    backup: Option<String>,
    /// Interface font chosen in the theme ("" = the default one).
    interface: String,
}

/// File of a font family known to the editor (system or library), for exporting to Unity.
pub fn font_path(name: &str) -> Option<std::path::PathBuf> {
    FONTS.lock().ok().and_then(|g| g.as_ref().and_then(|s| s.paths.get(name).cloned()))
}

static FONTS: std::sync::Mutex<Option<FontState>> = std::sync::Mutex::new(None);

/// Fonts that can be picked in the editor (system fonts + fonts added to the library).
pub fn available_fonts() -> Vec<String> {
    FONTS.lock().ok().and_then(|g| g.as_ref().map(|s| s.available.clone())).unwrap_or_default()
}

fn register(defs: &mut egui::FontDefinitions, name: &str, bytes: Vec<u8>) {
    let fallback = defs.families.get(&egui::FontFamily::Proportional).cloned().unwrap_or_default();
    defs.font_data.insert(name.to_string(), Arc::new(egui::FontData::from_owned(bytes)));
    let mut list = vec![name.to_string()];
    list.extend(fallback.into_iter().filter(|f| f != name));
    defs.families.insert(egui::FontFamily::Name(name.into()), list);
}

/// Look for common system fonts and the library's fonts and register them with egui.
pub fn load_system_fonts(ctx: &egui::Context) {
    let candidates = system_font_candidates();
    let mut defs = egui::FontDefinitions::default();
    let mut found = Vec::new();
    let mut paths = std::collections::HashMap::new();
    for (name, files) in candidates {
        if let Some((file, bytes)) = files.iter().find_map(|p| std::fs::read(p).ok().map(|b| (p, b))) {
            register(&mut defs, name, bytes);
            found.push(name.to_string());
            paths.insert(name.to_string(), std::path::PathBuf::from(file));
        }
    }
    for (name, path, bytes) in crate::library::library_fonts() {
        if !found.contains(&name) {
            register(&mut defs, &name, bytes);
            paths.insert(name.clone(), path);
            found.push(name);
        }
    }
    ctx.set_fonts(defs.clone());
    if let Ok(mut g) = FONTS.lock() {
        *g = Some(FontState { defs, available: found, pending: Vec::new(), paths, backup: None, interface: String::new() });
    }
}

/// The app's interface font (themes): `name` goes in front of the defaults; "" or one that is missing = the default.
pub fn interface_font(ctx: &egui::Context, name: &str) {
    if let Ok(mut g) = FONTS.lock() {
        if let Some(s) = g.as_mut() {
            s.interface = name.to_string();
            apply_families(ctx, s);
        }
    }
}

/// Rebuilds the Proportional and Monospace families: theme font in front, language fallback at the end.
fn apply_families(ctx: &egui::Context, s: &mut FontState) {
    let base = egui::FontDefinitions::default();
    let mut change = false;
    for fam in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        let mut list = base.families.get(&fam).cloned().unwrap_or_default();
        if fam == egui::FontFamily::Proportional && !s.interface.is_empty() && s.defs.font_data.contains_key(&s.interface) {
            list.insert(0, s.interface.clone());
        }
        if let Some(r) = &s.backup {
            list.push(r.clone());
        }
        if s.defs.families.get(&fam) != Some(&list) {
            s.defs.families.insert(fam, list);
            change = true;
        }
    }
    if change {
        ctx.set_fonts(s.defs.clone());
    }
}

/// System fonts with the characters of each language that egui's fonts do not cover (the first one that exists is used).
pub fn language_fonts(code: &str) -> &'static [&'static str] {
    const NOTO: &str = "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc";
    const NOTO2: &str = "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc";
    match code {
        "zh" => &["C:/Windows/Fonts/msyh.ttc", "C:/Windows/Fonts/simsun.ttc", "/System/Library/Fonts/PingFang.ttc", "/System/Library/Fonts/STHeiti Light.ttc", NOTO, NOTO2],
        "ja" => &["C:/Windows/Fonts/YuGothM.ttc", "C:/Windows/Fonts/meiryo.ttc", "C:/Windows/Fonts/msgothic.ttc", "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc", "/System/Library/Fonts/Hiragino Sans GB.ttc", NOTO, NOTO2],
        "ko" => &["C:/Windows/Fonts/malgun.ttf", "/System/Library/Fonts/AppleSDGothicNeo.ttc", NOTO, NOTO2],
        _ => &[],
    }
}

/// Loads (once) the fallback font for the interface language. Returns false if the language needs it
/// and the system has none (the text would show as boxes).
pub fn language_font(ctx: &egui::Context, code: &str) -> bool {
    let paths = language_fonts(code);
    let Ok(mut g) = FONTS.lock() else { return true };
    let Some(s) = g.as_mut() else { return true };
    let key = format!("language-{}", code);
    let mut ok = true;
    s.backup = if paths.is_empty() {
        None
    } else if s.defs.font_data.contains_key(&key) {
        Some(key)
    } else if let Some((path, bytes)) = paths.iter().find_map(|p| std::fs::read(p).ok().map(|b| (*p, b))) {
        // Yu Gothic and Meiryo draw glyphs higher than egui's Latin font: they are lowered a bit.
        let down = if path.contains("YuGoth") || path.contains("meiryo") { 0.2 } else { 0.0 };
        let tweak = egui::FontTweak { y_offset_factor: down, ..Default::default() };
        s.defs.font_data.insert(key.clone(), Arc::new(egui::FontData::from_owned(bytes).tweak(tweak)));
        Some(key)
    } else {
        ok = false;
        None
    };
    apply_families(ctx, s);
    ok
}

/// Add a font at run time. It becomes selectable from the next frame
/// (egui swaps fonts at the start of a frame).
pub fn add_font(ctx: &egui::Context, name: &str, bytes: Vec<u8>, path: std::path::PathBuf) {
    if let Ok(mut g) = FONTS.lock() {
        if let Some(s) = g.as_mut() {
            if s.available.iter().any(|f| f == name) || s.pending.iter().any(|f| f == name) {
                return;
            }
            s.paths.insert(name.to_string(), path);
            register(&mut s.defs, name, bytes);
            s.pending.push(name.to_string());
            ctx.set_fonts(s.defs.clone());
        }
    }
}

/// Call at the start of each frame: fonts added last frame are now loaded.
pub fn promote_pending_fonts() {
    if let Ok(mut g) = FONTS.lock() {
        if let Some(s) = g.as_mut() {
            let p: Vec<String> = s.pending.drain(..).collect();
            s.available.extend(p);
        }
    }
}

pub fn font_family(name: &str) -> egui::FontFamily {
    if name == "Monospace" {
        egui::FontFamily::Monospace
    } else if !name.is_empty() && FONTS.lock().ok().map(|g| g.as_ref().map(|s| s.available.iter().any(|f| f == name)).unwrap_or(false)).unwrap_or(false) {
        egui::FontFamily::Name(name.into())
    } else {
        egui::FontFamily::Proportional
    }
}

// ---------------------------------------------------------------- pictures

/// Pixel art mode of the current design (image filter on the canvas).
pub static PIXEL_ART: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A cached picture: where it came from (resolved path, modified time, size) and when that was last checked.
struct TexEntry {
    path: std::path::PathBuf,
    stamp: Option<(u128, u64)>,
    checked: std::time::Instant,
    tex: Option<egui::TextureHandle>,
}

static TEXTURES: std::sync::Mutex<Option<std::collections::HashMap<String, TexEntry>>> = std::sync::Mutex::new(None);

/// How often a cached picture is looked at again on disk (the file changed, appeared or was replaced).
const TEXTURE_RECHECK: std::time::Duration = std::time::Duration::from_millis(750);

/// Forget every cached picture (another design was opened: the same relative name may be another file).
pub fn clear_texture_cache() {
    if let Ok(mut g) = TEXTURES.lock() {
        *g = None;
    }
}

fn file_stamp(path: &std::path::Path) -> Option<(u128, u64)> {
    let m = std::fs::metadata(path).ok()?;
    let t = m.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_nanos()).unwrap_or(0);
    Some((t, m.len()))
}

/// Texture of an uploaded picture, cached by resolved path + modified time + size. A picture that is missing is
/// looked for again every moment, so it shows up when the file appears; a changed file is reloaded (None if it can't be read).
pub fn texture(ctx: &egui::Context, name: &str) -> Option<egui::TextureHandle> {
    let nearest = PIXEL_ART.load(std::sync::atomic::Ordering::Relaxed);
    let key = format!("{}|{}", name, nearest);
    let mut g = TEXTURES.lock().ok()?;
    let map = g.get_or_insert_with(Default::default);
    if let Some(ent) = map.get_mut(&key) {
        if ent.checked.elapsed() < TEXTURE_RECHECK {
            return ent.tex.clone();
        }
        let path = crate::library::resolve_image(name);
        let stamp = file_stamp(&path);
        ent.checked = std::time::Instant::now();
        if ent.path == path && ent.stamp == stamp {
            return ent.tex.clone(); // same file (or still missing / still unreadable)
        }
    }
    let path = crate::library::resolve_image(name);
    let stamp = file_stamp(&path);
    let tex = image::open(&path).ok().map(|img| {
        let filter = if nearest { image::imageops::FilterType::Nearest } else { image::imageops::FilterType::Triangle };
        let img = if img.width().max(img.height()) > 2048 { img.resize(2048, 2048, filter) } else { img };
        let rgba = img.to_rgba8();
        let ci = egui::ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw());
        ctx.load_texture(format!("img:{}", key), ci, if nearest { egui::TextureOptions::NEAREST } else { egui::TextureOptions::LINEAR })
    });
    map.insert(key, TexEntry { path, stamp, checked: std::time::Instant::now(), tex: tex.clone() });
    tex
}

fn textured_poly(p: &Painter, xf: &Xf, pts: &[[f32; 2]], hw: f32, hh: f32, tex: egui::TextureId, opacity: f32) {
    let tint = Color32::from_white_alpha((255.0 * opacity.clamp(0.0, 1.0)) as u8);
    let uv = |q: [f32; 2]| Pos2::new(q[0] / (2.0 * hw.max(1e-3)) + 0.5, q[1] / (2.0 * hh.max(1e-3)) + 0.5);
    let mut mesh = Mesh::with_texture(tex);
    mesh.vertices.push(egui::epaint::Vertex { pos: xf.map(0.0, 0.0), uv: Pos2::new(0.5, 0.5), color: tint });
    for q in pts {
        mesh.vertices.push(egui::epaint::Vertex { pos: xf.map(q[0], q[1]), uv: uv(*q), color: tint });
    }
    let n = pts.len() as u32;
    for i in 0..n {
        mesh.add_triangle(0, 1 + i, 1 + (i + 1) % n);
    }
    p.add(Shape::mesh(mesh));
}

/// Size in pixels of a library picture (cached; the texture itself may be downscaled).
pub fn image_dims(name: &str) -> Option<(u32, u32)> {
    static DIMS: std::sync::Mutex<Option<std::collections::HashMap<String, Option<(u32, u32)>>>> = std::sync::Mutex::new(None);
    let mut g = DIMS.lock().ok()?;
    let map = g.get_or_insert_with(Default::default);
    if let Some(d) = map.get(name) {
        return *d;
    }
    let d = crate::library::image_size(&crate::library::resolve_image(name));
    map.insert(name.to_string(), d);
    d
}

/// One piece of a 9-sliced image: destination box (x0, y0, x1, y1) in element-local units measured from the
/// top-left corner of the element, and the part of the picture it shows (u0, v0, u1, v1; v grows downward).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Patch {
    pub dest: [f32; 4],
    pub uv: [f32; 4],
}

/// The nine patches Unity's Image (type Sliced) draws. `slice` is [left, bottom, right, top] in source pixels,
/// `src` the picture size in pixels and `ppu` the sprite's pixels per unit (Unity scales borders by 100 / ppu;
/// the default sprite is 100). When the box is smaller than the borders, the borders shrink proportionally.
/// Empty patches are left out.
pub fn slice_patches(w: f32, h: f32, src: [f32; 2], slice: [f32; 4], ppu: f32) -> Vec<Patch> {
    let (sw, sh) = (src[0].max(1.0), src[1].max(1.0));
    let k = 100.0 / ppu.max(0.01);
    // borders in source pixels, never more than the picture
    let l = slice[0].clamp(0.0, sw);
    let r = slice[2].clamp(0.0, sw - l);
    let b = slice[1].clamp(0.0, sh);
    let t = slice[3].clamp(0.0, sh - b);
    let (mut dl, mut dr, mut dt, mut db) = (l * k, r * k, t * k, b * k);
    if dl + dr > w && dl + dr > 0.0 {
        let s = w.max(0.0) / (dl + dr);
        dl *= s;
        dr *= s;
    }
    if dt + db > h && dt + db > 0.0 {
        let s = h.max(0.0) / (dt + db);
        dt *= s;
        db *= s;
    }
    let xs = [0.0, dl, (w - dr).max(dl), w.max(dl)];
    let us = [0.0, l / sw, 1.0 - r / sw, 1.0];
    let ys = [0.0, dt, (h - db).max(dt), h.max(dt)];
    let vs = [0.0, t / sh, 1.0 - b / sh, 1.0];
    let mut out = Vec::with_capacity(9);
    for j in 0..3 {
        for i in 0..3 {
            if xs[i + 1] - xs[i] <= 1e-4 || ys[j + 1] - ys[j] <= 1e-4 {
                continue;
            }
            out.push(Patch { dest: [xs[i], ys[j], xs[i + 1], ys[j + 1]], uv: [us[i], vs[j], us[i + 1], vs[j + 1]] });
        }
    }
    out
}

/// Box (x0, y0, x1, y1, from the top-left corner) a picture of `src` size takes inside a `w` x `h` box when the
/// aspect ratio is preserved (Image.preserveAspect): fitted and centred.
pub fn fit_aspect(w: f32, h: f32, src: [f32; 2]) -> [f32; 4] {
    if src[0] <= 0.0 || src[1] <= 0.0 || w <= 0.0 || h <= 0.0 {
        return [0.0, 0.0, w, h];
    }
    let s = (w / src[0]).min(h / src[1]);
    let (iw, ih) = (src[0] * s, src[1] * s);
    [(w - iw) / 2.0, (h - ih) / 2.0, (w + iw) / 2.0, (h + ih) / 2.0]
}

/// Draws patches (element-local boxes from the top-left corner) of a texture.
fn textured_patches(p: &Painter, xf: &Xf, hw: f32, hh: f32, tex: egui::TextureId, opacity: f32, patches: &[Patch]) {
    let tint = Color32::from_white_alpha((255.0 * opacity.clamp(0.0, 1.0)) as u8);
    let mut mesh = Mesh::with_texture(tex);
    for pt in patches {
        let [x0, y0, x1, y1] = pt.dest;
        let [u0, v0, u1, v1] = pt.uv;
        let base = mesh.vertices.len() as u32;
        for (x, y, u, v) in [(x0, y0, u0, v0), (x1, y0, u1, v0), (x1, y1, u1, v1), (x0, y1, u0, v1)] {
            mesh.vertices.push(egui::epaint::Vertex { pos: xf.map(x - hw, y - hh), uv: Pos2::new(u, v), color: tint });
        }
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base, base + 2, base + 3);
    }
    p.add(Shape::mesh(mesh));
}

pub fn text_job(e: &Element, text: &str, scale: f32, wrap: f32, color_override: Option<Color32>) -> LayoutJob {
    let mut job = LayoutJob::default();
    job.wrap.max_width = wrap;
    job.halign = match e.align {
        0 => Align::LEFT,
        2 => Align::RIGHT,
        _ => Align::Center,
    };
    let color = color_override.unwrap_or_else(|| col(e.text_color, e.opacity));
    let size = (e.font_size * scale).max(1.0);
    job.append(
        text,
        0.0,
        TextFormat {
            font_id: FontId::new(size, font_family(&e.font)),
            color,
            italics: e.italic,
            underline: if e.underline { Stroke::new((size * 0.06).max(1.0), color) } else { Stroke::NONE },
            ..Default::default()
        },
    );
    job
}

pub fn layout(p: &Painter, job: LayoutJob) -> Arc<egui::Galley> {
    p.layout_job(job)
}

/// Inner text margins (left, right, top/bottom) of each kind, in page units.
pub fn text_padding(e: &Element) -> (f32, f32, f32) {
    match e.kind {
        Kind::Text => (0.0, 0.0, 0.0),
        Kind::Toggle => {
            let s = e.h.min(e.w) * 0.8;
            (s + e.h * 0.1 + 14.0, 12.0, 0.0)
        }
        Kind::Ellipse => (e.w * 0.15, e.w * 0.15, e.h * 0.15),
        Kind::InputField => (18.0, 18.0, 8.0),
        Kind::Dropdown => (18.0, 18.0 + e.h * 0.6, 8.0),
        _ => (16.0, 16.0, 12.0),
    }
}

/// Opacity used for drawing (Selectables that are not interactable look dimmed, like Unity's disabled tint).
fn draw_opacity(e: &Element) -> f32 {
    if e.kind.selectable() && !e.interactable { e.opacity * 0.55 } else { e.opacity }
}

/// Draw text inside the box, honouring horizontal (`align`) and vertical (`valign`) alignment.
fn draw_text(p: &Painter, xf: &Xf, e: &Element, op: f32) {
    let text = e.caption();
    if text.is_empty() {
        return;
    }
    let hw = e.w / 2.0;
    let hh = e.h / 2.0;
    let (pl, pr, pv) = text_padding(e);
    let inner_w = (e.w - pl - pr).max(1.0);
    let color = col(e.text_color, op);
    let galley = layout(p, text_job(e, text, xf.z, inner_w * xf.z, Some(color)));
    let gh = galley.size().y / xf.z;
    let left = -hw + pl;
    let ax = match e.align {
        0 => left,
        2 => left + inner_w,
        _ => left + inner_w / 2.0,
    };
    let ay = if e.kind == Kind::Text {
        -hh
    } else {
        match e.valign {
            0 => -hh + pv,
            2 => hh - pv - gh,
            _ => -gh / 2.0,
        }
    };
    let pos = xf.map(ax, ay);
    let angle = xf.sin.atan2(xf.cos);
    if e.bold {
        let off = xf.dir(e.font_size * xf.z * 0.035, 0.0);
        p.add(TextShape::new(pos + off, galley.clone(), color).with_angle(angle));
    }
    p.add(TextShape::new(pos, galley, color).with_angle(angle));
}

fn shifted(xf: &Xf, dx: f32, dy: f32) -> Xf {
    Xf { c: xf.c + Vec2::new(dx, dy) * xf.z, ..*xf }
}

pub fn paint_element(p: &Painter, e: &Element, v: &View, hide_text: bool) {
    if !e.visible {
        return;
    }
    // A stretched child of a parent smaller than its margins keeps a negative size in the model (so growing the
    // parent again restores the margins); it is drawn as an empty box around the same centre.
    if e.w < 0.0 || e.h < 0.0 {
        let mut c = e.clone();
        let [cx, cy] = e.center();
        c.w = c.w.max(0.0);
        c.h = c.h.max(0.0);
        c.x = cx - c.w / 2.0;
        c.y = cy - c.h / 2.0;
        return paint_element(p, &c, v, hide_text);
    }
    let xf = Xf::of(e, v);
    let hw = e.w / 2.0;
    let hh = e.h / 2.0;
    let op = draw_opacity(e);
    let stroke = Stroke::new(e.stroke_w * v.zoom, col(e.stroke, op));
    let body: Vec<[f32; 2]> = match e.kind {
        Kind::Ellipse => ellipse_pts(hw, hh),
        Kind::Panel | Kind::Button | Kind::InputField | Kind::Dropdown | Kind::ScrollView | Kind::Image => rounded_rect_pts(hw, hh, e.radius),
        _ => Vec::new(),
    };
    // uGUI Shadow effect: a copy of the graphic offset (x right, y up)
    if e.shadow.on && !body.is_empty() {
        let sx = shifted(&xf, e.shadow.offset[0], -e.shadow.offset[1]);
        let a = if e.kind == Kind::Image { 1.0 } else { (e.fill[3] as f32 / 255.0).max(0.2) };
        fill_poly(p, &sx, &body, col(e.shadow.color, op * a), Stroke::NONE);
    }
    match e.kind {
        Kind::Panel | Kind::Button | Kind::InputField | Kind::Ellipse | Kind::ScrollView => {
            fill_poly(p, &xf, &body, col(e.fill, op), stroke);
            if e.kind == Kind::ScrollView {
                // vertical scrollbar
                let bw = 10.0;
                let x = hw - bw / 2.0 - 6.0;
                let len = (hh - 10.0).max(bw);
                let track: Vec<[f32; 2]> = rounded_rect_pts(bw / 2.0, len, bw / 2.0).into_iter().map(|q| [q[0] + x, q[1]]).collect();
                fill_poly(p, &xf, &track, col([0, 0, 0, 25], op), Stroke::NONE);
                let hlen = len * 0.6;
                let handle: Vec<[f32; 2]> = rounded_rect_pts(bw / 2.0, hlen, bw / 2.0).into_iter().map(|q| [q[0] + x, q[1] - len + hlen]).collect();
                fill_poly(p, &xf, &handle, col([0, 0, 0, 70], op), Stroke::NONE);
            }
            if !hide_text {
                draw_text(p, &xf, e, op);
            }
        }
        Kind::Dropdown => {
            fill_poly(p, &xf, &body, col(e.fill, op), stroke);
            let a = e.h * 0.16;
            let cx = hw - e.h * 0.45;
            let tri = vec![xf.map(cx - a, -a * 0.5), xf.map(cx + a, -a * 0.5), xf.map(cx, a * 0.6)];
            p.add(Shape::convex_polygon(tri, col(e.text_color, op), Stroke::NONE));
            if !hide_text {
                draw_text(p, &xf, e, op);
            }
        }
        Kind::Image => {
            match (!e.image.is_empty()).then(|| texture(p.ctx(), &e.image)).flatten() {
                Some(tex) => {
                    let src = image_dims(&e.image).map(|(w, h)| [w as f32, h as f32]);
                    let sliced = e.effective_image_mode() == crate::docext::ImageMode::Sliced;
                    match (src, e.slice) {
                        // 9-slice: the canvas shows what Unity draws
                        (Some(src), Some(sl)) if sliced => textured_patches(p, &xf, hw, hh, tex.id(), op, &slice_patches(e.w, e.h, src, sl, e.ppu.unwrap_or(100.0))),
                        (Some(src), _) if e.preserve_aspect => {
                            let [x0, y0, x1, y1] = fit_aspect(e.w, e.h, src);
                            textured_patches(p, &xf, hw, hh, tex.id(), op, &[Patch { dest: [x0, y0, x1, y1], uv: [0.0, 0.0, 1.0, 1.0] }])
                        }
                        _ => textured_poly(p, &xf, &body, hw, hh, tex.id(), op),
                    }
                }
                None => gradient_poly(p, &xf, &body, hw, hh, e.fill, e.fill2, op),
            }
            if stroke.width > 0.0 {
                fill_poly(p, &xf, &body, Color32::TRANSPARENT, stroke);
            }
            if !hide_text {
                draw_text(p, &xf, e, op);
            }
        }
        Kind::Text => {
            if e.fill[3] > 0 {
                fill_poly(p, &xf, &rounded_rect_pts(hw, hh, e.radius), col(e.fill, op), Stroke::NONE);
            }
            if !hide_text {
                draw_text(p, &xf, e, op);
            }
        }
        Kind::Empty => {}
        Kind::Toggle => {
            let s = e.h.min(e.w) * 0.8;
            let bx = -hw + s / 2.0 + e.h * 0.1;
            let pts: Vec<[f32; 2]> = rounded_rect_pts(s / 2.0, s / 2.0, s * 0.2).into_iter().map(|q| [q[0] + bx, q[1]]).collect();
            if e.value > 0.5 {
                fill_poly(p, &xf, &pts, col(e.fill, op), Stroke::NONE);
                let ck = [[-0.28, 0.0], [-0.08, 0.2], [0.3, -0.2]];
                let line: Vec<Pos2> = ck.iter().map(|q| xf.map(bx + q[0] * s, q[1] * s)).collect();
                p.add(Shape::line(line, Stroke::new((s * 0.1 * v.zoom).max(1.0), col([255, 255, 255, 255], op))));
            } else {
                fill_poly(p, &xf, &pts, col([255, 255, 255, 255], op), Stroke::new((2.0 * v.zoom).max(1.0), col([170, 170, 180, 255], op)));
            }
            if !hide_text {
                draw_text(p, &xf, e, op);
            }
        }
        Kind::Slider => {
            let th = (e.h * 0.32).max(2.0);
            let knob = e.h * 0.5;
            let track_hw = hw - knob;
            let track: Vec<[f32; 2]> = rounded_rect_pts(track_hw.max(1.0), th / 2.0, th / 2.0);
            fill_poly(p, &xf, &track, col(e.fill2, op), Stroke::NONE);
            let v01 = e.value.clamp(0.0, 1.0);
            let fw = (track_hw * 2.0 * v01).max(th);
            let fx = -track_hw + fw / 2.0;
            let filled: Vec<[f32; 2]> = rounded_rect_pts(fw / 2.0, th / 2.0, th / 2.0).into_iter().map(|q| [q[0] + fx, q[1]]).collect();
            fill_poly(p, &xf, &filled, col(e.fill, op), Stroke::NONE);
            let kx = -track_hw + track_hw * 2.0 * v01;
            let kp: Vec<[f32; 2]> = ellipse_pts(knob * 0.8, knob * 0.8).into_iter().map(|q| [q[0] + kx, q[1]]).collect();
            fill_poly(p, &xf, &kp, col([255, 255, 255, 255], op), Stroke::new((2.0 * v.zoom).max(1.0), col(e.fill, op)));
        }
    }
}

/// Paint a whole document scaled into `rect` (used for template thumbnails).
pub fn paint_doc_thumb(p: &Painter, doc: &crate::model::Doc, rect: egui::Rect) {
    let zoom = (rect.width() / doc.width).min(rect.height() / doc.height);
    let size = Vec2::new(doc.width * zoom, doc.height * zoom);
    let origin = rect.center() - size / 2.0;
    let page = egui::Rect::from_min_size(origin, size);
    p.rect_filled(page, 4.0, col(doc.background, 1.0));
    let v = View { origin, zoom };
    let clipped = p.with_clip_rect(page.intersect(p.clip_rect()));
    for e in &doc.elements {
        if doc.shown(e.id) {
            paint_element(&clipped, e, &v, false);
        }
    }
}

/// Paint one element fitted into `rect` (used for element tiles and the drag ghost).
pub fn paint_element_thumb(p: &Painter, e: &Element, rect: egui::Rect) {
    let mut e = e.clone();
    // Library / community elements are untrusted: tiles never paint absurd sizes or glyphs.
    let fin = |v: f32, lo: f32, hi: f32, d: f32| if v.is_finite() { v.clamp(lo, hi) } else { d };
    e.w = fin(e.w, 1.0, crate::model::COORD_LIMIT, 100.0);
    e.h = fin(e.h, 1.0, crate::model::COORD_LIMIT, 100.0);
    e.font_size = fin(e.font_size, 1.0, 1000.0, 32.0);
    e.stroke_w = fin(e.stroke_w, 0.0, 1000.0, 0.0);
    e.radius = fin(e.radius, 0.0, 10_000.0, 0.0);
    let zoom = (rect.width() / e.w).min(rect.height() / e.h).min(1.0);
    e.rot = 0.0;
    e.x = 0.0;
    e.y = 0.0;
    let size = Vec2::new(e.w * zoom, e.h * zoom);
    let origin = rect.center() - size / 2.0;
    paint_element(p, &e, &View { origin, zoom }, false);
}

/// Checkerboard drawn behind a transparent page (background alpha 0 = transparent overlay).
pub fn paint_checkerboard(p: &Painter, rect: egui::Rect) {
    p.rect_filled(rect, 0.0, Color32::from_gray(205));
    let dark = Color32::from_gray(170);
    // Only the visible part is tiled; when it would need too many tiles (huge zoom or a huge page) the tile size is
    // doubled until it fits, so the pattern never disappears.
    let vis = rect.intersect(p.clip_rect());
    if !vis.is_positive() {
        return;
    }
    let mut cell = 12.0f32;
    while (vis.width() / cell).ceil() * (vis.height() / cell).ceil() > 20_000.0 {
        cell *= 2.0;
    }
    let (i0, j0) = (((vis.min.x - rect.min.x) / cell).floor() as i32, ((vis.min.y - rect.min.y) / cell).floor() as i32);
    let (i1, j1) = (((vis.max.x - rect.min.x) / cell).ceil() as i32, ((vis.max.y - rect.min.y) / cell).ceil() as i32);
    for j in j0..j1 {
        for i in i0..i1 {
            if (i + j).rem_euclid(2) == 0 {
                continue;
            }
            let min = rect.min + Vec2::new(i as f32 * cell, j as f32 * cell);
            p.rect_filled(egui::Rect::from_min_max(min, (min + Vec2::splat(cell)).min(rect.max)), 0.0, dark);
        }
    }
}

#[cfg(test)]
mod slice_tests {
    use super::*;

    fn area(ps: &[Patch]) -> f32 {
        ps.iter().map(|p| (p.dest[2] - p.dest[0]) * (p.dest[3] - p.dest[1])).sum()
    }

    #[test]
    fn nine_patches_cover_the_box_and_the_whole_picture() {
        let ps = slice_patches(200.0, 100.0, [64.0, 64.0], [16.0, 8.0, 16.0, 8.0], 100.0);
        assert_eq!(ps.len(), 9);
        assert!((area(&ps) - 200.0 * 100.0).abs() < 0.01);
        let uv: f32 = ps.iter().map(|p| (p.uv[2] - p.uv[0]) * (p.uv[3] - p.uv[1])).sum();
        assert!((uv - 1.0).abs() < 1e-4);
        // top-left corner: 16 px wide, top border 8 px (border order is left, bottom, right, top)
        let tl = ps[0];
        assert_eq!(tl.dest, [0.0, 0.0, 16.0, 8.0]);
        assert!((tl.uv[2] - 16.0 / 64.0).abs() < 1e-5 && (tl.uv[3] - 8.0 / 64.0).abs() < 1e-5);
        // bottom-right corner takes the right and bottom borders
        let br = ps[8];
        assert_eq!(br.dest, [184.0, 92.0, 200.0, 100.0]);
        assert!((br.uv[0] - 48.0 / 64.0).abs() < 1e-5 && (br.uv[1] - 56.0 / 64.0).abs() < 1e-5);
    }

    #[test]
    fn borders_follow_ppu_and_shrink_when_the_box_is_small() {
        // ppu 200 halves the borders on the canvas
        let ps = slice_patches(200.0, 100.0, [64.0, 64.0], [16.0, 16.0, 16.0, 16.0], 200.0);
        assert_eq!(ps[0].dest, [0.0, 0.0, 8.0, 8.0]);
        // a box narrower than left + right: borders shrink proportionally, the centre column disappears
        let ps = slice_patches(20.0, 100.0, [64.0, 64.0], [16.0, 0.0, 16.0, 0.0], 100.0);
        assert!((area(&ps) - 2000.0).abs() < 0.01);
        assert!(ps.iter().all(|p| p.dest[2] <= 20.0 + 1e-4));
        assert_eq!(ps.len(), 2);
        assert_eq!(ps[0].dest, [0.0, 0.0, 10.0, 100.0]);
        // no slice borders at all: one patch that is the whole picture
        let ps = slice_patches(50.0, 40.0, [10.0, 10.0], [0.0; 4], 100.0);
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0].uv, [0.0, 0.0, 1.0, 1.0]);
    }

    #[test]
    fn preserve_aspect_fits_and_centres() {
        let r = fit_aspect(200.0, 100.0, [50.0, 50.0]);
        assert_eq!(r, [50.0, 0.0, 150.0, 100.0]);
        assert_eq!(fit_aspect(100.0, 100.0, [0.0, 0.0]), [0.0, 0.0, 100.0, 100.0]);
    }
}

// ---- Lienzo export: system font table shared with the headless exporter (marked block)
/// Well-known system fonts and the files they can live in (the first that exists is used).
pub fn system_font_candidates() -> &'static [(&'static str, &'static [&'static str])] {
    &[
        ("Arial", &["C:/Windows/Fonts/arial.ttf", "/System/Library/Fonts/Supplemental/Arial.ttf", "/usr/share/fonts/truetype/msttcorefonts/Arial.ttf", "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf"]),
        ("Segoe UI", &["C:/Windows/Fonts/segoeui.ttf"]),
        ("Verdana", &["C:/Windows/Fonts/verdana.ttf", "/System/Library/Fonts/Supplemental/Verdana.ttf"]),
        ("Tahoma", &["C:/Windows/Fonts/tahoma.ttf", "/System/Library/Fonts/Supplemental/Tahoma.ttf"]),
        ("Trebuchet MS", &["C:/Windows/Fonts/trebuc.ttf", "/System/Library/Fonts/Supplemental/Trebuchet MS.ttf"]),
        ("Georgia", &["C:/Windows/Fonts/georgia.ttf", "/System/Library/Fonts/Supplemental/Georgia.ttf"]),
        ("Times New Roman", &["C:/Windows/Fonts/times.ttf", "/System/Library/Fonts/Supplemental/Times New Roman.ttf", "/usr/share/fonts/truetype/liberation/LiberationSerif-Regular.ttf"]),
        ("Courier New", &["C:/Windows/Fonts/cour.ttf", "/System/Library/Fonts/Supplemental/Courier New.ttf", "/usr/share/fonts/truetype/liberation/LiberationMono-Regular.ttf"]),
        ("Consolas", &["C:/Windows/Fonts/consola.ttf"]),
        ("Impact", &["C:/Windows/Fonts/impact.ttf", "/System/Library/Fonts/Supplemental/Impact.ttf"]),
        ("Comic Sans MS", &["C:/Windows/Fonts/comic.ttf", "/System/Library/Fonts/Supplemental/Comic Sans MS.ttf"]),
        ("DejaVu Sans", &["/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf"]),
    ]
}
