//! Gallery: title and system menus (title, settings, modals and overlays).

use super::*;

pub fn entries() -> Vec<Entry> {
    use Style::*;
    vec![
        Entry { ty: "Title screen", style: Fantasy, build: title_centered },
        Entry { ty: "Title screen", style: SciFi, build: title_side },
        Entry { ty: "Title screen", style: Minimal, build: title_press_start },
        Entry { ty: "Mode select", style: Cartoon, build: mode_cards },
        Entry { ty: "Mode select", style: Neon, build: mode_list },
        Entry { ty: "Mode select", style: Sports, build: mode_mosaic },
        Entry { ty: "Game difficulty", style: Elegant, build: difficulty_list },
        Entry { ty: "Game difficulty", style: Horror, build: difficulty_cards },
        Entry { ty: "Game difficulty", style: Retro, build: difficulty_custom },
        Entry { ty: "Load / save", style: Fantasy, build: save_slots },
        Entry { ty: "Load / save", style: Minimal, build: save_grid },
        Entry { ty: "Load / save", style: SciFi, build: save_detail },
        Entry { ty: "Loading screen", style: Nature, build: loading_tip },
        Entry { ty: "Loading screen", style: Neon, build: loading_spinner },
        Entry { ty: "Loading screen", style: Military, build: loading_briefing },
        Entry { ty: "Credits", style: Elegant, build: credits_roll },
        Entry { ty: "Credits", style: Cartoon, build: credits_teams },
        Entry { ty: "Credits", style: Retro, build: credits_thanks },
        Entry { ty: "Settings menu", style: Minimal, build: settings_sidebar },
        Entry { ty: "Settings menu", style: SciFi, build: settings_tabs },
        Entry { ty: "Settings menu", style: Fantasy, build: settings_hub },
        Entry { ty: "Settings options", style: Sports, build: options_audio },
        Entry { ty: "Settings options", style: Neon, build: options_graphics },
        Entry { ty: "Settings options", style: Pastel, build: options_accessibility },
        Entry { ty: "Button layout", style: Military, build: layout_gamepad },
        Entry { ty: "Button layout", style: Minimal, build: layout_keyboard },
        Entry { ty: "Button layout", style: Cartoon, build: layout_presets },
        Entry { ty: "Cutscene & story", style: Elegant, build: cutscene_letterbox },
        Entry { ty: "Cutscene & story", style: Fantasy, build: cutscene_storybook },
        Entry { ty: "Cutscene & story", style: Neon, build: cutscene_comic },
        Entry { ty: "Dialogue & speech", style: Retro, build: dialogue_box },
        Entry { ty: "Dialogue & speech", style: Pastel, build: dialogue_novel },
        Entry { ty: "Dialogue & speech", style: SciFi, build: dialogue_choices },
        Entry { ty: "Guided tutorial", style: Cartoon, build: tutorial_spotlight },
        Entry { ty: "Guided tutorial", style: Minimal, build: tutorial_checklist },
        Entry { ty: "Guided tutorial", style: Nature, build: tutorial_pages },
        Entry { ty: "Modal & popup", style: Minimal, build: modal_confirm },
        Entry { ty: "Modal & popup", style: Fantasy, build: modal_reward },
        Entry { ty: "Modal & popup", style: Sports, build: modal_warning },
        Entry { ty: "Character intro", style: Neon, build: intro_profile },
        Entry { ty: "Character intro", style: Fantasy, build: intro_nameplate },
        Entry { ty: "Character intro", style: Cartoon, build: intro_banner },
        Entry { ty: "Feature unlocked", style: SciFi, build: unlock_card },
        Entry { ty: "Feature unlocked", style: Pastel, build: unlock_toast },
        Entry { ty: "Feature unlocked", style: Retro, build: unlock_progress },
    ]
}

// ---- Title screen ------------------------------------------------------------------------------------------------

/// Logo in the middle of the art, vertical menu under it, version and copyright in the corners.
fn title_centered(g: &mut G) {
    let p = g.page();
    g.background();
    g.art(p, cc(0.0, -120.0, 1500.0, 520.0), "Key art");
    let logo = g.panel(p, tc(0.0, 90.0, 980.0, 220.0), "Logo plate");
    g.heading(logo, fill(20.0, 20.0, 20.0, 70.0), "Game title", "KINGDOM OF EMBERS", 84.0);
    g.txt(logo, bottom(18.0, 44.0, 20.0, 20.0), "Tagline", "A tale of ash and crowns", 30.0, g.t.muted, CENTER, false);
    let menu = g.vbox(p, cc(0.0, 230.0, 460.0, 420.0), "Main menu", 18.0);
    for (i, l) in ["Continue", "New game", "Options", "Quit"].iter().enumerate() {
        if i == 0 {
            g.button(menu, sz(460.0, 84.0), &format!("{} button", l), l);
        } else {
            g.button2(menu, sz(460.0, 84.0), &format!("{} button", l), l);
        }
    }
    let ver = g.panel(p, br(30.0, 30.0, 240.0, 56.0), "Version plate");
    g.txt(ver, full(), "Version", "v1.0.0", 24.0, g.t.muted, CENTER, false);
    let cr = g.panel(p, bl(30.0, 30.0, 460.0, 56.0), "Copyright plate");
    g.txt(cr, full(), "Copyright", "(c) 2026 Your Studio", 24.0, g.t.muted, CENTER, false);
}

/// Key art fills the right side; the menu sits in a dark column on the left (modern console layout).
fn title_side(g: &mut G) {
    let p = g.page();
    g.background();
    g.art(p, fill(560.0, 0.0, 0.0, 0.0), "Key art");
    let col = g.panel(p, left(0.0, 640.0, 0.0, 0.0), "Menu column");
    g.txt(col, tl(70.0, 110.0, 520.0, 110.0), "Game title", "STARFALL", 96.0, g.t.accent, LEFT, true);
    g.note(col, tl(74.0, 225.0, 520.0, 44.0), "Subtitle", "Frontier Protocol", 32.0);
    g.rect(col, tl(74.0, 290.0, 120.0, 6.0), "Accent line", g.t.accent, 0.0);
    let menu = g.vbox_fill(col, tl(60.0, 360.0, 500.0, 460.0), "Main menu", 14.0);
    for (i, l) in ["Resume mission", "New campaign", "Multiplayer", "Settings", "Exit to desktop"].iter().enumerate() {
        g.menu_item(menu, sz(500.0, 76.0), &format!("{} button", l), l, i == 0);
    }
    prompts(g, col, bl(70.0, 50.0, 520.0, 50.0), &[("A", "Select"), ("B", "Back")]);
    let news = g.panel(p, br(50.0, 50.0, 560.0, 190.0), "News card");
    g.badge(news, tl(24.0, 22.0, 120.0, 40.0), "News tag", "NEW");
    g.heading_l(news, tl(24.0, 76.0, 510.0, 46.0), "News title", "Season 2 is live", 34.0);
    g.note(news, tl(24.0, 126.0, 510.0, 40.0), "News text", "New maps, new ships and a ranked mode.", 26.0);
}

/// Almost nothing on screen: the logo and a «Press any button» prompt, legal text at the bottom.
fn title_press_start(g: &mut G) {
    let p = g.page();
    g.background();
    g.circle(p, cc(0.0, -90.0, 520.0, 520.0), "Emblem", alpha(g.t.accent, 60));
    g.bg_title(p, cc(0.0, -110.0, 1400.0, 150.0), "Game title", "QUIET HOURS", 120.0);
    g.bg_label(p, cc(0.0, -10.0, 1000.0, 50.0), "Subtitle", "a short story about a long night", 32.0, CENTER);
    let prompt = g.panel(p, cc(0.0, 230.0, 560.0, 90.0), "Prompt plate");
    g.heading(prompt, full(), "Press start", "Press any button", 36.0);
    g.bg_label(p, bottom(40.0, 40.0, 100.0, 100.0), "Legal", "(c) 2026 Your Studio. All rights reserved.", 22.0, CENTER);
}

/// Row of key/label pairs («A Select  B Back»), shared by several templates.
fn prompts(g: &mut G, parent: N, pos: Pos, items: &[(&str, &str)]) -> N {
    let row = g.hbox(parent, pos, "Prompts", 12.0);
    for (k, l) in items {
        g.key(row, sz(50.0, 50.0), &format!("{} key", l), k);
        g.note(row, sz(120.0, 50.0), &format!("{} label", l), l, 26.0);
    }
    row
}

