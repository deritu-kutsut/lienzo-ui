//! Document-format extensions: canvas / Unity settings, portrait layout (per-element overrides),
//! image slicing fields and the validators used by `sanitize()`, the review and the exporters.
//! Everything here is optional in the file and skipped when unused, so old designs round-trip unchanged.

#![allow(dead_code)] // public API used by the exporters / editor UI added on top of this model

use crate::model::{Doc, Element, LayoutGroup};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;

pub fn is_false(b: &bool) -> bool {
    !*b
}

fn yes() -> bool {
    true
}
fn no() -> Option<bool> {
    Some(false)
}
fn ser_imgui<S: serde::Serializer>(v: &Option<bool>, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_bool(v.unwrap_or(true))
}
fn plane_default() -> f32 {
    100.0
}

// ---------------------------------------------------------------------------
// Element: image mode
// ---------------------------------------------------------------------------

/// UnityEngine.UI.Image.type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ImageMode {
    #[default]
    Simple,
    Sliced,
    Tiled,
    Filled,
}

impl ImageMode {
    pub fn as_str(self) -> &'static str {
        match self {
            ImageMode::Simple => "Simple",
            ImageMode::Sliced => "Sliced",
            ImageMode::Tiled => "Tiled",
            ImageMode::Filled => "Filled",
        }
    }
}

// ---------------------------------------------------------------------------
// Doc: canvas
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CanvasRenderMode {
    #[default]
    ScreenSpaceOverlay,
    ScreenSpaceCamera,
    WorldSpace,
}

impl CanvasRenderMode {
    pub fn as_str(self) -> &'static str {
        match self {
            CanvasRenderMode::ScreenSpaceOverlay => "ScreenSpaceOverlay",
            CanvasRenderMode::ScreenSpaceCamera => "ScreenSpaceCamera",
            CanvasRenderMode::WorldSpace => "WorldSpace",
        }
    }
}

pub const SORT_MIN: i32 = -32768;
pub const SORT_MAX: i32 = 32767;

fn de_sort<'de, D: Deserializer<'de>>(d: D) -> Result<i32, D::Error> {
    let v = f64::deserialize(d)?;
    Ok(if v.is_finite() { v.clamp(SORT_MIN as f64, SORT_MAX as f64) as i32 } else { 0 })
}

/// Unity Canvas options.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CanvasSettings {
    pub render_mode: CanvasRenderMode,
    /// Canvas.sortingOrder, clamped to -32768..32767.
    #[serde(deserialize_with = "de_sort")]
    pub sort_order: i32,
    /// Apply the device safe area (notch / rounded corners) to the root.
    pub safe_area: bool,
    /// Export without its own Canvas / CanvasScaler: the root is a plain RectTransform that lives under the game's canvas.
    pub embed: bool,
    /// Canvas.planeDistance (ScreenSpaceCamera).
    #[serde(default = "plane_default")]
    pub plane_distance: f32,
}

impl Default for CanvasSettings {
    fn default() -> Self {
        CanvasSettings { render_mode: CanvasRenderMode::ScreenSpaceOverlay, sort_order: 0, safe_area: false, embed: false, plane_distance: 100.0 }
    }
}

impl CanvasSettings {
    pub fn is_default(&self) -> bool {
        *self == CanvasSettings::default()
    }
    pub fn sanitize(&mut self) {
        self.sort_order = self.sort_order.clamp(SORT_MIN, SORT_MAX);
        self.plane_distance = if self.plane_distance.is_finite() { self.plane_distance.clamp(0.01, 100_000.0) } else { 100.0 };
    }
}

// ---------------------------------------------------------------------------
// Doc: Unity settings
// ---------------------------------------------------------------------------

/// Which outputs the exporter generates.
///
/// `imgui` is `None` when the file has no `unity` block (old designs): the IMGUI script is then
/// generated as it always was. Use [`GenerateOutputs::imgui_enabled`]. Inside an explicit `unity` block the
/// default is `false`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GenerateOutputs {
    #[serde(default = "yes")]
    pub prefab: bool,
    #[serde(default = "yes")]
    pub uitk: bool,
    /// Always written as an explicit true/false when the `unity` block is serialized, so an old design (no block,
    /// IMGUI on) that is touched keeps IMGUI on after save / reload.
    #[serde(default = "no", serialize_with = "ser_imgui")]
    pub imgui: Option<bool>,
    #[serde(default = "yes")]
    pub view_scripts: bool,
}

