//! The page canvas: pan/zoom, rulers, hit testing through the hierarchy,
//! selection, move/resize/rotate with smart guides (children follow their
//! anchors), marquee selection, masks, inline text editing, test mode with
//! simulated screens.

use crate::app::{Drag, DragKind, EditorApp, Guide, SelFrame, SCREEN_PRESETS};
use crate::model::{Doc, Element, Frame, Kind, norm_angle};
use crate::render::{self, View, Xf};
use crate::theme;
use eframe::egui::{self, Align2, Color32, CursorIcon, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Vec2};

#[allow(non_snake_case)]
pub fn SEL() -> Color32 {
    theme::ACCENT()
}
#[allow(non_snake_case)]
pub fn GUIDE() -> Color32 {
    theme::GUIDE()
}
const HANDLE_R: f32 = 6.5;
const ROT_OFFSET: f32 = 34.0;
const SNAP_PX: f32 = 7.0;
const RULER: f32 = 20.0;

#[derive(Clone, Copy, PartialEq, Debug)]
enum Hit {
    Handle(i8, i8),
    Rotate,
}

fn frame_xf(f: &SelFrame, v: &View) -> Xf {
    let (sin, cos) = f.rot.to_radians().sin_cos();
    Xf { c: v.to_screen(f.cx, f.cy), sin, cos, z: v.zoom }
}

fn handles(f: &SelFrame, v: &View) -> Vec<(Hit, Pos2)> {
    let xf = frame_xf(f, v);
    let (hw, hh) = (f.w / 2.0, f.h / 2.0);
    let mut out = Vec::new();
    let sw = f.w * v.zoom;
    let sh = f.h * v.zoom;
    for (sx, sy) in [(-1, -1), (1, -1), (1, 1), (-1, 1)] {
        out.push((Hit::Handle(sx, sy), xf.map(sx as f32 * hw, sy as f32 * hh)));
    }
    if sh > 36.0 {
        out.push((Hit::Handle(-1, 0), xf.map(-hw, 0.0)));
        out.push((Hit::Handle(1, 0), xf.map(hw, 0.0)));
    }
    if sw > 36.0 {
        out.push((Hit::Handle(0, -1), xf.map(0.0, -hh)));
        out.push((Hit::Handle(0, 1), xf.map(0.0, hh)));
    }
    let bottom = xf.map(0.0, hh);
    out.push((Hit::Rotate, bottom + xf.dir(0.0, 1.0) * ROT_OFFSET));
    out
}

fn resize_cursor(dir: Vec2) -> CursorIcon {
    let mut a = dir.y.atan2(dir.x).to_degrees();
    if a < 0.0 {
        a += 180.0;
    }
    let a = a % 180.0;
    if !(22.5..157.5).contains(&a) {
        CursorIcon::ResizeHorizontal
    } else if a < 67.5 {
        CursorIcon::ResizeNwSe
    } else if a < 112.5 {
        CursorIcon::ResizeVertical
    } else {
        CursorIcon::ResizeNeSw
    }
}

/// Screen clip rectangle for an element (page ∩ masked ancestors).
fn clip_rect(doc: &Doc, e: &Element, v: &View, page: Rect) -> Rect {
    match doc.clip_of(e.id) {
        Some(c) => page.intersect(Rect::from_min_max(v.to_screen(c[0], c[1]), v.to_screen(c[2], c[3]))),
        None => page,
    }
}

/// Paint every visible element of `doc` (hierarchy order, masks applied).
fn paint_doc(painter: &egui::Painter, doc: &Doc, v: &View, page: Rect, hide_text_of: Option<u64>) {
    for e in &doc.elements {
        if !doc.shown(e.id) {
            continue;
        }
        let clip = clip_rect(doc, e, v, page);
        if clip.width() <= 0.0 || clip.height() <= 0.0 {
            continue;
        }
        render::paint_element(&painter.with_clip_rect(clip), e, v, hide_text_of == Some(e.id));
    }
}

/// Topmost element under a page point (Contenedores only when nothing else is there).
fn hit_doc(doc: &Doc, p: [f32; 2], tol: f32, filter: impl Fn(&Element) -> bool) -> Option<u64> {
    let ok = |e: &Element| {
        doc.shown(e.id)
            && e.contains(p[0], p[1], tol)
            && doc.clip_of(e.id).map(|c| p[0] >= c[0] && p[0] <= c[2] && p[1] >= c[1] && p[1] <= c[3]).unwrap_or(true)
            && filter(e)
    };
    doc.elements
        .iter()
        .rev()
        .find(|e| e.kind != Kind::Empty && ok(e))
        .or_else(|| doc.elements.iter().rev().find(|e| e.kind == Kind::Empty && ok(e)))
        .map(|e| e.id)
}

fn nice_step(zoom: f32) -> f32 {
    let target = 70.0 / zoom.max(1e-3);
    let mut step = 1.0;
    loop {
        for m in [1.0, 2.0, 5.0] {
            if step * m >= target {
                return step * m;
            }
        }
        step *= 10.0;
    }
}

impl EditorApp {
    pub fn view(&self) -> View {
        let size = Vec2::new(self.doc.width, self.doc.height) * self.zoom;
        View { origin: self.canvas_rect.center() + self.pan - size / 2.0 + Vec2::new(RULER, RULER) * 0.5, zoom: self.zoom }
    }

    pub fn fit(&mut self) {
        let r = self.canvas_rect;
        if r.width() < 10.0 {
            return;
        }
        self.zoom = ((r.width() - 90.0 - RULER) / self.doc.width).min((r.height() - 110.0 - RULER) / self.doc.height).clamp(0.05, 4.0);
        self.pan = Vec2::new(0.0, -6.0);
    }

    fn hit_element(&self, p: [f32; 2]) -> Option<u64> {
        hit_doc(&self.doc, p, 2.0 / self.zoom, |_| true)
    }

    pub fn canvas_ui(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(ui.available_size(), Sense::click_and_drag());
        self.canvas_rect = rect;
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, theme::BACKDROP());

        if self.preview {
            self.preview_ui(ui, rect, &response, &painter);
            return;
        }
        if self.fit_pending {
            self.fit();
            self.fit_pending = false;
        }

