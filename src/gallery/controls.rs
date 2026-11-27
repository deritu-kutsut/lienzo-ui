//! Gallery: overlay menus (wheels, targeting, placement, emotes). Controls and misc are in controls_b.rs.

use super::*;

pub fn entries() -> Vec<Entry> {
    use Style::*;
    vec![
        Entry { ty: "Item & ability wheel", style: Fantasy, build: wheel_radial },
        Entry { ty: "Item & ability wheel", style: SciFi, build: wheel_cross },
        Entry { ty: "Item & ability wheel", style: Cartoon, build: wheel_grid },
        Entry { ty: "Selection & targeting", style: Military, build: select_rts },
        Entry { ty: "Selection & targeting", style: Neon, build: select_lock },
        Entry { ty: "Selection & targeting", style: Elegant, build: select_commands },
        Entry { ty: "Object placement", style: Nature, build: place_build_bar },
        Entry { ty: "Object placement", style: Minimal, build: place_ghost },
        Entry { ty: "Object placement", style: Retro, build: place_catalog },
        Entry { ty: "Chat shortcuts & emotes", style: Pastel, build: emote_wheel },
        Entry { ty: "Chat shortcuts & emotes", style: Sports, build: ping_wheel },
        Entry { ty: "Chat shortcuts & emotes", style: Horror, build: quick_chat },
    ]
}

/// Point on a circle of radius `r` (reference px) around the centre, starting at the top and going clockwise.
fn around(i: usize, n: usize, r: f32) -> (f32, f32) {
    let a = (i as f32 / n as f32) * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
    (r * a.cos(), r * a.sin())
}

/// Ring outline (an ellipse with no fill and a border).
fn ring(g: &mut G, p: N, pos: Pos, name: &str, color: Rgba, width: f32) -> N {
    let s = g.s;
    g.el(p, pos, Kind::Ellipse, name, |e| {
        e.fill = [0, 0, 0, 0];
        e.stroke = color;
        e.stroke_w = width * s;
        e.raycast = false;
    })
}

/// Key prompt and its label in a small plate.
fn hint(g: &mut G, p: N, pos: Pos, key: &str, label: &str) {
    let plate = g.panel(p, pos, &format!("{} hint", label));
    g.key(plate, cl(16.0, 0.0, 50.0, 50.0), &format!("{} key", label), key);
    g.label(plate, cl(80.0, 0.0, 260.0, 50.0), &format!("{} label", label), label, 26.0);
}

// ---- Item & ability wheel ----------------------------------------------------------------------------------------

/// Eight item slots in a ring around the centre, the highlighted item described in the middle.
fn wheel_radial(g: &mut G) {
    let p = g.page();
    g.scene();
    g.dim(140);
    g.circle(p, cc(0.0, 0.0, 760.0, 760.0), "Wheel backdrop", alpha(g.t.panel, 200));
    let items = ["Potion", "Bomb", "Rope", "Torch", "Key", "Map", "Bow", "Herb"];
    for (i, it) in items.iter().enumerate() {
        let (x, y) = around(i, items.len(), 290.0);
        let slot = if i == 1 { g.card_on(p, cc(x, y, 150.0, 150.0), &format!("{} slot", it)) } else { g.card(p, cc(x, y, 150.0, 150.0), &format!("{} slot", it)) };
        g.art(slot, tc(0.0, 14.0, 80.0, 80.0), &format!("{} icon", it));
        g.heading(slot, bottom(8.0, 40.0, 6.0, 6.0), &format!("{} name", it), it, 24.0);
    }
    let core = g.panel(p, cc(0.0, 0.0, 320.0, 320.0), "Wheel centre");
    let s = g.s;
    g.edit(core, |e| e.radius = 160.0 * s);
    g.heading(core, top(70.0, 60.0, 20.0, 20.0), "Selected name", "Bomb", 44.0);
    g.txt(core, top(140.0, 44.0, 30.0, 30.0), "Selected count", "x 3", 32.0, g.t.accent, CENTER, true);
    g.note(core, top(200.0, 60.0, 40.0, 40.0), "Selected text", "Breaks cracked walls", 24.0);
    hint(g, p, bc(-200.0, 40.0, 360.0, 76.0), "LB", "Hold to open");
    hint(g, p, bc(200.0, 40.0, 360.0, 76.0), "A", "Equip");
}

