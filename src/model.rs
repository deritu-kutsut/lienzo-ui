//! Document model: a page (the Unity Canvas) holding a tree of elements, like
//! the RectTransform hierarchy of a uGUI prefab.
//!
//! Elements store their *world* box (page pixels, y grows downward) so the
//! editor can move, resize and rotate them directly. Their Unity values
//! (anchors, pivot, anchoredPosition, sizeDelta) are computed relative to the
//! parent's box. `Doc::elements` is always kept in depth-first order: a parent
//! comes before its children and siblings are drawn in list order (the last
//! one on top), exactly like the sibling order of a Unity hierarchy.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Kind {
    Panel,
    Ellipse,
    Image,
    Text,
    Button,
    Toggle,
    Slider,
    InputField,
    Dropdown,
    ScrollView,
    Empty,
}

pub const ALL_KINDS: [Kind; 11] = [
    Kind::Empty,
    Kind::Panel,
    Kind::Image,
    Kind::Ellipse,
    Kind::Text,
    Kind::Button,
    Kind::Toggle,
    Kind::Slider,
    Kind::InputField,
    Kind::Dropdown,
    Kind::ScrollView,
];

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Panel => "Panel",
            Kind::Ellipse => "Circle",
            Kind::Image => "Image",
            Kind::Text => "Text",
            Kind::Button => "Button",
            Kind::Toggle => "Toggle",
            Kind::Slider => "Slider",
            Kind::InputField => "Input field",
            Kind::Dropdown => "Dropdown",
            Kind::ScrollView => "Scroll view",
            Kind::Empty => "Container",
        }
    }
    pub fn unity_type(self) -> &'static str {
        match self {
            Kind::Panel => "Panel",
            Kind::Ellipse => "Image",
            Kind::Image => "Image",
            Kind::Text => "Text",
            Kind::Button => "Button",
            Kind::Toggle => "Toggle",
            Kind::Slider => "Slider",
            Kind::InputField => "InputField",
            Kind::Dropdown => "Dropdown",
            Kind::ScrollView => "ScrollView",
            Kind::Empty => "Empty",
        }
    }
    /// Component shown in the inspector (what Unity will create).
    pub fn unity_component(self) -> &'static str {
        match self {
            Kind::Panel | Kind::Ellipse | Kind::Image => "Image",
            Kind::Text => "TextMeshProUGUI",
            Kind::Button => "Button + Image",
            Kind::Toggle => "Toggle",
            Kind::Slider => "Slider",
            Kind::InputField => "TMP_InputField",
            Kind::Dropdown => "TMP_Dropdown",
            Kind::ScrollView => "ScrollRect",
            Kind::Empty => "RectTransform",
        }
    }
    /// Short glyph for the hierarchy.
    pub fn icon(self) -> &'static str {
        match self {
            Kind::Panel => "▭",
            Kind::Ellipse => "○",
            Kind::Image => "▣",
            Kind::Text => "T",
            Kind::Button => "◉",
            Kind::Toggle => "☑",
            Kind::Slider => "═",
            Kind::InputField => "⌨",
            Kind::Dropdown => "☰",
            Kind::ScrollView => "⇕",
            Kind::Empty => "⬚",
        }
    }
    /// Kinds that can show a label/caption.
    pub fn has_text(self) -> bool {
        !matches!(self, Kind::Slider | Kind::ScrollView | Kind::Empty)
    }
    /// Kinds with a background Image (fill colour, corners, border).
    pub fn has_fill(self) -> bool {
        !matches!(self, Kind::Text | Kind::Empty)
    }
    /// Unity Selectables (have `interactable`).
    pub fn selectable(self) -> bool {
        matches!(self, Kind::Button | Kind::Toggle | Kind::Slider | Kind::InputField | Kind::Dropdown)
    }
    /// Corner handles keep the proportions for these kinds (side handles always stretch).
    pub fn keeps_aspect_on_corners(self) -> bool {
        matches!(self, Kind::Image | Kind::Text | Kind::Ellipse)
    }
}

pub type Rgba = [u8; 4];

fn one_u8() -> u8 {
    1
}
fn yes() -> bool {
    true
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LayoutMode {
    #[default]
    None,
    Horizontal,
    Vertical,
    Grid,
}

impl LayoutMode {
    pub fn label(self) -> &'static str {
        match self {
            LayoutMode::None => "None",
            LayoutMode::Horizontal => "Horizontal Layout Group",
            LayoutMode::Vertical => "Vertical Layout Group",
            LayoutMode::Grid => "Grid Layout Group",
        }
    }
    pub fn index(self) -> i32 {
        match self {
            LayoutMode::None => 0,
            LayoutMode::Horizontal => 1,
            LayoutMode::Vertical => 2,
            LayoutMode::Grid => 3,
        }
    }
}

/// Unity Horizontal / Vertical / Grid Layout Group applied to the direct children.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutGroup {
    pub mode: LayoutMode,
    /// left, right, top, bottom
    pub padding: [f32; 4],
    pub spacing: [f32; 2],
    /// Unity TextAnchor: 0 UpperLeft … 8 LowerRight (row-major)
    pub align: u8,
    pub cell: [f32; 2],
    /// Control child size + child force expand (children share the space evenly)
    pub expand_w: bool,
    pub expand_h: bool,
}

impl Default for LayoutGroup {
    fn default() -> Self {
        LayoutGroup { mode: LayoutMode::None, padding: [16.0; 4], spacing: [12.0, 12.0], align: 0, cell: [100.0, 100.0], expand_w: false, expand_h: false }
    }
}

/// uGUI `Shadow` effect. Offset uses Unity's convention (x right, y up).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Shadow {
    pub on: bool,
    pub color: Rgba,
    pub offset: [f32; 2],
}

impl Default for Shadow {
    fn default() -> Self {
        Shadow { on: false, color: [0, 0, 0, 110], offset: [6.0, -6.0] }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Element {
    pub id: u64,
    pub name: String,
    pub kind: Kind,
    /// Top-left corner of the unrotated box, in page pixels (y grows downward).
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    /// Clockwise rotation around the box centre, degrees (world).
    pub rot: f32,
    pub fill: Rgba,
    pub fill2: Rgba,
    pub stroke: Rgba,
    pub stroke_w: f32,
    pub radius: f32,
    pub opacity: f32,
    pub text: String,
    pub font_size: f32,
    pub text_color: Rgba,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    /// 0 = left, 1 = centre, 2 = right
    pub align: u8,
    /// 0 = top, 1 = middle, 2 = bottom
    #[serde(default = "one_u8")]
    pub valign: u8,
    /// Font family name ("" = default font).
    #[serde(default)]
    pub font: String,
    /// Code run when the element is clicked in test mode.
    #[serde(default)]
    pub on_click: String,
    /// Uploaded picture shown by an Image element (library file name or absolute path).
    #[serde(default)]
    pub image: String,
    pub value: f32,
    pub visible: bool,
    pub locked: bool,
    /// Old flat "groups" (pre-hierarchy files). Converted to Empty parents on load.
    #[serde(default, skip_serializing)]
    pub group: Option<u64>,
    /// Parent element (None = child of the Canvas).
    #[serde(default)]
    pub parent: Option<u64>,
    /// Unity RectTransform anchors / pivot, normalised, y grows UPWARD (Unity convention),
    /// relative to the parent's rectangle.
    pub anchor_min: [f32; 2],
    pub anchor_max: [f32; 2],
    pub pivot: [f32; 2],
    /// Dropdown options.
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default = "yes")]
    pub interactable: bool,
    #[serde(default = "yes")]
    pub raycast: bool,
    /// RectMask2D: children are clipped to this box.
    #[serde(default)]
    pub mask: bool,
    #[serde(default)]
    pub shadow: Shadow,
    #[serde(default)]
    pub layout: LayoutGroup,
    /// Hierarchy row collapsed (editor only).
    #[serde(default)]
    pub collapsed: bool,
    /// Exact GameObject name to export instead of the display `name`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unity_name: Option<String>,
    /// 9-slice border in source-image pixels: left, bottom, right, top.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slice: Option<[f32; 4]>,
    /// Sprite pixels per unit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ppu: Option<f32>,
    /// Sub-sprite name inside a multi-sprite texture / atlas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sprite: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_mode: Option<crate::docext::ImageMode>,
    #[serde(skip_serializing_if = "crate::docext::is_false")]
    pub preserve_aspect: bool,
}

impl Default for Element {
    /// What a missing field of a hand-written file gets (the kind-neutral defaults of `Element::new`).
    fn default() -> Self {
        Element::base(0, Kind::Panel)
    }
}

/// A box in page space: centre, size and clockwise rotation (degrees).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub cx: f32,
    pub cy: f32,
    pub w: f32,
    pub h: f32,
    pub rot: f32,
}

impl Frame {
    pub fn page(w: f32, h: f32) -> Frame {
        Frame { cx: w / 2.0, cy: h / 2.0, w, h, rot: 0.0 }
    }
    /// World point → frame-local (relative to the centre, unrotated, y down).
    pub fn to_local(&self, p: [f32; 2]) -> [f32; 2] {
        let (s, c) = (-self.rot).to_radians().sin_cos();
        let (dx, dy) = (p[0] - self.cx, p[1] - self.cy);
        [dx * c - dy * s, dx * s + dy * c]
    }
    pub fn to_world(&self, l: [f32; 2]) -> [f32; 2] {
        let (s, c) = self.rot.to_radians().sin_cos();
        [self.cx + l[0] * c - l[1] * s, self.cy + l[0] * s + l[1] * c]
    }
    pub fn contains(&self, p: [f32; 2], tol: f32) -> bool {
        let l = self.to_local(p);
        l[0].abs() <= self.w / 2.0 + tol && l[1].abs() <= self.h / 2.0 + tol
    }
    pub fn close_to(&self, o: &Frame) -> bool {
        (self.cx - o.cx).abs() < 0.01 && (self.cy - o.cy).abs() < 0.01 && (self.w - o.w).abs() < 0.01 && (self.h - o.h).abs() < 0.01 && (self.rot - o.rot).abs() < 0.01
    }
    /// Axis-aligned bounds (min x, min y, max x, max y).
    pub fn aabb(&self) -> [f32; 4] {
        let (s, c) = self.rot.to_radians().sin_cos();
        let ex = (self.w / 2.0 * c).abs() + (self.h / 2.0 * s).abs();
        let ey = (self.w / 2.0 * s).abs() + (self.h / 2.0 * c).abs();
        [self.cx - ex, self.cy - ey, self.cx + ex, self.cy + ey]
    }
}

pub fn norm_angle(a: f32) -> f32 {
    let mut a = a % 360.0;
    if a > 180.0 {
        a -= 360.0;
    }
    if a <= -180.0 {
        a += 360.0;
    }
    a
}

impl Element {
    pub fn new(id: u64, kind: Kind) -> Self {
        let mut e = Element::base(id, kind);
        e.apply_kind_defaults();
        e
    }

    fn base(id: u64, kind: Kind) -> Self {
        Element {
            id,
            name: format!("{} {}", kind.label(), id),
            kind,
            x: 0.0,
            y: 0.0,
            w: 300.0,
            h: 200.0,
            rot: 0.0,
            fill: [124, 92, 255, 255],
            fill2: [0, 196, 204, 255],
            stroke: [30, 30, 40, 255],
            stroke_w: 0.0,
            radius: 0.0,
            opacity: 1.0,
            text: String::new(),
            font_size: 32.0,
            text_color: [20, 20, 30, 255],
            bold: false,
            italic: false,
            underline: false,
            align: 1,
            valign: 1,
            font: String::new(),
            on_click: String::new(),
            image: String::new(),
            value: 0.5,
            visible: true,
            locked: false,
            group: None,
            parent: None,
            anchor_min: [0.5, 0.5],
            anchor_max: [0.5, 0.5],
            pivot: [0.5, 0.5],
            options: Vec::new(),
            interactable: true,
            raycast: true,
            mask: false,
            shadow: Shadow::default(),
            layout: LayoutGroup::default(),
            collapsed: false,
            unity_name: None,
            slice: None,
            ppu: None,
            sprite: None,
            image_mode: None,
            preserve_aspect: false,
        }
    }