        // ---------------------------------------------------------- zoom/pan
        if response.contains_pointer() {
            let (scroll, zoom_delta, mouse) = ui.input(|i| (i.smooth_scroll_delta, i.zoom_delta(), i.pointer.hover_pos()));
            if zoom_delta != 1.0 {
                if let Some(m) = mouse {
                    let v = self.view();
                    let p = v.to_page(m);
                    self.zoom = (self.zoom * zoom_delta).clamp(0.05, 4.0);
                    let v2 = self.view();
                    let now = v2.to_screen(p[0], p[1]);
                    self.pan += m - now;
                }
            } else if scroll != Vec2::ZERO {
                self.pan += scroll;
            }
        }

        let v = self.view();
        let page = Rect::from_min_size(v.origin, Vec2::new(self.doc.width, self.doc.height) * v.zoom);

        // page with soft shadow
        for i in 1..=6 {
            let a = 10 - i;
            painter.rect_filled(page.expand(i as f32 * 1.5).translate(Vec2::new(0.0, 2.0)), 4.0 + i as f32, Color32::from_black_alpha(a as u8 * 4));
        }
        painter.rect_filled(page, 0.0, render::col(self.doc.background, 1.0));
        paint_doc(&painter, &self.doc, &v, page.intersect(rect), self.editing);
        // parts outside the page are shown faded while selected
        for e in self.doc.elements.iter().filter(|e| self.sel.contains(&e.id)) {
            let a = e.aabb();
            if a[0] < 0.0 || a[1] < 0.0 || a[2] > self.doc.width || a[3] > self.doc.height {
                let mut ghost = e.clone();
                ghost.opacity *= 0.35;
                render::paint_element(&painter, &ghost, &v, false);
            }
        }
        // Contenedores (empty RectTransforms) as faint dashed boxes
        for e in self.doc.elements.iter().filter(|e| e.kind == Kind::Empty && self.doc.shown(e.id) && !self.sel.contains(&e.id)) {
            dashed_outline(&painter, e, &v, Stroke::new(1.0, Color32::from_rgba_unmultiplied(120, 170, 255, 90)));
        }

        // -------------------------------------------------------- grid
        if self.grid.show {
            let g = self.grid;
            let col = render::col(g.color, 1.0);
            let gp = painter.with_clip_rect(page.intersect(rect));
            if g.w * v.zoom >= 4.0 {
                let mut x = g.w;
                while x < self.doc.width {
                    let sx = v.to_screen(x, 0.0).x;
                    gp.line_segment([Pos2::new(sx, page.top()), Pos2::new(sx, page.bottom())], Stroke::new(1.0, col));
                    x += g.w;
                }
            }
            if g.h * v.zoom >= 4.0 {
                let mut y = g.h;
                while y < self.doc.height {
                    let sy = v.to_screen(0.0, y).y;
                    gp.line_segment([Pos2::new(page.left(), sy), Pos2::new(page.right(), sy)], Stroke::new(1.0, col));
                    y += g.h;
                }
            }
        }

        // badges for elements that run code on click
        for e in self.doc.elements.iter().filter(|e| !e.on_click.trim().is_empty() && self.doc.shown(e.id)) {
            let a = e.aabb();
            let p = v.to_screen(a[2], a[1]);
            if rect.contains(p) {
                painter.circle_filled(p, 8.0, theme::WARN());
                painter.circle_stroke(p, 8.0, Stroke::new(1.0, Color32::from_rgb(120, 80, 0)));
                crate::icons::paint_or(ui, "bolt", Rect::from_center_size(p, Vec2::splat(11.0)), Color32::from_rgb(60, 40, 0), "!");
            }
        }

        // -------------------------------------------------------- interaction
        let (pointer, pressed, down, mods, double) = ui.input(|i| {
            (
                i.pointer.interact_pos().or(i.pointer.hover_pos()),
                i.pointer.primary_pressed(),
                i.pointer.primary_down(),
                i.modifiers,
                i.pointer.button_double_clicked(egui::PointerButton::Primary),
            )
        });
        let middle_pressed = ui.input(|i| i.pointer.button_pressed(egui::PointerButton::Middle));
        let middle_down = ui.input(|i| i.pointer.button_down(egui::PointerButton::Middle));
        let space = ui.input(|i| i.key_down(egui::Key::Space)) && ui.memory(|m| m.focused().is_none());
        let in_rulers = pointer.map(|m| self.rulers && (m.x < rect.left() + RULER || m.y < rect.top() + RULER)).unwrap_or(false);
        let over = response.contains_pointer() && !in_rulers;

        self.hover = None;
        let frame = self.sel_frame();
        let mut hit_handle = None;
        if let (Some(m), Some(f)) = (pointer, frame) {
            if self.drag.is_none() && self.editing.is_none() {
                let all_locked = self.selected().iter().all(|e| e.locked);
                if !all_locked {
                    for (h, p) in handles(&f, &v) {
                        if p.distance(m) <= HANDLE_R + 4.0 {
                            hit_handle = Some(h);
                        }
                    }
                }
            }
        }
        if let Some(m) = pointer {
            if over && self.drag.is_none() {
                self.hover = self.hit_element(v.to_page(m)).map(|h| self.click_target(h, mods.command));
            }
        }

        // double click: edit text, or enter a Contenedor
        if double && over && self.drag.is_none() {
            if let Some(m) = pointer {
                if let Some(id) = self.hit_element(v.to_page(m)) {
                    if let Some(e) = self.doc.get(id) {
                        if e.kind.has_text() && e.kind != Kind::Dropdown && !e.locked {
                            self.sel = vec![id];
                            self.editing = Some(id);
                            self.editing_new = true;
                        } else {
                            self.sel = vec![id];
                        }
                    }
                }
            }
        }

