//! «Themes» window: choose, customize, save and share the app's appearance (colors, corners and font).

use crate::app::EditorApp;
use crate::i18n::t;
use crate::theme::{self, Theme, ACCENT, MUTED};
use eframe::egui::{self, Color32, RichText, Vec2};
use std::path::PathBuf;

/// Folder of saved themes and themes installed from the community.
pub fn folder() -> PathBuf {
    crate::library::root_dir().join("temas")
}

/// Themes saved by the user (those that fail validation are ignored).
pub fn saved() -> Vec<(PathBuf, Theme)> {
    let Ok(rd) = std::fs::read_dir(folder()) else { return Vec::new() };
    let mut v: Vec<(PathBuf, Theme)> = rd
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .filter_map(|p| {
            let t: Theme = serde_json::from_str(&std::fs::read_to_string(&p).ok()?).ok()?;
            t.validate().ok()?;
            Some((p, t))
        })
        .collect();
    v.sort_by(|a, b| a.1.name.to_lowercase().cmp(&b.1.name.to_lowercase()));
    v
}

fn file_name(name: &str) -> String {
    let s: String = name.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    let s = s.trim_matches('_').chars().take(50).collect::<String>();
    if s.is_empty() { "tema".into() } else { s }
}

/// Saves a theme in the library. Returns the path.
pub fn save(t: &Theme) -> Result<PathBuf, String> {
    t.validate()?;
    let dir = folder();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let f = dir.join(format!("{}.json", file_name(&t.name)));
    std::fs::write(&f, serde_json::to_string_pretty(t).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    Ok(f)
}

/// Color strip to recognize a theme at a glance.
fn sample(ui: &mut egui::Ui, t: &Theme) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(70.0, 18.0), egui::Sense::hover());
    let colors = [t.color("background"), t.color("panel"), t.color("field"), t.color("accent"), t.color("text")];
    let w = r.width() / colors.len() as f32;
    for (i, c) in colors.iter().enumerate() {
        let cell = egui::Rect::from_min_size(egui::pos2(r.left() + w * i as f32, r.top()), Vec2::new(w, r.height()));
        ui.painter().rect_filled(cell, 0.0, *c);
    }
    ui.painter().rect_stroke(r, 3.0, egui::Stroke::new(1.0, theme::LINE()), egui::StrokeKind::Inside);
}

impl EditorApp {
    /// Sets a theme in use, saves it in the settings and applies it to the whole interface.
    pub fn apply_theme(&mut self, ctx: &egui::Context, t: Theme) {
        self.settings.theme = Some(t);
        self.settings.save();
        crate::accessibility::apply(ctx, &self.settings);
    }