    fn apply_kind_defaults(&mut self) {
        let e = self;
        match e.kind {
            Kind::Panel => {
                e.fill = [255, 255, 255, 235];
                e.radius = 24.0;
                e.w = 520.0;
                e.h = 360.0;
            }
            Kind::Ellipse => {
                e.fill = [225, 95, 30, 255];
                e.w = 200.0;
                e.h = 200.0;
            }
            Kind::Image => {
                e.fill = [255, 126, 179, 255];
                e.fill2 = [255, 117, 140, 255];
                e.w = 320.0;
                e.h = 220.0;
                e.radius = 12.0;
            }
            Kind::Text => {
                e.text = "Type your text".into();
                e.font_size = 40.0;
                e.w = 420.0;
                e.h = 60.0;
                e.fill = [0, 0, 0, 0];
                e.raycast = false;
            }
            Kind::Button => {
                e.text = "Button".into();
                e.font_size = 30.0;
                e.text_color = [255, 255, 255, 255];
                e.bold = true;
                e.fill = [124, 92, 255, 255];
                e.radius = 18.0;
                e.w = 320.0;
                e.h = 84.0;
            }
            Kind::Toggle => {
                e.text = "Option".into();
                e.font_size = 28.0;
                e.align = 0;
                e.fill = [124, 92, 255, 255];
                e.value = 1.0;
                e.w = 300.0;
                e.h = 50.0;
            }
            Kind::Slider => {
                e.fill = [124, 92, 255, 255];
                e.fill2 = [222, 222, 232, 255];
                e.w = 400.0;
                e.h = 40.0;
                e.value = 0.6;
            }
            Kind::InputField => {
                e.text = "Type here...".into();
                e.font_size = 26.0;
                e.align = 0;
                e.text_color = [140, 140, 150, 255];
                e.fill = [255, 255, 255, 255];
                e.stroke = [200, 200, 210, 255];
                e.stroke_w = 2.0;
                e.radius = 10.0;
                e.w = 420.0;
                e.h = 70.0;
            }
            Kind::Dropdown => {
                e.options = vec!["Option A".into(), "Option B".into(), "Option C".into()];
                e.text = "Option A".into();
                e.value = 0.0;
                e.font_size = 26.0;
                e.align = 0;
                e.fill = [255, 255, 255, 255];
                e.stroke = [200, 200, 210, 255];
                e.stroke_w = 2.0;
                e.radius = 10.0;
                e.w = 380.0;
                e.h = 70.0;
            }
            Kind::ScrollView => {
                e.fill = [245, 246, 250, 255];
                e.stroke = [210, 212, 222, 255];
                e.stroke_w = 2.0;
                e.radius = 14.0;
                e.w = 520.0;
                e.h = 380.0;
                e.mask = true;
            }
            Kind::Empty => {
                e.fill = [0, 0, 0, 0];
                e.w = 400.0;
                e.h = 300.0;
                e.raycast = false;
            }
        }
    }

    pub fn center(&self) -> [f32; 2] {
        [self.x + self.w / 2.0, self.y + self.h / 2.0]
    }

    pub fn frame(&self) -> Frame {
        let [cx, cy] = self.center();
        Frame { cx, cy, w: self.w, h: self.h, rot: self.rot }
    }

    pub fn set_frame(&mut self, f: &Frame) {
        self.w = f.w;
        self.h = f.h;
        self.x = f.cx - f.w / 2.0;
        self.y = f.cy - f.h / 2.0;
        self.rot = f.rot;
    }

    /// Axis-aligned bounds of the rotated box (min x, min y, max x, max y).
    pub fn aabb(&self) -> [f32; 4] {
        self.frame().aabb()
    }

    /// Is the page point inside the (rotated) box?
    pub fn contains(&self, px: f32, py: f32, tol: f32) -> bool {
        self.frame().contains([px, py], tol)
    }

    /// Caption shown by a dropdown (its selected option).
    pub fn caption(&self) -> &str {
        if self.kind == Kind::Dropdown && !self.options.is_empty() {
            let i = (self.value.round().max(0.0) as usize).min(self.options.len() - 1);
            &self.options[i]
        } else {
            &self.text
        }
    }
}

// ---------------------------------------------------------------------------
// RectTransform math
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RectTransform {
    pub anchor_min: [f32; 2],
    pub anchor_max: [f32; 2],
    pub pivot: [f32; 2],
    pub anchored_position: [f32; 2],
    pub size_delta: [f32; 2],
    pub rotation_z: f32,
}

impl RectTransform {
    pub fn stretch_x(&self) -> bool {
        (self.anchor_max[0] - self.anchor_min[0]).abs() > 1e-4
    }
    pub fn stretch_y(&self) -> bool {
        (self.anchor_max[1] - self.anchor_min[1]).abs() > 1e-4
    }
    /// Unity offsetMin (left, bottom).
    pub fn offset_min(&self) -> [f32; 2] {
        [self.anchored_position[0] - self.size_delta[0] * self.pivot[0], self.anchored_position[1] - self.size_delta[1] * self.pivot[1]]
    }
    /// Unity offsetMax (-right, -top).
    pub fn offset_max(&self) -> [f32; 2] {
        [self.anchored_position[0] + self.size_delta[0] * (1.0 - self.pivot[0]), self.anchored_position[1] + self.size_delta[1] * (1.0 - self.pivot[1])]
    }
    pub fn set_offsets(&mut self, min: [f32; 2], max: [f32; 2]) {
        for a in 0..2 {
            self.size_delta[a] = max[a] - min[a];
            self.anchored_position[a] = min[a] + self.size_delta[a] * self.pivot[a];
        }
    }
}

/// Unity values of `e` inside the parent box `p`.
pub fn rect_transform_in(e: &Element, p: &Frame) -> RectTransform {
    let lc = p.to_local(e.center());
    // parent space with origin at the bottom-left corner, y up
    let ux = lc[0] + p.w / 2.0;
    let uy = p.h / 2.0 - lc[1];
    let rl = e.rot - p.rot;
    let (s, c) = rl.to_radians().sin_cos();
    let ox = (e.pivot[0] - 0.5) * e.w;
    let oy = (0.5 - e.pivot[1]) * e.h; // y down
    let px = ux + (ox * c - oy * s);
    let py = uy - (ox * s + oy * c);
    let (ax0, ax1) = (e.anchor_min[0] * p.w, e.anchor_max[0] * p.w);
    let (ay0, ay1) = (e.anchor_min[1] * p.h, e.anchor_max[1] * p.h);
    let ref_x = ax0 + e.pivot[0] * (ax1 - ax0);
    let ref_y = ay0 + e.pivot[1] * (ay1 - ay0);
    RectTransform {
        anchor_min: e.anchor_min,
        anchor_max: e.anchor_max,
        pivot: e.pivot,
        anchored_position: [px - ref_x, py - ref_y],
        size_delta: [e.w - (ax1 - ax0), e.h - (ay1 - ay0)],
        rotation_z: -norm_angle(rl) + 0.0,
    }
}

/// Inverse of [`rect_transform_in`]: place `e` in the parent box `p` from Unity values.
pub fn apply_rect_transform(e: &mut Element, rt: &RectTransform, p: &Frame) {
    e.anchor_min = rt.anchor_min;
    e.anchor_max = rt.anchor_max;
    e.pivot = rt.pivot;
    let (ax0, ax1) = (rt.anchor_min[0] * p.w, rt.anchor_max[0] * p.w);
    let (ay0, ay1) = (rt.anchor_min[1] * p.h, rt.anchor_max[1] * p.h);
    let w = (rt.size_delta[0] + ax1 - ax0).max(0.0);
    let h = (rt.size_delta[1] + ay1 - ay0).max(0.0);
    let px = ax0 + rt.pivot[0] * (ax1 - ax0) + rt.anchored_position[0];
    let py = ay0 + rt.pivot[1] * (ay1 - ay0) + rt.anchored_position[1];
    let rl = -rt.rotation_z;
    let (s, c) = rl.to_radians().sin_cos();
    let ox = (rt.pivot[0] - 0.5) * w;
    let oy = (0.5 - rt.pivot[1]) * h;
    let ux = px - (ox * c - oy * s);
    let uy = py + (ox * s + oy * c);
    let wc = p.to_world([ux - p.w / 2.0, p.h / 2.0 - uy]);
    e.w = w;
    e.h = h;
    e.x = wc[0] - w / 2.0;
    e.y = wc[1] - h / 2.0;
    e.rot = norm_angle(rl + p.rot);
}

/// RectTransform of `e` under a page of the given size (root elements only).
#[cfg_attr(not(test), allow(dead_code))]
pub fn rect_transform(e: &Element, page_w: f32, page_h: f32) -> RectTransform {
    rect_transform_in(e, &Frame::page(page_w, page_h))
}

// ---------------------------------------------------------------------------
// Document
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Doc {
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub background: Rgba,
    pub elements: Vec<Element>,
    pub next_id: u64,
    /// CanvasScaler "Match" (0 = width, 1 = height).
    pub canvas_match: f32,
    /// Pixel art mode: unsmoothed images, integer pixel positions and,
    /// in Unity, Canvas "Pixel Perfect" + sprites with Point filter and no compression.
    #[serde(default)]
    pub pixel_art: bool,
    /// Unity Canvas options (render mode, sort order, safe area, embed).
    #[serde(skip_serializing_if = "crate::docext::CanvasSettings::is_default")]
    pub canvas: crate::docext::CanvasSettings,
    /// Unity export options (scripts folder, assembly, outputs, fonts).
    #[serde(skip_serializing_if = "crate::docext::UnitySettings::is_default")]
    pub unity: crate::docext::UnitySettings,
    /// Second layout for portrait screens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portrait: Option<crate::docext::PortraitLayout>,
}

impl Default for Doc {
    /// What a missing field of a hand-written file gets. Keeps the legacy `unity` behaviour (IMGUI output on).
    fn default() -> Self {
        Doc {
            title: "Untitled design".into(),
            width: 1920.0,
            height: 1080.0,
            background: [255, 255, 255, 255],
            elements: Vec::new(),
            next_id: 1,
            canvas_match: 0.5,
            pixel_art: false,
            canvas: Default::default(),
            unity: Default::default(),
            portrait: None,
        }
    }
}

/// Deepest parent chain a design may have (deeper elements are detached to the root when loading / normalising).
pub const MAX_DEPTH: usize = 128;
/// Most elements a design may have.
pub const MAX_ELEMENTS: usize = 20_000;
/// Element ids are kept below 2^52 so every id is an exact integer in JSON (JavaScript, Unity JsonUtility).
pub const MAX_ID: u64 = 1 << 52;
/// Anchors / pivots (and their portrait overrides) are clamped to this absolute value.
pub const ANCHOR_LIMIT: f32 = 1.0e4;
/// Element position / size limit in page pixels.
pub const COORD_LIMIT: f32 = 1.0e6;

impl Doc {
    /// A new design: like the default but IMGUI output is off.
    pub fn empty(width: f32, height: f32) -> Self {
        Doc { width, height, unity: crate::docext::UnitySettings { generate: crate::docext::GenerateOutputs::new_design(), ..Default::default() }, ..Doc::default() }
    }

    /// Append an element (new id). Call [`Doc::normalize`] if it has a parent.
    pub fn add(&mut self, mut e: Element) -> u64 {
        e.id = self.take_id();
        let id = e.id;
        self.elements.push(e);
        id
    }

    /// Hands out a fresh element id (`next_id` saturates well below 2^53).
    pub fn take_id(&mut self) -> u64 {
        let id = self.next_id.clamp(1, MAX_ID * 2);
        self.next_id = (id + 1).min(MAX_ID * 2);
        id
    }

    /// Map element id → position in `elements` (first element wins on duplicate ids).
    pub fn index_map(&self) -> HashMap<u64, usize> {
        let mut m = HashMap::with_capacity(self.elements.len());
        for (i, e) in self.elements.iter().enumerate() {
            m.entry(e.id).or_insert(i);
        }
        m
    }

    /// Parent (as an element position) of every element; `None` for roots and for parents that do not exist.
    pub fn parent_indices(&self, idx: &HashMap<u64, usize>) -> Vec<Option<usize>> {
        self.elements.iter().enumerate().map(|(i, e)| e.parent.and_then(|p| idx.get(&p).copied()).filter(|&p| p != i)).collect()
    }

    /// Direct children of every parent (`None` = root elements), in list order. O(n).
    pub fn child_map(&self) -> HashMap<Option<u64>, Vec<u64>> {
        let ids: HashSet<u64> = self.elements.iter().map(|e| e.id).collect();
        let mut m: HashMap<Option<u64>, Vec<u64>> = HashMap::new();
        for e in &self.elements {
            let p = e.parent.filter(|p| ids.contains(p));
            m.entry(p).or_default().push(e.id);
        }
        m
    }

    pub fn index_of(&self, id: u64) -> Option<usize> {
        self.elements.iter().position(|e| e.id == id)
    }

