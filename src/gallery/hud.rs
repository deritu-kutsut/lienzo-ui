//! Readable HUD overlays on preview scenes.

use super::*;

pub fn entries() -> Vec<Entry> {
    use Style::*;
    vec![
        Entry { ty: "Player vitals", style: Fantasy, build: vitals_a },
        Entry { ty: "Player vitals", style: Cartoon, build: vitals_b },
        Entry { ty: "Player vitals", style: Military, build: vitals_c },
        Entry { ty: "Equipped items & abilities", style: Retro, build: equipment_a },
        Entry { ty: "Equipped items & abilities", style: SciFi, build: equipment_b },
        Entry { ty: "Equipped items & abilities", style: Neon, build: equipment_c },
        Entry { ty: "Collection counters", style: Fantasy, build: collection_a },
        Entry { ty: "Collection counters", style: Nature, build: collection_b },
        Entry { ty: "Collection counters", style: Minimal, build: collection_c },
        Entry { ty: "Rank & position", style: Sports, build: rank_a },
        Entry { ty: "Rank & position", style: Military, build: rank_b },
        Entry { ty: "Rank & position", style: Retro, build: rank_c },
        Entry { ty: "Enemy health & damage", style: Horror, build: enemy_a },
        Entry { ty: "Enemy health & damage", style: Cartoon, build: enemy_b },
        Entry { ty: "Enemy health & damage", style: SciFi, build: enemy_c },
        Entry { ty: "Info & notifications", style: Minimal, build: info_a },
        Entry { ty: "Info & notifications", style: Elegant, build: info_b },
        Entry { ty: "Info & notifications", style: Retro, build: info_c },
        Entry { ty: "Area / level name", style: Fantasy, build: area_a },
        Entry { ty: "Area / level name", style: Horror, build: area_b },
        Entry { ty: "Area / level name", style: Elegant, build: area_c },
        Entry { ty: "Loot & XP log", style: Nature, build: loot_a },
        Entry { ty: "Loot & XP log", style: Pastel, build: loot_b },
        Entry { ty: "Loot & XP log", style: Military, build: loot_c },
        Entry { ty: "Scoring & combos", style: Neon, build: scoring_a },
        Entry { ty: "Scoring & combos", style: Sports, build: scoring_b },
        Entry { ty: "Scoring & combos", style: Cartoon, build: scoring_c },
        Entry { ty: "Clock & timer", style: SciFi, build: clock_a },
        Entry { ty: "Clock & timer", style: Sports, build: clock_b },
        Entry { ty: "Clock & timer", style: Nature, build: clock_c },
        Entry { ty: "Objectives & goals", style: Military, build: objectives_a },
        Entry { ty: "Objectives & goals", style: Fantasy, build: objectives_b },
        Entry { ty: "Objectives & goals", style: Minimal, build: objectives_c },
        Entry { ty: "Waypoints & markers", style: SciFi, build: waypoint_a },
        Entry { ty: "Waypoints & markers", style: Neon, build: waypoint_b },
        Entry { ty: "Waypoints & markers", style: Nature, build: waypoint_c },
        Entry { ty: "Minimap", style: Fantasy, build: minimap_a },
        Entry { ty: "Minimap", style: Military, build: minimap_b },
        Entry { ty: "Minimap", style: SciFi, build: minimap_c },
        Entry { ty: "Compass bar", style: Elegant, build: compass_a },
        Entry { ty: "Compass bar", style: Sports, build: compass_b },
        Entry { ty: "Compass bar", style: Pastel, build: compass_c },
    ]
}

// Los textos siempre se ubican dentro de placas del tema.
fn plaque(g: &mut G, pos: Pos, name: &str, title: &str, detail: &str) -> N {
    let n = g.panel(g.page(), pos, name);
    let w = n.r[2] / g.s - 32.0;
    g.heading_l(n, tl(16.0, 12.0, w, 42.0), "Heading", title, 29.0);
    g.note(n, tl(16.0, 58.0, w, 34.0), "Detail", detail, 23.0);
    n
}

fn scene(g: &mut G) {
    g.scene();
    let p = g.page();
    g.art(p, tl(470.0, 500.0, 270.0, 170.0), "Distant structure");
    g.art(p, tl(850.0, 455.0, 210.0, 210.0), "Middle structure");
    g.art(p, tl(1190.0, 525.0, 300.0, 140.0), "Far structure");
}

fn symbol_card(g: &mut G, p: N, pos: Pos, name: &str, glyph: &str) {
    let n = g.card(p, pos, name);
    g.heading(n, full(), "Symbol", glyph, 28.0);
}

