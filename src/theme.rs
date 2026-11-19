//! Interface themes: colors, corners and font of the app. Everything the interface draws takes its colors from here.
//!
//! Colors are read through functions (`MUTED()`, `ACCENT()`…) because they change live when another theme is chosen. A theme is a
//! small JSON (`Theme`) saved in the settings, in the library (`themes/`) and shared in the community.
#![allow(non_snake_case)]

use eframe::egui::{self, Color32, CornerRadius, Stroke};
use std::sync::atomic::{AtomicU32, Ordering};

/// A theme: colors as `#RRGGBB`, corner radius and interface font ("" = the default one).
/// Theme files written by older versions (Spanish field names: `nombre`, `fondo`, `acento`…) still load through the serde aliases.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    #[serde(alias = "nombre")]
    pub name: String,
    #[serde(alias = "autor")]
    pub author: String,
    /// Light base (dark text on light backgrounds) or dark.
    #[serde(alias = "claro")]
    pub light: bool,
    #[serde(alias = "fondo")]
    pub background: String,
    pub panel: String,
    #[serde(alias = "panel_oscuro")]
    pub dark_panel: String,
    #[serde(alias = "campo")]
    pub field: String,
    pub hover: String,
    #[serde(alias = "linea")]
    pub line: String,
    #[serde(alias = "borde")]
    pub border: String,
    #[serde(alias = "texto")]
    pub text: String,
    #[serde(alias = "apagado")]
    pub muted: String,
    #[serde(alias = "acento")]
    pub accent: String,
    #[serde(alias = "acento_suave")]
    pub soft_accent: String,
    pub ok: String,
    #[serde(alias = "aviso")]
    pub warning: String,
    #[serde(alias = "guia")]
    pub guide: String,
    #[serde(alias = "esquinas")]
    pub corners: f32,
    #[serde(alias = "fuente")]
    pub font: String,
}

impl Default for Theme {
    fn default() -> Self {
        defaults()[0].clone()
    }
}

/// The editable colors, in order: (JSON field, display name).
pub const COLORS: &[(&str, &str)] = &[
    ("background", "Background"),
    ("panel", "Panels"),
    ("dark_panel", "Dark panels"),
    ("field", "Fields and buttons"),
    ("hover", "Hover"),
    ("line", "Lines"),
    ("border", "Borders"),
    ("text", "Text"),
    ("muted", "Secondary text"),
    ("accent", "Accent"),
    ("soft_accent", "Soft accent"),
    ("ok", "Success"),
    ("warning", "Warning"),
    ("guide", "Canvas guides"),
];

impl Theme {
    pub fn color_mut(&mut self, field: &str) -> Option<&mut String> {
        Some(match field {
            "background" => &mut self.background,
            "panel" => &mut self.panel,
            "dark_panel" => &mut self.dark_panel,
            "field" => &mut self.field,
            "hover" => &mut self.hover,
            "line" => &mut self.line,
            "border" => &mut self.border,
            "text" => &mut self.text,
            "muted" => &mut self.muted,
            "accent" => &mut self.accent,
            "soft_accent" => &mut self.soft_accent,
            "ok" => &mut self.ok,
            "warning" => &mut self.warning,
            "guide" => &mut self.guide,
            _ => return None,
        })
    }

    pub fn color(&self, field: &str) -> Color32 {
        let mut t = self.clone();
        t.color_mut(field).and_then(|c| parse_hex(c)).unwrap_or(Color32::MAGENTA)
    }