    pub fn get(&self, id: u64) -> Option<&Element> {
        self.elements.iter().find(|e| e.id == id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Element> {
        self.elements.iter_mut().find(|e| e.id == id)
    }

    pub fn page_frame(&self) -> Frame {
        Frame::page(self.width, self.height)
    }

    /// Box of an element, or the page for `None`.
    pub fn frame_of(&self, id: Option<u64>) -> Frame {
        id.and_then(|i| self.get(i)).map(|e| e.frame()).unwrap_or_else(|| self.page_frame())
    }

    pub fn parent_of(&self, id: u64) -> Option<u64> {
        self.get(id).and_then(|e| e.parent).filter(|p| self.get(*p).is_some())
    }

    pub fn parent_frame(&self, id: u64) -> Frame {
        self.frame_of(self.parent_of(id))
    }

    /// Unity RectTransform of an element relative to its parent.
    #[allow(dead_code)]
    pub fn rt(&self, id: u64) -> Option<RectTransform> {
        let e = self.get(id)?;
        Some(rect_transform_in(e, &self.parent_frame(id)))
    }

    /// Set an element's RectTransform (Unity values) keeping its children attached.
    pub fn set_rt(&mut self, id: u64, rt: &RectTransform) {
        let p = self.parent_frame(id);
        let Some(old) = self.get(id).map(|e| e.frame()) else { return };
        if let Some(e) = self.get_mut(id) {
            apply_rect_transform(e, rt, &p);
        }
        self.reapply_children(Some(id), old);
    }

    /// Direct children in sibling order (`None` = root elements).
    pub fn children(&self, parent: Option<u64>) -> Vec<u64> {
        match parent {
            Some(p) => {
                if self.get(p).is_none() {
                    return Vec::new();
                }
                self.elements.iter().filter(|e| e.parent == Some(p)).map(|e| e.id).collect()
            }
            None => {
                let ids: HashSet<u64> = self.elements.iter().map(|e| e.id).collect();
                self.elements.iter().filter(|e| e.parent.is_none_or(|p| !ids.contains(&p))).map(|e| e.id).collect()
            }
        }
    }

    pub fn has_children(&self, id: u64) -> bool {
        self.elements.iter().any(|e| e.parent == Some(id))
    }

    /// All descendants in depth-first order (not including `id`). Iterative (any depth) and O(n).
    pub fn descendants(&self, id: u64) -> Vec<u64> {
        let map = self.child_map();
        let mut out = Vec::new();
        let mut seen: HashSet<u64> = HashSet::new();
        let mut stack: Vec<u64> = map.get(&Some(id)).map(|v| v.iter().rev().copied().collect()).unwrap_or_default();
        while let Some(c) = stack.pop() {
            // A file with cyclic parents must not hang the app: each element is visited only once.
            if c == id || !seen.insert(c) {
                continue;
            }
            out.push(c);
            if let Some(k) = map.get(&Some(c)) {
                stack.extend(k.iter().rev().copied());
            }
        }
        out
    }

    pub fn subtree(&self, id: u64) -> Vec<u64> {
        let mut v = vec![id];
        v.extend(self.descendants(id));
        v
    }

    /// Parent first, root last.
    pub fn ancestors(&self, id: u64) -> Vec<u64> {
        let mut out = Vec::new();
        let mut cur = self.get(id).and_then(|e| e.parent);
        while let Some(p) = cur {
            let Some(pe) = self.get(p) else { break };
            if out.contains(&p) || out.len() > self.elements.len() {
                break;
            }
            out.push(p);
            cur = pe.parent;
        }
        out
    }

    pub fn is_descendant(&self, id: u64, of: u64) -> bool {
        self.ancestors(id).contains(&of)
    }

    /// Same as `shown` for all elements at once (O(n) instead of O(n²); for drawing every frame).
    pub fn shown_all(&self) -> std::collections::HashSet<u64> {
        use std::collections::HashMap;
        let by_id: HashMap<u64, &Element> = self.elements.iter().map(|e| (e.id, e)).collect();
        let mut memo: HashMap<u64, bool> = HashMap::with_capacity(self.elements.len());
        for e in &self.elements {
            // Walk up to an already resolved ancestor (or the root) and resolve the chain downward.
            let mut chain = Vec::new();
            let mut cur = Some(e.id);
            let mut base = true;
            while let Some(id) = cur {
                if let Some(&v) = memo.get(&id) {
                    base = v;
                    break;
                }
                if chain.contains(&id) || chain.len() > self.elements.len() {
                    break; // cycle in a malformed file
                }
                chain.push(id);
                cur = by_id.get(&id).and_then(|x| x.parent).filter(|p| by_id.contains_key(p));
            }
            for id in chain.into_iter().rev() {
                base = base && by_id.get(&id).map(|x| x.visible).unwrap_or(false);
                memo.insert(id, base);
            }
        }
        memo.into_iter().filter(|(_, v)| *v).map(|(k, _)| k).collect()
    }

    /// Visible and every ancestor visible.
    pub fn shown(&self, id: u64) -> bool {
        self.get(id).map(|e| e.visible).unwrap_or(false) && self.ancestors(id).iter().all(|a| self.get(*a).map(|e| e.visible).unwrap_or(true))
    }

    /// Ids whose ancestors are not also in `ids`, in document order.
    pub fn top_level(&self, ids: &[u64]) -> Vec<u64> {
        self.elements.iter().map(|e| e.id).filter(|id| ids.contains(id) && !self.ancestors(*id).iter().any(|a| ids.contains(a))).collect()
    }

    /// Clip rectangle (page aabb) from masked ancestors (and masked self if `include_self`).
    pub fn clip_of(&self, id: u64) -> Option<[f32; 4]> {
        let mut clip: Option<[f32; 4]> = None;
        for a in self.ancestors(id) {
            if let Some(e) = self.get(a) {
                if e.mask || e.kind == Kind::ScrollView {
                    let b = e.aabb();
                    clip = Some(match clip {
                        None => b,
                        Some(c) => [c[0].max(b[0]), c[1].max(b[1]), c[2].min(b[2]), c[3].min(b[3])],
                    });
                }
            }
        }
        clip
    }

    /// Repair parents (missing / cycles), convert old groups, and put the list in
    /// depth-first order keeping the current sibling order.
    pub fn normalize(&mut self) {
        self.normalize_with(None);
    }

    fn normalize_with(&mut self, order: Option<(Option<u64>, &[u64])>) {
        self.repair_parents();
        // sibling lists (vec order, optionally overridden for one parent)
        let mut kids: HashMap<Option<u64>, Vec<usize>> = HashMap::new();
        for (i, e) in self.elements.iter().enumerate() {
            kids.entry(e.parent).or_default().push(i);
        }
        if let Some((parent, ord)) = order {
            if let Some(list) = kids.get_mut(&parent) {
                let mut sorted: Vec<usize> = ord.iter().filter_map(|id| list.iter().copied().find(|i| self.elements[*i].id == *id)).collect();
                for i in list.iter() {
                    if !sorted.contains(i) {
                        sorted.push(*i);
                    }
                }
                *list = sorted;
            }
        }
        let mut out_idx = Vec::with_capacity(self.elements.len());
        let mut visited = vec![false; self.elements.len()];
        let mut stack: Vec<usize> = kids.get(&None).map(|l| l.iter().rev().copied().collect()).unwrap_or_default();
        while let Some(i) = stack.pop() {
            if std::mem::replace(&mut visited[i], true) {
                continue;
            }
            out_idx.push(i);
            if let Some(l) = kids.get(&Some(self.elements[i].id)) {
                stack.extend(l.iter().rev().copied());
            }
        }
        let mut old: Vec<Option<Element>> = std::mem::take(&mut self.elements).into_iter().map(Some).collect();
        self.elements = out_idx.into_iter().filter_map(|i| old[i].take()).collect();
        let max = self.elements.iter().map(|e| e.id).max().unwrap_or(0);
        if self.next_id <= max {
            self.next_id = max.saturating_add(1);
        }
        self.prune_overrides();
    }

    /// Repairs the parent links, iteratively and in O(n): missing / self parents become roots, the members of a
    /// cycle become roots, and elements deeper than [`MAX_DEPTH`] are detached to the root (each element that
    /// would be at depth 129 becomes a root, and its subtree counts from there; elements are visited by
    /// increasing depth, ties in list order, so the result is deterministic).
    pub fn repair_parents(&mut self) {
        let n = self.elements.len();
        let idx = self.index_map();
        let mut par = self.parent_indices(&idx);
        let mut state = vec![0u8; n];
        let mut depth = vec![0usize; n];
        let mut stack: Vec<usize> = Vec::new();
        for s in 0..n {
            if state[s] != 0 {
                continue;
            }
            stack.clear();
            let mut cur = Some(s);
            while let Some(c) = cur {
                if state[c] == 2 {
                    break;
                }
                if state[c] == 1 {
                    // a cycle: every member becomes a root
                    if let Some(pos) = stack.iter().position(|&x| x == c) {
                        for &m in &stack[pos..] {
                            par[m] = None;
                        }
                    }
                    break;
                }
                state[c] = 1;
                stack.push(c);
                cur = par[c];
            }
            for &c in stack.iter().rev() {
                depth[c] = par[c].map(|p| depth[p] + 1).unwrap_or(0);
                state[c] = 2;
            }
        }
        if depth.iter().any(|&d| d > MAX_DEPTH) {
            let mut order: Vec<usize> = (0..n).collect();
            order.sort_by_key(|&i| depth[i]);
            let mut nd = vec![0usize; n];
            for i in order {
                nd[i] = par[i].map(|p| nd[p] + 1).unwrap_or(0);
                if nd[i] > MAX_DEPTH {
                    par[i] = None;
                    nd[i] = 0;
                }
            }
        }
        let ids: Vec<u64> = self.elements.iter().map(|e| e.id).collect();
        for (i, e) in self.elements.iter_mut().enumerate() {
            let want = par[i].map(|p| ids[p]);
            if e.parent != want {
                e.parent = want;
            }
        }
    }

    /// Reorder the children of `parent` (ids not listed keep their relative order after).
    pub fn set_sibling_order(&mut self, parent: Option<u64>, order: &[u64]) {
        self.normalize_with(Some((parent, order)));
    }

    /// Move `id` under `new_parent` at sibling position `index` (keeps its place on the page).
    pub fn reparent(&mut self, id: u64, new_parent: Option<u64>, index: Option<usize>) -> bool {
        if let Some(p) = new_parent {
            if p == id || self.is_descendant(p, id) || self.get(p).is_none() {
                return false;
            }
        }
        if let Some(e) = self.get_mut(id) {
            e.parent = new_parent;
        } else {
            return false;
        }
        let mut sibs: Vec<u64> = self.children(new_parent).into_iter().filter(|s| *s != id).collect();
        let at = index.unwrap_or(sibs.len()).min(sibs.len());
        sibs.insert(at, id);
        self.set_sibling_order(new_parent, &sibs);
        true
    }

    /// After the box of `id` (or the page for `None`) changed from `old`,
    /// move its children so they keep their RectTransform (anchors), recursively (iterative, any depth).
    pub fn reapply_children(&mut self, id: Option<u64>, old: Frame) {
        let map = self.child_map();
        let idx = self.index_map();
        let mut seen: HashSet<u64> = HashSet::new();
        let mut stack = vec![(id, old)];
        while let Some((id, old)) = stack.pop() {
            let new = match id {
                Some(i) => match idx.get(&i) {
                    Some(&k) => self.elements[k].frame(),
                    None => self.page_frame(),
                },
                None => self.page_frame(),
            };
            if new.close_to(&old) {
                continue;
            }
            let Some(kids) = map.get(&id) else { continue };
            for &c in kids {
                if !seen.insert(c) {
                    continue;
                }
                let Some(&k) = idx.get(&c) else { continue };
                let e = &self.elements[k];
                let rt = rect_transform_in(e, &old);
                let old_c = e.frame();
                apply_rect_transform(&mut self.elements[k], &rt, &new);
                stack.push((Some(c), old_c));
            }
        }
    }

    /// Change an element's box; its children follow their anchors.
    pub fn set_frame_keep_children(&mut self, id: u64, f: &Frame) {
        let Some(old) = self.get(id).map(|e| e.frame()) else { return };
        if let Some(e) = self.get_mut(id) {
            e.set_frame(f);
        }
        self.reapply_children(Some(id), old);
    }

    pub fn translate_subtree(&mut self, id: u64, dx: f32, dy: f32) {
        for i in self.subtree(id) {
            if let Some(e) = self.get_mut(i) {
                e.x += dx;
                e.y += dy;
            }
        }
    }

    /// Remove elements and everything inside them.
    pub fn remove_subtrees(&mut self, ids: &[u64]) {
        let mut gone: HashSet<u64> = HashSet::new();
        for id in ids {
            gone.extend(self.subtree(*id));
        }
        self.elements.retain(|e| !gone.contains(&e.id));
        self.prune_overrides();
    }

    /// Clone elements (with their internal parent links) into the document under
    /// `parent`, with fresh ids. Elements whose parent is not in `items` become
    /// children of `parent`. Returns the new ids of those top elements.
    pub fn insert_items(&mut self, items: &[Element], parent: Option<u64>) -> Vec<u64> {
        self.insert_items_map(items, parent).0
    }

    /// Like [`Doc::insert_items`], also returning the map old id -> new id of every inserted element
    /// (to carry per-element data such as portrait overrides along).
    pub fn insert_items_map(&mut self, items: &[Element], parent: Option<u64>) -> (Vec<u64>, HashMap<u64, u64>) {
        let inside: HashSet<u64> = items.iter().map(|e| e.id).collect();
        let mut map: HashMap<u64, u64> = HashMap::new();
        for e in items {
            let nid = self.take_id();
            map.insert(e.id, nid);
        }
        let mut roots = Vec::new();
        for e in items {
            let mut n = e.clone();
            n.id = map[&e.id];
            n.group = None;
            n.parent = match e.parent {
                Some(p) if inside.contains(&p) => Some(map[&p]),
                _ => {
                    roots.push(n.id);
                    parent
                }
            };
            self.elements.push(n);
        }
        self.normalize();
        (roots, map)
    }

    /// Elements of `ids` plus all their descendants, in document order (for copy / library).
    pub fn collect_subtrees(&self, ids: &[u64]) -> Vec<Element> {
        let top = self.top_level(ids);
        let mut set: HashSet<u64> = HashSet::new();
        for t in &top {
            set.extend(self.subtree(*t));
        }
        let mut v: Vec<Element> = self.elements.iter().filter(|e| set.contains(&e.id)).cloned().collect();
        for e in v.iter_mut() {
            if top.contains(&e.id) {
                e.parent = None;
            }
        }
        v
    }

    /// Fixes what a hand-edited or foreign file may bring broken: duplicate ids, `next_id` below
    /// some id, cyclic parents (or self-parents) and non-numeric coordinates.
    pub fn sanitize(&mut self) {
        self.elements.truncate(MAX_ELEMENTS);
        // ids: 1..=MAX_ID and unique. Elements with an id out of range (0, u64::MAX…) or repeating an earlier one are
        // renumbered; parent links and portrait overrides that named an out-of-range id follow its new number.
        let mut max_id = 0u64;
        for e in &self.elements {
            if e.id <= MAX_ID {
                max_id = max_id.max(e.id);
            }
        }
        if self.next_id > MAX_ID * 2 || self.next_id == 0 {
            self.next_id = max_id + 1;
        }
        self.next_id = self.next_id.max(max_id + 1);
        let mut seen = HashSet::new();
        let mut remap: HashMap<u64, u64> = HashMap::new();
        for i in 0..self.elements.len() {
            let id = self.elements[i].id;
            if id == 0 || id > MAX_ID || !seen.insert(id) {
                let new = self.take_id();
                if id > MAX_ID {
                    remap.entry(id).or_insert(new);
                }
                self.elements[i].id = new;
                seen.insert(new);
            }
        }
        if !remap.is_empty() {
            for e in self.elements.iter_mut() {
                if let Some(n) = e.parent.and_then(|p| remap.get(&p)) {
                    e.parent = Some(*n);
                }
            }
            if let Some(p) = self.portrait.as_mut() {
                for o in p.overrides.iter_mut() {
                    if let Some(n) = remap.get(&o.id) {
                        o.id = *n;
                    }
                }
            }
        }
        // U+2028/U+2029/U+0085 and lone \r are line breaks for C#: they would end a `//`
        // comment or a string literal in the exported scripts.
        let bad = |c: char, jumps: bool| (c.is_control() && !(jumps && (c == '\n' || c == '\t'))) || c == '\u{2028}' || c == '\u{2029}';
        let clear = |t: &mut String, jumps: bool| {
            if t.chars().any(|c| bad(c, jumps)) {
                *t = t.chars().filter(|c| !bad(*c, jumps)).collect();
            }
        };
        for e in self.elements.iter_mut() {
            for v in [&mut e.x, &mut e.y, &mut e.w, &mut e.h, &mut e.rot, &mut e.value, &mut e.stroke_w, &mut e.radius] {
                if !v.is_finite() {
                    *v = 0.0;
                }
            }
            e.x = e.x.clamp(-COORD_LIMIT, COORD_LIMIT);
            e.y = e.y.clamp(-COORD_LIMIT, COORD_LIMIT);
            e.w = e.w.clamp(0.0, COORD_LIMIT);
            e.h = e.h.clamp(0.0, COORD_LIMIT);
            e.value = e.value.clamp(-COORD_LIMIT, COORD_LIMIT);
            e.rot %= 360.0;
            e.stroke_w = e.stroke_w.clamp(0.0, 1000.0);
            e.radius = e.radius.clamp(0.0, 10_000.0);
            e.opacity = if e.opacity.is_finite() { e.opacity.clamp(0.0, 1.0) } else { 1.0 };
            // A 1e9 px letter would make egui try to draw a giant glyph.
            e.font_size = if e.font_size.is_finite() { e.font_size.clamp(1.0, 1000.0) } else { 32.0 };
            // Control characters (NUL…) break the UXML and the exported scripts.
            clear(&mut e.name, false);
            clear(&mut e.text, true);
            clear(&mut e.on_click, true);
            clear(&mut e.font, false);
            clear(&mut e.image, false);
            for o in e.options.iter_mut() {
                clear(o, false);
            }
        }
        clear(&mut self.title, false);
        for e in self.elements.iter_mut() {
            for v in e.anchor_min.iter_mut().chain(e.anchor_max.iter_mut()).chain(e.pivot.iter_mut()) {
                *v = if v.is_finite() { v.clamp(-ANCHOR_LIMIT, ANCHOR_LIMIT) } else { 0.5 };
            }
            for v in e.shadow.offset.iter_mut() {
                *v = if v.is_finite() { v.clamp(-ANCHOR_LIMIT, ANCHOR_LIMIT) } else { 0.5 };
            }
            let l = &mut e.layout;
            for v in l.padding.iter_mut().chain(l.spacing.iter_mut()) {
                *v = if v.is_finite() { v.clamp(-10_000.0, 10_000.0) } else { 0.0 };
            }
            for v in l.cell.iter_mut() {
                *v = if v.is_finite() { v.clamp(1.0, 10_000.0) } else { 100.0 };
            }
        }
        self.next_id = self.next_id.clamp(1, MAX_ID * 2);
        self.canvas_match = if self.canvas_match.is_finite() { self.canvas_match.clamp(0.0, 1.0) } else { 0.5 };
        self.sanitize_ext();
        // Page: with no size, or a huge one, the canvas ends up NaN or unscaled.
        if !self.width.is_finite() || self.width < 1.0 {
            self.width = 1920.0;
        }
        if !self.height.is_finite() || self.height < 1.0 {
            self.height = 1080.0;
        }
        self.width = self.width.min(16_384.0);
        self.height = self.height.min(16_384.0);
        self.repair_parents();
    }

    /// Convert old flat groups into Empty parent elements.
    pub fn migrate_groups(&mut self) {
        self.sanitize();
        let mut groups: Vec<u64> = Vec::new();
        for e in &self.elements {
            if let Some(g) = e.group {
                if e.parent.is_none() && !groups.contains(&g) {
                    groups.push(g);
                }
            }
        }
        for (n, g) in groups.into_iter().enumerate() {
            let members: Vec<u64> = self.elements.iter().filter(|e| e.group == Some(g) && e.parent.is_none()).map(|e| e.id).collect();
            if members.len() < 2 {
                continue;
            }
            let b = bounds_of(self.elements.iter().filter(|e| members.contains(&e.id)));
            let mut holder = Element::new(0, Kind::Empty);
            holder.name = format!("Group {}", n + 1);
            holder.x = b[0];
            holder.y = b[1];
            holder.w = (b[2] - b[0]).max(1.0);
            holder.h = (b[3] - b[1]).max(1.0);
            let first = self.index_of(members[0]).unwrap_or(0);
            holder.id = self.take_id();
            let hid = holder.id;
            self.elements.insert(first, holder);
            for e in self.elements.iter_mut().filter(|e| members.contains(&e.id)) {
                e.parent = Some(hid);
            }
        }
        for e in self.elements.iter_mut() {
            e.group = None;
        }
        self.normalize();
    }

    /// Lay out the children of every element that has a Layout Group (like Unity does).
    pub fn apply_layouts(&mut self) {
        let order: Vec<u64> = self.elements.iter().filter(|e| e.layout.mode != LayoutMode::None).map(|e| e.id).collect();
        for id in order {
            let Some(p) = self.get(id) else { continue };
            let pf = p.frame();
            let lg = p.layout.clone();
            let kids: Vec<u64> = self.children(Some(id)).into_iter().filter(|c| self.get(*c).map(|e| e.visible).unwrap_or(false)).collect();
            if kids.is_empty() {
                continue;
            }
            let [pl, pr, pt, pb] = lg.padding;
            let iw = (pf.w - pl - pr).max(0.0);
            let ih = (pf.h - pt - pb).max(0.0);
            let col = (lg.align % 3) as f32 * 0.5;
            let row = (lg.align / 3).min(2) as f32 * 0.5;
            let sizes: Vec<[f32; 2]> = kids.iter().map(|c| self.get(*c).map(|e| [e.w, e.h]).unwrap_or([0.0, 0.0])).collect();
            let n = kids.len();
            let nf = n as f32;
            let mut rects: Vec<[f32; 4]> = Vec::with_capacity(n);
            match lg.mode {
                LayoutMode::Horizontal => {
                    let ws: Vec<f32> = if lg.expand_w { vec![((iw - lg.spacing[0] * (nf - 1.0)) / nf).max(0.0); n] } else { sizes.iter().map(|s| s[0]).collect() };
                    let hs: Vec<f32> = if lg.expand_h { vec![ih; n] } else { sizes.iter().map(|s| s[1]).collect() };
                    let total: f32 = ws.iter().sum::<f32>() + lg.spacing[0] * (nf - 1.0);
                    let mut x = pl + (iw - total) * col;
                    for i in 0..n {
                        rects.push([x, pt + (ih - hs[i]) * row, ws[i], hs[i]]);
                        x += ws[i] + lg.spacing[0];
                    }
                }
                LayoutMode::Vertical => {
                    let hs: Vec<f32> = if lg.expand_h { vec![((ih - lg.spacing[1] * (nf - 1.0)) / nf).max(0.0); n] } else { sizes.iter().map(|s| s[1]).collect() };
                    let ws: Vec<f32> = if lg.expand_w { vec![iw; n] } else { sizes.iter().map(|s| s[0]).collect() };
                    let total: f32 = hs.iter().sum::<f32>() + lg.spacing[1] * (nf - 1.0);
                    let mut y = pt + (ih - total) * row;
                    for i in 0..n {
                        rects.push([pl + (iw - ws[i]) * col, y, ws[i], hs[i]]);
                        y += hs[i] + lg.spacing[1];
                    }
                }
                LayoutMode::Grid => {
                    let [cw, ch] = [lg.cell[0].max(1.0), lg.cell[1].max(1.0)];
                    let cols = (((iw + lg.spacing[0]) / (cw + lg.spacing[0])).floor() as usize).max(1);
                    let rows = n.div_ceil(cols);
                    let used = cols.min(n) as f32;
                    let bw = used * cw + (used - 1.0) * lg.spacing[0];
                    let bh = rows as f32 * ch + (rows as f32 - 1.0) * lg.spacing[1];
                    let ox = pl + (iw - bw) * col;
                    let oy = pt + (ih - bh) * row;
                    for i in 0..n {
                        let (c, r) = ((i % cols) as f32, (i / cols) as f32);
                        rects.push([ox + c * (cw + lg.spacing[0]), oy + r * (ch + lg.spacing[1]), cw, ch]);
                    }
                }
                LayoutMode::None => {}
            }
            for (c, r) in kids.into_iter().zip(rects) {
                let lc = [r[0] + r[2] / 2.0 - pf.w / 2.0, r[1] + r[3] / 2.0 - pf.h / 2.0];
                let wc = pf.to_world(lc);
                let Some(old) = self.get(c).map(|e| e.frame()) else { continue };
                let f = Frame { cx: wc[0], cy: wc[1], w: r[2], h: r[3], rot: old.rot };
                if !old.close_to(&f) {
                    self.set_frame_keep_children(c, &f);
                }
            }
        }
    }

    /// Is `id` positioned by its parent's Layout Group?
    pub fn driven_by_layout(&self, id: u64) -> bool {
        self.parent_of(id).and_then(|p| self.get(p)).map(|p| p.layout.mode != LayoutMode::None).unwrap_or(false)
    }

    /// Copy of the document as it would look on a screen of `screen` pixels with a
    /// CanvasScaler in "Scale With Screen Size" mode. Returns the doc (in canvas
    /// units) and the scale factor canvas → screen.
    pub fn simulate_screen(&self, screen: [f32; 2]) -> (Doc, f32) {
        let s = canvas_scale(screen, [self.width, self.height], self.canvas_match);
        let mut d = self.clone();
        let old = d.page_frame();
        d.width = screen[0] / s;
        d.height = screen[1] / s;
        d.reapply_children(None, old);
        d.apply_layouts();
        (d, s)
    }
}

/// CanvasScaler.ScaleWithScreenSize + MatchWidthOrHeight scale factor.
pub fn canvas_scale(screen: [f32; 2], reference: [f32; 2], m: f32) -> f32 {
    let lw = (screen[0] / reference[0].max(1.0)).max(1e-6).log2();
    let lh = (screen[1] / reference[1].max(1.0)).max(1e-6).log2();
    2f32.powf(lw + (lh - lw) * m.clamp(0.0, 1.0))
}

pub fn bounds_of<'a>(items: impl Iterator<Item = &'a Element>) -> [f32; 4] {
    let mut b = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    for e in items {
        let a = e.aabb();
        b[0] = b[0].min(a[0]);
        b[1] = b[1].min(a[1]);
        b[2] = b[2].max(a[2]);
        b[3] = b[3].max(a[3]);
    }
    b
}

pub fn hex(c: Rgba) -> String {
    format!("#{:02X}{:02X}{:02X}{:02X}", c[0], c[1], c[2], c[3])
}

// ---------------------------------------------------------------------------
// Templates (starting layouts for game UIs)
// ---------------------------------------------------------------------------

/// The numbers are stable (the command line and the MCP use them): new ones are added at the end.
pub const TEMPLATES: &[&str] = &[
    "Main menu",
    "Game HUD",
    "Dialog window",
    "Settings",
    "Inventory",
    "Blank",
    "Pause menu",
    "Game over",
    "Level select",
    "Loading screen",
    "Character dialogue",
    "Leaderboard",
    "Character select",
];

pub fn blank_template() -> usize {
    5
}

/// Order in which they are shown: "En blanco" last.
pub fn template_order() -> impl Iterator<Item = usize> {
    (0..TEMPLATES.len()).filter(|&i| i != blank_template()).chain(std::iter::once(blank_template()))
}

fn put(doc: &mut Doc, parent: Option<u64>, kind: Kind, name: &str, rect: [f32; 4], f: impl FnOnce(&mut Element)) -> u64 {
    let mut e = Element::new(0, kind);
    e.name = name.to_string();
    e.x = rect[0];
    e.y = rect[1];
    e.w = rect[2];
    e.h = rect[3];
    e.parent = parent;
    f(&mut e);
    doc.add(e)
}

fn anchor(e: &mut Element, min: [f32; 2], max: [f32; 2]) {
    e.anchor_min = min;
    e.anchor_max = max;
}

/// Templates are designed for 1920×1080: on smaller pages they are built at a
/// scale that fits (same page proportion) and then shrunk uniformly.
pub fn template(index: usize, width: f32, height: f32) -> Doc {
    let (width, height) = (width.max(1.0), height.max(1.0));
    let k = (1920.0 / width).max(1080.0 / height).max(1.0);
    let mut d = template_at(index, width * k, height * k);
    if k > 1.0 {
        let f = 1.0 / k;
        d.width = width;
        d.height = height;
        for e in &mut d.elements {
            e.x *= f;
            e.y *= f;
            e.w *= f;
            e.h *= f;
            e.radius *= f;
            e.stroke_w *= f;
            e.font_size = (e.font_size * f).max(8.0);
            e.shadow.offset = [e.shadow.offset[0] * f, e.shadow.offset[1] * f];
            let l = &mut e.layout;
            l.padding = l.padding.map(|v| v * f);
            l.spacing = l.spacing.map(|v| v * f);
            l.cell = l.cell.map(|v| v * f);
        }
        d.apply_layouts();
    }
    d
}

fn template_at(index: usize, width: f32, height: f32) -> Doc {
    let mut d = Doc::empty(width, height);
    let (w, h) = (width, height);
    match index {
        0 => {
            d.title = "Main menu".into();
            d.background = [24, 22, 48, 255];
            put(&mut d, None, Kind::Image, "Background", [0.0, 0.0, w, h], |e| {
                e.fill = [46, 32, 110, 255];
                e.fill2 = [12, 120, 150, 255];
                e.radius = 0.0;
                e.raycast = false;
                anchor(e, [0.0, 0.0], [1.0, 1.0]);
            });
            put(&mut d, None, Kind::Ellipse, "Moon", [w * 0.72, h * 0.12, 260.0, 260.0], |e| {
                e.fill = [255, 214, 120, 200];
                e.raycast = false;
                anchor(e, [1.0, 1.0], [1.0, 1.0]);
            });
            let head = put(&mut d, None, Kind::Empty, "Header", [w / 2.0 - 500.0, h * 0.12, 1000.0, 190.0], |e| anchor(e, [0.5, 1.0], [0.5, 1.0]));
            put(&mut d, Some(head), Kind::Text, "Title", [w / 2.0 - 500.0, h * 0.12, 1000.0, 120.0], |e| {
                e.text = "STAR ADVENTURE".into();
                e.font_size = 96.0;
                e.bold = true;
                e.text_color = [255, 255, 255, 255];
                anchor(e, [0.0, 1.0], [1.0, 1.0]);
            });
            put(&mut d, Some(head), Kind::Text, "Subtitle", [w / 2.0 - 400.0, h * 0.12 + 135.0, 800.0, 50.0], |e| {
                e.text = "An interface demo".into();
                e.font_size = 36.0;
                e.text_color = [190, 230, 255, 255];
                anchor(e, [0.5, 0.0], [0.5, 0.0]);
            });
            let menu = put(&mut d, None, Kind::Empty, "Menu", [w / 2.0 - 220.0, h * 0.42, 440.0, 470.0], |e| {
                e.layout.mode = LayoutMode::Vertical;
                e.layout.spacing = [0.0, 22.0];
                e.layout.padding = [20.0, 20.0, 10.0, 10.0];
                e.layout.align = 1;
                e.layout.expand_w = true;
            });
            let labels = ["Play", "Options", "Credits", "Exit"];
            for (i, l) in labels.iter().enumerate() {
                put(&mut d, Some(menu), Kind::Button, &format!("{} button", l), [0.0, 0.0, 400.0, 88.0], |e| {
                    e.text = l.to_string();
                    e.fill = if i == 0 { [225, 95, 30, 255] } else { [124, 92, 255, 255] };
                    e.radius = 44.0;
                    if i == 0 {
                        e.shadow.on = true;
                    }
                    e.on_click = match i {
                        0 => "set_text(\"Subtitle\", \"Loading level 1...!\");\nset_color(me, \"#46BE78\");\nmessage(\"Click on \" + me);".into(),
                        3 => "toggle(\"Moon\");\nmessage(\"The moon changed visibility\");".into(),
                        _ => format!("message(\"Open menu: {}\");", l),
                    };
                });
            }
            put(&mut d, None, Kind::Text, "Version", [w - 260.0, h - 70.0, 220.0, 40.0], |e| {
                e.text = "v1.0".into();
                e.font_size = 24.0;
                e.align = 2;
                e.text_color = [200, 200, 230, 200];
                e.pivot = [1.0, 0.0];
                anchor(e, [1.0, 0.0], [1.0, 0.0]);
            });
        }
        1 => {
            d.title = "Game HUD".into();
            d.background = [70, 120, 90, 255];
            put(&mut d, None, Kind::Image, "Scene (preview)", [0.0, 0.0, w, h], |e| {
                e.fill = [120, 190, 240, 255];
                e.fill2 = [70, 140, 80, 255];
                e.radius = 0.0;
                e.raycast = false;
                anchor(e, [0.0, 0.0], [1.0, 1.0]);
            });
            let top = put(&mut d, None, Kind::Empty, "Top bar", [0.0, 0.0, w, 170.0], |e| {
                anchor(e, [0.0, 1.0], [1.0, 1.0]);
                e.pivot = [0.5, 1.0];
            });
            let frame = put(&mut d, Some(top), Kind::Panel, "Health frame", [40.0, 40.0, 520.0, 110.0], |e| {
                e.fill = [10, 10, 20, 160];
                anchor(e, [0.0, 1.0], [0.0, 1.0]);
                e.pivot = [0.0, 1.0];
            });
            put(&mut d, Some(frame), Kind::Text, "Health label", [70.0, 50.0, 300.0, 40.0], |e| {
                e.text = "HEALTH 72/100".into();
                e.font_size = 28.0;
                e.bold = true;
                e.align = 0;
                e.text_color = [255, 255, 255, 255];
                anchor(e, [0.0, 1.0], [0.0, 1.0]);
                e.pivot = [0.0, 1.0];
            });
            put(&mut d, Some(frame), Kind::Slider, "Health bar", [70.0, 95.0, 460.0, 34.0], |e| {
                e.fill = [235, 70, 90, 255];
                e.fill2 = [60, 60, 70, 255];
                e.value = 0.72;
                e.interactable = false;
                anchor(e, [0.0, 0.0], [1.0, 0.0]);
                e.pivot = [0.5, 0.0];
            });
            put(&mut d, Some(top), Kind::Text, "Points", [w - 440.0, 50.0, 400.0, 60.0], |e| {
                e.text = "12 450".into();
                e.font_size = 52.0;
                e.bold = true;
                e.align = 2;
                e.text_color = [255, 240, 150, 255];
                anchor(e, [1.0, 1.0], [1.0, 1.0]);
                e.pivot = [1.0, 1.0];
            });
            let bar = put(&mut d, None, Kind::Panel, "Action bar", [w / 2.0 - 290.0, h - 170.0, 580.0, 140.0], |e| {
                e.fill = [10, 10, 20, 120];
                e.radius = 20.0;
                anchor(e, [0.5, 0.0], [0.5, 0.0]);
                e.pivot = [0.5, 0.0];
                e.layout.mode = LayoutMode::Horizontal;
                e.layout.spacing = [16.0, 0.0];
                e.layout.align = 4;
            });
            for i in 0..4 {
                put(&mut d, Some(bar), Kind::Panel, &format!("Slot {}", i + 1), [0.0, 0.0, 112.0, 112.0], |e| {
                    e.fill = [10, 10, 20, 150];
                    e.stroke = [255, 255, 255, 200];
                    e.stroke_w = 3.0;
                    e.radius = 16.0;
                    e.text = format!("{}", i + 1);
                    e.font_size = 22.0;
                    e.text_color = [255, 255, 255, 180];
                    e.align = 2;
                    e.valign = 2;
                });
            }
            put(&mut d, None, Kind::Ellipse, "Minimap", [w - 300.0, h - 300.0, 260.0, 260.0], |e| {
                e.fill = [20, 40, 30, 200];
                e.stroke = [255, 255, 255, 220];
                e.stroke_w = 4.0;
                e.mask = true;
                anchor(e, [1.0, 0.0], [1.0, 0.0]);
            });
        }
        2 => {
            d.title = "Dialog window".into();
            d.background = [40, 44, 60, 255];
            put(&mut d, None, Kind::Panel, "Dim", [0.0, 0.0, w, h], |e| {
                e.fill = [0, 0, 0, 120];
                e.radius = 0.0;
                anchor(e, [0.0, 0.0], [1.0, 1.0]);
            });
            let win = put(&mut d, None, Kind::Panel, "Window", [w / 2.0 - 450.0, h / 2.0 - 260.0, 900.0, 520.0], |e| {
                e.fill = [255, 255, 255, 255];
                e.radius = 32.0;
                e.shadow.on = true;
                e.shadow.offset = [0.0, -10.0];
            });
            put(&mut d, Some(win), Kind::Text, "Window title", [w / 2.0 - 400.0, h / 2.0 - 215.0, 800.0, 70.0], |e| {
                e.text = "Save the game?".into();
                e.font_size = 52.0;
                e.bold = true;
                anchor(e, [0.0, 1.0], [1.0, 1.0]);
                e.pivot = [0.5, 1.0];
            });
            put(&mut d, Some(win), Kind::Text, "Message", [w / 2.0 - 380.0, h / 2.0 - 120.0, 760.0, 100.0], |e| {
                e.text = "Your progress will be saved in slot 1. You can continue later.".into();
                e.font_size = 30.0;
                e.text_color = [90, 90, 105, 255];
                anchor(e, [0.0, 0.5], [1.0, 0.5]);
            });
            let row = put(&mut d, Some(win), Kind::Empty, "Buttons", [w / 2.0 - 410.0, h / 2.0 + 100.0, 820.0, 120.0], |e| {
                anchor(e, [0.0, 0.0], [1.0, 0.0]);
                e.pivot = [0.5, 0.0];
                e.layout.mode = LayoutMode::Horizontal;
                e.layout.spacing = [30.0, 0.0];
                e.layout.padding = [10.0, 10.0, 15.0, 15.0];
                e.layout.expand_w = true;
                e.layout.expand_h = true;
            });
            put(&mut d, Some(row), Kind::Button, "Cancel button", [0.0, 0.0, 360.0, 90.0], |e| {
                e.text = "Cancel".into();
                e.fill = [235, 235, 242, 255];
                e.text_color = [40, 40, 50, 255];
                e.on_click = "hide(\"Window\");\nhide(\"Dim\");".into();
            });
            put(&mut d, Some(row), Kind::Button, "Accept button", [0.0, 0.0, 360.0, 90.0], |e| {
                e.text = "Save".into();
                e.on_click = "set_text(\"Message\", \"Game saved!\");".into();
            });
        }
        3 => {
            d.title = "Settings".into();
            d.background = [236, 238, 245, 255];
            put(&mut d, None, Kind::Text, "Title", [120.0, 70.0, 800.0, 90.0], |e| {
                e.text = "Settings".into();
                e.font_size = 72.0;
                e.bold = true;
                e.align = 0;
                anchor(e, [0.0, 1.0], [0.0, 1.0]);
                e.pivot = [0.0, 1.0];
            });
            let card = put(&mut d, None, Kind::Panel, "Card", [120.0, 190.0, w - 240.0, h - 400.0], |e| {
                e.fill = [255, 255, 255, 255];
                e.radius = 24.0;
                e.shadow.on = true;
                e.shadow.color = [0, 0, 0, 40];
                anchor(e, [0.0, 0.0], [1.0, 1.0]);
            });
            let list = put(&mut d, Some(card), Kind::ScrollView, "Options", [150.0, 220.0, w - 300.0, h - 460.0], |e| {
                e.fill = [255, 255, 255, 0];
                e.stroke_w = 0.0;
                anchor(e, [0.0, 0.0], [1.0, 1.0]);
            });
            let rows = ["Music volume", "Effects volume", "Brightness"];
            for (i, r) in rows.iter().enumerate() {
                let y = 250.0 + i as f32 * 110.0;
                put(&mut d, Some(list), Kind::Text, r, [180.0, y, 500.0, 50.0], |e| {
                    e.text = r.to_string();
                    e.font_size = 34.0;
                    e.align = 0;
                    anchor(e, [0.0, 1.0], [0.0, 1.0]);
                });
                put(&mut d, Some(list), Kind::Slider, &format!("Slider {}", r), [720.0, y + 5.0, 600.0, 40.0], |e| {
                    e.value = 0.3 + 0.2 * i as f32;
                    anchor(e, [0.0, 1.0], [1.0, 1.0]);
                });
            }
            put(&mut d, Some(list), Kind::Toggle, "Fullscreen", [180.0, 590.0, 500.0, 50.0], |e| {
                e.text = "Fullscreen".into();
                e.font_size = 34.0;
                anchor(e, [0.0, 1.0], [0.0, 1.0]);
            });
            put(&mut d, Some(list), Kind::Dropdown, "Quality", [720.0, 580.0, 420.0, 70.0], |e| {
                e.options = vec!["Low".into(), "Medium".into(), "High".into(), "Ultra".into()];
                e.value = 2.0;
                anchor(e, [0.0, 1.0], [0.0, 1.0]);
            });
            put(&mut d, Some(list), Kind::InputField, "Player name", [180.0, 690.0, 600.0, 76.0], |e| {
                e.text = "Player name".into();
                anchor(e, [0.0, 1.0], [0.0, 1.0]);
            });
            put(&mut d, None, Kind::Button, "Back button", [120.0, h - 170.0, 300.0, 84.0], |e| {
                e.text = "Back".into();
                anchor(e, [0.0, 0.0], [0.0, 0.0]);
                e.on_click = "message(\"Back to the menu\");".into();
            });
        }
        4 => {
            d.title = "Inventory".into();
            d.background = [28, 30, 40, 255];
            let win = put(&mut d, None, Kind::Panel, "Inventory", [w / 2.0 - 420.0, h / 2.0 - 380.0, 840.0, 760.0], |e| {
                e.fill = [44, 47, 62, 255];
                e.radius = 28.0;
                e.stroke = [90, 96, 130, 255];
                e.stroke_w = 3.0;
                e.shadow.on = true;
            });
            put(&mut d, Some(win), Kind::Text, "Title", [w / 2.0 - 380.0, h / 2.0 - 350.0, 600.0, 70.0], |e| {
                e.text = "Inventory".into();
                e.font_size = 52.0;
                e.bold = true;
                e.align = 0;
                e.text_color = [255, 255, 255, 255];
                anchor(e, [0.0, 1.0], [0.0, 1.0]);
                e.pivot = [0.0, 1.0];
            });
            put(&mut d, Some(win), Kind::Button, "Close", [w / 2.0 + 320.0, h / 2.0 - 350.0, 70.0, 70.0], |e| {
                e.text = "X".into();
                e.font_size = 32.0;
                e.radius = 35.0;
                e.fill = [235, 70, 90, 255];
                anchor(e, [1.0, 1.0], [1.0, 1.0]);
                e.pivot = [1.0, 1.0];
                e.on_click = "hide(\"Inventory\");".into();
            });
            let grid = put(&mut d, Some(win), Kind::Panel, "Grid", [w / 2.0 - 380.0, h / 2.0 - 250.0, 760.0, 600.0], |e| {
                e.fill = [30, 32, 44, 255];
                e.radius = 18.0;
                anchor(e, [0.0, 0.0], [1.0, 1.0]);
                e.layout.mode = LayoutMode::Grid;
                e.layout.cell = [160.0, 160.0];
                e.layout.spacing = [24.0, 24.0];
                e.layout.padding = [24.0, 24.0, 24.0, 24.0];
                e.layout.align = 1;
            });
            for i in 0..12 {
                put(&mut d, Some(grid), Kind::Panel, &format!("Slot {}", i + 1), [0.0, 0.0, 160.0, 160.0], |e| {
                    e.fill = [60, 64, 84, 255];
                    e.radius = 14.0;
                    e.stroke = [110, 116, 150, 255];
                    e.stroke_w = 2.0;
                });
            }
        }
        6 => {
            d.title = "Pause menu".into();
            d.background = [70, 120, 90, 255];
            put(&mut d, None, Kind::Image, "Game (preview)", [0.0, 0.0, w, h], |e| {
                e.fill = [120, 190, 240, 255];
                e.fill2 = [70, 140, 80, 255];
                e.radius = 0.0;
                e.raycast = false;
                anchor(e, [0.0, 0.0], [1.0, 1.0]);
            });
            put(&mut d, None, Kind::Panel, "Dim", [0.0, 0.0, w, h], |e| {
                e.fill = [8, 8, 16, 170];
                e.radius = 0.0;
                anchor(e, [0.0, 0.0], [1.0, 1.0]);
            });
            let win = put(&mut d, None, Kind::Panel, "Pause", [w / 2.0 - 280.0, h / 2.0 - 330.0, 560.0, 660.0], |e| {
                e.fill = [30, 32, 48, 240];
                e.stroke = [124, 92, 255, 255];
                e.stroke_w = 3.0;
                e.radius = 30.0;
                e.shadow.on = true;
                e.layout.mode = LayoutMode::Vertical;
                e.layout.spacing = [0.0, 20.0];
                e.layout.padding = [50.0, 50.0, 50.0, 50.0];
                e.layout.align = 1;
                e.layout.expand_w = true;
            });
            put(&mut d, Some(win), Kind::Text, "Title", [0.0, 0.0, 460.0, 100.0], |e| {
                e.text = "PAUSED".into();
                e.font_size = 72.0;
                e.bold = true;
                e.text_color = [255, 255, 255, 255];
            });
            let opts = ["Continue", "Restart level", "Options", "Exit to menu"];
            for (i, l) in opts.iter().enumerate() {
                put(&mut d, Some(win), Kind::Button, &format!("{} button", l), [0.0, 0.0, 460.0, 84.0], |e| {
                    e.text = l.to_string();
                    e.radius = 42.0;
                    e.fill = if i == 0 { [35, 140, 85, 255] } else if i == 3 { [235, 70, 90, 255] } else { [124, 92, 255, 255] };
                    e.on_click = match i {
                        0 => "hide(\"Pause\");\nhide(\"Dim\");".into(),
                        _ => format!("message(\"{}\");", l),
                    };
                });
            }
        }
        7 => {
            d.title = "Game over".into();
            d.background = [20, 10, 18, 255];
            put(&mut d, None, Kind::Image, "Background", [0.0, 0.0, w, h], |e| {
                e.fill = [90, 20, 40, 255];
                e.fill2 = [15, 8, 20, 255];
                e.radius = 0.0;
                e.raycast = false;
                anchor(e, [0.0, 0.0], [1.0, 1.0]);
            });
            put(&mut d, None, Kind::Text, "Title", [w / 2.0 - 600.0, h * 0.14, 1200.0, 150.0], |e| {
                e.text = "GAME OVER".into();
                e.font_size = 120.0;
                e.bold = true;
                e.text_color = [255, 90, 110, 255];
                e.shadow.on = true;
                anchor(e, [0.5, 1.0], [0.5, 1.0]);
            });
            let card = put(&mut d, None, Kind::Panel, "Results", [w / 2.0 - 360.0, h * 0.36, 720.0, 330.0], |e| {
                e.fill = [255, 255, 255, 24];
                e.stroke = [255, 255, 255, 60];
                e.stroke_w = 2.0;
                e.radius = 24.0;
                e.layout.mode = LayoutMode::Vertical;
                e.layout.spacing = [0.0, 12.0];
                e.layout.padding = [40.0, 40.0, 36.0, 36.0];
                e.layout.align = 1;
                e.layout.expand_w = true;
            });
            for (n, v, c) in [("Score", "12 450", [255, 240, 150, 255]), ("Best score", "18 900", [190, 230, 255, 255]), ("Time", "04:32", [255, 255, 255, 255])] {
                put(&mut d, Some(card), Kind::Text, n, [0.0, 0.0, 640.0, 70.0], |e| {
                    e.text = format!("{}: {}", n, v);
                    e.font_size = 44.0;
                    e.bold = n == "Score";
                    e.text_color = c;
                });
            }
            let row = put(&mut d, None, Kind::Empty, "Buttons", [w / 2.0 - 420.0, h * 0.72, 840.0, 110.0], |e| {
                anchor(e, [0.5, 0.0], [0.5, 0.0]);
                e.layout.mode = LayoutMode::Horizontal;
                e.layout.spacing = [30.0, 0.0];
                e.layout.align = 4;
                e.layout.expand_w = true;
                e.layout.expand_h = true;
            });
            put(&mut d, Some(row), Kind::Button, "Retry button", [0.0, 0.0, 400.0, 100.0], |e| {
                e.text = "Retry".into();
                e.fill = [225, 95, 30, 255];
                e.radius = 50.0;
                e.shadow.on = true;
                e.on_click = "set_text(\"Title\", \"ONE MORE TIME!\");".into();
            });
            put(&mut d, Some(row), Kind::Button, "Menu button", [0.0, 0.0, 400.0, 100.0], |e| {
                e.text = "Main menu".into();
                e.fill = [60, 60, 80, 255];
                e.radius = 50.0;
                e.on_click = "message(\"Back to the menu\");".into();
            });
        }
        8 => {
            d.title = "Level select".into();
            d.background = [18, 24, 44, 255];
            put(&mut d, None, Kind::Text, "Title", [80.0, 50.0, 900.0, 100.0], |e| {
                e.text = "Choose a level".into();
                e.font_size = 72.0;
                e.bold = true;
                e.align = 0;
                e.text_color = [255, 255, 255, 255];
                anchor(e, [0.0, 1.0], [0.0, 1.0]);
                e.pivot = [0.0, 1.0];
            });
            put(&mut d, None, Kind::Text, "Stars", [w - 480.0, 70.0, 400.0, 60.0], |e| {
                e.text = "Stars: 14 / 36".into();
                e.font_size = 44.0;
                e.bold = true;
                e.align = 2;
                e.text_color = [255, 214, 90, 255];
                anchor(e, [1.0, 1.0], [1.0, 1.0]);
                e.pivot = [1.0, 1.0];
            });
            let grid = put(&mut d, None, Kind::Empty, "Levels", [w / 2.0 - 640.0, 200.0, 1280.0, h - 380.0], |e| {
                anchor(e, [0.5, 0.0], [0.5, 1.0]);
                e.layout.mode = LayoutMode::Grid;
                e.layout.cell = [200.0, 200.0];
                e.layout.spacing = [40.0, 40.0];
                e.layout.padding = [20.0, 20.0, 20.0, 20.0];
                e.layout.align = 1;
            });
            for i in 0..12 {
                let open = i < 5;
                put(&mut d, Some(grid), Kind::Button, &format!("Level {}", i + 1), [0.0, 0.0, 200.0, 200.0], |e| {
                    e.text = if open { format!("{}", i + 1) } else { "-".to_string() };
                    e.font_size = 64.0;
                    e.bold = true;
                    e.radius = 28.0;
                    e.fill = if i == 4 { [225, 95, 30, 255] } else if open { [124, 92, 255, 255] } else { [50, 56, 80, 255] };
                    e.text_color = if open { [255, 255, 255, 255] } else { [120, 126, 150, 255] };
                    e.interactable = open;
                    if open {
                        e.on_click = format!("message(\"Load level {}\");", i + 1);
                    }
                });
            }
            put(&mut d, None, Kind::Button, "Back button", [80.0, h - 140.0, 260.0, 80.0], |e| {
                e.text = "Back".into();
                e.fill = [60, 60, 80, 255];
                anchor(e, [0.0, 0.0], [0.0, 0.0]);
                e.pivot = [0.0, 0.0];
            });
        }
        9 => {
            d.title = "Loading screen".into();
            d.background = [10, 12, 20, 255];
            put(&mut d, None, Kind::Image, "Background", [0.0, 0.0, w, h], |e| {
                e.fill = [46, 32, 110, 255];
                e.fill2 = [10, 12, 20, 255];
                e.radius = 0.0;
                e.raycast = false;
                anchor(e, [0.0, 0.0], [1.0, 1.0]);
            });
            put(&mut d, None, Kind::Text, "Level name", [w / 2.0 - 600.0, h * 0.3, 1200.0, 110.0], |e| {
                e.text = "The Crystal Caves".into();
                e.font_size = 84.0;
                e.bold = true;
                e.text_color = [255, 255, 255, 255];
            });
            put(&mut d, None, Kind::Text, "Tip", [w / 2.0 - 650.0, h * 0.3 + 130.0, 1300.0, 60.0], |e| {
                e.text = "Tip: hold the jump button to reach higher.".into();
                e.font_size = 32.0;
                e.italic = true;
                e.text_color = [190, 200, 230, 255];
            });
            put(&mut d, None, Kind::Slider, "Loading bar", [w / 2.0 - 600.0, h - 220.0, 1200.0, 30.0], |e| {
                e.fill = [0, 196, 204, 255];
                e.fill2 = [40, 44, 64, 255];
                e.value = 0.64;
                e.interactable = false;
                anchor(e, [0.5, 0.0], [0.5, 0.0]);
            });
            put(&mut d, None, Kind::Text, "Percentage", [w / 2.0 - 600.0, h - 170.0, 1200.0, 50.0], |e| {
                e.text = "Loading... 64 %".into();
                e.font_size = 30.0;
                e.align = 2;
                e.text_color = [200, 210, 240, 255];
                anchor(e, [0.5, 0.0], [0.5, 0.0]);
            });
            put(&mut d, None, Kind::Ellipse, "Indicator", [w - 150.0, h - 150.0, 80.0, 80.0], |e| {
                e.fill = [0, 0, 0, 0];
                e.stroke = [0, 196, 204, 255];
                e.stroke_w = 8.0;
                e.raycast = false;
                anchor(e, [1.0, 0.0], [1.0, 0.0]);
            });
        }
        10 => {
            d.title = "Character dialogue".into();
            d.background = [70, 120, 90, 255];
            put(&mut d, None, Kind::Image, "Game (preview)", [0.0, 0.0, w, h], |e| {
                e.fill = [255, 200, 150, 255];
                e.fill2 = [120, 80, 130, 255];
                e.radius = 0.0;
                e.raycast = false;
                anchor(e, [0.0, 0.0], [1.0, 1.0]);
            });
            let dialog_box = put(&mut d, None, Kind::Panel, "Dialogue box", [60.0, h - 360.0, w - 120.0, 300.0], |e| {
                e.fill = [16, 16, 28, 230];
                e.stroke = [255, 255, 255, 200];
                e.stroke_w = 3.0;
                e.radius = 24.0;
                anchor(e, [0.0, 0.0], [1.0, 0.0]);
                e.pivot = [0.5, 0.0];
            });
            put(&mut d, Some(dialog_box), Kind::Panel, "Portrait", [100.0, h - 330.0, 240.0, 240.0], |e| {
                e.fill = [124, 92, 255, 255];
                e.fill2 = [0, 196, 204, 255];
                e.radius = 120.0;
                e.stroke = [255, 255, 255, 255];
                e.stroke_w = 4.0;
                anchor(e, [0.0, 0.5], [0.0, 0.5]);
            });
            put(&mut d, Some(dialog_box), Kind::Text, "Name", [380.0, h - 335.0, 600.0, 60.0], |e| {
                e.text = "Old woman of the forest".into();
                e.font_size = 40.0;
                e.bold = true;
                e.align = 0;
                e.text_color = [255, 214, 120, 255];
                anchor(e, [0.0, 1.0], [0.0, 1.0]);
                e.pivot = [0.0, 1.0];
            });
            put(&mut d, Some(dialog_box), Kind::Text, "Line", [380.0, h - 265.0, w - 560.0, 130.0], |e| {
                e.text = "Traveler, the north bridge fell last night. If you are looking for the temple, you will have to cross through the caves.".into();
                e.font_size = 34.0;
                e.align = 0;
                e.valign = 0;
                e.text_color = [240, 240, 250, 255];
                anchor(e, [0.0, 0.0], [1.0, 1.0]);
            });
            put(&mut d, Some(dialog_box), Kind::Button, "Continue button", [w - 340.0, h - 140.0, 230.0, 72.0], |e| {
                e.text = "Continue".into();
                e.font_size = 28.0;
                e.radius = 30.0;
                anchor(e, [1.0, 0.0], [1.0, 0.0]);
                e.on_click = "set_text(\"Line\", \"Be careful with the bats. And don't forget your torch.\");".into();
            });
        }
        11 => {
            d.title = "Leaderboard".into();
            d.background = [20, 22, 34, 255];
            put(&mut d, None, Kind::Text, "Title", [w / 2.0 - 500.0, 60.0, 1000.0, 100.0], |e| {
                e.text = "Leaderboard".into();
                e.font_size = 76.0;
                e.bold = true;
                e.text_color = [255, 255, 255, 255];
                anchor(e, [0.5, 1.0], [0.5, 1.0]);
                e.pivot = [0.5, 1.0];
            });
            let list = put(&mut d, None, Kind::ScrollView, "List", [w / 2.0 - 500.0, 200.0, 1000.0, h - 330.0], |e| {
                e.fill = [32, 35, 52, 255];
                e.stroke_w = 0.0;
                e.radius = 20.0;
                anchor(e, [0.5, 0.0], [0.5, 1.0]);
                e.layout.mode = LayoutMode::Vertical;
                e.layout.spacing = [0.0, 12.0];
                e.layout.padding = [24.0, 24.0, 24.0, 24.0];
                e.layout.align = 1;
                e.layout.expand_w = true;
            });
            let rows = [("Nebula", 98210), ("Kairo", 91440), ("You", 87300), ("Mirla", 80150), ("Shadow", 76020), ("Pixel", 70990)];
            for (i, (n, p)) in rows.iter().enumerate() {
                let medal = [[255, 200, 60, 255], [200, 205, 215, 255], [205, 127, 50, 255]];
                put(&mut d, Some(list), Kind::Panel, &format!("Row {}", i + 1), [0.0, 0.0, 950.0, 84.0], |e| {
                    e.fill = if *n == "You" { [124, 92, 255, 255] } else { [44, 48, 70, 255] };
                    e.radius = 14.0;
                    e.text = format!("{}.  {:<12}  {} pts", i + 1, n, p);
                    e.font_size = 34.0;
                    e.bold = i < 3 || *n == "You";
                    e.text_color = if *n == "You" { [255, 255, 255, 255] } else if i < 3 { medal[i] } else { [235, 235, 245, 255] };
                    e.align = 0;
                });
            }
            put(&mut d, None, Kind::Button, "Back button", [80.0, h - 120.0, 260.0, 80.0], |e| {
                e.text = "Back".into();
                e.fill = [60, 60, 80, 255];
                anchor(e, [0.0, 0.0], [0.0, 0.0]);
                e.pivot = [0.0, 0.0];
            });
        }
        12 => {
            d.title = "Character select".into();
            d.background = [16, 18, 30, 255];
            put(&mut d, None, Kind::Text, "Title", [w / 2.0 - 600.0, 50.0, 1200.0, 100.0], |e| {
                e.text = "Choose your character".into();
                e.font_size = 72.0;
                e.bold = true;
                e.text_color = [255, 255, 255, 255];
                anchor(e, [0.5, 1.0], [0.5, 1.0]);
                e.pivot = [0.5, 1.0];
            });
            let row = put(&mut d, None, Kind::Empty, "Personajes", [w / 2.0 - 820.0, 190.0, 1640.0, 620.0], |e| {
                anchor(e, [0.5, 0.5], [0.5, 0.5]);
                e.layout.mode = LayoutMode::Horizontal;
                e.layout.spacing = [40.0, 0.0];
                e.layout.align = 4;
            });
            let chars = [("Warrior", [235, 70, 90, 255], "High strength"), ("Mage", [124, 92, 255, 255], "High magic"), ("Archer", [70, 190, 120, 255], "High precision"), ("Rogue", [225, 95, 30, 255], "High stealth")];
            for (i, (n, c, stat)) in chars.iter().enumerate() {
                let card = put(&mut d, Some(row), Kind::Panel, &format!("Card {}", n), [0.0, 0.0, 370.0, 600.0], |e| {
                    e.fill = [36, 40, 60, 255];
                    e.radius = 26.0;
                    e.stroke = if i == 1 { [255, 214, 90, 255] } else { [70, 76, 104, 255] };
                    e.stroke_w = if i == 1 { 6.0 } else { 2.0 };
                    e.shadow.on = i == 1;
                    e.layout.mode = LayoutMode::Vertical;
                    e.layout.spacing = [0.0, 16.0];
                    e.layout.padding = [24.0, 24.0, 24.0, 24.0];
                    e.layout.align = 1;
                    e.layout.expand_w = true;
                });
                put(&mut d, Some(card), Kind::Panel, &format!("Portrait {}", n), [0.0, 0.0, 320.0, 360.0], |e| {
                    e.fill = *c;
                    e.fill2 = [20, 20, 30, 255];
                    e.radius = 18.0;
                });
                put(&mut d, Some(card), Kind::Text, &format!("Name {}", n), [0.0, 0.0, 320.0, 60.0], |e| {
                    e.text = n.to_string();
                    e.font_size = 44.0;
                    e.bold = true;
                    e.text_color = [255, 255, 255, 255];
                });
                put(&mut d, Some(card), Kind::Text, &format!("Attribute {}", n), [0.0, 0.0, 320.0, 44.0], |e| {
                    e.text = stat.to_string();
                    e.font_size = 30.0;
                    e.text_color = [200, 206, 230, 255];
                });
            }
            put(&mut d, None, Kind::Button, "Confirm button", [w / 2.0 - 220.0, h - 190.0, 440.0, 100.0], |e| {
                e.text = "Confirmar".into();
                e.fill = [225, 95, 30, 255];
                e.radius = 50.0;
                e.shadow.on = true;
                anchor(e, [0.5, 0.0], [0.5, 0.0]);
                e.on_click = "message(\"Character chosen: Mage\");".into();
            });
        }
        _ => {}
    }
    d.normalize();
    d.apply_layouts();
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-2
    }

