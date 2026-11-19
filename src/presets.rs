//! Ready-made buttons, HUD pieces and small components for the Elements tab, built from the existing element kinds.
//! Each preset is one element or a small tree (children keep their parent link), made once and kept in `presets()`
//! together with a thumbnail document. Click / drag inserts the whole tree like a library element.

use crate::model::{Doc, Element, Kind, LayoutMode, Rgba};
use std::sync::LazyLock;

pub struct Preset {
    pub section: &'static str,
    pub label: &'static str,
    /// The elements (ids 1.., parents inside the list, absolute page coordinates; the first one is the root).
    pub items: Vec<Element>,
    /// Small document used to paint the tile.
    pub thumb: Doc,
}

/// Section titles (English; translated when drawn).
pub const SECTIONS: &[&str] = &["Buttons", "HUD", "More components"];

/// Every English text this module shows through `i18n::t` besides the preset labels (checked by a test against all languages).
pub const UI_STRINGS: &[&str] = &["Buttons", "HUD", "More components", "Backgrounds", "Frames", "Icons", "Pixel art", "Gradients", "All", "Built-in images are copied to your library when you use them", "Tip: turn on Pixel art in the page settings so the sprites stay crisp.", "Could not copy the pack image: {}", "Pack image added: {}", "No images match the search"];

type Build = fn(&mut Doc);

const DEFS: &[(&str, &str, Build)] = &[
    // ---- buttons
    ("Buttons", "Primary button", b_primary),
    ("Buttons", "Secondary button", b_secondary),
    ("Buttons", "Danger button", b_danger),
    ("Buttons", "Success button", b_success),
    ("Buttons", "Ghost button", b_ghost),
    ("Buttons", "Icon button", b_icon),
    ("Buttons", "Toggle button", b_toggle),
    ("Buttons", "Tabs", b_tabs),
    ("Buttons", "Small pill button", b_pill),
    ("Buttons", "Square icon button", b_square),
    ("Buttons", "Link button", b_link),
    ("Buttons", "Floating action button", b_fab),
    // ---- HUD
    ("HUD", "Health bar (classic)", h_health_classic),
    ("HUD", "Health bar (segments)", h_health_segments),
    ("HUD", "Health orb", h_orb),
    ("HUD", "Mana bar with label", h_mana),
    ("HUD", "Stamina bar", h_stamina),
    ("HUD", "XP bar with level", h_xp),
    ("HUD", "Boss bar", h_boss),
    ("HUD", "Minimap frame", h_minimap),
    ("HUD", "Quest tracker", h_quest),
    ("HUD", "Skill slot (cooldown)", h_skill),
    ("HUD", "Hotbar (5 slots)", h_hotbar),
    ("HUD", "Currency counter", h_currency),
    ("HUD", "Nameplate", h_nameplate),
    ("HUD", "Toast notification", h_toast),
    ("HUD", "Tooltip", h_tooltip),
    ("HUD", "Dialog with portrait", h_dialog),
    ("HUD", "Inventory grid 4x3", h_inventory),
    ("HUD", "Compass strip", h_compass),
    // ---- more components
    ("More components", "Avatar with ring", c_avatar),
    ("More components", "Pill badge", c_pill_badge),
    ("More components", "Ribbon banner", c_ribbon),
    ("More components", "Divider", c_divider),
    ("More components", "Divider with label", c_divider_label),
    ("More components", "Progress ring", c_ring),
    ("More components", "Switch", c_switch),
    ("More components", "Chips", c_chips),
    ("More components", "Media card", c_card),
    ("More components", "List item", c_list_item),
    ("More components", "Search field", c_search),
    ("More components", "Stat row", c_stat),
];

/// All the presets, built once.
pub fn presets() -> &'static [Preset] {
    static ALL: LazyLock<Vec<Preset>> = LazyLock::new(|| DEFS.iter().map(|(section, label, build)| make(section, label, *build)).collect());
    &ALL
}

/// Labels of every preset (for the translation test).
pub fn labels() -> impl Iterator<Item = &'static str> {
    DEFS.iter().map(|d| d.1).chain(TEXT_HUD_LABELS.iter().copied())
}

/// Labels of the extra tiles of the Text tab («For buttons and HUD»).
pub const TEXT_HUD_LABELS: &[&str] = &["Level up", "Heal", "Critical hit", "Objective", "Ammo counter", "Wave counter", "FPS counter", "Distance", "Key prompt"];