fn vitals(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            let n = g.panel(g.page(), tl(30.0, 30.0, 650.0, 235.0), "Portrait vitals");
            g.art(n, tl(16.0, 16.0, 140.0, 200.0), "Hero portrait");
            g.heading_l(n, tl(172.0, 17.0, 440.0, 38.0), "Hero name", "Warden", 28.0);
            for (i, (label, value, color)) in [("Health", "87/100", g.t.bad), ("Mana", "42/60", g.t.accent), ("Stamina", "65/80", g.t.good)].iter().enumerate() {
                let y = 69.0 + i as f32 * 48.0;
                g.label(n, tl(172.0, y, 120.0, 33.0), "Vital label", label, 23.0);
                g.bar(n, tl(300.0, y + 4.0, 220.0, 25.0), "Vital bar", 0.7, *color);
                g.value(n, tl(530.0, y, 100.0, 33.0), "Vital value", value, 23.0);
            }
        }
        1 => {
            let n = g.panel(g.page(), bc(0.0, 30.0, 740.0, 170.0), "Heart strip");
            g.heading_l(n, tl(20.0, 12.0, 300.0, 36.0), "Health heading", "HEALTH", 25.0);
            for i in 0..7 { symbol_card(g, n, tl(20.0 + i as f32 * 100.0, 52.0, 70.0, 70.0), "Heart segment", "+"); }
            g.bar(n, tl(20.0, 139.0, 700.0, 18.0), "Stamina bar", 0.68, g.t.good);
            let p = plaque(g, bc(0.0, 220.0, 300.0, 104.0), "Stamina label plate", "STAMINA", "68 / 100");
            symbol_card(g, p, tr(15.0, 16.0, 45.0, 45.0), "Stamina icon", "+");
        }
        _ => {
            let n = g.panel(g.page(), bl(30.0, 30.0, 505.0, 210.0), "Shooter vitals");
            for (i, (label, number, color)) in [("HEALTH", "087", g.t.bad), ("ARMOR", "042", g.t.accent)].iter().enumerate() {
                let x = 20.0 + i as f32 * 245.0;
                g.note(n, tl(x, 18.0, 215.0, 34.0), "Vital name", label, 24.0);
                g.heading_l(n, tl(x, 58.0, 215.0, 83.0), "Vital number", number, 65.0);
                g.bar(n, tl(x, 163.0, 215.0, 23.0), "Vital meter", 0.67, *color);
            }
        }
    }
}

fn equipment(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            let n = g.panel(g.page(), bc(0.0, 25.0, 870.0, 162.0), "Quick item bar");
            for i in 0..8 {
                let x = 20.0 + i as f32 * 105.0;
                let slot = g.slot(n, tl(x, 20.0, 88.0, 88.0), "Quick item slot");
                g.icon(slot, tl(20.0, 16.0, 48.0, 48.0), "Item icon", "+");
                g.key(n, tl(x + 25.0, 115.0, 38.0, 36.0), "Item shortcut", &format!("{}", i + 1));
            }
        }
        1 => {
            let n = plaque(g, br(30.0, 30.0, 480.0, 275.0), "Weapon panel", "RANGER RIFLE", "Precision loadout");
            g.heading_l(n, tl(18.0, 105.0, 400.0, 78.0), "Ammunition", "24 / 120", 58.0);
            g.bar(n, tl(18.0, 190.0, 440.0, 25.0), "Magazine", 0.2, g.t.accent);
            g.icon(n, tl(18.0, 225.0, 42.0, 42.0), "Grenade icon", "!");
            g.label(n, tl(72.0, 229.0, 260.0, 34.0), "Grenade count", "Grenades  x3", 24.0);
            g.key(n, tr(15.0, 16.0, 42.0, 40.0), "Reload key", "R");
        }
        _ => {
            let n = g.panel(g.page(), bc(0.0, 24.0, 790.0, 172.0), "Ability dock");
            for i in 0..6 {
                let x = 18.0 + i as f32 * 128.0;
                let slot = g.slot(n, tl(x, 27.0, 108.0, 100.0), if i < 4 { "Ability slot" } else { "Item slot" });
                g.icon(slot, tl(27.0, 20.0, 50.0, 50.0), "Ability glyph", "*");
                g.bar(n, tl(x, 136.0, 108.0, 18.0), "Cooldown", if i == 1 { 0.3 } else { 1.0 }, g.t.accent);
                g.key(n, tl(x + 33.0, 1.0, 42.0, 34.0), "Ability key", ["Q", "W", "E", "R", "1", "2"][i]);
            }
        }
    }
}

fn collection(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            let n = g.panel(g.page(), tr(30.0, 30.0, 330.0, 230.0), "Currency wallet");
            for (i, (label, count)) in [("Coins", "1,240"), ("Gems", "38"), ("Keys", "4")].iter().enumerate() {
                let y = 16.0 + i as f32 * 70.0;
                g.icon(n, tl(16.0, y, 50.0, 50.0), "Currency icon", ["C", "G", "K"][i]);
                g.label(n, tl(78.0, y + 8.0, 135.0, 35.0), "Currency label", label, 24.0);
                g.value(n, tr(18.0, y + 8.0, 100.0, 35.0), "Currency value", count, 26.0);
            }
        }
        1 => {
            let n = g.panel(g.page(), cl(30.0, 0.0, 300.0, 340.0), "Gathered materials");
            g.heading_l(n, tl(16.0, 15.0, 265.0, 40.0), "Bag heading", "FIELD BAG", 26.0);
            for (i, (item, qty)) in [("Herbs", "12"), ("Ore", "07"), ("Seeds", "25"), ("Wood", "09")].iter().enumerate() {
                let row = g.card(n, tl(14.0, 66.0 + i as f32 * 66.0, 272.0, 55.0), "Material row");
                g.label(row, tl(12.0, 10.0, 170.0, 34.0), "Material name", item, 24.0);
                g.value(row, tr(12.0, 10.0, 65.0, 34.0), "Material count", qty, 26.0);
            }
        }
        _ => {
            let n = plaque(g, cc(0.0, -220.0, 500.0, 190.0), "Relic progress alert", "RELICS  3 / 10", "Seven still hidden");
            g.bar(n, tl(25.0, 111.0, 450.0, 27.0), "Relic progress", 0.3, g.t.accent);
            for i in 0..3 { g.icon(n, tl(164.0 + i as f32 * 54.0, 150.0, 34.0, 34.0), "Found relic", "*"); }
        }
    }
}