/// Title bar at the top of a full-screen menu: big title on the left, a hint on the right.
fn screen_header(g: &mut G, title: &str, hint: &str) -> N {
    let p = g.page();
    let bar = g.panel(p, top(0.0, 130.0, 0.0, 0.0), "Header");
    g.heading_l(bar, cl(60.0, 0.0, 1000.0, 80.0), "Screen title", title, 60.0);
    g.txt(bar, cr(60.0, 0.0, 700.0, 50.0), "Header hint", hint, 26.0, g.t.muted, RIGHT, false);
    bar
}

// ---- Mode select -------------------------------------------------------------------------------------------------

/// Four big mode cards in a row, each with art, name and a line of description.
fn mode_cards(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 60.0, 1200.0, 110.0), "Screen title", "Choose a mode", 80.0);
    let row = g.hbox_c(p, cc(0.0, 40.0, 1800.0, 600.0), "Modes", 40.0);
    let modes = [("Story", "Play the adventure from the start"), ("Arcade", "Beat the stages in a row"), ("Versus", "Two players, one couch"), ("Online", "Play with friends")];
    for (i, (m, d)) in modes.iter().enumerate() {
        let card = if i == 0 { g.card_on(row, sz(400.0, 580.0), &format!("{} card", m)) } else { g.card(row, sz(400.0, 580.0), &format!("{} card", m)) };
        g.art(card, top(24.0, 300.0, 24.0, 24.0), &format!("{} art", m));
        g.heading(card, top(350.0, 70.0, 20.0, 20.0), &format!("{} name", m), m, 48.0);
        g.txt(card, top(430.0, 110.0, 30.0, 30.0), &format!("{} text", m), d, 26.0, g.t.muted, CENTER, false);
    }
    let back = g.panel(p, bl(40.0, 40.0, 460.0, 80.0), "Prompt plate");
    prompts(g, back, cl(30.0, 0.0, 400.0, 50.0), &[("A", "Select"), ("B", "Back")]);
}

/// Vertical list of modes on the left, a preview with description and best score on the right.
fn mode_list(g: &mut G) {
    let p = g.page();
    g.background();
    screen_header(g, "GAME MODES", "Up / Down to browse");
    let list = g.vbox_fill(p, tl(60.0, 200.0, 620.0, 760.0), "Mode list", 16.0);
    for (i, m) in ["Campaign", "Time attack", "Survival", "Duel", "Practice", "Custom match"].iter().enumerate() {
        g.menu_item(list, sz(620.0, 100.0), &format!("{} button", m), m, i == 1);
    }
    let prev = g.panel(p, fill(740.0, 200.0, 60.0, 60.0), "Preview panel");
    g.art(prev, top(30.0, 460.0, 30.0, 30.0), "Mode art");
    g.heading_l(prev, top(520.0, 70.0, 40.0, 40.0), "Mode name", "Time attack", 52.0);
    g.note(prev, top(600.0, 90.0, 40.0, 40.0), "Mode text", "Race the clock through every stage. Each hit costs you two seconds.", 28.0);
    g.badge(prev, bl(40.0, 40.0, 360.0, 60.0), "Best time", "BEST 03:41.27");
    g.button(prev, br(40.0, 36.0, 300.0, 70.0), "Play button", "Play");
}

/// Mosaic of tiles: one big featured mode and five smaller ones.
fn mode_mosaic(g: &mut G) {
    let p = g.page();
    g.background();
    screen_header(g, "PLAY", "Season 4 · Week 2");
    let big = g.card_on(p, tl(60.0, 190.0, 860.0, 830.0), "Featured tile");
    g.art(big, fill(0.0, 0.0, 0.0, 220.0), "Featured art");
    g.badge(big, tl(30.0, 30.0, 200.0, 50.0), "Featured tag", "FEATURED");
    g.heading_l(big, bl(30.0, 120.0, 780.0, 70.0), "Featured name", "Championship", 56.0);
    g.note(big, bl(30.0, 60.0, 780.0, 50.0), "Featured text", "Ten rounds, one trophy.", 30.0);
    let grid = g.grid(p, tl(960.0, 190.0, 900.0, 830.0), "Mode tiles", 440.0, 260.0, 20.0);
    for m in ["Quick match", "Career", "Training", "Online league", "Custom", "Replays"] {
        let tile = g.card(grid, sz(440.0, 260.0), &format!("{} tile", m));
        g.art(tile, top(0.0, 160.0, 0.0, 0.0), &format!("{} art", m));
        g.heading_l(tile, bl(24.0, 26.0, 390.0, 60.0), &format!("{} name", m), m, 36.0);
    }
}

// ---- Game difficulty ---------------------------------------------------------------------------------------------

/// List of difficulties on the left; the chosen one is explained on the right.
fn difficulty_list(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tl(80.0, 80.0, 900.0, 100.0), "Screen title", "Select difficulty", 72.0);
    let list = g.vbox_fill(p, tl(80.0, 240.0, 640.0, 560.0), "Difficulty list", 20.0);
    for (i, d) in ["Story", "Balanced", "Veteran", "Legend"].iter().enumerate() {
        g.menu_item(list, sz(640.0, 110.0), &format!("{} button", d), d, i == 1);
    }
    let info = g.panel(p, tr(80.0, 240.0, 1000.0, 560.0), "Description panel");
    g.heading_l(info, tl(50.0, 40.0, 900.0, 70.0), "Difficulty name", "Balanced", 54.0);
    g.label(info, tl(50.0, 130.0, 900.0, 120.0), "Difficulty text", "The way the game was meant to be played. Enemies hit hard but fair, and checkpoints are generous.", 30.0);
    let stats = g.vbox_fill(info, tl(50.0, 290.0, 900.0, 220.0), "Difficulty stats", 16.0);
    for (k, v) in [("Enemy damage", 0.5), ("Enemy health", 0.5), ("Resources", 0.6)] {
        let row = g.el(stats, sz(900.0, 60.0), Kind::Empty, &format!("{} row", k), |_| {});
        g.note(row, cl(0.0, 0.0, 320.0, 50.0), &format!("{} label", k), k, 28.0);
        g.bar(row, cr(0.0, 0.0, 540.0, 24.0), &format!("{} bar", k), v, g.t.accent);
    }
    g.button(p, br(80.0, 80.0, 360.0, 90.0), "Confirm button", "Confirm");
    g.bg_label(p, bl(80.0, 100.0, 900.0, 50.0), "Warning", "You can change this later in Settings.", 28.0, LEFT);
}

/// Three big cards side by side with a short promise each; the middle one is recommended.
fn difficulty_cards(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 80.0, 1400.0, 110.0), "Screen title", "HOW BRAVE ARE YOU?", 80.0);
    let row = g.hbox_c(p, cc(0.0, 60.0, 1700.0, 620.0), "Difficulty cards", 50.0);
    let items = [("Calm", "For the story. Fewer monsters, more ammo.", "I"), ("Dread", "The intended experience.", "II"), ("Nightmare", "One life. No saves. No mercy.", "III")];
    for (i, (n, d, glyph)) in items.iter().enumerate() {
        let card = if i == 1 { g.card_on(row, sz(500.0, 600.0), &format!("{} card", n)) } else { g.card(row, sz(500.0, 600.0), &format!("{} card", n)) };
        g.icon(card, tc(0.0, 50.0, 170.0, 170.0), &format!("{} icon", n), glyph);
        g.heading(card, tc(0.0, 260.0, 440.0, 80.0), &format!("{} name", n), n, 56.0);
        g.txt(card, tc(0.0, 350.0, 420.0, 120.0), &format!("{} text", n), d, 28.0, g.t.muted, CENTER, false);
        if i == 1 {
            g.badge(card, bc(0.0, 40.0, 300.0, 54.0), "Recommended tag", "RECOMMENDED");
        }
    }
}

