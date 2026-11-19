//! Editor state, undo history and editing commands.

use crate::model::{self, Doc, Element, Frame, Kind};
use eframe::egui::{self, Event, Key, Modifiers, Pos2, Vec2};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Templates,
    Elements,
    Text,
    Images,
    Layers,
    Plugins,
    Community,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DragKind {
    Move,
    Resize(i8, i8),
    Rotate,
    Marquee,
    Pan,
}

#[derive(Clone, Debug)]
pub struct Drag {
    pub kind: DragKind,
    pub start_screen: Pos2,
    pub start_page: [f32; 2],
    pub cur_page: [f32; 2],
    /// whole document at drag start (every frame starts again from it)
    pub orig: Vec<Element>,
    /// selection frame at drag start: centre, w, h, rotation
    pub frame: SelFrame,
    pub moved: bool,
    /// element to select on release when the press did not turn into a drag
    pub click_select: Option<u64>,
    pub start_pan: Vec2,
    /// Alt+drag: duplicate the selection once the drag really starts.
    pub duplicate: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct SelFrame {
    pub cx: f32,
    pub cy: f32,
    pub w: f32,
    pub h: f32,
    pub rot: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Guide {
    pub vertical: bool,
    pub pos: f32,
    pub from: f32,
    pub to: f32,
}

/// Where a hierarchy row is dropped.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Drop {
    Before(u64),
    Into(u64),
    After(u64),
    Root,
}

pub struct EditorApp {
    pub doc: Doc,
    pub sel: Vec<u64>,
    undo: Vec<Doc>,
    redo: Vec<Doc>,
    committed: Doc,
    settle: u32,
    pub zoom: f32,
    pub pan: Vec2,
    pub fit_pending: bool,
    pub drag: Option<Drag>,
    pub editing: Option<u64>,
    pub editing_new: bool,
    pub tab: Tab,
    pub drawer_open: bool,
    pub inspector_open: bool,
    pub clipboard: Vec<Element>,
    pub guides: Vec<Guide>,
    pub drag_label: Option<String>,
    pub hover: Option<u64>,
    pub dnd: Option<Vec<Element>>,
    pub export_open: bool,
    pub export_tab: u8,
    pub project_path: String,
    pub status: String,
    pub renaming: Option<u64>,
    pub search: String,
    pub canvas_rect: egui::Rect,
    pub screenshot: Option<crate::Shot>,
    pub frame_no: u64,
    pub grid: GridSettings,
    pub rulers: bool,
    pub preview: bool,
    pub preview_backup: Option<Doc>,
    /// Screen simulated in test mode (None = the reference resolution).
    pub preview_screen: Option<[f32; 2]>,
    pub console: Vec<String>,
    pub library: crate::library::Library,
    pub settings: crate::library::Settings,
    pub save_open: bool,
    pub template_name: String,
    pub element_name: String,
    /// Thumbnails of the built-in templates for the current page size (generated once).
    pub template_thumbs: Option<((f32, f32), Vec<model::Doc>)>,
    /// Hierarchy drag & drop.
    pub tree_drag: Option<u64>,
    pub tutorial: Option<crate::tutorial::Tutorial>,
    pub plugins: Vec<crate::plugins::Plugin>,
    pub community: crate::community::Community,
    pub accessibility_open: bool,
    pub review_open: bool,
    pub themes_open: bool,
    pub updater: crate::updater::Updater,
    pub long_screenshot: bool,
    pub theme_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridSettings {
    pub show: bool,
    pub snap: bool,
    pub w: f32,
    pub h: f32,
    pub color: [u8; 4],
}

pub const SIZE_PRESETS: &[(&str, f32, f32)] = &[
    ("PC / console Full HD", 1920.0, 1080.0),
    ("PC HD", 1280.0, 720.0),
    ("PC 4K", 3840.0, 2160.0),
    ("Ultrawide 21:9", 2560.0, 1080.0),
    ("Phone portrait", 1080.0, 1920.0),
    ("Phone landscape", 1920.0, 1080.0),
    ("Tablet 4:3", 2048.0, 1536.0),
    ("Square", 1080.0, 1080.0),
];

/// Screens offered in test mode to check how the anchors adapt.
pub const SCREEN_PRESETS: &[(&str, f32, f32)] = &[
    ("Reference resolution", 0.0, 0.0),
    ("1280 × 720 (16:9)", 1280.0, 720.0),
    ("2560 × 1080 (21:9)", 2560.0, 1080.0),
    ("1024 × 768 (4:3)", 1024.0, 768.0),
    ("2732 × 2048 (iPad)", 2732.0, 2048.0),
    ("2400 × 1080 (phone landscape)", 2400.0, 1080.0),
    ("1080 × 2400 (phone portrait)", 1080.0, 2400.0),
    ("1080 × 1080 (square)", 1080.0, 1080.0),
];

impl EditorApp {
    pub fn new(screenshot: Option<crate::Shot>) -> Self {
        let mut doc = model::template(0, 1920.0, 1080.0);
        lock_backgrounds(&mut doc);
        let first_button = doc.elements.iter().find(|e| e.kind == Kind::Button).map(|e| e.id);
        let settings = crate::library::Settings::load();
        EditorApp {
            committed: doc.clone(),
            doc,
            sel: first_button.into_iter().collect(),
            undo: Vec::new(),
            redo: Vec::new(),
            settle: 3,
            zoom: 0.5,
            pan: Vec2::ZERO,
            fit_pending: true,
            drag: None,
            editing: None,
            editing_new: false,
            tab: Tab::Layers,
            drawer_open: true,
            inspector_open: true,
            clipboard: Vec::new(),
            guides: Vec::new(),
            drag_label: None,
            hover: None,
            dnd: None,
            export_open: false,
            export_tab: 0,
            project_path: if settings.last_project_file.is_empty() { "proyecto.editorgui.json".into() } else { settings.last_project_file.clone() },
            status: "Ready. Drag elements onto the canvas, organise them in the Hierarchy and export to Unity.".into(),
            renaming: None,
            search: String::new(),
            canvas_rect: egui::Rect::NOTHING,
            screenshot,
            frame_no: 0,
            grid: GridSettings { show: false, snap: false, w: 60.0, h: 60.0, color: [0, 170, 255, 70] },
            rulers: true,
            preview: false,
            preview_backup: None,
            preview_screen: None,
            console: Vec::new(),
            library: crate::library::Library::load(),
            settings,
            save_open: false,
            template_name: String::new(),
            element_name: String::new(),
            template_thumbs: None,
            tree_drag: None,
            tutorial: None,
            plugins: crate::plugins::load_all(),
            community: crate::community::Community::new(),
            accessibility_open: false,
            review_open: false,
            themes_open: false,
            updater: crate::updater::Updater::new(),
            long_screenshot: false,
            theme_name: String::new(),
        }
    }