fn rank(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            let n = plaque(g, tr(30.0, 30.0, 320.0, 255.0), "Race standing", "POSITION", "LAP 2 / 3");
            g.heading_l(n, tl(18.0, 105.0, 280.0, 84.0), "Race place", "2 / 8", 68.0);
            g.bar(n, tl(18.0, 210.0, 284.0, 23.0), "Lap progress", 0.64, g.t.accent);
            let gap = plaque(g, tr(30.0, 300.0, 320.0, 104.0), "Racer gap", "NEXT RACER", "+00:03.2");
            g.icon(gap, tr(14.0, 22.0, 43.0, 43.0), "Gap icon", ">");
        }
        1 => {
            let n = g.panel(g.page(), tl(30.0, 30.0, 500.0, 190.0), "Survival status");
            for (i, (label, number)) in [("ALIVE", "18"), ("KILLS", "04"), ("ZONE", "02")].iter().enumerate() {
                let card = g.card(n, tl(17.0 + i as f32 * 160.0, 18.0, 145.0, 150.0), "Survival metric");
                g.heading(card, tl(8.0, 15.0, 129.0, 38.0), "Metric label", label, 24.0);
                g.heading(card, tl(8.0, 60.0, 129.0, 72.0), "Metric value", number, 58.0);
            }
            let zone = plaque(g, tl(30.0, 240.0, 500.0, 108.0), "Zone countdown", "ZONE CLOSES", "01:24 remaining");
            g.bar(zone, tl(18.0, 96.0, 460.0, 10.0), "Zone progress", 0.45, g.t.warn);
        }
        _ => {
            let n = g.panel(g.page(), tc(0.0, 25.0, 620.0, 140.0), "Team scoreboard");
            for (x, team, score) in [(17.0, "BLUE", "12"), (403.0, "RED", "09")] {
                let card = g.card(n, tl(x, 15.0, 200.0, 110.0), "Team score");
                g.heading(card, tl(10.0, 10.0, 180.0, 38.0), "Team name", team, 24.0);
                g.heading(card, tl(10.0, 52.0, 180.0, 50.0), "Team points", score, 42.0);
            }
            g.heading(n, tl(225.0, 40.0, 170.0, 60.0), "Score divider", "VS", 34.0);
            let timer = plaque(g, tc(0.0, 180.0, 250.0, 105.0), "Match clock", "TIME", "04:32");
            g.icon(timer, tr(15.0, 20.0, 40.0, 40.0), "Clock glyph", "!");
        }
    }
}

fn enemy(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            let n = plaque(g, tc(0.0, 30.0, 1080.0, 190.0), "Boss health plate", "THE HOLLOW WARDEN", "Guardian of the lower gate");
            g.bar(n, tl(30.0, 116.0, 1020.0, 32.0), "Boss health", 0.64, g.t.bad);
            for i in 0..3 { g.icon(n, tl(460.0 + i as f32 * 56.0, 151.0, 35.0, 35.0), "Boss phase", "!"); }
            let phase = plaque(g, tc(0.0, 235.0, 360.0, 102.0), "Boss phase notice", "PHASE TWO", "The guard is broken");
            g.bar(phase, tl(16.0, 97.0, 328.0, 4.0), "Phase divider", 1.0, g.t.accent);
        }
        1 => {
            for (x, y, label, damage) in [(240.0, 310.0, "Raider", "-45"), (810.0, 215.0, "Scout", "-18"), (1360.0, 370.0, "Guard", "-32")] {
                let n = g.panel(g.page(), tl(x, y, 250.0, 112.0), "Floating enemy plate");
                g.heading_l(n, tl(14.0, 8.0, 140.0, 38.0), "Enemy name", label, 24.0);
                g.value(n, tr(14.0, 8.0, 70.0, 38.0), "Damage number", damage, 26.0);
                g.bar(n, tl(14.0, 67.0, 222.0, 22.0), "Enemy health", 0.5, g.t.bad);
            }
        }
        _ => {
            let n = g.panel(g.page(), cr(30.0, 0.0, 390.0, 360.0), "Target frame");
            g.art(n, tl(18.0, 18.0, 115.0, 115.0), "Target portrait");
            g.heading_l(n, tl(150.0, 25.0, 220.0, 45.0), "Target name", "SENTINEL", 30.0);
            g.badge(n, tl(150.0, 83.0, 110.0, 40.0), "Target level", "LV 18");
            g.label(n, tl(20.0, 160.0, 300.0, 35.0), "Health label", "TARGET HEALTH", 24.0);
            g.bar(n, tl(20.0, 210.0, 350.0, 30.0), "Target health", 0.43, g.t.bad);
            for i in 0..3 { g.icon(n, tl(25.0 + i as f32 * 65.0, 275.0, 48.0, 48.0), "Target effect", "!"); }
        }
    }
}

