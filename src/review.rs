//! «Review design» (design review): quick readability and usability checks before exporting to Unity.
//! Everything is pure (no UI) so it can be used from the window, the command line (`--review`) and the MCP.

use crate::model::{Doc, Element, Kind, Rgba};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Warning,
    Error,
}

#[derive(Clone, Debug)]
pub struct Problem {
    pub level: Level,
    pub id: u64,
    pub element: String,
    /// Text in Spanish (translated when displayed with `i18n::tf`).
    pub template: &'static str,
    pub data: Vec<String>,
}

/// Minimum legible font size at 1080 p height (scaled with the reference resolution).
const MIN_FONT: f32 = 18.0;
/// Minimum side of a tappable control at 1080 p (≈ 9 mm on a 6" phone).
const MIN_TOUCH: f32 = 64.0;

fn luminance(c: Rgba) -> f32 {
    let f = |v: u8| {
        let s = v as f32 / 255.0;
        if s <= 0.04045 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * f(c[0]) + 0.7152 * f(c[1]) + 0.0722 * f(c[2])
}

/// WCAG contrast ratio (1 to 21).
pub fn contrast(a: Rgba, b: Rgba) -> f32 {
    let (la, lb) = (luminance(a), luminance(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

fn blend(up: Rgba, down: Rgba) -> Rgba {
    let a = up[3] as f32 / 255.0;
    let m = |i: usize| (up[i] as f32 * a + down[i] as f32 * (1.0 - a)).round() as u8;
    [m(0), m(1), m(2), 255]
}

fn fill(e: &Element) -> Rgba {
    // Images without a file are gradients: the mid color is used.
    if e.kind == Kind::Image && e.image.is_empty() {
        let m = |i: usize| ((e.fill[i] as u16 + e.fill2[i] as u16) / 2) as u8;
        return [m(0), m(1), m(2), m(3)];
    }
    e.fill
}

fn visible(doc: &Doc, e: &Element) -> bool {
    let mut cur = Some(e);
    let mut steps = 0;
    while let Some(x) = cur {
        if !x.visible || x.opacity <= 0.01 {
            return false;
        }
        steps += 1;
        if steps > 256 {
            break;
        }
        cur = x.parent.and_then(|p| doc.get(p));
    }
    true
}

fn has_own_background(e: &Element) -> bool {
    matches!(e.kind, Kind::Button | Kind::InputField | Kind::Dropdown | Kind::Panel | Kind::Ellipse) && e.fill[3] > 0
}

/// Color the text of `idx` is seen against: its own fill if it has one, or whatever is underneath.
fn background_of(doc: &Doc, idx: usize) -> Rgba {
    let e = &doc.elements[idx];
    let (cx, cy) = (e.x + e.w / 2.0, e.y + e.h / 2.0);
    let mut color = doc.background;
    color[3] = 255;
    for (i, o) in doc.elements.iter().enumerate() {
        if i >= idx && !(i == idx && has_own_background(e)) {
            break;
        }
        if o.kind == Kind::Text || o.kind == Kind::Empty || !o.image.is_empty() || !visible(doc, o) {
            continue;
        }
        if cx >= o.x && cx <= o.x + o.w && cy >= o.y && cy <= o.y + o.h {
            color = blend(fill(o), color);
        }
    }
    color
}

fn is_pressable(e: &Element) -> bool {
    e.interactable && matches!(e.kind, Kind::Button | Kind::Toggle | Kind::InputField | Kind::Dropdown)
}

/// Reviews the design and returns the problems, errors first.
pub fn review(doc: &Doc) -> Vec<Problem> {
    let scale = (doc.height.min(doc.width) / 1080.0).max(0.1);
    let mut out = Vec::new();
    let mut names: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for e in &doc.elements {
        *names.entry(e.name.as_str()).or_default() += 1;
    }
    for (idx, e) in doc.elements.iter().enumerate() {
        let mut p = |level, template, data: Vec<String>| out.push(Problem { level, id: e.id, element: e.name.clone(), template, data });
        if !visible(doc, e) {
            continue;
        }
        let with_text = !e.text.trim().is_empty() && matches!(e.kind, Kind::Text | Kind::Button | Kind::Panel | Kind::Ellipse | Kind::Toggle);
        if with_text {
            let min = MIN_FONT * scale;
            if e.font_size < min {
                p(Level::Warning, "{} px text: hard to read on small screens (recommended minimum {} px)", vec![format!("{:.0}", e.font_size), format!("{:.0}", min)]);
            }
            // WCAG does not require contrast for disabled controls.
            let disabled = !e.interactable && matches!(e.kind, Kind::Button | Kind::Toggle);
            let background = background_of(doc, idx);
            let c = contrast(blend(e.text_color, background), background);
            // WCAG "large text": 18 pt (24 px) or 14 pt bold (≈18.7 px), at the reference scale.
            let big = e.font_size >= 24.0 * scale || (e.bold && e.font_size >= 18.7 * scale);
            let min_c = if big { 3.0 } else { 4.5 };
            if c < min_c && !disabled {
                let level = if c < 2.0 { Level::Error } else { Level::Warning };
                p(level, "Text contrast {}:1 (WCAG asks for {}:1)", vec![format!("{:.1}", c), format!("{:.1}", min_c)]);
            }
        }
        if is_pressable(e) {
            let min = MIN_TOUCH * scale;
            if e.w.min(e.h) < min {
                p(Level::Warning, "{} px control: hard to tap with a finger (recommended minimum {} px)", vec![format!("{:.0}", e.w.min(e.h)), format!("{:.0}", min)]);
            }
        }
        let outside = e.x + e.w < 0.0 || e.y + e.h < 0.0 || e.x > doc.width || e.y > doc.height;
        if outside && e.rot.abs() < 0.01 {
            p(Level::Error, "Completely outside the page: it will not be visible in the game", vec![]);
        } else if e.parent.is_none() && (e.x < -1.0 || e.y < -1.0 || e.x + e.w > doc.width + 1.0 || e.y + e.h > doc.height + 1.0) && e.rot.abs() < 0.01 {
            p(Level::Warning, "Partly outside the page", vec![]);
        }
        if is_pressable(e) && names.get(e.name.as_str()).copied().unwrap_or(0) > 1 {
            p(Level::Warning, "Several elements share this name: GameObject.Find and scripts will only find the first one", vec![]);
        }
        if e.kind == Kind::Image && !e.image.is_empty() && !crate::library::resolve_image(&e.image).exists() {
            p(Level::Error, "Missing image “{}”: it will be exported empty", vec![e.image.clone()]);
        }
    }
    out.sort_by(|a, b| b.level.cmp(&a.level));
    out
}

/// Text of a problem in the current language.
pub fn describe(p: &Problem) -> String {
    let args: Vec<&str> = p.data.iter().map(|s| s.as_str()).collect();
    crate::i18n::tf(p.template, &args)
}

/// `--review`: prints the problems; exit code 1 if there are errors.
pub fn report(doc: &Doc) -> (String, bool) {
    let ps = review(doc);
    if ps.is_empty() {
        return (crate::i18n::t("No issues: the design passes every check."), false);
    }
    let errors = ps.iter().any(|p| p.level == Level::Error);
    let lines: Vec<String> = ps
        .iter()
        .map(|p| format!("[{}] {}: {}", if p.level == Level::Error { "ERROR" } else { "WARNING" }, p.element, describe(p)))
        .collect();
    (lines.join("\n"), errors)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Doc, Element, Kind};

    fn doc_with(e: Element) -> Doc {
        let mut d = Doc::empty(1920.0, 1080.0);
        d.background = [255, 255, 255, 255];
        d.add(e);
        d
    }

    #[test]
    fn known_contrast() {
        assert!((contrast([0, 0, 0, 255], [255, 255, 255, 255]) - 21.0).abs() < 0.01);
        assert!((contrast([10, 20, 30, 255], [10, 20, 30, 255]) - 1.0).abs() < 0.001);
    }

    #[test]
    fn detects_illegible_text_and_small_button() {
        let mut t = Element::new(0, Kind::Text);
        t.text = "hola".into();
        t.text_color = [240, 240, 240, 255];
        t.font_size = 12.0;
        t.x = 100.0;
        t.y = 100.0;
        let ps = review(&doc_with(t));
        assert!(ps.iter().any(|p| p.template.starts_with("{} px text")));
        assert!(ps.iter().any(|p| p.template.starts_with("Text contrast") && p.level == Level::Error));

        let mut b = Element::new(0, Kind::Button);
        b.x = 100.0;
        b.y = 100.0;
        b.w = 40.0;
        b.h = 40.0;
        let ps = review(&doc_with(b));
        assert!(ps.iter().any(|p| p.template.starts_with("{} px control")));
    }

    #[test]
    fn off_page_and_duplicate_names() {
        let mut b = Element::new(0, Kind::Button);
        b.x = 5000.0;
        let ps = review(&doc_with(b));
        assert!(ps.iter().any(|p| p.level == Level::Error && p.template.contains("outside")));
        let mut d = Doc::empty(1920.0, 1080.0);
        for _ in 0..2 {
            let mut b = Element::new(0, Kind::Button);
            b.name = "Play".into();
            b.x = 100.0;
            b.y = 100.0;
            d.add(b);
        }
        assert!(review(&d).iter().any(|p| p.template.contains("GameObject.Find")));
    }

    #[test]
    fn bundled_templates_without_errors() {
        for i in 0..crate::model::TEMPLATES.len() {
            let d = crate::model::template(i, 1920.0, 1080.0);
            let errors: Vec<String> = review(&d).iter().filter(|p| p.level == Level::Error).map(|p| format!("{}: {}", p.element, describe(p))).collect();
            assert!(errors.is_empty(), "{}: {:?}", crate::model::TEMPLATES[i], errors);
        }
    }
}

impl crate::app::EditorApp {
    /// «Review design» window: list of problems; clicking selects the element.
    pub fn review_window(&mut self, ctx: &eframe::egui::Context) {
        use crate::i18n::t;
        use eframe::egui::{self, Color32, RichText};
        if !self.review_open {
            return;
        }
        let problems = review(&self.doc);
        let mut open = true;
        let mut choose = None;
        egui::Window::new(t("Review design")).open(&mut open).collapsible(false).default_width(460.0).show(ctx, |ui| {
            let errors = problems.iter().filter(|p| p.level == Level::Error).count();
            let warnings = problems.len() - errors;
            if problems.is_empty() {
                ui.label(RichText::new(t("No issues: the design passes every check.")).color(crate::theme::ACCENT()));
            } else {
                ui.label(crate::i18n::tf("{} errors · {} warnings. Click one to select it.", &[&errors.to_string(), &warnings.to_string()]));
            }
            ui.add_space(6.0);
            egui::ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                for p in &problems {
                    let (label, col) = match p.level {
                        Level::Error => (t("Error"), Color32::from_rgb(235, 80, 90)),
                        Level::Warning => (t("Warning"), Color32::from_rgb(230, 170, 60)),
                    };
                    ui.horizontal_wrapped(|ui| {
                        ui.label(RichText::new(label).strong().color(col));
                        if ui.link(&p.element).clicked() {
                            choose = Some(p.id);
                        }
                        ui.label(RichText::new(describe(p)).color(crate::theme::MUTED()));
                    });
                }
            });
            ui.add_space(6.0);
            ui.label(RichText::new(t("Minimums are based on the reference resolution (18 px text and 64 px per control at 1080p).")).small().color(crate::theme::MUTED()));
        });
        if let Some(id) = choose {
            self.sel = vec![id];
        }
        self.review_open = open;
    }
}