    // ------------------------------------------------------------------ undo

    // History is frozen in test mode: the document on screen is a throw-away copy.
    pub fn can_undo(&self) -> bool {
        !self.preview && !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.preview && !self.redo.is_empty()
    }

    pub fn undo(&mut self) {
        if self.preview {
            return;
        }
        // Changes made while a field had focus are not a step yet: they are saved before undoing.
        if !self.preview && self.doc != self.committed {
            let prev = std::mem::replace(&mut self.committed, self.doc.clone());
            self.undo.push(prev);
        }
        if let Some(prev) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.doc, prev));
            self.committed = self.doc.clone();
            self.after_history();
        }
    }

    pub fn redo(&mut self) {
        if self.preview {
            return;
        }
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.doc, next));
            self.committed = self.doc.clone();
            self.after_history();
        }
    }

    fn after_history(&mut self) {
        self.sel.retain(|id| self.doc.get(*id).is_some());
        self.editing = None;
        self.drag = None;
    }

    /// Called at the end of each frame: layout groups and text boxes are updated,
    /// then (when nothing is being dragged or typed) any change becomes one undo step.
    pub fn commit(&mut self, ctx: &egui::Context) {
        layout_texts(ctx, &mut self.doc);
        self.doc.apply_layouts();
        if self.preview {
            return; // test-mode changes are thrown away on exit
        }
        if self.settle > 0 {
            self.settle -= 1;
            self.committed = self.doc.clone();
            return;
        }
        let busy = ctx.input(|i| i.pointer.any_down()) || ctx.memory(|m| m.focused().is_some()) || self.drag.is_some();
        if self.doc.pixel_art && !busy {
            for e in &mut self.doc.elements {
                e.x = e.x.round();
                e.y = e.y.round();
                e.w = e.w.round().max(1.0);
                e.h = e.h.round().max(1.0);
            }
        }
        if !busy && self.doc != self.committed {
            let prev = std::mem::replace(&mut self.committed, self.doc.clone());
            self.undo.push(prev);
            if self.undo.len() > 200 {
                self.undo.remove(0);
            }
            self.redo.clear();
        }
    }

    // ------------------------------------------------------------- selection

    pub fn selected(&self) -> Vec<&Element> {
        self.doc.elements.iter().filter(|e| self.sel.contains(&e.id)).collect()
    }

    pub fn is_selected(&self, id: u64) -> bool {
        self.sel.contains(&id)
    }

    /// What a click on `id` selects: the outermost unselected "Container" (Empty)
    /// around it, unless `deep` (Ctrl) is held — like groups in a design editor.
    pub fn click_target(&self, id: u64, deep: bool) -> u64 {
        if deep {
            return id;
        }
        let mut t = id;
        for a in self.doc.ancestors(id) {
            let Some(e) = self.doc.get(a) else { break };
            if e.kind == Kind::Empty && !self.sel.contains(&a) && !e.locked {
                t = a;
            } else {
                break;
            }
        }
        t
    }

    pub fn select_only(&mut self, id: u64) {
        self.sel = vec![id];
    }

    pub fn toggle_select(&mut self, id: u64) {
        if self.sel.contains(&id) {
            self.sel.retain(|s| *s != id);
        } else {
            self.sel.push(id);
        }
    }

    /// Selected elements whose ancestors are not selected (what transforms act on).
    pub fn sel_top(&self) -> Vec<u64> {
        self.doc.top_level(&self.sel)
    }

    /// The frame drawn around the selection: the rotated box for one element,
    /// otherwise the axis-aligned bounds of everything selected.
    pub fn sel_frame(&self) -> Option<SelFrame> {
        let mut items = self.selected();
        if items.iter().any(|e| !e.locked) {
            items.retain(|e| !e.locked);
        }
        match items.len() {
            0 => None,
            1 => {
                let e = items[0];
                let [cx, cy] = e.center();
                Some(SelFrame { cx, cy, w: e.w, h: e.h, rot: e.rot })
            }
            _ => {
                let b = bounds(&items);
                Some(SelFrame { cx: (b[0] + b[2]) / 2.0, cy: (b[1] + b[3]) / 2.0, w: b[2] - b[0], h: b[3] - b[1], rot: 0.0 })
            }
        }
    }

    /// Container new elements go into: a single selected Panel / Container / ScrollView.
    pub fn insert_parent(&self) -> Option<u64> {
        if self.sel.len() != 1 {
            return None;
        }
        let e = self.doc.get(self.sel[0])?;
        matches!(e.kind, Kind::Panel | Kind::Empty | Kind::ScrollView).then_some(e.id)
    }

    // -------------------------------------------------------------- commands

    pub fn insert(&mut self, mut e: Element, at: Option<[f32; 2]>) {
        let parent = match at {
            Some(p) => self.container_at(p),
            None => self.insert_parent(),
        };
        let pf = self.doc.frame_of(parent);
        match at {
            Some([x, y]) => {
                e.x = x - e.w / 2.0;
                e.y = y - e.h / 2.0;
            }
            None => {
                let c = pf.to_world([0.0, 0.0]);
                e.x = c[0] - e.w / 2.0;
                e.y = c[1] - e.h / 2.0;
                let mut n = 0;
                while n < 20 && self.doc.elements.iter().any(|o| (o.x - e.x).abs() < 1.0 && (o.y - e.y).abs() < 1.0) {
                    e.x += 30.0;
                    e.y += 30.0;
                    n += 1;
                }
            }
        }
        e.rot = (pf.rot + e.rot) % 360.0;
        e.group = None;
        e.parent = parent;
        let kind = e.kind;
        let id = self.doc.add(e);
        self.doc.normalize();
        if let Some(el) = self.doc.get_mut(id) {
            if el.name.is_empty() || el.name == kind.label() {
                el.name = format!("{} {}", kind.label(), id);
            }
        }
        self.sel = vec![id];
        let pname = parent.and_then(|p| self.doc.get(p)).map(|p| format!(" inside “{}”", p.name)).unwrap_or_default();
        self.status = crate::i18n::tf("Added: {}{}", &[&crate::i18n::t(kind.label()), &pname]);
    }

    /// Topmost Panel / Container / ScrollView under a page point (drop target).
    pub fn container_at(&self, p: [f32; 2]) -> Option<u64> {
        self.doc
            .elements
            .iter()
            .rev()
            .find(|e| matches!(e.kind, Kind::Panel | Kind::Empty | Kind::ScrollView) && self.doc.shown(e.id) && !e.locked && e.contains(p[0], p[1], 0.0))
            .map(|e| e.id)
    }

    pub fn delete_selection(&mut self) {
        if self.sel.is_empty() {
            return;
        }
        let of: Vec<u64> = self.sel_top().into_iter().filter(|id| !self.doc.get(*id).map(|e| e.locked).unwrap_or(true)).collect();
        let n = of.len();
        self.doc.remove_subtrees(&of);
        self.sel.retain(|id| self.doc.get(*id).is_some());
        self.sel.clear();
        self.editing = None;
        self.status = crate::i18n::tf("Deleted {} element(s) with their contents (locked ones are kept)", &[&n.to_string()]);
    }

    pub fn duplicate_selection(&mut self, offset: f32) {
        let tops = self.sel_top();
        if tops.is_empty() {
            return;
        }
        let mut new_ids = Vec::new();
        for t in tops {
            let parent = self.doc.parent_of(t);
            let mut items = self.doc.collect_subtrees(&[t]);
            for e in items.iter_mut() {
                e.x += offset;
                e.y += offset;
                e.locked = false;
            }
            if let Some(first) = items.first_mut() {
                first.name = format!("{} (copia)", first.name);
            }
            let roots = self.doc.insert_items(&items, parent);
            // place right after the original among its siblings
            let mut sibs: Vec<u64> = self.doc.children(parent).into_iter().filter(|s| !roots.contains(s)).collect();
            let at = sibs.iter().position(|s| *s == t).map(|i| i + 1).unwrap_or(sibs.len());
            for (k, r) in roots.iter().enumerate() {
                sibs.insert(at + k, *r);
            }
            self.doc.set_sibling_order(parent, &sibs);
            new_ids.extend(roots);
        }
        self.sel = new_ids;
        self.status = crate::i18n::t("Duplicated (with its contents)");
    }

    pub fn copy(&mut self) {
        self.clipboard = self.doc.collect_subtrees(&self.sel);
        if !self.clipboard.is_empty() {
            self.status = crate::i18n::tf("Copied {} element(s)", &[&self.doc.top_level(&self.sel).len().to_string()]);
        }
    }

    pub fn paste(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }
        for e in self.clipboard.iter_mut() {
            e.x += 40.0;
            e.y += 40.0;
        }
        let items = self.clipboard.clone();
        let mut parent = self.insert_parent();
        if let Some(p) = parent {
            if items.iter().any(|e| e.id == p) || self.doc.ancestors(p).iter().any(|a| items.iter().any(|e| e.id == *a)) {
                parent = self.doc.parent_of(p);
            }
        }
        self.sel = self.doc.insert_items(&items, parent);
        self.doc.sanitize();
        self.status = crate::i18n::t("Pasted");
    }

    /// Wrap the selection in a new "Container" (an empty RectTransform), like grouping in Unity.
    pub fn group_selection(&mut self) {
        // A locked element inside a new (unlocked) Container could be moved or deleted along with it.
        let tops: Vec<u64> = self.sel_top().into_iter().filter(|i| !self.doc.get(*i).map(|e| e.locked).unwrap_or(true)).collect();
        if tops.is_empty() {
            return;
        }
        let parent = self.doc.parent_of(tops[0]);
        let items: Vec<&Element> = tops.iter().filter_map(|i| self.doc.get(*i)).collect();
        let b = bounds(&items);
        let mut g = Element::new(0, Kind::Empty);
        g.name = format!("Group {}", self.doc.next_id);
        g.x = b[0];
        g.y = b[1];
        g.w = (b[2] - b[0]).max(1.0);
        g.h = (b[3] - b[1]).max(1.0);
        g.parent = parent;
        let gid = self.doc.add(g);
        let mut sibs: Vec<u64> = self.doc.children(parent).into_iter().filter(|s| *s != gid).collect();
        let at = sibs.iter().position(|s| *s == tops[0]).unwrap_or(sibs.len());
        sibs.insert(at, gid);
        self.doc.set_sibling_order(parent, &sibs);
        for t in &tops {
            if let Some(e) = self.doc.get_mut(*t) {
                e.parent = Some(gid);
            }
        }
        self.doc.set_sibling_order(Some(gid), &tops);
        self.sel = vec![gid];
        self.status = crate::i18n::t("Grouped into a Container (empty RectTransform in Unity)");
    }

    /// Remove selected "Containers", keeping their children where they are.
    pub fn ungroup_selection(&mut self) {
        let mut freed = Vec::new();
        for id in self.sel.clone() {
            let Some(e) = self.doc.get(id) else { continue };
            if e.kind != Kind::Empty || e.locked {
                continue;
            }
            let parent = self.doc.parent_of(id);
            let kids = self.doc.children(Some(id));
            let mut sibs = self.doc.children(parent);
            let at = sibs.iter().position(|s| *s == id).unwrap_or(sibs.len());
            sibs.retain(|s| *s != id);
            for (k, c) in kids.iter().enumerate() {
                sibs.insert((at + k).min(sibs.len()), *c);
                if let Some(ce) = self.doc.get_mut(*c) {
                    ce.parent = parent;
                }
            }
            self.doc.elements.retain(|x| x.id != id);
            self.doc.set_sibling_order(parent, &sibs);
            freed.extend(kids);
        }
        freed.retain(|id| self.doc.get(*id).is_some());
        if !freed.is_empty() {
            self.sel = freed;
            self.status = crate::i18n::t("Ungrouped");
        } else {
            self.status = crate::i18n::t("Select a Container to ungroup");
        }
    }

    pub fn selection_grouped(&self) -> bool {
        self.sel.len() == 1 && self.doc.get(self.sel[0]).map(|e| e.kind == Kind::Empty).unwrap_or(false)
    }

    /// Move the selection out of its parent (one level up), keeping its place on the page.
    pub fn unparent_selection(&mut self) {
        for id in self.sel_top() {
            if let Some(p) = self.doc.parent_of(id) {
                let gp = self.doc.parent_of(p);
                let idx = self.doc.children(gp).iter().position(|s| *s == p).map(|i| i + 1);
                self.doc.reparent(id, gp, idx);
            }
        }
        self.status = crate::i18n::t("Moved up one level in the hierarchy");
    }

    pub fn toggle_lock(&mut self) {
        let sel = self.sel.clone();
        let any_unlocked = self.selected().iter().any(|e| !e.locked);
        for e in self.doc.elements.iter_mut().filter(|e| sel.contains(&e.id)) {
            e.locked = any_unlocked;
        }
        self.status = crate::i18n::t(if any_unlocked { "Locked: it can't be moved or deleted" } else { "Unlocked" });
    }

    /// Move the selection among its siblings. `to_front`: Some(true) last (drawn on top), Some(false) first.
    pub fn reorder(&mut self, to_front: Option<bool>, step: i32) {
        let mut tops = self.sel_top();
        // Moving forward (or to the back, inserting at 0) is traversed in reverse: so two adjacent siblings do not cancel out.
        if step > 0 || to_front == Some(false) {
            tops.reverse();
        }
        for id in tops {
            let parent = self.doc.parent_of(id);
            let mut sibs = self.doc.children(parent);
            let Some(i) = sibs.iter().position(|s| *s == id) else { continue };
            sibs.remove(i);
            let j = match to_front {
                Some(true) => sibs.len(),
                Some(false) => 0,
                None => (i as i32 + step).clamp(0, sibs.len() as i32) as usize,
            };
            sibs.insert(j, id);
            self.doc.set_sibling_order(parent, &sibs);
        }
    }

    /// Hierarchy drag & drop.
    pub fn drop_in_tree(&mut self, id: u64, target: Drop) {
        let ok = match target {
            Drop::Root => self.doc.reparent(id, None, None),
            Drop::Into(p) => self.doc.reparent(id, Some(p), None),
            Drop::Before(t) | Drop::After(t) => {
                if t == id {
                    return;
                }
                let parent = self.doc.parent_of(t);
                let sibs: Vec<u64> = self.doc.children(parent).into_iter().filter(|s| *s != id).collect();
                let i = sibs.iter().position(|s| *s == t).unwrap_or(sibs.len());
                let at = if matches!(target, Drop::After(_)) { i + 1 } else { i };
                self.doc.reparent(id, parent, Some(at))
            }
        };
        if ok {
            if let Drop::Into(p) = target {
                if let Some(pe) = self.doc.get_mut(p) {
                    pe.collapsed = false;
                }
            }
            self.status = crate::i18n::t("Hierarchy updated (the on-screen position is kept)");
        } else {
            self.status = crate::i18n::t("An element can't be placed inside itself");
        }
    }

    /// Align selection: axis 0 = horizontal, 1 = vertical; where 0 = start, 1 = centre, 2 = end.
    /// One element aligns to its parent, several align to their common bounds.
    pub fn align(&mut self, axis: usize, where_: usize) {
        let tops = self.sel_top();
        if tops.is_empty() {
            return;
        }
        let refb = if tops.len() == 1 {
            self.doc.parent_frame(tops[0]).aabb()
        } else {
            bounds(&tops.iter().filter_map(|i| self.doc.get(*i)).collect::<Vec<_>>())
        };
        for id in tops {
            let Some(e) = self.doc.get(id) else { continue };
            if e.locked {
                continue;
            }
            let b = e.aabb();
            let (lo, hi, rlo, rhi) = if axis == 0 { (b[0], b[2], refb[0], refb[2]) } else { (b[1], b[3], refb[1], refb[3]) };
            let target = match where_ {
                0 => rlo,
                2 => rhi - (hi - lo),
                _ => (rlo + rhi) / 2.0 - (hi - lo) / 2.0,
            };
            let d = target - lo;
            if axis == 0 { self.doc.translate_subtree(id, d, 0.0) } else { self.doc.translate_subtree(id, 0.0, d) }
        }
    }

    pub fn distribute(&mut self, axis: usize) {
        // Locked elements do not move (same as with the arrow keys).
        let mut items: Vec<Element> = self.sel_top().iter().filter_map(|i| self.doc.get(*i).cloned()).filter(|e| !e.locked).collect();
        if items.len() < 3 {
            return;
        }
        let key = |e: &Element| {
            let b = e.aabb();
            if axis == 0 { b[0] } else { b[1] }
        };
        items.sort_by(|a, b| key(a).partial_cmp(&key(b)).unwrap_or(std::cmp::Ordering::Equal));
        let sizes: Vec<f32> = items
            .iter()
            .map(|e| {
                let b = e.aabb();
                if axis == 0 { b[2] - b[0] } else { b[3] - b[1] }
            })
            .collect();
        let all = bounds(&items.iter().collect::<Vec<_>>());
        let (lo, hi) = if axis == 0 { (all[0], all[2]) } else { (all[1], all[3]) };
        let gap = (hi - lo - sizes.iter().sum::<f32>()) / (items.len() - 1) as f32;
        let mut cursor = lo;
        for (it, s) in items.iter().zip(sizes) {
            let d = cursor - key(it);
            if axis == 0 { self.doc.translate_subtree(it.id, d, 0.0) } else { self.doc.translate_subtree(it.id, 0.0, d) }
            cursor += s + gap;
        }
    }

    pub fn nudge(&mut self, dx: f32, dy: f32) {
        for id in self.sel_top() {
            if !self.doc.get(id).map(|e| e.locked).unwrap_or(true) {
                self.doc.translate_subtree(id, dx, dy);
            }
        }
    }

    /// Set the box of an element from the inspector; children follow their anchors.
    pub fn set_frame(&mut self, id: u64, f: Frame) {
        self.doc.set_frame_keep_children(id, &f);
    }

    pub fn load_template(&mut self, i: usize) {
        self.set_preview(false);
        let mut d = model::template(i, self.doc.width, self.doc.height);
        lock_backgrounds(&mut d);
        self.doc = d;
        self.sel.clear();
        self.editing = None;
        self.fit_pending = true;
        self.status = crate::i18n::tf("Template applied: {} (Ctrl+Z to undo)", &[&model::TEMPLATES[i].to_string()]);
    }

    /// Change the reference resolution: every element keeps its anchors (like Unity).
    pub fn resize_page(&mut self, w: f32, h: f32) {
        let old = self.doc.page_frame();
        self.doc.width = w;
        self.doc.height = h;
        self.doc.reapply_children(None, old);
        self.doc.apply_layouts();
        self.fit_pending = true;
        self.status = crate::i18n::tf("Reference resolution: {} × {} (elements follow their anchors)", &[&w.to_string(), &h.to_string()]);
    }

    fn remember_project(&mut self) {
        self.settings.last_project_file = self.project_path.clone();
        self.settings.save();
    }

    pub fn save_project(&mut self) {
        match serde_json::to_string_pretty(self.doc_real()).map_err(|e| e.to_string()).and_then(|s| std::fs::write(&self.project_path, s).map_err(|e| e.to_string())) {
            Ok(()) => {
                self.status = crate::i18n::tf("Project saved to {}", &[&self.project_path]);
                self.remember_project();
            }
            Err(e) => self.status = crate::i18n::tf("Could not save: {}", &[&e.to_string()]),
        }
    }

    pub fn open_project(&mut self) {
        match std::fs::read_to_string(&self.project_path).map_err(|e| e.to_string()).and_then(|s| serde_json::from_str::<Doc>(crate::library::strip_bom(&s)).map_err(|e| e.to_string())) {
            Ok(mut d) => {
                self.set_preview(false);
                crate::library::set_design_folder(std::path::Path::new(&self.project_path));
                d.migrate_groups();
                self.doc = d;
                // A different document: undo must not bring the previous one back.
                self.undo.clear();
                self.redo.clear();
                self.committed = self.doc.clone();
                self.settle = 3;
                self.sel.clear();
                self.editing = None;
                self.drag = None;
                self.fit_pending = true;
                self.status = format!("{} {}", crate::i18n::t("Project opened:"), self.project_path);
                self.remember_project();
            }
            Err(e) => self.status = crate::i18n::tf("Could not open: {}", &[&e.to_string()]),
        }
    }

    // ------------------------------------------------------------- library / uploads

    fn report(&mut self, what: &str, results: Vec<Result<String, String>>) {
        let ok = results.iter().filter(|r| r.is_ok()).count();
        let errs: Vec<String> = results.into_iter().filter_map(|r| r.err()).collect();
        self.status = if errs.is_empty() {
            crate::i18n::tf("{}: {} added to your library", &[what, &ok.to_string()])
        } else {
            crate::i18n::tf("{}: {} added, {} failed ({})", &[what, &ok.to_string(), &errs.len().to_string(), &errs.join("; ")])
        };
    }

    pub fn upload_images(&mut self) {
        let Some(files) = rfd::FileDialog::new().set_title("Upload images").add_filter("Images", crate::library::IMAGE_EXTS).pick_files() else { return };
        let r = files.iter().map(|f| self.library.add_image(f)).collect();
        self.report("Images", r);
    }

    pub fn upload_elements(&mut self) {
        let mut exts: Vec<&str> = crate::library::IMAGE_EXTS.to_vec();
        exts.push("json");
        let Some(files) = rfd::FileDialog::new()
            .set_title("Upload elements (PNG images with transparency or .json elements)")
            .add_filter("Elements", &exts)
            .pick_files()
        else {
            return;
        };
        let r = files.iter().map(|f| self.library.add_element_file(f)).collect();
        self.report("Elements", r);
    }

    pub fn upload_templates(&mut self) {
        let Some(files) = rfd::FileDialog::new().set_title("Upload templates (.json from this editor)").add_filter("Templates", &["json"]).pick_files() else { return };
        let r = files.iter().map(|f| self.library.add_template_file(f)).collect();
        self.report("Templates", r);
    }

    pub fn add_fonts(&mut self, ctx: &egui::Context) {
        let mut dialog = rfd::FileDialog::new().set_title("Add fonts (.ttf / .otf)").add_filter("Fonts", crate::library::FONT_EXTS);
        if cfg!(target_os = "windows") {
            dialog = dialog.set_directory("C:/Windows/Fonts");
        }
        let Some(files) = dialog.pick_files() else { return };
        let mut r = Vec::new();
        for f in files {
            r.push(crate::library::add_font_file(&f).map(|(name, bytes, path)| {
                crate::render::add_font(ctx, &name, bytes, path);
                name
            }));
        }
        self.report("Fonts", r);
    }

    pub fn insert_library_image(&mut self, file: &str, at: Option<[f32; 2]>) {
        let (w, h) = crate::library::image_size(&crate::library::resolve_image(file)).unwrap_or((400, 300));
        let max = (self.doc.width.min(self.doc.height) * 0.5).max(100.0);
        let e = crate::library::image_element(file, w, h, max);
        self.insert(e, at);
    }

    /// Insert a saved library element (one or more elements, with their hierarchy).
    pub fn insert_items(&mut self, items: &[Element], at: Option<[f32; 2]>) {
        if items.is_empty() {
            return;
        }
        let b = bounds(&items.iter().collect::<Vec<_>>());
        let (bw, bh) = (b[2] - b[0], b[3] - b[1]);
        let parent = match at {
            Some(p) => self.container_at(p),
            None => self.insert_parent(),
        };
        let (cx, cy) = match at {
            Some(p) => (p[0], p[1]),
            None => {
                let c = self.doc.frame_of(parent);
                (c.cx, c.cy)
            }
        };
        let mut items = items.to_vec();
        for e in items.iter_mut() {
            e.x = cx - bw / 2.0 + (e.x - b[0]);
            e.y = cy - bh / 2.0 + (e.y - b[1]);
            e.locked = false;
        }
        // several loose elements arrive wrapped in a Container
        let ids: Vec<u64> = items.iter().map(|e| e.id).collect();
        let loose = items.iter().filter(|e| e.parent.map(|p| !ids.contains(&p)).unwrap_or(true)).count();
        if loose > 1 {
            let mut g = Element::new(u64::MAX, Kind::Empty);
            g.name = "Grupo".into();
            g.x = cx - bw / 2.0;
            g.y = cy - bh / 2.0;
            g.w = bw.max(1.0);
            g.h = bh.max(1.0);
            for e in items.iter_mut() {
                if e.parent.map(|p| !ids.contains(&p)).unwrap_or(true) {
                    e.parent = Some(u64::MAX);
                }
            }
            items.insert(0, g);
        }
        self.sel = self.doc.insert_items(&items, parent);
        // What arrives from the library or the community may carry absurd values (1e9 px letter, NaN…).
        self.doc.sanitize();
        self.status = crate::i18n::t("Element from your library added");
    }

    pub fn save_selection_as_element(&mut self, name: &str) {
        let items = self.doc.collect_subtrees(&self.sel);
        let name = if name.trim().is_empty() { items.first().map(|e| e.name.clone()).unwrap_or_else(|| "Element".into()) } else { name.trim().to_string() };
        match self.library.save_element(&name, &items) {
            Ok(_) => self.status = crate::i18n::tf("“{}” saved to My elements", &[&name]),
            Err(e) => self.status = crate::i18n::tf("Could not save the element: {}", &[&e.to_string()]),
        }
    }

    pub fn save_as_template(&mut self, name: &str) {
        let name = if name.trim().is_empty() { self.doc.title.clone() } else { name.trim().to_string() };
        let doc = self.doc_real().clone();
        match self.library.save_template(&name, &doc) {
            Ok(_) => self.status = crate::i18n::tf("Template “{}” saved (Templates tab › My templates)", &[&name]),
            Err(e) => self.status = crate::i18n::tf("Could not save the template: {}", &[&e.to_string()]),
        }
    }

    pub fn load_user_template(&mut self, doc: &Doc) {
        self.set_preview(false);
        let mut d = doc.clone();
        d.migrate_groups();
        lock_backgrounds(&mut d);
        self.doc = d;
        self.sel.clear();
        self.editing = None;
        self.fit_pending = true;
        self.status = crate::i18n::tf("Template applied: {} (Ctrl+Z to undo)", &[&doc.title]);
    }

    pub fn save_project_as(&mut self) {
        let name = format!("{}.editorgui.json", self.doc.title.trim());
        if let Some(path) = rfd::FileDialog::new().set_title("Save project").set_file_name(&name).add_filter("Proyecto", &["json"]).save_file() {
            self.project_path = path.to_string_lossy().to_string();
            self.save_project();
        }
    }

    pub fn open_project_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().set_title("Open project").add_filter("Proyecto", &["json"]).pick_file() {
            self.project_path = path.to_string_lossy().to_string();
            self.open_project();
        }
    }

    // ------------------------------------------------------------- test mode / code

    /// The design that is saved or exported: in test mode, the one from before the test (clicks do not count).
    pub fn doc_real(&self) -> &Doc {
        self.preview_backup.as_ref().filter(|_| self.preview).unwrap_or(&self.doc)
    }

    /// Enter or leave test mode. Changes made by click code are undone on exit.
    pub fn set_preview(&mut self, on: bool) {
        if on && !self.preview {
            self.preview_backup = Some(self.doc.clone());
            self.preview = true;
            self.sel.clear();
            self.editing = None;
            self.drag = None;
            self.fit_pending = true;
            self.console.push("▶ Test mode: click the elements with ⚡ to run their code.".into());
            self.status = crate::i18n::t("Test mode on · Esc to go back to editing");
        } else if !on && self.preview {
            if let Some(d) = self.preview_backup.take() {
                self.doc = d;
            }
            self.preview = false;
            self.fit_pending = true;
            self.status = crate::i18n::t("Back to edit mode (the design is as it was before the test)");
        }
    }

    pub fn run_click(&mut self, id: u64) {
        let Some(e) = self.doc.get(id) else { return };
        let code = e.on_click.clone();
        if code.trim().is_empty() {
            return;
        }
        crate::script::run(&mut self.doc, id, &code, &mut self.console);
        if self.console.len() > 200 {
            let n = self.console.len() - 200;
            self.console.drain(..n);
        }
    }

    // ------------------------------------------------------------- export

    /// Write the whole package into the Unity project chosen before (or ask for one).
    pub fn export_to_unity(&mut self, ask: bool) {
        let mut dir = self.settings.unity_project.clone();
        if ask || dir.is_empty() || crate::export::find_unity_root(std::path::Path::new(&dir)).is_none() {
            let mut dlg = rfd::FileDialog::new().set_title("Unity project folder (the one that contains Assets)");
            if !dir.is_empty() {
                dlg = dlg.set_directory(&dir);
            }
            match dlg.pick_folder() {
                Some(p) => dir = p.to_string_lossy().to_string(),
                None => return,
            }
        }
        match crate::export::write_unity_package(self.doc_real(), std::path::Path::new(&dir)) {
            Ok(files) => {
                let root = crate::export::find_unity_root(std::path::Path::new(&dir)).map(|p| p.to_string_lossy().to_string()).unwrap_or(dir.clone());
                self.settings.unity_project = root.clone();
                self.settings.save();
                self.status = crate::i18n::tf(
                    "Exported to Unity ({} files) in {}\\{} · Unity creates the prefab in Assets/EditorGUI/Prefabs when you return to the window",
                    &[&files.len().to_string(), &root, &crate::export::UNITY_ROOT.replace('/', "\\")],
                );
            }
            Err(e) => self.status = crate::i18n::tf("Could not export: {}", &[&e.to_string()]),
        }
    }

    pub fn save_text_file(&mut self, title: &str, name: &str, ext: &str, text: &str) {
        if let Some(path) = rfd::FileDialog::new().set_title(title).set_file_name(name).add_filter(ext, &[ext]).save_file() {
            match std::fs::write(&path, text) {
                Ok(()) => self.status = crate::i18n::tf("Saved: {}", &[&path.display().to_string()]),
                Err(e) => self.status = format!("Could not save: {}", e),
            }
        }
    }

    // ------------------------------------------------------------- keyboard

    pub fn keyboard(&mut self, ctx: &egui::Context) {
        if self.tutorial.is_some() {
            return; // the tutorial handles its own keys
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F1)) {
            self.open_tutorial(crate::tutorial::Track::Beginner);
            return;
        }
        if ctx.input_mut(|i| i.consume_key(Modifiers::NONE, Key::F5)) {
            self.set_preview(!self.preview);
            return;
        }
        if self.preview {
            if ctx.input(|i| i.key_pressed(Key::Escape)) {
                self.set_preview(false);
            }
            return;
        }
        let cmd = Modifiers::COMMAND;
        let cmd_shift = Modifiers::COMMAND | Modifiers::SHIFT;
        if ctx.input_mut(|i| i.consume_key(cmd, Key::S)) {
            if std::path::Path::new(&self.project_path).is_absolute() { self.save_project() } else { self.save_project_as() }
        }
        if ctx.input_mut(|i| i.consume_key(cmd_shift, Key::E)) {
            self.export_open = true;
        } else if ctx.input_mut(|i| i.consume_key(cmd, Key::E)) {
            self.export_to_unity(false);
        }
        let typing = ctx.memory(|m| m.focused().is_some()) || self.editing.is_some();
        if typing {
            if self.editing.is_some() && ctx.input(|i| i.key_pressed(Key::Escape)) {
                self.editing = None;
            }
            return;
        }
        let events = ctx.input(|i| i.events.clone());
        for ev in &events {
            match ev {
                Event::Copy => self.copy(),
                Event::Cut => {
                    self.copy();
                    self.delete_selection();
                }
                Event::Paste(_) => self.paste(),
                _ => {}
            }
        }
        ctx.input_mut(|i| {
            if i.consume_key(cmd_shift, Key::Z) || i.consume_key(cmd, Key::Y) {
                self.redo();
            } else if i.consume_key(cmd, Key::Z) {
                self.undo();
            }
            if i.consume_key(cmd, Key::D) {
                self.duplicate_selection(30.0);
            }
            if i.consume_key(cmd, Key::A) {
                self.sel = self.doc.elements.iter().filter(|e| !e.locked && e.visible && e.parent.is_none()).map(|e| e.id).collect();
            }
            if i.consume_key(cmd_shift, Key::G) {
                self.ungroup_selection();
            } else if i.consume_key(cmd, Key::G) {
                self.group_selection();
            }
            if i.consume_key(cmd_shift, Key::L) {
                self.toggle_lock();
            }
            if i.consume_key(cmd, Key::CloseBracket) {
                self.reorder(None, 1);
            }
            if i.consume_key(cmd, Key::OpenBracket) {
                self.reorder(None, -1);
            }
            if i.consume_key(cmd, Key::Num0) {
                self.fit_pending = true;
            }
            if i.consume_key(cmd, Key::Plus) || i.consume_key(cmd, Key::Equals) {
                self.zoom = (self.zoom * 1.25).min(4.0);
            }
            if i.consume_key(cmd, Key::Minus) {
                self.zoom = (self.zoom / 1.25).max(0.05);
            }
            if i.key_pressed(Key::Delete) || i.key_pressed(Key::Backspace) {
                self.delete_selection();
            }
            if i.key_pressed(Key::Escape) {
                // Esc climbs to the parent, then clears
                match self.sel.as_slice() {
                    [one] => match self.doc.parent_of(*one) {
                        Some(p) => self.sel = vec![p],
                        None => self.sel.clear(),
                    },
                    _ => self.sel.clear(),
                }
            }
            let step = if i.modifiers.shift { 10.0 } else { 1.0 };
            let mut d = (0.0, 0.0);
            if i.key_pressed(Key::ArrowLeft) {
                d.0 -= step
            }
            if i.key_pressed(Key::ArrowRight) {
                d.0 += step
            }
            if i.key_pressed(Key::ArrowUp) {
                d.1 -= step
            }
            if i.key_pressed(Key::ArrowDown) {
                d.1 += step
            }
            if d != (0.0, 0.0) {
                self.nudge(d.0, d.1);
            }
            if i.key_pressed(Key::Enter) && self.sel.len() == 1 {
                if let Some(e) = self.doc.get(self.sel[0]) {
                    if e.kind.has_text() && e.kind != Kind::Dropdown && !e.locked {
                        self.editing = Some(e.id);
                        self.editing_new = true;
                    }
                }
            }
        });
    }
}