        // press: decide what the drag will do
        if over && (pressed || middle_pressed) && self.drag.is_none() {
            if let Some(m) = pointer {
                let pg = v.to_page(m);
                let mut d = Drag {
                    kind: DragKind::Marquee,
                    start_screen: m,
                    start_page: pg,
                    cur_page: pg,
                    orig: self.doc.elements.clone(),
                    frame: frame.unwrap_or(SelFrame { cx: 0.0, cy: 0.0, w: 0.0, h: 0.0, rot: 0.0 }),
                    moved: false,
                    click_select: None,
                    start_pan: self.pan,
                };
                if middle_pressed || space {
                    d.kind = DragKind::Pan;
                } else if let Some(h) = hit_handle {
                    d.kind = match h {
                        Hit::Handle(sx, sy) => DragKind::Resize(sx, sy),
                        Hit::Rotate => DragKind::Rotate,
                    };
                } else {
                    if let Some(ed) = self.editing {
                        if self.hit_element(pg) != Some(ed) {
                            self.editing = None;
                        }
                    }
                    match self.hit_element(pg) {
                        Some(id) if self.editing == Some(id) => {
                            d.kind = DragKind::Pan;
                            d.moved = true; // inert: the text box handles the click
                        }
                        Some(hit) => {
                            let target = self.click_target(hit, mods.command);
                            let locked = self.doc.get(target).map(|e| e.locked).unwrap_or(false);
                            let in_sel = self.sel.contains(&hit) || self.doc.ancestors(hit).iter().any(|a| self.sel.contains(a));
                            if mods.shift {
                                self.toggle_select(target);
                                d.kind = if self.is_selected(target) && !locked { DragKind::Move } else { DragKind::Marquee };
                                d.moved = !self.is_selected(target);
                            } else if locked {
                                d.kind = DragKind::Marquee;
                                d.click_select = Some(target);
                            } else {
                                if in_sel && !mods.command {
                                    // keep the selection to drag it; a plain click dives in
                                    d.click_select = Some(self.click_target(hit, false));
                                } else {
                                    self.select_only(target);
                                }
                                d.kind = DragKind::Move;
                                if mods.alt {
                                    self.duplicate_selection(0.0);
                                    d.click_select = None;
                                }
                            }
                        }
                        None => {
                            if !mods.shift {
                                self.sel.clear();
                            }
                        }
                    }
                    d.orig = self.doc.elements.clone();
                    d.frame = self.sel_frame().unwrap_or(d.frame);
                }
                self.drag = Some(d);
            }
        }

