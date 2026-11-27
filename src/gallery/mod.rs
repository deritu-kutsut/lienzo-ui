//! Template gallery: three starting layouts for each kind of game screen, grouped the way game UI references
//! usually are (title and system menus, game states, stats, information, meta-game, HUD, gameplay and controls).
//!
//! Every gallery template is written with the small builder in this file ([`G`]). Positions are given like a Unity
//! RectTransform (an anchor in the parent, an offset and a size, all in the 1920 × 1080 reference), so one description
//! works on every page size: the builder scales it by `min(width / 1920, height / 1080)` and keeps each element on its
//! anchor (corners stay in the corners, centred blocks stay centred). Anything that fits the 1920 × 1080 reference fits
//! every page, portrait included. See docs/EXTENDING.md («Add a gallery template»).

use crate::model::{Doc, Element, Kind, LayoutMode, Rgba};

mod controls;
mod hud;
mod info;
mod meta;
mod states;
mod stats;
mod title;

// ---------------------------------------------------------------------------------------------------------------
// Catalog
// ---------------------------------------------------------------------------------------------------------------

/// A section of the gallery (a heading in the Templates tab) and the screen types it holds.
pub struct Section {
    pub group: &'static str,
    pub name: &'static str,
    pub types: &'static [&'static str],
}

/// The screen types, in the order they are shown. Each one has exactly three templates (a test checks it).
pub const SECTIONS: &[Section] = &[
    Section { group: "Title & system menus", name: "Title", types: &["Title screen", "Mode select", "Game difficulty", "Load / save", "Loading screen", "Credits"] },
    Section { group: "Title & system menus", name: "Settings", types: &["Settings menu", "Settings options", "Button layout"] },
    Section { group: "Title & system menus", name: "Modals & overlays", types: &["Cutscene & story", "Dialogue & speech", "Guided tutorial", "Modal & popup", "Character intro", "Feature unlocked"] },
    Section { group: "Game states", name: "Pre-game", types: &["Character select screen", "Stage / level select", "Game setup", "Pre-game lobby"] },
    Section { group: "Game states", name: "Mid-game", types: &["Stage intro", "Pause screen", "Change class / party", "Mission accept", "Loot selection"] },
    Section { group: "Game states", name: "Game end", types: &["Failure & game over", "Level complete", "Results screen", "Rewards & experience", "Leaderboards & ranking", "Post-game menu"] },
    Section { group: "Stats & resources", name: "Stats", types: &["Player menu", "Unlocked units", "Team summary", "Overview & stats", "Ability list", "Maintenance & management"] },
    Section { group: "Stats & resources", name: "Resource management", types: &["Inventory grid", "Equipping", "Buying & trading", "Change skin or accessory", "Character customization", "Sub-equip & tweak", "World customization"] },
    Section { group: "Stats & resources", name: "Upgrades & unlocks", types: &["Upgrading & ranking up", "Unlockable upgrades", "Crafting", "Choose boon / upgrade"] },
    Section { group: "Information", name: "Knowledge", types: &["Codex & journal", "Tutorials & guides", "Logs & history"] },
    Section { group: "Information", name: "Progress", types: &["Area map", "Missions & quests", "Challenges & achievements", "Profile & game totals"] },
    Section { group: "Information", name: "Collection", types: &["Collection & extras hub", "Collectables menu", "Gallery menu", "Collectable viewer", "Sound & music player"] },
    Section { group: "Meta-game", name: "Service & social", types: &["News & notifications", "Profile customization", "Friends & players", "Chat & communication", "Online play"] },
    Section { group: "Meta-game", name: "Content creation", types: &["Photo mode", "Level editor", "Decals & liveries", "Content discovery"] },
    Section { group: "Meta-game", name: "Monetisation", types: &["Premium store", "Offers & bundles", "Currency store", "Daily rewards", "Season pass & events", "Gacha & crates", "Upcoming unlocks"] },
    Section { group: "HUD", name: "Stats & resources", types: &["Player vitals", "Equipped items & abilities", "Collection counters", "Rank & position", "Enemy health & damage"] },
    Section { group: "HUD", name: "HUD messages", types: &["Info & notifications", "Area / level name", "Loot & XP log", "Scoring & combos"] },
    Section { group: "HUD", name: "Goal & navigation", types: &["Clock & timer", "Objectives & goals", "Waypoints & markers", "Minimap", "Compass bar"] },
    Section { group: "Gameplay & controls", name: "Overlay menus", types: &["Item & ability wheel", "Selection & targeting", "Object placement", "Chat shortcuts & emotes"] },
    Section { group: "Gameplay & controls", name: "Controls", types: &["Button prompts", "Weapon pickup prompt", "Quick time event", "Controls reference", "Item & ability buttons", "Mobile controls"] },
    Section { group: "Gameplay & controls", name: "Misc", types: &["Spectral vision", "Weapon reticles", "Object / NPC info", "Minigame", "Hacking & lockpicking"] },
];

/// Groups in display order (each one is a collapsible block in the Templates tab).
pub fn groups() -> Vec<&'static str> {
    let mut out: Vec<&'static str> = Vec::new();
    for s in SECTIONS {
        if !out.contains(&s.group) {
            out.push(s.group);
        }
    }
    out
}

/// One gallery template: its screen type (one of [`SECTIONS`]), its visual style and the function that lays it out.
pub struct Entry {
    pub ty: &'static str,
    pub style: Style,
    pub build: fn(&mut G),
}

fn all_entries() -> Vec<Entry> {
    let mut v = Vec::new();
    v.extend(title::entries());
    v.extend(states::entries());
    v.extend(stats::entries());
    v.extend(info::entries());
    v.extend(meta::entries());
    v.extend(hud::entries());
    v.extend(controls::entries());
    v
}