    /// Checks a theme that came from outside (file or community): `#RRGGBB` colors, reasonable name and font.
    pub fn validate(&self) -> Result<(), String> {
        let n = self.name.trim().chars().count();
        if !(1..=60).contains(&n) {
            return Err("The theme needs a name of 1 to 60 characters.".into());
        }
        if self.author.chars().count() > 60 {
            return Err("The theme author is too long.".into());
        }
        if !crate::community_validate::valid_display_text(&self.name) || !crate::community_validate::valid_display_text(&self.author) {
            return Err("The theme name or author has control or invisible characters.".into());
        }
        let mut t = self.clone();
        for (field, name) in COLORS {
            let v = t.color_mut(field).map(|c| c.clone()).unwrap_or_default();
            if parse_hex(&v).is_none() {
                return Err(format!("Invalid color in “{}”: use #RRGGBB.", name));
            }
        }
        if !self.corners.is_finite() || !(0.0..=16.0).contains(&self.corners) {
            return Err("Corners must be between 0 and 16.".into());
        }
        if self.font.chars().count() > 60 || !self.font.chars().all(|c| c.is_alphanumeric() || " -_.".contains(c)) || self.font.contains("..") {
            return Err("Invalid font name.".into());
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

pub fn to_hex(c: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", c.r(), c.g(), c.b())
}

fn theme(name: &str, light: bool, c: [&str; 14], corners: f32) -> Theme {
    Theme {
        name: name.into(),
        author: "Lienzo UI".into(),
        light,
        background: c[0].into(),
        panel: c[1].into(),
        dark_panel: c[2].into(),
        field: c[3].into(),
        hover: c[4].into(),
        line: c[5].into(),
        border: c[6].into(),
        text: c[7].into(),
        muted: c[8].into(),
        accent: c[9].into(),
        soft_accent: c[10].into(),
        ok: c[11].into(),
        warning: c[12].into(),
        guide: c[13].into(),
        corners,
        font: String::new(),
    }
}

/// Old (Spanish) names of the bundled themes, as (old, new).
const LEGACY_NAMES: &[(&str, &str)] = &[("Lienzo oscuro", "Lienzo Dark"), ("Lienzo claro", "Lienzo Light"), ("Medianoche", "Midnight")];

/// The current name of a bundled theme given its old Spanish name (other names are returned unchanged).
pub fn canonical_name(name: &str) -> &str {
    LEGACY_NAMES.iter().find(|(old, _)| *old == name).map(|(_, new)| *new).unwrap_or(name)
}

/// A bundled theme by name (the old Spanish names also work).
pub fn find_default(name: &str) -> Option<Theme> {
    let name = canonical_name(name);
    defaults().into_iter().find(|t| t.name == name)
}

/// Themes bundled with the app.
pub fn defaults() -> Vec<Theme> {
    vec![
        theme("Lienzo Dark", false, ["#1D1E23", "#26272E", "#202127", "#30323B", "#3A3D49", "#393C47", "#16171B", "#DEE0E7", "#9296A5", "#4D8EFF", "#273D68", "#40BA70", "#FFB840", "#FF3EB5"], 5.0),
        theme("Lienzo Light", true, ["#E9EBF0", "#F7F8FA", "#EEF0F4", "#E2E5EC", "#D6DAE3", "#CDD2DC", "#B9BFCC", "#1E2230", "#5D6476", "#2F6FE4", "#CFDDFB", "#1F9D57", "#B86E00", "#D6248F"], 5.0),
        theme("Midnight", false, ["#0E1220", "#151B2E", "#111626", "#1D2540", "#27315A", "#26304F", "#0A0D17", "#E4E8F7", "#8B93B3", "#7C9CFF", "#253366", "#3CCB8B", "#FFC857", "#FF5CC8"], 7.0),
        theme("Sunset", false, ["#1E1714", "#29201B", "#231B17", "#352922", "#45352B", "#433328", "#150F0C", "#F4E7DC", "#B39C8B", "#FF9A2E", "#5A3415", "#5FC27E", "#FFD15C", "#FF5E8A"], 6.0),
        theme("Forest", false, ["#141A16", "#1B231D", "#171E19", "#24302A", "#2E3D34", "#2F3D33", "#0E130F", "#E2EDE5", "#93A79A", "#52C27C", "#1E4A30", "#7AD69A", "#F2C14E", "#E26FD0"], 4.0),
    ]
}

// ───────── theme in use (colors read thousands of times per frame: atomic, lock-free) ─────────

const N: usize = 14;
static CURRENT: [AtomicU32; N] = [const { AtomicU32::new(0) }; N];
static CORNERS: AtomicU32 = AtomicU32::new(5);
static LIGHT: AtomicU32 = AtomicU32::new(0);
static STARTED: AtomicU32 = AtomicU32::new(0);
/// High contrast requested (set by `apply`). With a light theme it switches every color to `HC_PALETTE`.
static HIGH_CONTRAST: AtomicU32 = AtomicU32::new(0);

/// Dark palette with white text used by high contrast on top of a light theme, so the colors drawn
/// directly by the interface (`TEXT()`, `MUTED()`, `ACCENT()`...) stay readable on the dark panels `apply` paints.
/// Same order as `COLORS`.
const HC_PALETTE: [[u8; 3]; N] = [
    [8, 8, 10],      // background
    [12, 12, 14],    // panel
    [4, 4, 6],       // dark panel
    [30, 30, 36],    // field
    [52, 52, 62],    // hover
    [170, 170, 180], // line
    [220, 220, 230], // border
    [255, 255, 255], // text
    [215, 215, 225], // muted
    [120, 180, 255], // accent
    [0, 90, 220],    // soft accent
    [90, 230, 130],  // ok
    [255, 200, 60],  // warning
    [255, 110, 190], // guide
];

fn high_contrast_on() -> bool {
    HIGH_CONTRAST.load(Ordering::Relaxed) == 1 && is_light()
}

fn read(i: usize) -> Color32 {
    if STARTED.load(Ordering::Relaxed) == 0 {
        activate(&Theme::default());
    }
    if high_contrast_on() {
        let [r, g, b] = HC_PALETTE[i];
        return Color32::from_rgb(r, g, b);
    }
    let [r, g, b, _] = CURRENT[i].load(Ordering::Relaxed).to_be_bytes();
    Color32::from_rgb(r, g, b)
}

/// Sets a theme in use (colors change immediately; then call `apply` for the egui styles).
pub fn activate(t: &Theme) {
    for (i, (field, _)) in COLORS.iter().enumerate() {
        let c = t.color(field);
        CURRENT[i].store(u32::from_be_bytes([c.r(), c.g(), c.b(), 255]), Ordering::Relaxed);
    }
    CORNERS.store(t.corners.clamp(0.0, 16.0).round() as u32, Ordering::Relaxed);
    LIGHT.store(t.light as u32, Ordering::Relaxed);
    STARTED.store(1, Ordering::Relaxed);
}

pub fn is_light() -> bool {
    LIGHT.load(Ordering::Relaxed) == 1
}

pub fn BACKDROP() -> Color32 {
    read(0)
}
pub fn PANEL() -> Color32 {
    read(1)
}
pub fn PANEL_DARK() -> Color32 {
    read(2)
}
pub fn FIELD() -> Color32 {
    read(3)
}
pub fn HOVER() -> Color32 {
    read(4)
}
pub fn LINE() -> Color32 {
    read(5)
}
pub fn BORDER() -> Color32 {
    read(6)
}
pub fn TEXT() -> Color32 {
    read(7)
}
pub fn MUTED() -> Color32 {
    read(8)
}
pub fn ACCENT() -> Color32 {
    read(9)
}
pub fn ACCENT_SOFT() -> Color32 {
    read(10)
}
pub fn OK() -> Color32 {
    read(11)
}
pub fn WARN() -> Color32 {
    read(12)
}
pub fn GUIDE() -> Color32 {
    read(13)
}

/// Applies the theme in use. With `high_contrast`: white text, strong borders and a more visible selection.
pub fn apply(ctx: &egui::Context, high_contrast: bool) {
    HIGH_CONTRAST.store(high_contrast as u32, Ordering::Relaxed);
    let light = is_light() && !high_contrast;
    let r = CORNERS.load(Ordering::Relaxed).min(16) as u8;
    let strong = if light { Color32::BLACK } else { Color32::WHITE };
    ctx.set_visuals(if light { egui::Visuals::light() } else { egui::Visuals::dark() });
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
        v.widgets.hovered.fg_stroke = Stroke::new(1.0, strong);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0, LINE());
        v.widgets.active.fg_stroke = Stroke::new(1.0, strong);
        v.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT());
        if high_contrast {
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
    fn bundled_themes_are_valid() {
        for t in defaults() {
            assert!(t.validate().is_ok(), "{}: {:?}", t.name, t.validate());
        }
    }

    fn luminance(c: [u8; 3]) -> f32 {
        let f = |v: u8| {
            let v = v as f32 / 255.0;
            if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
        };
        0.2126 * f(c[0]) + 0.7152 * f(c[1]) + 0.0722 * f(c[2])
    }

    fn ratio(a: [u8; 3], b: [u8; 3]) -> f32 {
        let (x, y) = (luminance(a), luminance(b));
        (x.max(y) + 0.05) / (x.min(y) + 0.05)
    }

    #[test]
    fn high_contrast_palette_is_readable_on_its_dark_panels() {
        for fg in [7, 8, 9, 11, 12] {
            for bg in [0, 1, 2, 3] {
                assert!(ratio(HC_PALETTE[fg], HC_PALETTE[bg]) >= 4.5, "{} on {}", COLORS[fg].0, COLORS[bg].0);
            }
        }
    }

    #[test]
    fn default_keeps_the_original_colors() {
        let t = Theme::default();
        assert_eq!(t.color("panel"), Color32::from_rgb(38, 39, 46));
        assert_eq!(t.color("accent"), Color32::from_rgb(77, 142, 255));
    }

    #[test]
    fn validation_of_external_themes() {
        let mut t = Theme::default();
        t.accent = "red".into();
        assert!(t.validate().is_err());
        let mut t = Theme::default();
        t.font = "../../malo".into();
        assert!(t.validate().is_err());
        let mut t = Theme::default();
        t.corners = 99.0;
        assert!(t.validate().is_err());
        assert!(serde_json::from_str::<Theme>(r#"{"name":"x","script":"bad"}"#).is_err(), "unknown fields are rejected");
    }

    #[test]
    fn theme_name_and_author_refuse_newlines_and_bidi_characters() {
        for bad in ["a\nb", "a\rb", "a\tb", "a\u{0}b", "a\u{85}b", "a\u{2028}b", "a\u{2029}b", "evil\u{202E}txt.exe", "a\u{200B}b", "a\u{2066}b", "a\u{FEFF}", "a\u{061C}b", "a\u{200F}b"] {
            let mut t = Theme::default();
            t.name = bad.into();
            assert!(t.validate().is_err(), "name {:?}", bad);
            let mut t = Theme::default();
            t.author = bad.into();
            assert!(t.validate().is_err(), "author {:?}", bad);
            // the same through the community file validator (what an uploaded / downloaded theme goes through)
            let mut t = Theme::default();
            t.author = bad.into();
            let json = serde_json::to_vec(&t).unwrap();
            assert!(crate::community_validate::validate("theme", "t.json", &json).is_err(), "community {:?}", bad);
        }
        let mut t = Theme::default();
        t.name = "Tema ñandú 日本語".into();
        t.author = "Álvaro".into();
        assert!(t.validate().is_ok());
    }

    #[test]
    fn old_spanish_theme_files_still_load() {
        let old = r##"{"nombre":"Viejo","autor":"yo","claro":true,"fondo":"#E9EBF0","panel":"#F7F8FA","panel_oscuro":"#EEF0F4","campo":"#E2E5EC","hover":"#D6DAE3",
            "linea":"#CDD2DC","borde":"#B9BFCC","texto":"#1E2230","apagado":"#5D6476","acento":"#2F6FE4","acento_suave":"#CFDDFB","ok":"#1F9D57","aviso":"#B86E00",
            "guia":"#D6248F","esquinas":5.0,"fuente":""}"##;
        let t: Theme = serde_json::from_str(old).expect("a theme with Spanish keys must load");
        assert_eq!((t.name.as_str(), t.light, t.accent.as_str(), t.corners), ("Viejo", true, "#2F6FE4", 5.0));
        assert!(t.validate().is_ok());
        // what is uploaded to the community now uses the English keys, and still loads again
        let json = serde_json::to_value(&t).unwrap();
        assert_eq!(json["accent"], "#2F6FE4");
        assert!(json.get("acento").is_none());
        assert_eq!(serde_json::from_value::<Theme>(json).unwrap(), t);
        // the old names of the bundled themes still select them
        assert_eq!(find_default("Lienzo claro").unwrap().name, "Lienzo Light");
        assert_eq!(find_default("Medianoche").unwrap().name, "Midnight");
        assert_eq!(find_default("Sunset").unwrap().name, "Sunset");
    }

    #[test]
    fn hex_round_trip() {
        assert_eq!(to_hex(parse_hex("#4d8eff").unwrap()), "#4D8EFF");
        assert!(parse_hex("#12345").is_none());
        assert!(parse_hex("123456").is_none());
    }
}