/// Custom difficulty: a master slider plus individual modifiers you can switch on and off.
fn difficulty_custom(g: &mut G) {
    let p = g.page();
    g.background();
    let win = g.panel(p, cc(0.0, 0.0, 1300.0, 900.0), "Custom difficulty window");
    g.heading(win, top(40.0, 80.0, 40.0, 40.0), "Window title", "CUSTOM DIFFICULTY", 60.0);
    g.label(win, tl(80.0, 170.0, 400.0, 50.0), "Overall label", "Overall", 32.0);
    g.slider(win, tl(480.0, 182.0, 600.0, 30.0), "Overall slider", 0.66);
    g.value(win, tr(80.0, 170.0, 100.0, 50.0), "Overall value", "4/6", 32.0);
    g.rect(win, top(250.0, 4.0, 80.0, 80.0), "Divider", g.t.line, 0.0);
    let mods = g.grid(win, top(290.0, 420.0, 80.0, 80.0), "Modifiers", 560.0, 70.0, 20.0);
    for (i, m) in ["Permadeath", "Limited saves", "Faster enemies", "No minimap", "Hunger", "Fragile weapons", "Friendly fire", "Hidden health"].iter().enumerate() {
        g.toggle(mods, sz(560.0, 70.0), &format!("{} toggle", m), m, i % 3 == 0);
    }
    let buttons = g.hbox_c(win, bottom(40.0, 90.0, 80.0, 80.0), "Buttons", 40.0);
    g.button2(buttons, sz(380.0, 90.0), "Reset button", "Reset");
    g.button(buttons, sz(380.0, 90.0), "Start button", "Start game");
}

// ---- Load / save -------------------------------------------------------------------------------------------------

/// Vertical list of save slots, each with a thumbnail, location, play time and date.
fn save_slots(g: &mut G) {
    let p = g.page();
    g.background();
    screen_header(g, "Load game", "3 of 8 slots used");
    let list = g.vbox_fill(p, tc(0.0, 180.0, 1400.0, 720.0), "Save slots", 20.0);
    let saves = [("The Ashen Road", "12 h 40 min", "Today 21:14"), ("Harbor of Kings", "9 h 02 min", "Yesterday 18:30"), ("Old Mill", "2 h 15 min", "Nov 20 10:05"), ("Empty slot", "", "")];
    for (i, (loc, time, date)) in saves.iter().enumerate() {
        let slot = if i == 0 { g.card_on(list, sz(1400.0, 160.0), &format!("Slot {}", i + 1)) } else { g.card(list, sz(1400.0, 160.0), &format!("Slot {}", i + 1)) };
        if time.is_empty() {
            g.txt(slot, full(), "Empty label", "Empty slot - press A to save", 32.0, g.t.muted, CENTER, false);
            continue;
        }
        g.art(slot, cl(20.0, 0.0, 220.0, 124.0), "Thumbnail");
        g.heading_l(slot, tl(270.0, 26.0, 700.0, 56.0), "Location", loc, 40.0);
        g.note(slot, tl(270.0, 90.0, 700.0, 44.0), "Chapter", &format!("Chapter {}", 7 - i * 2), 28.0);
        g.value(slot, tr(30.0, 30.0, 400.0, 50.0), "Play time", time, 32.0);
        g.txt(slot, tr(30.0, 90.0, 400.0, 40.0), "Date", date, 26.0, g.t.muted, RIGHT, false);
    }
    let foot = g.panel(p, bc(0.0, 40.0, 820.0, 80.0), "Prompt plate");
    prompts(g, foot, cc(0.0, 0.0, 780.0, 50.0), &[("A", "Load"), ("X", "Delete"), ("B", "Back")]);
}

/// Grid of save cards (3 × 2) with big thumbnails; the selected card has an accent border.
fn save_grid(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tl(80.0, 60.0, 800.0, 90.0), "Screen title", "Save game", 64.0);
    let grid = g.grid(p, tc(0.0, 190.0, 1740.0, 760.0), "Save grid", 560.0, 370.0, 20.0);
    for i in 0..6 {
        let c = if i == 1 { g.card_on(grid, sz(560.0, 370.0), &format!("Save card {}", i + 1)) } else { g.card(grid, sz(560.0, 370.0), &format!("Save card {}", i + 1)) };
        g.art(c, top(16.0, 230.0, 16.0, 16.0), "Thumbnail");
        g.heading_l(c, bl(24.0, 60.0, 360.0, 50.0), "Slot name", &format!("Slot {}", i + 1), 34.0);
        g.note(c, bl(24.0, 20.0, 360.0, 40.0), "Slot date", &format!("Day {} - 0{}:2{}", 12 + i, i + 1, i), 24.0);
        g.badge(c, br(20.0, 30.0, 140.0, 44.0), "Slot level", &format!("LV {}", 10 + i * 4));
    }
    g.button(p, br(80.0, 50.0, 320.0, 80.0), "Save button", "Save here");
}

/// Saves listed on the left; the selected one is shown big on the right with Load and Delete.
fn save_detail(g: &mut G) {
    let p = g.page();
    g.background();
    screen_header(g, "DATA CORE // SAVES", "Autosave every 5 minutes");
    let list = g.vbox_fill(p, tl(60.0, 190.0, 640.0, 830.0), "Save list", 12.0);
    for i in 0..7 {
        let label = if i == 0 { "AUTOSAVE - Sector 9".to_string() } else { format!("Manual {} - Sector {}", i, 9 - i) };
        g.menu_item(list, sz(640.0, 100.0), &format!("Save {}", i + 1), &label, i == 0);
    }
    let det = g.panel(p, fill(760.0, 190.0, 60.0, 60.0), "Detail panel");
    g.art(det, top(30.0, 500.0, 30.0, 30.0), "Screenshot");
    g.heading_l(det, tl(40.0, 560.0, 900.0, 60.0), "Save name", "AUTOSAVE - Sector 9", 44.0);
    let info = g.grid(det, tl(40.0, 640.0, 1000.0, 140.0), "Save info", 320.0, 60.0, 10.0);
    for (k, v) in [("Time", "31:07"), ("Level", "24"), ("Credits", "18 450"), ("Ship", "Wren II"), ("Crew", "5"), ("Date", "11/27")] {
        g.label(info, sz(320.0, 60.0), &format!("{} info", k), &format!("{}: {}", k, v), 28.0);
    }
    g.button2(det, bl(40.0, 30.0, 260.0, 70.0), "Delete button", "Delete");
    g.button(det, br(40.0, 30.0, 300.0, 70.0), "Load button", "Load");
}

// ---- Loading screen ----------------------------------------------------------------------------------------------

/// Full-screen art with a tip card and a progress bar at the bottom.
fn loading_tip(g: &mut G) {
    let p = g.page();
    g.background();
    g.art(p, full(), "Loading art");
    let card = g.panel(p, bottom(60.0, 220.0, 60.0, 60.0), "Tip card");
    g.badge(card, tl(40.0, 30.0, 160.0, 48.0), "Tip tag", "TIP");
    g.label(card, tl(230.0, 26.0, 1300.0, 60.0), "Tip text", "Berries heal a little, cooked fish heals a lot.", 34.0);
    g.note(card, tl(40.0, 110.0, 400.0, 40.0), "Area name", "Entering: Mossy Hollow", 28.0);
    g.bar(card, bottom(40.0, 26.0, 40.0, 200.0), "Progress bar", 0.62, g.t.accent);
    g.value(card, br(40.0, 30.0, 140.0, 40.0), "Percent", "62%", 30.0);
}

/// Dark screen with a ring in the centre, the percentage and a «Loading» label.
fn loading_spinner(g: &mut G) {
    let p = g.page();
    g.background();
    g.circle(p, cc(0.0, -60.0, 360.0, 360.0), "Spinner ring", g.t.accent);
    let inner = g.panel(p, cc(0.0, -60.0, 300.0, 300.0), "Spinner core");
    let s = g.s;
    g.edit(inner, |e| e.radius = 150.0 * s);
    g.heading(inner, full(), "Percent", "74%", 72.0);
    g.bg_title(p, cc(0.0, 200.0, 800.0, 70.0), "Loading label", "LOADING", 48.0);
    g.bg_label(p, cc(0.0, 270.0, 900.0, 50.0), "Hint", "Do not turn off the console while this icon is shown", 26.0, CENTER);
    g.circle(p, br(60.0, 60.0, 70.0, 70.0), "Save icon", g.t.accent2);
}

