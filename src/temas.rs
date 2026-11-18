//! «Temas» window: choose, customize, save and share the app's appearance (colors, corners and font).

use crate::app::EditorApp;
use crate::i18n::t;
use crate::theme::{self, Tema, ACCENT, MUTED};
use eframe::egui::{self, Color32, RichText, Vec2};
use std::path::PathBuf;

/// Folder of saved themes and themes installed from the community.
pub fn carpeta() -> PathBuf {
    crate::library::root_dir().join("temas")
}

/// Themes saved by the user (those that fail validation are ignored).
pub fn guardados() -> Vec<(PathBuf, Tema)> {
    let Ok(rd) = std::fs::read_dir(carpeta()) else { return Vec::new() };
    let mut v: Vec<(PathBuf, Tema)> = rd
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .filter_map(|p| {
            let t: Tema = serde_json::from_str(&std::fs::read_to_string(&p).ok()?).ok()?;
            t.validar().ok()?;
            Some((p, t))
        })
        .collect();
    v.sort_by(|a, b| a.1.nombre.to_lowercase().cmp(&b.1.nombre.to_lowercase()));
    v
}

fn nombre_archivo(nombre: &str) -> String {
    let s: String = nombre.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
    let s = s.trim_matches('_').chars().take(50).collect::<String>();
    if s.is_empty() { "tema".into() } else { s }
}

/// Saves a theme in the library. Returns the path.
pub fn guardar(t: &Tema) -> Result<PathBuf, String> {
    t.validar()?;
    let dir = carpeta();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let f = dir.join(format!("{}.json", nombre_archivo(&t.nombre)));
    std::fs::write(&f, serde_json::to_string_pretty(t).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    Ok(f)
}

/// Color strip to recognize a theme at a glance.
fn muestra(ui: &mut egui::Ui, t: &Tema) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(70.0, 18.0), egui::Sense::hover());
    let colores = [t.color("fondo"), t.color("panel"), t.color("campo"), t.color("acento"), t.color("texto")];
    let w = r.width() / colores.len() as f32;
    for (i, c) in colores.iter().enumerate() {
        let celda = egui::Rect::from_min_size(egui::pos2(r.left() + w * i as f32, r.top()), Vec2::new(w, r.height()));
        ui.painter().rect_filled(celda, 0.0, *c);
    }
    ui.painter().rect_stroke(r, 3.0, egui::Stroke::new(1.0, theme::LINE()), egui::StrokeKind::Inside);
}

impl EditorApp {
    /// Sets a theme in use, saves it in the settings and applies it to the whole interface.
    pub fn usar_tema(&mut self, ctx: &egui::Context, t: Tema) {
        self.settings.tema = Some(t);
        self.settings.save();
        crate::accesibilidad::aplicar(ctx, &self.settings);
    }