/// Quick-select cross: four slots around a d-pad in the lower left, the equipped item beside it.
fn wheel_cross(g: &mut G) {
    let p = g.page();
    g.scene();
    let pad = g.panel(p, bl(60.0, 60.0, 460.0, 460.0), "Quick select");
    for (i, (it, x, y)) in [("Medkit", 0.0, -150.0), ("Grenade", 150.0, 0.0), ("Shield", 0.0, 150.0), ("Scanner", -150.0, 0.0)].iter().enumerate() {
        let slot = if i == 1 { g.card_on(pad, cc(*x, *y, 130.0, 130.0), &format!("{} slot", it)) } else { g.card(pad, cc(*x, *y, 130.0, 130.0), &format!("{} slot", it)) };
        g.art(slot, tc(0.0, 12.0, 70.0, 60.0), &format!("{} icon", it));
        g.heading(slot, bottom(8.0, 36.0, 4.0, 4.0), &format!("{} name", it), it, 22.0);
    }
    g.key(pad, cc(0.0, 0.0, 90.0, 90.0), "D-pad", "+");
    let eq = g.panel(p, bl(560.0, 60.0, 520.0, 200.0), "Equipped panel");
    g.note(eq, tl(30.0, 20.0, 460.0, 40.0), "Equipped label", "EQUIPPED", 24.0);
    g.heading_l(eq, tl(30.0, 64.0, 300.0, 56.0), "Equipped name", "Grenade", 42.0);
    g.value(eq, tr(30.0, 64.0, 140.0, 56.0), "Equipped count", "x 2", 40.0);
    g.bar(eq, bottom(30.0, 20.0, 30.0, 30.0), "Cooldown bar", 0.7, g.t.accent);
    let vit = g.panel(p, tl(60.0, 60.0, 520.0, 130.0), "Vitals panel");
    g.note(vit, tl(24.0, 14.0, 200.0, 36.0), "Hull label", "HULL", 24.0);
    g.bar(vit, tl(24.0, 56.0, 470.0, 22.0), "Hull bar", 0.8, g.t.good);
    g.bar(vit, tl(24.0, 92.0, 470.0, 16.0), "Energy bar", 0.55, g.t.accent);
}

/// Pause-style grid overlay: categories on top, items in a 4 x 2 grid and the description at the bottom.
fn wheel_grid(g: &mut G) {
    let p = g.page();
    g.scene();
    g.dim(150);
    let win = g.panel(p, cc(0.0, 0.0, 1300.0, 820.0), "Item menu");
    let tabs = g.hbox_c(win, top(30.0, 80.0, 40.0, 40.0), "Categories", 20.0);
    for (i, c) in ["Tools", "Snacks", "Toys", "Keys"].iter().enumerate() {
        if i == 1 {
            g.button(tabs, sz(240.0, 76.0), &format!("{} tab", c), c);
        } else {
            g.button2(tabs, sz(240.0, 76.0), &format!("{} tab", c), c);
        }
    }
    let grid = g.grid(win, top(140.0, 440.0, 70.0, 70.0), "Item grid", 270.0, 210.0, 20.0);
    for (i, it) in ["Cookie", "Apple", "Jelly", "Cake", "Milk", "Candy", "Pie", "Juice"].iter().enumerate() {
        let c = if i == 2 { g.card_on(grid, sz(270.0, 210.0), &format!("{} card", it)) } else { g.card(grid, sz(270.0, 210.0), &format!("{} card", it)) };
        g.art(c, tc(0.0, 16.0, 120.0, 110.0), &format!("{} picture", it));
        g.heading(c, bottom(14.0, 50.0, 10.0, 10.0), &format!("{} name", it), it, 30.0);
    }
    let info = g.card(win, bottom(30.0, 150.0, 40.0, 40.0), "Description");
    g.heading_l(info, tl(30.0, 20.0, 700.0, 50.0), "Item title", "Jelly - restores 2 hearts", 34.0);
    g.note(info, tl(30.0, 80.0, 800.0, 44.0), "Item text", "Wobbly, sweet and very bouncy.", 28.0);
    g.button(info, cr(30.0, 0.0, 240.0, 80.0), "Use button", "Use");
}

// ---- Selection & targeting ---------------------------------------------------------------------------------------

