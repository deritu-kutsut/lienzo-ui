//! Accessibility: language, interface size, high contrast and reduced motion.
//! Saved in the user's settings and applied on startup and whenever any option changes.

use crate::app::EditorApp;
use crate::i18n::{self, t};
use crate::theme::{ACCENT, MUTED};
use eframe::egui::{self, RichText};

pub const ESCALAS: &[(f32, &str)] = &[(0.85, "85 %"), (1.0, "100 %"), (1.25, "125 %"), (1.5, "150 %"), (1.75, "175 %"), (2.0, "200 %")];

/// Applies the options to the egui context.
pub fn aplicar(ctx: &egui::Context, s: &crate::library::Settings) {
    i18n::set(if s.idioma.is_empty() { "es" } else { &s.idioma });
    let escala = if s.escala_ui <= 0.0 { 1.0 } else { s.escala_ui.clamp(0.75, 2.5) };
    ctx.set_zoom_factor(escala);
    let tema = s.tema.clone().filter(|t| t.validar().is_ok()).unwrap_or_default();
    crate::theme::usar(&tema);
    if !crate::render::fuente_idioma(ctx, i18n::codigo()) {
        eprintln!("No se encontró una fuente del sistema para el idioma «{}»", i18n::codigo());
    }
    crate::render::fuente_interfaz(ctx, &tema.fuente);
    crate::theme::apply(ctx, s.alto_contraste);
    ctx.all_styles_mut(|st| st.animation_time = if s.reducir_movimiento { 0.0 } else { 1.0 / 12.0 });
}

impl EditorApp {
    pub fn accesibilidad_window(&mut self, ctx: &egui::Context) {
        if !self.accesibilidad_abierta {
            return;
        }
        let mut abierta = true;
        let mut cambio = false;
        egui::Window::new(t("Accesibilidad")).open(&mut abierta).collapsible(false).resizable(false).default_width(380.0).show(ctx, |ui| {
            egui::Grid::new("accesibilidad").num_columns(2).spacing([14.0, 10.0]).show(ui, |ui| {
                ui.label(t("Idioma"));
                let actual = i18n::IDIOMAS.iter().find(|(c, _, _)| *c == i18n::codigo()).map(|x| x.1).unwrap_or("Español");
                egui::ComboBox::from_id_salt("idioma").selected_text(actual).show_ui(ui, |ui| {
                    for (codigo, nombre, _) in i18n::IDIOMAS {
                        if ui.selectable_label(i18n::codigo() == *codigo, *nombre).clicked() {
                            self.settings.idioma = codigo.to_string();
                            cambio = true;
                        }
                    }
                });
                ui.end_row();
                let rutas = crate::render::fuentes_de_idioma(i18n::codigo());
                if !rutas.is_empty() && !rutas.iter().any(|p| std::path::Path::new(p).exists()) {
                    ui.label("");
                    ui.label(RichText::new(t("No se encontró en el sistema una fuente con estos caracteres: el texto puede verse como cuadros.")).small().color(egui::Color32::from_rgb(230, 170, 60)));
                    ui.end_row();
                }

                ui.label(t("Tamaño de la interfaz"));
                ui.horizontal_wrapped(|ui| {
                    let actual = if self.settings.escala_ui <= 0.0 { 1.0 } else { self.settings.escala_ui };
                    for (v, nombre) in ESCALAS {
                        if ui.selectable_label((actual - v).abs() < 0.01, *nombre).clicked() {
                            self.settings.escala_ui = *v;
                            cambio = true;
                        }
                    }
                });
                ui.end_row();

                ui.label(t("Alto contraste"));
                cambio |= ui.checkbox(&mut self.settings.alto_contraste, t("Texto blanco y bordes marcados")).changed();
                ui.end_row();

                ui.label(t("Reducir movimiento"));
                cambio |= ui.checkbox(&mut self.settings.reducir_movimiento, t("Sin animaciones en la interfaz")).changed();
                ui.end_row();
            });
            ui.add_space(8.0);
            ui.label(RichText::new(t("Atajos: Ctrl+Alt+= y Ctrl+Alt+- cambian el tamaño de la interfaz; F1 abre el tutorial.")).small().color(MUTED()));
            ui.label(RichText::new(t("Para añadir un idioma, traduce assets/i18n/en.tsv (ver docs/EXTENDER.md).")).small().color(ACCENT()));
        });
        if cambio {
            self.settings.save();
            aplicar(ctx, &self.settings);
        }
        self.accesibilidad_abierta = abierta;
    }

    /// Ctrl+Alt+= / Ctrl+Alt+- : interface size.
    pub fn atajos_accesibilidad(&mut self, ctx: &egui::Context) {
        let m = egui::Modifiers::COMMAND | egui::Modifiers::ALT;
        let (mas, menos) = ctx.input_mut(|i| (i.consume_key(m, egui::Key::Equals) || i.consume_key(m, egui::Key::Plus), i.consume_key(m, egui::Key::Minus)));
        if mas || menos {
            let actual = if self.settings.escala_ui <= 0.0 { 1.0 } else { self.settings.escala_ui };
            self.settings.escala_ui = (actual + if mas { 0.25 } else { -0.25 }).clamp(0.75, 2.0);
            self.settings.save();
            aplicar(ctx, &self.settings);
            self.status = format!("{} {:.0} %", t("Tamaño de la interfaz:"), self.settings.escala_ui * 100.0);
        }
    }
}