/// Mission briefing while loading: text on the left, map on the right, progress at the bottom.
fn loading_briefing(g: &mut G) {
    let p = g.page();
    g.background();
    let brief = g.panel(p, tl(60.0, 60.0, 760.0, 820.0), "Briefing panel");
    g.badge(brief, tl(40.0, 40.0, 260.0, 50.0), "Classified tag", "CLASSIFIED");
    g.heading_l(brief, tl(40.0, 120.0, 680.0, 80.0), "Operation name", "OPERATION GRAY DAWN", 48.0);
    g.note(brief, tl(40.0, 210.0, 680.0, 40.0), "Location", "Northern ridge - 05:40", 28.0);
    let objs = g.vbox_fill(brief, tl(40.0, 290.0, 680.0, 400.0), "Objectives", 14.0);
    for o in ["Reach the relay tower", "Disable the radar", "Extract the pilot", "Leave no trace"] {
        let row = g.card(objs, sz(680.0, 84.0), &format!("{} row", o));
        g.label(row, cl(24.0, 0.0, 620.0, 50.0), &format!("{} text", o), &format!("- {}", o), 28.0);
    }
    let map = g.panel(p, tr(60.0, 60.0, 980.0, 820.0), "Map panel");
    g.art(map, fill(20.0, 20.0, 20.0, 20.0), "Map");
    for (i, (x, y)) in [(260.0, 300.0), (520.0, 420.0), (760.0, 220.0)].iter().enumerate() {
        g.icon(map, tl(*x, *y, 70.0, 70.0), &format!("Marker {}", i + 1), &format!("{}", i + 1));
    }
    let bar = g.panel(p, bottom(40.0, 100.0, 60.0, 60.0), "Progress plate");
    g.note(bar, cl(30.0, 0.0, 300.0, 44.0), "Loading label", "Deploying...", 30.0);
    g.bar(bar, hmid(0.0, 24.0, 340.0, 40.0), "Progress bar", 0.4, g.t.accent);
}

// ---- Credits -----------------------------------------------------------------------------------------------------

/// Centred column of roles and names, as if scrolling up.
fn credits_roll(g: &mut G) {
    let p = g.page();
    g.background();
    let col = g.vbox(p, vmid(0.0, 1000.0, 60.0, 140.0), "Credits column", 10.0);
    g.bg_title(col, sz(1000.0, 110.0), "Game title", "THE LAST LANTERN", 72.0);
    for (role, names) in [("Directed by", "Ada Morrow"), ("Design", "Ilya Brandt   Sora Vance"), ("Art", "Noor Haddad   Pim Okafor"), ("Music", "The Velvet Quartet"), ("Programming", "Lee Castell   Mara Dunn"), ("Special thanks", "Our families and every player")] {
        g.txt(col, sz(1000.0, 44.0), &format!("{} role", role), role, 28.0, g.t.accent, CENTER, false);
        g.bg_title(col, sz(1000.0, 60.0), &format!("{} names", role), names, 40.0);
    }
    let skip = g.panel(p, br(50.0, 40.0, 330.0, 70.0), "Skip plate");
    prompts(g, skip, cc(0.0, 0.0, 280.0, 50.0), &[("B", "Skip")]);
}

/// Teams in two columns of cards, each card listing the members of a team.
fn credits_teams(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 50.0, 1200.0, 100.0), "Screen title", "Made with love by", 64.0);
    let grid = g.grid(p, tc(0.0, 190.0, 1760.0, 780.0), "Team cards", 860.0, 240.0, 30.0);
    for (team, people) in [("Game design", "Rosa Pike, Theo Lark, June Abara"), ("Art & animation", "Kip Moreno, Ines Fold, Bo Yuen"), ("Sound", "Wade Sterling, Lulu Park"), ("Code", "Ari Novak, Dee Santos, Max Rune"), ("QA", "The Bug Hunters"), ("Community", "Every player who said hi")] {
        let c = g.card(grid, sz(860.0, 240.0), &format!("{} card", team));
        g.icon(c, cl(30.0, 0.0, 130.0, 130.0), &format!("{} icon", team), &team[..1]);
        g.heading_l(c, tl(190.0, 40.0, 640.0, 60.0), &format!("{} name", team), team, 40.0);
        g.note(c, tl(190.0, 120.0, 640.0, 90.0), &format!("{} people", team), people, 28.0);
    }
    g.button(p, br(60.0, 40.0, 260.0, 80.0), "Back button", "Back");
}

/// «Thank you for playing» ending card with the final score and a short staff list.
fn credits_thanks(g: &mut G) {
    let p = g.page();
    g.background();
    let card = g.panel(p, cc(0.0, -40.0, 1300.0, 760.0), "Ending card");
    g.heading(card, top(50.0, 110.0, 40.0, 40.0), "Thanks", "THANK YOU FOR PLAYING!", 76.0);
    g.txt(card, top(170.0, 60.0, 40.0, 40.0), "Final score", "FINAL SCORE 0098400", 44.0, g.t.accent, CENTER, true);
    let staff = g.grid(card, top(270.0, 340.0, 100.0, 100.0), "Staff", 540.0, 60.0, 20.0);
    for (r, n) in [("PRODUCER", "K. HOLT"), ("PROGRAM", "R. OYAMA"), ("GRAPHICS", "T. WILDE"), ("SOUND", "M. ROSS"), ("LEVELS", "A. PEREZ"), ("TESTING", "THE GANG")] {
        g.label(staff, sz(540.0, 60.0), &format!("{} line", r), &format!("{}  {}", r, n), 30.0);
    }
    g.note(card, bottom(40.0, 50.0, 40.0, 40.0), "The end", "THE END?  Press START to see the secret ending", 28.0);
}

// ---- Settings menu -----------------------------------------------------------------------------------------------

/// Categories in a sidebar, the options of the chosen category on the right.
fn settings_sidebar(g: &mut G) {
    let p = g.page();
    g.background();
    let side = g.panel(p, left(0.0, 480.0, 0.0, 0.0), "Sidebar");
    g.heading_l(side, tl(50.0, 60.0, 380.0, 80.0), "Screen title", "Settings", 56.0);
    let cats = g.vbox_fill(side, tl(30.0, 180.0, 420.0, 560.0), "Categories", 10.0);
    for (i, c) in ["Gameplay", "Video", "Audio", "Controls", "Accessibility", "Account"].iter().enumerate() {
        g.menu_item(cats, sz(420.0, 80.0), &format!("{} tab", c), c, i == 2);
    }
    let body = g.panel(p, fill(540.0, 60.0, 60.0, 60.0), "Options panel");
    g.heading_l(body, tl(50.0, 40.0, 800.0, 70.0), "Category title", "Audio", 48.0);
    let rows = g.vbox_fill(body, top(140.0, 600.0, 50.0, 50.0), "Option rows", 14.0);
    for (k, v) in [("Master volume", 0.8), ("Music", 0.6), ("Effects", 0.9), ("Voices", 0.7), ("Interface", 0.5)] {
        let row = g.card(rows, sz(1200.0, 100.0), &format!("{} row", k));
        g.label(row, cl(30.0, 0.0, 400.0, 50.0), &format!("{} label", k), k, 30.0);
        g.slider(row, cr(140.0, 0.0, 520.0, 28.0), &format!("{} slider", k), v);
        g.value(row, cr(30.0, 0.0, 90.0, 50.0), &format!("{} value", k), &format!("{}", (v * 100.0) as i32), 30.0);
    }
    g.button2(body, bl(50.0, 40.0, 340.0, 76.0), "Defaults button", "Restore defaults");
    g.button(body, br(50.0, 40.0, 260.0, 76.0), "Apply button", "Apply");
}

/// Tab bar on top, two columns of option rows and the key prompts at the bottom.
fn settings_tabs(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(0.0, 160.0, 0.0, 0.0), "Header");
    g.heading_l(head, tl(60.0, 20.0, 600.0, 60.0), "Screen title", "SYSTEM SETTINGS", 44.0);
    let tabs = g.hbox(head, bl(50.0, 14.0, 1500.0, 64.0), "Tabs", 12.0);
    for (i, t) in ["Game", "Display", "Graphics", "Sound", "Input", "Network"].iter().enumerate() {
        g.menu_item(tabs, sz(230.0, 60.0), &format!("{} tab", t), t, i == 2);
    }
    let cols = g.grid(p, tc(0.0, 210.0, 1800.0, 700.0), "Options", 880.0, 96.0, 20.0);
    for (k, v) in [("Preset", "High"), ("Resolution", "1920 x 1080"), ("Shadows", "Medium"), ("Textures", "Ultra"), ("Anti-aliasing", "TAA"), ("V-Sync", "On"), ("Motion blur", "Off"), ("Field of view", "90"), ("Frame limit", "144"), ("HDR", "Off")] {
        let row = g.card(cols, sz(880.0, 96.0), &format!("{} row", k));
        g.label(row, cl(30.0, 0.0, 420.0, 50.0), &format!("{} label", k), k, 30.0);
        g.value(row, cr(30.0, 0.0, 380.0, 50.0), &format!("{} value", k), &format!("<  {}  >", v), 30.0);
    }
    let foot = g.panel(p, bottom(0.0, 100.0, 0.0, 0.0), "Footer");
    prompts(g, foot, cl(60.0, 0.0, 900.0, 50.0), &[("A", "Change"), ("Y", "Defaults"), ("B", "Back")]);
}

