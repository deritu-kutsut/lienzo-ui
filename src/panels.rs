//! Header, left rail + drawer (templates, elements, text, images, hierarchy),
//! contextual toolbar, Unity-style inspector, status bar, save and export windows.

use crate::app::{Drop, EditorApp, SCREEN_PRESETS, SIZE_PRESETS, Tab};
use crate::model::{self, Element, Frame, Kind, LayoutMode, Rgba, rect_transform_in};
use crate::render;
use crate::theme::{self, ACCENT, ACCENT_SOFT, BORDER, FIELD, HOVER, LINE, MUTED, PANEL, PANEL_DARK, TEXT};
use eframe::egui::{self, Align, Color32, CornerRadius, FontId, Layout, Margin, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, Vec2};

fn rgba_edit(ui: &mut egui::Ui, c: &mut Rgba) -> egui::Response {
    let mut col = Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]);
    let r = egui::color_picker::color_edit_button_srgba(ui, &mut col, egui::color_picker::Alpha::OnlyBlend);
    if r.changed() {
        *c = col.to_srgba_unmultiplied();
    }
    r
}

const SWATCHES: &[Rgba] = &[
    [255, 255, 255, 255],
    [20, 20, 30, 255],
    [124, 92, 255, 255],
    [0, 196, 204, 255],
    [255, 145, 77, 255],
    [235, 70, 90, 255],
    [255, 214, 80, 255],
    [70, 190, 120, 255],
    [60, 130, 240, 255],
    [10, 10, 20, 160],
];

fn swatch_row(ui: &mut egui::Ui, target: &mut Rgba) {
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(4.0);
        for s in SWATCHES {
            let (r, resp) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
            ui.painter().rect_filled(r, 9.0, render::col(*s, 1.0));
            ui.painter().rect_stroke(r, 9.0, Stroke::new(1.0, LINE()), StrokeKind::Inside);
            if resp.clicked() {
                *target = *s;
            }
        }
    });
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(8.0);
    ui.label(RichText::new(crate::i18n::t(title)).strong().size(12.5).color(TEXT()));
    ui.add_space(3.0);
}

fn hint(ui: &mut egui::Ui, t: &str) {
    ui.label(RichText::new(t).small().color(MUTED()));
}

/// Unity-style component block (collapsible header with a bar).
fn component(ui: &mut egui::Ui, id: &str, title: &str, open: bool, add: impl FnOnce(&mut egui::Ui)) {
    ui.add_space(4.0);
    egui::Frame::new().fill(PANEL_DARK()).corner_radius(6.0).inner_margin(Margin::symmetric(8, 4)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        egui::CollapsingHeader::new(RichText::new(title).strong().size(12.5)).id_salt(id).default_open(open).show(ui, |ui| {
            ui.add_space(2.0);
            add(ui);
        });
    });
}

/// A tile that previews something; click = insert, drag = drop on canvas.
fn tile(ui: &mut egui::Ui, size: Vec2, label: &str, paint: impl FnOnce(&egui::Painter, Rect)) -> egui::Response {
    let (r, resp) = ui.allocate_exact_size(size + Vec2::new(0.0, 18.0), Sense::click_and_drag());
    let img = Rect::from_min_size(r.min, size);
    ui.painter().rect_filled(img, 7.0, if resp.hovered() { HOVER() } else { FIELD() });
    if resp.hovered() {
        ui.painter().rect_stroke(img, 7.0, Stroke::new(1.5, ACCENT()), StrokeKind::Inside);
    }
    paint(&ui.painter().with_clip_rect(img.shrink(2.0)), img.shrink(10.0));
    ui.painter().text(Pos2::new(img.center().x, img.bottom() + 9.0), egui::Align2::CENTER_CENTER, label, FontId::proportional(11.0), MUTED());
    resp.on_hover_cursor(egui::CursorIcon::Grab)
}

fn drag_f(ui: &mut egui::Ui, label: &str, v: &mut f32, speed: f32) -> bool {
    ui.label(RichText::new(label).size(11.5).color(MUTED()));
    ui.add(egui::DragValue::new(v).speed(speed).max_decimals(2)).changed()
}

