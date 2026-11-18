//! Interface themes: colors, corners and font of the app. Everything the interface draws takes its colors from here.
//!
//! Colors are read through functions (`MUTED()`, `ACCENT()`…) because they change live when another theme is chosen. A theme is a
//! small JSON (`Tema`) saved in the settings, in the library (`temas/`) and shared in the community.
#![allow(non_snake_case)]

use eframe::egui::{self, Color32, CornerRadius, Stroke};
use std::sync::atomic::{AtomicU32, Ordering};

/// A theme: colors as `#RRGGBB`, corner radius and interface font ("" = the default one).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Tema {
    pub nombre: String,
    pub autor: String,
    /// Light base (dark text on light backgrounds) or dark.
    pub claro: bool,
    pub fondo: String,
    pub panel: String,
    pub panel_oscuro: String,
    pub campo: String,
    pub hover: String,
    pub linea: String,
    pub borde: String,
    pub texto: String,
    pub apagado: String,
    pub acento: String,
    pub acento_suave: String,
    pub ok: String,
    pub aviso: String,
    pub guia: String,
    pub esquinas: f32,
    pub fuente: String,
}

impl Default for Tema {
    fn default() -> Self {
        predeterminados()[0].clone()
    }
}

/// The editable colors, in order: (JSON field, display name).
pub const COLORES: &[(&str, &str)] = &[
    ("fondo", "Fondo"),
    ("panel", "Paneles"),
    ("panel_oscuro", "Paneles oscuros"),
    ("campo", "Campos y botones"),
    ("hover", "Al pasar el ratón"),
    ("linea", "Líneas"),
    ("borde", "Bordes"),
    ("texto", "Texto"),
    ("apagado", "Texto secundario"),
    ("acento", "Acento"),
    ("acento_suave", "Acento suave"),
    ("ok", "Correcto"),
    ("aviso", "Aviso"),
    ("guia", "Guías del lienzo"),
];

impl Tema {
    pub fn color_mut(&mut self, campo: &str) -> Option<&mut String> {
        Some(match campo {
            "fondo" => &mut self.fondo,
            "panel" => &mut self.panel,
            "panel_oscuro" => &mut self.panel_oscuro,
            "campo" => &mut self.campo,
            "hover" => &mut self.hover,
            "linea" => &mut self.linea,
            "borde" => &mut self.borde,
            "texto" => &mut self.texto,
            "apagado" => &mut self.apagado,
            "acento" => &mut self.acento,
            "acento_suave" => &mut self.acento_suave,
            "ok" => &mut self.ok,
            "aviso" => &mut self.aviso,
            "guia" => &mut self.guia,
            _ => return None,
        })
    }

    pub fn color(&self, campo: &str) -> Color32 {
        let mut t = self.clone();
        t.color_mut(campo).and_then(|c| parse_hex(c)).unwrap_or(Color32::MAGENTA)
    }

    /// Checks a theme that came from outside (file or community): `#RRGGBB` colors, reasonable name and font.
    pub fn validar(&self) -> Result<(), String> {
        let n = self.nombre.trim().chars().count();
        if !(1..=60).contains(&n) {
            return Err("El tema necesita un nombre de 1 a 60 caracteres.".into());
        }
        if self.autor.chars().count() > 60 {
            return Err("El autor del tema es demasiado largo.".into());
        }
        let mut t = self.clone();
        for (campo, nombre) in COLORES {
            let v = t.color_mut(campo).map(|c| c.clone()).unwrap_or_default();
            if parse_hex(&v).is_none() {
                return Err(format!("Color no válido en «{}»: usa #RRGGBB.", nombre));
            }
        }
        if !self.esquinas.is_finite() || !(0.0..=16.0).contains(&self.esquinas) {
            return Err("Las esquinas deben estar entre 0 y 16.".into());
        }
        if self.fuente.chars().count() > 60 || !self.fuente.chars().all(|c| c.is_alphanumeric() || " -_.".contains(c)) || self.fuente.contains("..") {
            return Err("Nombre de fuente no válido.".into());
        }
        Ok(())
    }
}

pub fn parse_hex(s: &str) -> Option<Color32> {
    let h = s.strip_prefix('#')?;
    if h.len() != 6 || !h.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let v = u32::from_str_radix(h, 16).ok()?;
    Some(Color32::from_rgb((v >> 16) as u8, (v >> 8) as u8, v as u8))
}