pub fn bounds(items: &[&Element]) -> [f32; 4] {
    model::bounds_of(items.iter().copied())
}

fn lock_backgrounds(doc: &mut Doc) {
    let (w, h) = (doc.width, doc.height);
    for e in doc.elements.iter_mut() {
        if e.parent.is_none() && e.kind == Kind::Image && e.x <= 0.5 && e.y <= 0.5 && e.w >= w - 0.5 && e.h >= h - 0.5 {
            e.locked = true;
        }
    }
}

/// Text boxes grow and shrink in height with their content (like a design editor's text box).
pub fn layout_texts(ctx: &egui::Context, doc: &mut Doc) {
    let ids: Vec<u64> = doc.elements.iter().filter(|e| e.kind == Kind::Text).map(|e| e.id).collect();
    for id in ids {
        if doc.driven_by_layout(id) {
            continue;
        }
        let Some(e) = doc.get(id) else { continue };
        let job = crate::render::text_job(e, if e.text.is_empty() { " " } else { &e.text }, 1.0, e.w.max(1.0), Some(egui::Color32::WHITE));
        let g = ctx.fonts_mut(|f| f.layout_job(job));
        let h = g.size().y.max(e.font_size * 0.8);
        if (e.h - h).abs() > 0.5 {
            let mut f = e.frame();
            // keep the top edge in place (in the element's own rotation)
            let top = f.to_world([0.0, -f.h / 2.0]);
            f.h = h;
            let (s, c) = f.rot.to_radians().sin_cos();
            f.cx = top[0] - (h / 2.0) * s;
            f.cy = top[1] + (h / 2.0) * c;
            doc.set_frame_keep_children(id, &f);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app_with(template: usize) -> EditorApp {
        let mut a = EditorApp::new(None);
        a.doc = model::template(template, 1920.0, 1080.0);
        a.committed = a.doc.clone();
        a.sel.clear();
        a
    }

    fn id(a: &EditorApp, n: &str) -> u64 {
        a.doc.elements.iter().find(|e| e.name == n).unwrap().id
    }

    #[test]
    fn reordering_several_siblings_is_not_undone() {
        let mut a = app_with(0);
        let (j, o) = (id(&a, "Play button"), id(&a, "Options button"));
        let menu = a.doc.parent_of(j);
        a.sel = vec![j, o];
        a.reorder(None, 1);
        let order: Vec<String> = a.doc.children(menu).iter().map(|c| a.doc.get(*c).unwrap().name.clone()).collect();
        assert_eq!(order, ["Credits button", "Play button", "Options button", "Exit button"]);
        a.reorder(Some(false), 0);
        let order: Vec<String> = a.doc.children(menu).iter().map(|c| a.doc.get(*c).unwrap().name.clone()).collect();
        assert_eq!(&order[..2], ["Play button", "Options button"], "the background keeps its order");
    }

    #[test]
    fn ungroup_nested_no_leaves_ids_deleted() {
        let mut a = app_with(0);
        let menu = id(&a, "Menu");
        let head = id(&a, "Header");
        a.sel = vec![head, menu];
        a.group_selection();
        let g = a.sel[0];
        a.sel = vec![g, head];
        a.ungroup_selection();
        assert!(!a.sel.is_empty() && a.sel.iter().all(|s| a.doc.get(*s).is_some()), "{:?}", a.sel);
    }

    #[test]
    fn locked_are_not_grouped_or_ungrouped() {
        let mut a = app_with(0);
        let background = id(&a, "Background");
        let moon = id(&a, "Moon");
        a.doc.get_mut(background).unwrap().locked = true;
        a.sel = vec![background, moon];
        a.group_selection();
        assert_eq!(a.doc.parent_of(background), None, "the locked background does not enter the group");
        let menu = id(&a, "Menu");
        a.doc.get_mut(menu).unwrap().locked = true;
        a.sel = vec![menu];
        a.ungroup_selection();
        assert!(a.doc.get(menu).is_some(), "a locked Container is not undone");
    }

    #[test]
    fn pasting_a_container_puts_it_beside_itself() {
        let mut a = app_with(4);
        let win = id(&a, "Inventory");
        a.sel = vec![win];
        a.copy();
        a.paste();
        let new = a.sel[0];
        assert_ne!(new, win);
        assert_eq!(a.doc.parent_of(new), a.doc.parent_of(win), "hermano, no hijo");
    }

    #[test]
    fn insert_keeps_rotation_and_name() {
        let mut a = app_with(5);
        let mut e = Element::new(0, Kind::Button);
        e.name = "Pill button".into();
        e.rot = -8.0;
        a.insert(e, None);
        let n = a.doc.get(a.sel[0]).unwrap();
        assert_eq!(n.name, "Pill button");
        assert!((n.rot + 8.0).abs() < 0.01);
        a.insert(Element::new(0, Kind::Button), None);
        assert!(a.doc.get(a.sel[0]).unwrap().name.starts_with(&format!("{} ", Kind::Button.label())));
    }
}

#[cfg(test)]
mod history_tests {
    use super::*;

    #[test]
    fn open_project_clears_history_and_accepts_a_bom() {
        let mut a = EditorApp::new(None);
        a.doc = model::template(0, 1920.0, 1080.0);
        a.committed = a.doc.clone();
        // one undo step in the history
        a.doc.title = "Changed".into();
        a.undo.push(a.committed.clone());
        a.committed = a.doc.clone();
        assert!(a.can_undo());
        let other = model::template(1, 1920.0, 1080.0);
        let path = std::env::temp_dir().join(format!("lienzo-open-bom-{}.editorgui.json", std::process::id()));
        let mut text = String::from('\u{feff}');
        text.push_str(&serde_json::to_string(&other).unwrap());
        std::fs::write(&path, text).unwrap();
        a.project_path = path.to_string_lossy().to_string();
        a.open_project();
        assert_eq!(a.doc.title, other.title, "{}", a.status);
        assert!(!a.can_undo() && !a.can_redo());
        a.undo();
        assert_eq!(a.doc.title, other.title, "undo after Open must not resurrect the previous document");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn undo_and_redo_do_nothing_in_test_mode() {
        let mut a = EditorApp::new(None);
        a.doc = model::template(0, 1920.0, 1080.0);
        a.committed = a.doc.clone();
        let before = a.doc.clone();
        a.doc.title = "Edited".into();
        a.undo.push(before.clone());
        a.committed = a.doc.clone();
        a.redo.push(before.clone());
        a.set_preview(true);
        assert!(!a.can_undo() && !a.can_redo());
        let shown = a.doc.clone();
        a.undo();
        a.redo();
        assert_eq!(a.doc, shown);
        assert_eq!((a.undo.len(), a.redo.len()), (1, 1));
        a.set_preview(false);
        assert!(a.can_undo() && a.can_redo());
    }
}