impl Default for GenerateOutputs {
    /// Legacy behaviour (no `unity` block): everything including IMGUI.
    fn default() -> Self {
        GenerateOutputs { prefab: true, uitk: true, imgui: None, view_scripts: true }
    }
}

impl GenerateOutputs {
    /// Defaults of a new design: IMGUI off.
    pub fn new_design() -> Self {
        GenerateOutputs { prefab: true, uitk: true, imgui: Some(false), view_scripts: true }
    }
    pub fn imgui_enabled(&self) -> bool {
        self.imgui.unwrap_or(true)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct UnitySettings {
    /// Folder (relative to the Unity project, starting with `Assets`) where the generated scripts go.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scripts_dir: Option<String>,
    /// Name of an existing assembly definition the generated scripts live in / refer to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assembly: Option<String>,
    #[serde(default = "GenerateOutputs::new_design")]
    pub generate: GenerateOutputs,
    /// Name of a TextMeshPro font asset in the Unity project.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_font: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fallback_fonts: Vec<String>,
}

impl Default for UnitySettings {
    fn default() -> Self {
        UnitySettings { scripts_dir: None, assembly: None, generate: GenerateOutputs::default(), default_font: None, fallback_fonts: Vec::new() }
    }
}

impl UnitySettings {
    pub fn is_default(&self) -> bool {
        *self == UnitySettings::default()
    }
    pub fn sanitize(&mut self) {
        self.scripts_dir = self.scripts_dir.take().and_then(|s| normalize_scripts_dir(&s));
        self.assembly = self.assembly.take().filter(|s| valid_assembly(s));
        self.default_font = self.default_font.take().and_then(|s| clean_label(&s, 128)).filter(|s| valid_font_name(s));
        self.fallback_fonts = std::mem::take(&mut self.fallback_fonts).into_iter().filter_map(|s| clean_label(&s, 128)).filter(|s| valid_font_name(s)).take(16).collect();
    }
}

// ---------------------------------------------------------------------------
// Doc: portrait layout
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    Landscape,
    Portrait,
}

/// Values an element takes in the portrait layout. Every field is optional: `None` inherits the landscape value.
/// Geometry (`x`, `y`, `w`, `h`, `rot`) is in page pixels of the portrait page, like `Element`.
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ElementOverride {
    pub id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub w: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rot: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_min: Option<[f32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_max: Option<[f32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pivot: Option<[f32; 2]>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<LayoutGroup>,
}

impl ElementOverride {
    pub fn new(id: u64) -> Self {
        ElementOverride { id, ..Default::default() }
    }
    /// True when it overrides nothing.
    pub fn is_empty(&self) -> bool {
        *self == ElementOverride::new(self.id)
    }
    /// Clamps every value; non-finite values are dropped (inherit).
    pub fn sanitize(&mut self) {
        fn num(v: &mut Option<f32>, lo: f32, hi: f32) {
            *v = v.filter(|x| x.is_finite()).map(|x| x.clamp(lo, hi));
        }
        fn pair(v: &mut Option<[f32; 2]>) {
            if let Some(a) = v {
                if a.iter().any(|x| !x.is_finite()) {
                    *v = None;
                } else {
                    *a = a.map(|x| x.clamp(-crate::model::ANCHOR_LIMIT, crate::model::ANCHOR_LIMIT));
                }
            }
        }
        num(&mut self.x, -crate::model::COORD_LIMIT, crate::model::COORD_LIMIT);
        num(&mut self.y, -crate::model::COORD_LIMIT, crate::model::COORD_LIMIT);
        num(&mut self.w, 0.0, crate::model::COORD_LIMIT);
        num(&mut self.h, 0.0, crate::model::COORD_LIMIT);
        num(&mut self.rot, -360.0, 360.0);
        num(&mut self.font_size, 1.0, 1000.0);
        pair(&mut self.anchor_min);
        pair(&mut self.anchor_max);
        pair(&mut self.pivot);
        if let Some(l) = self.layout.as_mut() {
            sanitize_layout(l);
        }
    }
    /// Writes the overridden values onto `e`.
    pub fn apply(&self, e: &mut Element) {
        if let Some(v) = self.x {
            e.x = v;
        }
        if let Some(v) = self.y {
            e.y = v;
        }
        if let Some(v) = self.w {
            e.w = v;
        }
        if let Some(v) = self.h {
            e.h = v;
        }
        if let Some(v) = self.rot {
            e.rot = v;
        }
        if let Some(v) = self.anchor_min {
            e.anchor_min = v;
        }
        if let Some(v) = self.anchor_max {
            e.anchor_max = v;
        }
        if let Some(v) = self.pivot {
            e.pivot = v;
        }
        if let Some(v) = self.visible {
            e.visible = v;
        }
        if let Some(v) = self.font_size {
            e.font_size = v;
        }
        if let Some(v) = &self.layout {
            e.layout = v.clone();
        }
    }
}

pub fn sanitize_layout(l: &mut LayoutGroup) {
    for v in l.padding.iter_mut().chain(l.spacing.iter_mut()) {
        *v = if v.is_finite() { v.clamp(-10_000.0, 10_000.0) } else { 0.0 };
    }
    for v in l.cell.iter_mut() {
        *v = if v.is_finite() { v.clamp(1.0, 10_000.0) } else { 100.0 };
    }
    l.align = l.align.min(8);
}

/// The second layout, used when the screen is portrait.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PortraitLayout {
    /// Reference resolution of the portrait canvas.
    pub width: f32,
    pub height: f32,
    pub overrides: Vec<ElementOverride>,
}

impl Default for PortraitLayout {
    fn default() -> Self {
        PortraitLayout { width: 1080.0, height: 1920.0, overrides: Vec::new() }
    }
}

// ---------------------------------------------------------------------------
// Validators
// ---------------------------------------------------------------------------

fn bad_char(c: char) -> bool {
    c.is_control() || matches!(c, '\u{2028}' | '\u{2029}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}')
}

/// Removes control / line-separator characters and caps the length. `None` if nothing visible is left.
pub fn clean_label(s: &str, max: usize) -> Option<String> {
    let t: String = s.chars().filter(|c| !bad_char(*c)).take(max).collect();
    if t.trim().is_empty() { None } else { Some(t) }
}

pub const MAX_UNITY_NAME: usize = 128;

/// A GameObject name the exporter can use verbatim: not blank, no `/` (Transform.Find separator),
/// no control or line-separator characters, at most 128 characters, and not one of the names the importer
/// generates itself (`__border`, `__text`, `__tick`, `__safe_area`, `__canvas_background`).
pub fn valid_unity_name(s: &str) -> bool {
    !s.trim().is_empty() && s.chars().count() <= MAX_UNITY_NAME && !s.contains('/') && !s.chars().any(bad_char) && !is_generated_name(s)
}

/// One of the GameObject names the importer creates for its own helper objects.
pub fn is_generated_name(s: &str) -> bool {
    crate::export::GEN_NAMES.contains(&s)
}

/// A font family name that cannot be turned into a path: no slash, backslash, colon or `..`.
pub fn valid_font_name(s: &str) -> bool {
    !s.contains(['/', '\\', ':']) && !s.contains("..")
}

/// Assembly definition name: `[A-Za-z_][A-Za-z0-9_.-]*`, at most 128 characters.
pub fn valid_assembly(s: &str) -> bool {
    let mut c = s.chars();
    matches!(c.next(), Some(f) if f.is_ascii_alphabetic() || f == '_') && s.len() <= 128 && c.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

/// Validates a scripts folder: relative, inside `Assets`, no `..`, no drive, no empty segments.
/// Returns the normalised form (forward slashes, no trailing slash).
pub fn normalize_scripts_dir(s: &str) -> Option<String> {
    let t = s.trim().replace('\\', "/");
    let t = t.trim_end_matches('/');
    if t.is_empty() || t.starts_with('/') || t.contains(':') || t.chars().any(bad_char) || t.len() > 260 {
        return None;
    }
    let parts: Vec<&str> = t.split('/').collect();
    if parts[0] != "Assets" || parts.iter().any(|p| p.is_empty() || *p == "." || *p == ".." || p.ends_with(' ') || p.ends_with('.') || p.contains(['<', '>', '"', '|', '?', '*']) || reserved_device_name(p)) {
        return None;
    }
    Some(t.to_string())
}

/// `CON`, `PRN`, `AUX`, `NUL`, `COM1`-`COM9`, `LPT1`-`LPT9` (also with an extension: `CON.txt`) are not usable file names on Windows.
pub fn reserved_device_name(segment: &str) -> bool {
    let stem = segment.split('.').next().unwrap_or("").trim_end().to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "CONIN$" | "CONOUT$")
        || ((stem.starts_with("COM") || stem.starts_with("LPT")) && stem.len() == 4 && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

pub fn valid_scripts_dir(s: &str) -> bool {
    normalize_scripts_dir(s).as_deref() == Some(s)
}

// ---------------------------------------------------------------------------
// Element helpers
// ---------------------------------------------------------------------------

impl Element {
    /// Image type that Unity gets: the explicit `image_mode`, or Sliced when a `slice` is set.
    pub fn effective_image_mode(&self) -> ImageMode {
        self.image_mode.unwrap_or(if self.slice.is_some() { ImageMode::Sliced } else { ImageMode::Simple })
    }

    /// Clamps / cleans the optional fields added to the format.
    pub fn sanitize_ext(&mut self) {
        self.unity_name = self.unity_name.take().and_then(|s| clean_label(&s, 256));
        self.sprite = self.sprite.take().and_then(|s| clean_label(&s, 256));
        self.slice = self.slice.take().and_then(|s| {
            let s = s.map(|v| if v.is_finite() { v.clamp(0.0, 100_000.0) } else { 0.0 });
            if s.iter().all(|v| *v == 0.0) { None } else { Some(s) }
        });
        self.ppu = self.ppu.take().filter(|v| v.is_finite()).map(|v| v.clamp(0.01, 10_000.0));
        if !valid_font_name(&self.font) {
            self.font.clear();
        }
    }
}

// ---------------------------------------------------------------------------
// Doc helpers
// ---------------------------------------------------------------------------

impl Doc {
    /// False when the page background is fully transparent: exporters must not emit a canvas background.
    pub fn has_background(&self) -> bool {
        self.background[3] > 0
    }

    /// Parses a design file: strips a BOM, fills defaults, repairs ids / parents (`migrate_groups`).
    pub fn from_json_str(text: &str) -> Result<Doc, String> {
        let t = text.strip_prefix('\u{feff}').unwrap_or(text);
        let mut d: Doc = serde_json::from_str(t).map_err(|e| e.to_string())?;
        if d.elements.len() > crate::model::MAX_ELEMENTS {
            return Err(format!("the design has {} elements (the limit is {})", d.elements.len(), crate::model::MAX_ELEMENTS));
        }
        d.migrate_groups();
        Ok(d)
    }

    pub fn override_of(&self, id: u64) -> Option<&ElementOverride> {
        self.portrait.as_ref()?.overrides.iter().find(|o| o.id == id)
    }

    /// The design as the game sees it in the given orientation: a clone with the portrait page size and
    /// overrides applied (portrait), or unchanged (landscape). The result has no `portrait` block.
    ///
    /// Emulates Unity: an element keeps its landscape RectTransform (anchors, pivot, offsets) and is re-evaluated
    /// against its parent's portrait box, parents first, so a full-page background stretches and right-anchored
    /// elements follow the right edge. The override values of an element (box, rotation, anchors, pivot, visibility,
    /// font size, layout) are written over that result; children then follow the resolved box of their parent.
    /// O(n), iterative.
    pub fn resolved_for(&self, orientation: Orientation) -> Doc {
        let mut d = self.clone();
        if orientation == Orientation::Portrait {
            if let Some(p) = &self.portrait {
                d.width = p.width;
                d.height = p.height;
                let n = self.elements.len();
                let idx = self.index_map();
                let pidx = self.parent_indices(&idx);
                let mut ov: Vec<Option<&ElementOverride>> = vec![None; n];
                for o in &p.overrides {
                    if let Some(&i) = idx.get(&o.id) {
                        if ov[i].is_none() {
                            ov[i] = Some(o);
                        }
                    }
                }
                let mut kids: Vec<Vec<usize>> = vec![Vec::new(); n];
                let mut stack: Vec<usize> = Vec::new();
                for i in (0..n).rev() {
                    match pidx[i] {
                        Some(q) => kids[q].push(i),
                        None => stack.push(i),
                    }
                }
                let (land_page, port_page) = (self.page_frame(), d.page_frame());
                let mut done = vec![false; n];
                while let Some(i) = stack.pop() {
                    if std::mem::replace(&mut done[i], true) {
                        continue;
                    }
                    let (lf, pf) = match pidx[i] {
                        Some(q) => (self.elements[q].frame(), d.elements[q].frame()),
                        None => (land_page, port_page),
                    };
                    let rt = crate::model::rect_transform_in(&self.elements[i], &lf);
                    crate::model::apply_rect_transform(&mut d.elements[i], &rt, &pf);
                    if let Some(o) = ov[i] {
                        o.apply(&mut d.elements[i]);
                    }
                    for &c in kids[i].iter().rev() {
                        stack.push(c);
                    }
                }
                // elements in a parent cycle (never reached from a root): overrides only
                for i in 0..n {
                    if !done[i] {
                        if let Some(o) = ov[i] {
                            o.apply(&mut d.elements[i]);
                        }
                    }
                }
                // Layout groups of the portrait page place their children like Unity does.
                d.apply_layouts();
            }
        }
        d.portrait = None;
        d
    }

    /// Creates / replaces the portrait override of `ov.id` (an empty override removes it).
    /// Creates the portrait layout (height x width of the landscape page) if missing. False if the element does not exist.
    pub fn set_override(&mut self, mut ov: ElementOverride) -> bool {
        if self.get(ov.id).is_none() {
            return false;
        }
        ov.sanitize();
        if ov.is_empty() {
            return self.clear_override(ov.id);
        }
        let (w, h) = (self.height, self.width);
        let p = self.portrait.get_or_insert_with(|| PortraitLayout { width: w, height: h, overrides: Vec::new() });
        match p.overrides.iter_mut().find(|o| o.id == ov.id) {
            Some(slot) => *slot = ov,
            None => p.overrides.push(ov),
        }
        true
    }

    /// Copies the portrait overrides of the elements in `map` (old id -> new id) to their copies, moving the
    /// copied x / y by `(dx, dy)`. Used when elements are duplicated or pasted in the landscape view.
    pub fn copy_overrides(&mut self, overrides: &[ElementOverride], map: &std::collections::HashMap<u64, u64>, dx: f32, dy: f32) {
        if self.portrait.is_none() {
            return;
        }
        let mut copies = Vec::new();
        for o in overrides {
            if let Some(&n) = map.get(&o.id) {
                let mut c = o.clone();
                c.id = n;
                c.x = c.x.map(|v| v + dx);
                c.y = c.y.map(|v| v + dy);
                copies.push(c);
            }
        }
        if let Some(p) = self.portrait.as_mut() {
            p.overrides.extend(copies);
        }
        self.prune_overrides();
    }

    /// The portrait overrides of `ids` (for copy / paste).
    pub fn overrides_of(&self, ids: &[u64]) -> Vec<ElementOverride> {
        self.portrait.as_ref().map(|p| p.overrides.iter().filter(|o| ids.contains(&o.id)).cloned().collect()).unwrap_or_default()
    }

    /// Removes the portrait override of `id`. True if there was one.
    pub fn clear_override(&mut self, id: u64) -> bool {
        let Some(p) = self.portrait.as_mut() else { return false };
        let n = p.overrides.len();
        p.overrides.retain(|o| o.id != id);
        p.overrides.len() != n
    }

    /// Drops overrides of deleted elements and duplicates (first wins).
    pub fn prune_overrides(&mut self) {
        let ids: HashSet<u64> = self.elements.iter().map(|e| e.id).collect();
        if let Some(p) = self.portrait.as_mut() {
            let mut seen = HashSet::new();
            p.overrides.retain(|o| ids.contains(&o.id) && seen.insert(o.id));
        }
    }

    /// Clamps / cleans the optional Doc fields and every element's added fields.
    pub fn sanitize_ext(&mut self) {
        self.canvas.sanitize();
        self.unity.sanitize();
        for e in self.elements.iter_mut() {
            e.sanitize_ext();
        }
        if let Some(p) = self.portrait.as_mut() {
            let fix = |v: f32, d: f32| if v.is_finite() && v >= 1.0 { v.min(16_384.0) } else { d };
            p.width = fix(p.width, 1080.0);
            p.height = fix(p.height, 1920.0);
            for o in p.overrides.iter_mut() {
                o.sanitize();
            }
        }
        self.prune_overrides();
    }
}