/// A centred card with big category buttons (a hub that opens each settings page).
fn settings_hub(g: &mut G) {
    let p = g.page();
    g.background();
    let card = g.panel(p, cc(0.0, 0.0, 1100.0, 860.0), "Options card");
    g.heading(card, top(40.0, 90.0, 40.0, 40.0), "Screen title", "Options", 64.0);
    g.rect(card, top(140.0, 4.0, 200.0, 200.0), "Divider", g.t.line, 0.0);
    let grid = g.grid(card, top(190.0, 520.0, 70.0, 70.0), "Category buttons", 470.0, 160.0, 20.0);
    for (c, d) in [("Gameplay", "Difficulty, hints, subtitles"), ("Graphics", "Quality and display"), ("Sound", "Volume and voices"), ("Controls", "Keys and gamepad"), ("Language", "Text and audio"), ("Credits", "The people behind it")] {
        let b = g.card(grid, sz(470.0, 160.0), &format!("{} button", c));
        g.icon(b, cl(24.0, 0.0, 100.0, 100.0), &format!("{} icon", c), &c[..1]);
        g.heading_l(b, tl(150.0, 30.0, 300.0, 50.0), &format!("{} name", c), c, 36.0);
        g.note(b, tl(150.0, 90.0, 300.0, 44.0), &format!("{} text", c), d, 24.0);
    }
    g.button(card, bc(0.0, 40.0, 360.0, 80.0), "Close button", "Close");
}

// ---- Settings options --------------------------------------------------------------------------------------------

/// Audio mixer: one channel per column with a vertical level look (bars) and mute toggles.
fn options_audio(g: &mut G) {
    let p = g.page();
    g.background();
    let win = g.panel(p, cc(0.0, 0.0, 1640.0, 880.0), "Audio panel");
    g.heading_l(win, tl(60.0, 40.0, 900.0, 80.0), "Screen title", "AUDIO MIX", 60.0);
    g.dropdown(win, tr(60.0, 50.0, 420.0, 70.0), "Output dropdown", &["Speakers", "Headphones", "TV"]);
    let row = g.hbox_c(win, top(160.0, 560.0, 60.0, 60.0), "Channels", 30.0);
    for (k, v) in [("Master", 0.9), ("Music", 0.55), ("Crowd", 0.75), ("Commentary", 0.6), ("Effects", 0.8)] {
        let ch = g.card(row, sz(280.0, 560.0), &format!("{} channel", k));
        g.heading(ch, top(24.0, 50.0, 10.0, 10.0), &format!("{} name", k), k, 30.0);
        g.value(ch, top(80.0, 50.0, 30.0, 30.0), &format!("{} value", k), &format!("{}%", (v * 100.0) as i32), 32.0);
        g.bar(ch, top(150.0, 30.0, 30.0, 30.0), &format!("{} level", k), v, g.t.accent);
        g.slider(ch, top(220.0, 30.0, 30.0, 30.0), &format!("{} slider", k), v);
        g.toggle(ch, bottom(30.0, 60.0, 30.0, 30.0), &format!("{} mute", k), "Mute", false);
    }
    g.button2(win, bl(60.0, 40.0, 340.0, 80.0), "Test button", "Play test sound");
    g.button(win, br(60.0, 40.0, 260.0, 80.0), "Done button", "Done");
}

/// Graphics options with «< value >» selectors and a description of the focused option.
fn options_graphics(g: &mut G) {
    let p = g.page();
    g.background();
    screen_header(g, "GRAPHICS", "Changes apply right away");
    let rows = g.vbox_fill(p, tl(60.0, 180.0, 1100.0, 820.0), "Option rows", 12.0);
    for (i, (k, v)) in [("Quality preset", "Custom"), ("Shadows", "High"), ("Reflections", "Ray traced"), ("Bloom", "On"), ("Film grain", "Off"), ("Upscaling", "Quality"), ("Brightness", "50")].iter().enumerate() {
        let row = if i == 2 { g.card_on(rows, sz(1100.0, 100.0), &format!("{} row", k)) } else { g.card(rows, sz(1100.0, 100.0), &format!("{} row", k)) };
        g.label(row, cl(30.0, 0.0, 500.0, 50.0), &format!("{} label", k), k, 30.0);
        g.value(row, cr(30.0, 0.0, 460.0, 50.0), &format!("{} value", k), &format!("<  {}  >", v), 30.0);
    }
    let info = g.panel(p, tr(60.0, 180.0, 620.0, 820.0), "Description panel");
    g.art(info, top(30.0, 330.0, 30.0, 30.0), "Preview image");
    g.heading_l(info, tl(30.0, 390.0, 560.0, 60.0), "Option name", "Reflections", 40.0);
    g.note(info, tl(30.0, 460.0, 560.0, 180.0), "Option text", "Ray traced reflections look best but cost the most. Lower this first if the game stutters.", 28.0);
    g.badge(info, bl(30.0, 40.0, 300.0, 54.0), "Cost tag", "GPU COST: HIGH");
}

/// Accessibility toggles with a live preview of the subtitle style.
fn options_accessibility(g: &mut G) {
    let p = g.page();
    g.background();
    let win = g.panel(p, cc(0.0, 0.0, 1600.0, 900.0), "Accessibility panel");
    g.heading_l(win, tl(60.0, 40.0, 900.0, 80.0), "Screen title", "Accessibility", 56.0);
    let list = g.vbox_fill(win, tl(60.0, 150.0, 700.0, 640.0), "Toggles", 14.0);
    for (i, t) in ["Subtitles", "Speaker names", "Large text", "High contrast", "Reduce motion", "Colorblind filter", "Hold to press"].iter().enumerate() {
        let row = g.card(list, sz(700.0, 78.0), &format!("{} row", t));
        g.toggle(row, cl(24.0, 0.0, 640.0, 54.0), &format!("{} toggle", t), t, i < 3);
    }
    let prev = g.card(win, tr(60.0, 150.0, 720.0, 520.0), "Preview card");
    g.note(prev, tl(30.0, 24.0, 600.0, 40.0), "Preview label", "Preview", 26.0);
    g.art(prev, fill(30.0, 80.0, 30.0, 160.0), "Preview scene");
    let sub = g.panel(prev, bottom(40.0, 90.0, 60.0, 60.0), "Subtitle plate");
    g.heading(sub, full(), "Subtitle", "MIRA: Wait for me by the bridge!", 30.0);
    g.label(win, tr(60.0, 700.0, 720.0, 50.0), "Size label", "Subtitle size", 30.0);
    g.slider(win, tr(60.0, 760.0, 720.0, 30.0), "Size slider", 0.6);
}

// ---- Button layout -----------------------------------------------------------------------------------------------

/// Gamepad picture in the middle with callout labels on both sides.
fn layout_gamepad(g: &mut G) {
    let p = g.page();
    g.background();
    screen_header(g, "CONTROLLER LAYOUT", "Preset: Tactical");
    let pad = g.panel(p, cc(0.0, 60.0, 700.0, 460.0), "Gamepad");
    g.art(pad, fill(30.0, 30.0, 30.0, 30.0), "Gamepad picture");
    for (i, (k, l)) in [("LT", "Aim"), ("LB", "Grenade"), ("LS", "Move / Sprint"), ("D", "Equipment")].iter().enumerate() {
        let row = g.card(p, cl(80.0, -180.0 + i as f32 * 130.0, 440.0, 100.0), &format!("{} callout", l));
        g.key(row, cl(20.0, 0.0, 64.0, 64.0), &format!("{} key", l), k);
        g.label(row, cl(110.0, 0.0, 310.0, 50.0), &format!("{} text", l), l, 30.0);
    }
    for (i, (k, l)) in [("RT", "Fire"), ("RB", "Melee"), ("A", "Jump"), ("X", "Reload")].iter().enumerate() {
        let row = g.card(p, cr(80.0, -180.0 + i as f32 * 130.0, 440.0, 100.0), &format!("{} callout", l));
        g.key(row, cl(20.0, 0.0, 64.0, 64.0), &format!("{} key", l), k);
        g.label(row, cl(110.0, 0.0, 310.0, 50.0), &format!("{} text", l), l, 30.0);
    }
    let foot = g.panel(p, bottom(0.0, 100.0, 0.0, 0.0), "Footer");
    prompts(g, foot, cl(60.0, 0.0, 900.0, 50.0), &[("A", "Remap"), ("Y", "Preset"), ("B", "Back")]);
}