fn info(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            for (i, (title, detail)) in [("MAP UPDATED", "New route revealed"), ("SUPPLY FOUND", "Open the field cache"), ("CHECKPOINT", "Progress saved")].iter().enumerate() {
                let n = g.panel(g.page(), tr(30.0, 30.0 + i as f32 * 125.0, 440.0, 112.0), "Notification card");
                g.icon(n, tl(17.0, 29.0, 52.0, 52.0), "Notice icon", "!");
                g.heading_l(n, tl(85.0, 15.0, 335.0, 40.0), "Notice title", title, 25.0);
                g.note(n, tl(85.0, 62.0, 335.0, 34.0), "Notice detail", detail, 22.0);
            }
        }
        1 => {
            let n = plaque(g, cc(0.0, -100.0, 760.0, 220.0), "Major announcement", "GATE OPENED", "The northern passage is clear");
            g.rect(n, tl(25.0, 112.0, 710.0, 6.0), "Announcement rule", g.t.accent, 0.0);
            g.icon(n, tl(347.0, 145.0, 58.0, 58.0), "Announcement icon", "!");
            let hint = plaque(g, cc(0.0, 100.0, 360.0, 102.0), "Continue hint", "CONTINUE", "Press any key");
            g.icon(hint, tr(16.0, 18.0, 43.0, 43.0), "Continue glyph", ">");
        }
        _ => {
            let n = g.panel(g.page(), bl(30.0, 30.0, 530.0, 285.0), "Event feed");
            g.heading_l(n, tl(18.0, 15.0, 490.0, 40.0), "Feed heading", "FIELD LOG", 27.0);
            for (i, (time, event)) in [("00:43", "Bridge secured"), ("00:51", "Team regrouped"), ("01:04", "Signal restored")].iter().enumerate() {
                let row = g.card(n, tl(15.0, 67.0 + i as f32 * 69.0, 500.0, 58.0), "Event row");
                g.note(row, tl(12.0, 11.0, 90.0, 34.0), "Event time", time, 22.0);
                g.label(row, tl(112.0, 11.0, 370.0, 34.0), "Event text", event, 24.0);
            }
        }
    }
}

fn area(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            let n = plaque(g, cc(0.0, -180.0, 780.0, 175.0), "Centered area title", "ASHEN VALLEY", "The old road winds east");
            g.rect(n, tl(25.0, 112.0, 730.0, 5.0), "Title rule", g.t.accent, 0.0);
            for i in 0..3 { g.icon(n, tl(325.0 + i as f32 * 50.0, 131.0, 34.0, 34.0), "Title ornament", "*"); }
        }
        1 => {
            let n = g.panel(g.page(), bl(55.0, 80.0, 670.0, 240.0), "Lower third location");
            g.rect(n, tl(20.0, 22.0, 10.0, 194.0), "Location accent", g.t.accent, 0.0);
            g.note(n, tl(55.0, 22.0, 580.0, 38.0), "Chapter", "CHAPTER IV", 24.0);
            g.heading_l(n, tl(55.0, 72.0, 580.0, 75.0), "Place name", "THE DEEP HALL", 51.0);
            g.label(n, tl(55.0, 163.0, 580.0, 42.0), "Place detail", "Beneath the abandoned watchtower", 25.0);
            let found = plaque(g, bl(55.0, 335.0, 330.0, 105.0), "Discovery note", "AREA DISCOVERED", "A new path is open");
            g.icon(found, tr(15.0, 18.0, 40.0, 40.0), "Discovery icon", "!");
        }
        _ => {
            let n = g.panel(g.page(), tc(0.0, 28.0, 680.0, 165.0), "Top location banner");
            g.heading(n, tl(20.0, 18.0, 640.0, 65.0), "Place name", "GOLDEN ATRIUM", 46.0);
            g.note(n, tl(20.0, 90.0, 640.0, 36.0), "Place subtitle", "Sanctuary of the eastern court", 24.0);
            g.rect(n, tl(25.0, 138.0, 630.0, 5.0), "Banner rule", g.t.accent, 0.0);
            for x in [20.0, 620.0] { g.icon(n, tl(x, 61.0, 40.0, 40.0), "Banner ornament", "*"); }
        }
    }
}