impl EditorApp {
    // ------------------------------------------------------------- header
    pub fn header(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("header")
            .exact_size(44.0)
            .frame(egui::Frame::new().fill(PANEL_DARK()).inner_margin(Margin::symmetric(10, 6)).stroke(Stroke::new(1.0, BORDER())))
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.add(egui::Image::new(egui::include_image!("../assets/marca/icono.svg")).fit_to_exact_size(Vec2::splat(22.0)));
                    ui.label(RichText::new(crate::i18n::t("Lienzo UI")).strong().size(14.5).color(TEXT()));
                    ui.label(RichText::new(crate::i18n::t("para Unity")).size(11.5).color(MUTED()));
                    ui.add_space(8.0);
                    ui.menu_button(crate::i18n::t("Archivo"), |ui| {
                        ui.set_min_width(230.0);
                        if ui.button(crate::i18n::t("Nuevo diseño en blanco")).clicked() {
                            self.load_template(model::blank_template());
                            ui.close();
                        }
                        if ui.button(crate::i18n::t("Abrir proyecto…")).clicked() {
                            ui.close();
                            self.open_project_dialog();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new(crate::i18n::t("Guardar")).shortcut_text("Ctrl+S")).clicked() {
                            ui.close();
                            if std::path::Path::new(&self.project_path).is_absolute() { self.save_project() } else { self.save_project_as() }
                        }
                        if ui.button(crate::i18n::t("Guardar proyecto como…")).clicked() {
                            ui.close();
                            self.save_project_as();
                        }
                        if ui.button(crate::i18n::t("Guardar como plantilla…")).clicked() {
                            self.save_open = true;
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(crate::i18n::t("Buscar actualizaciones…")).clicked() {
                            ui.close();
                            self.actualizador.abierta = true;
                            self.actualizador_buscar();
                        }
                        ui.separator();
                        if ui.button(crate::i18n::t("Subir imágenes…")).clicked() {
                            ui.close();
                            self.upload_images();
                            self.tab = Tab::Images;
                            self.drawer_open = true;
                        }
                        if ui.button(crate::i18n::t("Subir elementos…")).clicked() {
                            ui.close();
                            self.upload_elements();
                            self.tab = Tab::Elements;
                            self.drawer_open = true;
                        }
                        if ui.button(crate::i18n::t("Subir plantillas…")).clicked() {
                            ui.close();
                            self.upload_templates();
                            self.tab = Tab::Templates;
                            self.drawer_open = true;
                        }
                        if ui.button(crate::i18n::t("Añadir fuentes…")).clicked() {
                            ui.close();
                            let ctx = ui.ctx().clone();
                            self.add_fonts(&ctx);
                        }
                        ui.separator();
                        if ui.add(egui::Button::new(crate::i18n::t("Exportar al proyecto de Unity")).shortcut_text("Ctrl+E")).clicked() {
                            ui.close();
                            self.export_to_unity(false);
                        }
                        if ui.add(egui::Button::new(crate::i18n::t("Exportar… (código y opciones)")).shortcut_text("Ctrl+Shift+E")).clicked() {
                            self.export_open = true;
                            ui.close();
                        }
                    });
                    ui.menu_button(crate::i18n::t("Editar"), |ui| {
                        ui.set_min_width(240.0);
                        let has_sel = !self.sel.is_empty();
                        let multi = self.sel.len() > 1;
                        if ui.add_enabled(self.can_undo(), egui::Button::new(crate::i18n::t("Deshacer")).shortcut_text("Ctrl+Z")).clicked() {
                            self.undo();
                            ui.close();
                        }
                        if ui.add_enabled(self.can_redo(), egui::Button::new(crate::i18n::t("Rehacer")).shortcut_text("Ctrl+Y")).clicked() {
                            self.redo();
                            ui.close();
                        }
                        ui.separator();
                        if ui.add_enabled(has_sel, egui::Button::new(crate::i18n::t("Copiar")).shortcut_text("Ctrl+C")).clicked() {
                            self.copy();
                            ui.close();
                        }
                        if ui.add_enabled(!self.clipboard.is_empty(), egui::Button::new(crate::i18n::t("Pegar")).shortcut_text("Ctrl+V")).clicked() {
                            self.paste();
                            ui.close();
                        }
                        if ui.add_enabled(has_sel, egui::Button::new(crate::i18n::t("Duplicar")).shortcut_text("Ctrl+D")).clicked() {
                            self.duplicate_selection(30.0);
                            ui.close();
                        }
                        if ui.add_enabled(has_sel, egui::Button::new(crate::i18n::t("Eliminar")).shortcut_text("Supr")).clicked() {
                            self.delete_selection();
                            ui.close();
                        }
                        ui.separator();
                        if ui.add_enabled(has_sel, egui::Button::new(crate::i18n::t("Agrupar en Contenedor")).shortcut_text("Ctrl+G")).clicked() {
                            self.group_selection();
                            ui.close();
                        }
                        if ui.add_enabled(self.selection_grouped(), egui::Button::new(crate::i18n::t("Desagrupar")).shortcut_text("Ctrl+Shift+G")).clicked() {
                            self.ungroup_selection();
                            ui.close();
                        }
                        if ui.add_enabled(has_sel, egui::Button::new(crate::i18n::t("Sacar del padre"))).clicked() {
                            self.unparent_selection();
                            ui.close();
                        }
                        if ui.add_enabled(has_sel, egui::Button::new(crate::i18n::t("Bloquear / desbloquear")).shortcut_text("Ctrl+Shift+L")).clicked() {
                            self.toggle_lock();
                            ui.close();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new(crate::i18n::t("Seleccionar todo")).shortcut_text("Ctrl+A")).clicked() {
                            self.sel = self.doc.elements.iter().filter(|e| !e.locked && e.visible && e.parent.is_none()).map(|e| e.id).collect();
                            ui.close();
                        }
                        let _ = multi;
                    });
                    ui.menu_button(crate::i18n::t("Ver"), |ui| {
                        ui.set_min_width(250.0);
                        ui.checkbox(&mut self.rulers, crate::i18n::t("Reglas"));
                        ui.checkbox(&mut self.grid.show, crate::i18n::t("Mostrar cuadrícula"));
                        ui.checkbox(&mut self.grid.snap, crate::i18n::t("Ajustar a la cuadrícula"));
                        egui::Grid::new("gridset").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                            ui.label(crate::i18n::t("Celda (ancho)"));
                            ui.add(egui::DragValue::new(&mut self.grid.w).range(2.0..=2000.0).speed(1.0).suffix(" px"));
                            ui.end_row();
                            ui.label(crate::i18n::t("Celda (alto)"));
                            ui.add(egui::DragValue::new(&mut self.grid.h).range(2.0..=2000.0).speed(1.0).suffix(" px"));
                            ui.end_row();
                            ui.label(crate::i18n::t("Columnas"));
                            let mut cols = (self.doc.width / self.grid.w).round().max(1.0) as u32;
                            if ui.add(egui::DragValue::new(&mut cols).range(1..=200)).changed() {
                                self.grid.w = self.doc.width / cols as f32;
                            }
                            ui.end_row();
                            ui.label(crate::i18n::t("Filas"));
                            let mut rows = (self.doc.height / self.grid.h).round().max(1.0) as u32;
                            if ui.add(egui::DragValue::new(&mut rows).range(1..=200)).changed() {
                                self.grid.h = self.doc.height / rows as f32;
                            }
                            ui.end_row();
                            ui.label(crate::i18n::t("Color"));
                            rgba_edit(ui, &mut self.grid.color);
                            ui.end_row();
                        });
                        ui.horizontal_wrapped(|ui| {
                            for s in [8.0, 16.0, 32.0, 50.0, 64.0, 100.0] {
                                if ui.small_button(format!("{}", s)).clicked() {
                                    self.grid.w = s;
                                    self.grid.h = s;
                                    self.grid.show = true;
                                }
                            }
                        });
                        ui.separator();
                        if ui.add(egui::Button::new(crate::i18n::t("Ajustar a la ventana")).shortcut_text("Ctrl+0")).clicked() {
                            self.fit_pending = true;
                            ui.close();
                        }
                    });
                    ui.menu_button(crate::i18n::t("Lienzo"), |ui| {
                        ui.label(RichText::new(crate::i18n::t("Resolución de referencia (CanvasScaler)")).small().color(MUTED()));
                        for (name, w, h) in SIZE_PRESETS {
                            if ui.button(format!("{}  ·  {} × {}", name, w, h)).clicked() {
                                self.resize_page(*w, *h);
                                ui.close();
                            }
                        }
                    });
                    ui.add_space(4.0);
                    if ui.add_enabled_ui(self.can_undo(), |ui| crate::icons::button(ui, "undo", "<", "Deshacer (Ctrl+Z)")).inner.clicked() {
                        self.undo();
                    }
                    if ui.add_enabled_ui(self.can_redo(), |ui| crate::icons::button(ui, "redo", ">", "Rehacer (Ctrl+Y)")).inner.clicked() {
                        self.redo();
                    }
                    ui.add_space(4.0);
                    let play = crate::i18n::t(if self.preview { "Detener" } else { "Probar" });
                    let tinta = if self.preview { Color32::WHITE } else { TEXT() };
                    let play_btn = crate::icons::btn_tint(if self.preview { "stop" } else { "play" }, RichText::new(play).strong().color(tinta), tinta).fill(if self.preview { theme::OK() } else { FIELD() });
                    if ui.add(play_btn).on_hover_text(crate::i18n::t("Modo prueba: clic ejecuta el código y puedes simular otras pantallas (F5)")).clicked() {
                        self.set_preview(!self.preview);
                    }
                    if self.preview {
                        let cur = match self.preview_screen {
                            Some(s) => format!("{} × {}", s[0], s[1]),
                            None => "Resolución de referencia".into(),
                        };
                        egui::ComboBox::from_id_salt("screen").selected_text(cur).width(190.0).show_ui(ui, |ui| {
                            for (name, w, h) in SCREEN_PRESETS {
                                let v = if *w == 0.0 { None } else { Some([*w, *h]) };
                                ui.selectable_value(&mut self.preview_screen, v, *name);
                            }
                        });
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        self.comunidad_boton_perfil(ui);
                        ui.add_space(4.0);
                        self.actualizador_boton(ui);
                        let export = crate::icons::btn_tint("export", RichText::new(crate::i18n::t("Exportar a Unity")).strong().color(Color32::WHITE), Color32::WHITE).fill(ACCENT()).corner_radius(6.0);
                        let tip = if self.settings.unity_project.is_empty() { "Elige la carpeta de tu proyecto de Unity (Ctrl+E)".to_string() } else { format!("Exporta a {} (Ctrl+E)", self.settings.unity_project) };
                        if ui.add(export).on_hover_text(tip).clicked() {
                            self.export_to_unity(false);
                        }
                        if ui.add(crate::icons::btn("code", crate::i18n::t("Código…"))).on_hover_text(crate::i18n::t("Ver y copiar JSON, UXML/USS, IMGUI y scripts (Ctrl+Shift+E)")).clicked() {
                            self.export_open = true;
                        }
                        if ui.add(crate::icons::btn("save", crate::i18n::t("Guardar"))).on_hover_text(crate::i18n::t("Guardar como proyecto o como plantilla")).clicked() {
                            self.save_open = true;
                        }
                        if ui.button(crate::i18n::t("Tutorial")).on_hover_text(crate::i18n::t("Abre el tutorial para novatos y profesionales (F1)")).clicked() {
                            self.abrir_tutorial(crate::tutorial::Pista::Novato);
                        }
                        if ui.button(crate::i18n::t("Temas")).on_hover_text(crate::i18n::t("Colores, esquinas y fuente de la app; temas de la comunidad")).clicked() {
                            self.temas_abierto = true;
                        }
                        if ui.button(crate::i18n::t("Accesibilidad")).on_hover_text(crate::i18n::t("Idioma, tamaño de la interfaz, alto contraste y movimiento reducido")).clicked() {
                            self.accesibilidad_abierta = true;
                        }
                        ui.add_space(6.0);
                        ui.label(RichText::new(format!("{} × {}", self.doc.width, self.doc.height)).size(11.5).color(MUTED()));
                        ui.add(egui::TextEdit::singleline(&mut self.doc.title).desired_width(180.0));
                    });
                });
            });
    }

    // ------------------------------------------------------------- left rail
    pub fn rail(&mut self, ui: &mut egui::Ui) {
        egui::Panel::left("rail")
            .exact_size(70.0)
            .resizable(false)
            .frame(egui::Frame::new().fill(PANEL_DARK()).inner_margin(Margin::symmetric(4, 8)).stroke(Stroke::new(1.0, BORDER())))
            .show(ui, |ui| {
                let tabs = [(Tab::Layers, "Jerarquía"), (Tab::Elements, "Elementos"), (Tab::Templates, "Plantillas"), (Tab::Text, "Texto"), (Tab::Images, "Imágenes"), (Tab::Plugins, "Plugins"), (Tab::Comunidad, "Comunidad")];
                ui.vertical_centered(|ui| {
                    for (t, label) in tabs {
                        let active = self.tab == t && self.drawer_open;
                        let (r, resp) = ui.allocate_exact_size(Vec2::new(62.0, 58.0), Sense::click());
                        if active {
                            ui.painter().rect_filled(r, 7.0, ACCENT_SOFT());
                        } else if resp.hovered() {
                            ui.painter().rect_filled(r, 7.0, HOVER());
                        }
                        let ic = Rect::from_center_size(Pos2::new(r.center().x, r.top() + 21.0), Vec2::splat(20.0));
                        let col = if active { Color32::WHITE } else { MUTED() };
                        if !crate::icons::paint(ui, crate::icons::tab_name(t), ic, col) {
                            rail_icon(ui.painter(), t, ic, col);
                        }
                        ui.painter().text(Pos2::new(r.center().x, r.bottom() - 12.0), egui::Align2::CENTER_CENTER, crate::i18n::t(label), FontId::proportional(10.5), col);
                        if resp.clicked() {
                            if self.tab == t {
                                self.drawer_open = !self.drawer_open;
                            } else {
                                self.tab = t;
                                self.drawer_open = true;
                            }
                        }
                        ui.add_space(3.0);
                    }
                });
            });
    }

    // ------------------------------------------------------------- drawer
    pub fn drawer(&mut self, ui: &mut egui::Ui) {
        if !self.drawer_open || self.preview {
            return;
        }
        egui::Panel::left("drawer")
            .default_size(300.0)
            .size_range(240.0..=480.0)
            .resizable(true)
            .frame(egui::Frame::new().fill(PANEL()).inner_margin(Margin::same(10)).stroke(Stroke::new(1.0, BORDER())))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                    crate::icons::paint_or(ui, "search", r, MUTED(), "?");
                    ui.add(egui::TextEdit::singleline(&mut self.search).hint_text(crate::i18n::t("Buscar…")).desired_width(f32::INFINITY).margin(Margin::symmetric(8, 5)));
                });
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match self.tab {
                    Tab::Templates => self.tab_templates(ui),
                    Tab::Elements => self.tab_elements(ui),
                    Tab::Text => self.tab_text(ui),
                    Tab::Images => self.tab_images(ui),
                    Tab::Layers => self.tab_layers(ui),
                    Tab::Plugins => self.tab_plugins(ui),
                    Tab::Comunidad => self.tab_comunidad(ui),
                });
            });
    }

    fn matches(&self, label: &str) -> bool {
        self.search.trim().is_empty() || label.to_lowercase().contains(&self.search.trim().to_lowercase())
    }

    fn place_or_drag(&mut self, resp: &egui::Response, proto: Element) {
        if resp.drag_started() {
            self.dnd = Some(vec![proto]);
        } else if resp.clicked() {
            self.insert(proto, None);
        }
    }

    fn tab_templates(&mut self, ui: &mut egui::Ui) {
        section(ui, "Plantillas de interfaz");
        hint(ui, &crate::i18n::t("Clic para reemplazar el diseño (Ctrl+Z deshace). Todas usan jerarquía, anclas y Layout Groups."));
        ui.add_space(6.0);
        let (w, h) = (self.doc.width, self.doc.height);
        let mut chosen = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            for (i, name) in model::TEMPLATES.iter().enumerate() {
                if !self.matches(name) {
                    continue;
                }
                let doc = model::template(i, w, h);
                let resp = tile(ui, Vec2::new(128.0, 82.0), &crate::i18n::t(name), |p, r| render::paint_doc_thumb(p, &doc, r.expand(6.0)));
                if resp.clicked() {
                    chosen = Some(i);
                }
            }
        });
        if let Some(i) = chosen {
            self.load_template(i);
        }

        section(ui, "Mis plantillas");
        ui.horizontal(|ui| {
            if ui.add(crate::icons::btn("plus", crate::i18n::t("Subir plantilla"))).on_hover_text(crate::i18n::t("Archivos .json guardados con este editor")).clicked() {
                self.upload_templates();
            }
            if ui.add(crate::icons::btn("save", crate::i18n::t("Guardar diseño actual"))).clicked() {
                self.save_open = true;
            }
        });
        ui.add_space(6.0);
        if self.library.templates.is_empty() {
            hint(ui, &crate::i18n::t("Aún no tienes plantillas propias. Guarda tu diseño como plantilla o sube una."));
        }
        let mut load = None;
        let mut remove = None;
        let templates = self.library.templates.clone();
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            for t in &templates {
                if !self.matches(&t.name) {
                    continue;
                }
                let resp = tile(ui, Vec2::new(128.0, 82.0), &t.name, |p, r| render::paint_doc_thumb(p, &t.doc, r.expand(6.0)));
                if resp.clicked() {
                    load = Some(t.doc.clone());
                }
                resp.context_menu(|ui| {
                    if ui.button(crate::i18n::t("Usar plantilla")).clicked() {
                        load = Some(t.doc.clone());
                        ui.close();
                    }
                    if ui.add(crate::icons::btn("trash", crate::i18n::t("Eliminar de mi biblioteca"))).clicked() {
                        remove = Some(t.path.clone());
                        ui.close();
                    }
                });
            }
        });
        if let Some(d) = load {
            self.load_user_template(&d);
        }
        if let Some(p) = remove {
            self.library.remove(&p);
        }
    }

    fn my_elements(&mut self, ui: &mut egui::Ui) {
        section(ui, "Mis elementos (prefabs reutilizables)");
        ui.horizontal_wrapped(|ui| {
            if ui.add(crate::icons::btn("plus", crate::i18n::t("Subir elemento"))).on_hover_text(crate::i18n::t("Imágenes PNG/JPG (íconos con transparencia) o elementos .json")).clicked() {
                self.upload_elements();
            }
        });
        if !self.sel.is_empty() {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.element_name).hint_text(crate::i18n::t("Nombre")).desired_width(120.0));
                if ui.button(crate::i18n::t("Guardar selección")).on_hover_text(crate::i18n::t("Guarda lo seleccionado (con sus hijos) para reutilizarlo")).clicked() {
                    let n = std::mem::take(&mut self.element_name);
                    self.save_selection_as_element(&n);
                }
            });
        }
        ui.add_space(6.0);
        if self.library.elements.is_empty() {
            hint(ui, &crate::i18n::t("Sube imágenes o guarda una selección para tenerla aquí."));
        }
        let items = self.library.elements.clone();
        let mut remove = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            for it in &items {
                if !self.matches(&it.name) {
                    continue;
                }
                let resp = tile(ui, Vec2::new(80.0, 66.0), &it.name, |p, r| {
                    let b = crate::app::bounds(&it.items.iter().collect::<Vec<_>>());
                    let mut doc = model::Doc::empty((b[2] - b[0]).max(1.0), (b[3] - b[1]).max(1.0));
                    doc.background = [0, 0, 0, 0];
                    doc.elements = it.items.clone();
                    render::paint_doc_thumb(p, &doc, r);
                });
                if resp.drag_started() {
                    self.dnd = Some(it.items.clone());
                } else if resp.clicked() {
                    if it.items.len() == 1 {
                        self.insert(it.items[0].clone(), None);
                    } else {
                        self.insert_items(&it.items, None);
                    }
                }
                resp.context_menu(|ui| {
                    if ui.add(crate::icons::btn("trash", crate::i18n::t("Eliminar de mi biblioteca"))).clicked() {
                        remove = Some(it.path.clone());
                        ui.close();
                    }
                });
            }
        });
        if let Some(p) = remove {
            self.library.remove(&p);
        }
    }

    fn tab_elements(&mut self, ui: &mut egui::Ui) {
        if let Some(p) = self.insert_parent().and_then(|p| self.doc.get(p)) {
            egui::Frame::new().fill(ACCENT_SOFT()).corner_radius(6.0).inner_margin(Margin::same(6)).show(ui, |ui| {
                ui.label(RichText::new(format!("Se agregarán dentro de «{}»", p.name)).small().color(TEXT()));
            });
        }
        let shapes: Vec<(&str, Element)> = {
            let mut sq = Element::new(0, Kind::Panel);
            sq.radius = 0.0;
            sq.fill = [124, 92, 255, 255];
            sq.w = 240.0;
            sq.h = 240.0;
            let mut rr = Element::new(0, Kind::Panel);
            rr.fill = [0, 196, 204, 255];
            rr.w = 300.0;
            rr.h = 200.0;
            rr.radius = 36.0;
            let ci = Element::new(0, Kind::Ellipse);
            let mut ol = Element::new(0, Kind::Panel);
            ol.fill = [0, 0, 0, 0];
            ol.stroke = [20, 20, 30, 255];
            ol.stroke_w = 6.0;
            ol.radius = 12.0;
            ol.w = 280.0;
            ol.h = 180.0;
            let mut bar = Element::new(0, Kind::Panel);
            bar.fill = [20, 20, 30, 255];
            bar.radius = 0.0;
            bar.w = 400.0;
            bar.h = 8.0;
            vec![("Cuadrado", sq), ("Redondeado", rr), ("Círculo", ci), ("Contorno", ol), ("Línea", bar)]
        };
        let controls: Vec<(&str, Element)> = [Kind::Button, Kind::Toggle, Kind::Slider, Kind::InputField, Kind::Dropdown, Kind::Panel, Kind::Image, Kind::ScrollView]
            .iter()
            .map(|k| (k.label(), Element::new(0, *k)))
            .collect();
        let containers: Vec<(&str, Element)> = {
            let mut c = Element::new(0, Kind::Empty);
            c.name = "Contenedor".into();
            let mut row = Element::new(0, Kind::Empty);
            row.name = "Fila".into();
            row.w = 600.0;
            row.h = 120.0;
            row.layout.mode = LayoutMode::Horizontal;
            row.layout.align = 4;
            let mut col = Element::new(0, Kind::Empty);
            col.name = "Columna".into();
            col.w = 360.0;
            col.h = 480.0;
            col.layout.mode = LayoutMode::Vertical;
            col.layout.align = 1;
            col.layout.expand_w = true;
            let mut grid = Element::new(0, Kind::Panel);
            grid.name = "Rejilla".into();
            grid.w = 560.0;
            grid.h = 400.0;
            grid.fill = [255, 255, 255, 40];
            grid.layout.mode = LayoutMode::Grid;
            grid.layout.align = 1;
            vec![("Contenedor", c), ("Fila (Horizontal)", row), ("Columna (Vertical)", col), ("Rejilla (Grid)", grid)]
        };
        for (title, items) in [("Controles de UI (uGUI)", controls), ("Contenedores y Layout Groups", containers), ("Formas", shapes)] {
            section(ui, title);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                for (label, proto) in items {
                    if !self.matches(label) {
                        continue;
                    }
                    let resp = tile(ui, Vec2::new(80.0, 66.0), label, |p, r| {
                        if proto.kind == Kind::Empty {
                            let c = r.shrink(6.0);
                            p.rect_stroke(c, 3.0, Stroke::new(1.0, ACCENT()), StrokeKind::Inside);
                            let n = match proto.layout.mode {
                                LayoutMode::Horizontal => (3, 1),
                                LayoutMode::Vertical => (1, 3),
                                _ => (0, 0),
                            };
                            for i in 0..(n.0 * n.1) {
                                let (cx, cy) = (i % n.0, i / n.0);
                                let cw = (c.width() - 8.0) / n.0 as f32;
                                let ch = (c.height() - 8.0) / n.1 as f32;
                                let b = Rect::from_min_size(c.min + Vec2::new(4.0 + cx as f32 * cw, 4.0 + cy as f32 * ch), Vec2::new(cw - 3.0, ch - 3.0));
                                p.rect_filled(b, 2.0, ACCENT_SOFT());
                            }
                        } else {
                            render::paint_element_thumb(p, &proto, r)
                        }
                    });
                    self.place_or_drag(&resp, proto);
                }
            });
        }
        ui.add_space(6.0);
        hint(ui, &crate::i18n::t("Clic agrega al centro (o dentro del panel/contenedor seleccionado). Arrastra para soltar donde quieras: si sueltas sobre un panel queda como hijo."));
        self.my_elements(ui);
    }

    fn tab_text(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        let add = crate::icons::btn_tint("text", RichText::new(crate::i18n::t("Agregar un cuadro de texto")).color(Color32::WHITE).strong(), Color32::WHITE).fill(ACCENT()).corner_radius(6.0).min_size(Vec2::new(ui.available_width(), 36.0));
        if ui.add(add).clicked() {
            let mut e = Element::new(0, Kind::Text);
            e.font_size = 32.0;
            self.insert(e, None);
            self.editing = self.sel.first().copied();
            self.editing_new = true;
        }
        hint(ui, &crate::i18n::t("En Unity se crean con TextMeshPro (TextMeshProUGUI)."));
        section(ui, "Estilos de texto");
        let styles: [(&str, f32, bool, f32); 4] = [("Agregar un título", 80.0, true, 900.0), ("Agregar un subtítulo", 52.0, true, 700.0), ("Agregar texto de cuerpo", 30.0, false, 600.0), ("Etiqueta pequeña", 22.0, false, 360.0)];
        for (label, size, bold, w) in styles {
            if !self.matches(label) {
                continue;
            }
            let shown = (size * 0.3).clamp(12.0, 24.0);
            let (r, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), shown + 22.0), Sense::click_and_drag());
            ui.painter().rect_filled(r, 7.0, if resp.hovered() { HOVER() } else { FIELD() });
            let pos = Pos2::new(r.left() + 12.0, r.center().y);
            ui.painter().text(pos, egui::Align2::LEFT_CENTER, label, FontId::proportional(shown), TEXT());
            if bold {
                ui.painter().text(pos + Vec2::new(0.7, 0.0), egui::Align2::LEFT_CENTER, label, FontId::proportional(shown), TEXT());
            }
            let mut e = Element::new(0, Kind::Text);
            e.text = label.replace("Agregar un ", "").replace("Agregar ", "");
            if let Some(f) = e.text.get_mut(0..1) {
                f.make_ascii_uppercase();
            }
            e.font_size = size;
            e.bold = bold;
            e.w = w;
            e.h = size * 1.3;
            self.place_or_drag(&resp.on_hover_cursor(egui::CursorIcon::Grab), e);
            ui.add_space(4.0);
        }
        section(ui, "Para botones y HUD");
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            let mut score = Element::new(0, Kind::Text);
            score.text = "000 000".into();
            score.font_size = 60.0;
            score.bold = true;
            score.text_color = [255, 200, 60, 255];
            score.w = 360.0;
            let mut tag = Element::new(0, Kind::Button);
            tag.text = "NUEVO".into();
            tag.fill = [235, 70, 90, 255];
            tag.radius = 40.0;
            tag.w = 200.0;
            tag.h = 64.0;
            for (label, proto) in [("Marcador", score), ("Etiqueta", tag)] {
                let resp = tile(ui, Vec2::new(124.0, 60.0), label, |p, r| {
                    p.rect_filled(r.expand(8.0), 6.0, Color32::from_rgb(28, 28, 38));
                    render::paint_element_thumb(p, &proto, r);
                });
                self.place_or_drag(&resp, proto);
            }
        });
    }

    fn tab_images(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        let up = crate::icons::btn_tint("plus", RichText::new(crate::i18n::t("Subir imágenes")).color(Color32::WHITE).strong(), Color32::WHITE).fill(ACCENT()).corner_radius(6.0).min_size(Vec2::new(ui.available_width(), 36.0));
        if ui.add(up).on_hover_text(crate::i18n::t("PNG, JPG, BMP, GIF o WEBP — se exportan como Sprites")).clicked() {
            self.upload_images();
        }
        section(ui, "Tus imágenes");
        if self.library.images.is_empty() {
            hint(ui, &crate::i18n::t("Las imágenes que subas aparecerán aquí y se guardan para tus próximos diseños."));
        }
        let images = self.library.images.clone();
        let mut remove = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            for im in &images {
                if !self.matches(&im.file) {
                    continue;
                }
                let resp = tile(ui, Vec2::new(124.0, 84.0), &im.file, |p, r| {
                    let e = crate::library::image_element(&im.file, im.w, im.h, 300.0);
                    render::paint_element_thumb(p, &e, r.expand(8.0));
                });
                if resp.drag_started() {
                    let max = (self.doc.width.min(self.doc.height) * 0.5).max(100.0);
                    self.dnd = Some(vec![crate::library::image_element(&im.file, im.w, im.h, max)]);
                } else if resp.clicked() {
                    self.insert_library_image(&im.file, None);
                }
                resp.context_menu(|ui| {
                    if ui.add(crate::icons::btn("trash", crate::i18n::t("Eliminar de mi biblioteca"))).clicked() {
                        remove = Some(crate::library::images_dir().join(&im.file));
                        ui.close();
                    }
                });
            }
        });
        if let Some(p) = remove {
            self.library.remove(&p);
        }
        section(ui, "Degradados (marcadores de posición)");
        hint(ui, &crate::i18n::t("En uGUI se exportan como un Sprite de degradado generado."));
        ui.add_space(4.0);
        let pairs: [(Rgba, Rgba, &str); 8] = [
            ([255, 126, 179, 255], [255, 117, 140, 255], "Atardecer"),
            ([46, 32, 110, 255], [12, 120, 150, 255], "Noche"),
            ([120, 190, 240, 255], [70, 140, 80, 255], "Paisaje"),
            ([255, 214, 120, 255], [255, 120, 60, 255], "Fuego"),
            ([0, 196, 204, 255], [124, 92, 255, 255], "Aurora"),
            ([240, 240, 245, 255], [180, 185, 200, 255], "Niebla"),
            ([30, 30, 40, 255], [90, 90, 110, 255], "Pizarra"),
            ([140, 230, 160, 255], [30, 130, 90, 255], "Bosque"),
        ];
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            for (a, b, name) in pairs {
                if !self.matches(name) {
                    continue;
                }
                let mut e = Element::new(0, Kind::Image);
                e.fill = a;
                e.fill2 = b;
                e.name = name.into();
                e.w = 480.0;
                e.h = 320.0;
                let resp = tile(ui, Vec2::new(124.0, 84.0), name, |p, r| {
                    let mut t = e.clone();
                    t.radius = 0.0;
                    render::paint_element_thumb(p, &t, r.expand(10.0));
                });
                self.place_or_drag(&resp, e);
            }
        });
    }

    // ------------------------------------------------------------- hierarchy
    fn tab_layers(&mut self, ui: &mut egui::Ui) {
        section(ui, &crate::i18n::tf("Jerarquía · {}", &[&self.doc.title]));
        hint(ui, &crate::i18n::t("Igual que en Unity: lo de abajo se dibuja encima. Arrastra filas para reordenar o meter un elemento dentro de otro."));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if ui.add(crate::icons::btn("empty", crate::i18n::t("Contenedor")).small()).on_hover_text(crate::i18n::t("RectTransform vacío (dentro de la selección si es un panel)")).clicked() {
                let mut c = Element::new(0, Kind::Empty);
                c.name = "Contenedor".into();
                self.insert(c, None);
            }
            if ui.add_enabled(!self.sel.is_empty(), egui::Button::new(crate::i18n::t("Agrupar")).small()).clicked() {
                self.group_selection();
            }
            if ui.add_enabled(self.selection_grouped(), egui::Button::new(crate::i18n::t("Desagrupar")).small()).clicked() {
                self.ungroup_selection();
            }
            if ui.add_enabled(self.sel.iter().any(|s| self.doc.parent_of(*s).is_some()), crate::icons::btn("parent_up", crate::i18n::t("Sacar")).small()).on_hover_text(crate::i18n::t("Sacar del padre")).clicked() {
                self.unparent_selection();
            }
        });
        ui.add_space(4.0);
        // rows: depth-first, honouring collapsed parents (search shows every match)
        let searching = !self.search.trim().is_empty();
        let mut rows: Vec<(u64, usize)> = Vec::new();
        let mut hidden_under: Vec<u64> = Vec::new();
        for e in &self.doc.elements {
            let anc = self.doc.ancestors(e.id);
            if !searching && anc.iter().any(|a| hidden_under.contains(a)) {
                continue;
            }
            if e.collapsed {
                hidden_under.push(e.id);
            }
            if searching && !self.matches(&e.name) {
                continue;
            }
            rows.push((e.id, anc.len()));
        }
        let mut rects: Vec<(u64, Rect)> = Vec::new();
        let row_h = 24.0;
        let w = ui.available_width();
        for (id, depth) in rows {
            let Some(e) = self.doc.get(id).cloned() else { continue };
            let (r, resp) = ui.allocate_exact_size(Vec2::new(w, row_h), Sense::click_and_drag());
            let selected = self.is_selected(id);
            let p = ui.painter();
            if selected {
                p.rect_filled(r, 4.0, ACCENT_SOFT());
            } else if resp.hovered() {
                p.rect_filled(r, 4.0, HOVER());
            }
            let x0 = r.left() + 4.0 + depth as f32 * 14.0;
            // collapse arrow
            let has_kids = self.doc.has_children(id);
            let arrow = Rect::from_min_size(Pos2::new(x0, r.top()), Vec2::new(14.0, row_h));
            if has_kids {
                let c = arrow.center();
                let pts = if e.collapsed {
                    vec![c + Vec2::new(-2.5, -4.0), c + Vec2::new(3.5, 0.0), c + Vec2::new(-2.5, 4.0)]
                } else {
                    vec![c + Vec2::new(-4.0, -2.5), c + Vec2::new(4.0, -2.5), c + Vec2::new(0.0, 3.5)]
                };
                p.add(egui::Shape::convex_polygon(pts, MUTED(), Stroke::NONE));
            }
            let dim = !self.doc.shown(id);
            let col = if dim { MUTED().gamma_multiply(0.7) } else { TEXT() };
            crate::icons::paint_or(ui, crate::icons::kind_name(e.kind), Rect::from_center_size(Pos2::new(x0 + 22.0, r.center().y), Vec2::splat(14.0)), if e.kind == Kind::Empty { ACCENT() } else { MUTED() }, e.kind.icon());
            let name_rect = Rect::from_min_max(Pos2::new(x0 + 32.0, r.top()), Pos2::new(r.right() - 48.0, r.bottom()));
            let mut badges: Vec<&str> = Vec::new();
            if !e.on_click.trim().is_empty() {
                badges.push("bolt");
            }
            if e.layout.mode != LayoutMode::None {
                badges.push("layout");
            }
            if e.mask || e.kind == Kind::ScrollView {
                badges.push("mask");
            }
            if self.renaming == Some(id) {
                let mut name = e.name.clone();
                let resp_t = ui.put(name_rect, egui::TextEdit::singleline(&mut name).margin(Margin::symmetric(2, 1)));
                resp_t.request_focus();
                if let Some(el) = self.doc.get_mut(id) {
                    el.name = name;
                }
                if resp_t.lost_focus() {
                    self.renaming = None;
                }
            } else {
                let galley = ui.painter().layout_no_wrap(e.name.clone(), FontId::proportional(12.5), col);
                let gw = galley.size().x;
                ui.painter().with_clip_rect(name_rect).galley(Pos2::new(name_rect.left(), r.center().y - galley.size().y / 2.0), galley, col);
                let mut bx = name_rect.left() + gw + 6.0;
                for b in badges {
                    let br = Rect::from_min_size(Pos2::new(bx, r.center().y - 6.0), Vec2::splat(12.0));
                    if br.right() < name_rect.right() {
                        crate::icons::paint_or(ui, b, br, if b == "bolt" { theme::WARN() } else { MUTED() }, "");
                    }
                    bx += 15.0;
                }
            }
            // eye / lock
            let eye = Rect::from_center_size(Pos2::new(r.right() - 34.0, r.center().y), Vec2::splat(18.0));
            let lock = Rect::from_center_size(Pos2::new(r.right() - 12.0, r.center().y), Vec2::splat(18.0));
            let eye_r = ui.interact(eye, ui.scope_id().with(("eye", id)), Sense::click());
            let lock_r = ui.interact(lock, ui.scope_id().with(("lock", id)), Sense::click());
            let arrow_r = if has_kids { Some(ui.interact(arrow, ui.scope_id().with(("arrow", id)), Sense::click())) } else { None };
            let show_icons = resp.hovered() || eye_r.hovered() || lock_r.hovered() || !e.visible || e.locked;
            if show_icons {
                crate::icons::paint_or(ui, if e.visible { "eye" } else { "eye_off" }, eye.shrink(2.0), if e.visible { MUTED() } else { theme::WARN() }, "o");
                crate::icons::paint_or(ui, if e.locked { "lock" } else { "unlock" }, lock.shrink(2.0), if e.locked { theme::WARN() } else { MUTED() }, "L");
            }
            if eye_r.on_hover_text(crate::i18n::t("Activo / inactivo (SetActive en Unity)")).clicked() {
                if let Some(el) = self.doc.get_mut(id) {
                    el.visible = !el.visible;
                }
            } else if lock_r.on_hover_text(crate::i18n::t("Bloquear en el editor")).clicked() {
                if let Some(el) = self.doc.get_mut(id) {
                    el.locked = !el.locked;
                }
            } else if arrow_r.map(|a| a.clicked()).unwrap_or(false) {
                if let Some(el) = self.doc.get_mut(id) {
                    el.collapsed = !el.collapsed;
                }
            } else if resp.double_clicked() {
                self.renaming = Some(id);
            } else if resp.clicked() {
                let m = ui.input(|i| i.modifiers);
                if m.shift || m.command {
                    self.toggle_select(id);
                } else {
                    self.sel = vec![id];
                }
            }
            if resp.drag_started() {
                self.tree_drag = Some(id);
                if !self.is_selected(id) {
                    self.sel = vec![id];
                }
            }
            resp.context_menu(|ui| {
                if !self.is_selected(id) {
                    self.sel = vec![id];
                }
                if ui.button(crate::i18n::t("Renombrar")).clicked() {
                    self.renaming = Some(id);
                    ui.close();
                }
                if ui.add(crate::icons::btn("duplicate", crate::i18n::t("Duplicar"))).clicked() {
                    self.duplicate_selection(30.0);
                    ui.close();
                }
                if ui.add(crate::icons::btn("empty", crate::i18n::t("Agrupar en Contenedor"))).clicked() {
                    self.group_selection();
                    ui.close();
                }
                if self.doc.parent_of(id).is_some() && ui.button(crate::i18n::t("Sacar del padre")).clicked() {
                    self.unparent_selection();
                    ui.close();
                }
                if ui.button(crate::i18n::t("Añadir Contenedor hijo")).clicked() {
                    self.sel = vec![id];
                    let mut c = Element::new(0, Kind::Empty);
                    c.name = "Contenedor".into();
                    if let Some(pe) = self.doc.get(id) {
                        c.w = pe.w * 0.8;
                        c.h = pe.h * 0.8;
                    }
                    self.insert(c, None);
                    ui.close();
                }
                ui.separator();
                if ui.add(crate::icons::btn("trash", crate::i18n::t("Eliminar"))).clicked() {
                    self.delete_selection();
                    ui.close();
                }
            });
            rects.push((id, r));
        }
        if self.doc.elements.is_empty() {
            hint(ui, &crate::i18n::t("El diseño está vacío."));
        }
        let rest_h = ui.available_height().max(40.0);
        let (rest, _) = ui.allocate_exact_size(Vec2::new(w, rest_h), Sense::hover());

        // drag & drop
        if let Some(dragged) = self.tree_drag {
            let (pointer, released) = ui.input(|i| (i.pointer.hover_pos(), i.pointer.any_released()));
            let mut target = None;
            if let Some(p) = pointer {
                for (id, r) in &rects {
                    if r.contains(p) {
                        let t = (p.y - r.top()) / r.height();
                        let painter = ui.painter();
                        target = Some(if t < 0.28 {
                            painter.line_segment([r.left_top(), r.right_top()], Stroke::new(2.0, ACCENT()));
                            Drop::Before(*id)
                        } else if t > 0.72 {
                            painter.line_segment([r.left_bottom(), r.right_bottom()], Stroke::new(2.0, ACCENT()));
                            Drop::After(*id)
                        } else {
                            painter.rect_stroke(*r, 4.0, Stroke::new(1.5, ACCENT()), StrokeKind::Inside);
                            Drop::Into(*id)
                        });
                    }
                }
                if target.is_none() && rest.contains(p) {
                    ui.painter().line_segment([rest.left_top(), rest.right_top()], Stroke::new(2.0, ACCENT()));
                    target = Some(Drop::Root);
                }
                if let Some(e) = self.doc.get(dragged) {
                    let fg = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::unique("tree_ghost")));
                    let g = fg.layout_no_wrap(e.name.clone(), FontId::proportional(12.5), Color32::WHITE);
                    let rr = Rect::from_min_size(p + Vec2::new(14.0, 6.0), g.size() + Vec2::new(12.0, 6.0));
                    fg.rect_filled(rr, 5.0, ACCENT());
                    fg.galley(rr.min + Vec2::new(6.0, 3.0), g, Color32::WHITE);
                }
            }
            if released {
                let moved = pointer.map(|p| rects.iter().find(|(i, _)| *i == dragged).map(|(_, r)| !r.contains(p)).unwrap_or(true)).unwrap_or(false);
                if moved {
                    if let Some(t) = target {
                        self.drop_in_tree(dragged, t);
                    }
                }
                self.tree_drag = None;
            }
        }
    }

    // ------------------------------------------------------------- context toolbar
    pub fn context_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("ctxbar")
            .exact_size(40.0)
            .frame(egui::Frame::new().fill(PANEL()).inner_margin(Margin::symmetric(10, 4)).stroke(Stroke::new(1.0, BORDER())))
            .show(ui, |ui| {
                egui::ScrollArea::horizontal().scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden).show(ui, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.spacing_mut().item_spacing.x = 5.0;
                        if self.preview {
                            let (r, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                            crate::icons::paint_or(ui, "play", r, theme::OK(), ">");
                            ui.label(RichText::new(crate::i18n::t("Modo prueba")).strong().color(theme::OK()));
                            ui.label(RichText::new(crate::i18n::t("Los cambios que hace el código se descartan al salir.")).color(MUTED()));
                            if ui.button(crate::i18n::t("Limpiar consola")).clicked() {
                                self.console.clear();
                            }
                            if ui.add(crate::icons::btn("stop", crate::i18n::t("Volver a editar"))).clicked() {
                                self.set_preview(false);
                            }
                            return;
                        }
                        if self.sel.is_empty() {
                            ui.label(crate::i18n::t("Fondo"));
                            rgba_edit(ui, &mut self.doc.background);
                            ui.separator();
                            ui.label(RichText::new(crate::i18n::t("Clic selecciona · Ctrl+clic entra en grupos · doble clic edita texto · Espacio+arrastrar mueve la vista")).color(MUTED()));
                            return;
                        }
                        let first_id = self.sel[0];
                        let Some(orig) = self.doc.get(first_id).cloned() else { return };
                        let mut p = orig.clone();
                        let mut add_font_clicked = false;
                        let k = p.kind;
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                        crate::icons::paint_or(ui, crate::icons::kind_name(k), r, MUTED(), k.icon());
                        ui.label(RichText::new(k.label()).color(MUTED()));
                        ui.separator();
                        if k.has_fill() {
                            rgba_edit(ui, &mut p.fill).on_hover_text(crate::i18n::t("Color (Image.color)"));
                            if matches!(k, Kind::Image | Kind::Slider) {
                                rgba_edit(ui, &mut p.fill2).on_hover_text(if k == Kind::Image { "Segundo color del degradado" } else { "Color de la pista" });
                            }
                            ui.separator();
                            ui.label(crate::i18n::t("Borde"));
                            rgba_edit(ui, &mut p.stroke);
                            ui.add(egui::DragValue::new(&mut p.stroke_w).range(0.0..=40.0).speed(0.2).suffix(" px"));
                            if !matches!(k, Kind::Ellipse | Kind::Slider | Kind::Toggle) {
                                ui.label(crate::i18n::t("Esquinas"));
                                ui.add(egui::DragValue::new(&mut p.radius).range(0.0..=500.0).speed(0.5));
                            }
                        }
                        if k.has_text() {
                            ui.separator();
                            if ui.small_button(crate::i18n::t("−")).clicked() {
                                p.font_size = (p.font_size - 2.0).max(4.0);
                            }
                            ui.add(egui::DragValue::new(&mut p.font_size).range(4.0..=400.0).speed(0.5)).on_hover_text(crate::i18n::t("Tamaño de letra"));
                            if ui.small_button(crate::i18n::t("+")).clicked() {
                                p.font_size += 2.0;
                            }
                            rgba_edit(ui, &mut p.text_color).on_hover_text(crate::i18n::t("Color del texto"));
                            ui.toggle_value(&mut p.bold, RichText::new(crate::i18n::t("B")).strong()).on_hover_text(crate::i18n::t("Negrita"));
                            ui.toggle_value(&mut p.italic, RichText::new(crate::i18n::t("I")).italics()).on_hover_text(crate::i18n::t("Cursiva"));
                            ui.toggle_value(&mut p.underline, RichText::new(crate::i18n::t("U")).underline()).on_hover_text(crate::i18n::t("Subrayado"));
                            let font_label = if p.font.is_empty() { "Predeterminada".to_string() } else { p.font.clone() };
                            egui::ComboBox::from_id_salt("font_pick")
                                .selected_text(font_label)
                                .width(120.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut p.font, String::new(), "Predeterminada");
                                    ui.selectable_value(&mut p.font, "Monoespaciada".to_string(), RichText::new(crate::i18n::t("Monoespaciada")).monospace());
                                    for f in render::available_fonts() {
                                        let fam = egui::FontFamily::Name(f.as_str().into());
                                        ui.selectable_value(&mut p.font, f.clone(), RichText::new(f).family(fam));
                                    }
                                    ui.separator();
                                    if ui.button(crate::i18n::t("+ Añadir fuente…")).clicked() {
                                        add_font_clicked = true;
                                    }
                                })
                                .response
                                .on_hover_text(crate::i18n::t("Fuente (se exporta como TMP Font Asset)"));
                            ui.separator();
                            for (i, (icon, tip)) in [("align_left", "Izquierda"), ("align_center", "Centro"), ("align_right", "Derecha")].iter().enumerate() {
                                if crate::icons::toggle(ui, p.align == i as u8, icon, &tip[..1], tip).clicked() {
                                    p.align = i as u8;
                                }
                            }
                            if k != Kind::Text {
                                ui.add_space(3.0);
                                for (i, (icon, tip)) in [("align_top", "Arriba"), ("align_middle", "Medio"), ("align_bottom", "Abajo")].iter().enumerate() {
                                    if crate::icons::toggle(ui, p.valign == i as u8, icon, &tip[..1], tip).clicked() {
                                        p.valign = i as u8;
                                    }
                                }
                            }
                        }
                        ui.separator();
                        let mut pct = p.opacity * 100.0;
                        if ui.add(egui::DragValue::new(&mut pct).range(0.0..=100.0).speed(0.5).prefix("◐ ").suffix("%").max_decimals(0)).on_hover_text(crate::i18n::t("Opacidad")).changed() {
                            p.opacity = pct / 100.0;
                        }
                        ui.separator();
                        if ui.selectable_label(self.inspector_open, crate::i18n::t("Inspector")).clicked() {
                            self.inspector_open = !self.inspector_open;
                        }
                        if p != orig {
                            self.apply_style(&orig, &p);
                        }
                        if add_font_clicked {
                            let ctx = ui.ctx().clone();
                            self.add_fonts(&ctx);
                        }
                    });
                });
            });
    }

    /// Apply the style fields that changed on the first selected element to the whole selection.
    fn apply_style(&mut self, old: &Element, new: &Element) {
        let sel = self.sel.clone();
        for e in self.doc.elements.iter_mut().filter(|e| sel.contains(&e.id)) {
            macro_rules! sync {
                ($($f:ident),*) => { $( if old.$f != new.$f { e.$f = new.$f.clone(); } )* };
            }
            sync!(fill, fill2, stroke, stroke_w, radius, opacity, font_size, text_color, bold, italic, underline, align, valign, font, value);
        }
    }

    // ------------------------------------------------------------- inspector
    pub fn inspector(&mut self, ui: &mut egui::Ui) {
        if !self.inspector_open || self.preview {
            return;
        }
        egui::Panel::right("inspector")
            .default_size(330.0)
            .size_range(280.0..=520.0)
            .resizable(true)
            .frame(egui::Frame::new().fill(PANEL()).inner_margin(Margin::same(10)).stroke(Stroke::new(1.0, BORDER())))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(crate::i18n::t("Inspector")).strong().size(14.0));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if crate::icons::button(ui, "close", "x", "Cerrar el inspector").clicked() {
                            self.inspector_open = false;
                        }
                    });
                });
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    if self.sel.is_empty() {
                        self.page_settings(ui);
                    } else if self.sel.len() > 1 {
                        self.multi_settings(ui);
                    } else {
                        let id = self.sel[0];
                        self.element_settings(ui, id);
                    }
                });
            });
    }

    fn page_settings(&mut self, ui: &mut egui::Ui) {
        component(ui, "canvas", &crate::i18n::t("Canvas · Canvas Scaler"), true, |ui| {
            hint(ui, &crate::i18n::t("UI Scale Mode: Scale With Screen Size"));
            egui::Grid::new("page").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                let mut w = self.doc.width;
                let mut h = self.doc.height;
                ui.label(crate::i18n::t("Referencia X"));
                let rw = ui.add(egui::DragValue::new(&mut w).range(64.0..=8192.0).suffix(" px"));
                ui.end_row();
                ui.label(crate::i18n::t("Referencia Y"));
                let rh = ui.add(egui::DragValue::new(&mut h).range(64.0..=8192.0).suffix(" px"));
                ui.end_row();
                if (rw.changed() || rh.changed()) && (w != self.doc.width || h != self.doc.height) {
                    self.resize_page(w.round(), h.round());
                }
                ui.label(crate::i18n::t("Match"));
                ui.add(egui::Slider::new(&mut self.doc.canvas_match, 0.0..=1.0).text(crate::i18n::t("Ancho / Alto")));
                ui.end_row();
                ui.label(crate::i18n::t("Fondo"));
                rgba_edit(ui, &mut self.doc.background);
                ui.end_row();
                ui.label(crate::i18n::t("Pixel art"));
                ui.checkbox(&mut self.doc.pixel_art, crate::i18n::t("Sin suavizado, píxeles enteros"))
                    .on_hover_text(crate::i18n::t("Imágenes con filtro de vecino más cercano, posiciones y tamaños en píxeles enteros. En Unity: Canvas Pixel Perfect y sprites con filtro Point sin compresión."));
                ui.end_row();
            });
            let mut preset = None;
            egui::ComboBox::from_id_salt("res_presets").selected_text(crate::i18n::t("Resoluciones comunes…")).width(ui.available_width() - 8.0).show_ui(ui, |ui| {
                for (name, w, h) in SIZE_PRESETS {
                    if ui.button(format!("{}  ·  {} × {}", name, w, h)).clicked() {
                        preset = Some((*w, *h));
                    }
                }
            });
            if let Some((w, h)) = preset {
                self.resize_page(w, h);
            }
            swatch_row(ui, &mut self.doc.background);
        });
        component(ui, "unity_export", &crate::i18n::t("Proyecto de Unity"), true, |ui| {
            if self.settings.unity_project.is_empty() {
                hint(ui, &crate::i18n::t("Aún no elegiste un proyecto. «Exportar a Unity» te pedirá la carpeta (la que contiene Assets)."));
            } else {
                ui.label(RichText::new(&self.settings.unity_project).small().monospace());
            }
            ui.horizontal(|ui| {
                if ui.add(crate::icons::btn("export", crate::i18n::t("Elegir proyecto…"))).clicked() {
                    self.export_to_unity(true);
                }
                if ui.add(crate::icons::btn("code", crate::i18n::t("Ver código…"))).clicked() {
                    self.export_open = true;
                }
            });
        });
        component(ui, "shortcuts", &crate::i18n::t("Atajos"), false, |ui| {
            for (k, d) in [
                ("Clic / Shift+clic", "Seleccionar / sumar"),
                ("Ctrl+clic", "Seleccionar dentro de un grupo"),
                ("Esc", "Seleccionar el padre"),
                ("Arrastrar vacío", "Selección por área"),
                ("Doble clic", "Editar texto"),
                ("Esquinas", "Escalar (Shift: libre)"),
                ("Tirador de giro", "Girar (Shift: 15°)"),
                ("Alt+arrastrar", "Duplicar y mover"),
                ("Ctrl+G / Ctrl+Shift+G", "Agrupar / desagrupar"),
                ("Ctrl+D / Supr", "Duplicar / borrar"),
                ("Ctrl+Z / Ctrl+Y", "Deshacer / rehacer"),
                ("Flechas", "Mover 1 px (Shift: 10)"),
                ("Ctrl+rueda", "Zoom"),
                ("Ctrl+S", "Guardar proyecto"),
                ("Ctrl+E", "Exportar a Unity"),
                ("F5", "Probar / simular pantallas"),
                ("F1", "Tutorial"),
            ] {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(k).monospace().size(11.0));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| ui.label(RichText::new(d).size(11.0).color(MUTED())));
                });
            }
        });
    }

    fn arrange(&mut self, ui: &mut egui::Ui, multi: bool) {
        component(ui, "arrange", if multi { "Alinear y ordenar" } else { "Alinear al padre y ordenar" }, true, |ui| {
            ui.columns(3, |c| {
                let b = |ui: &mut egui::Ui, t: &str| ui.add_sized([ui.available_width(), 22.0], egui::Button::new(t)).clicked();
                if b(&mut c[0], "Izquierda") {
                    self.align(0, 0)
                }
                if b(&mut c[1], "Centro") {
                    self.align(0, 1)
                }
                if b(&mut c[2], "Derecha") {
                    self.align(0, 2)
                }
                if b(&mut c[0], "Arriba") {
                    self.align(1, 0)
                }
                if b(&mut c[1], "Medio") {
                    self.align(1, 1)
                }
                if b(&mut c[2], "Abajo") {
                    self.align(1, 2)
                }
            });
            if self.sel.len() >= 3 {
                ui.columns(2, |c| {
                    if c[0].button(crate::i18n::t("Distribuir horiz.")).clicked() {
                        self.distribute(0);
                    }
                    if c[1].button(crate::i18n::t("Distribuir vert.")).clicked() {
                        self.distribute(1);
                    }
                });
            }
            ui.columns(4, |c| {
                if c[0].button(crate::i18n::t("Atrás")).on_hover_text(crate::i18n::t("Un lugar antes entre hermanos (Ctrl+[)")).clicked() {
                    self.reorder(None, -1);
                }
                if c[1].button(crate::i18n::t("Adelante")).on_hover_text(crate::i18n::t("Ctrl+]")).clicked() {
                    self.reorder(None, 1);
                }
                if c[2].button(crate::i18n::t("Al fondo")).clicked() {
                    self.reorder(Some(false), 0);
                }
                if c[3].button(crate::i18n::t("Al frente")).clicked() {
                    self.reorder(Some(true), 0);
                }
            });
        });
    }

    fn multi_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(format!("{} {}", self.sel.len(), crate::i18n::t("elementos seleccionados"))).color(MUTED()));
        self.arrange(ui, true);
        ui.add_space(6.0);
        if ui.button(crate::i18n::t("Agrupar en Contenedor (Ctrl+G)")).clicked() {
            self.group_selection();
        }
    }

    fn element_settings(&mut self, ui: &mut egui::Ui, id: u64) {
        let Some(mut e) = self.doc.get(id).cloned() else { return };
        let before = e.clone();
        let names = crate::export::unity_names(&self.doc);
        let path = crate::export::unity_path(&self.doc, &names, id);
        let pf = self.doc.parent_frame(id);
        let rt_before = rect_transform_in(&e, &pf);
        let mut rt = rt_before;
        let mut world = e.frame();
        let world_before = world;
        let driven = self.doc.driven_by_layout(id);
        let parent_name = self.doc.parent_of(id).and_then(|p| self.doc.get(p)).map(|p| p.name.clone());

        // header (GameObject)
        ui.horizontal(|ui| {
            if crate::icons::toggle(ui, !e.visible, if e.visible { "eye" } else { "eye_off" }, "o", "Activo / inactivo (GameObject.SetActive)").clicked() {
                e.visible = !e.visible;
            }
            ui.add(egui::TextEdit::singleline(&mut e.name).desired_width(ui.available_width() - 30.0));
            if crate::icons::toggle(ui, e.locked, if e.locked { "lock" } else { "unlock" }, "L", "Bloquear en el editor").clicked() {
                e.locked = !e.locked;
            }
        });
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
            crate::icons::paint_or(ui, crate::icons::kind_name(e.kind), r, ACCENT(), e.kind.icon());
            egui::ComboBox::from_id_salt("kind").selected_text(e.kind.label()).width(150.0).show_ui(ui, |ui| {
                for k in model::ALL_KINDS {
                    ui.selectable_value(&mut e.kind, k, k.label());
                }
            });
            ui.label(RichText::new(e.kind.unity_component()).small().color(MUTED()));
        });
        ui.label(RichText::new(&path).small().monospace().color(MUTED())).on_hover_text(crate::i18n::t("Ruta dentro del Canvas en Unity"));

        // RectTransform
        let mut preset_click: Option<([f32; 2], [f32; 2], bool, bool)> = None;
        component(ui, "rt", &crate::i18n::t("Rect Transform"), true, |ui| {
            if driven {
                egui::Frame::new().fill(ACCENT_SOFT()).corner_radius(4.0).inner_margin(Margin::same(5)).show(ui, |ui| {
                    ui.label(RichText::new(crate::i18n::tf("Algunos valores los controla el Layout Group de «{}».", &[&parent_name.clone().unwrap_or_default()])).small());
                });
            }
            ui.horizontal(|ui| {
                preset_click = anchor_presets(ui, &e);
                ui.vertical(|ui| {
                    hint(ui, &crate::i18n::t("Clic: anclas"));
                    hint(ui, &crate::i18n::t("Shift: + pivote"));
                    hint(ui, &crate::i18n::t("Alt: + posición"));
                    ui.label(RichText::new(crate::i18n::tf("Padre: {}", &[&parent_name.clone().unwrap_or_else(|| "Canvas".into())])).small().color(MUTED()));
                });
            });
            ui.add_space(4.0);
            let sx = rt.stretch_x();
            let sy = rt.stretch_y();
            let mut omin = rt.offset_min();
            let mut omax = rt.offset_max();
            let mut changed_off = false;
            egui::Grid::new("rtgrid").num_columns(4).spacing([6.0, 4.0]).show(ui, |ui| {
                if sx {
                    let mut l = omin[0];
                    let mut r = -omax[0];
                    if drag_f(ui, "Left", &mut l, 1.0) {
                        omin[0] = l;
                        changed_off = true;
                    }
                    if drag_f(ui, "Right", &mut r, 1.0) {
                        omax[0] = -r;
                        changed_off = true;
                    }
                } else {
                    drag_f(ui, "Pos X", &mut rt.anchored_position[0], 1.0);
                    drag_f(ui, "Width", &mut rt.size_delta[0], 1.0);
                }
                ui.end_row();
                if sy {
                    let mut t = -omax[1];
                    let mut b = omin[1];
                    if drag_f(ui, "Top", &mut t, 1.0) {
                        omax[1] = -t;
                        changed_off = true;
                    }
                    if drag_f(ui, "Bottom", &mut b, 1.0) {
                        omin[1] = b;
                        changed_off = true;
                    }
                } else {
                    drag_f(ui, "Pos Y", &mut rt.anchored_position[1], 1.0);
                    drag_f(ui, "Height", &mut rt.size_delta[1], 1.0);
                }
                ui.end_row();
                if sx && !sy {
                    // keep Width editable for the non-stretched axis only
                }
            });
            if changed_off {
                rt.set_offsets(omin, omax);
            }
            egui::Grid::new("rtanch").num_columns(3).spacing([6.0, 4.0]).show(ui, |ui| {
                ui.label(RichText::new(crate::i18n::t("Anchor Min")).size(11.5).color(MUTED()));
                ui.add(egui::DragValue::new(&mut e.anchor_min[0]).range(0.0..=1.0).speed(0.01).max_decimals(3));
                ui.add(egui::DragValue::new(&mut e.anchor_min[1]).range(0.0..=1.0).speed(0.01).max_decimals(3));
                ui.end_row();
                ui.label(RichText::new(crate::i18n::t("Anchor Max")).size(11.5).color(MUTED()));
                ui.add(egui::DragValue::new(&mut e.anchor_max[0]).range(0.0..=1.0).speed(0.01).max_decimals(3));
                ui.add(egui::DragValue::new(&mut e.anchor_max[1]).range(0.0..=1.0).speed(0.01).max_decimals(3));
                ui.end_row();
                ui.label(RichText::new(crate::i18n::t("Pivot")).size(11.5).color(MUTED()));
                ui.add(egui::DragValue::new(&mut e.pivot[0]).range(0.0..=1.0).speed(0.01).max_decimals(3));
                ui.add(egui::DragValue::new(&mut e.pivot[1]).range(0.0..=1.0).speed(0.01).max_decimals(3));
                ui.end_row();
                ui.label(RichText::new(crate::i18n::t("Rotation Z")).size(11.5).color(MUTED()));
                ui.add(egui::DragValue::new(&mut rt.rotation_z).speed(0.5).suffix("°"));
                ui.end_row();
            });
            egui::CollapsingHeader::new(RichText::new(crate::i18n::t("Medidas en el lienzo (px)")).small()).id_salt("world").show(ui, |ui| {
                egui::Grid::new("worldgrid").num_columns(4).spacing([6.0, 4.0]).show(ui, |ui| {
                    let mut x = world.cx - world.w / 2.0;
                    let mut y = world.cy - world.h / 2.0;
                    if drag_f(ui, "X", &mut x, 1.0) {
                        world.cx = x + world.w / 2.0;
                    }
                    if drag_f(ui, "Y", &mut y, 1.0) {
                        world.cy = y + world.h / 2.0;
                    }
                    ui.end_row();
                    let (mut w, mut h) = (world.w, world.h);
                    if drag_f(ui, "Ancho", &mut w, 1.0) {
                        world.cx += (w.max(1.0) - world.w) / 2.0;
                        world.w = w.max(1.0);
                    }
                    if drag_f(ui, "Alto", &mut h, 1.0) {
                        world.cy += (h.max(1.0) - world.h) / 2.0;
                        world.h = h.max(1.0);
                    }
                    ui.end_row();
                    drag_f(ui, "Girar", &mut world.rot, 0.5);
                    ui.end_row();
                });
            });
        });

        // Image
        let mut pick_image = false;
        if e.kind.has_fill() && !matches!(e.kind, Kind::Toggle | Kind::Slider) {
            component(ui, "img", &crate::i18n::t("Image"), true, |ui| {
                egui::Grid::new("imggrid").num_columns(2).spacing([8.0, 5.0]).show(ui, |ui| {
                    ui.label(crate::i18n::t("Color"));
                    rgba_edit(ui, &mut e.fill);
                    ui.end_row();
                    if e.kind == Kind::Image && e.image.is_empty() {
                        ui.label(crate::i18n::t("Degradado hasta"));
                        rgba_edit(ui, &mut e.fill2);
                        ui.end_row();
                    }
                    if e.kind == Kind::Image {
                        ui.label(crate::i18n::t("Sprite"));
                        ui.horizontal(|ui| {
                            if ui.button(crate::i18n::t(if e.image.is_empty() { "Elegir imagen…" } else { "Cambiar…" })).clicked() {
                                pick_image = true;
                            }
                            if !e.image.is_empty() && ui.small_button(crate::i18n::t("Quitar")).clicked() {
                                e.image.clear();
                            }
                        });
                        ui.end_row();
                    }
                    if e.kind != Kind::Ellipse {
                        ui.label(crate::i18n::t("Radio de esquinas"));
                        ui.add(egui::DragValue::new(&mut e.radius).range(0.0..=500.0).speed(0.5).suffix(" px")).on_hover_text(crate::i18n::t("Se exporta como Sprite 9-slice generado"));
                        ui.end_row();
                    }
                    ui.label(crate::i18n::t("Borde"));
                    ui.horizontal(|ui| {
                        rgba_edit(ui, &mut e.stroke);
                        ui.add(egui::DragValue::new(&mut e.stroke_w).range(0.0..=60.0).speed(0.2).suffix(" px"));
                    });
                    ui.end_row();
                    ui.label(crate::i18n::t("Opacidad"));
                    ui.add(egui::Slider::new(&mut e.opacity, 0.0..=1.0));
                    ui.end_row();
                    ui.label(crate::i18n::t("Raycast Target"));
                    ui.checkbox(&mut e.raycast, crate::i18n::t(""));
                    ui.end_row();
                });
                swatch_row(ui, &mut e.fill);
            });
        }

        // Text
        if e.kind.has_text() {
            let title = if e.kind == Kind::Dropdown { "TMP_Dropdown · opciones" } else { "TextMeshPro · texto" };
            component(ui, "txt", title, true, |ui| {
                if e.kind == Kind::Dropdown {
                    let mut opts = e.options.join("\n");
                    hint(ui, &crate::i18n::t("Una opción por línea"));
                    if ui.add(egui::TextEdit::multiline(&mut opts).desired_rows(3).desired_width(f32::INFINITY)).changed() {
                        e.options = opts.lines().map(|l| l.to_string()).filter(|l| !l.trim().is_empty()).collect();
                    }
                    if !e.options.is_empty() {
                        let mut idx = (e.value.round().max(0.0) as usize).min(e.options.len() - 1);
                        egui::ComboBox::from_id_salt("dd_value").selected_text(e.options[idx].clone()).show_ui(ui, |ui| {
                            for (i, o) in e.options.iter().enumerate() {
                                ui.selectable_value(&mut idx, i, o);
                            }
                        });
                        e.value = idx as f32;
                    }
                } else {
                    if e.kind == Kind::InputField {
                        hint(ui, &crate::i18n::t("Texto de ejemplo (Placeholder)"));
                    }
                    ui.add(egui::TextEdit::multiline(&mut e.text).desired_rows(2).desired_width(f32::INFINITY));
                }
                egui::Grid::new("txtgrid").num_columns(2).spacing([8.0, 5.0]).show(ui, |ui| {
                    ui.label(crate::i18n::t("Tamaño"));
                    ui.add(egui::DragValue::new(&mut e.font_size).range(4.0..=400.0).speed(0.5));
                    ui.end_row();
                    ui.label(crate::i18n::t("Color"));
                    rgba_edit(ui, &mut e.text_color);
                    ui.end_row();
                    ui.label(crate::i18n::t("Estilo"));
                    ui.horizontal(|ui| {
                        ui.toggle_value(&mut e.bold, RichText::new(crate::i18n::t("B")).strong());
                        ui.toggle_value(&mut e.italic, RichText::new(crate::i18n::t("I")).italics());
                        ui.toggle_value(&mut e.underline, RichText::new(crate::i18n::t("U")).underline());
                    });
                    ui.end_row();
                    ui.label(crate::i18n::t("Alineación"));
                    ui.horizontal(|ui| {
                        for (i, t) in ["align_left", "align_center", "align_right"].iter().enumerate() {
                            if crate::icons::toggle(ui, e.align == i as u8, t, "-", ["Izquierda", "Centro", "Derecha"][i]).clicked() {
                                e.align = i as u8;
                            }
                        }
                        if e.kind != Kind::Text {
                            ui.add_space(4.0);
                            for (i, t) in ["align_top", "align_middle", "align_bottom"].iter().enumerate() {
                                if crate::icons::toggle(ui, e.valign == i as u8, t, "|", ["Arriba", "Medio", "Abajo"][i]).clicked() {
                                    e.valign = i as u8;
                                }
                            }
                        }
                    });
                    ui.end_row();
                    ui.label(crate::i18n::t("Fuente"));
                    let font_label = if e.font.is_empty() { "Predeterminada".to_string() } else { e.font.clone() };
                    egui::ComboBox::from_id_salt("font_pick_insp").selected_text(font_label).width(150.0).show_ui(ui, |ui| {
                        ui.selectable_value(&mut e.font, String::new(), "Predeterminada");
                        for f in render::available_fonts() {
                            let fam = egui::FontFamily::Name(f.as_str().into());
                            ui.selectable_value(&mut e.font, f.clone(), RichText::new(f).family(fam));
                        }
                    });
                    ui.end_row();
                });
            });
        }

        // Controls
        match e.kind {
            Kind::Toggle => component(ui, "tog", &crate::i18n::t("Toggle"), true, |ui| {
                let mut on = e.value > 0.5;
                if ui.checkbox(&mut on, crate::i18n::t("Is On")).changed() {
                    e.value = if on { 1.0 } else { 0.0 };
                }
                ui.horizontal(|ui| {
                    ui.label(crate::i18n::t("Color de la casilla"));
                    rgba_edit(ui, &mut e.fill);
                });
                ui.checkbox(&mut e.interactable, crate::i18n::t("Interactable"));
            }),
            Kind::Slider => component(ui, "sld", &crate::i18n::t("Slider"), true, |ui| {
                ui.add(egui::Slider::new(&mut e.value, 0.0..=1.0).text(crate::i18n::t("Value")));
                ui.horizontal(|ui| {
                    ui.label(crate::i18n::t("Relleno"));
                    rgba_edit(ui, &mut e.fill);
                    ui.label(crate::i18n::t("Pista"));
                    rgba_edit(ui, &mut e.fill2);
                });
                ui.checkbox(&mut e.interactable, crate::i18n::t("Interactable")).on_hover_text(crate::i18n::t("Desactívalo para barras de vida o progreso"));
            }),
            k if k.selectable() => component(ui, "sel", &crate::i18n::t("Selectable"), true, |ui| {
                ui.checkbox(&mut e.interactable, crate::i18n::t("Interactable"));
            }),
            Kind::ScrollView => component(ui, "scroll", &crate::i18n::t("Scroll Rect"), true, |ui| {
                hint(ui, &crate::i18n::t("Desplazamiento vertical. Los hijos van al Content (Viewport/Content); su alto crece para que todos quepan."));
            }),
            _ => {}
        }

        // Mask + Shadow
        component(ui, "effects", &crate::i18n::t("Máscara y efectos"), e.mask || e.shadow.on, |ui| {
            if e.kind != Kind::ScrollView {
                ui.checkbox(&mut e.mask, crate::i18n::t("Rect Mask 2D (recorta a los hijos)"));
            }
            if e.kind.has_fill() {
                ui.checkbox(&mut e.shadow.on, crate::i18n::t("Shadow"));
                if e.shadow.on {
                    ui.horizontal(|ui| {
                        rgba_edit(ui, &mut e.shadow.color);
                        ui.label(crate::i18n::t("Distancia"));
                        ui.add(egui::DragValue::new(&mut e.shadow.offset[0]).speed(0.5).prefix("x "));
                        ui.add(egui::DragValue::new(&mut e.shadow.offset[1]).speed(0.5).prefix("y "));
                    });
                }
            }
        });

        // Layout group
        component(ui, "layout", &crate::i18n::t("Layout Group"), e.layout.mode != LayoutMode::None, |ui| {
            egui::ComboBox::from_id_salt("lmode").selected_text(e.layout.mode.label()).width(200.0).show_ui(ui, |ui| {
                for m in [LayoutMode::None, LayoutMode::Horizontal, LayoutMode::Vertical, LayoutMode::Grid] {
                    ui.selectable_value(&mut e.layout.mode, m, m.label());
                }
            });
            if e.layout.mode == LayoutMode::None {
                hint(ui, &crate::i18n::t("Organiza automáticamente a los hijos directos, como en Unity."));
                return;
            }
            egui::Grid::new("lgrid").num_columns(4).spacing([6.0, 4.0]).show(ui, |ui| {
                drag_f(ui, "Left", &mut e.layout.padding[0], 0.5);
                drag_f(ui, "Right", &mut e.layout.padding[1], 0.5);
                ui.end_row();
                drag_f(ui, "Top", &mut e.layout.padding[2], 0.5);
                drag_f(ui, "Bottom", &mut e.layout.padding[3], 0.5);
                ui.end_row();
                match e.layout.mode {
                    LayoutMode::Horizontal => {
                        drag_f(ui, "Spacing", &mut e.layout.spacing[0], 0.5);
                    }
                    LayoutMode::Vertical => {
                        drag_f(ui, "Spacing", &mut e.layout.spacing[1], 0.5);
                    }
                    _ => {
                        drag_f(ui, "Spacing X", &mut e.layout.spacing[0], 0.5);
                        drag_f(ui, "Spacing Y", &mut e.layout.spacing[1], 0.5);
                        ui.end_row();
                        drag_f(ui, "Cell W", &mut e.layout.cell[0], 1.0);
                        drag_f(ui, "Cell H", &mut e.layout.cell[1], 1.0);
                    }
                }
                ui.end_row();
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new(crate::i18n::t("Child Alignment")).size(11.5).color(MUTED()));
                align_grid(ui, &mut e.layout.align);
            });
            if e.layout.mode != LayoutMode::Grid {
                ui.checkbox(&mut e.layout.expand_w, crate::i18n::t("Control + Expand ancho"));
                ui.checkbox(&mut e.layout.expand_h, crate::i18n::t("Control + Expand alto"));
            }
        });

        // click code
        let mut run_now = false;
        let mut go_test = false;
        component(ui, "code", &crate::i18n::t("Al hacer clic (código de prueba)"), !e.on_click.trim().is_empty(), |ui| {
            hint(ui, &crate::i18n::t("Se ejecuta en Probar. Al exportar queda como comentario en el método OnClick del script View."));
            ui.add(egui::TextEdit::multiline(&mut e.on_click).code_editor().desired_rows(4).desired_width(f32::INFINITY).hint_text(crate::i18n::t("mensaje(\"¡Hola!\");\nocultar(\"Ventana\");")));
            ui.horizontal(|ui| {
                if ui.add_enabled(!e.on_click.trim().is_empty(), crate::icons::btn("play", crate::i18n::t("Ejecutar ahora"))).clicked() {
                    run_now = true;
                }
                if ui.button(crate::i18n::t("Probar diseño")).on_hover_text(crate::i18n::t("F5")).clicked() {
                    go_test = true;
                }
            });
            egui::CollapsingHeader::new(RichText::new(crate::i18n::t("Funciones disponibles")).small()).id_salt("code_help").show(ui, |ui| {
                ui.label(RichText::new(crate::script::HELP).monospace().size(10.5));
            });
            if !self.console.is_empty() {
                egui::Frame::new().fill(Color32::from_rgb(20, 21, 27)).corner_radius(5.0).inner_margin(Margin::same(6)).show(ui, |ui| {
                    for l in self.console.iter().rev().take(4).collect::<Vec<_>>().into_iter().rev() {
                        let c = if l.starts_with('⚠') { Color32::from_rgb(255, 140, 120) } else { Color32::from_rgb(220, 240, 220) };
                        ui.label(RichText::new(l).monospace().size(11.0).color(c));
                    }
                });
            }
        });

        // ---- write back
        if e != before {
            // style / identity / anchors (the box itself stays where it is)
            let mut keep = e.clone();
            keep.set_frame(&before.frame());
            if let Some(el) = self.doc.get_mut(id) {
                *el = keep;
            }
        }
        if let Some((min, max, shift, alt)) = preset_click {
            let cur = self.doc.get(id).cloned();
            if let Some(mut c) = cur {
                c.anchor_min = min;
                c.anchor_max = max;
                if shift {
                    c.pivot = [if min[0] == max[0] { min[0] } else { 0.5 }, if min[1] == max[1] { min[1] } else { 0.5 }];
                }
                if let Some(el) = self.doc.get_mut(id) {
                    el.anchor_min = c.anchor_min;
                    el.anchor_max = c.anchor_max;
                    el.pivot = c.pivot;
                }
                if alt {
                    let mut r = rect_transform_in(&c, &pf);
                    r.anchored_position = [0.0, 0.0];
                    if r.stretch_x() {
                        r.size_delta[0] = 0.0;
                    }
                    if r.stretch_y() {
                        r.size_delta[1] = 0.0;
                    }
                    self.doc.set_rt(id, &r);
                }
            }
        } else if world != world_before {
            self.set_frame(id, world);
        } else if rt != rt_before {
            if let Some(cur) = self.doc.get(id) {
                let mut r = rt;
                r.anchor_min = cur.anchor_min;
                r.anchor_max = cur.anchor_max;
                r.pivot = cur.pivot;
                self.doc.set_rt(id, &r);
            }
        }
        if pick_image {
            let file = rfd::FileDialog::new().set_title("Elegir imagen").add_filter("Imágenes", crate::library::IMAGE_EXTS).pick_file();
            if let Some(f) = file {
                match self.library.add_image(&f) {
                    Ok(name) => {
                        if let Some(el) = self.doc.get_mut(id) {
                            el.image = name;
                        }
                        self.status = crate::i18n::t("Imagen cambiada");
                    }
                    Err(err) => self.status = crate::i18n::tf("No se pudo usar la imagen: {}", &[&err.to_string()]),
                }
            }
        }
        if run_now {
            self.run_click(id);
        }
        if go_test {
            self.set_preview(true);
        }
        self.arrange(ui, false);
        let _ = Frame::page(1.0, 1.0);
    }

    // ------------------------------------------------------------- status bar
    pub fn status_bar(&mut self, ui: &mut egui::Ui) {
        egui::Panel::bottom("status")
            .exact_size(28.0)
            .frame(egui::Frame::new().fill(PANEL_DARK()).inner_margin(Margin::symmetric(10, 2)).stroke(Stroke::new(1.0, BORDER())))
            .show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.small_button(crate::i18n::t("Ajustar")).on_hover_text(crate::i18n::t("Ctrl+0")).clicked() {
                            self.fit_pending = true;
                        }
                        ui.label(RichText::new(format!("{:.0}%", self.zoom * 100.0)).size(11.5));
                        let mut z = self.zoom;
                        if ui.add(egui::Slider::new(&mut z, 0.05..=4.0).logarithmic(true).show_value(false)).changed() {
                            self.zoom = z;
                        }
                        ui.separator();
                        ui.label(RichText::new(format!("{} {} · {} {}", self.doc.elements.len(), crate::i18n::t("elementos"), self.sel.len(), crate::i18n::t("seleccionados"))).size(11.5).color(MUTED()));
                        ui.separator();
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.add(egui::Label::new(RichText::new(crate::i18n::t(&self.status)).size(11.5).color(MUTED())).truncate());
                        });
                    });
                });
            });
    }

    // ------------------------------------------------------------- save window
    pub fn save_window(&mut self, ctx: &egui::Context) {
        if !self.save_open {
            return;
        }
        let mut open = true;
        let mut close = false;
        egui::Window::new(crate::i18n::t("Guardar")).open(&mut open).collapsible(false).resizable(false).default_width(380.0).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            ui.label(RichText::new(crate::i18n::t("Como proyecto")).strong());
            hint(ui, &crate::i18n::t("Un archivo que puedes volver a abrir y seguir editando."));
            ui.horizontal(|ui| {
                if ui.add(crate::icons::btn("save", crate::i18n::t("Guardar proyecto"))).on_hover_text(&self.project_path).clicked() {
                    if std::path::Path::new(&self.project_path).is_absolute() { self.save_project() } else { self.save_project_as() }
                    close = true;
                }
                if ui.button(crate::i18n::t("Guardar como…")).clicked() {
                    self.save_project_as();
                    close = true;
                }
            });
            ui.separator();
            ui.label(RichText::new(crate::i18n::t("Como plantilla")).strong());
            hint(ui, &crate::i18n::t("Aparecerá en Plantillas › Mis plantillas para empezar nuevos diseños."));
            if self.template_name.is_empty() {
                self.template_name = self.doc.title.clone();
            }
            ui.horizontal(|ui| {
                ui.label(crate::i18n::t("Nombre"));
                ui.add(egui::TextEdit::singleline(&mut self.template_name).desired_width(200.0));
            });
            if ui.add(crate::icons::btn("tab_templates", crate::i18n::t("Guardar como plantilla"))).clicked() {
                let n = std::mem::take(&mut self.template_name);
                self.save_as_template(&n);
                self.tab = Tab::Templates;
                self.drawer_open = true;
                close = true;
            }
        });
        if !open || close {
            self.save_open = false;
        }
    }

    // ------------------------------------------------------------- export window
    pub fn export_window(&mut self, ctx: &egui::Context) {
        if !self.export_open {
            return;
        }
        let mut open = true;
        let cls = crate::export::class_name(&self.doc.title);
        egui::Window::new(crate::i18n::t("Exportar a Unity")).open(&mut open).default_size([760.0, 600.0]).collapsible(false).show(ctx, |ui| {
            egui::Frame::new().fill(PANEL_DARK()).corner_radius(6.0).inner_margin(Margin::same(8)).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(crate::i18n::t("Proyecto:")).strong());
                    let shown = if self.settings.unity_project.is_empty() { "(sin elegir)".to_string() } else { self.settings.unity_project.clone() };
                    ui.label(RichText::new(shown).monospace().size(11.5));
                });
                ui.horizontal(|ui| {
                    if ui.add(crate::icons::btn_tint("export", RichText::new(crate::i18n::t("Exportar todo al proyecto")).strong().color(Color32::WHITE), Color32::WHITE).fill(ACCENT())).clicked() {
                        self.export_to_unity(false);
                    }
                    if ui.button(crate::i18n::t("Cambiar proyecto…")).clicked() {
                        self.export_to_unity(true);
                    }
                });
                hint(
                    ui,
                    &format!(
                        "Escribe en Assets/EditorGUI: Editor/EditorGuiImporter.cs, Designs/{c}.json, Sprites, Fonts, UI Toolkit/{c}.uxml+.uss y Scripts/{c}OnGUI.cs, {c}ViewBase.cs, {c}View.cs. Al volver a Unity se genera Prefabs/{c}.prefab (uGUI + TextMeshPro) automáticamente.",
                        c = cls
                    ),
                );
            });
            ui.add_space(6.0);
            let tabs = ["uGUI (JSON)", "UI Toolkit UXML", "UI Toolkit USS", "IMGUI OnGUI", "Script View", "Importador C#"];
            ui.horizontal_wrapped(|ui| {
                for (i, t) in tabs.iter().enumerate() {
                    ui.selectable_value(&mut self.export_tab, i as u8, *t);
                }
            });
            ui.separator();
            let (text, file, ext) = match self.export_tab {
                0 => (crate::export::ugui::json(&self.doc, false), format!("{}.json", cls), "json"),
                1 => (crate::export::uitk::uxml(&self.doc), format!("{}.uxml", cls), "uxml"),
                2 => (crate::export::uitk::uss(&self.doc), format!("{}.uss", cls), "uss"),
                3 => (crate::export::imgui::script(&self.doc), format!("{}OnGUI.cs", cls), "cs"),
                4 => (crate::export::view::base_script(&self.doc), format!("{}ViewBase.cs", cls), "cs"),
                _ => (crate::export::IMPORTER_CS.to_string(), "EditorGuiImporter.cs".to_string(), "cs"),
            };
            let help = match self.export_tab {
                0 => "Diseño para el importador. Unity lo convierte en prefab (Tools › Lienzo UI).",
                1 | 2 => "Usa un UIDocument con PanelSettings en «Scale With Screen Size» y esta resolución de referencia.",
                3 => "Componente autónomo: dibuja la interfaz con UnityEngine.GUI dentro de OnGUI (sin Canvas).",
                4 => "Referencias tipadas y eventos onClick/onValueChanged. Hereda en <Clase>View para tu lógica.",
                _ => "Va en Assets/EditorGUI/Editor/. Construye el prefab con uGUI + TextMeshPro.",
            };
            hint(ui, help);
            ui.horizontal(|ui| {
                if ui.add(crate::icons::btn("duplicate", crate::i18n::t("Copiar"))).clicked() {
                    ui.ctx().copy_text(text.clone());
                    self.status = crate::i18n::t("Copiado al portapapeles");
                }
                if ui.add(crate::icons::btn("save", crate::i18n::t("Guardar archivo…"))).clicked() {
                    self.save_text_file("Guardar archivo", &file, ext, &text);
                }
                ui.label(RichText::new(format!("{} líneas", text.lines().count())).small().color(MUTED()));
            });
            egui::ScrollArea::both().max_height(ui.available_height() - 6.0).auto_shrink([false, false]).show(ui, |ui| {
                let mut t = text;
                ui.add(egui::TextEdit::multiline(&mut t).code_editor().desired_width(f32::INFINITY));
            });
        });
        if !open {
            self.export_open = false;
        }
    }
}

