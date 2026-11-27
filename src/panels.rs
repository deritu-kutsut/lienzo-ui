//! Header, left rail + drawer (templates, elements, text, images, hierarchy),
//! contextual toolbar, Unity-style inspector, status bar, save and export windows.

use crate::app::{Drop, EditorApp, SCREEN_PRESETS, SIZE_PRESETS, Tab};
use crate::docext::{self, CanvasRenderMode, ImageMode, Orientation};
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
    [225, 95, 30, 255],
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
        egui::CollapsingHeader::new(RichText::new(crate::i18n::t(title)).strong().size(12.5)).id_salt(id).default_open(open).show(ui, |ui| {
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
    // Long labels (German, Russian…): shortened with «…» so they don't overlap the neighbor; the full name goes in the tooltip.
    let full = crate::i18n::t(label);
    let font = FontId::proportional(11.0);
    let fits = |s: &str| ui.fonts_mut(|f| f.layout_no_wrap(s.to_string(), font.clone(), MUTED()).size().x) <= size.x;
    let mut shown = full.clone();
    if !fits(&shown) {
        let chars: Vec<char> = full.chars().collect();
        let mut n = chars.len();
        while n > 1 {
            n -= 1;
            shown = chars[..n].iter().collect::<String>().trim_end().to_string() + "…";
            if fits(&shown) {
                break;
            }
        }
    }
    ui.painter().text(Pos2::new(img.center().x, img.bottom() + 9.0), egui::Align2::CENTER_CENTER, &shown, font, MUTED());
    let resp = resp.on_hover_cursor(egui::CursorIcon::Grab);
    if shown != full { resp.on_hover_text(full) } else { resp }
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
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        self.community_button_profile(ui);
                        ui.add_space(4.0);
                        self.updater_button(ui);
                        let export = crate::icons::btn_tint("export", RichText::new(crate::i18n::t("Export to Unity")).strong().color(Color32::WHITE), Color32::WHITE).fill(ACCENT()).corner_radius(6.0);
                        let tip = if self.settings.unity_project.is_empty() { "Choose your Unity project folder (Ctrl+E)".to_string() } else { format!("Exporta a {} (Ctrl+E)", self.settings.unity_project) };
                        if ui.add(export).on_hover_text(tip).clicked() {
                            self.export_to_unity(false);
                        }
                        if ui.add(crate::icons::btn("code", crate::i18n::t("Code…"))).on_hover_text(crate::i18n::t("View and copy JSON, UXML/USS, IMGUI and scripts (Ctrl+Shift+E)")).clicked() {
                            self.export_open = true;
                        }
                        if ui.add(crate::icons::btn("save", crate::i18n::t("Save"))).on_hover_text(crate::i18n::t("Save as project or as template")).clicked() {
                            self.save_open = true;
                        }
                        if ui.button(crate::i18n::t("Tutorial")).on_hover_text(crate::i18n::t("Open the beginner and pro tutorial (F1)")).clicked() {
                            self.open_tutorial(crate::tutorial::Track::Beginner);
                        }
                        if ui.button(crate::i18n::t("Themes")).on_hover_text(crate::i18n::t("App colours, corners and font; community themes")).clicked() {
                            self.themes_open = true;
                        }
                        if ui.add(crate::icons::btn("check", crate::i18n::t("Review"))).on_hover_text(crate::i18n::t("Checks readability, contrast, touch size and missing images before exporting")).clicked() {
                            self.review_open = true;
                        }
                        if ui.button(crate::i18n::t("Accessibility")).on_hover_text(crate::i18n::t("Language, interface size, high contrast and reduced motion")).clicked() {
                            self.accessibility_open = true;
                        }
                    // everything else gets the space left of the right-hand buttons
                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.add(egui::Image::new(egui::include_image!("../assets/brand/icon.svg")).fit_to_exact_size(Vec2::splat(22.0)));
                    ui.label(RichText::new(crate::i18n::t("Lienzo UI")).strong().size(14.5).color(TEXT()));
                    ui.label(RichText::new(crate::i18n::t("for Unity")).size(11.5).color(MUTED()));
                    ui.add_space(8.0);
                    ui.menu_button(crate::i18n::t("File"), |ui| {
                        ui.set_min_width(230.0);
                        if ui.button(crate::i18n::t("New blank design")).clicked() {
                            self.guard(crate::app::Pending::Template(model::blank_template()));
                            ui.close();
                        }
                        if ui.button(crate::i18n::t("Open project…")).clicked() {
                            ui.close();
                            self.open_project_dialog();
                        }
                        if ui.button(crate::i18n::t("Import from Unity…")).on_hover_text(crate::i18n::t("Turn a uGUI prefab (.prefab) or a Lienzo UI exported design into an editable design")).clicked() {
                            ui.close();
                            self.import_unity_dialog();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new(crate::i18n::t("Save")).shortcut_text("Ctrl+S")).clicked() {
                            ui.close();
                            self.save_current();
                        }
                        if ui.button(crate::i18n::t("Save project as…")).clicked() {
                            ui.close();
                            self.save_project_as();
                        }
                        if ui.button(crate::i18n::t("Save as template…")).clicked() {
                            self.save_open = true;
                            ui.close();
                        }
                        ui.separator();
                        if ui.button(crate::i18n::t("Check for updates…")).clicked() {
                            ui.close();
                            self.updater.open = true;
                            self.updater_search();
                        }
                        ui.separator();
                        if ui.button(crate::i18n::t("Upload images…")).clicked() {
                            ui.close();
                            self.upload_images();
                            self.tab = Tab::Images;
                            self.drawer_open = true;
                        }
                        if ui.button(crate::i18n::t("Upload elements…")).clicked() {
                            ui.close();
                            self.upload_elements();
                            self.tab = Tab::Elements;
                            self.drawer_open = true;
                        }
                        if ui.button(crate::i18n::t("Upload templates…")).clicked() {
                            ui.close();
                            self.upload_templates();
                            self.tab = Tab::Templates;
                            self.drawer_open = true;
                        }
                        if ui.button(crate::i18n::t("Add fonts…")).clicked() {
                            ui.close();
                            let ctx = ui.ctx().clone();
                            self.add_fonts(&ctx);
                        }
                        ui.separator();
                        if ui.add(egui::Button::new(crate::i18n::t("Export to Unity project")).shortcut_text("Ctrl+E")).clicked() {
                            ui.close();
                            self.export_to_unity(false);
                        }
                        if ui.add(egui::Button::new(crate::i18n::t("Export… (code and options)")).shortcut_text("Ctrl+Shift+E")).clicked() {
                            self.export_open = true;
                            ui.close();
                        }
                    });
                    ui.menu_button(crate::i18n::t("Edit"), |ui| {
                        ui.set_min_width(240.0);
                        let has_sel = !self.sel.is_empty();
                        let multi = self.sel.len() > 1;
                        if ui.add_enabled(self.can_undo(), egui::Button::new(crate::i18n::t("Undo")).shortcut_text("Ctrl+Z")).clicked() {
                            self.undo();
                            ui.close();
                        }
                        if ui.add_enabled(self.can_redo(), egui::Button::new(crate::i18n::t("Redo")).shortcut_text("Ctrl+Y")).clicked() {
                            self.redo();
                            ui.close();
                        }
                        ui.separator();
                        if ui.add_enabled(has_sel, egui::Button::new(crate::i18n::t("Copy")).shortcut_text("Ctrl+C")).clicked() {
                            self.copy();
                            ui.close();
                        }
                        if ui.add_enabled(!self.clipboard.is_empty(), egui::Button::new(crate::i18n::t("Paste")).shortcut_text("Ctrl+V")).clicked() {
                            self.paste();
                            ui.close();
                        }
                        if ui.add_enabled(has_sel, egui::Button::new(crate::i18n::t("Duplicate")).shortcut_text("Ctrl+D")).clicked() {
                            self.duplicate_selection(30.0);
                            ui.close();
                        }
                        if ui.add_enabled(has_sel, egui::Button::new(crate::i18n::t("Delete")).shortcut_text("Supr")).clicked() {
                            self.delete_selection();
                            ui.close();
                        }
                        ui.separator();
                        if ui.add_enabled(has_sel, egui::Button::new(crate::i18n::t("Group into Container")).shortcut_text("Ctrl+G")).clicked() {
                            self.group_selection();
                            ui.close();
                        }
                        if ui.add_enabled(self.selection_grouped(), egui::Button::new(crate::i18n::t("Ungroup")).shortcut_text("Ctrl+Shift+G")).clicked() {
                            self.ungroup_selection();
                            ui.close();
                        }
                        if ui.add_enabled(has_sel, egui::Button::new(crate::i18n::t("Unparent"))).clicked() {
                            self.unparent_selection();
                            ui.close();
                        }
                        if ui.add_enabled(has_sel, egui::Button::new(crate::i18n::t("Lock / unlock")).shortcut_text("Ctrl+Shift+L")).clicked() {
                            self.toggle_lock();
                            ui.close();
                        }
                        ui.separator();
                        if ui.add(egui::Button::new(crate::i18n::t("Select all")).shortcut_text("Ctrl+A")).clicked() {
                            self.sel = self.doc.elements.iter().filter(|e| !e.locked && e.visible && e.parent.is_none()).map(|e| e.id).collect();
                            ui.close();
                        }
                        let _ = multi;
                    });
                    ui.menu_button(crate::i18n::t("View"), |ui| {
                        ui.set_min_width(250.0);
                        ui.checkbox(&mut self.rulers, crate::i18n::t("Rules"));
                        ui.checkbox(&mut self.grid.show, crate::i18n::t("Show grid"));
                        ui.checkbox(&mut self.grid.snap, crate::i18n::t("Snap to grid"));
                        egui::Grid::new("gridset").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                            ui.label(crate::i18n::t("Cell (width)"));
                            ui.add(egui::DragValue::new(&mut self.grid.w).range(2.0..=2000.0).speed(1.0).suffix(" px"));
                            ui.end_row();
                            ui.label(crate::i18n::t("Cell (height)"));
                            ui.add(egui::DragValue::new(&mut self.grid.h).range(2.0..=2000.0).speed(1.0).suffix(" px"));
                            ui.end_row();
                            ui.label(crate::i18n::t("Columns"));
                            let mut cols = (self.doc.width / self.grid.w).round().max(1.0) as u32;
                            if ui.add(egui::DragValue::new(&mut cols).range(1..=200)).changed() {
                                self.grid.w = self.doc.width / cols as f32;
                            }
                            ui.end_row();
                            ui.label(crate::i18n::t("Rows"));
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
                        if ui.add(egui::Button::new(crate::i18n::t("Fit to window")).shortcut_text("Ctrl+0")).clicked() {
                            self.fit_pending = true;
                            ui.close();
                        }
                    });
                    ui.menu_button(crate::i18n::t("Canvas"), |ui| {
                        ui.label(RichText::new(crate::i18n::t("Reference resolution (CanvasScaler)")).small().color(MUTED()));
                        for (name, w, h) in SIZE_PRESETS {
                            if ui.button(format!("{}  ·  {} × {}", crate::i18n::t(name), w, h)).clicked() {
                                self.resize_page(*w, *h);
                                ui.close();
                            }
                        }
                    });
                    ui.add_space(4.0);
                    // Landscape / Portrait: which layout is on screen (edits in portrait are stored as overrides)
                    let portrait = self.orientation() == Orientation::Portrait;
                    // room left for the whole bar: labelled toggle only when there is plenty, icon-only otherwise
                    let roomy = ui.available_width() > 560.0;
                    ui.add_enabled_ui(!self.preview, |ui| {
                        ui.spacing_mut().item_spacing.x = 2.0;
                        let land = if roomy { crate::icons::btn("orient_landscape", crate::i18n::t("Landscape")) } else { crate::icons::btn("orient_landscape", "") }.selected(!portrait);
                        if ui.add(land).on_hover_text(format!("{}: {}", crate::i18n::t("Landscape"), crate::i18n::t("Edit the landscape layout"))).clicked() {
                            self.set_orientation(Orientation::Landscape);
                        }
                        let port = if roomy { crate::icons::btn("orient_portrait", crate::i18n::t("Portrait")) } else { crate::icons::btn("orient_portrait", "") }.selected(portrait);
                        if ui.add(port).on_hover_text(format!("{}: {}", crate::i18n::t("Portrait"), crate::i18n::t("Edit the portrait layout: changes are stored as overrides and the landscape layout is not touched"))).clicked() {
                            self.set_orientation(Orientation::Portrait);
                        }
                    });
                    ui.add_space(4.0);
                    if ui.add_enabled_ui(self.can_undo(), |ui| crate::icons::button(ui, "undo", "<", "Undo (Ctrl+Z)")).inner.clicked() {
                        self.undo();
                    }
                    if ui.add_enabled_ui(self.can_redo(), |ui| crate::icons::button(ui, "redo", ">", "Redo (Ctrl+Y)")).inner.clicked() {
                        self.redo();
                    }
                    ui.add_space(4.0);
                    let play = crate::i18n::t(if self.preview { "Stop" } else { "Test" });
                    let ink = if self.preview { Color32::WHITE } else { TEXT() };
                    let play_btn = crate::icons::btn_tint(if self.preview { "stop" } else { "play" }, RichText::new(play).strong().color(ink), ink).fill(if self.preview { theme::OK() } else { FIELD() });
                    if ui.add(play_btn).on_hover_text(crate::i18n::t("Test mode: click runs the code and you can simulate other screens (F5)")).clicked() {
                        self.set_preview(!self.preview);
                    }
                    if self.preview {
                        let cur = match self.preview_screen {
                            Some(s) => format!("{} × {}", s[0], s[1]),
                            None => crate::i18n::t("Reference resolution"),
                        };
                        egui::ComboBox::from_id_salt("screen").selected_text(cur).width(190.0).show_ui(ui, |ui| {
                            for (name, w, h) in SCREEN_PRESETS {
                                let v = if *w == 0.0 { None } else { Some([*w, *h]) };
                                ui.selectable_value(&mut self.preview_screen, v, crate::i18n::t(name));
                            }
                        });
                    }
                    // design title + resolution take what is left and truncate instead of overlapping
                    let room = ui.available_width() - 12.0;
                    if room >= 70.0 {
                        ui.add_space(6.0);
                        let res = format!("{} × {}", self.doc.width, self.doc.height);
                        let res_w = if room >= 190.0 { 72.0 } else { 0.0 };
                        if res_w > 0.0 {
                            ui.add_sized([res_w, 20.0], egui::Label::new(RichText::new(res).size(11.5).color(MUTED())).truncate());
                        }
                        let w = (ui.available_width() - 8.0).clamp(40.0, 180.0);
                        ui.add(egui::TextEdit::singleline(&mut self.doc.title).desired_width(w));
                    }
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
                let tabs = [(Tab::Layers, "Hierarchy"), (Tab::Elements, "Elements"), (Tab::Templates, "Templates"), (Tab::Text, "Text"), (Tab::Images, "Images"), (Tab::Plugins, "Plugins"), (Tab::Community, "Community")];
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
                    ui.add(egui::TextEdit::singleline(&mut self.search).hint_text(crate::i18n::t("Search…")).desired_width(ui.available_width()).margin(Margin::symmetric(8, 5)));
                });
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| match self.tab {
                    Tab::Templates => self.tab_templates(ui),
                    Tab::Elements => self.tab_elements(ui),
                    Tab::Text => self.tab_text(ui),
                    Tab::Images => self.tab_images(ui),
                    Tab::Layers => self.tab_layers(ui),
                    Tab::Plugins => self.tab_plugins(ui),
                    Tab::Community => self.tab_community(ui),
                });
            });
    }

    fn matches(&self, label: &str) -> bool {
        let q = self.search.trim().to_lowercase();
        q.is_empty() || label.to_lowercase().contains(&q) || crate::i18n::t(label).to_lowercase().contains(&q)
    }

    fn place_or_drag(&mut self, resp: &egui::Response, proto: Element) {
        if resp.drag_started() {
            self.dnd = Some(vec![proto]);
            self.dnd_ov.clear();
        } else if resp.clicked() {
            self.insert(proto, None);
        }
    }

    fn tab_templates(&mut self, ui: &mut egui::Ui) {
        section(ui, "Interface templates");
        hint(ui, &crate::i18n::t("Click to replace the design (Ctrl+Z undoes). All use hierarchy, anchors and Layout Groups."));
        ui.add_space(6.0);
        let (w, h) = (self.doc.width, self.doc.height);
        let mut chosen = None;
        let n = model::TEMPLATES.len();
        // Thumbnails are built the first time their block is open (there are hundreds) and kept for this page size.
        if self.template_thumbs.as_ref().is_none_or(|(size, v)| *size != (w, h) || v.len() != n) {
            self.template_thumbs = Some(((w, h), (0..n).map(|_| None).collect()));
        }
        let Some((_, mut thumbs)) = self.template_thumbs.take() else { return };
        let searching = !self.search.trim().is_empty();
        let mut tile_of = |ui: &mut egui::Ui, i: usize, caption: &str| {
            let doc = thumbs[i].get_or_insert_with(|| model::template(i, w, h));
            if tile(ui, Vec2::new(128.0, 82.0), caption, |p, r| render::paint_doc_thumb(p, doc, r.expand(6.0))).clicked() {
                chosen = Some(i);
            }
        };
        let classic: Vec<usize> = model::template_order().filter(|&i| i < model::BUILTIN_TEMPLATES.len()).filter(|&i| self.matches(model::TEMPLATES[i])).collect();
        if !classic.is_empty() {
            egui::CollapsingHeader::new(RichText::new(format!("{} ({})", crate::i18n::t("Classic templates"), classic.len())).strong()).id_salt("tpl_classic").default_open(false).open(searching.then_some(true)).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                    for &i in &classic {
                        tile_of(ui, i, &model::template_label(i));
                    }
                });
            });
        }
        let base = model::BUILTIN_TEMPLATES.len();
        for group in crate::gallery::groups() {
            // (section, [(type, [template indices that match the search])])
            let mut shown: Vec<(&str, Vec<(&str, Vec<usize>)>)> = Vec::new();
            for sec in crate::gallery::SECTIONS.iter().filter(|s| s.group == group) {
                let head = self.matches(group) || self.matches(sec.name);
                let mut types = Vec::new();
                for ty in sec.types {
                    let idx: Vec<usize> = crate::gallery::of_type(ty)
                        .into_iter()
                        .filter(|&g| head || self.matches(ty) || self.matches(crate::gallery::name(g)) || self.matches(crate::gallery::entry(g).style.name()))
                        .map(|g| base + g)
                        .collect();
                    if !idx.is_empty() {
                        types.push((*ty, idx));
                    }
                }
                if !types.is_empty() {
                    shown.push((sec.name, types));
                }
            }
            if shown.is_empty() {
                continue;
            }
            let count: usize = shown.iter().flat_map(|(_, t)| t.iter()).map(|(_, v)| v.len()).sum();
            let title = format!("{} ({})", crate::i18n::t(group), count);
            egui::CollapsingHeader::new(RichText::new(title).strong()).id_salt(("tpl_group", group)).default_open(false).open(searching.then_some(true)).show(ui, |ui| {
                for (sec, types) in &shown {
                    ui.add_space(4.0);
                    ui.label(RichText::new(crate::i18n::t(sec).to_uppercase()).size(10.5).color(MUTED()));
                    for (ty, idx) in types {
                        ui.label(RichText::new(crate::i18n::t(ty)).size(12.0).color(TEXT()));
                        ui.horizontal_wrapped(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                            for &i in idx {
                                tile_of(ui, i, &crate::i18n::t(crate::gallery::entry(i - base).style.name()));
                            }
                        });
                    }
                }
            });
        }
        self.template_thumbs = Some(((w, h), thumbs));
        if let Some(i) = chosen {
            self.guard(crate::app::Pending::Template(i));
        }

        section(ui, "My templates");
        ui.horizontal(|ui| {
            if ui.add(crate::icons::btn("plus", crate::i18n::t("Upload template"))).on_hover_text(crate::i18n::t(".json files saved with this editor")).clicked() {
                self.upload_templates();
            }
            if ui.add(crate::icons::btn("save", crate::i18n::t("Save current design"))).clicked() {
                self.save_open = true;
            }
        });
        ui.add_space(6.0);
        if self.library.templates.is_empty() {
            hint(ui, &crate::i18n::t("You don't have any templates yet. Save your design as a template or upload one."));
        }
        let mut load = None;
        let mut remove = None;
        // The list is borrowed (not cloned every frame) and returned at the end.
        let templates = std::mem::take(&mut self.library.templates);
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
                    if ui.button(crate::i18n::t("Use template")).clicked() {
                        load = Some(t.doc.clone());
                        ui.close();
                    }
                    if ui.add(crate::icons::btn("trash", crate::i18n::t("Remove from my library"))).clicked() {
                        remove = Some(t.path.clone());
                        ui.close();
                    }
                });
            }
        });
        if let Some(d) = load {
            self.guard(crate::app::Pending::UserTemplate(Box::new(d)));
        }
        if self.library.templates.is_empty() {
            self.library.templates = templates;
        }
        if let Some(p) = remove {
            self.library.remove(&p);
        }
    }

    fn my_elements(&mut self, ui: &mut egui::Ui) {
        section(ui, "My elements (reusable prefabs)");
        ui.horizontal_wrapped(|ui| {
            if ui.add(crate::icons::btn("plus", crate::i18n::t("Upload element"))).on_hover_text(crate::i18n::t("PNG/JPG images (icons with transparency) or .json elements")).clicked() {
                self.upload_elements();
            }
        });
        if !self.sel.is_empty() {
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.element_name).hint_text(crate::i18n::t("Name")).desired_width(120.0));
                if ui.button(crate::i18n::t("Save selection")).on_hover_text(crate::i18n::t("Saves the selection (with its children) for reuse")).clicked() {
                    let n = std::mem::take(&mut self.element_name);
                    self.save_selection_as_element(&n);
                }
            });
        }
        ui.add_space(6.0);
        if self.library.elements.is_empty() {
            hint(ui, &crate::i18n::t("Upload images or save a selection to have it here."));
        }
        // The list is borrowed (not cloned every frame) and returned at the end.
        let items = std::mem::take(&mut self.library.elements);
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
                    self.dnd_ov = it.overrides.clone();
                } else if resp.clicked() {
                    if it.items.len() == 1 && it.overrides.is_empty() {
                        self.insert(it.items[0].clone(), None);
                    } else {
                        self.insert_items_with(&it.items, &it.overrides, None);
                    }
                }
                resp.context_menu(|ui| {
                    if ui.add(crate::icons::btn("trash", crate::i18n::t("Remove from my library"))).clicked() {
                        remove = Some(it.path.clone());
                        ui.close();
                    }
                });
            }
        });
        if self.library.elements.is_empty() {
            self.library.elements = items;
        }
        if let Some(p) = remove {
            self.library.remove(&p);
        }
    }

    fn tab_elements(&mut self, ui: &mut egui::Ui) {
        if let Some(p) = self.insert_parent().and_then(|p| self.doc.get(p)) {
            egui::Frame::new().fill(ACCENT_SOFT()).corner_radius(6.0).inner_margin(Margin::same(6)).show(ui, |ui| {
                ui.label(RichText::new(format!("They will be added inside “{}”", p.name)).small().color(TEXT()));
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
            vec![("Square", sq), ("Rounded", rr), ("Circle", ci), ("Outline", ol), ("Line", bar)]
        };
        let controls: Vec<(&str, Element)> = [Kind::Button, Kind::Toggle, Kind::Slider, Kind::InputField, Kind::Dropdown, Kind::Panel, Kind::Image, Kind::ScrollView]
            .iter()
            .map(|k| (k.label(), Element::new(0, *k)))
            .collect();
        let containers: Vec<(&str, Element)> = {
            let mut c = Element::new(0, Kind::Empty);
            c.name = "Container".into();
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
            grid.name = "Grid".into();
            grid.w = 560.0;
            grid.h = 400.0;
            grid.fill = [255, 255, 255, 40];
            grid.layout.mode = LayoutMode::Grid;
            grid.layout.align = 1;
            vec![("Container", c), ("Row (Horizontal)", row), ("Column (Vertical)", col), ("Grid", grid)]
        };
        let presets: Vec<(&str, Element)> = {
            let mut pill = Element::new(0, Kind::Button);
            pill.name = "Pill button".into();
            pill.text = "Play".into();
            pill.fill = [225, 95, 30, 255];
            pill.radius = 44.0;
            pill.w = 360.0;
            pill.h = 88.0;
            pill.shadow.on = true;
            let mut ghost = Element::new(0, Kind::Button);
            ghost.name = "Outline button".into();
            ghost.text = "Options".into();
            ghost.fill = [0, 0, 0, 0];
            ghost.stroke = [124, 92, 255, 255];
            ghost.stroke_w = 4.0;
            ghost.text_color = [124, 92, 255, 255];
            ghost.radius = 44.0;
            ghost.w = 360.0;
            ghost.h = 88.0;
            let mut close = Element::new(0, Kind::Button);
            close.name = "Close button".into();
            close.text = "X".into();
            close.fill = [235, 70, 90, 255];
            close.radius = 35.0;
            close.w = 70.0;
            close.h = 70.0;
            let mut hp = Element::new(0, Kind::Slider);
            hp.name = "Health bar".into();
            hp.fill = [235, 70, 90, 255];
            hp.fill2 = [60, 60, 70, 255];
            hp.value = 0.72;
            hp.interactable = false;
            hp.w = 460.0;
            hp.h = 34.0;
            let mut mana = hp.clone();
            mana.name = "Mana bar".into();
            mana.fill = [70, 130, 255, 255];
            mana.value = 0.45;
            let mut card = Element::new(0, Kind::Panel);
            card.name = "Card".into();
            card.fill = [255, 255, 255, 255];
            card.radius = 24.0;
            card.shadow.on = true;
            card.shadow.color = [0, 0, 0, 50];
            card.w = 420.0;
            card.h = 300.0;
            let mut glass = Element::new(0, Kind::Panel);
            glass.name = "Translucent panel".into();
            glass.fill = [10, 10, 20, 150];
            glass.stroke = [255, 255, 255, 60];
            glass.stroke_w = 2.0;
            glass.radius = 20.0;
            glass.w = 520.0;
            glass.h = 320.0;
            let mut badge = Element::new(0, Kind::Ellipse);
            badge.name = "Badge".into();
            badge.fill = [235, 70, 90, 255];
            badge.text = "3".into();
            badge.font_size = 30.0;
            badge.bold = true;
            badge.text_color = [255, 255, 255, 255];
            badge.w = 56.0;
            badge.h = 56.0;
            let mut slot = Element::new(0, Kind::Panel);
            slot.name = "Slot".into();
            slot.fill = [60, 64, 84, 255];
            slot.stroke = [110, 116, 150, 255];
            slot.stroke_w = 2.0;
            slot.radius = 14.0;
            slot.w = 140.0;
            slot.h = 140.0;
            vec![
                ("Pill button", pill),
                ("Outline button", ghost),
                ("Close button", close),
                ("Health bar", hp),
                ("Mana bar", mana),
                ("Card", card),
                ("Translucent panel", glass),
                ("Badge", badge),
                ("Slot", slot),
            ]
        };
        for (title, items) in [("UI controls (uGUI)", controls), ("Ready-made components", presets), ("Containers and Layout Groups", containers), ("Shapes", shapes)] {
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
        self.preset_sections(ui);
        ui.add_space(6.0);
        hint(ui, &crate::i18n::t("Click adds to the center (or inside the selected panel/container). Drag to drop wherever you want: if you drop it on a panel, it becomes a child."));
        self.my_elements(ui);
    }

    fn tab_text(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        let add = crate::icons::btn_tint("text", RichText::new(crate::i18n::t("Add a text box")).color(Color32::WHITE).strong(), Color32::WHITE).fill(ACCENT()).corner_radius(6.0).min_size(Vec2::new(ui.available_width(), 36.0));
        if ui.add(add).clicked() {
            let mut e = Element::new(0, Kind::Text);
            e.font_size = 32.0;
            self.insert(e, None);
            self.editing = self.sel.first().copied();
            self.editing_new = true;
        }
        hint(ui, &crate::i18n::t("In Unity they are created with TextMeshPro (TextMeshProUGUI)."));
        section(ui, "Text styles");
        let styles: [(&str, f32, bool, f32); 4] = [("Add a heading", 80.0, true, 900.0), ("Add a subheading", 52.0, true, 700.0), ("Add body text", 30.0, false, 600.0), ("Small label", 22.0, false, 360.0)];
        for (label, size, bold, w) in styles {
            if !self.matches(label) {
                continue;
            }
            let shown = (size * 0.3).clamp(12.0, 24.0);
            let (r, resp) = ui.allocate_exact_size(Vec2::new(ui.available_width(), shown + 22.0), Sense::click_and_drag());
            ui.painter().rect_filled(r, 7.0, if resp.hovered() { HOVER() } else { FIELD() });
            let pos = Pos2::new(r.left() + 12.0, r.center().y);
            let shown_label = crate::i18n::t(label);
            ui.painter().text(pos, egui::Align2::LEFT_CENTER, &shown_label, FontId::proportional(shown), TEXT());
            if bold {
                ui.painter().text(pos + Vec2::new(0.7, 0.0), egui::Align2::LEFT_CENTER, &shown_label, FontId::proportional(shown), TEXT());
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
        section(ui, "For buttons and HUD");
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            let mut score = Element::new(0, Kind::Text);
            score.text = "000 000".into();
            score.font_size = 60.0;
            score.bold = true;
            score.text_color = [255, 200, 60, 255];
            score.w = 360.0;
            let mut tag = Element::new(0, Kind::Button);
            tag.text = "NEW".into();
            tag.fill = [235, 70, 90, 255];
            tag.radius = 40.0;
            tag.w = 200.0;
            tag.h = 64.0;
            let mut timer = Element::new(0, Kind::Text);
            timer.name = "Timer".into();
            timer.text = "02:45".into();
            timer.font_size = 64.0;
            timer.bold = true;
            timer.text_color = [255, 255, 255, 255];
            timer.w = 240.0;
            timer.h = 84.0;
            let mut coins = Element::new(0, Kind::Text);
            coins.name = "Coins".into();
            coins.text = "x 1 250".into();
            coins.font_size = 44.0;
            coins.bold = true;
            coins.align = 0;
            coins.text_color = [255, 214, 90, 255];
            coins.w = 260.0;
            coins.h = 60.0;
            let mut dmg = Element::new(0, Kind::Text);
            dmg.name = "Damage".into();
            dmg.text = "-128".into();
            dmg.font_size = 56.0;
            dmg.bold = true;
            dmg.italic = true;
            dmg.text_color = [255, 80, 80, 255];
            dmg.w = 200.0;
            dmg.h = 72.0;
            let mut combo = Element::new(0, Kind::Text);
            combo.name = "Combo".into();
            combo.text = "COMBO x12!".into();
            combo.font_size = 58.0;
            combo.bold = true;
            combo.rot = -8.0;
            combo.text_color = [0, 220, 230, 255];
            combo.w = 420.0;
            combo.h = 80.0;
            let mut lvl = Element::new(0, Kind::Button);
            lvl.name = "Level".into();
            lvl.text = "NV. 12".into();
            lvl.font_size = 28.0;
            lvl.bold = true;
            lvl.fill = [124, 92, 255, 255];
            lvl.radius = 12.0;
            lvl.w = 140.0;
            lvl.h = 52.0;
            let mut sub = Element::new(0, Kind::Panel);
            sub.name = "Dialogue subtitle".into();
            sub.text = "Watch out, it's coming from the left!".into();
            sub.font_size = 32.0;
            sub.fill = [0, 0, 0, 170];
            sub.text_color = [255, 255, 255, 255];
            sub.radius = 10.0;
            sub.w = 820.0;
            sub.h = 70.0;
            let mut all: Vec<(&str, Element)> = vec![("Score", score), ("Tag", tag), ("Timer", timer), ("Coins", coins), ("Damage", dmg), ("Combo", combo), ("Level", lvl), ("Dialogue subtitle", sub)];
            all.extend(crate::presets::text_hud());
            for (label, proto) in all {
                if !self.matches(label) {
                    continue;
                }
                let resp = tile(ui, Vec2::new(124.0, 60.0), label, |p, r| {
                    p.rect_filled(r.expand(8.0), 6.0, Color32::from_rgb(28, 28, 38));
                    render::paint_element_thumb(p, &proto, r);
                });
                self.place_or_drag(&resp, proto);
            }
        });
    }

    /// Buttons, HUD and small components (see `presets.rs`): click or drag inserts the whole tree.
    fn preset_sections(&mut self, ui: &mut egui::Ui) {
        for sec in crate::presets::SECTIONS {
            section(ui, sec);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                for p in crate::presets::presets().iter().filter(|p| p.section == *sec) {
                    if !self.matches(p.label) {
                        continue;
                    }
                    let resp = tile(ui, Vec2::new(124.0, 64.0), p.label, |pt, r| render::paint_doc_thumb(pt, &p.thumb, r.expand(8.0)));
                    if resp.drag_started() {
                        self.dnd = Some(p.items.clone());
                        self.dnd_ov.clear();
                    } else if resp.clicked() {
                        self.insert_items_with(&p.items, &[], None);
                    }
                }
            });
        }
    }

    /// Copies a built-in pack image into the library and builds its element (None + status message when it can't be copied).
    fn pack_element(&mut self, it: &crate::pack::PackItem) -> Option<Element> {
        match crate::pack::extract(it) {
            Ok(file) => Some(crate::pack::element_for(it, &file, (self.doc.width, self.doc.height))),
            Err(e) => {
                self.status = crate::i18n::tf("Could not copy the pack image: {}", &[&e]);
                None
            }
        }
    }

    /// One category of the built-in pack as tiles with lazily decoded thumbnails.
    fn pack_tiles(&mut self, ui: &mut egui::Ui, cat: &str) {
        let items: Vec<&'static crate::pack::PackItem> = crate::pack::items().iter().filter(|i| i.category == cat && self.matches_pack(i)).collect();
        if items.is_empty() {
            if !self.search.trim().is_empty() {
                hint(ui, &crate::i18n::t("No images match the search"));
            }
            return;
        }
        // at most a few new thumbnails are decoded per frame; the rest appear on the next ones
        let mut budget = 6u32;
        let ctx = ui.ctx().clone();
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            for it in items {
                let tex = crate::pack::thumbnail(&ctx, it, &mut budget);
                let resp = tile(ui, Vec2::new(124.0, 84.0), &it.name, |p, r| {
                    if let Some(t) = &tex {
                        let sz = t.size_vec2();
                        let s = (r.width() / sz.x).min(r.height() / sz.y);
                        let dst = Rect::from_center_size(r.center(), sz * s);
                        p.image(t.id(), dst, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                    }
                });
                let resp = resp.on_hover_text(it.tags.join(", "));
                if resp.drag_started() {
                    if let Some(e) = self.pack_element(it) {
                        self.dnd = Some(vec![e]);
                        self.dnd_ov.clear();
                    }
                } else if resp.clicked() {
                    if let Some(e) = self.pack_element(it) {
                        let name = e.name.clone();
                        self.insert(e, None);
                        if it.category == "backgrounds" {
                            self.reorder(Some(false), 0);
                        }
                        self.status = crate::i18n::tf("Pack image added: {}", &[&name]);
                    }
                }
            }
        });
    }

    fn matches_pack(&self, it: &crate::pack::PackItem) -> bool {
        crate::pack::matches(it, &self.search)
    }

    fn tab_images(&mut self, ui: &mut egui::Ui) {
        ui.add_space(6.0);
        let up = crate::icons::btn_tint("plus", RichText::new(crate::i18n::t("Upload images")).color(Color32::WHITE).strong(), Color32::WHITE).fill(ACCENT()).corner_radius(6.0).min_size(Vec2::new(ui.available_width(), 36.0));
        if ui.add(up).on_hover_text(crate::i18n::t("PNG, JPG, BMP, GIF or WEBP — exported as Sprites")).clicked() {
            self.upload_images();
        }
        // category chips: «Your images» first, then the built-in pack, then the generated gradients
        let chip_id = egui::Id::NULL.with("images_tab_category");
        let mut cat: String = ui.ctx().data_mut(|d| d.get_temp::<String>(chip_id)).unwrap_or_else(|| "all".to_string());
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 4.0);
            let mut chips: Vec<(String, String)> = vec![("all".into(), crate::i18n::t("All")), ("mine".into(), crate::i18n::t("Your images"))];
            chips.extend(crate::pack::CATEGORIES.iter().map(|(c, _)| (c.to_string(), crate::pack::category_label(c))));
            chips.push(("gradients".into(), crate::i18n::t("Gradients")));
            for (id, label) in chips {
                if ui.selectable_label(cat == id, RichText::new(label).size(12.0)).clicked() {
                    cat = id;
                }
            }
        });
        ui.ctx().data_mut(|d| d.insert_temp(chip_id, cat.clone()));
        let all = cat == "all";
        if all || cat == "mine" {
            self.your_images(ui);
        }
        for (c, label) in crate::pack::CATEGORIES {
            if all || cat == *c {
                if all {
                    section(ui, label);
                } else {
                    ui.add_space(6.0);
                }
                if *c == "pixel" && !self.doc.pixel_art {
                    hint(ui, &crate::i18n::t("Tip: turn on Pixel art in the page settings so the sprites stay crisp."));
                }
                self.pack_tiles(ui, c);
            }
        }
        if !all && cat != "mine" && cat != "gradients" {
            ui.add_space(6.0);
            hint(ui, &crate::i18n::t("Built-in images are copied to your library when you use them"));
        }
        if all || cat == "gradients" {
            self.gradients(ui);
        }
    }

    fn your_images(&mut self, ui: &mut egui::Ui) {
        section(ui, "Your images");
        // the built-in pack copies its files here as `pack_<id>.<ext>`: they belong to the pack tiles, not to "Your images"
        if self.library.images.iter().all(|i| i.file.starts_with("pack_")) {
            hint(ui, &crate::i18n::t("The images you upload will appear here and are saved for your next designs."));
        }
        // The list is borrowed (not cloned every frame) and returned at the end.
        let images = std::mem::take(&mut self.library.images);
        let mut remove = None;
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
            for im in &images {
                if im.file.starts_with("pack_") || !self.matches(&im.file) {
                    continue;
                }
                let resp = tile(ui, Vec2::new(124.0, 84.0), &im.file, |p, r| {
                    let e = crate::library::image_element(&im.file, im.w, im.h, 300.0);
                    render::paint_element_thumb(p, &e, r.expand(8.0));
                });
                if resp.drag_started() {
                    let max = (self.doc.width.min(self.doc.height) * 0.5).max(100.0);
                    self.dnd = Some(vec![crate::library::image_element(&im.file, im.w, im.h, max)]);
                    self.dnd_ov.clear();
                } else if resp.clicked() {
                    self.insert_library_image(&im.file, None);
                }
                resp.context_menu(|ui| {
                    if ui.add(crate::icons::btn("trash", crate::i18n::t("Remove from my library"))).clicked() {
                        remove = Some(crate::library::images_dir().join(&im.file));
                        ui.close();
                    }
                });
            }
        });
        if self.library.images.is_empty() {
            self.library.images = images;
        }
        if let Some(p) = remove {
            self.library.remove(&p);
        }
    }

    fn gradients(&mut self, ui: &mut egui::Ui) {
        section(ui, "Gradients");
        hint(ui, &crate::i18n::t("In uGUI they are exported as a generated gradient Sprite."));
        ui.add_space(4.0);
        let pairs: [(Rgba, Rgba, &str); 16] = [
            ([255, 126, 179, 255], [255, 117, 140, 255], "Sunset"),
            ([46, 32, 110, 255], [12, 120, 150, 255], "Night"),
            ([120, 190, 240, 255], [70, 140, 80, 255], "Landscape"),
            ([255, 214, 120, 255], [255, 120, 60, 255], "Fire"),
            ([0, 196, 204, 255], [124, 92, 255, 255], "Aurora"),
            ([240, 240, 245, 255], [180, 185, 200, 255], "Fog"),
            ([30, 30, 40, 255], [90, 90, 110, 255], "Slate"),
            ([140, 230, 160, 255], [30, 130, 90, 255], "Forest"),
            ([130, 220, 255, 255], [20, 60, 140, 255], "Ocean"),
            ([255, 240, 200, 255], [230, 170, 90, 255], "Desert"),
            ([255, 90, 200, 255], [40, 10, 90, 255], "Neon"),
            ([230, 245, 255, 255], [150, 190, 230, 255], "Ice"),
            ([140, 30, 30, 255], [20, 5, 10, 255], "Volcano"),
            ([255, 220, 240, 255], [200, 170, 255, 255], "Candy"),
            ([90, 70, 50, 255], [40, 30, 20, 255], "Wood"),
            ([10, 10, 14, 255], [10, 10, 14, 0], "Vignette"),
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
        section(ui, &crate::i18n::tf("Hierarchy · {}", &[&self.doc.title]));
        hint(ui, &crate::i18n::t("As in Unity: items below are drawn on top. Drag rows to reorder or to nest an element inside another."));
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            if ui.add(crate::icons::btn("empty", crate::i18n::t("Container")).small()).on_hover_text(crate::i18n::t("Empty RectTransform (inside the selection if it is a panel)")).clicked() {
                let mut c = Element::new(0, Kind::Empty);
                c.name = "Container".into();
                self.insert(c, None);
            }
            if ui.add_enabled(!self.sel.is_empty(), egui::Button::new(crate::i18n::t("Group")).small()).clicked() {
                self.group_selection();
            }
            if ui.add_enabled(self.selection_grouped(), egui::Button::new(crate::i18n::t("Ungroup")).small()).clicked() {
                self.ungroup_selection();
            }
            if ui.add_enabled(self.sel.iter().any(|s| self.doc.parent_of(*s).is_some()), crate::icons::btn("parent_up", crate::i18n::t("Take out")).small()).on_hover_text(crate::i18n::t("Unparent")).clicked() {
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
            let overridden = self.is_overridden(id);
            if overridden {
                badges.push("override");
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
                        crate::icons::paint_or(ui, b, br, if b == "bolt" { theme::WARN() } else if b == "override" { ACCENT() } else { MUTED() }, if b == "override" { "P" } else { "" });
                        if b == "override" {
                            ui.interact(br, ui.scope_id().with(("ovr", id)), Sense::hover()).on_hover_text(crate::i18n::t("Overridden in the portrait layout (right-click: Reset to landscape)"));
                        }
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
            if eye_r.on_hover_text(crate::i18n::t("Active / inactive (SetActive in Unity)")).clicked() {
                if let Some(el) = self.doc.get_mut(id) {
                    el.visible = !el.visible;
                }
            } else if lock_r.on_hover_text(crate::i18n::t("Lock in editor")).clicked() {
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
                if ui.button(crate::i18n::t("Rename")).clicked() {
                    self.renaming = Some(id);
                    ui.close();
                }
                if overridden && ui.add(crate::icons::btn("override", crate::i18n::t("Reset to landscape"))).clicked() {
                    let ids: Vec<u64> = self.sel.iter().copied().filter(|s| self.is_overridden(*s)).collect();
                    self.reset_to_landscape(&ids);
                    ui.close();
                }
                if ui.add(crate::icons::btn("duplicate", crate::i18n::t("Duplicate"))).clicked() {
                    self.duplicate_selection(30.0);
                    ui.close();
                }
                if ui.add(crate::icons::btn("empty", crate::i18n::t("Group into Container"))).clicked() {
                    self.group_selection();
                    ui.close();
                }
                if self.doc.parent_of(id).is_some() && ui.button(crate::i18n::t("Unparent")).clicked() {
                    self.unparent_selection();
                    ui.close();
                }
                if ui.button(crate::i18n::t("Add child Container")).clicked() {
                    self.sel = vec![id];
                    let mut c = Element::new(0, Kind::Empty);
                    c.name = "Container".into();
                    if let Some(pe) = self.doc.get(id) {
                        c.w = pe.w * 0.8;
                        c.h = pe.h * 0.8;
                    }
                    self.insert(c, None);
                    ui.close();
                }
                ui.separator();
                if ui.add(crate::icons::btn("trash", crate::i18n::t("Delete"))).clicked() {
                    self.delete_selection();
                    ui.close();
                }
            });
            rects.push((id, r));
        }
        if self.doc.elements.is_empty() {
            hint(ui, &crate::i18n::t("The design is empty."));
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
                            ui.label(RichText::new(crate::i18n::t("Test mode")).strong().color(theme::OK()));
                            ui.label(RichText::new(crate::i18n::t("Changes made by the code are discarded when you exit.")).color(MUTED()));
                            if ui.button(crate::i18n::t("Clear console")).clicked() {
                                self.console.clear();
                            }
                            if ui.add(crate::icons::btn("stop", crate::i18n::t("Back to editing"))).clicked() {
                                self.set_preview(false);
                            }
                            return;
                        }
                        if self.sel.is_empty() {
                            ui.label(crate::i18n::t("Background"));
                            rgba_edit(ui, &mut self.doc.background);
                            ui.separator();
                            ui.label(RichText::new(crate::i18n::t("Click selects · Ctrl+click enters groups · double-click edits text · Space+drag pans the view")).color(MUTED()));
                            return;
                        }
                        let first_id = self.sel[0];
                        let Some(orig) = self.doc.get(first_id).cloned() else { return };
                        let mut p = orig.clone();
                        let mut add_font_clicked = false;
                        let k = p.kind;
                        let (r, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                        crate::icons::paint_or(ui, crate::icons::kind_name(k), r, MUTED(), k.icon());
                        ui.label(RichText::new(crate::i18n::t(k.label())).color(MUTED()));
                        ui.separator();
                        if k.has_fill() {
                            rgba_edit(ui, &mut p.fill).on_hover_text(crate::i18n::t("Color (Image.color)"));
                            if matches!(k, Kind::Image | Kind::Slider) {
                                rgba_edit(ui, &mut p.fill2).on_hover_text(if k == Kind::Image { "Second gradient colour" } else { "Track colour" });
                            }
                            ui.separator();
                            ui.label(crate::i18n::t("Border"));
                            rgba_edit(ui, &mut p.stroke);
                            ui.add(egui::DragValue::new(&mut p.stroke_w).range(0.0..=40.0).speed(0.2).suffix(" px"));
                            if !matches!(k, Kind::Ellipse | Kind::Slider | Kind::Toggle) {
                                ui.label(crate::i18n::t("Corners"));
                                ui.add(egui::DragValue::new(&mut p.radius).range(0.0..=500.0).speed(0.5));
                            }
                        }
                        if k.has_text() {
                            ui.separator();
                            if ui.small_button(crate::i18n::t("−")).clicked() {
                                p.font_size = (p.font_size - 2.0).max(4.0);
                            }
                            ui.add(egui::DragValue::new(&mut p.font_size).range(4.0..=400.0).speed(0.5)).on_hover_text(crate::i18n::t("Font size"));
                            if ui.small_button(crate::i18n::t("+")).clicked() {
                                p.font_size += 2.0;
                            }
                            rgba_edit(ui, &mut p.text_color).on_hover_text(crate::i18n::t("Text color"));
                            ui.toggle_value(&mut p.bold, RichText::new(crate::i18n::t("B")).strong()).on_hover_text(crate::i18n::t("Bold"));
                            ui.toggle_value(&mut p.italic, RichText::new(crate::i18n::t("I")).italics()).on_hover_text(crate::i18n::t("Italic"));
                            ui.toggle_value(&mut p.underline, RichText::new(crate::i18n::t("U")).underline()).on_hover_text(crate::i18n::t("Underline"));
                            let font_label = if p.font.is_empty() { crate::i18n::t("Default") } else { p.font.clone() };
                            egui::ComboBox::from_id_salt("font_pick")
                                .selected_text(font_label)
                                .width(120.0)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut p.font, String::new(), crate::i18n::t("Default"));
                                    ui.selectable_value(&mut p.font, "Monospace".to_string(), RichText::new(crate::i18n::t("Monospace")).monospace());
                                    for f in render::available_fonts() {
                                        let fam = egui::FontFamily::Name(f.as_str().into());
                                        ui.selectable_value(&mut p.font, f.clone(), RichText::new(f).family(fam));
                                    }
                                    ui.separator();
                                    if ui.button(crate::i18n::t("+ Add font…")).clicked() {
                                        add_font_clicked = true;
                                    }
                                })
                                .response
                                .on_hover_text(crate::i18n::t("Font (exported as TMP Font Asset)"));
                            ui.separator();
                            for (i, (icon, tip)) in [("align_left", "Left"), ("align_center", "Center"), ("align_right", "Right")].iter().enumerate() {
                                if crate::icons::toggle(ui, p.align == i as u8, icon, &tip[..1], tip).clicked() {
                                    p.align = i as u8;
                                }
                            }
                            if k != Kind::Text {
                                ui.add_space(3.0);
                                for (i, (icon, tip)) in [("align_top", "Top"), ("align_middle", "Middle"), ("align_bottom", "Bottom")].iter().enumerate() {
                                    if crate::icons::toggle(ui, p.valign == i as u8, icon, &tip[..1], tip).clicked() {
                                        p.valign = i as u8;
                                    }
                                }
                            }
                        }
                        ui.separator();
                        let mut pct = p.opacity * 100.0;
                        if ui.add(egui::DragValue::new(&mut pct).range(0.0..=100.0).speed(0.5).prefix("◐ ").suffix("%").max_decimals(0)).on_hover_text(crate::i18n::t("Opacity")).changed() {
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
                        if crate::icons::button(ui, "close", "x", "Close the inspector").clicked() {
                            self.inspector_open = false;
                        }
                    });
                });
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    // long (German, Russian…) texts wrap inside the panel instead of widening it
                    ui.set_max_width(ui.available_width());
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
        let portrait_view = self.orientation() == Orientation::Portrait;
        component(ui, "canvas", &crate::i18n::t("Canvas · Canvas Scaler"), true, |ui| {
            hint(ui, &crate::i18n::t("UI Scale Mode: Scale With Screen Size"));
            if portrait_view {
                ui.label(RichText::new(crate::i18n::t("Portrait view: the reference resolution below is the portrait one.")).small().color(ACCENT()));
            }
            egui::Grid::new("page").num_columns(2).spacing([8.0, 6.0]).show(ui, |ui| {
                let mut w = self.doc.width;
                let mut h = self.doc.height;
                ui.label(crate::i18n::t("Reference X"));
                let rw = ui.add(egui::DragValue::new(&mut w).range(64.0..=8192.0).suffix(" px"));
                ui.end_row();
                ui.label(crate::i18n::t("Reference Y"));
                let rh = ui.add(egui::DragValue::new(&mut h).range(64.0..=8192.0).suffix(" px"));
                ui.end_row();
                if (rw.changed() || rh.changed()) && (w != self.doc.width || h != self.doc.height) {
                    self.resize_page(w.round(), h.round());
                }
                ui.label(crate::i18n::t("Match"));
                ui.vertical(|ui| {
                    ui.add(egui::Slider::new(&mut self.doc.canvas_match, 0.0..=1.0));
                    ui.label(RichText::new(crate::i18n::t("Width / Height")).small().color(MUTED()));
                });
                ui.end_row();
                ui.label(crate::i18n::t("Background"));
                ui.horizontal(|ui| {
                    rgba_edit(ui, &mut self.doc.background);
                    if self.doc.background[3] == 0 {
                        ui.label(RichText::new(crate::i18n::t("Transparent overlay")).small().color(ACCENT()));
                    }
                });
                ui.end_row();
            });
            // outside the Grid: a Grid cell never wraps, so long translated labels would widen the panel
            ui.add_space(2.0);
            ui.label(RichText::new(crate::i18n::t("Pixel art")).size(11.5).color(MUTED()));
            ui.checkbox(&mut self.doc.pixel_art, crate::i18n::t("No smoothing, whole pixels"))
                .on_hover_text(crate::i18n::t("Images with nearest-neighbor filtering, positions and sizes in whole pixels. In Unity: Canvas Pixel Perfect and sprites with Point filter and no compression."));
            let mut preset = None;
            egui::ComboBox::from_id_salt("res_presets").selected_text(crate::i18n::t("Common resolutions…")).width((ui.available_width() - 30.0).max(100.0)).show_ui(ui, |ui| {
                for (name, w, h) in SIZE_PRESETS {
                    if ui.button(format!("{}  ·  {} × {}", crate::i18n::t(name), w, h)).clicked() {
                        preset = Some((*w, *h));
                    }
                }
            });
            if let Some((w, h)) = preset {
                self.resize_page(w, h);
            }
            swatch_row(ui, &mut self.doc.background);
            hint(ui, &crate::i18n::t("Alpha 0 = transparent overlay: the page shows a checkerboard and no background image is exported to Unity."));
        });
        self.canvas_unity_settings(ui);
        self.unity_export_settings(ui);
        self.portrait_settings(ui);
        component(ui, "unity_export", &crate::i18n::t("Unity project"), true, |ui| {
            if self.settings.unity_project.is_empty() {
                hint(ui, &crate::i18n::t("No project selected yet. “Export to Unity” will ask for the folder (the one containing Assets)."));
            } else {
                ui.label(RichText::new(&self.settings.unity_project).small().monospace());
            }
            ui.horizontal(|ui| {
                if ui.add(crate::icons::btn("export", crate::i18n::t("Choose project…"))).clicked() {
                    self.export_to_unity(true);
                }
                if ui.add(crate::icons::btn("code", crate::i18n::t("View code…"))).clicked() {
                    self.export_open = true;
                }
            });
        });
        component(ui, "shortcuts", &crate::i18n::t("Shortcuts"), false, |ui| {
            for (k, d) in [
                ("Click / Shift+click", "Select / add"),
                ("Ctrl+click", "Select inside a group"),
                ("Esc", "Select the parent"),
                ("Drag on empty space", "Area selection"),
                ("Double-click", "Edit text"),
                ("Corners", "Scale (Shift: free)"),
                ("Rotation handle", "Rotate (Shift: 15°)"),
                ("Alt+drag", "Duplicate and move"),
                ("Ctrl+G / Ctrl+Shift+G", "Group / ungroup"),
                ("Ctrl+D / Del", "Duplicate / delete"),
                ("Ctrl+Z / Ctrl+Y", "Undo / redo"),
                ("Arrows", "Move 1 px (Shift: 10)"),
                ("Ctrl+wheel", "Zoom"),
                ("Ctrl+S", "Save project"),
                ("Ctrl+E", "Export to Unity"),
                ("F5", "Test / simulate screens"),
                ("F1", "Tutorial"),
            ] {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(crate::i18n::t(k)).monospace().size(11.0));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| ui.label(RichText::new(crate::i18n::t(d)).size(11.0).color(MUTED())));
                });
            }
        });
    }

    /// Unity Canvas: render mode, sorting order, safe area, embedding.
    fn canvas_unity_settings(&mut self, ui: &mut egui::Ui) {
        let t = crate::i18n::t;
        component(ui, "canvas_unity", &t("Canvas (Unity)"), true, |ui| {
            let c = &mut self.doc.canvas;
            egui::Grid::new("canvas_unity_grid").num_columns(2).spacing([8.0, 5.0]).show(ui, |ui| {
                ui.label(t("Render mode"));
                let label = |m: CanvasRenderMode| match m {
                    CanvasRenderMode::ScreenSpaceOverlay => t("Screen Space - Overlay"),
                    CanvasRenderMode::ScreenSpaceCamera => t("Screen Space - Camera"),
                    CanvasRenderMode::WorldSpace => t("World Space"),
                };
                egui::ComboBox::from_id_salt("render_mode").selected_text(label(c.render_mode)).width(170.0).show_ui(ui, |ui| {
                    for m in [CanvasRenderMode::ScreenSpaceOverlay, CanvasRenderMode::ScreenSpaceCamera, CanvasRenderMode::WorldSpace] {
                        ui.selectable_value(&mut c.render_mode, m, label(m));
                    }
                });
                ui.end_row();
                ui.label(t("Sort order"));
                ui.add(egui::DragValue::new(&mut c.sort_order).range(docext::SORT_MIN..=docext::SORT_MAX).speed(1.0));
                ui.end_row();
                if c.render_mode != CanvasRenderMode::ScreenSpaceOverlay {
                    ui.label(t("Plane distance"));
                    ui.add(egui::DragValue::new(&mut c.plane_distance).range(0.01..=100_000.0).speed(0.5));
                    ui.end_row();
                }
            });
            // checkboxes live outside the Grid so their labels wrap inside the panel
            ui.add_space(2.0);
            ui.label(RichText::new(t("Safe area")).size(11.5).color(MUTED()));
            ui.checkbox(&mut c.safe_area, t("Respect notch and rounded corners")).on_hover_text(t("Applies the device safe area to the root, so nothing hides under a notch."));
            ui.label(RichText::new(t("Embed")).size(11.5).color(MUTED()));
            ui.checkbox(&mut c.embed, t("Embed (no own Canvas)")).on_hover_text(t("Exports a plain RectTransform meant to live under the Canvas of your game: no Canvas or CanvasScaler is created."));
        });
    }

    /// Unity export options: scripts folder, assembly, outputs, fonts.
    fn unity_export_settings(&mut self, ui: &mut egui::Ui) {
        let t = crate::i18n::t;
        let red = Color32::from_rgb(235, 80, 90);
        component(ui, "unity_opts", &t("Unity export"), true, |ui| {
            // Text is validated before it reaches the design: while it is invalid it lives only in the field.
            let u = &mut self.doc.unity;
            let (dir_id, asm_id) = (ui.scope_id().with("unity_scripts_dir_buf"), ui.scope_id().with("unity_assembly_buf"));
            let mut dir = ui.data(|d| d.get_temp::<String>(dir_id)).unwrap_or_else(|| u.scripts_dir.clone().unwrap_or_default());
            ui.label(RichText::new(t("Scripts folder")).size(11.5).color(MUTED()));
            let r = ui.add(egui::TextEdit::singleline(&mut dir).desired_width(ui.available_width()).char_limit(260).hint_text("Assets/EditorGUI/Scripts"));
            let dir_ok = dir.is_empty() || docext::valid_scripts_dir(&dir);
            if r.changed() && dir_ok {
                u.scripts_dir = if dir.is_empty() { None } else { Some(dir.clone()) };
            }
            if r.has_focus() || (!dir_ok && r.changed()) {
                ui.data_mut(|d| d.insert_temp(dir_id, dir.clone()));
            } else {
                ui.data_mut(|d| d.remove::<String>(dir_id));
            }
            if !dir_ok {
                ui.label(RichText::new(t("Invalid folder: it must start with Assets, use / and contain no “..”, no drive, no empty parts, none of <>\"|?*, no trailing dot or space and no reserved names (CON, NUL, COM1…).")).small().color(red));
            } else {
                hint(ui, &t("Where the generated scripts go (empty = the default folder)."));
            }
            let mut asm = ui.data(|d| d.get_temp::<String>(asm_id)).unwrap_or_else(|| u.assembly.clone().unwrap_or_default());
            ui.add_space(4.0);
            ui.label(RichText::new(t("Assembly name")).size(11.5).color(MUTED()));
            let r = ui.add(egui::TextEdit::singleline(&mut asm).desired_width(ui.available_width()).char_limit(128).hint_text(t("Assembly Definition of your game (optional)")));
            let asm_ok = asm.is_empty() || docext::valid_assembly(&asm);
            if r.changed() && asm_ok {
                u.assembly = if asm.is_empty() { None } else { Some(asm.clone()) };
            }
            if r.has_focus() || (!asm_ok && r.changed()) {
                ui.data_mut(|d| d.insert_temp(asm_id, asm.clone()));
            } else {
                ui.data_mut(|d| d.remove::<String>(asm_id));
            }
            if !asm_ok {
                ui.label(RichText::new(t("Invalid assembly name: letters, digits, _ . - and it must start with a letter or _.")).small().color(red));
            }
            ui.add_space(6.0);
            ui.label(RichText::new(t("Outputs")).size(11.5).color(MUTED()));
            ui.checkbox(&mut u.generate.prefab, t("Prefab"));
            ui.checkbox(&mut u.generate.uitk, t("UI Toolkit (UXML / USS)"));
            let mut imgui = u.generate.imgui_enabled();
            if ui.checkbox(&mut imgui, t("IMGUI script")).changed() {
                u.generate.imgui = Some(imgui);
            }
            ui.checkbox(&mut u.generate.view_scripts, t("View scripts"));
            ui.add_space(6.0);
            ui.label(RichText::new(t("Default font (TextMeshPro asset)")).size(11.5).color(MUTED()));
            let mut font = u.default_font.clone().unwrap_or_default();
            let font_ok = docext::valid_font_name(&font);
            if ui.add(egui::TextEdit::singleline(&mut font).desired_width(ui.available_width()).char_limit(128).hint_text(t("Name of a font asset in your project"))).changed() && docext::valid_font_name(&font) {
                u.default_font = if font.trim().is_empty() { None } else { Some(font) };
            }
            if !font_ok {
                ui.label(RichText::new(t("Invalid font name: it must not contain / \\ : or ..")).small().color(red));
            }
            ui.label(RichText::new(t("Fallback fonts")).size(11.5).color(MUTED()));
            let mut remove = None;
            for (i, f) in u.fallback_fonts.iter().enumerate() {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(f).monospace().size(11.5));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if crate::icons::button(ui, "trash", "x", "Remove").clicked() {
                            remove = Some(i);
                        }
                    });
                });
            }
            if let Some(i) = remove {
                u.fallback_fonts.remove(i);
            }
            if u.fallback_fonts.len() < 16 {
                let new_id = ui.scope_id().with("unity_fallback_new");
                let mut new_font = ui.data(|d| d.get_temp::<String>(new_id)).unwrap_or_default();
                let mut add = false;
                // button first (right-aligned), the field takes what is left: never wider than the panel
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    add = ui.add_enabled(!new_font.trim().is_empty() && docext::valid_font_name(&new_font), crate::icons::btn("plus", t("Add")).small()).clicked();
                    let r = ui.add(egui::TextEdit::singleline(&mut new_font).desired_width((ui.available_width() - 4.0).max(40.0)).char_limit(128).hint_text(t("Fallback font asset name")));
                    add = add || (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) && !new_font.trim().is_empty());
                });
                if add {
                    u.fallback_fonts.push(new_font.trim().to_string());
                    new_font.clear();
                }
                ui.data_mut(|d| d.insert_temp(new_id, new_font));
            }
        });
    }

    /// Portrait layout: create / size / remove (the view toggle is in the header).
    fn portrait_settings(&mut self, ui: &mut egui::Ui) {
        let t = crate::i18n::t;
        let portrait_view = self.orientation() == Orientation::Portrait;
        let current = self.real_doc().portrait.as_ref().map(|p| (p.width, p.height, p.overrides.len()));
        component(ui, "portrait_layout", &t("Portrait layout"), current.is_some(), |ui| {
            match current {
                None => {
                    hint(ui, &t("No portrait layout yet. Create one to design the same screen for devices held upright; elements start where they are in landscape."));
                    if ui.add(crate::icons::btn("orient_portrait", t("Create portrait layout"))).clicked() {
                        self.create_portrait_layout();
                    }
                }
                Some((w, h, n)) => {
                    let (mut nw, mut nh) = (w, h);
                    egui::Grid::new("portrait_size").num_columns(2).spacing([8.0, 5.0]).show(ui, |ui| {
                        ui.label(t("Portrait size"));
                        ui.horizontal(|ui| {
                            ui.add(egui::DragValue::new(&mut nw).range(64.0..=8192.0).suffix(" px"));
                            ui.label("×");
                            ui.add(egui::DragValue::new(&mut nh).range(64.0..=8192.0).suffix(" px"));
                        });
                        ui.end_row();
                    });
                    if (nw.round() != w || nh.round() != h) && nw >= 64.0 && nh >= 64.0 {
                        self.set_portrait_size(nw.round(), nh.round());
                    }
                    hint(ui, &crate::i18n::tf("{} element(s) differ from the landscape layout. Use the Landscape / Portrait buttons in the top bar to switch the view.", &[&n.to_string()]));
                    if !portrait_view && ui.add(egui::Button::new(t("Remove portrait layout")).small()).on_hover_text(t("Deletes every portrait override and the portrait size")).clicked() {
                        self.edit_full(|d| d.portrait = None);
                    }
                }
            }
        });
    }

    fn arrange(&mut self, ui: &mut egui::Ui, multi: bool) {
        component(ui, "arrange", if multi { "Align and order" } else { "Align to parent and order" }, true, |ui| {
            ui.columns(3, |c| {
                let b = |ui: &mut egui::Ui, t: &str| ui.add_sized([ui.available_width(), 22.0], egui::Button::new(t)).clicked();
                if b(&mut c[0], "Left") {
                    self.align(0, 0)
                }
                if b(&mut c[1], "Center") {
                    self.align(0, 1)
                }
                if b(&mut c[2], "Right") {
                    self.align(0, 2)
                }
                if b(&mut c[0], "Top") {
                    self.align(1, 0)
                }
                if b(&mut c[1], "Middle") {
                    self.align(1, 1)
                }
                if b(&mut c[2], "Bottom") {
                    self.align(1, 2)
                }
            });
            if self.sel.len() >= 3 {
                ui.columns(2, |c| {
                    if c[0].button(crate::i18n::t("Distribute horiz.")).clicked() {
                        self.distribute(0);
                    }
                    if c[1].button(crate::i18n::t("Distribute vert.")).clicked() {
                        self.distribute(1);
                    }
                });
            }
            ui.columns(4, |c| {
                if c[0].button(crate::i18n::t("Backward")).on_hover_text(crate::i18n::t("One place earlier among siblings (Ctrl+[)")).clicked() {
                    self.reorder(None, -1);
                }
                if c[1].button(crate::i18n::t("Forward")).on_hover_text(crate::i18n::t("Ctrl+]")).clicked() {
                    self.reorder(None, 1);
                }
                if c[2].button(crate::i18n::t("To back")).clicked() {
                    self.reorder(Some(false), 0);
                }
                if c[3].button(crate::i18n::t("To front")).clicked() {
                    self.reorder(Some(true), 0);
                }
            });
        });
    }

    fn multi_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(crate::i18n::tf("Elements selected: {}", &[&self.sel.len().to_string()])).color(MUTED()));
        self.arrange(ui, true);
        ui.add_space(6.0);
        if ui.button(crate::i18n::t("Group into Container (Ctrl+G)")).clicked() {
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
            if crate::icons::toggle(ui, !e.visible, if e.visible { "eye" } else { "eye_off" }, "o", "Active / inactive (GameObject.SetActive)").clicked() {
                e.visible = !e.visible;
            }
            ui.add(egui::TextEdit::singleline(&mut e.name).desired_width(ui.available_width() - 30.0));
            if crate::icons::toggle(ui, e.locked, if e.locked { "lock" } else { "unlock" }, "L", "Lock in editor").clicked() {
                e.locked = !e.locked;
            }
        });
        ui.horizontal(|ui| {
            let (r, _) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
            crate::icons::paint_or(ui, crate::icons::kind_name(e.kind), r, ACCENT(), e.kind.icon());
            egui::ComboBox::from_id_salt("kind").selected_text(crate::i18n::t(e.kind.label())).width(150.0).show_ui(ui, |ui| {
                for k in model::ALL_KINDS {
                    ui.selectable_value(&mut e.kind, k, crate::i18n::t(k.label()));
                }
            });
            ui.label(RichText::new(e.kind.unity_component()).small().color(MUTED()));
        });
        ui.label(RichText::new(&path).small().monospace().color(MUTED())).on_hover_text(crate::i18n::t("Path inside the Canvas in Unity"));

        // Unity name: exact GameObject name (empty = automatic, from the name above)
        let collisions = crate::export::unity_name_collisions(&self.doc);
        let mut unity_name = e.unity_name.clone().unwrap_or_default();
        ui.horizontal(|ui| {
            ui.label(RichText::new(crate::i18n::t("Unity name")).size(11.5).color(MUTED()));
            ui.add(egui::TextEdit::singleline(&mut unity_name).desired_width(ui.available_width()).char_limit(256).hint_text(crate::i18n::t("Automatic (same as the name)")))
                .on_hover_text(crate::i18n::t("Exact GameObject name in Unity. Leave it empty to use the name above."));
        });
        e.unity_name = if unity_name.is_empty() { None } else { Some(unity_name.clone()) };
        let red = Color32::from_rgb(235, 80, 90);
        if !unity_name.is_empty() && !docext::valid_unity_name(&unity_name) {
            ui.label(RichText::new(crate::i18n::t("Invalid Unity name: it must not be blank, contain “/” or control characters, or exceed 128 characters (the name above is used instead).")).small().color(red));
        } else if collisions.contains(&id) {
            let used = names.get(&id).cloned().unwrap_or_default();
            ui.label(RichText::new(crate::i18n::tf("A sibling already uses this name: it will be exported as “{}”.", &[&used])).small().color(red));
        } else if let Some(n) = names.get(&id) {
            ui.label(RichText::new(crate::i18n::tf("Exported as “{}”", &[n])).small().color(MUTED()));
        }

        // portrait view: this element differs from the landscape layout
        if let Some(ov) = self.portrait_override(id).cloned() {
            let t = crate::i18n::t;
            let mut parts: Vec<String> = Vec::new();
            for (on, name) in [
                (ov.x.is_some() || ov.y.is_some(), "position"),
                (ov.w.is_some() || ov.h.is_some(), "size"),
                (ov.rot.is_some(), "rotation"),
                (ov.anchor_min.is_some() || ov.anchor_max.is_some() || ov.pivot.is_some(), "anchors"),
                (ov.visible.is_some(), "visibility"),
                (ov.font_size.is_some(), "font size"),
                (ov.layout.is_some(), "layout"),
            ] {
                if on {
                    parts.push(t(name));
                }
            }
            let mut reset = false;
            egui::Frame::new().fill(ACCENT_SOFT()).corner_radius(4.0).inner_margin(Margin::same(5)).show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(RichText::new(crate::i18n::tf("Differs from landscape: {}", &[&parts.join(", ")])).small());
                    if ui.add(crate::icons::btn("override", t("Reset to landscape")).small()).clicked() {
                        reset = true;
                    }
                });
            });
            if reset {
                self.reset_to_landscape(&[id]);
                return;
            }
        }

        // RectTransform
        let mut preset_click: Option<([f32; 2], [f32; 2], bool, bool)> = None;
        component(ui, "rt", &crate::i18n::t("Rect Transform"), true, |ui| {
            if driven {
                egui::Frame::new().fill(ACCENT_SOFT()).corner_radius(4.0).inner_margin(Margin::same(5)).show(ui, |ui| {
                    ui.label(RichText::new(crate::i18n::tf("Some values are controlled by the Layout Group of “{}”.", &[&parent_name.clone().unwrap_or_default()])).small());
                });
            }
            ui.horizontal(|ui| {
                preset_click = anchor_presets(ui, &e);
                ui.vertical(|ui| {
                    hint(ui, &crate::i18n::t("Click: anchors"));
                    hint(ui, &crate::i18n::t("Shift: + pivot"));
                    hint(ui, &crate::i18n::t("Alt: + position"));
                    ui.label(RichText::new(crate::i18n::tf("Parent: {}", &[&parent_name.clone().unwrap_or_else(|| "Canvas".into())])).small().color(MUTED()));
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
            egui::CollapsingHeader::new(RichText::new(crate::i18n::t("Canvas size (px)")).small()).id_salt("world").show(ui, |ui| {
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
                    if drag_f(ui, "Width", &mut w, 1.0) {
                        world.cx += (w.max(1.0) - world.w) / 2.0;
                        world.w = w.max(1.0);
                    }
                    if drag_f(ui, "Height", &mut h, 1.0) {
                        world.cy += (h.max(1.0) - world.h) / 2.0;
                        world.h = h.max(1.0);
                    }
                    ui.end_row();
                    drag_f(ui, "Rotation", &mut world.rot, 0.5);
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
                        ui.label(crate::i18n::t("Gradient to"));
                        rgba_edit(ui, &mut e.fill2);
                        ui.end_row();
                    }
                    if e.kind == Kind::Image {
                        ui.label(crate::i18n::t("Sprite"));
                        ui.horizontal(|ui| {
                            if ui.button(crate::i18n::t(if e.image.is_empty() { "Choose image…" } else { "Change…" })).clicked() {
                                pick_image = true;
                            }
                            if !e.image.is_empty() && ui.small_button(crate::i18n::t("Remove")).clicked() {
                                e.image.clear();
                            }
                        });
                        ui.end_row();
                    }
                    if e.kind != Kind::Ellipse {
                        ui.label(crate::i18n::t("Corner radius"));
                        ui.add(egui::DragValue::new(&mut e.radius).range(0.0..=500.0).speed(0.5).suffix(" px")).on_hover_text(crate::i18n::t("Exported as a generated 9-slice Sprite"));
                        ui.end_row();
                    }
                    ui.label(crate::i18n::t("Border"));
                    ui.horizontal(|ui| {
                        rgba_edit(ui, &mut e.stroke);
                        ui.add(egui::DragValue::new(&mut e.stroke_w).range(0.0..=60.0).speed(0.2).suffix(" px"));
                    });
                    ui.end_row();
                    ui.label(crate::i18n::t("Opacity"));
                    ui.add(egui::Slider::new(&mut e.opacity, 0.0..=1.0));
                    ui.end_row();
                    ui.label(crate::i18n::t("Raycast Target"));
                    ui.checkbox(&mut e.raycast, crate::i18n::t(""));
                    ui.end_row();
                });
                swatch_row(ui, &mut e.fill);
            });
        }

        // Unity image settings: 9-slice, pixels per unit, sprite, image type
        if matches!(e.kind, Kind::Image | Kind::Panel) && !e.image.is_empty() {
            component(ui, "unity_image", &crate::i18n::t("Image settings (Unity)"), true, |ui| {
                unity_image_settings(ui, &mut e);
            });
        }

        // Text
        if e.kind.has_text() {
            let title = if e.kind == Kind::Dropdown { "TMP_Dropdown · options" } else { "TextMeshPro · text" };
            component(ui, "txt", title, true, |ui| {
                if e.kind == Kind::Dropdown {
                    // While typing, the text is stored as is (with the empty line after pressing Enter);
                    // the options are taken from it without the empty lines.
                    #[allow(deprecated)]
                    let buf_id = egui::Id::new(("dd_opts", e.id));
                    let mut opts = ui.data(|d| d.get_temp::<String>(buf_id)).unwrap_or_else(|| e.options.join("\n"));
                    hint(ui, &crate::i18n::t("One option per line"));
                    let r = ui.add(egui::TextEdit::multiline(&mut opts).desired_rows(3).desired_width(ui.available_width()));
                    if r.changed() {
                        e.options = opts.lines().map(|l| l.to_string()).filter(|l| !l.trim().is_empty()).collect();
                    }
                    if r.has_focus() {
                        ui.data_mut(|d| d.insert_temp(buf_id, opts));
                    } else {
                        ui.data_mut(|d| d.remove::<String>(buf_id));
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
                        hint(ui, &crate::i18n::t("Placeholder text"));
                    }
                    ui.add(egui::TextEdit::multiline(&mut e.text).desired_rows(2).desired_width(ui.available_width()));
                }
                egui::Grid::new("txtgrid").num_columns(2).spacing([8.0, 5.0]).show(ui, |ui| {
                    ui.label(crate::i18n::t("Size"));
                    ui.add(egui::DragValue::new(&mut e.font_size).range(4.0..=400.0).speed(0.5));
                    ui.end_row();
                    ui.label(crate::i18n::t("Color"));
                    rgba_edit(ui, &mut e.text_color);
                    ui.end_row();
                    ui.label(crate::i18n::t("Style"));
                    ui.horizontal(|ui| {
                        ui.toggle_value(&mut e.bold, RichText::new(crate::i18n::t("B")).strong());
                        ui.toggle_value(&mut e.italic, RichText::new(crate::i18n::t("I")).italics());
                        ui.toggle_value(&mut e.underline, RichText::new(crate::i18n::t("U")).underline());
                    });
                    ui.end_row();
                    ui.label(crate::i18n::t("Alignment"));
                    ui.horizontal(|ui| {
                        for (i, t) in ["align_left", "align_center", "align_right"].iter().enumerate() {
                            if crate::icons::toggle(ui, e.align == i as u8, t, "-", ["Left", "Center", "Right"][i]).clicked() {
                                e.align = i as u8;
                            }
                        }
                        if e.kind != Kind::Text {
                            ui.add_space(4.0);
                            for (i, t) in ["align_top", "align_middle", "align_bottom"].iter().enumerate() {
                                if crate::icons::toggle(ui, e.valign == i as u8, t, "|", ["Top", "Middle", "Bottom"][i]).clicked() {
                                    e.valign = i as u8;
                                }
                            }
                        }
                    });
                    ui.end_row();
                    ui.label(crate::i18n::t("Font"));
                    let font_label = if e.font.is_empty() { crate::i18n::t("Default") } else { e.font.clone() };
                    egui::ComboBox::from_id_salt("font_pick_insp").selected_text(font_label).width(150.0).show_ui(ui, |ui| {
                        ui.selectable_value(&mut e.font, String::new(), crate::i18n::t("Default"));
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
                    ui.label(crate::i18n::t("Checkbox color"));
                    rgba_edit(ui, &mut e.fill);
                });
                ui.checkbox(&mut e.interactable, crate::i18n::t("Interactable"));
            }),
            Kind::Slider => component(ui, "sld", &crate::i18n::t("Slider"), true, |ui| {
                ui.add(egui::Slider::new(&mut e.value, 0.0..=1.0).text(crate::i18n::t("Value")));
                ui.horizontal(|ui| {
                    ui.label(crate::i18n::t("Padding"));
                    rgba_edit(ui, &mut e.fill);
                    ui.label(crate::i18n::t("Hint"));
                    rgba_edit(ui, &mut e.fill2);
                });
                ui.checkbox(&mut e.interactable, crate::i18n::t("Interactable")).on_hover_text(crate::i18n::t("Turn it off for health or progress bars"));
            }),
            k if k.selectable() => component(ui, "sel", &crate::i18n::t("Selectable"), true, |ui| {
                ui.checkbox(&mut e.interactable, crate::i18n::t("Interactable"));
            }),
            Kind::ScrollView => component(ui, "scroll", &crate::i18n::t("Scroll Rect"), true, |ui| {
                hint(ui, &crate::i18n::t("Vertical scrolling. Children go into Content (Viewport/Content); its height grows so they all fit."));
            }),
            _ => {}
        }

        // Mask + Shadow
        component(ui, "effects", &crate::i18n::t("Mask and effects"), e.mask || e.shadow.on, |ui| {
            if e.kind != Kind::ScrollView {
                ui.checkbox(&mut e.mask, crate::i18n::t("Rect Mask 2D (clips children)"));
            }
            if e.kind.has_fill() {
                ui.checkbox(&mut e.shadow.on, crate::i18n::t("Shadow"));
                if e.shadow.on {
                    ui.horizontal(|ui| {
                        rgba_edit(ui, &mut e.shadow.color);
                        ui.label(crate::i18n::t("Distance"));
                        ui.add(egui::DragValue::new(&mut e.shadow.offset[0]).speed(0.5).prefix("x "));
                        ui.add(egui::DragValue::new(&mut e.shadow.offset[1]).speed(0.5).prefix("y "));
                    });
                }
            }
        });

        // Layout group
        component(ui, "layout", &crate::i18n::t("Layout Group"), e.layout.mode != LayoutMode::None, |ui| {
            egui::ComboBox::from_id_salt("lmode").selected_text(crate::i18n::t(e.layout.mode.label())).width(200.0).show_ui(ui, |ui| {
                for m in [LayoutMode::None, LayoutMode::Horizontal, LayoutMode::Vertical, LayoutMode::Grid] {
                    ui.selectable_value(&mut e.layout.mode, m, crate::i18n::t(m.label()));
                }
            });
            if e.layout.mode == LayoutMode::None {
                hint(ui, &crate::i18n::t("Automatically arranges direct children, as in Unity."));
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
                ui.checkbox(&mut e.layout.expand_w, crate::i18n::t("Control + Expand width"));
                ui.checkbox(&mut e.layout.expand_h, crate::i18n::t("Control + Expand height"));
            }
        });

        // click code
        let mut run_now = false;
        let mut go_test = false;
        component(ui, "code", &crate::i18n::t("On click (test code)"), !e.on_click.trim().is_empty(), |ui| {
            hint(ui, &crate::i18n::t("Runs in Test. When exported, it becomes a comment in the OnClick method of the View script."));
            ui.add(egui::TextEdit::multiline(&mut e.on_click).code_editor().desired_rows(4).desired_width(ui.available_width()).hint_text(crate::i18n::t("message(\"Hello!\");\nhide(\"Window\");")));
            ui.horizontal(|ui| {
                if ui.add_enabled(!e.on_click.trim().is_empty(), crate::icons::btn("play", crate::i18n::t("Run now"))).clicked() {
                    run_now = true;
                }
                if ui.button(crate::i18n::t("Test design")).on_hover_text(crate::i18n::t("F5")).clicked() {
                    go_test = true;
                }
            });
            egui::CollapsingHeader::new(RichText::new(crate::i18n::t("Available functions")).small()).id_salt("code_help").show(ui, |ui| {
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
            let file = rfd::FileDialog::new().set_title("Choose image").add_filter("Images", crate::library::IMAGE_EXTS).pick_file();
            if let Some(f) = file {
                match self.library.add_image(&f) {
                    Ok(name) => {
                        if let Some(el) = self.doc.get_mut(id) {
                            el.image = name;
                        }
                        self.status = crate::i18n::t("Image changed");
                    }
                    Err(err) => self.status = crate::i18n::tf("Could not use the image: {}", &[&err.to_string()]),
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
                        if ui.small_button(crate::i18n::t("Snap")).on_hover_text(crate::i18n::t("Ctrl+0")).clicked() {
                            self.fit_pending = true;
                        }
                        ui.label(RichText::new(format!("{:.0}%", self.zoom * 100.0)).size(11.5));
                        let mut z = self.zoom;
                        if ui.add(egui::Slider::new(&mut z, 0.05..=4.0).logarithmic(true).show_value(false)).changed() {
                            self.zoom = z;
                        }
                        ui.separator();
                        ui.label(RichText::new(format!("{} {} · {} {}", self.doc.elements.len(), crate::i18n::t("elements"), self.sel.len(), crate::i18n::t("selected"))).size(11.5).color(MUTED()));
                        ui.separator();
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            ui.add(egui::Label::new(RichText::new(crate::i18n::t(&self.status)).size(11.5).color(MUTED())).truncate());
                        });
                    });
                });
            });
    }

    // ------------------------------------------------------------- import warnings window
    pub fn import_window(&mut self, ctx: &egui::Context) {
        if self.import_warnings.is_empty() {
            return;
        }
        let mut open = true;
        egui::Window::new(crate::i18n::t("Import from Unity")).open(&mut open).collapsible(false).default_width(480.0).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            ui.label(crate::i18n::tf("{} things could not be mapped exactly:", &[&self.import_warnings.len().to_string()]));
            egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                for w in &self.import_warnings {
                    ui.label(RichText::new(format!("• {}", crate::unity_import::clean_msg(w))).small().color(MUTED()));
                }
            });
        });
        if !open {
            self.import_warnings.clear();
        }
    }

    // ------------------------------------------------------------- save window
    pub fn save_window(&mut self, ctx: &egui::Context) {
        if !self.save_open {
            return;
        }
        let mut open = true;
        let mut close = false;
        egui::Window::new(crate::i18n::t("Save")).open(&mut open).collapsible(false).resizable(false).default_width(380.0).anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0]).show(ctx, |ui| {
            ui.label(RichText::new(crate::i18n::t("As project")).strong());
            hint(ui, &crate::i18n::t("A file you can reopen and keep editing."));
            ui.horizontal(|ui| {
                if ui.add(crate::icons::btn("save", crate::i18n::t("Save project"))).on_hover_text(self.recovery.bound.as_deref().unwrap_or("")).clicked() {
                    self.save_current();
                    close = true;
                }
                if ui.button(crate::i18n::t("Save as…")).clicked() {
                    self.save_project_as();
                    close = true;
                }
            });
            ui.separator();
            ui.label(RichText::new(crate::i18n::t("As template")).strong());
            hint(ui, &crate::i18n::t("It will appear in Templates › My templates to start new designs."));
            if self.template_name.is_empty() {
                self.template_name = self.doc.title.clone();
            }
            ui.horizontal(|ui| {
                ui.label(crate::i18n::t("Name"));
                ui.add(egui::TextEdit::singleline(&mut self.template_name).desired_width(200.0));
            });
            if ui.add(crate::icons::btn("tab_templates", crate::i18n::t("Save as template"))).clicked() {
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
        egui::Window::new(crate::i18n::t("Export to Unity")).open(&mut open).default_size([760.0, 600.0]).collapsible(false).show(ctx, |ui| {
            egui::Frame::new().fill(PANEL_DARK()).corner_radius(6.0).inner_margin(Margin::same(8)).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(crate::i18n::t("Project:")).strong());
                    let shown = if self.settings.unity_project.is_empty() { "(not chosen)".to_string() } else { self.settings.unity_project.clone() };
                    ui.label(RichText::new(shown).monospace().size(11.5));
                });
                ui.horizontal(|ui| {
                    if ui.add(crate::icons::btn_tint("export", RichText::new(crate::i18n::t("Export everything to project")).strong().color(Color32::WHITE), Color32::WHITE).fill(ACCENT())).clicked() {
                        self.export_to_unity(false);
                    }
                    if ui.button(crate::i18n::t("Change project…")).clicked() {
                        self.export_to_unity(true);
                    }
                });
                hint(
                    ui,
                    &format!(
                        "Writes to Assets/EditorGUI: Editor/EditorGuiImporter.cs, Designs/{c}.json, Sprites, Fonts, UI Toolkit/{c}.uxml+.uss and Scripts/{c}OnGUI.cs, {c}ViewBase.cs, {c}View.cs. When you return to Unity, Prefabs/{c}.prefab (uGUI + TextMeshPro) is generated automatically.",
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
                0 => "Design for the importer. Unity turns it into a prefab (Tools › Lienzo UI).",
                1 | 2 => "Uses a UIDocument with PanelSettings set to “Scale With Screen Size” and this reference resolution.",
                3 => "Standalone component: draws the interface with UnityEngine.GUI inside OnGUI (no Canvas).",
                4 => "Typed references and onClick/onValueChanged events. Inherit in <Class>View for your logic.",
                _ => "Goes in Assets/EditorGUI/Editor/. Builds the prefab with uGUI + TextMeshPro.",
            };
            hint(ui, help);
            ui.horizontal(|ui| {
                if ui.add(crate::icons::btn("duplicate", crate::i18n::t("Copy"))).clicked() {
                    ui.ctx().copy_text(text.clone());
                    self.status = crate::i18n::t("Copied to the clipboard");
                }
                if ui.add(crate::icons::btn("save", crate::i18n::t("Save file…"))).clicked() {
                    self.save_text_file(&crate::i18n::t("Save file"), &file, ext, &text);
                }
                ui.label(RichText::new(format!("{} lines", text.lines().count())).small().color(MUTED()));
            });
            egui::ScrollArea::both().max_height(ui.available_height() - 6.0).auto_shrink([false, false]).show(ui, |ui| {
                let mut t = text;
                ui.add(egui::TextEdit::multiline(&mut t).code_editor().desired_width(ui.available_width()));
            });
        });
        if !open {
            self.export_open = false;
        }
    }
}