fn loot(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            let n = g.panel(g.page(), tr(30.0, 100.0, 400.0, 370.0), "Pickup log");
            g.heading_l(n, tl(17.0, 15.0, 365.0, 40.0), "Pickup heading", "PICKED UP", 26.0);
            for (i, (item, qty)) in [("Silver leaf", "x2"), ("River stone", "x4"), ("Ancient coin", "x1"), ("Healing herb", "x3")].iter().enumerate() {
                let row = g.card(n, tl(15.0, 66.0 + i as f32 * 73.0, 370.0, 62.0), "Pickup row");
                g.icon(row, tl(10.0, 10.0, 43.0, 43.0), "Pickup icon", "+");
                g.label(row, tl(62.0, 12.0, 225.0, 38.0), "Item name", item, 23.0);
                g.value(row, tr(10.0, 12.0, 65.0, 38.0), "Item quantity", qty, 24.0);
            }
        }
        1 => {
            let reward = plaque(g, cc(0.0, -110.0, 400.0, 145.0), "XP popup", "+150 XP", "Quest completed");
            g.icon(reward, tr(16.0, 17.0, 45.0, 45.0), "XP icon", "+");
            let n = g.panel(g.page(), bc(0.0, 25.0, 850.0, 105.0), "Experience strip");
            g.label(n, tl(20.0, 13.0, 200.0, 35.0), "Level", "LEVEL 12", 26.0);
            g.value(n, tr(20.0, 13.0, 250.0, 35.0), "XP value", "2,450 / 3,000", 24.0);
            g.bar(n, tl(20.0, 62.0, 810.0, 22.0), "Experience bar", 0.82, g.t.accent);
        }
        _ => {
            let n = g.panel(g.page(), tr(30.0, 30.0, 500.0, 300.0), "Elimination feed");
            g.heading_l(n, tl(16.0, 15.0, 465.0, 38.0), "Feed heading", "ELIMINATIONS", 25.0);
            for (i, (actor, target)) in [("Ranger", "Scout"), ("Pilot", "Guard"), ("Ranger", "Sentry")].iter().enumerate() {
                let row = g.card(n, tl(15.0, 64.0 + i as f32 * 72.0, 470.0, 61.0), "Elimination row");
                g.label(row, tl(12.0, 12.0, 155.0, 36.0), "Attacker", actor, 23.0);
                g.badge(row, tl(180.0, 11.0, 70.0, 40.0), "Elimination marker", "x");
                g.value(row, tr(12.0, 12.0, 175.0, 36.0), "Target", target, 23.0);
            }
        }
    }
}

fn scoring(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            let n = g.panel(g.page(), cr(35.0, 0.0, 340.0, 390.0), "Combo meter");
            g.heading(n, tl(20.0, 20.0, 300.0, 55.0), "Combo heading", "COMBO", 32.0);
            g.heading(n, tl(20.0, 84.0, 300.0, 125.0), "Multiplier", "x 4.5", 80.0);
            g.bar(n, tl(25.0, 230.0, 290.0, 28.0), "Combo duration", 0.72, g.t.accent);
            g.note(n, tl(25.0, 276.0, 290.0, 36.0), "Combo hint", "Keep the chain alive", 24.0);
            g.badge(n, tl(85.0, 334.0, 170.0, 42.0), "Combo rank", "GREAT");
        }
        1 => {
            let n = plaque(g, tc(0.0, 25.0, 530.0, 165.0), "Score display", "SCORE  18,450", "Best  21,000");
            g.bar(n, tl(20.0, 115.0, 490.0, 25.0), "Score progress", 0.88, g.t.accent);
            for (x, y, points) in [(380.0, 340.0, "+200"), (1080.0, 260.0, "+75"), (1350.0, 460.0, "+500")] {
                let popup = g.panel(g.page(), tl(x, y, 180.0, 70.0), "Bonus popup");
                g.heading(popup, full(), "Bonus value", points, 32.0);
            }
        }
        _ => {
            let n = g.panel(g.page(), cl(35.0, 0.0, 305.0, 390.0), "Style rank");
            g.note(n, tl(20.0, 15.0, 265.0, 40.0), "Rank heading", "STYLE RANK", 25.0);
            g.heading(n, tl(20.0, 65.0, 265.0, 185.0), "Grade", "A", 155.0);
            g.bar(n, tl(20.0, 270.0, 265.0, 29.0), "Style progress", 0.76, g.t.warn);
            g.label(n, tl(20.0, 318.0, 265.0, 40.0), "Next grade", "Next: S rank", 26.0);
            let feat = plaque(g, cl(35.0, 250.0, 305.0, 108.0), "Style feat", "CLEAN FINISH", "+250 points");
            symbol_card(g, feat, tr(12.0, 18.0, 44.0, 44.0), "Feat icon", "+");
        }
    }
}