/// RTS command bar: minimap, the selected squad and a grid of orders.
fn select_rts(g: &mut G) {
    let p = g.page();
    g.scene();
    for (i, (x, y)) in [(-180.0, -40.0), (-60.0, 10.0), (70.0, -30.0)].iter().enumerate() {
        ring(g, p, cc(*x, *y, 110.0, 60.0), &format!("Selection ring {}", i + 1), g.t.good, 4.0);
    }
    let bar = g.panel(p, bottom(0.0, 300.0, 0.0, 0.0), "Command bar");
    let map = g.card(bar, cl(24.0, 0.0, 260.0, 260.0), "Minimap");
    g.art(map, fill(10.0, 10.0, 10.0, 10.0), "Minimap terrain");
    let unit = g.card(bar, cl(310.0, 0.0, 520.0, 260.0), "Selected unit");
    g.art(unit, cl(20.0, 0.0, 180.0, 220.0), "Unit portrait");
    g.heading_l(unit, tl(220.0, 20.0, 280.0, 50.0), "Unit name", "Rifle squad", 34.0);
    g.note(unit, tl(220.0, 74.0, 280.0, 36.0), "Unit count", "3 selected", 24.0);
    g.bar(unit, tl(220.0, 130.0, 270.0, 20.0), "Health bar", 0.8, g.t.good);
    g.note(unit, tl(220.0, 160.0, 280.0, 36.0), "Unit stats", "ATK 12  ARM 4", 24.0);
    let squad = g.grid(bar, cl(860.0, 0.0, 560.0, 260.0), "Squad", 130.0, 120.0, 10.0);
    for i in 0..8 {
        let c = if i < 3 { g.card_on(squad, sz(130.0, 120.0), &format!("Member {}", i + 1)) } else { g.card(squad, sz(130.0, 120.0), &format!("Member {}", i + 1)) };
        g.art(c, fill(10.0, 10.0, 10.0, 30.0), &format!("Member {} icon", i + 1));
        g.bar(c, bottom(8.0, 12.0, 10.0, 10.0), &format!("Member {} health", i + 1), 0.4 + i as f32 * 0.07, g.t.good);
    }
    let orders = g.grid(bar, cr(24.0, 0.0, 440.0, 260.0), "Orders", 140.0, 80.0, 10.0);
    for (k, o) in [("Q", "Move"), ("W", "Stop"), ("E", "Hold"), ("A", "Attack"), ("S", "Patrol"), ("D", "Build")] {
        g.button2(orders, sz(140.0, 80.0), &format!("{} order", o), &format!("{} {}", k, o));
    }
    let res = g.panel(p, tr(24.0, 24.0, 520.0, 70.0), "Resources");
    g.label(res, cl(24.0, 0.0, 480.0, 44.0), "Resources text", "Supply 24/60   Steel 1 240   Fuel 380", 26.0);
}

/// Lock-on reticle around the target, with a card that shows its name, level, health and weak point.
fn select_lock(g: &mut G) {
    let p = g.page();
    g.scene();
    g.art(p, cc(0.0, 0.0, 220.0, 300.0), "Target");
    ring(g, p, cc(0.0, 0.0, 380.0, 380.0), "Lock ring", g.t.accent, 6.0);
    ring(g, p, cc(0.0, 0.0, 440.0, 440.0), "Outer ring", alpha(g.t.accent2, 160), 2.0);
    for (i, (x, y)) in [(0.0, -230.0), (230.0, 0.0), (0.0, 230.0), (-230.0, 0.0)].iter().enumerate() {
        g.rect(p, cc(*x, *y, 16.0, 16.0), &format!("Lock tick {}", i + 1), g.t.accent, 0.0);
    }
    let card = g.panel(p, cc(470.0, -120.0, 460.0, 250.0), "Target card");
    g.badge(card, tl(24.0, 20.0, 160.0, 44.0), "Lock tag", "LOCKED");
    g.heading_l(card, tl(24.0, 76.0, 410.0, 54.0), "Target name", "Riot Drone MK-II", 36.0);
    g.note(card, tl(24.0, 134.0, 410.0, 36.0), "Target level", "Level 14 - Shielded", 24.0);
    g.bar(card, bottom(30.0, 22.0, 24.0, 24.0), "Target health", 0.65, g.t.bad);
    let weak = g.panel(p, cc(470.0, 120.0, 460.0, 110.0), "Weak point card");
    g.icon(weak, cl(20.0, 0.0, 70.0, 70.0), "Weak icon", "!");
    g.label(weak, cl(110.0, 0.0, 330.0, 50.0), "Weak text", "Weak point: rear battery", 26.0);
    hint(g, p, bc(-200.0, 50.0, 360.0, 76.0), "R3", "Switch target");
    hint(g, p, bc(200.0, 50.0, 360.0, 76.0), "L2", "Release lock");
}