/// Key binding table: action, primary key and secondary key columns.
fn layout_keyboard(g: &mut G) {
    let p = g.page();
    g.background();
    let win = g.panel(p, cc(0.0, 0.0, 1500.0, 920.0), "Bindings panel");
    g.heading_l(win, tl(60.0, 40.0, 900.0, 70.0), "Screen title", "Key bindings", 52.0);
    g.input(win, tr(60.0, 44.0, 420.0, 64.0), "Search field", "Search actions...");
    let head = g.el(win, top(140.0, 50.0, 60.0, 60.0), Kind::Empty, "Table header", |_| {});
    g.note(head, cl(20.0, 0.0, 500.0, 44.0), "Action column", "ACTION", 24.0);
    g.note(head, cl(700.0, 0.0, 300.0, 44.0), "Primary column", "PRIMARY", 24.0);
    g.note(head, cr(20.0, 0.0, 300.0, 44.0), "Secondary column", "SECONDARY", 24.0);
    let rows = g.vbox_fill(win, top(200.0, 580.0, 60.0, 60.0), "Binding rows", 8.0);
    for (a, k1, k2) in [("Move forward", "W", "Up"), ("Move back", "S", "Down"), ("Jump", "Space", "-"), ("Crouch", "Ctrl", "C"), ("Interact", "E", "F"), ("Inventory", "Tab", "I"), ("Map", "M", "-")] {
        let row = g.card(rows, sz(1380.0, 76.0), &format!("{} row", a));
        g.label(row, cl(20.0, 0.0, 600.0, 50.0), &format!("{} label", a), a, 28.0);
        g.button2(row, cl(700.0, 0.0, 220.0, 60.0), &format!("{} primary", a), k1);
        g.button2(row, cr(20.0, 0.0, 220.0, 60.0), &format!("{} secondary", a), k2);
    }
    g.button2(win, bl(60.0, 36.0, 320.0, 76.0), "Reset button", "Reset all");
    g.button(win, br(60.0, 36.0, 260.0, 76.0), "Save button", "Save");
}

/// Three preset cards (Type A, Type B, Custom) with a small diagram each.
fn layout_presets(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 60.0, 1200.0, 100.0), "Screen title", "Pick your controls!", 72.0);
    let row = g.hbox_c(p, cc(0.0, 40.0, 1740.0, 640.0), "Presets", 40.0);
    for (i, (n, d)) in [("Type A", "Jump on A, attack on X"), ("Type B", "Jump on B, attack on Y"), ("Custom", "Make your own layout")].iter().enumerate() {
        let c = if i == 0 { g.card_on(row, sz(540.0, 620.0), &format!("{} card", n)) } else { g.card(row, sz(540.0, 620.0), &format!("{} card", n)) };
        g.heading(c, top(30.0, 70.0, 20.0, 20.0), &format!("{} name", n), n, 48.0);
        let pad = g.art(c, top(120.0, 300.0, 50.0, 50.0), &format!("{} diagram", n));
        let _ = pad;
        g.txt(c, top(450.0, 80.0, 30.0, 30.0), &format!("{} text", n), d, 28.0, g.t.muted, CENTER, false);
        if i == 0 {
            g.badge(c, bc(0.0, 30.0, 220.0, 50.0), "Current tag", "CURRENT");
        }
    }
    g.button(p, bc(0.0, 50.0, 360.0, 90.0), "Confirm button", "Let's go!");
}

// ---- Cutscene & story --------------------------------------------------------------------------------------------

/// Cinematic letterbox: black bars, a subtitle line and a «hold to skip» prompt.
fn cutscene_letterbox(g: &mut G) {
    let p = g.page();
    g.art_c(p, full(), "Scene", g.t.bg2, g.t.accent2);
    let top_bar = g.rect(p, top(0.0, 140.0, 0.0, 0.0), "Top bar", [0, 0, 0, 255], 0.0);
    let _ = top_bar;
    let bottom_bar = g.el(p, bottom(0.0, 200.0, 0.0, 0.0), Kind::Panel, "Bottom bar", |e| e.fill = [0, 0, 0, 255]);
    g.txt(bottom_bar, top(30.0, 50.0, 200.0, 200.0), "Speaker", "ELEANOR", 30.0, g.t.accent, CENTER, true);
    g.txt(bottom_bar, top(84.0, 60.0, 200.0, 200.0), "Subtitle", "You kept the letter all these years?", 38.0, [240, 240, 240, 255], CENTER, false);
    let skip = g.el(p, br(50.0, 220.0, 300.0, 60.0), Kind::Panel, "Skip plate", |e| e.fill = [0, 0, 0, 200]);
    g.txt(skip, full(), "Skip", "Hold B to skip", 26.0, [230, 230, 230, 255], CENTER, false);
}

/// Storybook page: an illustration in a frame, the narration under it and a «next» button.
fn cutscene_storybook(g: &mut G) {
    let p = g.page();
    g.background();
    let page = g.panel(p, cc(0.0, 0.0, 1400.0, 940.0), "Book page");
    g.art(page, top(50.0, 520.0, 80.0, 80.0), "Illustration");
    g.label(page, top(610.0, 160.0, 120.0, 120.0), "Narration", "Long ago, before the rivers learned their names, a lantern was lit in the tallest tower of the kingdom.", 36.0);
    g.note(page, bl(80.0, 50.0, 300.0, 50.0), "Page number", "Page 1 / 6", 28.0);
    g.button(page, br(80.0, 40.0, 260.0, 80.0), "Next button", "Next >");
}

/// Comic strip: three panels with captions, the current one highlighted.
fn cutscene_comic(g: &mut G) {
    let p = g.page();
    g.background();
    let row = g.hbox_c(p, cc(0.0, -40.0, 1800.0, 760.0), "Comic panels", 30.0);
    for (i, c) in ["The city never sleeps.", "Neither does she.", "Tonight, the lights go out."].iter().enumerate() {
        let f = if i == 2 { g.card_on(row, sz(580.0, 740.0), &format!("Panel {}", i + 1)) } else { g.card(row, sz(580.0, 740.0), &format!("Panel {}", i + 1)) };
        g.art(f, fill(20.0, 20.0, 20.0, 140.0), &format!("Panel {} art", i + 1));
        g.heading(f, bottom(30.0, 80.0, 20.0, 20.0), &format!("Caption {}", i + 1), c, 32.0);
    }
    let bar = g.panel(p, bc(0.0, 40.0, 700.0, 80.0), "Prompt plate");
    prompts(g, bar, cc(0.0, 0.0, 640.0, 50.0), &[("A", "Next"), ("X", "Auto"), ("B", "Skip")]);
}

// ---- Dialogue & speech -------------------------------------------------------------------------------------------

/// Classic text box at the bottom with a portrait and a name tag.
fn dialogue_box(g: &mut G) {
    let p = g.page();
    g.scene();
    let bx = g.panel(p, bottom(40.0, 300.0, 80.0, 80.0), "Dialogue box");
    g.art(bx, cl(30.0, 0.0, 240.0, 240.0), "Portrait");
    let name = g.card(bx, tl(300.0, 24.0, 300.0, 60.0), "Name tag");
    g.heading(name, full(), "Speaker name", "OLD MAGE", 32.0);
    g.label(bx, tl(300.0, 100.0, 1300.0, 150.0), "Dialogue text", "The cave to the north hides the second crystal. Take this lantern, and do not wake the guardian.", 36.0);
    g.txt(bx, br(30.0, 20.0, 100.0, 50.0), "Next arrow", "v", 40.0, g.t.accent, CENTER, true);
}