fn clock(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            let n = plaque(g, tc(0.0, 30.0, 470.0, 185.0), "Countdown panel", "TIME REMAINING", "Complete the extraction");
            g.heading(n, tl(20.0, 105.0, 430.0, 65.0), "Countdown", "02:45", 53.0);
            let warning = plaque(g, tc(0.0, 235.0, 280.0, 105.0), "Time warning", "HURRY UP", "The gate closes soon");
            g.icon(warning, tr(14.0, 18.0, 40.0, 40.0), "Warning icon", "!");
        }
        1 => {
            let n = g.panel(g.page(), tr(30.0, 30.0, 400.0, 290.0), "Race stopwatch");
            g.note(n, tl(18.0, 15.0, 360.0, 35.0), "Timer heading", "CURRENT TIME", 24.0);
            g.heading_l(n, tl(18.0, 56.0, 360.0, 82.0), "Current time", "01:42.38", 50.0);
            for (i, (label, value)) in [("BEST LAP", "00:48.12"), ("LAST SPLIT", "+00:02.40")].iter().enumerate() {
                let row = g.card(n, tl(15.0, 154.0 + i as f32 * 58.0, 370.0, 51.0), "Timing row");
                g.label(row, tl(10.0, 9.0, 180.0, 34.0), "Split label", label, 23.0);
                g.value(row, tr(10.0, 9.0, 175.0, 34.0), "Split time", value, 23.0);
            }
        }
        _ => {
            let n = g.panel(g.page(), tr(30.0, 30.0, 340.0, 230.0), "Day clock");
            g.icon(n, tl(20.0, 20.0, 75.0, 75.0), "Sun icon", "*");
            g.heading_l(n, tl(112.0, 25.0, 210.0, 60.0), "World time", "16:30", 46.0);
            g.label(n, tl(20.0, 120.0, 300.0, 40.0), "Day", "DAY  08", 28.0);
            g.bar(n, tl(20.0, 182.0, 300.0, 21.0), "Daylight", 0.68, g.t.warn);
            let weather = plaque(g, tr(30.0, 275.0, 340.0, 106.0), "Weather", "CLEAR SKIES", "Evening approaches");
            g.icon(weather, tr(13.0, 19.0, 42.0, 42.0), "Weather icon", "*");
        }
    }
}

fn objectives(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            let n = g.panel(g.page(), tl(30.0, 30.0, 500.0, 295.0), "Mission checklist");
            g.heading_l(n, tl(18.0, 15.0, 460.0, 40.0), "Objective heading", "OBJECTIVES", 27.0);
            for (i, (mark, task)) in [("x", "Enter the relay"), ("x", "Restore power"), ("?", "Reach the roof")].iter().enumerate() {
                let row = g.card(n, tl(15.0, 68.0 + i as f32 * 69.0, 470.0, 58.0), "Objective row");
                g.badge(row, tl(9.0, 9.0, 40.0, 40.0), "Objective state", mark);
                g.label(row, tl(62.0, 11.0, 390.0, 36.0), "Objective task", task, 24.0);
            }
        }
        1 => {
            let n = plaque(g, cc(0.0, -140.0, 700.0, 245.0), "New objective callout", "NEW OBJECTIVE", "Light the eastern beacon");
            g.rect(n, tl(25.0, 123.0, 650.0, 5.0), "Objective rule", g.t.accent, 0.0);
            g.heading(n, tl(30.0, 150.0, 640.0, 60.0), "Chapter", "CHAPTER 04", 28.0);
            g.icon(n, tr(20.0, 18.0, 46.0, 46.0), "Objective icon", "!");
            let hint = plaque(g, cc(0.0, 60.0, 350.0, 105.0), "Tracking notice", "TRACKED", "Follow the marker");
            g.icon(hint, tr(15.0, 20.0, 42.0, 42.0), "Tracking icon", ">");
        }
        _ => {
            let n = plaque(g, tr(30.0, 30.0, 460.0, 190.0), "Goal tracker", "COLLECT  7 / 12", "Signal fragments");
            g.bar(n, tl(20.0, 115.0, 420.0, 27.0), "Goal progress", 0.58, g.t.accent);
            for i in 0..4 { g.icon(n, tl(80.0 + i as f32 * 75.0, 152.0, 34.0, 34.0), "Collected fragment", "+"); }
        }
    }
}

fn waypoint(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            for (x, y, text) in [(230.0, 270.0, "CACHE  80m"), (810.0, 220.0, "EXIT  240m"), (1380.0, 360.0, "ALLY  55m")] {
                let n = g.panel(g.page(), tl(x, y, 240.0, 100.0), "World marker");
                g.icon(n, tl(14.0, 23.0, 50.0, 50.0), "Marker glyph", "!");
                g.heading_l(n, tl(76.0, 29.0, 152.0, 40.0), "Marker distance", text, 23.0);
            }
            let legend = plaque(g, bl(30.0, 30.0, 480.0, 105.0), "Marker legend", "MARKERS", "! Goal    + Ally");
            g.icon(legend, tr(15.0, 20.0, 43.0, 43.0), "Legend glyph", "!");
        }
        1 => {
            let n = plaque(g, cc(0.0, -130.0, 400.0, 175.0), "Focused waypoint", "RELAY TOWER", "Target distance  180m");
            g.icon(n, tl(172.0, 117.0, 50.0, 50.0), "Goal glyph", "!");
            for (x, glyph) in [(420.0, "<"), (1380.0, ">")] {
                let side = g.panel(g.page(), tl(x, 420.0, 120.0, 95.0), "Side indicator");
                g.heading(side, full(), "Direction", glyph, 53.0);
            }
            let hint = plaque(g, cc(0.0, 90.0, 300.0, 105.0), "Waypoint hint", "ON COURSE", "Follow the signal");
            g.icon(hint, tr(15.0, 20.0, 40.0, 40.0), "Course icon", ">");
        }
        _ => {
            for (x, y, glyph) in [(380.0, 340.0, "!"), (930.0, 270.0, "+"), (1240.0, 430.0, "?")] {
                let n = g.panel(g.page(), tl(x, y, 78.0, 78.0), "World pin");
                g.icon(n, tl(13.0, 13.0, 52.0, 52.0), "Pin glyph", glyph);
            }
            let n = plaque(g, br(30.0, 30.0, 430.0, 180.0), "Tracked destination", "FOREST SHRINE", "Travel  420m northeast");
            g.bar(n, tl(20.0, 116.0, 390.0, 25.0), "Travel progress", 0.35, g.t.accent);
            g.badge(n, tl(20.0, 148.0, 115.0, 30.0), "Pin category", "QUEST");
        }
    }
}