        // while dragging
        self.guides.clear();
        self.drag_label = None;
        if let (Some(mut d), Some(m)) = (self.drag.take(), pointer) {
            let still_down = down || middle_down;
            let pg = v.to_page(m);
            d.cur_page = pg;
            if m.distance(d.start_screen) > 3.0 {
                d.moved = true;
            }
            if d.moved {
                match d.kind {
                    DragKind::Pan => {
                        if self.editing.is_none() {
                            self.pan = d.start_pan + (m - d.start_screen);
                        }
                    }
                    DragKind::Move => self.drag_move(&d, pg, mods.alt),
                    DragKind::Resize(sx, sy) => self.drag_resize(&d, pg, sx, sy, mods.shift, mods.alt),
                    DragKind::Rotate => self.drag_rotate(&d, pg, mods.shift),
                    DragKind::Marquee => {}
                }
            }
            if still_down {
                self.drag = Some(d);
            } else {
                match d.kind {
                    DragKind::Marquee => {
                        if d.moved {
                            let (x0, x1) = (d.start_page[0].min(pg[0]), d.start_page[0].max(pg[0]));
                            let (y0, y1) = (d.start_page[1].min(pg[1]), d.start_page[1].max(pg[1]));
                            // pick the outermost elements fully inside the area
                            let inside: Vec<u64> = self
                                .doc
                                .elements
                                .iter()
                                .filter(|e| {
                                    let a = e.aabb();
                                    self.doc.shown(e.id) && !e.locked && a[0] >= x0 && a[2] <= x1 && a[1] >= y0 && a[3] <= y1
                                })
                                .map(|e| e.id)
                                .collect();
                            let mut ids = self.doc.top_level(&inside);
                            if mods.shift {
                                for id in self.sel.clone() {
                                    if !ids.contains(&id) {
                                        ids.push(id);
                                    }
                                }
                            }
                            self.sel = ids;
                        } else if let Some(id) = d.click_select {
                            self.sel = vec![id];
                            self.status = crate::i18n::t("Elemento bloqueado: desbloquéalo para moverlo");
                        }
                    }
                    DragKind::Move => {
                        if !d.moved {
                            if let Some(id) = d.click_select {
                                self.sel = vec![id];
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else if !(down || middle_down) {
            self.drag = None;
        }

        // ------------------------------------------------------- overlays
        let v = self.view();
        if let Some(h) = self.hover {
            if !self.is_selected(h) && self.drag.is_none() {
                if let Some(e) = self.doc.get(h) {
                    outline(&painter, e, &v, Stroke::new(1.5, SEL()));
                }
            }
        }
        for g in &self.guides {
            let (a, b) = if g.vertical { (v.to_screen(g.pos, g.from), v.to_screen(g.pos, g.to)) } else { (v.to_screen(g.from, g.pos), v.to_screen(g.to, g.pos)) };
            painter.line_segment([a, b], Stroke::new(1.2, GUIDE()));
        }
        if let Some(d) = &self.drag {
            if d.kind == DragKind::Marquee && d.moved {
                let r = Rect::from_two_pos(v.to_screen(d.start_page[0], d.start_page[1]), v.to_screen(d.cur_page[0], d.cur_page[1]));
                painter.rect_filled(r, 0.0, SEL().gamma_multiply(0.12));
                painter.rect_stroke(r, 0.0, Stroke::new(1.0, SEL()), StrokeKind::Inside);
            }
        }
        let moving = matches!(self.drag.as_ref().map(|d| (d.kind, d.moved)), Some((DragKind::Move, true)));
        if let Some(f) = self.sel_frame() {
            let multi = self.sel.len() > 1;
            if !multi {
                // parent rectangle (what the anchors refer to)
                if let Some(p) = self.selected().first().and_then(|e| self.doc.parent_of(e.id)).and_then(|p| self.doc.get(p)) {
                    dashed_outline(&painter, p, &v, Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 200, 80, 170)));
                }
            }
            if multi {
                for e in self.selected() {
                    outline(&painter, e, &v, Stroke::new(1.0, SEL().gamma_multiply(0.7)));
                }
            }
            let xf = frame_xf(&f, &v);
            let (hw, hh) = (f.w / 2.0, f.h / 2.0);
            let pts: Vec<Pos2> = [[-hw, -hh], [hw, -hh], [hw, hh], [-hw, hh]].iter().map(|q| xf.map(q[0], q[1])).collect();
            let all_locked = self.selected().iter().all(|e| e.locked);
            painter.add(Shape::closed_line(pts, Stroke::new(1.6, SEL())));
            if all_locked {
                let tl = xf.map(hw, -hh);
                crate::icons::paint_or(ui, "lock", Rect::from_min_size(tl + Vec2::new(6.0, -20.0), Vec2::splat(15.0)), SEL(), "L");
            } else if !moving && self.editing.is_none() {
                for (h, p) in handles(&f, &v) {
                    match h {
                        Hit::Rotate => {
                            painter.circle_filled(p, 10.0, Color32::WHITE);
                            painter.circle_stroke(p, 10.0, Stroke::new(1.0, SEL()));
                            let pts: Vec<Pos2> = (0..14)
                                .map(|i| {
                                    let a = (40.0 + i as f32 * 20.0f32).to_radians();
                                    p + Vec2::new(a.cos(), a.sin()) * 4.5
                                })
                                .collect();
                            let end = *pts.last().unwrap();
                            painter.add(Shape::line(pts, Stroke::new(1.5, Color32::from_gray(50))));
                            painter.add(Shape::convex_polygon(vec![end + Vec2::new(-3.0, -3.5), end + Vec2::new(3.5, -1.0), end + Vec2::new(-1.0, 2.5)], Color32::from_gray(50), Stroke::NONE));
                        }
                        Hit::Handle(sx, sy) if sx != 0 && sy != 0 => {
                            let r = Rect::from_center_size(p, Vec2::splat(HANDLE_R * 1.6));
                            painter.rect_filled(r, 2.0, Color32::WHITE);
                            painter.rect_stroke(r, 2.0, Stroke::new(1.2, SEL()), StrokeKind::Inside);
                        }
                        Hit::Handle(sx, _) => {
                            let along = if sx != 0 { xf.dir(0.0, 1.0) } else { xf.dir(1.0, 0.0) };
                            let a = p - along * 7.0;
                            let b = p + along * 7.0;
                            painter.line_segment([a, b], Stroke::new(6.0, SEL()));
                            painter.line_segment([a, b], Stroke::new(4.0, Color32::WHITE));
                        }
                    }
                }
                if !multi {
                    if let Some(e) = self.selected().first() {
                        self.paint_anchors(&painter, e, &v);
                    }
                }
            }
        }
        if let (Some(label), Some(m)) = (&self.drag_label, pointer) {
            let galley = painter.layout_no_wrap(label.clone(), FontId::proportional(13.0), Color32::WHITE);
            let r = Rect::from_min_size(m + Vec2::new(16.0, 18.0), galley.size() + Vec2::new(14.0, 8.0));
            painter.rect_filled(r, 6.0, Color32::from_rgb(20, 20, 26));
            painter.galley(r.min + Vec2::new(7.0, 4.0), galley, Color32::WHITE);
        }

        // cursor
        if let Some(d) = &self.drag {
            match d.kind {
                DragKind::Move if d.moved => ui.ctx().set_cursor_icon(CursorIcon::Grabbing),
                DragKind::Pan | DragKind::Rotate => ui.ctx().set_cursor_icon(CursorIcon::Grabbing),
                DragKind::Resize(sx, sy) => ui.ctx().set_cursor_icon(resize_cursor(frame_xf(&d.frame, &v).dir(sx as f32 * d.frame.w.max(1.0), sy as f32 * d.frame.h.max(1.0)))),
                _ => {}
            }
        } else if over {
            if space {
                ui.ctx().set_cursor_icon(CursorIcon::Grab);
            } else if let (Some(h), Some(f)) = (hit_handle, frame) {
                match h {
                    Hit::Rotate => ui.ctx().set_cursor_icon(CursorIcon::Alias),
                    Hit::Handle(sx, sy) => ui.ctx().set_cursor_icon(resize_cursor(frame_xf(&f, &v).dir(sx as f32 * f.w, sy as f32 * f.h))),
                }
            } else if let Some(h) = self.hover {
                if self.doc.get(h).map(|e| !e.locked).unwrap_or(false) {
                    ui.ctx().set_cursor_icon(CursorIcon::Move);
                }
            }
        }

        if self.rulers {
            self.paint_rulers(&painter, rect, &v, pointer);
        }

        self.text_editor(ui, &v);
        if self.drag.is_none() && self.editing.is_none() {
            self.quick_actions(ui, &v);
        }

        // ------------------------------------------------------- drop from side panel
        if let Some(items) = self.dnd.clone() {
            let (m, released) = ui.input(|i| (i.pointer.hover_pos(), i.pointer.any_released()));
            if let Some(m) = m {
                let fg = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::unique("dnd_ghost")));
                let b = crate::app::bounds(&items.iter().collect::<Vec<_>>());
                let size = Vec2::new(b[2] - b[0], b[3] - b[1]) * v.zoom;
                if rect.contains(m) {
                    if let Some(c) = self.container_at(v.to_page(m)).and_then(|c| self.doc.get(c)) {
                        outline(&painter, c, &v, Stroke::new(2.0, theme::OK()));
                        painter.text(v.to_screen(c.aabb()[0], c.aabb()[1]) + Vec2::new(0.0, -4.0), Align2::LEFT_BOTTOM, format!("Soltar dentro de «{}»", c.name), FontId::proportional(12.0), theme::OK());
                    }
                }
                for it in &items {
                    let mut ghost = it.clone();
                    ghost.opacity *= 0.75;
                    ghost.x -= b[0];
                    ghost.y -= b[1];
                    render::paint_element(&fg, &ghost, &View { origin: m - size / 2.0, zoom: v.zoom }, false);
                }
                if released {
                    if rect.contains(m) {
                        let p = v.to_page(m);
                        if items.len() == 1 {
                            self.insert(items[0].clone(), Some(p));
                        } else {
                            self.insert_items(&items, Some(p));
                        }
                    }
                    self.dnd = None;
                }
            } else if released {
                self.dnd = None;
            }
        }
    }

    fn paint_rulers(&self, painter: &egui::Painter, rect: Rect, v: &View, pointer: Option<Pos2>) {
        let top = Rect::from_min_max(rect.min, Pos2::new(rect.right(), rect.top() + RULER));
        let left = Rect::from_min_max(rect.min, Pos2::new(rect.left() + RULER, rect.bottom()));
        painter.rect_filled(top, 0.0, theme::PANEL_DARK());
        painter.rect_filled(left, 0.0, theme::PANEL_DARK());
        // selection extents
        if let Some(e) = self.selected().first() {
            let b = crate::app::bounds(&self.selected());
            let _ = e;
            let (a, z) = (v.to_screen(b[0], b[1]), v.to_screen(b[2], b[3]));
            painter.rect_filled(Rect::from_min_max(Pos2::new(a.x.max(top.left() + RULER), top.top()), Pos2::new(z.x, top.bottom())), 0.0, theme::ACCENT_SOFT());
            painter.rect_filled(Rect::from_min_max(Pos2::new(left.left(), a.y.max(left.top() + RULER)), Pos2::new(left.right(), z.y)), 0.0, theme::ACCENT_SOFT());
        }
        let step = nice_step(v.zoom);
        let minor = step / 5.0;
        let tick = Stroke::new(1.0, theme::MUTED().gamma_multiply(0.6));
        let font = FontId::proportional(9.5);
        // horizontal
        let p0 = v.to_page(Pos2::new(top.left() + RULER, 0.0))[0];
        let p1 = v.to_page(Pos2::new(top.right(), 0.0))[0];
        let mut x = (p0 / minor).floor() * minor;
        while x <= p1 {
            let sx = v.to_screen(x, 0.0).x;
            if sx >= top.left() + RULER {
                let major = (x / step).round() * step == x || ((x / step) - (x / step).round()).abs() < 1e-3;
                let h = if major { RULER * 0.6 } else { RULER * 0.25 };
                painter.line_segment([Pos2::new(sx, top.bottom() - h), Pos2::new(sx, top.bottom())], tick);
                if major {
                    painter.text(Pos2::new(sx + 3.0, top.top() + 1.0), Align2::LEFT_TOP, format!("{}", x.round() as i64), font.clone(), theme::MUTED());
                }
            }
            x += minor;
        }
        // vertical
        let q0 = v.to_page(Pos2::new(0.0, left.top() + RULER))[1];
        let q1 = v.to_page(Pos2::new(0.0, left.bottom()))[1];
        let mut y = (q0 / minor).floor() * minor;
        while y <= q1 {
            let sy = v.to_screen(0.0, y).y;
            if sy >= left.top() + RULER {
                let major = ((y / step) - (y / step).round()).abs() < 1e-3;
                let w = if major { RULER * 0.6 } else { RULER * 0.25 };
                painter.line_segment([Pos2::new(left.right() - w, sy), Pos2::new(left.right(), sy)], tick);
                if major {
                    let g = painter.layout_no_wrap(format!("{}", y.round() as i64), font.clone(), theme::MUTED());
                    let pos = Pos2::new(left.left() + 2.0, sy + 3.0 + g.size().x);
                    painter.add(egui::epaint::TextShape::new(pos, g, theme::MUTED()).with_angle(-std::f32::consts::FRAC_PI_2));
                }
            }
            y += minor;
        }
        // pointer markers
        if let Some(m) = pointer {
            if rect.contains(m) {
                painter.line_segment([Pos2::new(m.x, top.top()), Pos2::new(m.x, top.bottom())], Stroke::new(1.0, theme::ACCENT()));
                painter.line_segment([Pos2::new(left.left(), m.y), Pos2::new(left.right(), m.y)], Stroke::new(1.0, theme::ACCENT()));
            }
        }
        painter.line_segment([Pos2::new(top.left(), top.bottom()), Pos2::new(top.right(), top.bottom())], Stroke::new(1.0, theme::BORDER()));
        painter.line_segment([Pos2::new(left.right(), left.top()), Pos2::new(left.right(), left.bottom())], Stroke::new(1.0, theme::BORDER()));
        painter.rect_filled(Rect::from_min_size(rect.min, Vec2::splat(RULER)), 0.0, theme::PANEL());
    }

    /// Test mode: the design as a player would see it on the chosen screen. Clicking
    /// runs the element's code (toggles switch, sliders and dropdowns respond).
    fn preview_ui(&mut self, ui: &mut egui::Ui, rect: Rect, response: &egui::Response, painter: &egui::Painter) {
        let (sim, scale) = match self.preview_screen {
            Some(s) => self.doc.simulate_screen(s),
            None => (self.doc.clone(), 1.0),
        };
        let zoom = ((rect.width() - 80.0) / sim.width).min((rect.height() - 130.0) / sim.height).clamp(0.02, 4.0);
        let size = Vec2::new(sim.width, sim.height) * zoom;
        let origin = rect.center() - size / 2.0 + Vec2::new(0.0, 22.0);
        let v = View { origin, zoom };
        let page = Rect::from_min_size(origin, size);
        // device bezel
        painter.rect_filled(page.expand(14.0), 18.0, Color32::from_rgb(12, 12, 16));
        painter.rect_stroke(page.expand(14.0), 18.0, Stroke::new(1.0, theme::LINE()), StrokeKind::Inside);
        painter.rect_filled(page, 0.0, render::col(sim.background, 1.0));
        paint_doc(painter, &sim, &v, page.intersect(rect), None);

        let pointer = ui.input(|i| i.pointer.hover_pos()).filter(|_| response.contains_pointer());
        let interactive = |e: &Element| e.interactable && (!e.on_click.trim().is_empty() || matches!(e.kind, Kind::Toggle | Kind::Slider | Kind::Dropdown));
        let target = pointer.filter(|m| page.contains(*m)).and_then(|m| hit_doc(&sim, v.to_page(m), 0.0, interactive));
        if let Some(id) = target {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            if let Some(e) = sim.get(id) {
                outline(painter, e, &v, Stroke::new(2.0, theme::WARN()));
            }
            let clicked = response.clicked();
            let slider_drag = response.dragged() || clicked;
            if let (Some(e), Some(m)) = (sim.get(id).cloned(), pointer) {
                if e.kind == Kind::Slider && slider_drag {
                    let l = e.frame().to_local(v.to_page(m));
                    let k = e.h * 0.5;
                    let t = ((l[0] + e.w / 2.0 - k) / (e.w - 2.0 * k).max(1.0)).clamp(0.0, 1.0);
                    if let Some(de) = self.doc.get_mut(id) {
                        de.value = t;
                    }
                }
                if clicked {
                    if let Some(de) = self.doc.get_mut(id) {
                        match de.kind {
                            Kind::Toggle => de.value = if de.value > 0.5 { 0.0 } else { 1.0 },
                            Kind::Dropdown if !de.options.is_empty() => de.value = ((de.value.round() as usize + 1) % de.options.len()) as f32,
                            _ => {}
                        }
                    }
                    self.run_click(id);
                }
            }
        }
        // banner
        let banner = "MODO PRUEBA · clic en los elementos con código · Esc o F5 para volver a editar";
        let galley = painter.layout_no_wrap(banner.into(), FontId::proportional(13.0), Color32::WHITE);
        let r = Rect::from_center_size(Pos2::new(rect.center().x, rect.top() + 20.0), galley.size() + Vec2::new(24.0, 10.0));
        painter.rect_filled(r, 14.0, theme::OK().gamma_multiply(0.9));
        painter.galley(r.min + Vec2::new(12.0, 5.0), galley, Color32::WHITE);
        let screen = self.preview_screen.unwrap_or([self.doc.width, self.doc.height]);
        let info = format!(
            "Pantalla {} × {}  ·  lienzo virtual {} × {}  ·  escala del CanvasScaler {:.2}",
            screen[0], screen[1], sim.width.round(), sim.height.round(), scale
        );
        painter.text(Pos2::new(rect.center().x, rect.top() + 44.0), Align2::CENTER_CENTER, info, FontId::proportional(12.0), theme::MUTED());
        // console
        let lines: Vec<&String> = self.console.iter().rev().take(6).collect();
        if !lines.is_empty() {
            let w = (rect.width() - 40.0).min(640.0);
            let h = 22.0 + lines.len() as f32 * 17.0;
            let r = Rect::from_min_size(Pos2::new(rect.left() + 20.0, rect.bottom() - h - 16.0), Vec2::new(w, h));
            painter.rect_filled(r, 8.0, Color32::from_rgba_unmultiplied(14, 15, 20, 235));
            painter.text(r.min + Vec2::new(10.0, 6.0), Align2::LEFT_TOP, "Consola", FontId::proportional(11.0), theme::MUTED());
            for (i, l) in lines.iter().rev().enumerate() {
                let c = if l.starts_with('⚠') { Color32::from_rgb(255, 140, 120) } else { Color32::from_rgb(220, 240, 220) };
                painter.text(r.min + Vec2::new(10.0, 22.0 + i as f32 * 17.0), Align2::LEFT_TOP, l.as_str(), FontId::monospace(12.0), c);
            }
        }
        let _ = SCREEN_PRESETS;
    }

    /// Anchors (relative to the parent rectangle) and pivot of one element.
    fn paint_anchors(&self, painter: &egui::Painter, e: &Element, v: &View) {
        let pf = self.doc.parent_frame(e.id);
        let col = Color32::from_white_alpha(230);
        let edge = Color32::from_rgb(40, 40, 50);
        let xs = [e.anchor_min[0], e.anchor_max[0]];
        let ys = [e.anchor_max[1], e.anchor_min[1]]; // top, bottom (y up)
        let pxf = |ax: f32, ay: f32| {
            let w = pf.to_world([(ax - 0.5) * pf.w, (0.5 - ay) * pf.h]);
            v.to_screen(w[0], w[1])
        };
        let (s, c) = pf.rot.to_radians().sin_cos();
        let rot = |d: Vec2| Vec2::new(d.x * c - d.y * s, d.x * s + d.y * c);
        for (ix, ax) in xs.iter().enumerate() {
            for (iy, ay) in ys.iter().enumerate() {
                let p = pxf(*ax, *ay);
                let dx = if ix == 0 { -1.0 } else { 1.0 };
                let dy = if iy == 0 { -1.0 } else { 1.0 };
                let a = p + rot(Vec2::new(dx * 10.0, dy * 3.0));
                let b = p + rot(Vec2::new(dx * 3.0, dy * 10.0));
                painter.add(Shape::convex_polygon(vec![p, a, b], col, Stroke::new(1.0, edge)));
            }
        }
        let xf = Xf::of(e, v);
        let piv = xf.map((e.pivot[0] - 0.5) * e.w, (0.5 - e.pivot[1]) * e.h);
        painter.circle_stroke(piv, 5.0, Stroke::new(2.0, Color32::from_rgb(0, 170, 255)));
        painter.circle_filled(piv, 2.0, Color32::from_rgb(0, 170, 255));
    }

    // --------------------------------------------------------------- snapping

    /// Lines the moving box can snap to: page edges/centre and other elements' edges/centres.
    fn snap_targets(&self, d: &Drag) -> (Vec<(f32, [f32; 2])>, Vec<(f32, [f32; 2])>) {
        let (w, h) = (self.doc.width, self.doc.height);
        let mut xs = vec![(0.0, [0.0, h]), (w / 2.0, [0.0, h]), (w, [0.0, h])];
        let mut ys = vec![(0.0, [0.0, w]), (h / 2.0, [0.0, w]), (h, [0.0, w])];
        let moving: Vec<u64> = self.sel_top().iter().flat_map(|t| self.doc.subtree(*t)).collect();
        for e in d.orig.iter().filter(|e| e.visible && !moving.contains(&e.id) && e.kind != Kind::Empty) {
            let a = e.aabb();
            if a[0] <= 0.5 && a[1] <= 0.5 && a[2] >= w - 0.5 && a[3] >= h - 0.5 {
                continue;
            }
            for x in [a[0], (a[0] + a[2]) / 2.0, a[2]] {
                xs.push((x, [a[1], a[3]]));
            }
            for y in [a[1], (a[1] + a[3]) / 2.0, a[3]] {
                ys.push((y, [a[0], a[2]]));
            }
        }
        (xs, ys)
    }

    fn snap_axis(&self, lines: [f32; 3], targets: &[(f32, [f32; 2])]) -> Option<(f32, f32, [f32; 2])> {
        let thr = SNAP_PX / self.zoom;
        let mut best: Option<(f32, f32, [f32; 2])> = None;
        for l in lines {
            for (t, span) in targets {
                let delta = t - l;
                if delta.abs() <= thr && best.map(|b| delta.abs() < b.0.abs()).unwrap_or(true) {
                    best = Some((delta, *t, *span));
                }
            }
        }
        best
    }

    /// Restart from the document as it was when the drag began.
    fn restore(&mut self, d: &Drag) {
        self.doc.elements = d.orig.clone();
    }

    fn movable_tops(&self) -> Vec<u64> {
        self.sel_top().into_iter().filter(|id| self.doc.get(*id).map(|e| !e.locked).unwrap_or(false)).collect()
    }

    fn drag_move(&mut self, d: &Drag, pg: [f32; 2], no_snap: bool) {
        self.restore(d);
        let tops = self.movable_tops();
        if tops.is_empty() {
            return;
        }
        let mut dx = pg[0] - d.start_page[0];
        let mut dy = pg[1] - d.start_page[1];
        let b = crate::app::bounds(&tops.iter().filter_map(|i| self.doc.get(*i)).collect::<Vec<_>>());
        if self.grid.snap && !no_snap {
            dx = (((b[0] + dx) / self.grid.w).round() * self.grid.w) - b[0];
            dy = (((b[1] + dy) / self.grid.h).round() * self.grid.h) - b[1];
        } else if !no_snap {
            let (xs, ys) = self.snap_targets(d);
            let mb = [b[0] + dx, b[1] + dy, b[2] + dx, b[3] + dy];
            if let Some((delta, x, span)) = self.snap_axis([mb[0], (mb[0] + mb[2]) / 2.0, mb[2]], &xs) {
                dx += delta;
                self.guides.push(Guide { vertical: true, pos: x, from: span[0].min(mb[1]), to: span[1].max(mb[3]) });
            }
            let mb = [b[0] + dx, b[1] + dy, b[2] + dx, b[3] + dy];
            if let Some((delta, y, span)) = self.snap_axis([mb[1], (mb[1] + mb[3]) / 2.0, mb[3]], &ys) {
                dy += delta;
                self.guides.push(Guide { vertical: false, pos: y, from: span[0].min(mb[0]), to: span[1].max(mb[2]) });
            }
        }
        for id in &tops {
            self.doc.translate_subtree(*id, dx, dy);
        }
        let driven = tops.iter().any(|t| self.doc.driven_by_layout(*t));
        self.drag_label = Some(if driven {
            "Lo coloca el Layout Group del padre (usa la Jerarquía para reordenar)".into()
        } else {
            format!("X {:.0}  Y {:.0}", b[0] + dx, b[1] + dy)
        });
    }

    fn drag_resize(&mut self, d: &Drag, pg: [f32; 2], sx: i8, sy: i8, shift: bool, no_snap: bool) {
        self.restore(d);
        let f = d.frame;
        let tops = self.movable_tops();
        if tops.is_empty() {
            return;
        }
        let corner = sx != 0 && sy != 0;
        let single = tops.len() == 1 && self.sel.len() == 1;
        let default_keep = if single { self.doc.get(tops[0]).map(|e| e.kind.keeps_aspect_on_corners()).unwrap_or(false) } else { true };
        let keep = corner && (default_keep != shift);
        let (sin, cos) = (-f.rot.to_radians()).sin_cos();
        let mut m = pg;
        if f.rot.abs() < 0.01 && !keep && !no_snap && self.grid.snap {
            if sx != 0 {
                m[0] = (m[0] / self.grid.w).round() * self.grid.w;
            }
            if sy != 0 {
                m[1] = (m[1] / self.grid.h).round() * self.grid.h;
            }
        } else if f.rot.abs() < 0.01 && !keep && !no_snap {
            let (xs, ys) = self.snap_targets(d);
            if sx != 0 {
                if let Some((delta, x, span)) = self.snap_axis([m[0]; 3], &xs) {
                    m[0] += delta;
                    self.guides.push(Guide { vertical: true, pos: x, from: span[0].min(f.cy - f.h / 2.0), to: span[1].max(f.cy + f.h / 2.0) });
                }
            }
            if sy != 0 {
                if let Some((delta, y, span)) = self.snap_axis([m[1]; 3], &ys) {
                    m[1] += delta;
                    self.guides.push(Guide { vertical: false, pos: y, from: span[0].min(f.cx - f.w / 2.0), to: span[1].max(f.cx + f.w / 2.0) });
                }
            }
        }
        let (dx, dy) = (m[0] - f.cx, m[1] - f.cy);
        let lx = dx * cos - dy * sin;
        let ly = dx * sin + dy * cos;
        let (sxf, syf) = (sx as f32, sy as f32);
        let ax = -sxf * f.w / 2.0;
        let ay = -syf * f.h / 2.0;
        let min = 4.0;
        let (mut nw, mut nh) = (f.w, f.h);
        if keep {
            let dir = [sxf * f.w, syf * f.h];
            let len2 = dir[0] * dir[0] + dir[1] * dir[1];
            let s = (((lx - ax) * dir[0] + (ly - ay) * dir[1]) / len2.max(1e-3)).max(min / f.w.min(f.h).max(1.0));
            nw = f.w * s;
            nh = f.h * s;
        } else {
            if sx != 0 {
                nw = ((lx - ax) * sxf).max(min);
            }
            if sy != 0 {
                nh = ((ly - ay) * syf).max(min);
            }
        }
        let ncx_l = if sx != 0 { ax + sxf * nw / 2.0 } else { 0.0 };
        let ncy_l = if sy != 0 { ay + syf * nh / 2.0 } else { 0.0 };
        let (s2, c2) = f.rot.to_radians().sin_cos();
        let ncx = f.cx + ncx_l * c2 - ncy_l * s2;
        let ncy = f.cy + ncx_l * s2 + ncy_l * c2;
        let rx = nw / f.w.max(1e-3);
        let ry = nh / f.h.max(1e-3);
        if single {
            let id = tops[0];
            let Some(o) = self.doc.get(id).cloned() else { return };
            self.doc.set_frame_keep_children(id, &Frame { cx: ncx, cy: ncy, w: nw, h: nh, rot: o.rot });
            if o.kind == Kind::Text && corner && keep {
                if let Some(e) = self.doc.get_mut(id) {
                    e.font_size = (o.font_size * rx).max(4.0);
                }
            }
        } else {
            let (bx0, by0) = (f.cx - f.w / 2.0, f.cy - f.h / 2.0);
            let (nbx0, nby0) = (ncx - nw / 2.0, ncy - nh / 2.0);
            for id in tops {
                let Some(o) = self.doc.get(id).cloned() else { continue };
                let [ocx, ocy] = o.center();
                let nf = Frame { cx: nbx0 + (ocx - bx0) * rx, cy: nby0 + (ocy - by0) * ry, w: o.w * rx, h: o.h * ry, rot: o.rot };
                self.doc.set_frame_keep_children(id, &nf);
                if keep && o.kind.has_text() {
                    if let Some(e) = self.doc.get_mut(id) {
                        e.font_size = (o.font_size * rx).max(4.0);
                    }
                }
            }
        }
        self.drag_label = Some(format!("An {:.0} × Al {:.0}", nw, nh));
    }

    fn drag_rotate(&mut self, d: &Drag, pg: [f32; 2], fine: bool) {
        self.restore(d);
        let f = d.frame;
        let a0 = (d.start_page[1] - f.cy).atan2(d.start_page[0] - f.cx);
        let a1 = (pg[1] - f.cy).atan2(pg[0] - f.cx);
        let mut delta = (a1 - a0).to_degrees();
        let single = self.sel.len() == 1;
        let base = if single { f.rot } else { 0.0 };
        let mut target = base + delta;
        if fine {
            target = (target / 15.0).round() * 15.0;
        } else {
            let snapped = (target / 45.0).round() * 45.0;
            if (snapped - target).abs() < 4.0 {
                target = snapped;
            }
        }
        delta = target - base;
        let (s, c) = delta.to_radians().sin_cos();
        for id in self.movable_tops() {
            let Some(o) = self.doc.get(id).cloned() else { continue };
            let [ocx, ocy] = o.center();
            let (rx, ry) = (ocx - f.cx, ocy - f.cy);
            let nf = Frame { cx: f.cx + rx * c - ry * s, cy: f.cy + rx * s + ry * c, w: o.w, h: o.h, rot: norm_angle(o.rot + delta) };
            self.doc.set_frame_keep_children(id, &nf);
        }
        self.drag_label = Some(format!("{:.0}°", norm_angle(target)));
    }

    // ------------------------------------------------------- text editing

    fn text_editor(&mut self, ui: &mut egui::Ui, v: &View) {
        let Some(id) = self.editing else { return };
        let Some(idx) = self.doc.index_of(id) else {
            self.editing = None;
            return;
        };
        let e = &self.doc.elements[idx];
        let a = e.aabb();
        let mut r = Rect::from_min_max(v.to_screen(a[0], a[1]), v.to_screen(a[2], a[3]));
        if e.kind != Kind::Text {
            let (pl, pr, pv) = render::text_padding(e);
            r = Rect::from_min_max(r.min + Vec2::new(pl, pv) * v.zoom, r.max - Vec2::new(pr, pv) * v.zoom);
            if r.width() < 20.0 || r.height() < 10.0 {
                r = r.expand2(Vec2::new(20.0, 10.0));
            }
        }
        let font = FontId::new((e.font_size * v.zoom).max(6.0), render::font_family(&e.font));
        let color = render::col(e.text_color, 1.0);
        let hx = match e.align {
            0 => egui::Align::LEFT,
            2 => egui::Align::RIGHT,
            _ => egui::Align::Center,
        };
        let vy = match (e.kind, e.valign) {
            (Kind::Text, _) | (_, 0) => egui::Align::TOP,
            (_, 2) => egui::Align::BOTTOM,
            _ => egui::Align::Center,
        };
        let align = egui::Align2([hx, vy]);
        ui.painter().rect_stroke(r.expand(3.0), 2.0, Stroke::new(1.5, SEL()), StrokeKind::Outside);
        let new = self.editing_new;
        let e = &mut self.doc.elements[idx];
        let resp = ui.put(
            r,
            egui::TextEdit::multiline(&mut e.text)
                .font(font)
                .text_color(color)
                .align(align)
                .frame(egui::Frame::NONE)
                .desired_width(r.width())
                .margin(egui::Margin::ZERO),
        );
        if new {
            resp.request_focus();
            self.editing_new = false;
        } else if resp.lost_focus() {
            self.editing = None;
        }
    }

    fn quick_actions(&mut self, ui: &mut egui::Ui, v: &View) {
        let Some(f) = self.sel_frame() else { return };
        let xf = frame_xf(&f, v);
        let (hw, hh) = (f.w / 2.0, f.h / 2.0);
        let ys: Vec<f32> = [[-hw, -hh], [hw, -hh], [hw, hh], [-hw, hh]].iter().map(|q| xf.map(q[0], q[1]).y).collect();
        let top = ys.iter().cloned().fold(f32::MAX, f32::min);
        let pos = Pos2::new(xf.c.x, top - 14.0);
        if !self.canvas_rect.shrink(RULER).contains(pos) {
            return;
        }
        let locked = self.selected().iter().all(|e| e.locked);
        egui::Area::new(egui::Id::unique("quick_actions"))
            .fixed_pos(pos)
            .pivot(Align2::CENTER_BOTTOM)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                egui::Frame::new()
                    .fill(theme::PANEL())
                    .corner_radius(7.0)
                    .stroke(Stroke::new(1.0, theme::LINE()))
                    .shadow(egui::epaint::Shadow { offset: [0, 2], blur: 8, spread: 0, color: Color32::from_black_alpha(90) })
                    .inner_margin(egui::Margin::symmetric(4, 2))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 2.0;
                            if self.sel.len() > 1 && ui.button("Agrupar").on_hover_text("Crea un Contenedor con la selección (Ctrl+G)").clicked() {
                                self.group_selection();
                            }
                            if self.selection_grouped() && ui.button("Desagrupar").on_hover_text("Ctrl+Shift+G").clicked() {
                                self.ungroup_selection();
                            }
                            if self.sel.len() == 1 && self.doc.parent_of(self.sel[0]).is_some() && crate::icons::button(ui, "parent_up", "^", "Seleccionar el padre (Esc)").clicked() {
                                if let Some(p) = self.doc.parent_of(self.sel[0]) {
                                    self.sel = vec![p];
                                }
                            }
                            if crate::icons::toggle(ui, locked, if locked { "lock" } else { "unlock" }, "L", "Bloquear / desbloquear").clicked() {
                                self.toggle_lock();
                            }
                            if crate::icons::button(ui, "duplicate", "D", "Duplicar (Ctrl+D)").clicked() {
                                self.duplicate_selection(30.0);
                            }
                            if ui.add_enabled_ui(!locked, |ui| crate::icons::button(ui, "trash", "X", "Eliminar con su contenido (Supr)")).inner.clicked() {
                                self.delete_selection();
                            }
                        });
                    });
            });
    }
}

pub fn outline(p: &egui::Painter, e: &Element, v: &View, stroke: Stroke) {
    let xf = Xf::of(e, v);
    let (hw, hh) = (e.w / 2.0, e.h / 2.0);
    let pts: Vec<Pos2> = [[-hw, -hh], [hw, -hh], [hw, hh], [-hw, hh]].iter().map(|q| xf.map(q[0], q[1])).collect();
    p.add(Shape::closed_line(pts, stroke));
}

fn dashed_outline(p: &egui::Painter, e: &Element, v: &View, stroke: Stroke) {
    let xf = Xf::of(e, v);
    let (hw, hh) = (e.w / 2.0, e.h / 2.0);
    let mut pts: Vec<Pos2> = [[-hw, -hh], [hw, -hh], [hw, hh], [-hw, hh]].iter().map(|q| xf.map(q[0], q[1])).collect();
    pts.push(pts[0]);
    p.extend(Shape::dashed_line(&pts, stroke, 5.0, 4.0));
}
