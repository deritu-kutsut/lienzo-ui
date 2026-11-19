//! Accessibility: language, interface size, high contrast and reduced motion.
//! Saved in the user's settings and applied on startup and whenever any option changes.

use crate::app::EditorApp;
use crate::i18n::{self, t};
use crate::theme::{ACCENT, MUTED};
use eframe::egui::{self, RichText};

pub const SCALES: &[(f32, &str)] = &[(0.85, "85 %"), (1.0, "100 %"), (1.25, "125 %"), (1.5, "150 %"), (1.75, "175 %"), (2.0, "200 %")];

/// Applies the options to the egui context.
pub fn apply(ctx: &egui::Context, s: &crate::library::Settings) {
    i18n::set(if s.language.is_empty() { "es" } else { &s.language });
    let scale = if s.ui_scale <= 0.0 { 1.0 } else { s.ui_scale.clamp(0.75, 2.5) };
    ctx.set_zoom_factor(scale);
    let theme = s.theme.clone().filter(|t| t.validate().is_ok()).unwrap_or_default();
    crate::theme::activate(&theme);
    if !crate::render::language_font(ctx, i18n::code()) {
        eprintln!("No system font was found for the language “{}”", i18n::code());
    }
    crate::render::interface_font(ctx, &theme.font);
    crate::theme::apply(ctx, s.high_contrast);
    ctx.all_styles_mut(|st| st.animation_time = if s.reduce_motion { 0.0 } else { 1.0 / 12.0 });
}

impl EditorApp {
    pub fn accessibility_window(&mut self, ctx: &egui::Context) {
        if !self.accessibility_open {
            return;
        }
        let mut open = true;
        let mut change = false;
        egui::Window::new(t("Accessibility")).open(&mut open).collapsible(false).resizable(false).default_width(380.0).show(ctx, |ui| {
            egui::Grid::new("accessibility").num_columns(2).spacing([14.0, 10.0]).show(ui, |ui| {
                ui.label(t("Language"));
                let current = i18n::LANGUAGES.iter().find(|(c, _, _)| *c == i18n::code()).map(|x| x.1).unwrap_or("English");
                egui::ComboBox::from_id_salt("language").selected_text(current).show_ui(ui, |ui| {
                    for (code, name, _) in i18n::LANGUAGES {
                        if ui.selectable_label(i18n::code() == *code, *name).clicked() {
                            self.settings.language = code.to_string();
                            change = true;
                        }
                    }
                });
                ui.end_row();
                let paths = crate::render::language_fonts(i18n::code());
                if !paths.is_empty() && !paths.iter().any(|p| std::path::Path::new(p).exists()) {
                    ui.label("");
                    ui.label(RichText::new(t("No system font with these characters was found: text may show as boxes.")).small().color(egui::Color32::from_rgb(230, 170, 60)));
                    ui.end_row();
                }

                ui.label(t("Interface size"));
                ui.horizontal_wrapped(|ui| {
                    let current = if self.settings.ui_scale <= 0.0 { 1.0 } else { self.settings.ui_scale };
                    for (v, name) in SCALES {
                        if ui.selectable_label((current - v).abs() < 0.01, *name).clicked() {
                            self.settings.ui_scale = *v;
                            change = true;
                        }
                    }
                });
                ui.end_row();

                ui.label(t("High contrast"));
                change |= ui.checkbox(&mut self.settings.high_contrast, t("White text and strong borders")).changed();
                ui.end_row();

                ui.label(t("Reduce motion"));
                change |= ui.checkbox(&mut self.settings.reduce_motion, t("No animations in the interface")).changed();
                ui.end_row();
            });
            ui.add_space(8.0);
            ui.label(RichText::new(t("Shortcuts: Ctrl+Alt+= and Ctrl+Alt+- change the interface size; F1 opens the tutorial.")).small().color(MUTED()));
            ui.label(RichText::new(t("To add a language, copy assets/i18n/es.tsv and translate it (see docs/EXTENDING.md).")).small().color(ACCENT()));
        });
        if change {
            self.settings.save();
            apply(ctx, &self.settings);
        }
        self.accessibility_open = open;
    }

    /// Ctrl+Alt+= / Ctrl+Alt+- : interface size.
    pub fn accessibility_shortcuts(&mut self, ctx: &egui::Context) {
        let m = egui::Modifiers::COMMAND | egui::Modifiers::ALT;
        let (plus, minus) = ctx.input_mut(|i| (i.consume_key(m, egui::Key::Equals) || i.consume_key(m, egui::Key::Plus), i.consume_key(m, egui::Key::Minus)));
        if plus || minus {
            let current = if self.settings.ui_scale <= 0.0 { 1.0 } else { self.settings.ui_scale };
            self.settings.ui_scale = (current + if plus { 0.25 } else { -0.25 }).clamp(0.75, 2.5);
            self.settings.save();
            apply(ctx, &self.settings);
            self.status = format!("{} {:.0} %", t("Interface size:"), self.settings.ui_scale * 100.0);
        }
    }
}