/// 3×3 child alignment picker (Unity TextAnchor).
fn align_grid(ui: &mut egui::Ui, a: &mut u8) {
    egui::Grid::new("align9").spacing([2.0, 2.0]).show(ui, |ui| {
        for row in 0..3u8 {
            for col in 0..3u8 {
                let i = row * 3 + col;
                let (r, resp) = ui.allocate_exact_size(Vec2::splat(14.0), Sense::click());
                ui.painter().rect_filled(r, 2.0, if *a == i { ACCENT() } else if resp.hovered() { HOVER() } else { FIELD() });
                if resp.clicked() {
                    *a = i;
                }
            }
            ui.end_row();
        }
    });
}

/// Unity-like 4×4 anchor preset grid. Returns (min, max, shift, alt) when clicked.
fn anchor_presets(ui: &mut egui::Ui, e: &Element) -> Option<([f32; 2], [f32; 2], bool, bool)> {
    let xs: [(f32, f32); 4] = [(0.0, 0.0), (0.5, 0.5), (1.0, 1.0), (0.0, 1.0)];
    let ys: [(f32, f32); 4] = [(1.0, 1.0), (0.5, 0.5), (0.0, 0.0), (0.0, 1.0)];
    let cell = 28.0;
    let mut out = None;
    egui::Grid::new("anchors").spacing([3.0, 3.0]).show(ui, |ui| {
        for (iy, (ymin, ymax)) in ys.iter().enumerate() {
            for (ix, (xmin, xmax)) in xs.iter().enumerate() {
                let active = e.anchor_min == [*xmin, *ymin] && e.anchor_max == [*xmax, *ymax];
                let (r, resp) = ui.allocate_exact_size(Vec2::splat(cell), Sense::click());
                let bg = if active { ACCENT_SOFT() } else if resp.hovered() { HOVER() } else { FIELD() };
                ui.painter().rect_filled(r, CornerRadius::same(3), bg);
                ui.painter().rect_stroke(r, CornerRadius::same(3), Stroke::new(1.0, if active { ACCENT() } else { LINE() }), StrokeKind::Inside);
                let inner = r.shrink(7.0);
                ui.painter().rect_stroke(inner, 1.0, Stroke::new(1.0, MUTED().gamma_multiply(0.6)), StrokeKind::Inside);
                let red = Color32::from_rgb(235, 90, 90);
                let px = |t: f32| inner.left() + inner.width() * t;
                let py = |t: f32| inner.bottom() - inner.height() * t;
                if ix == 3 {
                    ui.painter().line_segment([Pos2::new(inner.left(), inner.center().y), Pos2::new(inner.right(), inner.center().y)], Stroke::new(1.5, red));
                }
                if iy == 3 {
                    ui.painter().line_segment([Pos2::new(inner.center().x, inner.top()), Pos2::new(inner.center().x, inner.bottom())], Stroke::new(1.5, red));
                }
                if ix != 3 && iy != 3 {
                    ui.painter().circle_filled(Pos2::new(px(*xmin), py(*ymin)), 2.5, red);
                }
                let resp = resp.on_hover_text(format!("min ({}, {}) · max ({}, {})", xmin, ymin, xmax, ymax));
                if resp.clicked() {
                    let m = ui.input(|i| i.modifiers);
                    out = Some(([*xmin, *ymin], [*xmax, *ymax], m.shift, m.alt));
                }
            }
            ui.end_row();
        }
    });
    out
}