fn minimap(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            let n = g.panel(g.page(), tr(30.0, 30.0, 360.0, 400.0), "Round minimap panel");
            g.circle(n, tl(25.0, 20.0, 310.0, 310.0), "Map field", g.t.panel2);
            for (x, y, glyph) in [(150.0, 130.0, "+"), (85.0, 100.0, "!"), (220.0, 205.0, "?")] {
                g.icon(n, tl(x, y, 42.0, 42.0), "Map point", glyph);
            }
            g.heading(n, tl(20.0, 340.0, 320.0, 40.0), "Coordinates", "N 42  E 18", 25.0);
        }
        1 => {
            let n = g.panel(g.page(), bl(30.0, 30.0, 400.0, 440.0), "Tactical minimap");
            let map = g.card(n, tl(20.0, 20.0, 360.0, 330.0), "Square map");
            g.rect(map, tl(110.0, 25.0, 15.0, 275.0), "Map road", g.t.line, 0.0);
            for (x, y, glyph) in [(45.0, 55.0, "+"), (175.0, 140.0, "!"), (275.0, 245.0, "?")] {
                g.icon(map, tl(x, y, 42.0, 42.0), "Tactical pin", glyph);
            }
            g.label(n, tl(20.0, 370.0, 360.0, 45.0), "Map legend", "+ Squad   ! Goal   ? Cache", 23.0);
        }
        _ => {
            let n = g.panel(g.page(), tr(30.0, 30.0, 390.0, 440.0), "Radar display");
            for (i, inset) in [25.0, 75.0, 125.0].iter().enumerate() {
                g.circle(n, tl(*inset, *inset, 340.0 - i as f32 * 100.0, 340.0 - i as f32 * 100.0), "Radar ring", alpha(g.t.accent, 65 + i as u8 * 35));
            }
            for (x, y) in [(130.0, 90.0), (240.0, 220.0), (95.0, 250.0)] {
                g.icon(n, tl(x, y, 32.0, 32.0), "Radar contact", "!");
            }
            g.heading(n, tl(20.0, 365.0, 350.0, 48.0), "Radar zoom", "RADAR  x2", 26.0);
        }
    }
}

fn compass(g: &mut G, v: u8) {
    scene(g);
    match v {
        0 => {
            let n = g.panel(g.page(), tc(0.0, 25.0, 860.0, 140.0), "Top compass");
            for (i, dir) in ["W", "NW", "N", "NE", "E"].iter().enumerate() {
                g.heading(n, tl(25.0 + i as f32 * 166.0, 15.0, 130.0, 45.0), "Direction", dir, 30.0);
                g.rect(n, tl(88.0 + i as f32 * 166.0, 65.0, 4.0, 20.0), "Bearing tick", g.t.line, 0.0);
            }
            g.badge(n, tl(360.0, 95.0, 140.0, 36.0), "Bearing degrees", "012 DEG");
            g.icon(n, tl(413.0, 60.0, 35.0, 35.0), "Goal bearing", "!");
        }
        1 => {
            let n = g.panel(g.page(), bc(0.0, 25.0, 720.0, 150.0), "Bottom compass");
            for (i, dir) in ["SW", "W", "NW", "N", "NE"].iter().enumerate() {
                let x = 20.0 + i as f32 * 138.0;
                g.heading(n, tl(x, 18.0, 120.0, 48.0), "Direction", dir, 28.0);
                g.rect(n, tl(x + 57.0, 70.0, 6.0, 26.0), "Compass tick", g.t.line, 0.0);
            }
            g.heading(n, tl(235.0, 108.0, 250.0, 35.0), "Heading", "HEADING  344", 24.0);
            g.icon(n, tl(340.0, 69.0, 42.0, 42.0), "Route marker", "!");
        }
        _ => {
            let n = g.panel(g.page(), tr(30.0, 30.0, 340.0, 395.0), "Compass rose");
            g.circle(n, tl(35.0, 20.0, 270.0, 270.0), "Compass field", g.t.panel2);
            for (x, y, dir) in [(147.0, 28.0, "N"), (260.0, 135.0, "E"), (147.0, 245.0, "S"), (36.0, 135.0, "W")] {
                let tag = g.card(n, tl(x, y, 45.0, 42.0), "Rose direction");
                g.heading(tag, full(), "Direction letter", dir, 24.0);
            }
            g.icon(n, tl(142.0, 130.0, 55.0, 55.0), "Compass needle", "^");
            g.heading(n, tl(20.0, 315.0, 300.0, 55.0), "Current bearing", "NORTH  008", 26.0);
        }
    }
}