/// Visual-novel layout: big character art, a wide text box and three answer choices.
fn dialogue_novel(g: &mut G) {
    let p = g.page();
    g.background();
    g.art(p, br(160.0, 0.0, 620.0, 980.0), "Character art");
    let choices = g.vbox_fill(p, tl(120.0, 120.0, 820.0, 360.0), "Choices", 20.0);
    for (i, c) in ["Of course I remember!", "Remind me again?", "...I'd rather not talk about it."].iter().enumerate() {
        if i == 0 {
            g.button(choices, sz(820.0, 100.0), &format!("Choice {}", i + 1), c);
        } else {
            g.button2(choices, sz(820.0, 100.0), &format!("Choice {}", i + 1), c);
        }
    }
    let bx = g.panel(p, bottom(40.0, 280.0, 80.0, 80.0), "Text box");
    let tag = g.card(bx, tl(40.0, -0.0, 280.0, 64.0), "Name tag");
    g.heading(tag, full(), "Speaker name", "Hana", 34.0);
    g.label(bx, tl(40.0, 90.0, 1600.0, 140.0), "Dialogue text", "Hey... do you remember the summer festival? You promised you would come back.", 38.0);
    let opts = g.hbox(bx, tr(30.0, 14.0, 420.0, 50.0), "Text options", 12.0);
    for o in ["Log", "Auto", "Skip"] {
        g.button2(opts, sz(130.0, 50.0), &format!("{} button", o), o);
    }
}

/// Conversation choices listed beside the speaker, with the line being said at the top.
fn dialogue_choices(g: &mut G) {
    let p = g.page();
    g.scene();
    let line = g.panel(p, top(60.0, 150.0, 300.0, 300.0), "Line plate");
    g.txt(line, tl(40.0, 20.0, 600.0, 44.0), "Speaker", "CAPTAIN REYES", 28.0, g.t.accent, LEFT, true);
    g.label(line, tl(40.0, 70.0, 1200.0, 60.0), "Line", "We have one shot at the gate. Your call, commander.", 34.0);
    let wheel = g.vbox_fill(p, cr(100.0, 120.0, 720.0, 520.0), "Choices", 16.0);
    for (i, (tag, c)) in [("[Charm]", "We go in quiet."), ("[Force]", "We blow the gate."), ("[Ask]", "What do the scouts say?"), ("[Leave]", "Give me a minute.")].iter().enumerate() {
        let b = if i == 0 { g.card_on(wheel, sz(720.0, 116.0), &format!("Choice {}", i + 1)) } else { g.card(wheel, sz(720.0, 116.0), &format!("Choice {}", i + 1)) };
        g.txt(b, cl(30.0, 0.0, 170.0, 50.0), &format!("Choice {} tag", i + 1), tag, 28.0, g.t.accent, LEFT, true);
        g.label(b, cl(210.0, 0.0, 480.0, 50.0), &format!("Choice {} text", i + 1), c, 30.0);
    }
    let timer = g.panel(p, cl(100.0, 120.0, 360.0, 120.0), "Timer plate");
    g.note(timer, top(14.0, 40.0, 20.0, 20.0), "Timer label", "Decide", 26.0);
    g.bar(timer, bottom(24.0, 26.0, 20.0, 20.0), "Timer bar", 0.55, g.t.accent2);
}

// ---- Guided tutorial ---------------------------------------------------------------------------------------------

/// Dimmed screen with a highlighted frame around one button and a speech bubble explaining it.
fn tutorial_spotlight(g: &mut G) {
    let p = g.page();
    g.background();
    let bar = g.panel(p, bottom(30.0, 160.0, 300.0, 300.0), "Action bar");
    let row = g.hbox_c(bar, full(), "Action buttons", 30.0);
    for a in ["Build", "Plant", "Water", "Sell"] {
        g.button2(row, sz(240.0, 110.0), &format!("{} button", a), a);
    }
    g.dim(150);
    g.el(p, bc(-135.0, 40.0, 280.0, 140.0), Kind::Panel, "Spotlight frame", |e| {
        e.fill = [0, 0, 0, 0];
        e.stroke = [255, 230, 90, 255];
        e.stroke_w = 8.0;
        e.radius = 30.0;
        e.raycast = false;
    });
    let bubble = g.panel(p, bc(-135.0, 230.0, 760.0, 260.0), "Tip bubble");
    g.heading_l(bubble, tl(40.0, 30.0, 680.0, 60.0), "Tip title", "Plant your first seed!", 44.0);
    g.label(bubble, tl(40.0, 100.0, 680.0, 80.0), "Tip text", "Tap Plant, then tap any empty soil tile.", 32.0);
    g.button(bubble, br(30.0, 24.0, 200.0, 64.0), "Next button", "Got it");
    g.txt(bubble, bl(40.0, 30.0, 200.0, 44.0), "Step", "Step 2 of 5", 26.0, g.t.muted, LEFT, false);
}

/// Checklist of tutorial steps in a side panel, with the current step and a progress bar.
fn tutorial_checklist(g: &mut G) {
    let p = g.page();
    g.scene();
    let side = g.panel(p, tr(40.0, 40.0, 620.0, 760.0), "Tutorial panel");
    g.heading_l(side, tl(36.0, 30.0, 540.0, 60.0), "Tutorial title", "Getting started", 42.0);
    g.bar(side, tl(36.0, 106.0, 548.0, 18.0), "Progress bar", 0.4, g.t.accent);
    g.note(side, tl(36.0, 134.0, 548.0, 40.0), "Progress text", "2 of 5 done", 26.0);
    let steps = g.vbox_fill(side, tl(36.0, 200.0, 548.0, 520.0), "Steps", 12.0);
    for (i, s) in ["Look around with the mouse", "Walk with W A S D", "Pick up the torch (E)", "Open the inventory (Tab)", "Light the campfire"].iter().enumerate() {
        let c = if i == 2 { g.card_on(steps, sz(548.0, 92.0), &format!("Step {}", i + 1)) } else { g.card(steps, sz(548.0, 92.0), &format!("Step {}", i + 1)) };
        let mark = if i < 2 { "x" } else { " " };
        g.key(c, cl(20.0, 0.0, 50.0, 50.0), &format!("Step {} check", i + 1), mark);
        g.label(c, cl(90.0, 0.0, 440.0, 50.0), &format!("Step {} text", i + 1), s, 26.0);
    }
    let hint = g.panel(p, bc(0.0, 120.0, 760.0, 100.0), "Hint plate");
    g.key(hint, cl(30.0, 0.0, 64.0, 64.0), "Hint key", "E");
    g.label(hint, cl(120.0, 0.0, 600.0, 50.0), "Hint text", "Press E to pick up the torch", 32.0);
}

/// Modal tutorial card with a picture, a paragraph and page dots.
fn tutorial_pages(g: &mut G) {
    let p = g.page();
    g.background();
    g.dim(140);
    let card = g.panel(p, cc(0.0, 0.0, 1200.0, 860.0), "Tutorial card");
    g.heading(card, top(40.0, 80.0, 40.0, 40.0), "Tutorial title", "Seasons change your garden", 50.0);
    g.art(card, top(140.0, 400.0, 80.0, 80.0), "Tutorial picture");
    g.label(card, top(570.0, 120.0, 100.0, 100.0), "Tutorial text", "Some plants only grow in spring. Check the calendar in the top corner before you plant.", 32.0);
    let dots = g.hbox_c(card, bc(0.0, 60.0, 200.0, 30.0), "Page dots", 16.0);
    for i in 0..4 {
        let c = if i == 1 { g.t.accent } else { g.t.line };
        g.circle(dots, sz(24.0, 24.0), &format!("Dot {}", i + 1), c);
    }
    g.button2(card, bl(40.0, 40.0, 220.0, 76.0), "Back button", "< Back");
    g.button(card, br(40.0, 40.0, 220.0, 76.0), "Next button", "Next >");
}

// ---- Modal & popup -----------------------------------------------------------------------------------------------

/// Confirmation dialog: question, explanation and two buttons (cancel / confirm).
fn modal_confirm(g: &mut G) {
    let p = g.page();
    g.background();
    g.dim(140);
    let win = g.panel(p, cc(0.0, 0.0, 900.0, 480.0), "Dialog window");
    g.heading(win, top(50.0, 70.0, 40.0, 40.0), "Dialog title", "Quit to main menu?", 48.0);
    g.txt(win, top(140.0, 120.0, 80.0, 80.0), "Dialog text", "Progress since the last checkpoint will be lost.", 32.0, g.t.muted, CENTER, false);
    let row = g.hbox_c(win, bottom(50.0, 90.0, 60.0, 60.0), "Buttons", 30.0);
    g.button2(row, sz(360.0, 86.0), "Cancel button", "Cancel");
    g.button(row, sz(360.0, 86.0), "Quit button", "Quit");
}