static ENTRIES: std::sync::LazyLock<Vec<Entry>> = std::sync::LazyLock::new(all_entries);

static NAMES: std::sync::LazyLock<Vec<&'static str>> =
    std::sync::LazyLock::new(|| ENTRIES.iter().map(|e| &*Box::leak(format!("{} · {}", e.ty, e.style.name()).into_boxed_str())).collect());

/// Number of gallery templates.
pub fn count() -> usize {
    ENTRIES.len()
}

/// English name of gallery template `i` («Title screen · Fantasy»): stable, used by the command line and the MCP.
pub fn name(i: usize) -> &'static str {
    NAMES[i]
}

pub fn entry(i: usize) -> &'static Entry {
    &ENTRIES[i]
}

/// Name in the interface language.
pub fn label(i: usize) -> String {
    let e = &ENTRIES[i];
    format!("{} · {}", crate::i18n::t(e.ty), crate::i18n::t(e.style.name()))
}

/// Gallery template indices of a screen type, in their order.
pub fn of_type(ty: &str) -> Vec<usize> {
    ENTRIES.iter().enumerate().filter(|(_, e)| e.ty == ty).map(|(i, _)| i).collect()
}

/// Builds gallery template `i` on a page of `width` × `height` (already at least the reference size, see `model::template`).
pub fn build(i: usize, width: f32, height: f32) -> Doc {
    let e = &ENTRIES[i];
    let mut d = Doc::empty(width, height);
    d.title = name(i).to_string();
    let t = e.style.theme();
    d.background = t.bg;
    {
        let mut g = G { d: &mut d, s: (width / 1920.0).min(height / 1080.0), t, names: std::collections::HashMap::new() };
        (e.build)(&mut g);
    }
    d.normalize();
    d.apply_layouts();
    d
}

// ---------------------------------------------------------------------------------------------------------------
// Styles
// ---------------------------------------------------------------------------------------------------------------

/// Visual style of a gallery template. The three templates of a screen type use three different styles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Style {
    Fantasy,
    SciFi,
    Minimal,
    Cartoon,
    Neon,
    Military,
    Horror,
    Sports,
    Retro,
    Elegant,
    Nature,
    Pastel,
}

pub const STYLES: [Style; 12] = [Style::Fantasy, Style::SciFi, Style::Minimal, Style::Cartoon, Style::Neon, Style::Military, Style::Horror, Style::Sports, Style::Retro, Style::Elegant, Style::Nature, Style::Pastel];

/// Colours and shapes of a style. Contrast pairs that templates rely on (checked by a test):
/// `text`/`muted` on `panel` and `panel2`, `on_accent` on `accent`, `bg_text` on `bg` and `bg2`.
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    /// Page background (gradient from `bg` to `bg2`).
    pub bg: Rgba,
    pub bg2: Rgba,
    /// Text placed straight on the page background.
    pub bg_text: Rgba,
    /// Main panels and the raised cards inside them.
    pub panel: Rgba,
    pub panel2: Rgba,
    /// Borders and dividers.
    pub line: Rgba,
    /// Main text and secondary text (on panels).
    pub text: Rgba,
    pub muted: Rgba,
    /// Highlight colour (primary buttons, selected items) and the text on it.
    pub accent: Rgba,
    pub on_accent: Rgba,
    /// Second highlight (art placeholders, secondary bars, tags).
    pub accent2: Rgba,
    pub good: Rgba,
    pub bad: Rgba,
    pub warn: Rgba,
    /// Fake game scene behind HUD templates: sky top, sky bottom, ground.
    pub sky: Rgba,
    pub sky2: Rgba,
    pub ground: Rgba,
    /// Corner radius and border width (reference pixels).
    pub radius: f32,
    pub stroke: f32,
}

