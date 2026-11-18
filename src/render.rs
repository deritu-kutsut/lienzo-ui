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
    /// Fuente de respaldo del idioma de la interfaz (chino, japonés, coreano), al final de las listas.
    respaldo: Option<String>,
    /// Fuente de la interfaz elegida en el tema ("" = la predeterminada).
    interfaz: String,
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
    let candidates: &[(&str, &[&str])] = &[
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
    ];
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
        *g = Some(FontState { defs, available: found, pending: Vec::new(), paths, respaldo: None, interfaz: String::new() });
    }
}

/// Fuente de la interfaz de la app (temas): `nombre` delante de las predeterminadas; "" o una que no está = la predeterminada.
pub fn fuente_interfaz(ctx: &egui::Context, nombre: &str) {
    if let Ok(mut g) = FONTS.lock() {
        if let Some(s) = g.as_mut() {
            s.interfaz = nombre.to_string();
            aplicar_familias(ctx, s);
        }
    }
}

/// Rehace las familias Proportional y Monospace: fuente del tema delante, respaldo del idioma al final.
fn aplicar_familias(ctx: &egui::Context, s: &mut FontState) {
    let base = egui::FontDefinitions::default();
    let mut cambio = false;
    for fam in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        let mut lista = base.families.get(&fam).cloned().unwrap_or_default();
        if fam == egui::FontFamily::Proportional && !s.interfaz.is_empty() && s.defs.font_data.contains_key(&s.interfaz) {
            lista.insert(0, s.interfaz.clone());
        }
        if let Some(r) = &s.respaldo {
            lista.push(r.clone());
        }
        if s.defs.families.get(&fam) != Some(&lista) {
            s.defs.families.insert(fam, lista);
            cambio = true;
        }
    }
    if cambio {
        ctx.set_fonts(s.defs.clone());
    }
}

/// Fuentes del sistema con los caracteres de cada idioma que no cubren las de egui (se busca la primera que exista).
pub fn fuentes_de_idioma(codigo: &str) -> &'static [&'static str] {
    const NOTO: &str = "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc";
    const NOTO2: &str = "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc";
    match codigo {
        "zh" => &["C:/Windows/Fonts/msyh.ttc", "C:/Windows/Fonts/simsun.ttc", "/System/Library/Fonts/PingFang.ttc", "/System/Library/Fonts/STHeiti Light.ttc", NOTO, NOTO2],
        "ja" => &["C:/Windows/Fonts/YuGothM.ttc", "C:/Windows/Fonts/meiryo.ttc", "C:/Windows/Fonts/msgothic.ttc", "/System/Library/Fonts/ヒラギノ角ゴシック W3.ttc", "/System/Library/Fonts/Hiragino Sans GB.ttc", NOTO, NOTO2],
        "ko" => &["C:/Windows/Fonts/malgun.ttf", "/System/Library/Fonts/AppleSDGothicNeo.ttc", NOTO, NOTO2],
        _ => &[],
    }
}

/// Carga (una vez) la fuente de respaldo del idioma de la interfaz. Devuelve false si el idioma la necesita
/// y no hay ninguna en el sistema (los textos se verían como cuadros).
pub fn fuente_idioma(ctx: &egui::Context, codigo: &str) -> bool {
    let rutas = fuentes_de_idioma(codigo);
    let Ok(mut g) = FONTS.lock() else { return true };
    let Some(s) = g.as_mut() else { return true };
    let clave = format!("idioma-{}", codigo);
    let mut ok = true;
    s.respaldo = if rutas.is_empty() {
        None
    } else if s.defs.font_data.contains_key(&clave) {
        Some(clave)
    } else if let Some((ruta, bytes)) = rutas.iter().find_map(|p| std::fs::read(p).ok().map(|b| (*p, b))) {
        // Yu Gothic y Meiryo dibujan los glifos más arriba que la fuente latina de egui: se bajan un poco.
        let bajar = if ruta.contains("YuGoth") || ruta.contains("meiryo") { 0.2 } else { 0.0 };
        let tweak = egui::FontTweak { y_offset_factor: bajar, ..Default::default() };
        s.defs.font_data.insert(clave.clone(), Arc::new(egui::FontData::from_owned(bytes).tweak(tweak)));
        Some(clave)
    } else {
        ok = false;
        None
    };
    aplicar_familias(ctx, s);
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
    if name == "Monoespaciada" {
        egui::FontFamily::Monospace
    } else if !name.is_empty() && FONTS.lock().ok().map(|g| g.as_ref().map(|s| s.available.iter().any(|f| f == name)).unwrap_or(false)).unwrap_or(false) {
        egui::FontFamily::Name(name.into())
    } else {
        egui::FontFamily::Proportional
    }
}

// ---------------------------------------------------------------- pictures

/// Modo pixel art del diseño actual (filtro de las imágenes en el lienzo).
pub static PIXEL_ART: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

static TEXTURES: std::sync::Mutex<Option<std::collections::HashMap<String, Option<egui::TextureHandle>>>> = std::sync::Mutex::new(None);

/// Texture of an uploaded picture, loaded once and cached (None if it can't be read).
pub fn texture(ctx: &egui::Context, name: &str) -> Option<egui::TextureHandle> {
    let nearest = PIXEL_ART.load(std::sync::atomic::Ordering::Relaxed);
    let key = format!("{}|{}", name, nearest);
    let mut g = TEXTURES.lock().ok()?;
    let map = g.get_or_insert_with(Default::default);
    if let Some(t) = map.get(&key) {
        return t.clone();
    }
    let path = crate::library::resolve_image(name);
    let tex = image::open(&path).ok().map(|img| {
        let filtro = if nearest { image::imageops::FilterType::Nearest } else { image::imageops::FilterType::Triangle };
        let img = if img.width().max(img.height()) > 2048 { img.resize(2048, 2048, filtro) } else { img };
        let rgba = img.to_rgba8();
        let ci = egui::ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw());
        ctx.load_texture(format!("img:{}", key), ci, if nearest { egui::TextureOptions::NEAREST } else { egui::TextureOptions::LINEAR })
    });
    map.insert(key, tex.clone());
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
                Some(tex) => textured_poly(p, &xf, &body, hw, hh, tex.id(), op),
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
    let zoom = (rect.width() / e.w).min(rect.height() / e.h).min(1.0);
    let mut e = e.clone();
    e.rot = 0.0;
    e.x = 0.0;
    e.y = 0.0;
    let size = Vec2::new(e.w * zoom, e.h * zoom);
    let origin = rect.center() - size / 2.0;
    paint_element(p, &e, &View { origin, zoom }, false);
}