/// Retrato y barras de salud, mana y resistencia en la esquina.
fn vitals_a(g: &mut G) { vitals(g, 0); }
/// Corazones segmentados y resistencia en la franja inferior.
fn vitals_b(g: &mut G) { vitals(g, 1); }
/// Numeros grandes de salud y armadura al estilo shooter.
fn vitals_c(g: &mut G) { vitals(g, 2); }
/// Ocho huecos numerados de acceso rapido en la base.
fn equipment_a(g: &mut G) { equipment(g, 0); }
/// Arma, municion y granadas en la esquina inferior derecha.
fn equipment_b(g: &mut G) { equipment(g, 1); }
/// Cuatro habilidades y dos objetos con enfriamientos.
fn equipment_c(g: &mut G) { equipment(g, 2); }
/// Monedas, gemas y llaves apiladas arriba a la derecha.
fn collection_a(g: &mut G) { collection(g, 0); }
/// Materiales recogidos en una lista vertical izquierda.
fn collection_b(g: &mut G) { collection(g, 1); }
/// Aviso central de reliquias con barra de avance.
fn collection_c(g: &mut G) { collection(g, 2); }
/// Puesto de carrera con vuelta y distancia al siguiente rival.
fn rank_a(g: &mut G) { rank(g, 0); }
/// Supervivientes, bajas y zona en tarjetas superiores.
fn rank_b(g: &mut G) { rank(g, 1); }
/// Marcador enfrentado de dos equipos arriba al centro.
fn rank_c(g: &mut G) { rank(g, 2); }
/// Barra extensa de jefe con nombre y fases.
fn enemy_a(g: &mut G) { enemy(g, 0); }
/// Tres placas flotantes de enemigos con daÃ±o.
fn enemy_b(g: &mut G) { enemy(g, 1); }
/// Marco de objetivo con retrato, nivel y efectos.
fn enemy_c(g: &mut G) { enemy(g, 2); }
/// Pila de tres notificaciones en la esquina superior.
fn info_a(g: &mut G) { info(g, 0); }
/// Anuncio principal centrado con aviso para continuar.
fn info_b(g: &mut G) { info(g, 1); }
/// Registro de eventos en la zona inferior izquierda.
fn info_c(g: &mut G) { info(g, 2); }
/// Nombre de zona y subtitulo centrados.
fn area_a(g: &mut G) { area(g, 0); }
/// Tercio inferior con capitulo, lugar y descubrimiento.
fn area_b(g: &mut G) { area(g, 1); }
/// Estandarte de ubicacion en el borde superior.
fn area_c(g: &mut G) { area(g, 2); }
/// Lista de botin con iconos y cantidades a la derecha.
fn loot_a(g: &mut G) { loot(g, 0); }
/// Ventana emergente de experiencia y barra inferior.
fn loot_b(g: &mut G) { loot(g, 1); }
/// Registro de eliminaciones arriba a la derecha.
fn loot_c(g: &mut G) { loot(g, 2); }
/// Multiplicador grande con medidor de combo lateral.
fn scoring_a(g: &mut G) { scoring(g, 0); }
/// Puntuacion superior y ventanas flotantes de puntos.
fn scoring_b(g: &mut G) { scoring(g, 1); }
/// Letra de rango de estilo y progreso en el lateral.
fn scoring_c(g: &mut G) { scoring(g, 2); }
/// Cuenta regresiva prominente en el borde superior.
fn clock_a(g: &mut G) { clock(g, 0); }
/// Cronometro de carrera con mejor vuelta y parcial.
fn clock_b(g: &mut G) { clock(g, 1); }
/// Reloj diurno con dia, luz restante y clima.
fn clock_c(g: &mut G) { clock(g, 2); }
/// Lista de objetivos con estados en la esquina izquierda.
fn objectives_a(g: &mut G) { objectives(g, 0); }
/// Aviso central de nuevo objetivo con capitulo.
fn objectives_b(g: &mut G) { objectives(g, 1); }
/// Contador de fragmentos con barra de progreso.
fn objectives_c(g: &mut G) { objectives(g, 2); }
/// Marcadores de distancia flotantes y leyenda inferior.
fn waypoint_a(g: &mut G) { waypoint(g, 0); }
/// Objetivo enfocado con indicadores laterales.
fn waypoint_b(g: &mut G) { waypoint(g, 1); }
/// Pines variados con tarjeta de destino seguido.
fn waypoint_c(g: &mut G) { waypoint(g, 2); }
/// Mapa circular superior con puntos y coordenadas.
fn minimap_a(g: &mut G) { minimap(g, 0); }
/// Mapa tactico cuadrado inferior con leyenda.
fn minimap_b(g: &mut G) { minimap(g, 1); }
/// Radar de anillos y contactos con indicador de zoom.
fn minimap_c(g: &mut G) { minimap(g, 2); }
/// Franja superior con puntos cardinales y grados.
fn compass_a(g: &mut G) { compass(g, 0); }
/// Franja inferior de rumbo y marcador de ruta.
fn compass_b(g: &mut G) { compass(g, 1); }
/// Rosa circular con direcciones y rumbo actual.
fn compass_c(g: &mut G) { compass(g, 2); }