    #[test]
    fn centred_element_has_zero_anchored_position() {
        let mut e = Element::new(1, Kind::Button);
        e.w = 400.0;
        e.h = 100.0;
        e.x = 760.0;
        e.y = 490.0;
        let rt = rect_transform(&e, 1920.0, 1080.0);
        assert!(rt.anchored_position[0].abs() < 1e-3 && rt.anchored_position[1].abs() < 1e-3);
        assert_eq!(rt.size_delta, [400.0, 100.0]);
    }

    #[test]
    fn top_left_anchor_and_stretch() {
        let mut e = Element::new(1, Kind::Panel);
        e.x = 40.0;
        e.y = 40.0;
        e.w = 500.0;
        e.h = 100.0;
        e.anchor_min = [0.0, 1.0];
        e.anchor_max = [0.0, 1.0];
        e.pivot = [0.0, 1.0];
        let rt = rect_transform(&e, 1920.0, 1080.0);
        assert_eq!(rt.anchored_position, [40.0, -40.0]);
        let mut bg = Element::new(2, Kind::Image);
        bg.x = 0.0;
        bg.y = 0.0;
        bg.w = 1920.0;
        bg.h = 1080.0;
        bg.anchor_min = [0.0, 0.0];
        bg.anchor_max = [1.0, 1.0];
        let rt = rect_transform(&bg, 1920.0, 1080.0);
        assert_eq!(rt.size_delta, [0.0, 0.0]);
        assert_eq!(rt.anchored_position, [0.0, 0.0]);
    }