/// Command card on the right: the selected hero and a list of abilities with costs.
fn select_commands(g: &mut G) {
    let p = g.page();
    g.scene();
    ring(g, p, cc(-200.0, 120.0, 220.0, 90.0), "Selection ring", g.t.accent, 4.0);
    let card = g.panel(p, right(40.0, 560.0, 40.0, 40.0), "Command card");
    g.art(card, top(30.0, 260.0, 30.0, 30.0), "Hero portrait");
    g.heading_l(card, tl(30.0, 310.0, 500.0, 56.0), "Hero name", "Lady Seraphine", 40.0);
    g.note(card, tl(30.0, 370.0, 500.0, 36.0), "Hero role", "Duelist - Level 22", 26.0);
    let list = g.vbox_fill(card, tl(30.0, 430.0, 500.0, 520.0), "Abilities", 12.0);
    for (i, (k, a, c)) in [("1", "Riposte", "20 SP"), ("2", "Silver Step", "15 SP"), ("3", "Duel", "40 SP"), ("4", "Inspire", "30 SP"), ("5", "Retreat", "Free")].iter().enumerate() {
        let row = if i == 0 { g.card_on(list, sz(500.0, 92.0), &format!("{} row", a)) } else { g.card(list, sz(500.0, 92.0), &format!("{} row", a)) };
        g.key(row, cl(18.0, 0.0, 54.0, 54.0), &format!("{} key", a), k);
        g.label(row, cl(90.0, 0.0, 260.0, 50.0), &format!("{} name", a), a, 28.0);
        g.value(row, cr(18.0, 0.0, 130.0, 50.0), &format!("{} cost", a), c, 26.0);
    }
    let tip = g.panel(p, tl(40.0, 40.0, 620.0, 110.0), "Order tip");
    g.label(tip, cl(30.0, 0.0, 560.0, 50.0), "Order text", "Choose an ability, then a target", 30.0);
}

// ---- Object placement --------------------------------------------------------------------------------------------

/// Building bar at the bottom and a placement grid over the ground (green = valid, red = blocked).
fn place_build_bar(g: &mut G) {
    let p = g.page();
    g.scene();
    let field = g.el(p, cc(0.0, 60.0, 720.0, 360.0), Kind::Empty, "Placement grid", |_| {});
    for r in 0..3 {
        for c in 0..6 {
            let blocked = r == 0 && c >= 4;
            let col = if blocked { alpha(g.t.bad, 150) } else { alpha(g.t.good, 120) };
            g.rect(field, tl(c as f32 * 120.0 + 4.0, r as f32 * 120.0 + 4.0, 112.0, 112.0), &format!("Cell {}-{}", r + 1, c + 1), col, 8.0);
        }
    }
    g.art(field, tl(124.0, 124.0, 232.0, 232.0), "Ghost building");
    let bar = g.panel(p, bottom(30.0, 250.0, 200.0, 200.0), "Build bar");
    let cats = g.vbox_fill(bar, cl(24.0, 0.0, 220.0, 210.0), "Categories", 8.0);
    for (i, c) in ["Farm", "Homes", "Decor"].iter().enumerate() {
        g.menu_item(cats, sz(220.0, 62.0), &format!("{} tab", c), c, i == 0);
    }
    let items = g.hbox(bar, fill(270.0, 20.0, 24.0, 20.0), "Buildings", 16.0);
    for (it, cost) in [("Barn", "120"), ("Coop", "60"), ("Silo", "90"), ("Well", "40"), ("Windmill", "200")] {
        let c = g.card(items, sz(220.0, 210.0), &format!("{} card", it));
        g.art(c, top(12.0, 110.0, 12.0, 12.0), &format!("{} picture", it));
        g.heading(c, top(128.0, 40.0, 6.0, 6.0), &format!("{} name", it), it, 28.0);
        g.txt(c, bottom(10.0, 36.0, 6.0, 6.0), &format!("{} cost", it), &format!("{} wood", cost), 24.0, g.t.muted, CENTER, false);
    }
    hint(g, p, tr(40.0, 40.0, 360.0, 76.0), "R", "Rotate");
}