/// 3×3 child alignment picker (Unity TextAnchor).
/// 9-slice borders (left, bottom, right, top in source pixels), pixels per unit, sub-sprite, image type, preserve aspect.
fn unity_image_settings(ui: &mut egui::Ui, e: &mut Element) {
    let t = crate::i18n::t;
    let dims = render::image_dims(&e.image);
    let (mw, mh) = dims.map(|(w, h)| (w as f32, h as f32)).unwrap_or((100_000.0, 100_000.0));
    ui.label(RichText::new(t("9-slice")).strong().size(11.5)).on_hover_text(t("Border of the 9-slice in pixels of the source image (left, bottom, right, top), like Unity's Sprite Editor. The canvas shows the result."));
    let mut sl = e.slice.unwrap_or([0.0; 4]);
    let before = sl;
    egui::Grid::new("slicegrid").num_columns(4).spacing([6.0, 4.0]).show(ui, |ui| {
        for (row, (a, b)) in [(0usize, 1usize), (2, 3)].into_iter().enumerate() {
            let _ = row;
            for i in [a, b] {
                let (name, max) = [("Left", mw), ("Bottom", mh), ("Right", mw), ("Top", mh)][i];
                ui.label(RichText::new(t(name)).size(11.5).color(MUTED()));
                ui.add(egui::DragValue::new(&mut sl[i]).range(0.0..=max).speed(0.5).max_decimals(0));
            }
            ui.end_row();
        }
    });
    if sl != before {
        e.slice = if sl.iter().all(|v| *v == 0.0) { None } else { Some(sl) };
    }
    ui.horizontal(|ui| {
        if ui.add_enabled(e.slice.is_some(), egui::Button::new(t("Clear")).small()).clicked() {
            e.slice = None;
        }
        if let Some((w, h)) = dims {
            ui.label(RichText::new(crate::i18n::tf("Source image: {} × {} px", &[&w.to_string(), &h.to_string()])).small().color(MUTED()));
        }
    });
    // source picture with the slice lines
    if let (Some((w, h)), Some(tex)) = (dims, render::texture(ui.ctx(), &e.image)) {
        let max = Vec2::new(ui.available_width().min(240.0), 110.0);
        let s = (max.x / w as f32).min(max.y / h as f32);
        let (r, _) = ui.allocate_exact_size(Vec2::new(w as f32 * s, h as f32 * s), Sense::hover());
        ui.painter().rect_filled(r, 2.0, FIELD());
        ui.painter().image(tex.id(), r, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        if let Some(sl) = e.slice {
            let st = Stroke::new(1.0, ACCENT());
            let x = |v: f32| r.left() + (v / w as f32).clamp(0.0, 1.0) * r.width();
            let y = |v: f32| r.top() + (v / h as f32).clamp(0.0, 1.0) * r.height();
            ui.painter().line_segment([Pos2::new(x(sl[0]), r.top()), Pos2::new(x(sl[0]), r.bottom())], st);
            ui.painter().line_segment([Pos2::new(x(w as f32 - sl[2]), r.top()), Pos2::new(x(w as f32 - sl[2]), r.bottom())], st);
            ui.painter().line_segment([Pos2::new(r.left(), y(sl[3])), Pos2::new(r.right(), y(sl[3]))], st);
            ui.painter().line_segment([Pos2::new(r.left(), y(h as f32 - sl[1])), Pos2::new(r.right(), y(h as f32 - sl[1]))], st);
        }
    }
    ui.add_space(4.0);
    egui::Grid::new("unityimggrid").num_columns(2).spacing([8.0, 5.0]).show(ui, |ui| {
        ui.label(t("Pixels per unit"));
        ui.horizontal(|ui| {
            let mut ppu = e.ppu.unwrap_or(100.0);
            if ui.add(egui::DragValue::new(&mut ppu).range(0.01..=10_000.0).speed(1.0).max_decimals(2)).on_hover_text(t("Sprite pixels per unit: 100 keeps the borders at their pixel size, 200 makes them half as big.")).changed() {
                e.ppu = Some(ppu);
            }
            if ui.add_enabled(e.ppu.is_some(), egui::Button::new(t("Auto")).small()).clicked() {
                e.ppu = None;
            }
        });
        ui.end_row();
        ui.label(t("Sub-sprite"));
        let mut sprite = e.sprite.clone().unwrap_or_default();
        ui.add(egui::TextEdit::singleline(&mut sprite).desired_width(150.0).char_limit(256).hint_text(t("Name inside an atlas (optional)")));
        e.sprite = if sprite.is_empty() { None } else { Some(sprite) };
        ui.end_row();
        ui.label(t("Image mode"));
        let shown = match e.image_mode {
            Some(m) => t(m.as_str()),
            None => format!("{} ({})", t("Automatic"), t(e.effective_image_mode().as_str())),
        };
        egui::ComboBox::from_id_salt("image_mode").selected_text(shown).width(150.0).show_ui(ui, |ui| {
            ui.selectable_value(&mut e.image_mode, None, t("Automatic"));
            for m in [ImageMode::Simple, ImageMode::Sliced, ImageMode::Tiled, ImageMode::Filled] {
                ui.selectable_value(&mut e.image_mode, Some(m), t(m.as_str()));
            }
        });
        ui.end_row();
        ui.label(t("Preserve aspect"));
        ui.checkbox(&mut e.preserve_aspect, "");
        ui.end_row();
    });
}

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
            // simplified puzzle piece
            p.rect_stroke(Rect::from_min_max(Pos2::new(r.left() + 2.0, r.top() + 5.0), Pos2::new(r.right() - 5.0, r.bottom() - 2.0)), 3.0, s, StrokeKind::Inside);
            p.circle_stroke(Pos2::new(r.center().x - 1.5, r.top() + 4.0), 3.0, s);
            p.circle_stroke(Pos2::new(r.right() - 3.5, r.center().y + 1.5), 3.0, s);
        }
        Tab::Community => {
            // two people
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