pub fn a_hex(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

fn tema(nombre: &str, claro: bool, c: [&str; 14], esquinas: f32) -> Tema {
    Tema {
        nombre: nombre.into(),
        autor: "Lienzo UI".into(),
        claro,
        fondo: c[0].into(),
        panel: c[1].into(),
        panel_oscuro: c[2].into(),
        campo: c[3].into(),
        hover: c[4].into(),
        linea: c[5].into(),
        borde: c[6].into(),
        texto: c[7].into(),
        apagado: c[8].into(),
        acento: c[9].into(),
        acento_suave: c[10].into(),
        ok: c[11].into(),
        aviso: c[12].into(),
        guia: c[13].into(),
        esquinas,
        fuente: String::new(),
    }
}

/// Themes bundled with the app.
pub fn predeterminados() -> Vec<Tema> {
    vec![
        tema("Lienzo oscuro", false, ["#1D1E23", "#26272E", "#202127", "#30323B", "#3A3D49", "#393C47", "#16171B", "#DEE0E7", "#9296A5", "#4D8EFF", "#273D68", "#40BA70", "#FFB840", "#FF3EB5"], 5.0),
        tema("Lienzo claro", true, ["#E9EBF0", "#F7F8FA", "#EEF0F4", "#E2E5EC", "#D6DAE3", "#CDD2DC", "#B9BFCC", "#1E2230", "#5D6476", "#2F6FE4", "#CFDDFB", "#1F9D57", "#B86E00", "#D6248F"], 5.0),
        tema("Medianoche", false, ["#0E1220", "#151B2E", "#111626", "#1D2540", "#27315A", "#26304F", "#0A0D17", "#E4E8F7", "#8B93B3", "#7C9CFF", "#253366", "#3CCB8B", "#FFC857", "#FF5CC8"], 7.0),
        tema("Atardecer", false, ["#1E1714", "#29201B", "#231B17", "#352922", "#45352B", "#433328", "#150F0C", "#F4E7DC", "#B39C8B", "#FF9A2E", "#5A3415", "#5FC27E", "#FFD15C", "#FF5E8A"], 6.0),
        tema("Bosque", false, ["#141A16", "#1B231D", "#171E19", "#24302A", "#2E3D34", "#2F3D33", "#0E130F", "#E2EDE5", "#93A79A", "#52C27C", "#1E4A30", "#7AD69A", "#F2C14E", "#E26FD0"], 4.0),
    ]
}

// ───────── theme in use (colors read thousands of times per frame: atomic, lock-free) ─────────

const N: usize = 14;
static ACTUAL: [AtomicU32; N] = [const { AtomicU32::new(0) }; N];
static ESQUINAS: AtomicU32 = AtomicU32::new(5);
static CLARO: AtomicU32 = AtomicU32::new(0);
static INICIADO: AtomicU32 = AtomicU32::new(0);

fn leer(i: usize) -> Color32 {
    if INICIADO.load(Ordering::Relaxed) == 0 {
        usar(&Tema::default());
    }
    let [r, g, b, _] = ACTUAL[i].load(Ordering::Relaxed).to_be_bytes();
    Color32::from_rgb(r, g, b)
}

/// Sets a theme in use (colors change immediately; then call `apply` for the egui styles).
pub fn usar(t: &Tema) {
    for (i, (campo, _)) in COLORES.iter().enumerate() {
        let c = t.color(campo);
        ACTUAL[i].store(u32::from_be_bytes([c.r(), c.g(), c.b(), 255]), Ordering::Relaxed);
    }
    ESQUINAS.store(t.esquinas.clamp(0.0, 16.0).round() as u32, Ordering::Relaxed);
    CLARO.store(t.claro as u32, Ordering::Relaxed);
    INICIADO.store(1, Ordering::Relaxed);
}

pub fn es_claro() -> bool {
    CLARO.load(Ordering::Relaxed) == 1
}

pub fn BACKDROP() -> Color32 {
    leer(0)
}
pub fn PANEL() -> Color32 {
    leer(1)
}
pub fn PANEL_DARK() -> Color32 {
    leer(2)
}
pub fn FIELD() -> Color32 {
    leer(3)
}
pub fn HOVER() -> Color32 {
    leer(4)
}
pub fn LINE() -> Color32 {
    leer(5)
}
pub fn BORDER() -> Color32 {
    leer(6)
}
pub fn TEXT() -> Color32 {
    leer(7)
}
pub fn MUTED() -> Color32 {
    leer(8)
}
pub fn ACCENT() -> Color32 {
    leer(9)
}
pub fn ACCENT_SOFT() -> Color32 {
    leer(10)
}
pub fn OK() -> Color32 {
    leer(11)
}
pub fn WARN() -> Color32 {
    leer(12)
}
pub fn GUIDE() -> Color32 {
    leer(13)
}

/// Applies the theme in use. With `alto_contraste`: white text, strong borders and a more visible selection.
pub fn apply(ctx: &egui::Context, alto_contraste: bool) {
    let claro = es_claro() && !alto_contraste;
    let r = ESQUINAS.load(Ordering::Relaxed).min(16) as u8;
    let fuerte = if claro { Color32::BLACK } else { Color32::WHITE };
    ctx.set_visuals(if claro { egui::Visuals::light() } else { egui::Visuals::dark() });
    ctx.all_styles_mut(|s| {
        s.spacing.item_spacing = egui::vec2(6.0, 5.0);
        s.spacing.button_padding = egui::vec2(8.0, 4.0);
        s.spacing.interact_size.y = 24.0;
        let v = &mut s.visuals;
        v.panel_fill = PANEL();
        v.window_fill = PANEL();
        v.window_stroke = Stroke::new(1.0, LINE());
        v.extreme_bg_color = PANEL_DARK();
        v.faint_bg_color = PANEL_DARK();
        v.text_edit_bg_color = Some(PANEL_DARK());
        v.override_text_color = None;
        v.selection.bg_fill = ACCENT_SOFT();
        v.selection.stroke = Stroke::new(1.0, ACCENT());
        v.hyperlink_color = ACCENT();
        v.window_corner_radius = CornerRadius::same(r.saturating_add(3));
        v.menu_corner_radius = CornerRadius::same(r.saturating_add(1));
        for (w, fill) in [
            (&mut v.widgets.noninteractive, PANEL()),
            (&mut v.widgets.inactive, FIELD()),
            (&mut v.widgets.hovered, HOVER()),
            (&mut v.widgets.active, ACCENT_SOFT()),
            (&mut v.widgets.open, HOVER()),
        ] {
            w.corner_radius = CornerRadius::same(r);
            w.bg_fill = fill;
            w.weak_bg_fill = fill;
        }
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, LINE());
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, TEXT());
        v.widgets.inactive.fg_stroke = Stroke::new(1.0, TEXT());
        v.widgets.inactive.bg_stroke = Stroke::NONE;
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, fuerte);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, LINE());
        v.widgets.active.fg_stroke = Stroke::new(1.0, fuerte);
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT());
        if alto_contraste {
            v.override_text_color = Some(Color32::WHITE);
            v.panel_fill = Color32::from_rgb(12, 12, 14);
            v.window_fill = Color32::from_rgb(12, 12, 14);
            v.window_stroke = Stroke::new(2.0, Color32::WHITE);
            v.selection.bg_fill = Color32::from_rgb(0, 90, 220);
            v.selection.stroke = Stroke::new(2.0, Color32::from_rgb(255, 220, 0));
            for w in [&mut v.widgets.noninteractive, &mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active, &mut v.widgets.open] {
                w.fg_stroke = Stroke::new(1.5, Color32::WHITE);
                w.bg_stroke = Stroke::new(1.5, Color32::from_gray(200));
            }
            v.widgets.inactive.bg_fill = Color32::from_rgb(30, 30, 36);
            v.widgets.inactive.weak_bg_fill = Color32::from_rgb(30, 30, 36);
            v.widgets.hovered.bg_stroke = Stroke::new(2.0, Color32::from_rgb(255, 220, 0));
            v.widgets.active.bg_stroke = Stroke::new(2.0, Color32::from_rgb(255, 220, 0));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn los_temas_que_trae_la_app_son_validos() {
        for t in predeterminados() {
            assert!(t.validar().is_ok(), "{}: {:?}", t.nombre, t.validar());
        }
    }

    #[test]
    fn el_predeterminado_conserva_los_colores_de_siempre() {
        let t = Tema::default();
        assert_eq!(t.color("panel"), Color32::from_rgb(38, 39, 46));
        assert_eq!(t.color("acento"), Color32::from_rgb(77, 142, 255));
    }

    #[test]
    fn validacion_de_temas_de_fuera() {
        let mut t = Tema::default();
        t.acento = "rojo".into();
        assert!(t.validar().is_err());
        let mut t = Tema::default();
        t.fuente = "../../malo".into();
        assert!(t.validar().is_err());
        let mut t = Tema::default();
        t.esquinas = 99.0;
        assert!(t.validar().is_err());
        assert!(serde_json::from_str::<Tema>(r#"{"nombre":"x","script":"malo"}"#).is_err(), "campos desconocidos se rechazan");
    }

    #[test]
    fn hex_ida_y_vuelta() {
        assert_eq!(a_hex(parse_hex("#4d8eff").unwrap()), "#4D8EFF");
        assert!(parse_hex("#12345").is_none());
        assert!(parse_hex("123456").is_none());
    }
}