/// More text styles for the Text tab: single elements.
pub fn text_hud() -> Vec<(&'static str, Element)> {
    let mk = |label: &'static str, s: &str, size: f32, color: Rgba, w: f32, f: &dyn Fn(&mut Element)| {
        let mut e = Element::new(0, Kind::Text);
        e.name = label.to_string();
        e.text = s.to_string();
        e.font_size = size;
        e.bold = true;
        e.text_color = color;
        e.w = w;
        e.h = size * 1.3;
        f(&mut e);
        (label, e)
    };
    let mut key = Element::new(0, Kind::Panel);
    key.name = "Key prompt".into();
    key.text = "E  Interact".into();
    key.font_size = 30.0;
    key.bold = true;
    key.text_color = WHITE;
    key.fill = DARK_GLASS;
    key.stroke = [255, 255, 255, 80];
    key.stroke_w = 2.0;
    key.radius = 12.0;
    key.w = 300.0;
    key.h = 68.0;
    vec![
        mk("Level up", "LEVEL UP!", 64.0, GOLD, 420.0, &|e| e.shadow.on = true),
        mk("Heal", "+45", 56.0, [110, 235, 140, 255], 200.0, &|_| {}),
        mk("Critical hit", "CRIT 256!", 60.0, [255, 150, 50, 255], 360.0, &|e| {
            e.italic = true;
            e.rot = -6.0;
        }),
        mk("Objective", "Reach the bridge", 34.0, WHITE, 420.0, &|e| e.bold = false),
        mk("Ammo counter", "24 / 90", 48.0, WHITE, 220.0, &|e| e.align = 2),
        mk("Wave counter", "WAVE 3", 52.0, [255, 120, 120, 255], 260.0, &|_| {}),
        mk("FPS counter", "60 FPS", 28.0, [140, 255, 160, 255], 160.0, &|e| e.bold = false),
        mk("Distance", "128 m", 40.0, [150, 200, 255, 255], 200.0, &|_| {}),
        ("Key prompt", key),
    ]
}

fn make(section: &'static str, label: &'static str, build: Build) -> Preset {
    let mut d = Doc::empty(4000.0, 4000.0);
    build(&mut d);
    d.apply_layouts();
    d.sanitize();
    let items = d.elements.clone();
    let b = crate::model::bounds_of(items.iter());
    let pad = 24.0;
    let mut t = Doc::empty((b[2] - b[0]) + pad * 2.0, (b[3] - b[1]) + pad * 2.0);
    t.background = [34, 36, 50, 255];
    t.elements = items
        .iter()
        .map(|e| {
            let mut c = e.clone();
            c.x += pad - b[0];
            c.y += pad - b[1];
            c
        })
        .collect();
    t.next_id = items.iter().map(|e| e.id).max().unwrap_or(0) + 1;
    Preset { section, label, items, thumb: t }
}

// ---------------------------------------------------------------- helpers

const WHITE: Rgba = [255, 255, 255, 255];
const PURPLE: Rgba = [124, 92, 255, 255];
const RED: Rgba = [200, 45, 70, 255];
const GREEN: Rgba = [28, 130, 78, 255];
const GOLD: Rgba = [255, 205, 70, 255];
const BLUE: Rgba = [70, 130, 255, 255];
const DARK: Rgba = [26, 28, 44, 255];
const DARK_GLASS: Rgba = [14, 16, 30, 215];
const TRACK: Rgba = [58, 60, 76, 255];
const MUTED_TEXT: Rgba = [190, 196, 222, 255];

fn put(d: &mut Doc, parent: Option<u64>, kind: Kind, name: &str, rect: [f32; 4], f: impl FnOnce(&mut Element)) -> u64 {
    let mut e = Element::new(0, kind);
    e.name = name.to_string();
    e.x = rect[0];
    e.y = rect[1];
    e.w = rect[2];
    e.h = rect[3];
    e.parent = parent;
    f(&mut e);
    d.add(e)
}

fn text(d: &mut Doc, parent: Option<u64>, name: &str, rect: [f32; 4], s: &str, size: f32, color: Rgba, align: u8, bold: bool) -> u64 {
    put(d, parent, Kind::Text, name, rect, |e| {
        e.text = s.to_string();
        e.font_size = size;
        e.text_color = color;
        e.align = align;
        e.bold = bold;
    })
}

fn bar(d: &mut Doc, parent: Option<u64>, name: &str, rect: [f32; 4], fill: Rgba, value: f32) -> u64 {
    put(d, parent, Kind::Slider, name, rect, |e| {
        e.fill = fill;
        e.fill2 = TRACK;
        e.value = value;
        e.interactable = false;
    })
}

fn panel(d: &mut Doc, parent: Option<u64>, name: &str, rect: [f32; 4], fill: Rgba, radius: f32) -> u64 {
    put(d, parent, Kind::Panel, name, rect, |e| {
        e.fill = fill;
        e.radius = radius;
        e.raycast = false;
    })
}

fn button(d: &mut Doc, name: &str, label: &str, rect: [f32; 4], fill: Rgba, color: Rgba, radius: f32) -> u64 {
    put(d, None, Kind::Button, name, rect, |e| {
        e.text = label.to_string();
        e.fill = fill;
        e.text_color = color;
        e.radius = radius;
        e.bold = true;
    })
}

// ---------------------------------------------------------------- buttons