/// Free placement: the ghost of the object with its footprint, a status line and the key prompts.
fn place_ghost(g: &mut G) {
    let p = g.page();
    g.scene();
    g.circle(p, cc(0.0, 140.0, 520.0, 160.0), "Footprint", alpha(g.t.good, 110));
    g.art(p, cc(0.0, 0.0, 260.0, 300.0), "Ghost object");
    let status = g.panel(p, tc(0.0, 40.0, 700.0, 110.0), "Status plate");
    g.heading(status, top(14.0, 50.0, 20.0, 20.0), "Object name", "Lamp post", 36.0);
    g.txt(status, bottom(12.0, 36.0, 20.0, 20.0), "Valid text", "Valid spot - snapping to grid", 24.0, g.t.text, CENTER, true);
    let keys = g.vbox_fill(p, cr(40.0, 0.0, 380.0, 420.0), "Key list", 12.0);
    for (k, l) in [("LMB", "Place"), ("Q", "Rotate left"), ("E", "Rotate right"), ("G", "Snap on/off"), ("Esc", "Cancel")] {
        let row = g.panel(keys, sz(380.0, 72.0), &format!("{} row", l));
        g.key(row, cl(16.0, 0.0, 90.0, 50.0), &format!("{} key", l), k);
        g.label(row, cl(124.0, 0.0, 240.0, 50.0), &format!("{} label", l), l, 26.0);
    }
    let cnt = g.panel(p, bl(40.0, 40.0, 420.0, 90.0), "Budget plate");
    g.label(cnt, cl(24.0, 0.0, 240.0, 50.0), "Budget label", "Budget", 28.0);
    g.value(cnt, cr(24.0, 0.0, 140.0, 50.0), "Budget value", "38/50", 30.0);
}

/// Side catalog with categories and prices, the selected object and its cost on top.
fn place_catalog(g: &mut G) {
    let p = g.page();
    g.scene();
    g.art(p, cc(-260.0, 40.0, 240.0, 240.0), "Ghost object");
    let side = g.panel(p, left(0.0, 560.0, 0.0, 0.0), "Catalog");
    g.heading_l(side, tl(30.0, 30.0, 500.0, 60.0), "Catalog title", "BUILD MODE", 44.0);
    g.input(side, tl(30.0, 110.0, 500.0, 64.0), "Search field", "Search...");
    let grid = g.grid(side, tl(30.0, 200.0, 500.0, 720.0), "Catalog grid", 240.0, 160.0, 20.0);
    for (i, (it, c)) in [("Chair", 10), ("Table", 25), ("Lamp", 15), ("Plant", 8), ("Shelf", 30), ("Rug", 12), ("Bed", 60), ("Clock", 20)].iter().enumerate() {
        let card = if i == 2 { g.card_on(grid, sz(240.0, 160.0), &format!("{} card", it)) } else { g.card(grid, sz(240.0, 160.0), &format!("{} card", it)) };
        g.art(card, top(10.0, 90.0, 10.0, 10.0), &format!("{} picture", it));
        g.label(card, bl(14.0, 14.0, 130.0, 40.0), &format!("{} name", it), it, 24.0);
        g.value(card, br(14.0, 14.0, 80.0, 40.0), &format!("{} price", it), &format!("{}G", c), 24.0);
    }
    let money = g.panel(p, tr(40.0, 40.0, 360.0, 90.0), "Money plate");
    g.label(money, cl(24.0, 0.0, 160.0, 50.0), "Money label", "GOLD", 28.0);
    g.value(money, cr(24.0, 0.0, 160.0, 50.0), "Money value", "245G", 32.0);
    hint(g, p, br(40.0, 40.0, 360.0, 76.0), "A", "Place lamp");
}

// ---- Chat shortcuts & emotes -------------------------------------------------------------------------------------

/// Six emotes in a ring around the cursor, the highlighted one named in the middle.
fn emote_wheel(g: &mut G) {
    let p = g.page();
    g.scene();
    g.circle(p, cc(0.0, 0.0, 640.0, 640.0), "Wheel backdrop", alpha(g.t.panel, 210));
    let emotes = ["Wave", "Dance", "Laugh", "Heart", "Cry", "Clap"];
    for (i, em) in emotes.iter().enumerate() {
        let (x, y) = around(i, emotes.len(), 220.0);
        let c = if i == 3 { g.card_on(p, cc(x, y, 150.0, 150.0), &format!("{} emote", em)) } else { g.card(p, cc(x, y, 150.0, 150.0), &format!("{} emote", em)) };
        g.art(c, tc(0.0, 14.0, 80.0, 80.0), &format!("{} picture", em));
        g.heading(c, bottom(8.0, 40.0, 6.0, 6.0), &format!("{} name", em), em, 26.0);
    }
    let core = g.panel(p, cc(0.0, 0.0, 200.0, 200.0), "Wheel centre");
    let s = g.s;
    g.edit(core, |e| e.radius = 100.0 * s);
    g.heading(core, full(), "Selected emote", "Heart", 36.0);
    hint(g, p, bc(0.0, 40.0, 380.0, 76.0), "T", "Release to send");
}