    #[test]
    fn rect_transform_round_trip_in_rotated_parent() {
        let parent = Frame { cx: 500.0, cy: 400.0, w: 600.0, h: 300.0, rot: 30.0 };
        let mut e = Element::new(1, Kind::Panel);
        e.x = 420.0;
        e.y = 350.0;
        e.w = 120.0;
        e.h = 80.0;
        e.rot = 45.0;
        e.pivot = [0.2, 0.9];
        e.anchor_min = [0.1, 0.3];
        e.anchor_max = [0.6, 0.3];
        let rt = rect_transform_in(&e, &parent);
        let mut back = e.clone();
        back.x = 0.0;
        back.y = 0.0;
        back.rot = 0.0;
        apply_rect_transform(&mut back, &rt, &parent);
        assert!(close(back.x, e.x) && close(back.y, e.y) && close(back.w, e.w) && close(back.h, e.h) && close(back.rot, e.rot), "{:?}", back);
        assert!(close(rt.rotation_z, -15.0));
    }

    #[test]
    fn children_follow_anchors_when_parent_resizes() {
        let mut d = Doc::empty(1000.0, 800.0);
        let p = put(&mut d, None, Kind::Panel, "P", [100.0, 100.0, 400.0, 300.0], |_| {});
        // stretched horizontally with 20 px margins, glued to the top
        let c = put(&mut d, Some(p), Kind::Panel, "C", [120.0, 110.0, 360.0, 50.0], |e| {
            e.anchor_min = [0.0, 1.0];
            e.anchor_max = [1.0, 1.0];
            e.pivot = [0.5, 1.0];
        });
        // glued to bottom-right corner
        let k = put(&mut d, Some(p), Kind::Button, "K", [380.0, 340.0, 100.0, 40.0], |e| {
            e.anchor_min = [1.0, 0.0];
            e.anchor_max = [1.0, 0.0];
        });
        d.normalize();
        d.set_frame_keep_children(p, &Frame { cx: 400.0, cy: 300.0, w: 600.0, h: 400.0, rot: 0.0 });
        let c = d.get(c).unwrap();
        assert!(close(c.x, 120.0) && close(c.w, 560.0) && close(c.y, 110.0), "{:?}", c.frame());
        let k = d.get(k).unwrap();
        // parent now spans x 100..700, y 100..500; keeps 20 px to the right and bottom
        assert!(close(k.x + k.w, 680.0) && close(k.y + k.h, 480.0), "{:?}", k.frame());
    }