fn b_primary(d: &mut Doc) {
    let id = button(d, "Primary button", "Continue", [0.0, 0.0, 360.0, 88.0], PURPLE, WHITE, 18.0);
    if let Some(e) = d.get_mut(id) {
        e.shadow.on = true;
    }
}
fn b_secondary(d: &mut Doc) {
    let id = button(d, "Secondary button", "Cancel", [0.0, 0.0, 360.0, 88.0], [66, 72, 100, 255], WHITE, 18.0);
    if let Some(e) = d.get_mut(id) {
        e.stroke = [120, 128, 170, 255];
        e.stroke_w = 2.0;
    }
}
fn b_danger(d: &mut Doc) {
    button(d, "Danger button", "Delete", [0.0, 0.0, 360.0, 88.0], RED, WHITE, 18.0);
}
fn b_success(d: &mut Doc) {
    button(d, "Success button", "Accept", [0.0, 0.0, 360.0, 88.0], GREEN, WHITE, 18.0);
}
fn b_ghost(d: &mut Doc) {
    let id = button(d, "Ghost button", "Skip", [0.0, 0.0, 300.0, 80.0], [255, 255, 255, 28], [215, 205, 255, 255], 16.0);
    if let Some(e) = d.get_mut(id) {
        e.bold = false;
    }
}
fn b_icon(d: &mut Doc) {
    let id = button(d, "Icon button", "+", [0.0, 0.0, 96.0, 96.0], PURPLE, WHITE, 28.0);
    if let Some(e) = d.get_mut(id) {
        e.font_size = 56.0;
    }
}
fn b_toggle(d: &mut Doc) {
    let id = button(d, "Toggle button", "ON", [0.0, 0.0, 200.0, 80.0], GREEN, WHITE, 40.0);
    if let Some(e) = d.get_mut(id) {
        e.stroke = [190, 255, 215, 255];
        e.stroke_w = 4.0;
        e.font_size = 34.0;
    }
}
fn b_tabs(d: &mut Doc) {
    let row = put(d, None, Kind::Empty, "Tabs", [0.0, 0.0, 660.0, 80.0], |e| {
        e.layout.mode = LayoutMode::Horizontal;
        e.layout.spacing = [8.0, 0.0];
        e.layout.padding = [0.0; 4];
        e.layout.align = 4;
    });
    for (i, l) in ["Items", "Skills", "Map"].iter().enumerate() {
        put(d, Some(row), Kind::Button, &format!("Tab {}", l), [0.0, 0.0, 212.0, 80.0], |e| {
            e.text = l.to_string();
            e.font_size = 30.0;
            e.bold = i == 0;
            e.radius = 14.0;
            e.fill = if i == 0 { PURPLE } else { [48, 52, 76, 255] };
            e.text_color = WHITE;
        });
    }
}
fn b_pill(d: &mut Doc) {
    let id = button(d, "Small pill button", "Follow", [0.0, 0.0, 200.0, 60.0], [255, 140, 60, 255], [30, 16, 8, 255], 30.0);
    if let Some(e) = d.get_mut(id) {
        e.font_size = 26.0;
    }
}
fn b_square(d: &mut Doc) {
    let id = button(d, "Square icon button", "?", [0.0, 0.0, 88.0, 88.0], [52, 56, 82, 255], WHITE, 10.0);
    if let Some(e) = d.get_mut(id) {
        e.stroke = [130, 138, 185, 255];
        e.stroke_w = 3.0;
        e.font_size = 46.0;
    }
}
fn b_link(d: &mut Doc) {
    let id = button(d, "Link button", "Learn more", [0.0, 0.0, 260.0, 64.0], [0, 0, 0, 0], [140, 185, 255, 255], 0.0);
    if let Some(e) = d.get_mut(id) {
        e.underline = true;
        e.bold = false;
        e.font_size = 30.0;
    }
}
fn b_fab(d: &mut Doc) {
    let id = button(d, "Floating action button", "+", [0.0, 0.0, 120.0, 120.0], [255, 140, 60, 255], [30, 16, 8, 255], 60.0);
    if let Some(e) = d.get_mut(id) {
        e.font_size = 64.0;
        e.shadow.on = true;
        e.shadow.offset = [0.0, -8.0];
    }
}

// ---------------------------------------------------------------- HUD

fn frame(d: &mut Doc, name: &str, rect: [f32; 4]) -> u64 {
    put(d, None, Kind::Panel, name, rect, |e| {
        e.fill = DARK_GLASS;
        e.stroke = [255, 255, 255, 60];
        e.stroke_w = 2.0;
        e.radius = 14.0;
        e.raycast = false;
    })
}