fn rail_icon(p: &egui::Painter, t: Tab, r: Rect, c: Color32) {
    let s = Stroke::new(1.5, c);
    match t {
        Tab::Templates => {
            let a = Rect::from_min_size(r.min, Vec2::new(r.width() * 0.45, r.height()));
            let b = Rect::from_min_max(Pos2::new(r.left() + r.width() * 0.55, r.top()), Pos2::new(r.right(), r.center().y - 1.0));
            let d = Rect::from_min_max(Pos2::new(r.left() + r.width() * 0.55, r.center().y + 1.0), r.max);
            for q in [a, b, d] {
                p.rect_stroke(q, 3.0, s, StrokeKind::Inside);
            }
        }
        Tab::Elements => {
            p.circle_stroke(Pos2::new(r.left() + 5.5, r.top() + 5.5), 5.0, s);
            p.rect_stroke(Rect::from_min_size(Pos2::new(r.right() - 10.0, r.top() + 1.0), Vec2::splat(9.0)), 1.0, s, StrokeKind::Inside);
            let tri = vec![Pos2::new(r.left() + 6.0, r.bottom() - 10.0), Pos2::new(r.left() + 11.0, r.bottom() - 1.0), Pos2::new(r.left() + 1.0, r.bottom() - 1.0)];
            p.add(egui::Shape::closed_line(tri, s));
            p.rect_stroke(Rect::from_min_size(Pos2::new(r.right() - 10.0, r.bottom() - 10.0), Vec2::splat(9.0)), 4.5, s, StrokeKind::Inside);
        }
        Tab::Text => {
            p.text(r.center(), egui::Align2::CENTER_CENTER, "T", FontId::proportional(20.0), c);
        }
        Tab::Images => {
            p.rect_stroke(r, 3.0, s, StrokeKind::Inside);
            p.circle_filled(Pos2::new(r.left() + 7.0, r.top() + 7.0), 2.5, c);
            p.add(egui::Shape::line(vec![Pos2::new(r.left() + 2.0, r.bottom() - 3.0), Pos2::new(r.left() + 9.0, r.center().y + 1.0), Pos2::new(r.left() + 14.0, r.bottom() - 6.0), Pos2::new(r.right() - 2.0, r.center().y)], s));
        }
        Tab::Plugins => {
            // pieza de rompecabezas simplificada
            p.rect_stroke(Rect::from_min_max(Pos2::new(r.left() + 2.0, r.top() + 5.0), Pos2::new(r.right() - 5.0, r.bottom() - 2.0)), 3.0, s, StrokeKind::Inside);
            p.circle_stroke(Pos2::new(r.center().x - 1.5, r.top() + 4.0), 3.0, s);
            p.circle_stroke(Pos2::new(r.right() - 3.5, r.center().y + 1.5), 3.0, s);
        }
        Tab::Comunidad => {
            // dos personas
            for dx in [-4.0f32, 4.0] {
                let c = Pos2::new(r.center().x + dx, r.top() + 6.0);
                p.circle_stroke(c, 3.5, s);
                p.add(egui::Shape::line(vec![Pos2::new(c.x - 6.0, r.bottom() - 1.0), Pos2::new(c.x - 4.0, r.center().y + 2.0), Pos2::new(c.x + 4.0, r.center().y + 2.0), Pos2::new(c.x + 6.0, r.bottom() - 1.0)], s));
            }
        }
        Tab::Layers => {
            // tree glyph
            p.line_segment([Pos2::new(r.left() + 4.0, r.top() + 3.0), Pos2::new(r.left() + 4.0, r.bottom() - 4.0)], s);
            for (i, y) in [r.top() + 4.0, r.center().y, r.bottom() - 4.0].iter().enumerate() {
                let x = if i == 0 { r.left() + 1.0 } else { r.left() + 4.0 };
                p.line_segment([Pos2::new(x, *y), Pos2::new(r.left() + 9.0, *y)], s);
                p.rect_filled(Rect::from_min_size(Pos2::new(r.left() + 10.0, y - 2.5), Vec2::new(r.width() - 11.0, 5.0)), 2.0, c);
            }
        }
    }
}