    #[test]
    fn page_resize_and_screen_simulation() {
        let d = template(1, 1920.0, 1080.0);
        let (sim, s) = d.simulate_screen([1080.0, 1920.0]);
        assert!(s > 0.0);
        let pts = sim.elements.iter().find(|e| e.name == "Points").unwrap();
        // still glued 40 px from the right edge of the (narrower) virtual canvas
        assert!(close(sim.width - (pts.x + pts.w), 40.0), "{} {}", sim.width, pts.x + pts.w);
    }

    #[test]
    fn layout_groups_place_children() {
        let mut d = Doc::empty(1000.0, 1000.0);
        let p = put(&mut d, None, Kind::Panel, "P", [0.0, 0.0, 500.0, 200.0], |e| {
            e.layout.mode = LayoutMode::Horizontal;
            e.layout.padding = [10.0, 10.0, 10.0, 10.0];
            e.layout.spacing = [20.0, 0.0];
            e.layout.expand_w = true;
            e.layout.expand_h = true;
        });
        let a = put(&mut d, Some(p), Kind::Button, "A", [600.0, 600.0, 50.0, 50.0], |_| {});
        let b = put(&mut d, Some(p), Kind::Button, "B", [700.0, 700.0, 50.0, 50.0], |_| {});
        d.normalize();
        d.apply_layouts();
        let (a, b) = (d.get(a).unwrap(), d.get(b).unwrap());
        assert!(close(a.x, 10.0) && close(a.w, 230.0) && close(a.h, 180.0));
        assert!(close(b.x, 260.0) && close(b.y, 10.0));
        // grid
        let t = template(4, 1920.0, 1080.0);
        let cells: Vec<&Element> = t.elements.iter().filter(|e| e.name.starts_with("Slot")).collect();
        assert_eq!(cells.len(), 12);
        assert!(close(cells[1].x - cells[0].x, 184.0));
    }