fn h_health_classic(d: &mut Doc) {
    let f = frame(d, "Health bar (classic)", [0.0, 0.0, 520.0, 64.0]);
    bar(d, Some(f), "Health bar", [10.0, 12.0, 500.0, 40.0], [225, 60, 80, 255], 0.72);
    text(d, Some(f), "Health value", [20.0, 12.0, 480.0, 40.0], "72 / 100", 26.0, WHITE, 1, true);
}
fn h_health_segments(d: &mut Doc) {
    let f = put(d, None, Kind::Empty, "Health bar (segments)", [0.0, 0.0, 520.0, 44.0], |e| {
        e.layout.mode = LayoutMode::Horizontal;
        e.layout.spacing = [6.0, 0.0];
        e.layout.padding = [0.0; 4];
        e.layout.align = 4;
        e.layout.expand_w = true;
        e.layout.expand_h = true;
    });
    for i in 0..10 {
        put(d, Some(f), Kind::Panel, &format!("Segment {}", i + 1), [0.0, 0.0, 46.0, 44.0], |e| {
            e.fill = if i < 7 { [225, 60, 80, 255] } else { [62, 40, 52, 255] };
            e.radius = 6.0;
            e.raycast = false;
        });
    }
}
fn h_orb(d: &mut Doc) {
    let o = put(d, None, Kind::Ellipse, "Health orb", [0.0, 0.0, 180.0, 180.0], |e| {
        e.fill = [40, 20, 30, 255];
        e.stroke = GOLD;
        e.stroke_w = 8.0;
        e.raycast = false;
    });
    put(d, Some(o), Kind::Ellipse, "Orb liquid", [14.0, 14.0, 152.0, 152.0], |e| {
        e.fill = [220, 40, 60, 255];
        e.raycast = false;
    });
    text(d, Some(o), "Orb value", [30.0, 62.0, 120.0, 56.0], "72", 48.0, WHITE, 1, true);
}
fn h_mana(d: &mut Doc) {
    let f = frame(d, "Mana bar with label", [0.0, 0.0, 520.0, 64.0]);
    bar(d, Some(f), "Mana bar", [10.0, 12.0, 500.0, 40.0], BLUE, 0.45);
    text(d, Some(f), "Mana value", [20.0, 12.0, 480.0, 40.0], "MANA 45 / 100", 26.0, WHITE, 1, true);
}
fn h_stamina(d: &mut Doc) {
    let f = put(d, None, Kind::Empty, "Stamina bar", [0.0, 0.0, 420.0, 40.0], |_| {});
    text(d, Some(f), "Stamina label", [0.0, 0.0, 90.0, 40.0], "STA", 24.0, [210, 245, 190, 255], 0, true);
    bar(d, Some(f), "Stamina fill", [96.0, 8.0, 324.0, 24.0], [120, 215, 90, 255], 0.8);
}
fn h_xp(d: &mut Doc) {
    let f = frame(d, "XP bar with level", [0.0, 0.0, 640.0, 70.0]);
    bar(d, Some(f), "XP bar", [90.0, 22.0, 540.0, 26.0], [110, 215, 120, 255], 0.62);
    let l = put(d, Some(f), Kind::Ellipse, "Level badge", [8.0, 5.0, 60.0, 60.0], |e| {
        e.fill = GOLD;
        e.text = "12".into();
        e.font_size = 30.0;
        e.bold = true;
        e.text_color = [30, 20, 5, 255];
        e.raycast = false;
    });
    let _ = l;
}
fn h_boss(d: &mut Doc) {
    let f = put(d, None, Kind::Panel, "Boss bar", [0.0, 0.0, 1000.0, 96.0], |e| {
        e.fill = DARK_GLASS;
        e.stroke = GOLD;
        e.stroke_w = 3.0;
        e.radius = 10.0;
        e.raycast = false;
    });
    text(d, Some(f), "Boss name", [20.0, 6.0, 960.0, 38.0], "ANCIENT DRAGON", 30.0, [255, 225, 150, 255], 1, true);
    bar(d, Some(f), "Boss health", [20.0, 52.0, 960.0, 30.0], [200, 40, 50, 255], 0.85);
}
fn h_minimap(d: &mut Doc) {
    let f = put(d, None, Kind::Panel, "Minimap frame", [0.0, 0.0, 300.0, 300.0], |e| {
        e.fill = [20, 24, 36, 255];
        e.stroke = [220, 190, 120, 255];
        e.stroke_w = 6.0;
        e.radius = 150.0;
        e.raycast = false;
    });
    put(d, Some(f), Kind::Image, "Map", [22.0, 22.0, 256.0, 256.0], |e| {
        e.fill = [70, 140, 90, 255];
        e.fill2 = [30, 70, 60, 255];
        e.radius = 128.0;
        e.raycast = false;
    });
    put(d, Some(f), Kind::Ellipse, "Player marker", [138.0, 138.0, 24.0, 24.0], |e| {
        e.fill = [255, 240, 120, 255];
        e.stroke = [20, 20, 30, 255];
        e.stroke_w = 3.0;
        e.raycast = false;
    });
    text(d, Some(f), "North", [130.0, 8.0, 40.0, 30.0], "N", 24.0, WHITE, 1, true);
}
fn h_quest(d: &mut Doc) {
    let f = put(d, None, Kind::Panel, "Quest tracker", [0.0, 0.0, 440.0, 250.0], |e| {
        e.fill = DARK_GLASS;
        e.stroke = [255, 255, 255, 50];
        e.stroke_w = 2.0;
        e.radius = 14.0;
        e.raycast = false;
    });
    text(d, Some(f), "Quest title", [20.0, 14.0, 400.0, 40.0], "Lost relic", 32.0, GOLD, 0, true);
    for (i, (s, done)) in [("Find the key", true), ("Open the vault", false), ("Return to the guard", false)].iter().enumerate() {
        let y = 66.0 + i as f32 * 56.0;
        put(d, Some(f), Kind::Panel, &format!("Check {}", i + 1), [22.0, y + 8.0, 28.0, 28.0], |e| {
            e.fill = if *done { [90, 210, 120, 255] } else { [0, 0, 0, 0] };
            e.stroke = [200, 205, 225, 255];
            e.stroke_w = 3.0;
            e.radius = 6.0;
            e.raycast = false;
        });
        text(d, Some(f), &format!("Objective {}", i + 1), [64.0, y, 360.0, 44.0], s, 28.0, if *done { [150, 160, 180, 255] } else { WHITE }, 0, false);
    }
}
fn h_skill(d: &mut Doc) {
    let s = put(d, None, Kind::Panel, "Skill slot", [0.0, 0.0, 132.0, 132.0], |e| {
        e.fill = [60, 66, 100, 255];
        e.stroke = [170, 176, 220, 255];
        e.stroke_w = 3.0;
        e.radius = 18.0;
        e.raycast = false;
    });
    put(d, Some(s), Kind::Ellipse, "Skill icon", [26.0, 26.0, 80.0, 80.0], |e| {
        e.fill = [255, 150, 60, 255];
        e.raycast = false;
    });
    put(d, Some(s), Kind::Panel, "Cooldown shade", [0.0, 0.0, 132.0, 84.0], |e| {
        e.fill = [0, 0, 0, 170];
        e.radius = 18.0;
        e.raycast = false;
    });
    text(d, Some(s), "Cooldown", [26.0, 16.0, 80.0, 52.0], "3", 44.0, WHITE, 1, true);
    put(d, Some(s), Kind::Panel, "Key hint", [88.0, 92.0, 38.0, 34.0], |e| {
        e.fill = DARK;
        e.text = "Q".into();
        e.font_size = 24.0;
        e.bold = true;
        e.text_color = WHITE;
        e.radius = 8.0;
        e.raycast = false;
    });
}
fn h_hotbar(d: &mut Doc) {
    let bar_ = put(d, None, Kind::Panel, "Hotbar (5 slots)", [0.0, 0.0, 700.0, 140.0], |e| {
        e.fill = DARK_GLASS;
        e.radius = 18.0;
        e.stroke = [255, 255, 255, 50];
        e.stroke_w = 2.0;
        e.raycast = false;
        e.layout.mode = LayoutMode::Horizontal;
        e.layout.spacing = [12.0, 0.0];
        e.layout.padding = [16.0, 16.0, 12.0, 12.0];
        e.layout.align = 4;
    });
    for i in 0..5 {
        let s = put(d, Some(bar_), Kind::Panel, &format!("Slot {}", i + 1), [0.0, 0.0, 116.0, 116.0], |e| {
            e.fill = [58, 62, 92, 255];
            e.stroke = if i == 0 { GOLD } else { [110, 116, 160, 255] };
            e.stroke_w = if i == 0 { 4.0 } else { 2.0 };
            e.radius = 12.0;
            e.raycast = false;
        });
        text(d, Some(s), &format!("Key {}", i + 1), [6.0, 4.0, 30.0, 30.0], &(i + 1).to_string(), 22.0, WHITE, 1, true);
    }
}
fn h_currency(d: &mut Doc) {
    let f = put(d, None, Kind::Panel, "Currency counter", [0.0, 0.0, 300.0, 76.0], |e| {
        e.fill = DARK_GLASS;
        e.radius = 38.0;
        e.stroke = [255, 255, 255, 50];
        e.stroke_w = 2.0;
        e.raycast = false;
    });
    put(d, Some(f), Kind::Ellipse, "Coin", [8.0, 8.0, 60.0, 60.0], |e| {
        e.fill = GOLD;
        e.stroke = [200, 140, 20, 255];
        e.stroke_w = 5.0;
        e.raycast = false;
    });
    text(d, Some(f), "Amount", [78.0, 8.0, 205.0, 60.0], "1 250", 40.0, GOLD, 2, true);
}
fn h_nameplate(d: &mut Doc) {
    let f = put(d, None, Kind::Panel, "Nameplate", [0.0, 0.0, 360.0, 96.0], |e| {
        e.fill = DARK_GLASS;
        e.radius = 12.0;
        e.raycast = false;
    });
    text(d, Some(f), "Player name", [16.0, 6.0, 230.0, 40.0], "Aria", 32.0, WHITE, 0, true);
    text(d, Some(f), "Player level", [230.0, 10.0, 118.0, 32.0], "Lv 12", 24.0, GOLD, 2, true);
    bar(d, Some(f), "Nameplate health", [16.0, 58.0, 328.0, 22.0], [225, 60, 80, 255], 0.8);
}
fn h_toast(d: &mut Doc) {
    let f = put(d, None, Kind::Panel, "Toast notification", [0.0, 0.0, 560.0, 100.0], |e| {
        e.fill = [30, 34, 52, 245];
        e.radius = 14.0;
        e.shadow.on = true;
        e.raycast = false;
    });
    put(d, Some(f), Kind::Panel, "Toast accent", [0.0, 0.0, 14.0, 100.0], |e| {
        e.fill = [90, 210, 120, 255];
        e.radius = 7.0;
        e.raycast = false;
    });
    text(d, Some(f), "Toast title", [34.0, 10.0, 510.0, 40.0], "Achievement unlocked", 30.0, WHITE, 0, true);
    text(d, Some(f), "Toast message", [34.0, 52.0, 510.0, 36.0], "Defeat 10 enemies without damage", 24.0, MUTED_TEXT, 0, false);
}
fn h_tooltip(d: &mut Doc) {
    let f = put(d, None, Kind::Panel, "Tooltip", [0.0, 0.0, 380.0, 170.0], |e| {
        e.fill = [18, 20, 34, 245];
        e.stroke = [150, 130, 255, 255];
        e.stroke_w = 3.0;
        e.radius = 12.0;
        e.raycast = false;
    });
    text(d, Some(f), "Tooltip title", [18.0, 12.0, 344.0, 40.0], "Iron Sword", 32.0, [255, 225, 150, 255], 0, true);
    text(d, Some(f), "Tooltip stat", [18.0, 58.0, 344.0, 36.0], "+12 attack", 26.0, [130, 230, 150, 255], 0, false);
    text(d, Some(f), "Tooltip text", [18.0, 98.0, 344.0, 56.0], "A reliable blade for any hero.", 24.0, MUTED_TEXT, 0, false);
}
fn h_dialog(d: &mut Doc) {
    let f = put(d, None, Kind::Panel, "Dialog with portrait", [0.0, 0.0, 900.0, 220.0], |e| {
        e.fill = DARK_GLASS;
        e.stroke = [255, 255, 255, 70];
        e.stroke_w = 2.0;
        e.radius = 16.0;
        e.raycast = false;
    });
    put(d, Some(f), Kind::Image, "Portrait", [20.0, 20.0, 180.0, 180.0], |e| {
        e.fill = [124, 92, 255, 255];
        e.fill2 = [30, 20, 70, 255];
        e.radius = 14.0;
        e.raycast = false;
    });
    text(d, Some(f), "Speaker", [224.0, 18.0, 640.0, 44.0], "Elder", 34.0, GOLD, 0, true);
    text(d, Some(f), "Line", [224.0, 68.0, 640.0, 100.0], "The old road is dangerous at night. Take this lantern with you.", 28.0, WHITE, 0, false);
    text(d, Some(f), "Continue hint", [680.0, 176.0, 200.0, 32.0], "Press E", 22.0, MUTED_TEXT, 2, false);
}
fn h_inventory(d: &mut Doc) {
    let g = put(d, None, Kind::Panel, "Inventory grid 4x3", [0.0, 0.0, 508.0, 386.0], |e| {
        e.fill = DARK_GLASS;
        e.radius = 16.0;
        e.stroke = [255, 255, 255, 50];
        e.stroke_w = 2.0;
        e.raycast = false;
        e.layout.mode = LayoutMode::Grid;
        e.layout.cell = [110.0, 110.0];
        e.layout.spacing = [12.0, 12.0];
        e.layout.padding = [16.0, 16.0, 16.0, 16.0];
        e.layout.align = 4;
    });
    for i in 0..12 {
        put(d, Some(g), Kind::Panel, &format!("Inventory slot {}", i + 1), [0.0, 0.0, 110.0, 110.0], |e| {
            e.fill = [58, 62, 92, 255];
            e.stroke = [110, 116, 160, 255];
            e.stroke_w = 2.0;
            e.radius = 12.0;
            e.raycast = false;
        });
    }
}
fn h_compass(d: &mut Doc) {
    let f = put(d, None, Kind::Panel, "Compass strip", [0.0, 0.0, 700.0, 60.0], |e| {
        e.fill = DARK_GLASS;
        e.radius = 30.0;
        e.raycast = false;
    });
    for (i, l) in ["W", "NW", "N", "NE", "E"].iter().enumerate() {
        text(d, Some(f), &format!("Direction {}", l), [20.0 + i as f32 * 132.0, 8.0, 100.0, 44.0], l, if *l == "N" { 34.0 } else { 26.0 }, if *l == "N" { GOLD } else { MUTED_TEXT }, 1, *l == "N");
    }
    put(d, Some(f), Kind::Panel, "Center tick", [346.0, 52.0, 8.0, 8.0], |e| {
        e.fill = GOLD;
        e.radius = 4.0;
        e.raycast = false;
    });
}