    pub fn temas_window(&mut self, ctx: &egui::Context) {
        if !self.temas_abierto {
            return;
        }
        let mut abierta = true;
        let actual = self.settings.tema.clone().unwrap_or_default();
        let mut nuevo: Option<Tema> = None;
        let mut borrar: Option<PathBuf> = None;
        egui::Window::new(t("Temas")).open(&mut abierta).collapsible(false).default_width(430.0).show(ctx, |ui| {
            egui::ScrollArea::vertical().max_height(ctx.content_rect().height() * 0.8).show(ui, |ui| {
                ui.label(RichText::new(t("Cambia los colores, las esquinas y la fuente de toda la app. Se guarda al momento.")).small().color(MUTED()));
                ui.add_space(4.0);
                // ── Choose ──
                let mut lista: Vec<(Option<PathBuf>, Tema)> = theme::predeterminados().into_iter().map(|t| (None, t)).collect();
                lista.extend(guardados().into_iter().map(|(p, t)| (Some(p), t)));
                for (ruta, tema) in &lista {
                    ui.horizontal(|ui| {
                        muestra(ui, tema);
                        let en_uso = tema.nombre == actual.nombre && *tema == actual;
                        let nombre = if tema.autor.is_empty() || tema.autor == "Lienzo UI" { tema.nombre.clone() } else { format!("{} · {}", tema.nombre, tema.autor) };
                        ui.label(if en_uso { RichText::new(nombre).strong().color(ACCENT()) } else { RichText::new(nombre) });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if let Some(p) = ruta {
                                if ui.small_button(t("Borrar")).clicked() {
                                    borrar = Some(p.clone());
                                }
                            }
                            if ui.add_enabled(!en_uso, egui::Button::new(if en_uso { t("En uso") } else { t("Usar") }).small()).clicked() {
                                nuevo = Some(tema.clone());
                            }
                        });
                    });
                }
                ui.horizontal(|ui| {
                    if ui.add(crate::icons::btn("heart", t("Más temas en la comunidad"))).clicked() {
                        self.comunidad_ir_a(ctx, "tema");
                    }
                });
                ui.separator();
                // ── Customize ──
                ui.label(RichText::new(t("Personalizar")).strong());
                let mut t2 = actual.clone();
                egui::Grid::new("tema_colores").num_columns(4).spacing([10.0, 6.0]).show(ui, |ui| {
                    for (i, (campo, nombre)) in theme::COLORES.iter().enumerate() {
                        let mut c: Color32 = t2.color(campo);
                        if egui::color_picker::color_edit_button_srgba(ui, &mut c, egui::color_picker::Alpha::Opaque).changed() {
                            if let Some(v) = t2.color_mut(campo) {
                                *v = theme::a_hex(c);
                            }
                        }
                        ui.label(t(nombre));
                        if i % 2 == 1 {
                            ui.end_row();
                        }
                    }
                });
                ui.add(egui::Slider::new(&mut t2.esquinas, 0.0..=16.0).step_by(1.0).text(t("Esquinas")));
                ui.checkbox(&mut t2.claro, t("Base clara (texto oscuro)"));
                ui.horizontal(|ui| {
                    ui.label(t("Fuente de la interfaz"));
                    let actual_fuente = if t2.fuente.is_empty() { t("Predeterminada") } else { t2.fuente.clone() };
                    egui::ComboBox::from_id_salt("tema_fuente").selected_text(actual_fuente).show_ui(ui, |ui| {
                        if ui.selectable_label(t2.fuente.is_empty(), t("Predeterminada")).clicked() {
                            t2.fuente.clear();
                        }
                        for f in crate::render::available_fonts() {
                            if ui.selectable_label(t2.fuente == f, &f).clicked() {
                                t2.fuente = f.clone();
                            }
                        }
                    });
                });
                if !t2.fuente.is_empty() && !crate::render::available_fonts().contains(&t2.fuente) {
                    ui.label(RichText::new(format!("{} «{}». {}", t("No tienes la fuente"), t2.fuente, t("Búscala en Comunidad › Fuentes."))).small().color(theme::WARN()));
                }
                if t2 != actual {
                    if t2.nombre == actual.nombre && theme::predeterminados().iter().any(|p| p.nombre == t2.nombre) {
                        t2.nombre = format!("{} ({})", actual.nombre, t("personalizado"));
                        t2.autor = String::new();
                    }
                    nuevo = Some(t2.clone());
                }
                ui.separator();
                // ── Save and share ──
                ui.horizontal(|ui| {
                    ui.label(t("Nombre"));
                    if self.tema_nombre.is_empty() {
                        self.tema_nombre = actual.nombre.clone();
                    }
                    ui.add(egui::TextEdit::singleline(&mut self.tema_nombre).desired_width(200.0));
                });
                ui.horizontal_wrapped(|ui| {
                    if ui.button(t("Guardar como tema")).clicked() {
                        let mut g = self.settings.tema.clone().unwrap_or_default();
                        g.nombre = self.tema_nombre.trim().to_string();
                        if let Some(p) = self.comunidad.perfil_nombre() {
                            g.autor = p;
                        }
                        match guardar(&g) {
                            Ok(_) => {
                                self.status = format!("{} «{}»", t("Tema guardado:"), g.nombre);
                                nuevo = Some(g);
                            }
                            Err(e) => self.status = e,
                        }
                    }
                    if ui.button(t("Compartir en la comunidad")).clicked() {
                        let mut g = self.settings.tema.clone().unwrap_or_default();
                        g.nombre = self.tema_nombre.trim().to_string();
                        if let Some(p) = self.comunidad.perfil_nombre() {
                            g.autor = p;
                        }
                        match g.validar().and_then(|_| {
                            let dir = std::env::temp_dir().join("lienzo-ui-comunidad");
                            std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
                            let f = dir.join(format!("{}.json", nombre_archivo(&g.nombre)));
                            std::fs::write(&f, serde_json::to_string_pretty(&g).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
                            Ok(f)
                        }) {
                            Ok(f) => {
                                self.comunidad_preparar_subida(ctx, "tema", f, &g.nombre);
                                self.temas_abierto = false;
                            }
                            Err(e) => self.status = e,
                        }
                    }
                    if ui.button(t("Volver al predeterminado")).clicked() {
                        nuevo = Some(Tema::default());
                    }
                });
            });
        });
        if let Some(p) = borrar {
            let _ = std::fs::remove_file(p);
        }
        if let Some(n) = nuevo {
            self.usar_tema(ctx, n);
        }
        if !abierta {
            self.temas_abierto = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nombres_de_archivo_seguros() {
        assert_eq!(nombre_archivo("Mi tema ✨"), "Mi_tema");
        assert_eq!(nombre_archivo("../../malo"), "malo");
        assert_eq!(nombre_archivo("   "), "tema");
    }
}