    #[test]
    fn old_groups_become_parents_and_order_is_depth_first() {
        let mut d = Doc::empty(1000.0, 1000.0);
        let a = put(&mut d, None, Kind::Panel, "A", [0.0, 0.0, 10.0, 10.0], |e| e.group = Some(99));
        put(&mut d, None, Kind::Panel, "X", [50.0, 0.0, 10.0, 10.0], |_| {});
        let b = put(&mut d, None, Kind::Panel, "B", [100.0, 100.0, 10.0, 10.0], |e| e.group = Some(99));
        d.migrate_groups();
        let pa = d.parent_of(a).unwrap();
        assert_eq!(d.parent_of(b), Some(pa));
        assert_eq!(d.get(pa).unwrap().kind, Kind::Empty);
        // depth first: group, A, B, X
        let names: Vec<&str> = d.elements.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names[1..3], ["A", "B"]);
        // reparent + sibling order
        assert!(d.reparent(pa, None, Some(1)));
        assert_eq!(d.children(None).len(), 2);
        assert!(!d.reparent(pa, Some(a), None), "no cycles");
        let copy = d.collect_subtrees(&[pa]);
        let roots = d.insert_items(&copy, None);
        assert_eq!(roots.len(), 1);
        assert_eq!(d.descendants(roots[0]).len(), 2);
    }
}