/// Reward popup: a shiny item in the middle, its name and rarity, and a Claim button.
fn modal_reward(g: &mut G) {
    let p = g.page();
    g.background();
    g.dim(150);
    let win = g.panel(p, cc(0.0, 0.0, 820.0, 760.0), "Reward window");
    g.heading(win, top(40.0, 70.0, 40.0, 40.0), "Reward title", "Treasure found!", 52.0);
    g.circle(win, tc(0.0, 140.0, 300.0, 300.0), "Glow", alpha(g.t.accent, 90));
    g.art(win, tc(0.0, 190.0, 200.0, 200.0), "Item picture");
    g.heading(win, top(470.0, 60.0, 40.0, 40.0), "Item name", "Dragonbone Bow", 40.0);
    g.badge(win, tc(0.0, 540.0, 220.0, 46.0), "Rarity tag", "LEGENDARY");
    g.button(win, bc(0.0, 40.0, 340.0, 86.0), "Claim button", "Claim");
}

/// Warning popup with an icon, a short message and two actions.
fn modal_warning(g: &mut G) {
    let p = g.page();
    g.background();
    g.dim(120);
    let win = g.panel(p, cc(0.0, 0.0, 1000.0, 420.0), "Warning window");
    g.icon(win, cl(50.0, -40.0, 150.0, 150.0), "Warning icon", "!");
    g.heading_l(win, tl(240.0, 60.0, 700.0, 60.0), "Warning title", "Connection lost", 46.0);
    g.note(win, tl(240.0, 140.0, 700.0, 100.0), "Warning text", "We could not reach the match server. Your season progress is safe.", 30.0);
    let row = g.hbox(win, bl(240.0, 50.0, 720.0, 86.0), "Buttons", 30.0);
    g.button(row, sz(340.0, 84.0), "Retry button", "Retry");
    g.button2(row, sz(340.0, 84.0), "Offline button", "Play offline");
}

// ---- Character intro ---------------------------------------------------------------------------------------------

/// Big character art on the left, the name, title, quote and three stat bars on the right.
fn intro_profile(g: &mut G) {
    let p = g.page();
    g.background();
    g.art(p, left(0.0, 900.0, 0.0, 0.0), "Character art");
    let info = g.panel(p, cr(80.0, 0.0, 860.0, 760.0), "Profile panel");
    g.txt(info, tl(50.0, 50.0, 760.0, 50.0), "Character title", "THE STREET PROPHET", 30.0, g.t.accent, LEFT, true);
    g.heading_l(info, tl(50.0, 110.0, 760.0, 110.0), "Character name", "VEX NOVA", 96.0);
    g.note(info, tl(50.0, 240.0, 760.0, 90.0), "Quote", "\"The city talks. I just listen louder.\"", 32.0);
    let stats = g.vbox_fill(info, tl(50.0, 380.0, 760.0, 300.0), "Stats", 20.0);
    for (k, v) in [("Speed", 0.9), ("Power", 0.45), ("Tech", 0.75)] {
        let row = g.el(stats, sz(760.0, 70.0), Kind::Empty, &format!("{} row", k), |_| {});
        g.label(row, cl(0.0, 0.0, 220.0, 50.0), &format!("{} label", k), k, 30.0);
        g.bar(row, cr(0.0, 0.0, 520.0, 28.0), &format!("{} bar", k), v, g.t.accent);
    }
}

/// Lower-left name plate over full art: name, role and a line of lore.
fn intro_nameplate(g: &mut G) {
    let p = g.page();
    g.art(p, full(), "Scene art");
    let plate = g.panel(p, bl(80.0, 100.0, 960.0, 270.0), "Name plate");
    g.rect(plate, left(0.0, 12.0, 0.0, 0.0), "Accent edge", g.t.accent, 0.0);
    g.heading_l(plate, tl(50.0, 30.0, 860.0, 100.0), "Character name", "Sir Aldric Vane", 72.0);
    g.txt(plate, tl(50.0, 130.0, 860.0, 50.0), "Character role", "Knight-Commander of the Ember Guard", 34.0, g.t.accent, LEFT, false);
    g.note(plate, tl(50.0, 190.0, 860.0, 50.0), "Lore", "Has never lost a duel. Has never forgiven a traitor.", 28.0);
}

/// Bold banner across the screen with the name and three trait tags.
fn intro_banner(g: &mut G) {
    let p = g.page();
    g.background();
    g.art(p, cc(0.0, -60.0, 760.0, 760.0), "Character art");
    let banner = g.panel(p, hmid(220.0, 220.0, 0.0, 0.0), "Name banner");
    g.heading(banner, top(20.0, 120.0, 40.0, 40.0), "Character name", "CAPTAIN WHISKERS", 96.0);
    let tags = g.hbox_c(banner, bottom(16.0, 60.0, 40.0, 40.0), "Traits", 20.0);
    for t in ["Brave", "Hungry", "Very fluffy"] {
        g.badge(tags, sz(260.0, 56.0), &format!("{} tag", t), t);
    }
    g.bg_title(p, tc(0.0, 50.0, 1000.0, 80.0), "Joins label", "A NEW FRIEND JOINS!", 56.0);
}

// ---- Feature unlocked --------------------------------------------------------------------------------------------

/// Centred card: «New feature unlocked», an icon, what it does and an OK button.
fn unlock_card(g: &mut G) {
    let p = g.page();
    g.background();
    g.dim(150);
    let win = g.panel(p, cc(0.0, 0.0, 900.0, 640.0), "Unlock window");
    g.txt(win, top(40.0, 50.0, 40.0, 40.0), "Unlock label", "NEW SYSTEM UNLOCKED", 30.0, g.t.accent, CENTER, true);
    g.icon(win, tc(0.0, 110.0, 180.0, 180.0), "Feature icon", "+");
    g.heading(win, top(320.0, 70.0, 40.0, 40.0), "Feature name", "Drone Bay", 54.0);
    g.txt(win, top(400.0, 90.0, 80.0, 80.0), "Feature text", "Build and send drones to scout sectors while you are away.", 30.0, g.t.muted, CENTER, false);
    g.button(win, bc(0.0, 40.0, 300.0, 80.0), "OK button", "Got it");
}

/// Slim banner that slides in at the top with an icon and one line.
fn unlock_toast(g: &mut G) {
    let p = g.page();
    g.scene();
    let toast = g.panel(p, tc(0.0, 50.0, 1000.0, 150.0), "Unlock toast");
    g.icon(toast, cl(30.0, 0.0, 100.0, 100.0), "Toast icon", "!");
    g.heading_l(toast, tl(160.0, 24.0, 800.0, 56.0), "Toast title", "Recipes unlocked!", 40.0);
    g.note(toast, tl(160.0, 84.0, 800.0, 44.0), "Toast text", "You can now cook at any campfire.", 28.0);
    let hint = g.panel(p, br(40.0, 40.0, 360.0, 80.0), "Hint plate");
    prompts(g, hint, cc(0.0, 0.0, 320.0, 50.0), &[("Y", "Open")]);
}

/// Level-up progress screen: what was unlocked at this level and what comes next.
fn unlock_progress(g: &mut G) {
    let p = g.page();
    g.background();
    let win = g.panel(p, cc(0.0, 0.0, 1400.0, 860.0), "Level up window");
    g.heading(win, top(40.0, 100.0, 40.0, 40.0), "Level title", "LEVEL 5!", 80.0);
    g.bar(win, top(160.0, 30.0, 200.0, 200.0), "Level bar", 0.15, g.t.accent);
    g.note(win, top(200.0, 40.0, 200.0, 200.0), "Level text", "150 / 1000 XP to level 6", 28.0);
    let row = g.hbox_c(win, top(280.0, 360.0, 60.0, 60.0), "Unlocks", 40.0);
    for (n, d) in [("Crafting", "Make tools"), ("Fishing rod", "Catch fish"), ("Pet slot", "Adopt a pet")] {
        let c = g.card(row, sz(380.0, 340.0), &format!("{} card", n));
        g.icon(c, tc(0.0, 40.0, 140.0, 140.0), &format!("{} icon", n), &n[..1]);
        g.heading(c, top(200.0, 56.0, 20.0, 20.0), &format!("{} name", n), n, 36.0);
        g.txt(c, top(260.0, 50.0, 20.0, 20.0), &format!("{} text", n), d, 26.0, g.t.muted, CENTER, false);
    }
    g.button(win, bc(0.0, 50.0, 340.0, 90.0), "Continue button", "Continue");
}