// ---------------------------------------------------------------- more components

fn c_avatar(d: &mut Doc) {
    let r = put(d, None, Kind::Ellipse, "Avatar with ring", [0.0, 0.0, 150.0, 150.0], |e| {
        e.fill = GOLD;
        e.raycast = false;
    });
    put(d, Some(r), Kind::Ellipse, "Avatar", [10.0, 10.0, 130.0, 130.0], |e| {
        e.fill = PURPLE;
        e.text = "A".into();
        e.font_size = 64.0;
        e.bold = true;
        e.text_color = WHITE;
        e.raycast = false;
    });
}
fn c_pill_badge(d: &mut Doc) {
    put(d, None, Kind::Panel, "Pill badge", [0.0, 0.0, 150.0, 48.0], |e| {
        e.fill = PURPLE;
        e.radius = 24.0;
        e.text = "BETA".into();
        e.font_size = 24.0;
        e.bold = true;
        e.text_color = WHITE;
        e.raycast = false;
    });
}
fn c_ribbon(d: &mut Doc) {
    let r = put(d, None, Kind::Empty, "Ribbon banner", [0.0, 0.0, 620.0, 120.0], |_| {});
    put(d, Some(r), Kind::Panel, "Ribbon tail left", [0.0, 26.0, 110.0, 80.0], |e| {
        e.fill = [150, 25, 45, 255];
        e.radius = 4.0;
        e.raycast = false;
    });
    put(d, Some(r), Kind::Panel, "Ribbon tail right", [510.0, 26.0, 110.0, 80.0], |e| {
        e.fill = [150, 25, 45, 255];
        e.radius = 4.0;
        e.raycast = false;
    });
    put(d, Some(r), Kind::Panel, "Ribbon body", [50.0, 0.0, 520.0, 96.0], |e| {
        e.fill = RED;
        e.radius = 6.0;
        e.text = "SPECIAL OFFER".into();
        e.font_size = 42.0;
        e.bold = true;
        e.text_color = WHITE;
        e.shadow.on = true;
        e.raycast = false;
    });
}
fn c_divider(d: &mut Doc) {
    panel(d, None, "Divider", [0.0, 0.0, 600.0, 4.0], [255, 255, 255, 90], 2.0);
}
fn c_divider_label(d: &mut Doc) {
    let r = put(d, None, Kind::Empty, "Divider with label", [0.0, 0.0, 600.0, 40.0], |_| {});
    panel(d, Some(r), "Line left", [0.0, 18.0, 250.0, 3.0], [255, 255, 255, 90], 1.0);
    text(d, Some(r), "Label", [250.0, 0.0, 100.0, 40.0], "OR", 26.0, MUTED_TEXT, 1, true);
    panel(d, Some(r), "Line right", [350.0, 18.0, 250.0, 3.0], [255, 255, 255, 90], 1.0);
}
fn c_ring(d: &mut Doc) {
    let r = put(d, None, Kind::Ellipse, "Progress ring", [0.0, 0.0, 200.0, 200.0], |e| {
        e.fill = [0, 0, 0, 0];
        e.stroke = PURPLE;
        e.stroke_w = 16.0;
        e.raycast = false;
    });
    text(d, Some(r), "Progress value", [30.0, 62.0, 140.0, 76.0], "72%", 52.0, WHITE, 1, true);
}
fn c_switch(d: &mut Doc) {
    let s = panel(d, None, "Switch", [0.0, 0.0, 150.0, 76.0], [60, 190, 110, 255], 38.0);
    put(d, Some(s), Kind::Ellipse, "Switch knob", [80.0, 8.0, 60.0, 60.0], |e| {
        e.fill = WHITE;
        e.shadow.on = true;
        e.shadow.offset = [0.0, -3.0];
        e.raycast = false;
    });
}
fn c_chips(d: &mut Doc) {
    let row = put(d, None, Kind::Empty, "Chips", [0.0, 0.0, 520.0, 56.0], |e| {
        e.layout.mode = LayoutMode::Horizontal;
        e.layout.spacing = [10.0, 0.0];
        e.layout.padding = [0.0; 4];
        e.layout.align = 4;
    });
    for (i, l) in ["Action", "RPG", "Co-op"].iter().enumerate() {
        put(d, Some(row), Kind::Panel, &format!("Chip {}", l), [0.0, 0.0, if i == 0 { 160.0 } else { 120.0 }, 56.0], |e| {
            e.fill = [66, 72, 108, 255];
            e.stroke = [150, 158, 210, 255];
            e.stroke_w = 2.0;
            e.radius = 28.0;
            e.text = l.to_string();
            e.font_size = 26.0;
            e.text_color = WHITE;
            e.raycast = false;
        });
    }
}
fn c_card(d: &mut Doc) {
    let c = put(d, None, Kind::Panel, "Media card", [0.0, 0.0, 380.0, 470.0], |e| {
        e.fill = [38, 42, 64, 255];
        e.radius = 22.0;
        e.shadow.on = true;
        e.raycast = false;
    });
    put(d, Some(c), Kind::Image, "Card image", [0.0, 0.0, 380.0, 220.0], |e| {
        e.fill = [255, 126, 179, 255];
        e.fill2 = [124, 92, 255, 255];
        e.radius = 22.0;
        e.raycast = false;
    });
    text(d, Some(c), "Card title", [22.0, 236.0, 336.0, 44.0], "Starter pack", 34.0, WHITE, 0, true);
    text(d, Some(c), "Card text", [22.0, 284.0, 336.0, 66.0], "Everything you need to begin your adventure.", 24.0, MUTED_TEXT, 0, false);
    put(d, Some(c), Kind::Button, "Card button", [22.0, 376.0, 336.0, 72.0], |e| {
        e.text = "Get it".into();
        e.fill = PURPLE;
        e.text_color = WHITE;
        e.bold = true;
        e.radius = 16.0;
    });
}
fn c_list_item(d: &mut Doc) {
    let r = put(d, None, Kind::Panel, "List item", [0.0, 0.0, 680.0, 112.0], |e| {
        e.fill = [40, 44, 68, 255];
        e.radius = 16.0;
        e.raycast = false;
    });
    put(d, Some(r), Kind::Ellipse, "Item icon", [18.0, 18.0, 76.0, 76.0], |e| {
        e.fill = [255, 150, 60, 255];
        e.raycast = false;
    });
    text(d, Some(r), "Item title", [112.0, 16.0, 480.0, 44.0], "Daily reward", 32.0, WHITE, 0, true);
    text(d, Some(r), "Item subtitle", [112.0, 60.0, 480.0, 36.0], "Come back tomorrow for more", 24.0, MUTED_TEXT, 0, false);
    text(d, Some(r), "Item arrow", [610.0, 28.0, 56.0, 56.0], ">", 44.0, MUTED_TEXT, 1, true);
}
fn c_search(d: &mut Doc) {
    put(d, None, Kind::InputField, "Search field", [0.0, 0.0, 520.0, 76.0], |e| {
        e.text = "Search…".into();
        e.radius = 38.0;
        e.text_color = [96, 100, 118, 255];
    });
}
fn c_stat(d: &mut Doc) {
    let r = put(d, None, Kind::Panel, "Stat row", [0.0, 0.0, 520.0, 64.0], |e| {
        e.fill = [40, 44, 68, 255];
        e.radius = 10.0;
        e.raycast = false;
    });
    text(d, Some(r), "Stat name", [20.0, 10.0, 260.0, 44.0], "Strength", 28.0, MUTED_TEXT, 0, false);
    text(d, Some(r), "Stat value", [300.0, 10.0, 200.0, 44.0], "24", 32.0, WHITE, 2, true);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_preset_is_a_valid_tree() {
        let all = presets();
        assert_eq!(all.len(), DEFS.len());
        assert!(all.iter().filter(|p| p.section == "Buttons").count() >= 12);
        assert!(all.iter().filter(|p| p.section == "HUD").count() >= 14);
        assert!(all.iter().filter(|p| p.section == "More components").count() >= 10);
        let mut names = std::collections::HashSet::new();
        for p in all {
            assert!(SECTIONS.contains(&p.section));
            assert!(names.insert(p.label), "duplicate label {}", p.label);
            assert!(!p.items.is_empty(), "{} is empty", p.label);
            let ids: std::collections::HashSet<u64> = p.items.iter().map(|e| e.id).collect();
            assert_eq!(ids.len(), p.items.len(), "{}: duplicate ids", p.label);
            assert_eq!(p.items.iter().filter(|e| e.parent.is_none()).count(), 1, "{}: needs exactly one root", p.label);
            for e in &p.items {
                assert!(e.parent.map(|q| ids.contains(&q)).unwrap_or(true), "{}: missing parent", p.label);
                assert!(e.w > 0.0 && e.h > 0.0 && e.x.is_finite() && e.y.is_finite(), "{}: {} has a bad rect", p.label, e.name);
            }
            // sanitizing changes nothing
            let mut d = Doc::empty(1920.0, 1080.0);
            d.elements = p.items.clone();
            d.next_id = ids.iter().max().unwrap() + 1;
            let before = d.elements.clone();
            d.sanitize();
            assert_eq!(d.elements, before, "{}: sanitize changed the preset", p.label);
        }
    }

    /// A design with every preset (each one in its own slot) has no review errors and exports to a Unity package.
    #[test]
    fn all_presets_pass_review_and_export() {
        let mut doc = Doc::empty(1920.0, 1080.0);
        doc.background = [24, 26, 40, 255];
        for p in presets() {
            // each preset alone in a design of its own: the review is per design
            let mut d = Doc::empty(1920.0, 1080.0);
            d.background = [24, 26, 40, 255];
            let b = crate::model::bounds_of(p.items.iter());
            let (dx, dy) = (960.0 - (b[0] + b[2]) / 2.0, 540.0 - (b[1] + b[3]) / 2.0);
            let items: Vec<Element> = p
                .items
                .iter()
                .map(|e| {
                    let mut c = e.clone();
                    c.x += dx;
                    c.y += dy;
                    c
                })
                .collect();
            d.insert_items_map(&items, None);
            d.sanitize();
            d.apply_layouts();
            let errors: Vec<String> = crate::review::review(&d).into_iter().filter(|q| q.level == crate::review::Level::Error).map(|q| format!("{}: {} {:?}", q.element, q.template, q.data)).collect();
            assert!(errors.is_empty(), "{}: {:?}", p.label, errors);
            doc.insert_items_map(&items, None);
        }
        doc.sanitize();
        let dir = std::env::temp_dir().join(format!("lienzo-presets-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("Assets")).unwrap();
        let files = crate::export::write_unity_package(&doc, &dir).unwrap();
        assert!(!files.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn text_presets_are_valid() {
        let v = text_hud();
        assert_eq!(v.len(), TEXT_HUD_LABELS.len());
        for (l, e) in v {
            assert!(TEXT_HUD_LABELS.contains(&l));
            assert!(e.w > 0.0 && e.h > 0.0 && !e.text.is_empty());
        }
    }

    #[test]
    fn labels_and_strings_are_translated_in_every_language() {
        let mut missing = Vec::new();
        let strings: Vec<&str> = labels().chain(SECTIONS.iter().copied()).chain(UI_STRINGS.iter().copied()).collect();
        for (code, _, tsv) in &crate::i18n::LANGUAGES[1..] {
            let keys: std::collections::HashSet<&str> = tsv.lines().map(|l| l.trim_end_matches('\r')).filter_map(|l| l.split_once('\t')).map(|(k, _)| k).collect();
            for s in &strings {
                if !keys.contains(s) {
                    missing.push(format!("{}: {}", code, s));
                }
            }
        }
        assert!(missing.is_empty(), "untranslated: {:?}", missing);
    }
}