impl Style {
    pub fn name(self) -> &'static str {
        match self {
            Style::Fantasy => "Fantasy",
            Style::SciFi => "Sci-fi",
            Style::Minimal => "Minimal",
            Style::Cartoon => "Cartoon",
            Style::Neon => "Neon",
            Style::Military => "Military",
            Style::Horror => "Horror",
            Style::Sports => "Sports",
            Style::Retro => "Retro",
            Style::Elegant => "Elegant",
            Style::Nature => "Nature",
            Style::Pastel => "Pastel",
        }
    }

    pub fn theme(self) -> Theme {
        const W: Rgba = [255, 255, 255, 255];
        match self {
            Style::Fantasy => Theme {
                bg: [38, 24, 16, 255], bg2: [92, 58, 30, 255], bg_text: [250, 236, 206, 255],
                panel: [52, 34, 22, 240], panel2: [74, 50, 32, 255], line: [196, 150, 72, 255],
                text: [250, 236, 206, 255], muted: [214, 190, 150, 255], accent: [214, 162, 64, 255], on_accent: [40, 24, 10, 255],
                accent2: [150, 40, 40, 255], good: [120, 190, 90, 255], bad: [220, 80, 60, 255], warn: [240, 190, 80, 255],
                sky: [96, 130, 170, 255], sky2: [210, 180, 140, 255], ground: [70, 90, 50, 255], radius: 6.0, stroke: 3.0,
            },
            Style::SciFi => Theme {
                bg: [6, 14, 28, 255], bg2: [12, 46, 78, 255], bg_text: [226, 246, 255, 255],
                panel: [10, 24, 44, 235], panel2: [16, 38, 64, 255], line: [70, 200, 240, 255],
                text: [226, 246, 255, 255], muted: [150, 190, 215, 255], accent: [60, 210, 250, 255], on_accent: [4, 20, 34, 255],
                accent2: [250, 120, 60, 255], good: [80, 230, 150, 255], bad: [255, 80, 90, 255], warn: [255, 200, 70, 255],
                sky: [10, 20, 50, 255], sky2: [60, 40, 90, 255], ground: [30, 34, 48, 255], radius: 2.0, stroke: 2.0,
            },
            Style::Minimal => Theme {
                bg: [236, 238, 242, 255], bg2: [210, 216, 226, 255], bg_text: [24, 28, 38, 255],
                panel: W, panel2: [244, 246, 250, 255], line: [200, 206, 216, 255],
                text: [24, 28, 38, 255], muted: [84, 92, 108, 255], accent: [40, 90, 220, 255], on_accent: W,
                accent2: [230, 80, 60, 255], good: [30, 150, 90, 255], bad: [210, 60, 60, 255], warn: [220, 150, 20, 255],
                sky: [190, 215, 240, 255], sky2: [240, 244, 248, 255], ground: [160, 170, 180, 255], radius: 12.0, stroke: 2.0,
            },
            Style::Cartoon => Theme {
                bg: [40, 110, 220, 255], bg2: [110, 60, 200, 255], bg_text: W,
                panel: [255, 250, 236, 255], panel2: [255, 232, 186, 255], line: [40, 30, 80, 255],
                text: [40, 30, 80, 255], muted: [96, 78, 128, 255], accent: [200, 70, 10, 255], on_accent: W,
                accent2: [60, 200, 120, 255], good: [60, 200, 90, 255], bad: [240, 60, 70, 255], warn: [255, 200, 40, 255],
                sky: [110, 200, 255, 255], sky2: [200, 240, 255, 255], ground: [90, 190, 80, 255], radius: 28.0, stroke: 4.0,
            },
            Style::Neon => Theme {
                bg: [8, 4, 18, 255], bg2: [40, 10, 60, 255], bg_text: [250, 240, 255, 255],
                panel: [18, 10, 34, 235], panel2: [30, 16, 54, 255], line: [255, 60, 200, 255],
                text: [250, 240, 255, 255], muted: [200, 170, 230, 255], accent: [255, 50, 190, 255], on_accent: [20, 4, 24, 255],
                accent2: [40, 240, 240, 255], good: [60, 250, 160, 255], bad: [255, 70, 90, 255], warn: [255, 220, 60, 255],
                sky: [20, 6, 40, 255], sky2: [90, 20, 90, 255], ground: [16, 10, 30, 255], radius: 4.0, stroke: 2.0,
            },
            Style::Military => Theme {
                bg: [30, 34, 26, 255], bg2: [70, 76, 52, 255], bg_text: [234, 236, 220, 255],
                panel: [36, 40, 30, 240], panel2: [52, 58, 42, 255], line: [170, 180, 120, 255],
                text: [234, 236, 220, 255], muted: [180, 186, 160, 255], accent: [200, 190, 90, 255], on_accent: [26, 28, 18, 255],
                accent2: [210, 90, 40, 255], good: [130, 200, 90, 255], bad: [220, 70, 50, 255], warn: [240, 190, 60, 255],
                sky: [150, 160, 150, 255], sky2: [200, 196, 170, 255], ground: [90, 84, 60, 255], radius: 0.0, stroke: 2.0,
            },
            Style::Horror => Theme {
                bg: [10, 6, 6, 255], bg2: [60, 8, 10, 255], bg_text: [236, 226, 220, 255],
                panel: [20, 12, 12, 240], panel2: [36, 18, 18, 255], line: [140, 30, 30, 255],
                text: [236, 226, 220, 255], muted: [180, 160, 150, 255], accent: [190, 30, 40, 255], on_accent: [255, 240, 235, 255],
                accent2: [230, 200, 160, 255], good: [120, 170, 110, 255], bad: [220, 40, 40, 255], warn: [220, 160, 60, 255],
                sky: [24, 20, 26, 255], sky2: [60, 40, 40, 255], ground: [20, 16, 14, 255], radius: 2.0, stroke: 2.0,
            },
            Style::Sports => Theme {
                bg: [14, 18, 30, 255], bg2: [30, 40, 70, 255], bg_text: W,
                panel: [255, 255, 255, 245], panel2: [236, 240, 246, 255], line: [210, 30, 50, 255],
                text: [16, 20, 34, 255], muted: [80, 88, 104, 255], accent: [210, 30, 50, 255], on_accent: W,
                accent2: [20, 100, 220, 255], good: [30, 160, 80, 255], bad: [210, 30, 50, 255], warn: [230, 160, 20, 255],
                sky: [90, 160, 230, 255], sky2: [200, 225, 245, 255], ground: [60, 140, 60, 255], radius: 4.0, stroke: 3.0,
            },
            Style::Retro => Theme {
                bg: [24, 20, 44, 255], bg2: [60, 40, 90, 255], bg_text: [255, 250, 230, 255],
                panel: [44, 36, 74, 255], panel2: [62, 52, 100, 255], line: [255, 220, 120, 255],
                text: [255, 250, 230, 255], muted: [200, 190, 230, 255], accent: [250, 200, 60, 255], on_accent: [30, 20, 40, 255],
                accent2: [90, 220, 140, 255], good: [90, 220, 140, 255], bad: [240, 80, 90, 255], warn: [250, 200, 60, 255],
                sky: [70, 120, 200, 255], sky2: [150, 200, 240, 255], ground: [70, 140, 60, 255], radius: 0.0, stroke: 4.0,
            },
            Style::Elegant => Theme {
                bg: [16, 16, 20, 255], bg2: [44, 40, 36, 255], bg_text: [244, 238, 226, 255],
                panel: [24, 24, 28, 240], panel2: [36, 34, 34, 255], line: [200, 170, 110, 255],
                text: [244, 238, 226, 255], muted: [184, 176, 160, 255], accent: [200, 170, 110, 255], on_accent: [20, 18, 14, 255],
                accent2: [140, 150, 170, 255], good: [140, 190, 130, 255], bad: [210, 90, 80, 255], warn: [220, 180, 90, 255],
                sky: [40, 44, 60, 255], sky2: [120, 110, 100, 255], ground: [40, 36, 32, 255], radius: 2.0, stroke: 2.0,
            },
            Style::Nature => Theme {
                bg: [20, 48, 36, 255], bg2: [50, 100, 56, 255], bg_text: [240, 246, 230, 255],
                panel: [244, 240, 224, 250], panel2: [228, 222, 198, 255], line: [90, 130, 70, 255],
                text: [34, 40, 28, 255], muted: [80, 88, 64, 255], accent: [50, 120, 60, 255], on_accent: W,
                accent2: [210, 140, 50, 255], good: [60, 160, 70, 255], bad: [200, 70, 50, 255], warn: [220, 160, 40, 255],
                sky: [140, 200, 230, 255], sky2: [220, 236, 210, 255], ground: [80, 140, 60, 255], radius: 16.0, stroke: 2.0,
            },
            Style::Pastel => Theme {
                bg: [250, 226, 236, 255], bg2: [210, 226, 250, 255], bg_text: [60, 40, 70, 255],
                panel: W, panel2: [255, 244, 248, 255], line: [230, 170, 200, 255],
                text: [60, 40, 70, 255], muted: [104, 84, 116, 255], accent: [140, 80, 190, 255], on_accent: W,
                accent2: [60, 170, 160, 255], good: [80, 180, 120, 255], bad: [230, 90, 110, 255], warn: [240, 180, 80, 255],
                sky: [200, 225, 255, 255], sky2: [255, 235, 245, 255], ground: [170, 220, 170, 255], radius: 24.0, stroke: 2.0,
            },
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Positions (RectTransform-like, in the 1920 × 1080 reference)
// ---------------------------------------------------------------------------------------------------------------

/// Where an element goes inside its parent. On each axis it is either *pinned* to an anchor (offset + size, the pivot
/// sits on the anchor) or *stretched* between two anchors (insets from each side). Fractions go left→right and
/// top→bottom (0 = left/top edge of the parent, 1 = right/bottom). Build them with the short functions below.
#[derive(Clone, Copy, Debug)]
pub struct Pos {
    amin: [f32; 2],
    amax: [f32; 2],
    /// x axis: pinned → (offset, width); stretched → (left inset, right inset)
    xs: [f32; 2],
    /// y axis: pinned → (offset, height); stretched → (top inset, bottom inset)
    ys: [f32; 2],
    /// Child of a Layout Group: only the size counts.
    auto: bool,
}

/// Pinned to the anchor (ax, ay) of the parent with the pivot on the anchor; (x, y) moves it (+ right, + down).
pub fn at(ax: f32, ay: f32, x: f32, y: f32, w: f32, h: f32) -> Pos {
    Pos { amin: [ax, ay], amax: [ax, ay], xs: [x, w], ys: [y, h], auto: false }
}
/// Top-left corner, `mx`/`my` inward margins.
pub fn tl(mx: f32, my: f32, w: f32, h: f32) -> Pos {
    at(0.0, 0.0, mx, my, w, h)
}
/// Top-right corner, inward margins.
pub fn tr(mx: f32, my: f32, w: f32, h: f32) -> Pos {
    at(1.0, 0.0, -mx, my, w, h)
}
/// Bottom-left corner, inward margins.
pub fn bl(mx: f32, my: f32, w: f32, h: f32) -> Pos {
    at(0.0, 1.0, mx, -my, w, h)
}
/// Bottom-right corner, inward margins.
pub fn br(mx: f32, my: f32, w: f32, h: f32) -> Pos {
    at(1.0, 1.0, -mx, -my, w, h)
}
/// Top edge, centred horizontally: `dx` signed offset, `my` margin from the top.
pub fn tc(dx: f32, my: f32, w: f32, h: f32) -> Pos {
    at(0.5, 0.0, dx, my, w, h)
}
/// Bottom edge, centred horizontally: `dx` signed offset, `my` margin from the bottom.
pub fn bc(dx: f32, my: f32, w: f32, h: f32) -> Pos {
    at(0.5, 1.0, dx, -my, w, h)
}
/// Left edge, centred vertically: `mx` margin, `dy` signed offset.
pub fn cl(mx: f32, dy: f32, w: f32, h: f32) -> Pos {
    at(0.0, 0.5, mx, dy, w, h)
}
/// Right edge, centred vertically: `mx` margin, `dy` signed offset.
pub fn cr(mx: f32, dy: f32, w: f32, h: f32) -> Pos {
    at(1.0, 0.5, -mx, dy, w, h)
}
/// Centre of the parent, signed offsets.
pub fn cc(dx: f32, dy: f32, w: f32, h: f32) -> Pos {
    at(0.5, 0.5, dx, dy, w, h)
}
/// Stretched over the whole parent with insets (left, top, right, bottom).
pub fn fill(l: f32, t: f32, r: f32, b: f32) -> Pos {
    Pos { amin: [0.0, 0.0], amax: [1.0, 1.0], xs: [l, r], ys: [t, b], auto: false }
}
/// The whole parent.
pub fn full() -> Pos {
    fill(0.0, 0.0, 0.0, 0.0)
}
/// Full-width strip at the top: `my` from the top, height `h`, side insets `l`/`r`.
pub fn top(my: f32, h: f32, l: f32, r: f32) -> Pos {
    Pos { amin: [0.0, 0.0], amax: [1.0, 0.0], xs: [l, r], ys: [my, h], auto: false }
}
/// Full-width strip at the bottom: `my` from the bottom.
pub fn bottom(my: f32, h: f32, l: f32, r: f32) -> Pos {
    Pos { amin: [0.0, 1.0], amax: [1.0, 1.0], xs: [l, r], ys: [-my, h], auto: false }
}
/// Full-width strip centred vertically (`dy` signed offset).
pub fn hmid(dy: f32, h: f32, l: f32, r: f32) -> Pos {
    Pos { amin: [0.0, 0.5], amax: [1.0, 0.5], xs: [l, r], ys: [dy, h], auto: false }
}
/// Full-height column on the left: `mx` from the left, width `w`, insets `t`/`b`.
pub fn left(mx: f32, w: f32, t: f32, b: f32) -> Pos {
    Pos { amin: [0.0, 0.0], amax: [0.0, 1.0], xs: [mx, w], ys: [t, b], auto: false }
}
/// Full-height column on the right: `mx` from the right.
pub fn right(mx: f32, w: f32, t: f32, b: f32) -> Pos {
    Pos { amin: [1.0, 0.0], amax: [1.0, 1.0], xs: [-mx, w], ys: [t, b], auto: false }
}
/// Full-height column centred horizontally (`dx` signed offset).
pub fn vmid(dx: f32, w: f32, t: f32, b: f32) -> Pos {
    Pos { amin: [0.5, 0.0], amax: [0.5, 1.0], xs: [dx, w], ys: [t, b], auto: false }
}
/// Child of a Layout Group (vbox/hbox/grid): only its size; the group places it.
pub fn sz(w: f32, h: f32) -> Pos {
    Pos { amin: [0.5, 0.5], amax: [0.5, 0.5], xs: [0.0, w], ys: [0.0, h], auto: true }
}

/// An element that was added: its id and its box on the page (x, y, w, h). Pass it as the parent of other elements.
#[derive(Clone, Copy, Debug)]
pub struct N {
    pub id: Option<u64>,
    pub r: [f32; 4],
}

// ---------------------------------------------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------------------------------------------

/// Builds one gallery template. `t` is the style's palette; `page()` is the root to hang elements from.
pub struct G<'a> {
    d: &'a mut Doc,
    /// Reference → page scale.
    pub s: f32,
    pub t: Theme,
    names: std::collections::HashMap<String, usize>,
}

/// Text alignment for [`G::txt`].
pub const LEFT: u8 = 0;
pub const CENTER: u8 = 1;
pub const RIGHT: u8 = 2;

/// Darker or lighter copy of a colour (`k` < 1 darkens, > 1 lightens towards white).
pub fn shade(c: Rgba, k: f32) -> Rgba {
    let f = |v: u8| if k <= 1.0 { (v as f32 * k).round().clamp(0.0, 255.0) as u8 } else { (v as f32 + (255.0 - v as f32) * (k - 1.0)).round().clamp(0.0, 255.0) as u8 };
    [f(c[0]), f(c[1]), f(c[2]), c[3]]
}

/// The same colour with another alpha.
pub fn alpha(c: Rgba, a: u8) -> Rgba {
    [c[0], c[1], c[2], a]
}

impl G<'_> {
    /// The page (root parent).
    pub fn page(&self) -> N {
        N { id: None, r: [0.0, 0.0, self.d.width, self.d.height] }
    }

    fn unique(&mut self, name: &str) -> String {
        let n = self.names.entry(name.to_string()).or_insert(0);
        *n += 1;
        if *n == 1 { name.to_string() } else { format!("{} {}", name, n) }
    }

    /// Adds any element: `f` sets its look after the position, anchors and name are filled in. Sizes inside `f` are
    /// page pixels: multiply by `g.s` (or use the helpers, which already do).
    pub fn el(&mut self, p: N, pos: Pos, kind: Kind, name: &str, f: impl FnOnce(&mut Element)) -> N {
        let s = self.s;
        let [px, py, pw, ph] = p.r;
        let mut e = Element::new(0, kind);
        e.name = self.unique(name);
        e.parent = p.id;
        if pos.auto {
            e.x = px;
            e.y = py;
            e.w = pos.xs[1] * s;
            e.h = pos.ys[1] * s;
        } else {
            let mut anchor_min = [0.0f32; 2];
            let mut anchor_max = [0.0f32; 2];
            let mut pivot = [0.5f32; 2];
            let mut rect = [0.0f32; 4];
            for axis in 0..2 {
                let (o, len) = if axis == 0 { (px, pw) } else { (py, ph) };
                let v = if axis == 0 { pos.xs } else { pos.ys };
                let (a0, a1) = (pos.amin[axis], pos.amax[axis]);
                let (start, size) = if (a0 - a1).abs() < 1e-6 {
                    let size = v[1] * s;
                    let pv = o + a0 * len + v[0] * s;
                    pivot[axis] = a0;
                    (pv - a0 * size, size)
                } else {
                    let start = o + a0 * len + v[0] * s;
                    let end = o + a1 * len - v[1] * s;
                    (start, (end - start).max(1.0))
                };
                rect[axis] = start;
                rect[axis + 2] = size;
                // Unity's y goes up: page fraction f is anchor 1 - f.
                if axis == 0 {
                    anchor_min[0] = a0;
                    anchor_max[0] = a1;
                } else {
                    anchor_min[1] = 1.0 - a1;
                    anchor_max[1] = 1.0 - a0;
                    pivot[1] = 1.0 - pivot[1];
                }
            }
            e.x = rect[0];
            e.y = rect[1];
            e.w = rect[2];
            e.h = rect[3];
            e.anchor_min = anchor_min;
            e.anchor_max = anchor_max;
            e.pivot = pivot;
        }
        e.radius = 0.0;
        f(&mut e);
        let r = [e.x, e.y, e.w, e.h];
        let id = self.d.add(e);
        N { id: Some(id), r }
    }

    /// Changes an element after it was added (colours, scripts, layout…).
    pub fn edit(&mut self, n: N, f: impl FnOnce(&mut Element)) {
        if let Some(e) = n.id.and_then(|id| self.d.get_mut(id)) {
            f(e);
        }
    }

    /// Makes `n` call a script when clicked (see the Script tab: message(), toggle(), set_text()…).
    pub fn click(&mut self, n: N, script: &str) {
        self.edit(n, |e| e.on_click = script.to_string());
    }

    // ---- backgrounds ------------------------------------------------------------------------------------------

    /// Full-page gradient in the style's background colours (put it first).
    pub fn background(&mut self) -> N {
        let (a, b) = (self.t.bg, self.t.bg2);
        let p = self.page();
        self.el(p, full(), Kind::Image, "Background", |e| {
            e.fill = a;
            e.fill2 = b;
            e.raycast = false;
        })
    }

    /// Fake game scene for HUD templates: sky and ground. Never put text straight on it: use a panel behind text.
    pub fn scene(&mut self) -> N {
        let t = self.t;
        let p = self.page();
        let sky = self.el(p, full(), Kind::Image, "Scene (preview)", |e| {
            e.fill = t.sky;
            e.fill2 = t.sky2;
            e.raycast = false;
        });
        self.el(p, bottom(0.0, 360.0, 0.0, 0.0), Kind::Panel, "Ground (preview)", |e| {
            e.fill = t.ground;
            e.raycast = false;
        });
        sky
    }

    /// Dark veil over the whole page (behind modals and pause menus).
    pub fn dim(&mut self, a: u8) -> N {
        let p = self.page();
        self.el(p, full(), Kind::Panel, "Dim", |e| {
            e.fill = [0, 0, 0, a];
        })
    }

    /// Gradient placeholder for artwork (portraits, key art, item pictures, map tiles…).
    pub fn art(&mut self, p: N, pos: Pos, name: &str) -> N {
        let (a, b) = (self.t.accent2, self.t.bg2);
        self.art_c(p, pos, name, a, b)
    }

    pub fn art_c(&mut self, p: N, pos: Pos, name: &str, c1: Rgba, c2: Rgba) -> N {
        let r = self.t.radius * self.s;
        self.el(p, pos, Kind::Image, name, |e| {
            e.fill = c1;
            e.fill2 = c2;
            e.radius = r;
            e.raycast = false;
        })
    }

    // ---- boxes ------------------------------------------------------------------------------------------------

    /// Main panel of the style (border in the line colour).
    pub fn panel(&mut self, p: N, pos: Pos, name: &str) -> N {
        let t = self.t;
        let s = self.s;
        self.el(p, pos, Kind::Panel, name, |e| {
            e.fill = t.panel;
            e.radius = t.radius * s;
            e.stroke = t.line;
            e.stroke_w = t.stroke * s;
        })
    }

    /// Raised card inside a panel (`panel2`, thin border).
    pub fn card(&mut self, p: N, pos: Pos, name: &str) -> N {
        let t = self.t;
        let s = self.s;
        self.el(p, pos, Kind::Panel, name, |e| {
            e.fill = t.panel2;
            e.radius = t.radius * s;
            e.stroke = alpha(t.line, 140);
            e.stroke_w = (t.stroke * 0.5).max(1.0) * s;
        })
    }

    /// Selected card: like [`G::card`] with an accent border.
    pub fn card_on(&mut self, p: N, pos: Pos, name: &str) -> N {
        let t = self.t;
        let s = self.s;
        self.el(p, pos, Kind::Panel, name, |e| {
            e.fill = t.panel2;
            e.radius = t.radius * s;
            e.stroke = t.accent;
            e.stroke_w = (t.stroke + 2.0) * s;
            e.shadow.on = true;
        })
    }

    /// Plain coloured box (`radius` in reference pixels). Good for dividers, highlights, map shapes.
    pub fn rect(&mut self, p: N, pos: Pos, name: &str, color: Rgba, radius: f32) -> N {
        let s = self.s;
        self.el(p, pos, Kind::Panel, name, |e| {
            e.fill = color;
            e.radius = radius * s;
            e.raycast = false;
        })
    }

    /// Plain circle / ellipse.
    pub fn circle(&mut self, p: N, pos: Pos, name: &str, color: Rgba) -> N {
        self.el(p, pos, Kind::Ellipse, name, |e| {
            e.fill = color;
            e.raycast = false;
        })
    }

    /// Circle with a short label inside (icon stand-in): accent fill, `on_accent` text. Use ASCII glyphs.
    pub fn icon(&mut self, p: N, pos: Pos, name: &str, glyph: &str) -> N {
        let t = self.t;
        let s = self.s;
        let size = pos.ys[1].min(pos.xs[1]).max(20.0);
        self.el(p, pos, Kind::Ellipse, name, |e| {
            e.fill = t.accent;
            e.text = glyph.to_string();
            e.text_color = t.on_accent;
            e.bold = true;
            e.font_size = (size * 0.45 * s).max(8.0);
            e.raycast = false;
        })
    }

    /// Small rounded tag with text (accent fill).
    pub fn badge(&mut self, p: N, pos: Pos, name: &str, text: &str) -> N {
        let t = self.t;
        let s = self.s;
        let h = pos.ys[1];
        self.el(p, pos, Kind::Panel, name, |e| {
            e.fill = t.accent;
            e.radius = (h * 0.5).min(t.radius.max(6.0)) * s;
            e.text = text.to_string();
            e.text_color = t.on_accent;
            e.bold = true;
            e.font_size = ((h * 0.5).clamp(14.0, 30.0) * s).max(8.0);
            e.raycast = false;
        })
    }

    /// Item / ability slot (empty square with a border).
    pub fn slot(&mut self, p: N, pos: Pos, name: &str) -> N {
        let t = self.t;
        let s = self.s;
        self.el(p, pos, Kind::Panel, name, |e| {
            e.fill = shade(t.panel2, 0.8);
            e.radius = (t.radius * 0.6) * s;
            e.stroke = t.line;
            e.stroke_w = (t.stroke * 0.75).max(1.0) * s;
        })
    }

    /// Controller / keyboard key prompt («A», «E», «Esc»…): a small key cap with the label.
    pub fn key(&mut self, p: N, pos: Pos, name: &str, label: &str) -> N {
        let t = self.t;
        let s = self.s;
        let h = pos.ys[1];
        self.el(p, pos, Kind::Panel, name, |e| {
            e.fill = t.text;
            e.radius = (h * 0.25) * s;
            e.text = label.to_string();
            e.text_color = t.panel2;
            e.text_color[3] = 255;
            e.bold = true;
            e.font_size = ((h * 0.5).max(14.0) * s).max(8.0);
            e.raycast = false;
        })
    }

    // ---- text -------------------------------------------------------------------------------------------------

    /// Text with every option: `size` in reference pixels, `align` LEFT / CENTER / RIGHT.
    pub fn txt(&mut self, p: N, pos: Pos, name: &str, text: &str, size: f32, color: Rgba, align: u8, bold: bool) -> N {
        let s = self.s;
        self.el(p, pos, Kind::Text, name, |e| {
            e.text = text.to_string();
            e.font_size = (size * s).max(8.0);
            e.text_color = color;
            e.align = align;
            e.bold = bold;
        })
    }
    /// Body text on a panel, left aligned.
    pub fn label(&mut self, p: N, pos: Pos, name: &str, text: &str, size: f32) -> N {
        let c = self.t.text;
        self.txt(p, pos, name, text, size, c, LEFT, false)
    }
    /// Secondary text on a panel, left aligned.
    pub fn note(&mut self, p: N, pos: Pos, name: &str, text: &str, size: f32) -> N {
        let c = self.t.muted;
        self.txt(p, pos, name, text, size, c, LEFT, false)
    }
    /// Bold centred heading on a panel.
    pub fn heading(&mut self, p: N, pos: Pos, name: &str, text: &str, size: f32) -> N {
        let c = self.t.text;
        self.txt(p, pos, name, text, size, c, CENTER, true)
    }
    /// Bold left-aligned heading on a panel.
    pub fn heading_l(&mut self, p: N, pos: Pos, name: &str, text: &str, size: f32) -> N {
        let c = self.t.text;
        self.txt(p, pos, name, text, size, c, LEFT, true)
    }
    /// Bold right-aligned value (numbers, prices, scores) on a panel.
    pub fn value(&mut self, p: N, pos: Pos, name: &str, text: &str, size: f32) -> N {
        let c = self.t.text;
        self.txt(p, pos, name, text, size, c, RIGHT, true)
    }
    /// Text straight on the page background (`bg_text`), centred and bold: big titles, logos.
    pub fn bg_title(&mut self, p: N, pos: Pos, name: &str, text: &str, size: f32) -> N {
        let c = self.t.bg_text;
        self.txt(p, pos, name, text, size, c, CENTER, true)
    }
    /// Text straight on the page background, not bold; `align` LEFT / CENTER / RIGHT.
    pub fn bg_label(&mut self, p: N, pos: Pos, name: &str, text: &str, size: f32, align: u8) -> N {
        let c = self.t.bg_text;
        self.txt(p, pos, name, text, size, c, align, false)
    }

    // ---- controls ---------------------------------------------------------------------------------------------

    /// Primary button (accent fill).
    pub fn button(&mut self, p: N, pos: Pos, name: &str, label: &str) -> N {
        let t = self.t;
        let s = self.s;
        let h = pos.ys[1];
        self.el(p, pos, Kind::Button, name, |e| {
            e.text = label.to_string();
            e.fill = t.accent;
            e.text_color = t.on_accent;
            e.radius = t.radius.min(h * 0.5) * s;
            e.font_size = ((h * 0.36).clamp(18.0, 40.0) * s).max(8.0);
        })
    }

    /// Secondary button (card colour, border, normal text).
    pub fn button2(&mut self, p: N, pos: Pos, name: &str, label: &str) -> N {
        let t = self.t;
        let s = self.s;
        let h = pos.ys[1];
        self.el(p, pos, Kind::Button, name, |e| {
            e.text = label.to_string();
            e.fill = t.panel2;
            e.text_color = t.text;
            e.stroke = t.line;
            e.stroke_w = (t.stroke * 0.75).max(1.0) * s;
            e.radius = t.radius.min(h * 0.5) * s;
            e.font_size = ((h * 0.36).clamp(18.0, 40.0) * s).max(8.0);
        })
    }

    /// Menu entry: a left-aligned flat button (for vertical menus on dark art). `on` marks the focused one.
    pub fn menu_item(&mut self, p: N, pos: Pos, name: &str, label: &str, on: bool) -> N {
        let t = self.t;
        let s = self.s;
        let h = pos.ys[1];
        self.el(p, pos, Kind::Button, name, |e| {
            e.text = label.to_string();
            e.align = LEFT;
            e.fill = if on { t.accent } else { t.panel };
            e.text_color = if on { t.on_accent } else { t.text };
            e.radius = t.radius.min(h * 0.5) * s;
            e.font_size = ((h * 0.4).clamp(18.0, 40.0) * s).max(8.0);
        })
    }

    /// Read-only bar (health, progress, XP…): `value` 0–1, `color` fill.
    pub fn bar(&mut self, p: N, pos: Pos, name: &str, value: f32, color: Rgba) -> N {
        let track = shade(self.t.panel2, 0.6);
        self.el(p, pos, Kind::Slider, name, |e| {
            e.fill = color;
            e.fill2 = [track[0], track[1], track[2], 255];
            e.value = value.clamp(0.0, 1.0);
            e.interactable = false;
            e.raycast = false;
        })
    }

    /// Slider the player can drag (volume, sensitivity…).
    pub fn slider(&mut self, p: N, pos: Pos, name: &str, value: f32) -> N {
        let t = self.t;
        self.el(p, pos, Kind::Slider, name, |e| {
            e.fill = t.accent;
            e.fill2 = shade(t.panel2, 0.6);
            e.fill2[3] = 255;
            e.value = value.clamp(0.0, 1.0);
        })
    }

    /// Check box with its label.
    pub fn toggle(&mut self, p: N, pos: Pos, name: &str, label: &str, on: bool) -> N {
        let t = self.t;
        let s = self.s;
        let h = pos.ys[1];
        self.el(p, pos, Kind::Toggle, name, |e| {
            e.text = label.to_string();
            e.fill = t.accent;
            e.text_color = t.text;
            e.value = if on { 1.0 } else { 0.0 };
            e.font_size = ((h * 0.55).clamp(16.0, 34.0) * s).max(8.0);
        })
    }

    /// Drop-down list (the first option is the shown one).
    pub fn dropdown(&mut self, p: N, pos: Pos, name: &str, options: &[&str]) -> N {
        let t = self.t;
        let s = self.s;
        let h = pos.ys[1];
        self.el(p, pos, Kind::Dropdown, name, |e| {
            e.options = options.iter().map(|o| o.to_string()).collect();
            e.text = options.first().map(|o| o.to_string()).unwrap_or_default();
            e.value = 0.0;
            e.fill = t.panel2;
            e.text_color = t.text;
            e.stroke = t.line;
            e.stroke_w = (t.stroke * 0.75).max(1.0) * s;
            e.radius = (t.radius * 0.6) * s;
            e.font_size = ((h * 0.42).clamp(16.0, 32.0) * s).max(8.0);
        })
    }

    /// Text box with a placeholder.
    pub fn input(&mut self, p: N, pos: Pos, name: &str, placeholder: &str) -> N {
        let t = self.t;
        let s = self.s;
        let h = pos.ys[1];
        self.el(p, pos, Kind::InputField, name, |e| {
            e.text = placeholder.to_string();
            e.fill = t.panel2;
            e.text_color = t.muted;
            e.stroke = t.line;
            e.stroke_w = (t.stroke * 0.75).max(1.0) * s;
            e.radius = (t.radius * 0.6) * s;
            e.font_size = ((h * 0.4).clamp(16.0, 30.0) * s).max(8.0);
        })
    }

    /// Scroll view (its content is the children you add to it).
    pub fn scroll(&mut self, p: N, pos: Pos, name: &str) -> N {
        let t = self.t;
        self.el(p, pos, Kind::ScrollView, name, |e| {
            e.fill = alpha(t.panel2, 120);
        })
    }

    // ---- layout groups ----------------------------------------------------------------------------------------

    fn group(&mut self, p: N, pos: Pos, name: &str, mode: LayoutMode, spacing: f32, align: u8) -> N {
        let s = self.s;
        self.el(p, pos, Kind::Empty, name, |e| {
            e.layout.mode = mode;
            e.layout.padding = [0.0; 4];
            e.layout.spacing = [spacing * s, spacing * s];
            e.layout.align = align;
        })
    }
    /// Vertical Layout Group, children stacked from the top and centred (add them with `sz(w, h)`).
    pub fn vbox(&mut self, p: N, pos: Pos, name: &str, spacing: f32) -> N {
        self.group(p, pos, name, LayoutMode::Vertical, spacing, 1)
    }
    /// Vertical Layout Group whose children take the full width.
    pub fn vbox_fill(&mut self, p: N, pos: Pos, name: &str, spacing: f32) -> N {
        let n = self.group(p, pos, name, LayoutMode::Vertical, spacing, 1);
        self.edit(n, |e| e.layout.expand_w = true);
        n
    }
    /// Horizontal Layout Group, children in a row from the left, centred vertically.
    pub fn hbox(&mut self, p: N, pos: Pos, name: &str, spacing: f32) -> N {
        self.group(p, pos, name, LayoutMode::Horizontal, spacing, 3)
    }
    /// Horizontal Layout Group, the row centred.
    pub fn hbox_c(&mut self, p: N, pos: Pos, name: &str, spacing: f32) -> N {
        self.group(p, pos, name, LayoutMode::Horizontal, spacing, 4)
    }
    /// Grid Layout Group with cells of `cw` × `ch` reference pixels, filled from the top-left.
    pub fn grid(&mut self, p: N, pos: Pos, name: &str, cw: f32, ch: f32, spacing: f32) -> N {
        let s = self.s;
        let n = self.group(p, pos, name, LayoutMode::Grid, spacing, 0);
        self.edit(n, |e| e.layout.cell = [cw * s, ch * s]);
        n
    }
    /// Inner padding of a layout group (left, right, top, bottom), reference pixels.
    pub fn pad(&mut self, n: N, l: f32, r: f32, t: f32, b: f32) {
        let s = self.s;
        self.edit(n, |e| e.layout.padding = [l * s, r * s, t * s, b * s]);
    }
}

#[cfg(test)]
mod tests;