/// Ping wheel with four callouts, plus the ping already placed on the map and in the chat feed.
fn ping_wheel(g: &mut G) {
    let p = g.page();
    g.scene();
    let pings = [("Enemy here", 0.0, -200.0), ("On my way", 200.0, 0.0), ("Need help", 0.0, 200.0), ("Watch out", -200.0, 0.0)];
    g.circle(p, cc(0.0, 0.0, 600.0, 600.0), "Wheel backdrop", alpha(g.t.panel, 220));
    for (i, (l, x, y)) in pings.iter().enumerate() {
        let b = if i == 0 { g.card_on(p, cc(*x, *y, 220.0, 110.0), &format!("{} ping", l)) } else { g.card(p, cc(*x, *y, 220.0, 110.0), &format!("{} ping", l)) };
        g.heading(b, full(), &format!("{} text", l), l, 28.0);
    }
    g.icon(p, cc(0.0, 0.0, 120.0, 120.0), "Ping centre", "!");
    let feed = g.panel(p, bl(40.0, 200.0, 560.0, 220.0), "Chat feed");
    let rows = g.vbox_fill(feed, fill(20.0, 20.0, 20.0, 20.0), "Feed rows", 8.0);
    for (who, what) in [("Rook", "Enemy here (B site)"), ("Mika", "On my way"), ("You", "Need help")] {
        g.label(rows, sz(520.0, 52.0), &format!("{} line", who), &format!("{}: {}", who, what), 26.0);
    }
    let map = g.panel(p, tr(40.0, 40.0, 300.0, 300.0), "Minimap");
    g.art(map, fill(12.0, 12.0, 12.0, 12.0), "Minimap terrain");
    g.icon(map, cc(40.0, -50.0, 50.0, 50.0), "Map ping", "!");
}

/// Numbered quick-chat list on the left, with the recent messages above it.
fn quick_chat(g: &mut G) {
    let p = g.page();
    g.scene();
    let log = g.panel(p, tl(40.0, 200.0, 640.0, 230.0), "Chat log");
    let rows = g.vbox_fill(log, fill(20.0, 20.0, 20.0, 20.0), "Log rows", 6.0);
    for (who, what) in [("Nell", "I heard something upstairs."), ("Ash", "Stay together."), ("You", "Follow me.")] {
        g.label(rows, sz(600.0, 56.0), &format!("{} line", who), &format!("{}: {}", who, what), 26.0);
    }
    let menu = g.panel(p, tl(40.0, 460.0, 640.0, 560.0), "Quick chat");
    g.heading_l(menu, tl(24.0, 20.0, 590.0, 56.0), "Quick chat title", "QUICK CHAT", 36.0);
    let list = g.vbox_fill(menu, tl(24.0, 90.0, 590.0, 450.0), "Phrases", 10.0);
    for (i, ph) in ["Follow me.", "Wait here.", "I need a light!", "Run!", "Found a key.", "Don't open that door."].iter().enumerate() {
        let row = if i == 2 { g.card_on(list, sz(590.0, 66.0), &format!("Phrase {}", i + 1)) } else { g.card(list, sz(590.0, 66.0), &format!("Phrase {}", i + 1)) };
        g.key(row, cl(12.0, 0.0, 46.0, 46.0), &format!("Phrase {} key", i + 1), &format!("{}", i + 1));
        g.label(row, cl(76.0, 0.0, 490.0, 50.0), &format!("Phrase {} text", i + 1), ph, 26.0);
    }
    let sanity = g.panel(p, tr(40.0, 40.0, 420.0, 110.0), "Sanity panel");
    g.note(sanity, tl(24.0, 14.0, 360.0, 36.0), "Sanity label", "SANITY", 24.0);
    g.bar(sanity, bottom(22.0, 22.0, 24.0, 24.0), "Sanity bar", 0.35, g.t.accent);
}