    pub fn themes_window(&mut self, ctx: &egui::Context) {
        if !self.themes_open {
            return;
        }
        let mut open = true;
        let current = self.settings.theme.clone().unwrap_or_default();
        let mut new: Option<Theme> = None;
        let mut delete: Option<PathBuf> = None;
        egui::Window::new(t("Themes")).open(&mut open).collapsible(false).default_width(430.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().max_height(ctx.content_rect().height() * 0.8).show(ui, |ui| {
                ui.label(RichText::new(t("Change the colours, corners and font of the whole app. Saved instantly.")).small().color(MUTED()));
                ui.add_space(4.0);
                // ── Choose ──
                let mut list: Vec<(Option<PathBuf>, Theme)> = theme::defaults().into_iter().map(|t| (None, t)).collect();
                list.extend(saved().into_iter().map(|(p, t)| (Some(p), t)));
                for (path, theme) in &list {
                    ui.horizontal(|ui| {
                        sample(ui, theme);
                        let in_use = theme.name == current.name && *theme == current;
                        let name = if theme.author.is_empty() || theme.author == "Lienzo UI" { theme.name.clone() } else { format!("{} · {}", theme.name, theme.author) };
                        ui.label(if in_use { RichText::new(name).strong().color(ACCENT()) } else { RichText::new(name) });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(p) = path {
                                if ui.small_button(t("Delete")).clicked() {
                                    delete = Some(p.clone());
                                }
                            }
                            if ui.add_enabled(!in_use, egui::Button::new(if in_use { t("In use") } else { t("Use") }).small()).clicked() {
                                new = Some(theme.clone());
                            }
                        });
                    });
                }
                ui.horizontal(|ui| {
                    if ui.add(crate::icons::btn("heart", t("More themes in the community"))).clicked() {
                        self.community_goto(ctx, "tema");
                    }
                });
                ui.separator();
                // ── Customize ──
                ui.label(RichText::new(t("Customise")).strong());
                let mut t2 = current.clone();
                egui::Grid::new("theme_colors").num_columns(4).spacing([10.0, 6.0]).show(ui, |ui| {
                    for (i, (field, name)) in theme::COLORS.iter().enumerate() {
                        let mut c: Color32 = t2.color(field);
                        if egui::color_picker::color_edit_button_srgba(ui, &mut c, egui::color_picker::Alpha::Opaque).changed() {
                            if let Some(v) = t2.color_mut(field) {
                                *v = theme::to_hex(c);
                            }
                        }
                        ui.label(t(name));
                        if i % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });
                ui.add(egui::Slider::new(&mut t2.corners, 0.0..=16.0).step_by(1.0).text(t("Corners")));
                ui.checkbox(&mut t2.light, t("Light base (dark text)"));
                ui.horizontal(|ui| {
                    ui.label(t("Interface font"));
                    let current_font = if t2.font.is_empty() { t("Default") } else { t2.font.clone() };
                    egui::ComboBox::from_id_salt("theme_font").selected_text(current_font).show_ui(ui, |ui| {
                        if ui.selectable_label(t2.font.is_empty(), t("Default")).clicked() {
                            t2.font.clear();
                        }
                        for f in crate::render::available_fonts() {
                            if ui.selectable_label(t2.font == f, &f).clicked() {
                                t2.font = f.clone();
                            }
                        }
                    });
                });
                if !t2.font.is_empty() && !crate::render::available_fonts().contains(&t2.font) {
                    ui.label(RichText::new(format!("{} “{}”. {}", t("You don't have the font"), t2.font, t("Find it in Community › Fonts."))).small().color(theme::WARN()));
                }
                if t2 != current {
                    if t2.name == current.name && theme::defaults().iter().any(|p| p.name == t2.name) {
                        t2.name = format!("{} ({})", current.name, t("custom"));
                        t2.author = String::new();
                    }
                    new = Some(t2.clone());
                }
                ui.separator();
                // ── Save and share ──
                ui.horizontal(|ui| {
                    ui.label(t("Name"));
                    if self.theme_name.is_empty() {
                        self.theme_name = current.name.clone();
                    }
                    ui.add(egui::TextEdit::singleline(&mut self.theme_name).desired_width(200.0));
                });
                ui.horizontal_wrapped(|ui| {
                    if ui.button(t("Save as theme")).clicked() {
                        let mut g = self.settings.theme.clone().unwrap_or_default();
                        g.name = self.theme_name.trim().to_string();
                        if let Some(p) = self.community.profile_name() {
                            g.author = p;
                        }
                        match save(&g) {
                            Ok(_) => {
                                self.status = format!("{} “{}”", t("Theme saved:"), g.name);
                                new = Some(g);
                            }
                            Err(e) => self.status = e,
                        }
                    }
                    if ui.button(t("Share with the community")).clicked() {
                        let mut g = self.settings.theme.clone().unwrap_or_default();
                        g.name = self.theme_name.trim().to_string();
                        if let Some(p) = self.community.profile_name() {
                            g.author = p;
                        }
                        match g.validate().and_then(|_| {
                            let dir = std::env::temp_dir().join("lienzo-ui-community");
                            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                            let f = dir.join(format!("{}.json", file_name(&g.name)));
                            std::fs::write(&f, serde_json::to_string_pretty(&g).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
                            Ok(f)
                        }) {
                            Ok(f) => {
                                self.community_prepare_upload(ctx, "tema", f, &g.name);
                                self.themes_open = false;
                            }
                            Err(e) => self.status = e,
                        }
                    }
                    if ui.button(t("Back to default")).clicked() {
                        new = Some(Theme::default());
                    }
                });
            });
        });
        if let Some(p) = delete {
            let _ = std::fs::remove_file(p);
        }
        if let Some(n) = new {
            self.apply_theme(ctx, n);
        }
        if !open {
            self.themes_open = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_file_names() {
        assert_eq!(file_name("My theme ✨"), "My_theme");
        assert_eq!(file_name("../../malo"), "malo");
        assert_eq!(file_name("   "), "tema");
    }
}