#[cfg(test)]
mod sanitize_tests {
    use super::*;

    fn doc_with(n: usize) -> (Doc, Vec<u64>) {
        let mut d = Doc::empty(100.0, 100.0);
        let ids = (0..n).map(|_| d.add(Element::new(0, Kind::Panel))).collect();
        (d, ids)
    }

    #[test]
    fn cyclic_or_self_parents_do_not_hang() {
        let (mut d, ids) = doc_with(3);
        d.get_mut(ids[0]).unwrap().parent = Some(ids[1]);
        d.get_mut(ids[1]).unwrap().parent = Some(ids[0]);
        d.get_mut(ids[2]).unwrap().parent = Some(ids[2]);
        d.migrate_groups();
        assert!(d.elements.iter().all(|e| e.parent != Some(e.id)));
        for e in &d.elements {
            let _ = d.descendants(e.id);
            let _ = d.ancestors(e.id);
        }
    }

    #[test]
    fn descendants_with_cycle_terminates_even_if_not_sanitized() {
        let (mut d, ids) = doc_with(2);
        d.get_mut(ids[0]).unwrap().parent = Some(ids[1]);
        d.get_mut(ids[1]).unwrap().parent = Some(ids[0]);
        assert!(d.descendants(ids[0]).len() <= 2);
    }

    #[test]
    fn duplicate_ids_and_low_next_id() {
        let (mut d, ids) = doc_with(3);
        d.get_mut(ids[2]).unwrap().id = ids[1];
        d.next_id = 1;
        d.migrate_groups();
        let mut v: Vec<u64> = d.elements.iter().map(|e| e.id).collect();
        v.sort();
        v.dedup();
        assert_eq!(v.len(), 3);
        assert!(d.next_id > *v.last().unwrap());
        let new = d.add(Element::new(0, Kind::Panel));
        assert_eq!(d.elements.iter().filter(|e| e.id == new).count(), 1);
    }

    #[test]
    fn non_numeric_coords_are_fixed() {
        let (mut d, ids) = doc_with(1);
        d.get_mut(ids[0]).unwrap().x = f32::NAN;
        d.get_mut(ids[0]).unwrap().w = f32::INFINITY;
        d.migrate_groups();
        let e = d.get(ids[0]).unwrap();
        assert!(e.x.is_finite() && e.w.is_finite());
    }
}

#[cfg(test)]
mod sanitize_extra_tests {
    use super::*;

    #[test]
    fn absurd_values_and_control_chars() {
        let mut d = Doc::empty(1920.0, 1080.0);
        let mut e = Element::new(0, Kind::Text);
        e.name = "a\u{0}b\u{7}".into();
        e.text = "uno\ndos\u{0}".into();
        e.font_size = 1.0e9;
        e.w = -40.0;
        e.opacity = 7.0;
        e.stroke_w = -3.0;
        e.rot = 1.0e30;
        d.add(e);
        d.title = "T\u{1}".into();
        d.sanitize();
        let e = &d.elements[0];
        assert_eq!(e.name, "ab");
        assert_eq!(e.text, "uno\ndos");
        assert_eq!(e.font_size, 1000.0);
        assert_eq!((e.w, e.opacity, e.stroke_w), (0.0, 1.0, 0.0));
        assert!(e.rot.abs() < 360.0);
        assert_eq!(d.title, "T");
        let mut p = Doc::empty(0.0, f32::INFINITY);
        p.sanitize();
        assert_eq!((p.width, p.height), (1920.0, 1080.0));
    }
}

#[cfg(test)]
mod template_tests {
    use super::*;

    #[test]
    fn every_template_is_sane_at_several_resolutions() {
        assert_eq!(template_order().count(), TEMPLATES.len());
        let mut d = template(1, 1920.0, 1080.0);
        d.elements[2].visible = false;
        let all = d.shown_all();
        for e in &d.elements {
            assert_eq!(all.contains(&e.id), d.shown(e.id), "{}", e.name);
        }
        assert_eq!(template_order().last(), Some(blank_template()));
        assert_eq!(TEMPLATES[blank_template()], "Blank");
        for &(w, h) in &[(1920.0, 1080.0), (1280.0, 720.0), (2560.0, 1440.0), (1080.0, 1920.0), (800.0, 600.0)] {
            for i in 0..TEMPLATES.len() {
                let d = template(i, w, h);
                if i == blank_template() {
                    assert!(d.elements.is_empty());
                    continue;
                }
                assert!(!d.elements.is_empty(), "{} empty", TEMPLATES[i]);
                assert_eq!(d.title, TEMPLATES[i], "the title matches the name");
                let mut ids = std::collections::HashSet::new();
                for e in &d.elements {
                    assert!(ids.insert(e.id), "{}: duplicate id", TEMPLATES[i]);
                    assert!(e.w > 0.0 && e.h > 0.0 && e.w.is_finite() && e.h.is_finite(), "{}: {} has no size", TEMPLATES[i], e.name);
                    if let Some(p) = e.parent {
                        assert!(d.get(p).is_some(), "{}: missing parent", TEMPLATES[i]);
                    }
                    // Nothing goes outside the page (margin for shadows/borders).
                    assert!(e.x >= -1.0 && e.y >= -1.0 && e.x + e.w <= w + 1.0 && e.y + e.h <= h + 1.0, "{} @{}x{}: “{}” outside the page {:?}", TEMPLATES[i], w, h, e.name, [e.x, e.y, e.w, e.h]);
                }
            }
        }
    }
}
