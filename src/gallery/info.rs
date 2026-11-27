//! Gallery: information screens (knowledge, progress and collection).
//!
//! Each type of the "Information" sections of the catalog gets three layouts (A, B, C) in three visual styles.

use super::*;

pub fn entries() -> Vec<Entry> {
    use Style::*;
    vec![
        // Knowledge
        Entry { ty: "Codex & journal", style: Fantasy, build: codex_columns },
        Entry { ty: "Codex & journal", style: Elegant, build: codex_book },
        Entry { ty: "Codex & journal", style: Nature, build: codex_grid },
        Entry { ty: "Tutorials & guides", style: SciFi, build: tutorial_video },
        Entry { ty: "Tutorials & guides", style: Cartoon, build: tutorial_carousel },
        Entry { ty: "Tutorials & guides", style: Minimal, build: tutorial_diagram },
        Entry { ty: "Logs & history", style: Military, build: logs_messages },
        Entry { ty: "Logs & history", style: Horror, build: logs_timeline },
        Entry { ty: "Logs & history", style: Retro, build: logs_recordings },
        // Progress
        Entry { ty: "Area map", style: Nature, build: area_world_map },
        Entry { ty: "Area map", style: Fantasy, build: area_regions },
        Entry { ty: "Area map", style: SciFi, build: area_dungeon },
        Entry { ty: "Missions & quests", style: Military, build: quests_detail },
        Entry { ty: "Missions & quests", style: Sports, build: quests_cards },
        Entry { ty: "Missions & quests", style: Elegant, build: quests_chapters },
        Entry { ty: "Challenges & achievements", style: Sports, build: challenges_list },
        Entry { ty: "Challenges & achievements", style: Retro, build: challenges_badges },
        Entry { ty: "Challenges & achievements", style: Neon, build: challenges_weekly },
        Entry { ty: "Profile & game totals", style: Minimal, build: profile_card },
        Entry { ty: "Profile & game totals", style: Elegant, build: profile_records },
        Entry { ty: "Profile & game totals", style: Neon, build: profile_compare },
        // Collection
        Entry { ty: "Collection & extras hub", style: Pastel, build: hub_tiles },
        Entry { ty: "Collection & extras hub", style: Retro, build: hub_list },
        Entry { ty: "Collection & extras hub", style: Elegant, build: hub_shelves },
        Entry { ty: "Collectables menu", style: Nature, build: collect_grid },
        Entry { ty: "Collectables menu", style: Horror, build: collect_list },
        Entry { ty: "Collectables menu", style: SciFi, build: collect_regions },
        Entry { ty: "Gallery menu", style: Elegant, build: gallery_grid },
        Entry { ty: "Gallery menu", style: Horror, build: gallery_viewer },
        Entry { ty: "Gallery menu", style: Pastel, build: gallery_albums },
        Entry { ty: "Collectable viewer", style: Fantasy, build: collectable_pedestal },
        Entry { ty: "Collectable viewer", style: Cartoon, build: collectable_card },
        Entry { ty: "Collectable viewer", style: SciFi, build: collectable_case },
        Entry { ty: "Sound & music player", style: Neon, build: music_list },
        Entry { ty: "Sound & music player", style: Retro, build: music_jukebox },
        Entry { ty: "Sound & music player", style: SciFi, build: music_equalizer },
    ]
}

// ---------------------------------------------------------------------------------------------------------------
// Knowledge - Codex & journal
// ---------------------------------------------------------------------------------------------------------------

/// Categories column, entry list and the article with a picture: three columns side by side.
fn codex_columns(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(24.0, 92.0, 24.0, 24.0), "Header bar");
    g.heading_l(head, tl(28.0, 22.0, 700.0, 48.0), "Screen title", "CODEX", 40.0);
    g.note(head, tr(28.0, 30.0, 520.0, 32.0), "Header note", "142 of 180 entries found", 24.0);

    let cats = g.panel(p, left(24.0, 300.0, 136.0, 24.0), "Category panel");
    g.heading_l(cats, tl(22.0, 20.0, 256.0, 36.0), "Category heading", "Categories", 28.0);
    g.rect(cats, tl(22.0, 66.0, 256.0, 2.0), "Category divider", g.t.line, 0.0);
    let list = g.vbox_fill(cats, tl(12.0, 82.0, 276.0, 600.0), "Category list", 8.0);
    for (i, c) in ["People", "Places", "Creatures", "Artifacts", "Legends", "Events"].iter().enumerate() {
        g.menu_item(list, sz(276.0, 64.0), &format!("Category {}", c), c, i == 2);
    }

    let entries = g.panel(p, left(344.0, 436.0, 136.0, 24.0), "Entry panel");
    g.heading_l(entries, tl(22.0, 20.0, 392.0, 36.0), "Entry heading", "Creatures: 5 of 12", 28.0);
    g.rect(entries, tl(22.0, 66.0, 392.0, 2.0), "Entry divider", g.t.line, 0.0);
    let rows = g.vbox_fill(entries, tl(12.0, 82.0, 412.0, 640.0), "Entry list", 10.0);
    for (i, e) in ["Marsh Lurker", "Ember Wisp", "Stone Warden", "Glass Moth", "Frost Hound"].iter().enumerate() {
        let r = g.card(rows, sz(412.0, 96.0), &format!("Entry {}", e));
        g.slot(r, tl(14.0, 16.0, 64.0, 64.0), &format!("Entry icon {}", e));
        g.label(r, tl(92.0, 20.0, 300.0, 32.0), &format!("Entry name {}", e), e, 26.0);
        g.note(r, tl(92.0, 54.0, 300.0, 28.0), &format!("Entry state {}", e), if i < 2 { "Read" } else { "Unread" }, 22.0);
    }

    let art = g.panel(p, right(24.0, 1120.0, 136.0, 24.0), "Article panel");
    g.art(art, tl(24.0, 24.0, 1072.0, 380.0), "Article illustration");
    g.heading_l(art, tl(24.0, 424.0, 1072.0, 48.0), "Article title", "Marsh Lurker", 40.0);
    g.note(art, tl(24.0, 480.0, 1072.0, 30.0), "Article tag", "Creature - swamp dweller - threat: low", 24.0);
    g.label(art, tl(24.0, 526.0, 1072.0, 80.0), "Article body", "A pale hunter that waits in still water. It strikes only at dusk, and never near open fire.", 26.0);
    g.rect(art, tl(24.0, 626.0, 1072.0, 2.0), "Article divider", g.t.line, 0.0);
    g.heading_l(art, tl(24.0, 640.0, 1072.0, 34.0), "Related heading", "Related entries", 28.0);
    let related = g.hbox(art, tl(24.0, 684.0, 1072.0, 150.0), "Related list", 16.0);
    for n in ["Ember Wisp", "Frost Hound", "Reed Hut"].iter() {
        let c = g.card(related, sz(320.0, 150.0), &format!("Related card {}", n));
        g.slot(c, tl(12.0, 12.0, 60.0, 60.0), &format!("Related icon {}", n));
        g.note(c, tl(12.0, 80.0, 296.0, 28.0), &format!("Related name {}", n), n, 24.0);
    }
    let btns = g.hbox(art, br(24.0, 24.0, 640.0, 76.0), "Article buttons", 16.0);
    g.button2(btns, sz(280.0, 76.0), "Previous entry button", "< Previous");
    g.button(btns, sz(280.0, 76.0), "Next entry button", "Next >");
}

/// An open book: spine in the middle, a text page and a page with a picture.
fn codex_book(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 20.0, 900.0, 50.0), "Book title", "Journal", 40.0);
    let book = g.panel(p, cc(0.0, 20.0, 1560.0, 900.0), "Open book");
    let left_page = g.card(book, tl(30.0, 30.0, 735.0, 840.0), "Left page");
    let right_page = g.card(book, tr(30.0, 30.0, 735.0, 840.0), "Right page");
    g.rect(book, vmid(0.0, 8.0, 30.0, 30.0), "Spine", g.t.line, 4.0);

    g.heading_l(left_page, tl(28.0, 26.0, 680.0, 44.0), "Chapter title", "Chapter 3: The Long Road", 34.0);
    g.note(left_page, tl(28.0, 76.0, 680.0, 28.0), "Chapter date", "Day 12 - Dusk", 24.0);
    g.rect(left_page, tl(28.0, 116.0, 680.0, 2.0), "Chapter divider", g.t.line, 0.0);
    g.label(left_page, tl(28.0, 134.0, 680.0, 240.0), "Page text", "We left before the fog lifted. The north trail was quiet, but the bridge down the valley had given way, so we followed the river instead.", 26.0);
    let b1 = g.panel(left_page, tl(28.0, 400.0, 660.0, 200.0), "Found box");
    g.heading_l(b1, tl(18.0, 16.0, 620.0, 32.0), "Found heading", "Found on the way", 26.0);
    g.note(b1, tl(18.0, 60.0, 620.0, 28.0), "Found entry 1", "A torn map of the valley", 24.0);
    g.note(b1, tl(18.0, 96.0, 620.0, 28.0), "Found entry 2", "A brass key, unmarked", 24.0);
    let page_nav = g.hbox(left_page, bl(0.0, 24.0, 360.0, 60.0), "Page navigation", 12.0);
    g.button2(page_nav, sz(170.0, 60.0), "Previous page button", "< Page 6");
    g.button2(page_nav, sz(170.0, 60.0), "Next page button", "Page 8 >");

    g.art(right_page, tl(28.0, 28.0, 679.0, 420.0), "Page illustration");
    g.heading_l(right_page, tl(28.0, 472.0, 679.0, 36.0), "Illustration caption", "The river crossing", 28.0);
    let facts = g.vbox_fill(right_page, tl(28.0, 520.0, 679.0, 176.0), "Fact list", 10.0);
    for (n, v) in [("Region", "North valley"), ("Weather", "Fog, light rain"), ("Threat", "Low")].iter() {
        let r = g.card(facts, sz(679.0, 52.0), &format!("Fact {}", n));
        g.note(r, tl(12.0, 12.0, 340.0, 28.0), &format!("Fact name {}", n), n, 26.0);
        g.value(r, tr(12.0, 12.0, 260.0, 28.0), &format!("Fact value {}", n), v, 26.0);
    }
    g.button(right_page, bl(0.0, 24.0, 300.0, 66.0), "Bookmark button", "Bookmark page");
}

/// Grid of found entries with locked silhouettes, the chosen entry detailed on the right.
fn codex_grid(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(24.0, 96.0, 24.0, 24.0), "Header panel");
    g.heading_l(head, tl(28.0, 14.0, 600.0, 40.0), "Screen title", "Bestiary", 36.0);
    g.note(head, tl(28.0, 56.0, 600.0, 28.0), "Header note", "Five of sixteen creatures recorded", 24.0);
    g.bar(head, tr(28.0, 34.0, 360.0, 28.0), "Discovery bar", 0.31, g.t.accent);
    g.badge(head, tr(404.0, 30.0, 96.0, 36.0), "Discovery badge", "31%");

    let grid_panel = g.panel(p, left(24.0, 1120.0, 136.0, 24.0), "Grid panel");
    let grid = g.grid(grid_panel, tl(20.0, 20.0, 1080.0, 672.0), "Creature grid", 196.0, 196.0, 16.0);
    for i in 0..20u8 {
        let c = g.card(grid, sz(196.0, 196.0), &format!("Creature card {}", i + 1));
        g.rect(c, tl(8.0, 8.0, 180.0, 180.0), "Creature picture", [90u8.saturating_add(i * 8), 130u8.saturating_add(i * 6), 80u8.saturating_add(i * 4), 255], 0.0);
        if i >= 5 {
            g.rect(c, full(), "Locked veil", alpha(g.t.panel, 170), 0.0);
            g.icon(c, tl(58.0, 58.0, 80.0, 80.0), "Locked icon", "?");
        }
        g.badge(c, bl(8.0, 8.0, 92.0, 28.0), "Creature number", if i < 5 { "FOUND" } else { "NEW" });
    }
    let nav = g.hbox(grid_panel, bl(20.0, 20.0, 1080.0, 70.0), "Grid navigation", 12.0);
    g.button2(nav, sz(190.0, 70.0), "Previous page button", "< Previous");
    g.note(nav, sz(160.0, 70.0), "Page label", "Page 1 / 4", 28.0);
    g.button2(nav, sz(190.0, 70.0), "Next page button", "Next >");
    g.note(grid_panel, tl(190.0, 704.0, 880.0, 28.0), "Grid note", "Recorded creatures gain detail when you meet them again", 24.0);

    let detail = g.panel(p, right(24.0, 728.0, 136.0, 24.0), "Detail panel");
    g.art(detail, tl(24.0, 24.0, 300.0, 300.0), "Creature portrait");
    g.heading_l(detail, tl(344.0, 24.0, 360.0, 46.0), "Detail title", "Marsh Lurker", 40.0);
    g.note(detail, tl(344.0, 76.0, 360.0, 28.0), "Detail tag", "Creature 01", 24.0);
    for (i, (n, v)) in [("Threat", "Low"), ("Habitat", "Swamp"), ("First seen", "Day 9"), ("Weakness", "Fire")].iter().enumerate() {
        g.note(detail, tl(344.0, 128.0 + i as f32 * 44.0, 180.0, 32.0), &format!("Stat name {}", n), n, 24.0);
        g.value(detail, tl(524.0, 128.0 + i as f32 * 44.0, 180.0, 32.0), &format!("Stat value {}", n), v, 26.0);
    }
    g.rect(detail, tl(24.0, 336.0, 680.0, 2.0), "Detail divider", g.t.line, 0.0);
    g.heading_l(detail, tl(24.0, 352.0, 680.0, 34.0), "Notes heading", "Notes", 28.0);
    g.label(detail, tl(24.0, 396.0, 680.0, 120.0), "Creature notes", "* Waits in still water, invisible\n* Never hunts near open fire\n* Burn the reeds to force it out", 26.0);
    let drops = g.hbox(detail, tl(24.0, 530.0, 680.0, 96.0), "Drop slots", 14.0);
    for i in 0..4 {
        let sl = g.slot(drops, sz(96.0, 96.0), &format!("Drop slot {}", i + 1));
        g.note(sl, tl(4.0, 34.0, 88.0, 26.0), &format!("Drop label {}", i + 1), "???", 22.0);
    }
    let detail_btns = g.vbox_fill(detail, bl(24.0, 24.0, 680.0, 156.0), "Detail buttons", 12.0);
    g.button(detail_btns, sz(680.0, 72.0), "Mark as read button", "Mark as read");
    g.button2(detail_btns, sz(680.0, 72.0), "Share entry button", "Share entry");
}

// ---------------------------------------------------------------------------------------------------------------
// Knowledge - Tutorials & guides
// ---------------------------------------------------------------------------------------------------------------

/// Topic list on the left, a video player in the middle and the written steps under it.
fn tutorial_video(g: &mut G) {
    let p = g.page();
    g.background();
    let left_panel = g.panel(p, left(24.0, 420.0, 24.0, 24.0), "Topic panel");
    g.heading_l(left_panel, tl(20.0, 20.0, 380.0, 36.0), "Topic heading", "Topics", 28.0);
    let topics = g.vbox_fill(left_panel, tl(12.0, 70.0, 396.0, 490.0), "Topic list", 8.0);
    for n in ["Moving", "Combat basics", "Inventory", "Crafting", "Stealth", "Maps", "Trading"].iter() {
        g.menu_item(topics, sz(396.0, 62.0), &format!("Topic {}", n), n, *n == "Combat basics");
    }
    let progress_card = g.card(left_panel, bl(12.0, 20.0, 396.0, 200.0), "Progress card");
    g.heading_l(progress_card, tl(16.0, 14.0, 360.0, 32.0), "Progress heading", "Your progress", 26.0);
    g.note(progress_card, tl(16.0, 54.0, 200.0, 28.0), "Progress label", "Guides read", 24.0);
    g.value(progress_card, tr(16.0, 54.0, 120.0, 28.0), "Progress value", "3 / 8", 24.0);
    g.bar(progress_card, tl(16.0, 96.0, 364.0, 20.0), "Progress bar", 0.375, g.t.accent);
    g.note(progress_card, tl(16.0, 128.0, 364.0, 56.0), "Progress note", "Finish a guide to unlock the next one", 22.0);

    let main = g.panel(p, right(24.0, 1428.0, 24.0, 24.0), "Main panel");
    g.heading_l(main, tl(28.0, 24.0, 900.0, 42.0), "Video title", "Combat basics", 36.0);
    g.badge(main, tl(1060.0, 24.0, 280.0, 36.0), "Video tag", "Guide 02 / 08");
    let player = g.card(main, tl(28.0, 84.0, 1372.0, 560.0), "Video player");
    g.art(player, tl(12.0, 12.0, 1348.0, 476.0), "Video frame");
    g.icon(player, cc(0.0, -38.0, 140.0, 140.0), "Play icon", ">");
    g.bar(player, tl(24.0, 508.0, 1324.0, 16.0), "Playback bar", 0.35, g.t.accent);
    let controls = g.hbox(player, tl(24.0, 530.0, 700.0, 20.0), "Playback labels", 24.0);
    g.note(controls, sz(160.0, 20.0), "Time now", "0:52", 22.0);
    g.note(controls, sz(120.0, 20.0), "Time total", "2:30", 22.0);

    let steps = g.panel(main, tl(28.0, 664.0, 1372.0, 304.0), "Step panel");
    g.heading_l(steps, tl(24.0, 16.0, 600.0, 34.0), "Steps heading", "In this guide", 28.0);
    let cols = g.hbox(steps, tl(20.0, 60.0, 1332.0, 220.0), "Step columns", 20.0);
    for i in 0..3usize {
        let c = g.card(cols, sz(424.0, 220.0), &format!("Step {}", i + 1));
        g.icon(c, tl(18.0, 18.0, 48.0, 48.0), &format!("Step number {}", i + 1), &format!("{}", i + 1));
        let text = match i {
            0 => "Tap attack to land quick hits",
            1 => "Hold back to raise your guard",
            _ => "Dodge after the third strike",
        };
        g.label(c, tl(18.0, 80.0, 388.0, 100.0), &format!("Step text {}", i + 1), text, 26.0);
    }
    let foot = g.hbox(main, bl(28.0, 24.0, 1372.0, 76.0), "Footer buttons", 16.0);
    g.button2(foot, sz(320.0, 76.0), "Back button", "< Back");
    g.button2(foot, sz(320.0, 76.0), "Replay button", "Replay");
    g.button(foot, sz(320.0, 76.0), "Next guide button", "Next guide >");
}

/// A carousel of tip cards: the middle card is bigger, page dots and arrows under it.
fn tutorial_carousel(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(40.0, 110.0, 260.0, 260.0), "Header plate");
    g.heading(head, tl(0.0, 20.0, 1400.0, 46.0), "Carousel title", "Quick tips", 40.0);
    g.txt(head, tl(0.0, 70.0, 1400.0, 30.0), "Carousel subtitle", "Swipe or use the arrows", 24.0, g.t.muted, CENTER, false);

    let row = g.hbox_c(p, hmid(20.0, 620.0, 60.0, 60.0), "Tip carousel", 24.0);
    let tips = [
        ("Rest often", "Sleeping at a camp restores everything, but the world moves on."),
        ("Share water", "Every teammate can carry one flask. Ask before you drink."),
        ("Mark the trail", "A single flag is enough to find your way back at night."),
        ("Check the sky", "Storms arrive by afternoon. Build before it rains."),
        ("Read the signs", "Painted stones always point to the nearest safe path."),
    ];
    for (i, (t, d)) in tips.iter().enumerate() {
        let mid = i == 2;
        let (w, h) = if mid { (460.0, 520.0) } else { (360.0, 440.0) };
        let c = if mid { g.card_on(row, sz(w, h), &format!("Tip card {}", t)) } else { g.card(row, sz(w, h), &format!("Tip card {}", t)) };
        g.art(c, tl(20.0, 20.0, w - 40.0, h * 0.42), &format!("Tip picture {}", t));
        g.heading_l(c, tl(20.0, h * 0.42 + 34.0, w - 40.0, 38.0), &format!("Tip title {}", t), t, if mid { 32.0 } else { 28.0 });
        g.note(c, tl(20.0, h * 0.42 + 86.0, w - 40.0, h * 0.42), &format!("Tip text {}", t), d, 24.0);
    }

    let dots = g.hbox_c(p, bc(0.0, 200.0, 400.0, 24.0), "Page dots", 16.0);
    for i in 0..5 {
        g.circle(dots, sz(20.0, 20.0), &format!("Dot {}", i + 1), if i == 2 { g.t.accent } else { g.t.muted });
    }
    let arrows = g.hbox_c(p, bc(0.0, 110.0, 520.0, 76.0), "Arrow buttons", 24.0);
    g.button2(arrows, sz(240.0, 76.0), "Previous tip button", "< Back");
    g.button(arrows, sz(240.0, 76.0), "Next tip button", "Next >");
    let foot = g.hbox(p, bl(60.0, 40.0, 600.0, 40.0), "Footer prompts", 14.0);
    g.key(foot, sz(52.0, 52.0), "Back key", "B");
    g.note(foot, sz(200.0, 52.0), "Back label", "Close", 26.0);
}

/// A controller picture in the middle with numbered callouts and the full list of controls beside it.
fn tutorial_diagram(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 40.0, 1200.0, 60.0), "Screen title", "Controls guide", 48.0);
    g.bg_label(p, tc(0.0, 106.0, 1200.0, 34.0), "Screen subtitle", "Everything you can press, in one place", 26.0, CENTER);

    let pad = g.panel(p, left(80.0, 1000.0, 170.0, 80.0), "Diagram panel");
    g.art(pad, cc(0.0, 30.0, 860.0, 540.0), "Controller picture");
    let calls = [
        (140.0, 140.0, "1", "Left stick"),
        (280.0, 380.0, "2", "D-pad"),
        (500.0, 120.0, "3", "Face buttons"),
        (680.0, 380.0, "4", "Triggers"),
    ];
    for (x, y, n, label) in calls.iter() {
        g.icon(pad, tl(*x, *y, 56.0, 56.0), &format!("Callout {} pin", n), n);
        let plate = g.card(pad, tl(*x - 40.0, *y + 64.0, 260.0, 56.0), &format!("Callout {} plate", n));
        g.txt(plate, full(), &format!("Callout {} text", n), label, 24.0, g.t.text, CENTER, false);
    }
    let legend = g.hbox(pad, bl(24.0, 24.0, 900.0, 60.0), "Legend", 18.0);
    g.icon(legend, sz(40.0, 40.0), "Legend icon", "+");
    g.note(legend, sz(560.0, 60.0), "Legend text", "Hold a button to open the radial menu", 24.0);

    let list = g.panel(p, right(80.0, 680.0, 170.0, 80.0), "Control panel");
    g.heading_l(list, tl(24.0, 20.0, 632.0, 36.0), "Control heading", "All controls", 28.0);
    let rows = g.vbox_fill(list, tl(16.0, 68.0, 648.0, 604.0), "Control list", 8.0);
    for (n, k) in [("Move", "Stick"), ("Sprint", "L3"), ("Attack", "X"), ("Guard", "L2"), ("Inventory", "Options"), ("Map", "Pad"), ("Emote", "D-pad"), ("Pause", "Start")].iter() {
        let r = g.card(rows, sz(648.0, 68.0), &format!("Control row {}", n));
        g.label(r, tl(16.0, 18.0, 300.0, 32.0), &format!("Control name {}", n), n, 26.0);
        g.key(r, tr(16.0, 12.0, 180.0, 44.0), &format!("Control key {}", n), k);
    }
    let foot = g.hbox(p, br(80.0, 30.0, 560.0, 80.0), "Footer buttons", 16.0);
    g.button2(foot, sz(260.0, 76.0), "Remap button", "Remap keys");
    g.button(foot, sz(260.0, 76.0), "Done button", "Done");
}

// ---------------------------------------------------------------------------------------------------------------
// Knowledge - Logs & history
// ---------------------------------------------------------------------------------------------------------------

/// A message log with time stamps and channel tags, filter chips above it.
fn logs_messages(g: &mut G) {
    let p = g.page();
    g.background();
    let win = g.panel(p, cc(0.0, 0.0, 1400.0, 960.0), "Log window");
    g.heading_l(win, tl(28.0, 22.0, 700.0, 42.0), "Window title", "Mission log", 36.0);
    let filters = g.hbox(win, tr(28.0, 28.0, 560.0, 44.0), "Filter chips", 12.0);
    for f in ["All", "Squad", "Command", "System"].iter() {
        g.badge(filters, sz(130.0, 44.0), &format!("Filter {}", f), f);
    }
    g.rect(win, tl(28.0, 84.0, 1344.0, 2.0), "Window divider", g.t.line, 0.0);
    let rows = g.vbox_fill(win, tl(20.0, 100.0, 1360.0, 736.0), "Message list", 8.0);
    let msgs = [
        ("08:14", "Command", "Push east to the relay tower."),
        ("08:16", "Squad", "Copy. Moving in two groups."),
        ("08:22", "System", "Checkpoint saved."),
        ("08:31", "Command", "Hold the bridge until noon."),
        ("08:40", "Squad", "We need more ammo up front."),
        ("08:41", "System", "Supply drop marked on the map."),
        ("08:55", "Command", "Enemy armor spotted north."),
        ("09:02", "Squad", "Tower secured. Awaiting orders."),
    ];
    for (t, ch, m) in msgs.iter() {
        let r = g.card(rows, sz(1360.0, 88.0), &format!("Message at {}", t));
        g.value(r, tl(12.0, 28.0, 110.0, 32.0), &format!("Stamp {}", t), t, 24.0);
        g.badge(r, tl(136.0, 26.0, 170.0, 36.0), &format!("Channel {}", t), ch);
        g.label(r, tl(330.0, 28.0, 1000.0, 32.0), &format!("Text {}", t), m, 26.0);
    }
    let foot = g.hbox(win, bl(28.0, 28.0, 1344.0, 76.0), "Log footer", 16.0);
    g.button2(foot, sz(280.0, 76.0), "Export button", "Export log");
    g.input(foot, sz(760.0, 76.0), "Search field", "Search the log...");
    g.button(foot, sz(240.0, 76.0), "Close button", "Close");
}

/// A vertical timeline: markers on a rail, each event in a card on the right.
fn logs_timeline(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 40.0, 1100.0, 60.0), "Screen title", "History of the valley", 48.0);
    g.bg_label(p, tc(0.0, 106.0, 1100.0, 32.0), "Screen subtitle", "Seven events recorded", 26.0, CENTER);

    let panel = g.panel(p, vmid(0.0, 1360.0, 170.0, 60.0), "Timeline panel");
    g.rect(panel, tl(120.0, 30.0, 6.0, 780.0), "Timeline rail", g.t.line, 3.0);
    let events = [
        ("Day 1", "Arrival", "We reached the valley before nightfall."),
        ("Day 2", "First camp", "Built a shelter near the old mill."),
        ("Day 4", "The hermit", "A stranger traded herbs for bread."),
        ("Day 6", "Bridge collapse", "The north road is closed for now."),
        ("Day 8", "Storm", "Two days of rain, no travel."),
        ("Day 10", "The watchtower", "Lights seen on the far ridge."),
        ("Day 12", "Departure", "Left for the river crossing."),
    ];
    for (i, (d, t, s)) in events.iter().enumerate() {
        let y = 40.0 + i as f32 * 112.0;
        g.circle(panel, tl(97.0, y + 2.0, 52.0, 52.0), &format!("Timeline dot {}", i + 1), g.t.accent);
        let c = g.card(panel, tl(190.0, y - 8.0, 1100.0, 96.0), &format!("Timeline card {}", d));
        g.note(c, tl(20.0, 16.0, 140.0, 30.0), &format!("Event day {}", d), d, 24.0);
        g.heading_l(c, tl(170.0, 16.0, 600.0, 32.0), &format!("Event title {}", t), t, 28.0);
        g.note(c, tl(170.0, 52.0, 900.0, 28.0), &format!("Event text {}", t), s, 24.0);
    }
    let foot = g.hbox(panel, bl(190.0, 24.0, 1100.0, 70.0), "Timeline footer", 16.0);
    g.note(foot, sz(600.0, 70.0), "Footer note", "Older events are in the archive", 24.0);
    g.button2(foot, sz(260.0, 70.0), "Archive button", "Open archive");
}

/// A list of recordings on the left and a player with a wave placeholder on the right.
fn logs_recordings(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(24.0, 96.0, 24.0, 24.0), "Header bar");
    g.heading_l(head, tl(28.0, 16.0, 700.0, 40.0), "Screen title", "Voice archive", 36.0);
    g.note(head, tl(28.0, 58.0, 700.0, 28.0), "Header note", "9 recordings found", 24.0);
    g.input(head, tr(28.0, 20.0, 420.0, 56.0), "Search field", "Search recordings...");

    let list_panel = g.panel(p, left(24.0, 720.0, 136.0, 24.0), "Recording panel");
    let rows = g.vbox_fill(list_panel, tl(16.0, 16.0, 688.0, 800.0), "Recording list", 10.0);
    for i in 0..8usize {
        let active = i == 2;
        let c = if active { g.card_on(rows, sz(688.0, 92.0), &format!("Recording {}", i + 1)) } else { g.card(rows, sz(688.0, 92.0), &format!("Recording {}", i + 1)) };
        g.icon(c, tl(14.0, 18.0, 56.0, 56.0), &format!("Recording icon {}", i + 1), ">");
        g.label(c, tl(84.0, 16.0, 380.0, 30.0), &format!("Recording name {}", i + 1), "Log entry", 26.0);
        g.note(c, tl(84.0, 50.0, 380.0, 28.0), &format!("Recording stamp {}", i + 1), "Day 4 - 7:20", 22.0);
        g.value(c, tr(14.0, 30.0, 120.0, 30.0), &format!("Recording length {}", i + 1), "1:12", 24.0);
    }

    let player = g.panel(p, right(24.0, 1128.0, 136.0, 24.0), "Player panel");
    g.art(player, tl(24.0, 24.0, 1080.0, 420.0), "Player picture");
    g.heading_l(player, tl(24.0, 468.0, 1080.0, 44.0), "Player title", "Log entry 3: The hermit", 36.0);
    g.note(player, tl(24.0, 520.0, 1080.0, 30.0), "Player subtitle", "Recorded on day 4 - Speaker unknown", 24.0);
    let wave = g.hbox(player, tl(24.0, 580.0, 1080.0, 120.0), "Wave bars", 6.0);
    for i in 0..18usize {
        let h = 30.0 + ((i * 37) % 86) as f32;
        g.rect(wave, sz(52.0, h), &format!("Wave bar {}", i + 1), if i < 7 { g.t.accent } else { g.t.muted }, 2.0);
    }
    g.bar(player, tl(24.0, 720.0, 1080.0, 16.0), "Playback bar", 0.4, g.t.accent);
    let controls = g.hbox(player, tl(24.0, 760.0, 1080.0, 84.0), "Player controls", 16.0);
    g.button2(controls, sz(140.0, 84.0), "Rewind button", "-15");
    g.button(controls, sz(180.0, 84.0), "Play button", "Play");
    g.button2(controls, sz(140.0, 84.0), "Forward button", "+15");
    g.button2(controls, sz(240.0, 84.0), "Transcript button", "Transcript");
    g.note(player, tl(24.0, 864.0, 1080.0, 28.0), "Player note", "Transcript available for 5 of 9 recordings", 24.0);
}

// ---------------------------------------------------------------------------------------------------------------
// Progress - Area map
// ---------------------------------------------------------------------------------------------------------------

/// A world map with numbered nodes joined by a path, the level details in a side panel.
fn area_world_map(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(24.0, 90.0, 24.0, 24.0), "Header bar");
    g.heading_l(head, tl(28.0, 22.0, 700.0, 40.0), "Screen title", "Valley map", 36.0);
    g.note(head, tl(28.0, 22.0, 760.0, 40.0), "Header note", "Six stops on the road east", 24.0);

    let map = g.panel(p, left(24.0, 1300.0, 130.0, 24.0), "Map panel");
    g.art(map, tl(20.0, 20.0, 1260.0, 880.0), "Map terrain");
    let nodes = [(200.0, 700.0), (400.0, 520.0), (620.0, 640.0), (820.0, 420.0), (1040.0, 540.0), (300.0, 300.0)];
    for i in 0..5usize {
        let (x1, y1) = nodes[i];
        let (x2, y2) = nodes[i + 1];
        let (lx, ly) = if x1 < x2 { (x1, y1) } else { (x2, y2) };
        g.rect(map, tl(20.0 + lx, 20.0 + ly, (x2 - x1).abs() + 8.0, (y2 - y1).abs() + 8.0), &format!("Path segment {}", i + 1), alpha(g.t.bg_text, 90), 4.0);
    }
    for (i, (x, y)) in nodes.iter().enumerate() {
        g.icon(map, tl(20.0 + x - 30.0, 20.0 + y - 30.0, 64.0, 64.0), &format!("Map node {}", i + 1), &format!("{}", i + 1));
    }
    let legend = g.card(map, bl(20.0, 20.0, 420.0, 210.0), "Map legend");
    g.heading_l(legend, tl(18.0, 14.0, 380.0, 32.0), "Legend heading", "Legend", 26.0);
    for (i, l) in ["Cleared", "Current", "Undiscovered"].iter().enumerate() {
        g.icon(legend, tl(18.0, 56.0 + i as f32 * 46.0, 32.0, 32.0), &format!("Legend dot {}", i + 1), "+");
        g.note(legend, tl(64.0, 58.0 + i as f32 * 46.0, 320.0, 28.0), &format!("Legend label {}", i + 1), l, 24.0);
    }

    let side = g.panel(p, right(24.0, 548.0, 130.0, 24.0), "Level panel");
    g.art(side, tl(24.0, 24.0, 500.0, 240.0), "Level picture");
    g.heading_l(side, tl(24.0, 288.0, 500.0, 44.0), "Level name", "Old Mill Crossing", 36.0);
    g.note(side, tl(24.0, 340.0, 500.0, 30.0), "Level type", "Stage 2-3 - River route", 24.0);
    let info = g.vbox_fill(side, tl(24.0, 388.0, 500.0, 230.0), "Level info", 10.0);
    for (n, v) in [("Best time", "1:42"), ("Collectables", "4 / 7"), ("Rank", "B")].iter() {
        let r = g.card(info, sz(500.0, 70.0), &format!("Level info row {}", n));
        g.note(r, tl(16.0, 22.0, 240.0, 30.0), &format!("Level info name {}", n), n, 26.0);
        g.value(r, tr(16.0, 22.0, 200.0, 30.0), &format!("Level info value {}", n), v, 26.0);
    }
    let btns = g.vbox_fill(side, bl(24.0, 24.0, 500.0, 232.0), "Level buttons", 12.0);
    g.button(btns, sz(500.0, 96.0), "Travel button", "Travel here");
    let back = g.hbox(side, bl(210.0, 24.0, 500.0, 64.0), "Back row", 12.0);
    g.button2(back, sz(460.0, 64.0), "Back to menu button", "< Back to menu");
}

/// Regions listed with their completion on the left; the region map with soft markers on the right.
fn area_regions(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 30.0, 1100.0, 56.0), "Screen title", "Regions", 48.0);
    g.bg_label(p, tc(0.0, 92.0, 1100.0, 32.0), "Screen subtitle", "Overall completion 63 percent", 26.0, CENTER);

    let list = g.panel(p, left(40.0, 700.0, 150.0, 40.0), "Region panel");
    g.heading_l(list, tl(24.0, 20.0, 660.0, 36.0), "Region heading", "Areas", 28.0);
    let rows = g.vbox_fill(list, tl(16.0, 68.0, 668.0, 640.0), "Region list", 10.0);
    let regions = [("Harbor", "92%", 0.92), ("Lowlands", "74%", 0.74), ("Old Woods", "58%", 0.58), ("North Pass", "34%", 0.34), ("Salt Flats", "12%", 0.12)];
    for (n, pc, v) in regions.iter() {
        let c = g.card(rows, sz(668.0, 116.0), &format!("Region card {}", n));
        g.heading_l(c, tl(18.0, 14.0, 360.0, 34.0), &format!("Region name {}", n), n, 28.0);
        g.bar(c, tl(18.0, 60.0, 520.0, 20.0), &format!("Region bar {}", n), *v, g.t.accent);
        g.value(c, tr(18.0, 14.0, 120.0, 34.0), &format!("Region percent {}", n), pc, 28.0);
        g.note(c, tl(18.0, 86.0, 500.0, 26.0), &format!("Region note {}", n), "Chests, camps and pages", 22.0);
    }

    let map = g.panel(p, right(40.0, 1100.0, 150.0, 40.0), "Map panel");
    g.art(map, tl(24.0, 24.0, 1052.0, 700.0), "Region map");
    for i in 0..5usize {
        let x = 120.0 + i as f32 * 200.0;
        let y = 140.0 + ((i * 157) % 420) as f32;
        g.circle(map, tl(24.0 + x, 24.0 + y, 40.0, 40.0), &format!("Region marker {}", i + 1), g.t.accent);
        let tag = g.card(map, tl(24.0 + x + 46.0, 24.0 + y - 8.0, 240.0, 56.0), &format!("Region tag {}", i + 1));
        g.note(tag, full(), &format!("Region tag text {}", i + 1), regions[i].0, 24.0);
    }
    let foot = g.hbox(map, bl(24.0, 24.0, 1052.0, 76.0), "Map footer", 16.0);
    g.button2(foot, sz(300.0, 76.0), "Travel to region button", "Travel");
    g.button(foot, sz(300.0, 76.0), "Mark region button", "Mark on map");
}

/// A dungeon made of rooms you walk through, with a floor selector above.
fn area_dungeon(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(24.0, 96.0, 24.0, 24.0), "Header panel");
    g.heading_l(head, tl(28.0, 14.0, 600.0, 40.0), "Screen title", "The Undercroft", 36.0);
    g.note(head, tl(28.0, 56.0, 600.0, 28.0), "Header note", "Floor 2 - 6 rooms explored", 24.0);
    let floors = g.hbox(head, tr(28.0, 16.0, 560.0, 64.0), "Floor selector", 12.0);
    for i in 0..4usize {
        g.button2(floors, sz(128.0, 64.0), &format!("Floor button {}", i + 1), &format!("B{}", i + 1));
    }

    let plan = g.panel(p, fill(24.0, 136.0, 484.0, 24.0), "Dungeon plan");
    let rooms = [
        (80.0, 400.0, 220.0, 180.0, "Stairs"),
        (360.0, 400.0, 260.0, 180.0, "Hall"),
        (680.0, 120.0, 240.0, 200.0, "Library"),
        (680.0, 400.0, 240.0, 180.0, "Guard room"),
        (980.0, 400.0, 260.0, 180.0, "Shrine"),
        (980.0, 660.0, 260.0, 200.0, "Vault"),
        (360.0, 100.0, 240.0, 200.0, "Cells"),
    ];
    for (i, (x, y, w, h, n)) in rooms.iter().enumerate() {
        let c = if i == 3 { g.card_on(plan, tl(*x, *y, *w, *h), &format!("Room {}", n)) } else { g.card(plan, tl(*x, *y, *w, *h), &format!("Room {}", n)) };
        g.heading(c, tl(0.0, *h / 2.0 - 18.0, *w, 32.0), &format!("Room name {}", n), n, 26.0);
        if i == 5 {
            g.badge(c, tl(*w / 2.0 - 50.0, *h - 44.0, 100.0, 32.0), &format!("Room tag {}", n), "LOCKED");
        }
    }
    g.rect(plan, tl(300.0, 488.0, 60.0, 12.0), "Corridor 1", alpha(g.t.line, 200), 4.0);
    g.rect(plan, tl(860.0, 488.0, 120.0, 12.0), "Corridor 2", alpha(g.t.line, 200), 4.0);
    g.rect(plan, tl(620.0, 300.0, 12.0, 100.0), "Corridor 3", alpha(g.t.line, 200), 4.0);
    g.rect(plan, tl(1110.0, 580.0, 12.0, 80.0), "Corridor 4", alpha(g.t.line, 200), 4.0);

    let side = g.panel(p, right(24.0, 436.0, 136.0, 24.0), "Room panel");
    g.heading_l(side, tl(20.0, 18.0, 396.0, 34.0), "Room heading", "Guard room", 28.0);
    g.art(side, tl(20.0, 62.0, 396.0, 240.0), "Room picture");
    g.note(side, tl(20.0, 316.0, 396.0, 28.0), "Room state", "Explored - 2 chests left", 24.0);
    let facts = g.vbox_fill(side, tl(20.0, 356.0, 396.0, 260.0), "Room facts", 10.0);
    for (n, v) in [("Enemies", "3"), ("Traps", "1"), ("Key", "Found")].iter() {
        let r = g.card(facts, sz(396.0, 72.0), &format!("Room fact {}", n));
        g.note(r, tl(14.0, 22.0, 200.0, 30.0), &format!("Room fact name {}", n), n, 24.0);
        g.value(r, tr(14.0, 22.0, 160.0, 30.0), &format!("Room fact value {}", n), v, 26.0);
    }
    g.button(side, bl(20.0, 20.0, 396.0, 84.0), "Enter room button", "Enter room");
    g.button2(side, bl(20.0, 116.0, 396.0, 72.0), "Leave dungeon button", "Leave dungeon");
}

// ---------------------------------------------------------------------------------------------------------------
// Progress - Missions & quests
// ---------------------------------------------------------------------------------------------------------------

/// Quest categories | quest list | selected quest with objectives and rewards.
fn quests_detail(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(24.0, 92.0, 24.0, 24.0), "Header bar");
    g.heading_l(head, tl(28.0, 22.0, 700.0, 40.0), "Screen title", "Quest log", 36.0);
    g.note(head, tr(28.0, 30.0, 460.0, 32.0), "Header note", "3 quests tracked", 24.0);

    let cats = g.panel(p, left(24.0, 300.0, 136.0, 24.0), "Category panel");
    g.heading_l(cats, tl(22.0, 20.0, 256.0, 34.0), "Category heading", "Track", 28.0);
    let cat_list = g.vbox_fill(cats, tl(12.0, 66.0, 276.0, 520.0), "Category list", 8.0);
    for (i, c) in ["Main story", "Side quests", "Contracts", "Bounties", "Events", "Completed"].iter().enumerate() {
        g.menu_item(cat_list, sz(276.0, 64.0), &format!("Category {}", c), c, i == 0);
    }
    let counter = g.card(cats, bl(12.0, 20.0, 276.0, 160.0), "Count card");
    g.note(counter, tl(16.0, 16.0, 244.0, 28.0), "Count label", "Active", 24.0);
    g.heading(counter, tl(16.0, 48.0, 244.0, 48.0), "Count value", "3", 44.0);
    g.note(counter, tl(16.0, 108.0, 244.0, 28.0), "Count note", "No limit reached", 22.0);

    let list = g.panel(p, left(344.0, 500.0, 136.0, 24.0), "Quest panel");
    g.heading_l(list, tl(22.0, 20.0, 456.0, 34.0), "Quest heading", "Main story", 28.0);
    let rows = g.vbox_fill(list, tl(12.0, 66.0, 476.0, 640.0), "Quest list", 10.0);
    for (i, (n, d)) in [("The river crossing", "Medium"), ("Lights on the ridge", "Long"), ("The hermit's map", "Short"), ("Papers at the mill", "Short")].iter().enumerate() {
        let c = if i == 0 { g.card_on(rows, sz(476.0, 140.0), &format!("Quest {}", n)) } else { g.card(rows, sz(476.0, 140.0), &format!("Quest {}", n)) };
        g.heading_l(c, tl(18.0, 16.0, 440.0, 34.0), &format!("Quest name {}", n), n, 28.0);
        g.note(c, tl(18.0, 58.0, 440.0, 28.0), &format!("Quest tag {}", n), "Main story - level 4", 22.0);
        g.badge(c, tl(18.0, 96.0, 140.0, 32.0), &format!("Quest length {}", n), d);
    }

    let detail = g.panel(p, right(24.0, 1000.0, 136.0, 24.0), "Detail panel");
    g.heading_l(detail, tl(26.0, 22.0, 700.0, 44.0), "Detail title", "The river crossing", 36.0);
    g.note(detail, tl(26.0, 72.0, 940.0, 30.0), "Detail description", "The old crossing is gone. Find another way east before nightfall.", 24.0);
    g.rect(detail, tl(26.0, 118.0, 948.0, 2.0), "Detail divider", g.t.line, 0.0);
    g.heading_l(detail, tl(26.0, 132.0, 600.0, 32.0), "Objective heading", "Objectives", 28.0);
    let objs = g.vbox_fill(detail, tl(20.0, 176.0, 960.0, 244.0), "Objective list", 8.0);
    for (i, o) in ["Scout the river bank", "Find the ferryman", "Cross with the cart", "Report to the mill"].iter().enumerate() {
        let r = g.card(objs, sz(960.0, 56.0), &format!("Objective {}", i + 1));
        g.note(r, tl(16.0, 12.0, 40.0, 32.0), &format!("Objective tick {}", i + 1), if i < 2 { "x" } else { ">" }, 28.0);
        g.label(r, tl(64.0, 12.0, 860.0, 32.0), &format!("Objective text {}", i + 1), o, 26.0);
    }
    g.heading_l(detail, tl(26.0, 436.0, 600.0, 32.0), "Reward heading", "Rewards", 28.0);
    let rewards = g.hbox(detail, tl(26.0, 480.0, 948.0, 120.0), "Reward list", 16.0);
    for (i, (a, b)) in [("Coins", "320"), ("Charm", "1"), ("XP", "900")].iter().enumerate() {
        let r = g.card(rewards, sz(300.0, 120.0), &format!("Reward {}", a));
        g.slot(r, tl(16.0, 30.0, 60.0, 60.0), &format!("Reward icon {}", i + 1));
        g.note(r, tl(88.0, 30.0, 190.0, 28.0), &format!("Reward name {}", a), a, 24.0);
        g.value(r, tl(88.0, 62.0, 190.0, 30.0), &format!("Reward amount {}", a), b, 26.0);
    }
    let btns = g.hbox(detail, bl(26.0, 26.0, 948.0, 260.0), "Detail buttons", 16.0);
    g.button2(btns, sz(300.0, 84.0), "Abandon button", "Abandon");
    g.button(btns, sz(300.0, 84.0), "Track button", "Track quest");
}

/// The tracked quest as a big card with the other quests in smaller cards under it.
fn quests_cards(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 36.0, 1200.0, 60.0), "Screen title", "TRACKED QUESTS", 52.0);
    g.bg_label(p, tc(0.0, 102.0, 1200.0, 32.0), "Screen subtitle", "Your three active quests this week", 26.0, CENTER);

    let main = g.panel(p, tc(0.0, 160.0, 1400.0, 400.0), "Main quest card");
    g.art(main, tl(24.0, 24.0, 520.0, 352.0), "Quest picture");
    g.badge(main, tl(568.0, 30.0, 200.0, 40.0), "Main quest tag", "MAIN STORY");
    g.heading_l(main, tl(568.0, 84.0, 800.0, 46.0), "Main quest title", "The river crossing", 36.0);
    g.note(main, tl(568.0, 136.0, 800.0, 64.0), "Main quest text", "The bridge is gone. Find the ferryman and get the cart across before nightfall.", 24.0);
    g.note(main, tl(568.0, 216.0, 300.0, 30.0), "Main progress label", "Progress", 24.0);
    g.bar(main, tl(568.0, 252.0, 780.0, 22.0), "Main progress bar", 0.5, g.t.accent);
    g.badge(main, tl(568.0, 300.0, 180.0, 34.0), "Main step tag", "Step 2 of 4");
    g.button(main, tr(24.0, 300.0, 300.0, 76.0), "Continue quest button", "Continue");

    let row = g.hbox_c(p, bl(0.0, 60.0, 1440.0, 300.0), "Quest cards", 24.0);
    for (i, (n, d, v)) in [("Lights on the ridge", "Night watch", 0.3), ("The hermit's map", "Explore", 0.75), ("Papers at the mill", "Collect", 1.0)].iter().enumerate() {
        let c = g.card(row, sz(440.0, 300.0), &format!("Quest card {}", n));
        g.art(c, tl(16.0, 16.0, 408.0, 130.0), &format!("Quest card picture {}", i + 1));
        g.heading_l(c, tl(16.0, 156.0, 408.0, 32.0), &format!("Quest card name {}", n), n, 28.0);
        g.note(c, tl(16.0, 192.0, 408.0, 28.0), &format!("Quest card type {}", n), d, 24.0);
        g.bar(c, tl(16.0, 226.0, 408.0, 18.0), &format!("Quest card bar {}", n), *v, g.t.accent);
        g.note(c, tl(16.0, 252.0, 408.0, 28.0), &format!("Quest card state {}", n), if *v >= 1.0 { "Ready to turn in" } else { "In progress" }, 22.0);
    }
}

/// Chapters as a list with their quests underneath, the next steps told on the right.
fn quests_chapters(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 30.0, 1000.0, 56.0), "Screen title", "Story chapters", 48.0);
    g.bg_label(p, tc(0.0, 92.0, 1000.0, 32.0), "Screen subtitle", "Chapter 2 of 6 in progress", 26.0, CENTER);

    let list = g.panel(p, left(40.0, 1000.0, 150.0, 40.0), "Chapter panel");
    let rows = g.vbox_fill(list, tl(16.0, 16.0, 968.0, 820.0), "Chapter list", 12.0);
    let chapters = [("I. Arrival", "Done"), ("II. The long road", "In progress"), ("III. Under the ridge", "Locked"), ("IV. The far harbor", "Locked")];
    for (i, (n, s)) in chapters.iter().enumerate() {
        let c = if i == 1 { g.card_on(rows, sz(968.0, 180.0), &format!("Chapter {}", n)) } else { g.card(rows, sz(968.0, 180.0), &format!("Chapter {}", n)) };
        g.icon(c, tl(20.0, 22.0, 56.0, 56.0), &format!("Chapter icon {}", i + 1), &format!("{}", i + 1));
        g.heading_l(c, tl(96.0, 20.0, 500.0, 34.0), &format!("Chapter name {}", n), n, 28.0);
        g.badge(c, tl(96.0, 66.0, 190.0, 34.0), &format!("Chapter state {}", n), s);
        g.note(c, tl(96.0, 122.0, 840.0, 30.0), &format!("Chapter steps {}", n), "Four of seven quests finished", 24.0);
    }
    g.note(list, bl(16.0, 16.0, 968.0, 28.0), "Chapter note", "Locked chapters open when the story moves on", 22.0);

    let side = g.panel(p, right(40.0, 800.0, 150.0, 40.0), "Next step panel");
    g.heading_l(side, tl(24.0, 20.0, 752.0, 36.0), "Next heading", "Next steps", 28.0);
    g.art(side, tl(24.0, 68.0, 752.0, 280.0), "Next step picture");
    let steps = g.vbox_fill(side, tl(20.0, 368.0, 760.0, 300.0), "Next step list", 10.0);
    for (i, s) in ["Speak to the ferryman", "Cross the river", "Reach the far shore"].iter().enumerate() {
        let r = g.card(steps, sz(760.0, 90.0), &format!("Next step {}", i + 1));
        g.icon(r, tl(14.0, 17.0, 56.0, 56.0), &format!("Next step number {}", i + 1), &format!("{}", i + 1));
        g.label(r, tl(86.0, 30.0, 640.0, 32.0), &format!("Next step text {}", i + 1), s, 26.0);
    }
    let foot = g.vbox_fill(side, bl(24.0, 24.0, 752.0, 160.0), "Side buttons", 12.0);
    g.button(foot, sz(752.0, 74.0), "Open chapter button", "Open chapter");
    g.button2(foot, sz(752.0, 74.0), "Story recap button", "Story recap");
}

// ---------------------------------------------------------------------------------------------------------------
// Progress - Challenges & achievements
// ---------------------------------------------------------------------------------------------------------------

/// Achievement rows with a trophy, a description and a progress bar; earned ones on top.
fn challenges_list(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(24.0, 96.0, 24.0, 24.0), "Header panel");
    g.heading_l(head, tl(28.0, 14.0, 700.0, 40.0), "Screen title", "ACHIEVEMENTS", 36.0);
    g.note(head, tl(28.0, 58.0, 700.0, 28.0), "Header note", "12 earned - 28 still open", 24.0);
    g.bar(head, tr(28.0, 34.0, 420.0, 26.0), "Overall bar", 0.3, g.t.accent);

    let list = g.panel(p, left(40.0, 1180.0, 136.0, 40.0), "Achievement panel");
    let rows = g.vbox_fill(list, tl(16.0, 16.0, 1148.0, 830.0), "Achievement list", 10.0);
    let items = [("First steps", "Finish the first chapter", 1.0), ("Collector", "Find 20 chests", 0.65), ("Sharp eye", "Spot 10 hidden marks", 0.4), ("Long road", "Walk 10 km", 0.25), ("Duelist", "Win 5 duels", 0.0)];
    for (n, d, v) in items.iter() {
        let c = g.card(rows, sz(1148.0, 156.0), &format!("Achievement {}", n));
        g.icon(c, tl(20.0, 46.0, 64.0, 64.0), &format!("Trophy icon {}", n), "*");
        g.heading_l(c, tl(104.0, 20.0, 760.0, 34.0), &format!("Achievement name {}", n), n, 28.0);
        g.note(c, tl(104.0, 60.0, 760.0, 28.0), &format!("Achievement text {}", n), d, 24.0);
        g.bar(c, tl(104.0, 104.0, 740.0, 22.0), &format!("Achievement bar {}", n), *v, g.t.accent);
        g.badge(c, tr(20.0, 20.0, 150.0, 34.0), &format!("Achievement tag {}", n), if *v >= 1.0 { "EARNED" } else { "OPEN" });
        g.value(c, tr(20.0, 64.0, 150.0, 30.0), &format!("Achievement percent {}", n), &format!("{:.0}%", v * 100.0), 24.0);
    }
    let foot = g.hbox(list, bl(16.0, 16.0, 1148.0, 70.0), "List footer", 16.0);
    g.note(foot, sz(700.0, 70.0), "Footer note", "Achievements are shared with your profile", 24.0);
    g.button2(foot, sz(300.0, 70.0), "Filter button", "Show all");

    let side = g.panel(p, right(40.0, 620.0, 136.0, 40.0), "Highlight panel");
    g.art(side, tl(24.0, 24.0, 572.0, 300.0), "Highlight trophy");
    g.heading(side, tl(24.0, 344.0, 572.0, 40.0), "Highlight title", "Rarest achievement", 30.0);
    g.note(side, tl(24.0, 390.0, 572.0, 30.0), "Highlight text", "Sharp eye - earned by 4 percent", 24.0);
    let stats = g.vbox_fill(side, tl(24.0, 436.0, 572.0, 246.0), "Highlight stats", 10.0);
    for (n, v) in [("Earned", "12"), ("Total", "40"), ("Score", "640")].iter() {
        let r = g.card(stats, sz(572.0, 70.0), &format!("Highlight stat {}", n));
        g.note(r, tl(16.0, 22.0, 300.0, 30.0), &format!("Highlight stat name {}", n), n, 26.0);
        g.value(r, tr(16.0, 22.0, 200.0, 30.0), &format!("Highlight stat value {}", n), v, 26.0);
    }
    g.button(side, bl(24.0, 24.0, 572.0, 80.0), "Compare button", "Compare with friends");
}

/// A grid of badges: earned ones in colour, the rest dark with a lock.
fn challenges_badges(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 30.0, 1200.0, 56.0), "Screen title", "BADGE CASE", 52.0);
    g.bg_label(p, tc(0.0, 94.0, 1200.0, 30.0), "Screen subtitle", "9 of 24 collected", 26.0, CENTER);
    let counter = g.hbox_c(p, tc(0.0, 138.0, 560.0, 30.0), "Counter row", 16.0);
    g.circle(counter, sz(26.0, 26.0), "Counter dot", g.t.accent);
    g.bg_label(counter, sz(300.0, 30.0), "Counter text", "Every badge gives a reward", 26.0, LEFT);

    let panel = g.panel(p, fill(160.0, 190.0, 160.0, 60.0), "Badge panel");
    let grid = g.grid(panel, tl(24.0, 24.0, 1576.0, 780.0), "Badge grid", 180.0, 180.0, 16.0);
    for i in 0..24usize {
        let earned = i % 3 == 0;
        let c = if earned { g.card_on(grid, sz(180.0, 180.0), &format!("Badge {}", i + 1)) } else { g.card(grid, sz(180.0, 180.0), &format!("Badge {}", i + 1)) };
        if earned {
            g.icon(c, cc(0.0, -14.0, 96.0, 96.0), &format!("Badge icon {}", i + 1), "*");
        } else {
            g.slot(c, cc(0.0, -14.0, 96.0, 96.0), &format!("Badge slot {}", i + 1));
            g.note(c, tl(50.0, 40.0, 80.0, 32.0), &format!("Badge lock {}", i + 1), "?", 30.0);
        }
        g.note(c, tl(10.0, 132.0, 160.0, 28.0), &format!("Badge name {}", i + 1), if earned { "Earned" } else { "Locked" }, 22.0);
    }
    let foot = g.hbox(p, bc(0.0, 80.0, 700.0, 76.0), "Badge footer", 16.0);
    g.button2(foot, sz(320.0, 76.0), "Filter earned button", "Earned");
    g.button(foot, sz(320.0, 76.0), "Filter all button", "All badges");
}

/// Weekly challenges with a countdown at the top, each challenge with its reward and claim button.
fn challenges_weekly(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(40.0, 120.0, 180.0, 180.0), "Header panel");
    g.heading_l(head, tl(32.0, 26.0, 700.0, 48.0), "Screen title", "WEEKLY CHALLENGES", 40.0);
    g.note(head, tl(32.0, 76.0, 900.0, 30.0), "Header note", "Reset in 2 days, 6 hours", 24.0);
    g.badge(head, tr(32.0, 42.0, 420.0, 44.0), "Countdown badge", "ENDS 02d 06h 14m");

    let row = g.hbox(p, tl(180.0, 200.0, 1560.0, 420.0), "Challenge row", 24.0);
    let cards = [("Sprint master", "Win 3 races", 0.66, "500 coins"), ("Night hunter", "Clear 5 night raids", 0.2, "1 crate"), ("Perfect run", "Finish without damage", 1.0, "Badge")];
    for (n, d, v, rw) in cards.iter() {
        let c = g.card_on(row, sz(504.0, 420.0), &format!("Challenge {}", n));
        g.art(c, tl(20.0, 20.0, 464.0, 180.0), &format!("Challenge picture {}", n));
        g.heading_l(c, tl(20.0, 216.0, 464.0, 36.0), &format!("Challenge name {}", n), n, 30.0);
        g.note(c, tl(20.0, 256.0, 464.0, 28.0), &format!("Challenge text {}", n), d, 24.0);
        g.bar(c, tl(20.0, 296.0, 464.0, 22.0), &format!("Challenge bar {}", n), *v, g.t.accent);
        g.note(c, tl(20.0, 330.0, 300.0, 28.0), &format!("Challenge reward {}", n), "Reward", 22.0);
        g.value(c, tl(280.0, 330.0, 204.0, 28.0), &format!("Challenge reward value {}", n), rw, 24.0);
        let btns = g.vbox_fill(c, bl(20.0, 20.0, 464.0, 84.0), &format!("Challenge buttons {}", n), 0.0);
        g.button(btns, sz(464.0, 84.0), &format!("Claim button {}", n), if *v >= 1.0 { "Claim reward" } else { "In progress" });
    }
    let foot = g.panel(p, bottom(40.0, 180.0, 180.0, 180.0), "Foot panel");
    g.note(foot, tl(32.0, 24.0, 900.0, 30.0), "Foot note", "Challenges are shared by the whole group", 24.0);
    let total = g.hbox(foot, tl(32.0, 74.0, 900.0, 80.0), "Total row", 16.0);
    g.note(total, sz(260.0, 80.0), "Total label", "Weekly score", 24.0);
    g.bar(total, sz(560.0, 24.0), "Total bar", 0.62, g.t.accent2);
    g.button(foot, tr(32.0, 48.0, 300.0, 84.0), "Open events button", "All events >");
}

// ---------------------------------------------------------------------------------------------------------------
// Progress - Profile & game totals
// ---------------------------------------------------------------------------------------------------------------

/// A player card with the avatar and a grid of figures beside it.
fn profile_card(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 40.0, 1200.0, 56.0), "Screen title", "PROFILE", 48.0);

    let card = g.panel(p, tl(80.0, 140.0, 640.0, 820.0), "Profile card");
    g.circle(card, tc(0.0, 40.0, 260.0, 260.0), "Avatar", g.t.accent);
    g.badge(card, cc(96.0, 96.0, 84.0, 84.0), "Level badge", "42");
    g.heading(card, tl(0.0, 320.0, 640.0, 46.0), "Player name", "A. Rivera", 36.0);
    g.txt(card, tl(0.0, 372.0, 640.0, 30.0), "Player title", "Wanderer of the north", 24.0, g.t.muted, CENTER, false);
    g.rect(card, tl(60.0, 420.0, 520.0, 2.0), "Card divider", g.t.line, 0.0);
    let rows = g.vbox_fill(card, tl(40.0, 444.0, 560.0, 280.0), "Profile rows", 10.0);
    for (n, v) in [("Play time", "41h 12m"), ("Completion", "63%"), ("Last played", "Today")].iter() {
        let r = g.card(rows, sz(560.0, 86.0), &format!("Profile row {}", n));
        g.note(r, tl(18.0, 28.0, 280.0, 30.0), &format!("Profile row name {}", n), n, 24.0);
        g.value(r, tr(18.0, 28.0, 220.0, 30.0), &format!("Profile row value {}", n), v, 26.0);
    }
    g.button(card, bl(40.0, 40.0, 560.0, 80.0), "Edit profile button", "Edit profile");

    let grid_panel = g.panel(p, right(80.0, 1040.0, 140.0, 140.0), "Figure panel");
    g.heading_l(grid_panel, tl(28.0, 24.0, 700.0, 36.0), "Figure heading", "Game totals", 28.0);
    let grid = g.grid(grid_panel, tl(28.0, 76.0, 984.0, 430.0), "Figure grid", 312.0, 200.0, 24.0);
    for (n, v) in [("Quests", "37"), ("Chests", "112"), ("Duels won", "58"), ("Maps cleared", "9"), ("Records", "14"), ("Achievements", "12")].iter() {
        let c = g.card(grid, sz(312.0, 200.0), &format!("Figure {}", n));
        g.icon(c, tl(24.0, 24.0, 56.0, 56.0), &format!("Figure icon {}", n), "+");
        g.heading_l(c, tl(24.0, 100.0, 264.0, 50.0), &format!("Figure value {}", n), v, 44.0);
        g.note(c, tl(24.0, 152.0, 264.0, 28.0), &format!("Figure name {}", n), n, 24.0);
    }
    let foot = g.hbox(grid_panel, bl(28.0, 28.0, 984.0, 80.0), "Figure footer", 16.0);
    g.button2(foot, sz(300.0, 80.0), "Share profile button", "Share profile");
    g.button(foot, sz(300.0, 80.0), "Details button", "All statistics");
}

/// Play time and personal records in two lists.
fn profile_records(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 30.0, 1100.0, 56.0), "Screen title", "Your record", 48.0);
    g.bg_label(p, tc(0.0, 94.0, 1100.0, 30.0), "Screen subtitle", "Everything you have done, in numbers", 26.0, CENTER);

    let left_col = g.panel(p, left(40.0, 900.0, 150.0, 40.0), "Play time panel");
    g.heading_l(left_col, tl(24.0, 20.0, 852.0, 36.0), "Play heading", "Play time", 28.0);
    let rows = g.vbox_fill(left_col, tl(16.0, 68.0, 868.0, 420.0), "Play time list", 10.0);
    for (n, v) in [("Total", "41h 12m"), ("This week", "6h 05m"), ("Sessions", "128"), ("Average", "19m")].iter() {
        let r = g.card(rows, sz(868.0, 96.0), &format!("Play row {}", n));
        g.note(r, tl(20.0, 32.0, 400.0, 32.0), &format!("Play row name {}", n), n, 26.0);
        g.value(r, tr(20.0, 32.0, 300.0, 32.0), &format!("Play row value {}", n), v, 28.0);
    }
    let months = g.panel(left_col, bl(16.0, 16.0, 868.0, 320.0), "Months panel");
    g.heading_l(months, tl(20.0, 14.0, 828.0, 32.0), "Months heading", "Last six months", 26.0);
    let bars = g.hbox(months, tl(20.0, 56.0, 828.0, 220.0), "Month bars", 14.0);
    for i in 0..6usize {
        let v = 0.2 + ((i * 29) % 70) as f32 / 100.0;
        let col = g.vbox(bars, sz(122.0, 220.0), &format!("Month column {}", i + 1), 0.0);
        g.bar(col, sz(48.0, 170.0), &format!("Month bar {}", i + 1), v, g.t.accent);
        g.note(col, sz(100.0, 40.0), &format!("Month label {}", i + 1), &format!("M{}", i + 1), 22.0);
    }

    let right_col = g.panel(p, right(40.0, 900.0, 150.0, 40.0), "Records panel");
    g.heading_l(right_col, tl(24.0, 20.0, 852.0, 36.0), "Records heading", "Personal records", 28.0);
    let recs = g.vbox_fill(right_col, tl(16.0, 68.0, 868.0, 700.0), "Record list", 10.0);
    let records = [("Fastest stage", "1:42"), ("Longest combo", "x34"), ("Most coins", "8,450"), ("Deepest floor", "12"), ("Best duel", "3-0"), ("Perfect runs", "7")];
    for (i, (n, v)) in records.iter().enumerate() {
        let r = g.card(recs, sz(868.0, 96.0), &format!("Record {}", n));
        g.icon(r, tl(16.0, 22.0, 52.0, 52.0), &format!("Record icon {}", i + 1), "+");
        g.label(r, tl(86.0, 32.0, 480.0, 32.0), &format!("Record name {}", n), n, 26.0);
        g.value(r, tr(20.0, 32.0, 260.0, 32.0), &format!("Record value {}", n), v, 28.0);
    }
    let foot = g.hbox(right_col, bl(16.0, 16.0, 868.0, 80.0), "Record footer", 16.0);
    g.button2(foot, sz(300.0, 80.0), "Records button", "All records");
    g.button(foot, sz(300.0, 80.0), "Back button", "Back");
}

/// The player compared with the average player: stat bars with both values.
fn profile_compare(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(30.0, 100.0, 30.0, 30.0), "Header bar");
    g.heading_l(head, tl(28.0, 26.0, 700.0, 44.0), "Screen title", "YOU VS AVERAGE", 40.0);
    g.note(head, tr(28.0, 34.0, 560.0, 32.0), "Header note", "Compared with players at level 42", 24.0);

    let panel = g.panel(p, vmid(0.0, 1300.0, 160.0, 60.0), "Compare panel");
    g.bar(panel, tl(24.0, 24.0, 1252.0, 16.0), "Compare legend bar", 0.0, g.t.accent);
    g.note(panel, tl(640.0, 20.0, 620.0, 28.0), "Legend text", "You in accent, average in grey", 24.0);
    let rows = g.vbox_fill(panel, tl(20.0, 56.0, 1260.0, 640.0), "Compare list", 10.0);
    let stats = [("Attack", 0.82, 0.55), ("Defense", 0.44, 0.58), ("Speed", 0.71, 0.5), ("Luck", 0.35, 0.4), ("Craft", 0.9, 0.62), ("Survival", 0.58, 0.52)];
    for (n, you, avg) in stats.iter() {
        let r = g.card(rows, sz(1260.0, 96.0), &format!("Compare row {}", n));
        g.label(r, tl(20.0, 32.0, 240.0, 32.0), &format!("Compare name {}", n), n, 26.0);
        g.bar(r, tl(280.0, 22.0, 760.0, 24.0), &format!("Compare yours {}", n), *you, g.t.accent);
        g.bar(r, tl(280.0, 54.0, 760.0, 24.0), &format!("Compare average {}", n), *avg, g.t.muted);
        g.value(r, tr(20.0, 32.0, 180.0, 32.0), &format!("Compare delta {}", n), &format!("{}", if you > avg { "+" } else { "-" }), 28.0);
    }
    let foot = g.hbox(panel, bl(20.0, 24.0, 1260.0, 90.0), "Compare footer", 16.0);
    g.button2(foot, sz(320.0, 90.0), "Share button", "Share results");
    g.button(foot, sz(320.0, 90.0), "Tips button", "How to improve");
}

// ---------------------------------------------------------------------------------------------------------------
// Collection - Collection & extras hub
// ---------------------------------------------------------------------------------------------------------------

/// Big tiles, one per extra, each with its count badge.
fn hub_tiles(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 40.0, 1200.0, 56.0), "Screen title", "EXTRAS", 50.0);
    g.bg_label(p, tc(0.0, 102.0, 1200.0, 30.0), "Screen subtitle", "Everything you have found so far", 26.0, CENTER);

    let panel = g.panel(p, fill(160.0, 150.0, 160.0, 60.0), "Tile panel");
    let grid = g.grid(panel, tl(24.0, 24.0, 1576.0, 640.0), "Tile grid", 382.0, 300.0, 22.0);
    let tiles = [("Gallery", "24 of 60 pictures"), ("Music", "12 tracks"), ("Movies", "4 films"), ("Bestiary", "16 creatures"), ("Concept art", "18 sketches"), ("Rewards", "6 unopened")];
    for (n, d) in tiles.iter() {
        let c = g.card_on(grid, sz(382.0, 300.0), &format!("Tile {}", n));
        g.art(c, tl(16.0, 16.0, 350.0, 180.0), &format!("Tile picture {}", n));
        g.heading_l(c, tl(16.0, 212.0, 350.0, 34.0), &format!("Tile name {}", n), n, 28.0);
        g.note(c, tl(16.0, 250.0, 350.0, 28.0), &format!("Tile text {}", n), d, 22.0);
    }
    let foot = g.hbox(p, bc(0.0, 80.0, 760.0, 76.0), "Hub footer", 16.0);
    g.button2(foot, sz(340.0, 76.0), "Rewards button", "Open rewards");
    g.button(foot, sz(340.0, 76.0), "Back button", "Back");
}

/// A vertical list of extras on the left and a large preview of the selected one on the right.
fn hub_list(g: &mut G) {
    let p = g.page();
    g.background();
    let side = g.panel(p, left(40.0, 380.0, 40.0, 40.0), "Extra list panel");
    g.heading_l(side, tl(20.0, 20.0, 340.0, 36.0), "List heading", "EXTRAS", 26.0);
    let list = g.vbox_fill(side, tl(14.0, 66.0, 352.0, 520.0), "Extra list", 10.0);
    let extras = [("Gallery", true), ("Music", false), ("Movies", false), ("Bestiary", false), ("Concept art", false)];
    for (n, on) in extras.iter() {
        g.menu_item(list, sz(352.0, 92.0), &format!("Extra item {}", n), n, *on);
    }
    let foot = g.vbox_fill(side, bl(14.0, 14.0, 352.0, 110.0), "Extra footer", 10.0);
    g.note(foot, sz(352.0, 40.0), "Footer note", "Progress saved locally", 22.0);
    g.button2(foot, sz(352.0, 60.0), "Back button", "Back");

    let prev = g.panel(p, fill(440.0, 40.0, 60.0, 40.0), "Preview panel");
    g.art(prev, tl(24.0, 24.0, 1392.0, 560.0), "Preview picture");
    g.heading_l(prev, tl(24.0, 604.0, 900.0, 46.0), "Preview title", "Gallery", 38.0);
    g.note(prev, tl(24.0, 660.0, 900.0, 60.0), "Preview text", "Every picture you unlock in the story ends up here, sorted by chapter and by character.", 24.0);
    let stats = g.hbox(prev, tl(24.0, 740.0, 900.0, 90.0), "Preview stats", 16.0);
    g.note(stats, sz(280.0, 90.0), "Preview count label", "24 of 60", 26.0);
    g.bar(stats, sz(560.0, 24.0), "Preview bar", 0.4, g.t.accent);
    let btns = g.hbox(prev, bl(24.0, 24.0, 1392.0, 90.0), "Preview buttons", 16.0);
    g.button(btns, sz(400.0, 90.0), "Open extra button", "Open gallery");
    g.button2(btns, sz(400.0, 90.0), "Favourites button", "Favourites");
}

/// Shelves, like a library: each row is a shelf with its items and its own percentage.
fn hub_shelves(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 30.0, 1300.0, 56.0), "Screen title", "The collection room", 46.0);
    g.bg_label(p, tc(0.0, 92.0, 1300.0, 30.0), "Screen subtitle", "Four shelves, one room", 26.0, CENTER);

    let panel = g.panel(p, fill(120.0, 150.0, 120.0, 40.0), "Shelf panel");
    let rows = g.vbox_fill(panel, tl(20.0, 20.0, 1640.0, 700.0), "Shelf list", 16.0);
    let shelves = [("Gallery shelf", "Pictures", 0.4), ("Music shelf", "Records", 0.7), ("Movies shelf", "Films", 0.25), ("Bestiary shelf", "Creatures", 0.6)];
    for (n, sub, v) in shelves.iter() {
        let c = g.card(rows, sz(1640.0, 163.0), &format!("Shelf {}", n));
        g.heading_l(c, tl(24.0, 20.0, 360.0, 34.0), &format!("Shelf name {}", n), n, 28.0);
        g.note(c, tl(24.0, 60.0, 360.0, 28.0), &format!("Shelf sub {}", n), sub, 22.0);
        g.bar(c, tl(24.0, 104.0, 360.0, 22.0), &format!("Shelf bar {}", n), *v, g.t.accent);
        let strip = g.hbox(c, tl(420.0, 20.0, 1200.0, 123.0), &format!("Shelf items {}", n), 14.0);
        for i in 0..6usize {
            g.art(strip, sz(180.0, 123.0), &format!("Shelf item {} {}", n, i + 1));
        }
    }
    let foot = g.hbox(p, bc(0.0, 80.0, 800.0, 76.0), "Shelf footer", 16.0);
    g.button2(foot, sz(360.0, 76.0), "Rewards button", "Unopened rewards");
    g.button(foot, sz(360.0, 76.0), "Close button", "Close");
}

// ---------------------------------------------------------------------------------------------------------------
// Collection - Collectables menu
// ---------------------------------------------------------------------------------------------------------------

/// A grid of collectables with a counter per zone and the total progress on top.
fn collect_grid(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tl(60.0, 32.0, 900.0, 56.0), "Screen title", "COLLECTABLES", 46.0);
    g.bg_label(p, tl(60.0, 94.0, 900.0, 30.0), "Screen subtitle", "Found in every region of the map", 26.0, LEFT);
    let total = g.panel(p, tr(60.0, 40.0, 620.0, 96.0), "Total panel");
    g.note(total, tl(20.0, 14.0, 300.0, 28.0), "Total label", "Total progress", 24.0);
    g.bar(total, tl(20.0, 52.0, 420.0, 22.0), "Total bar", 0.45, g.t.accent);
    g.value(total, tr(20.0, 14.0, 160.0, 30.0), "Total value", "27 / 60", 26.0);

    let panel = g.panel(p, fill(60.0, 150.0, 60.0, 64.0), "Collect panel");
    let zones = g.hbox(panel, tl(24.0, 24.0, 1780.0, 90.0), "Zone row", 16.0);
    for (n, v) in [("Forest", "8 of 12"), ("Ruins", "5 of 12"), ("Coast", "9 of 16"), ("Peaks", "5 of 20")].iter() {
        let c = g.card(zones, sz(433.0, 90.0), &format!("Zone {}", n));
        g.note(c, tl(18.0, 18.0, 240.0, 30.0), &format!("Zone name {}", n), n, 24.0);
        g.value(c, tr(18.0, 18.0, 200.0, 30.0), &format!("Zone value {}", n), v, 24.0);
    }
    let grid = g.grid(panel, tl(24.0, 130.0, 1780.0, 560.0), "Collect grid", 340.0, 176.0, 20.0);
    for i in 0..10usize {
        let found = i % 3 != 2;
        let c = if found { g.card_on(grid, sz(340.0, 176.0), &format!("Collectable {}", i + 1)) } else { g.card(grid, sz(340.0, 176.0), &format!("Collectable {}", i + 1)) };
        if found {
            g.art(c, tl(16.0, 16.0, 120.0, 120.0), &format!("Collectable picture {}", i + 1));
        } else {
            g.slot(c, tl(16.0, 16.0, 120.0, 120.0), &format!("Collectable slot {}", i + 1));
        }
        g.note(c, tl(152.0, 24.0, 172.0, 32.0), &format!("Collectable name {}", i + 1), if found { "Amber charm" } else { "Undiscovered" }, 24.0);
        g.note(c, tl(152.0, 62.0, 172.0, 56.0), &format!("Collectable zone {}", i + 1), if found { "Ruins" } else { "Find it nearby" }, 22.0);
    }
    let foot = g.hbox(panel, bl(24.0, 24.0, 1780.0, 80.0), "Collect footer", 16.0);
    g.note(foot, sz(1000.0, 80.0), "Footer note", "New items are marked with a dot", 24.0);
    g.button(foot, sz(360.0, 80.0), "Hint button", "Ask for a hint");
}

/// Filter chips on top, a list of items in the middle and the detail of the selected one on the right.
fn collect_list(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(30.0, 96.0, 30.0, 30.0), "Header panel");
    g.heading_l(head, tl(24.0, 26.0, 600.0, 40.0), "Screen title", "CURIOS", 34.0);
    let chips = g.hbox(head, tr(24.0, 30.0, 1040.0, 60.0), "Filter chips", 12.0);
    for (i, n) in ["All", "Charms", "Letters", "Maps", "Masks"].iter().enumerate() {
        if i == 0 {
            g.button(chips, sz(200.0, 60.0), &format!("Chip {} button", n), n);
        } else {
            g.button2(chips, sz(200.0, 60.0), &format!("Chip {} button", n), n);
        }
    }

    let list_panel = g.panel(p, left(30.0, 900.0, 146.0, 40.0), "Curio panel");
    let list = g.vbox_fill(list_panel, tl(16.0, 16.0, 868.0, 700.0), "Curio list", 10.0);
    let curios = [("Amber charm", "Ruins", true), ("Torn letter", "Coast", true), ("Old map", "Peaks", false), ("Silver mask", "Forest", true), ("Bone ring", "Ruins", false), ("Glass eye", "Coast", true)];
    for (n, z, got) in curios.iter() {
        let r = g.card(list, sz(868.0, 106.0), &format!("Curio {}", n));
        if *got {
            g.icon(r, tl(16.0, 26.0, 54.0, 54.0), &format!("Curio icon {}", n), "*");
        } else {
            g.slot(r, tl(16.0, 26.0, 54.0, 54.0), &format!("Curio slot {}", n));
        }
        g.label(r, tl(88.0, 24.0, 460.0, 32.0), &format!("Curio name {}", n), n, 26.0);
        g.note(r, tl(88.0, 60.0, 460.0, 28.0), &format!("Curio zone {}", n), z, 22.0);
        g.badge(r, tr(18.0, 34.0, 200.0, 34.0), &format!("Curio state {}", n), if *got { "FOUND" } else { "MISSING" });
    }
    let foot = g.hbox(list_panel, bl(16.0, 16.0, 868.0, 70.0), "Curio footer", 16.0);
    g.note(foot, sz(500.0, 70.0), "Footer note", "Sorted by region", 24.0);
    g.button2(foot, sz(300.0, 70.0), "Sort button", "Sort");

    let detail = g.panel(p, right(30.0, 900.0, 146.0, 40.0), "Detail panel");
    g.art(detail, tl(24.0, 24.0, 852.0, 360.0), "Detail picture");
    g.heading_l(detail, tl(24.0, 400.0, 852.0, 40.0), "Detail title", "Amber charm", 32.0);
    g.note(detail, tl(24.0, 448.0, 852.0, 100.0), "Detail text", "A small charm carved from amber. It still smells of the forest where it was left.", 24.0);
    let rows = g.vbox_fill(detail, tl(24.0, 560.0, 852.0, 200.0), "Detail rows", 10.0);
    for (n, v) in [("Region", "Ruins"), ("Found on", "Night of the storm"), ("Rarity", "Uncommon")].iter() {
        let r = g.card(rows, sz(852.0, 60.0), &format!("Detail row {}", n));
        g.note(r, tl(16.0, 16.0, 380.0, 28.0), &format!("Detail row name {}", n), n, 24.0);
        g.value(r, tr(16.0, 16.0, 380.0, 28.0), &format!("Detail row value {}", n), v, 24.0);
    }
    let btns = g.hbox(detail, bl(24.0, 24.0, 852.0, 80.0), "Detail buttons", 14.0);
    g.button(btns, sz(400.0, 80.0), "Inspect button", "Inspect");
    g.button2(btns, sz(400.0, 80.0), "Back detail button", "Back");
}

/// Completion map: a region overview with a progress bar per region.
fn collect_regions(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(30.0, 100.0, 30.0, 30.0), "Header panel");
    g.heading_l(head, tl(24.0, 26.0, 900.0, 42.0), "Screen title", "REGION COMPLETION", 36.0);
    g.note(head, tr(24.0, 34.0, 700.0, 30.0), "Header note", "Every region has its own chests", 24.0);

    let map = g.panel(p, left(30.0, 1100.0, 166.0, 40.0), "Map panel");
    g.art(map, tl(24.0, 24.0, 1052.0, 620.0), "Map picture");
    g.heading_l(map, tl(24.0, 664.0, 1052.0, 36.0), "Map caption", "World map", 28.0);
    let legend = g.hbox(map, tl(24.0, 712.0, 1052.0, 60.0), "Map legend", 20.0);
    g.circle(legend, sz(28.0, 28.0), "Legend complete", g.t.good);
    g.note(legend, sz(240.0, 60.0), "Legend complete label", "Complete", 24.0);
    g.circle(legend, sz(28.0, 28.0), "Legend partial", g.t.warn);
    g.note(legend, sz(240.0, 60.0), "Legend partial label", "In progress", 24.0);
    let btns = g.hbox(map, bl(24.0, 20.0, 1052.0, 80.0), "Map buttons", 14.0);
    g.button(btns, sz(500.0, 80.0), "Track button", "Track nearest");
    g.button2(btns, sz(500.0, 80.0), "Travel button", "Travel");

    let side = g.panel(p, right(30.0, 760.0, 166.0, 40.0), "Region panel");
    g.heading_l(side, tl(24.0, 20.0, 712.0, 34.0), "Region heading", "Regions", 28.0);
    let rows = g.vbox_fill(side, tl(16.0, 62.0, 728.0, 700.0), "Region list", 10.0);
    let regions = [("Forest", "12 of 12"), ("Ruins", "5 of 12"), ("Coast", "9 of 12"), ("Peaks", "5 of 12"), ("Marshes", "2 of 12"), ("Isles", "0 of 12")];
    for (i, (n, c)) in regions.iter().enumerate() {
        let r = g.card(rows, sz(728.0, 106.0), &format!("Region {}", n));
        g.heading_l(r, tl(18.0, 16.0, 380.0, 32.0), &format!("Region name {}", n), n, 26.0);
        g.value(r, tr(18.0, 16.0, 220.0, 32.0), &format!("Region value {}", n), c, 24.0);
        g.bar(r, tl(18.0, 62.0, 620.0, 20.0), &format!("Region bar {}", n), (12 - i) as f32 / 12.0, g.t.accent);
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Collection - Gallery menu
// ---------------------------------------------------------------------------------------------------------------

/// A wall of thumbnails with a count header and a chapter filter.
fn gallery_grid(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(30.0, 110.0, 30.0, 30.0), "Header panel");
    g.heading_l(head, tl(28.0, 22.0, 700.0, 44.0), "Screen title", "Picture gallery", 38.0);
    g.note(head, tl(28.0, 70.0, 700.0, 28.0), "Header note", "24 of 60 pictures unlocked", 24.0);
    let chapters = g.hbox(head, tr(28.0, 32.0, 900.0, 60.0), "Chapter filter", 12.0);
    for (i, n) in ["All", "Chapter I", "Chapter II", "Extras"].iter().enumerate() {
        if i == 0 {
            g.button(chapters, sz(210.0, 60.0), &format!("Chapter {} button", n), n);
        } else {
            g.button2(chapters, sz(210.0, 60.0), &format!("Chapter {} button", n), n);
        }
    }

    let panel = g.panel(p, fill(30.0, 160.0, 30.0, 40.0), "Wall panel");
    let grid = g.grid(panel, tl(24.0, 24.0, 1812.0, 760.0), "Thumb grid", 340.0, 240.0, 20.0);
    for i in 0..15usize {
        let open = i < 11;
        let c = g.card(grid, sz(340.0, 240.0), &format!("Thumb {}", i + 1));
        if open {
            g.art(c, tl(14.0, 14.0, 312.0, 170.0), &format!("Thumb picture {}", i + 1));
            g.note(c, tl(14.0, 192.0, 312.0, 28.0), &format!("Thumb name {}", i + 1), "The road east", 22.0);
        } else {
            g.slot(c, tl(14.0, 14.0, 312.0, 170.0), &format!("Thumb slot {}", i + 1));
            g.note(c, tl(14.0, 192.0, 312.0, 28.0), &format!("Thumb name {}", i + 1), "Locked", 22.0);
        }
    }
    let foot = g.hbox(p, bc(0.0, 80.0, 800.0, 76.0), "Gallery footer", 16.0);
    g.button2(foot, sz(360.0, 76.0), "Slideshow button", "Slideshow");
    g.button(foot, sz(360.0, 76.0), "Back gallery button", "Back");
}

/// A large picture with a thumbnail strip under it and arrows on the sides.
fn gallery_viewer(g: &mut G) {
    let p = g.page();
    g.background();
    let stage = g.panel(p, tc(0.0, 30.0, 1500.0, 760.0), "Viewer panel");
    g.art(stage, tl(24.0, 24.0, 1452.0, 580.0), "Viewer picture");
    g.heading_l(stage, tl(24.0, 620.0, 900.0, 40.0), "Viewer title", "The road east", 32.0);
    g.note(stage, tl(24.0, 666.0, 900.0, 28.0), "Viewer caption", "Chapter II - unlocked in the third act", 24.0);
    g.button2(stage, tr(24.0, 632.0, 200.0, 64.0), "Zoom button", "Zoom in");
    g.button2(stage, tl(1100.0, 632.0, 200.0, 64.0), "Share picture button", "Share");
    let _ = g.button(stage, tl(1240.0, 632.0, 188.0, 64.0), "Fullscreen button", "Full");

    let prev = g.panel(p, cl(-20.0, 86.0, 400.0, 86.0), "Previous plate");
    let _ = g.button(prev, sz(86.0, 86.0), "Previous button", "<");
    let next = g.panel(p, cr(-20.0, 86.0, 400.0, 86.0), "Next plate");
    let _ = g.button(next, sz(86.0, 86.0), "Next button", ">");

    let strip_panel = g.panel(p, bc(0.0, 60.0, 1200.0, 150.0), "Strip panel");
    let strip = g.hbox(strip_panel, tl(16.0, 16.0, 1168.0, 118.0), "Strip items", 14.0);
    for i in 0..6usize {
        let c = if i == 2 { g.card_on(strip, sz(180.0, 118.0), &format!("Strip item {}", i + 1)) } else { g.card(strip, sz(180.0, 118.0), &format!("Strip item {}", i + 1)) };
        g.art(c, tl(8.0, 8.0, 164.0, 90.0), &format!("Strip picture {}", i + 1));
        g.note(c, tl(8.0, 98.0, 164.0, 24.0), &format!("Strip label {}", i + 1), "Fine", 20.0);
    }
}

/// Albums by category: each card is a cover with the number of pictures inside.
fn gallery_albums(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 40.0, 1200.0, 56.0), "Screen title", "Albums", 50.0);
    g.bg_label(p, tc(0.0, 102.0, 1200.0, 30.0), "Screen subtitle", "Pictures sorted by the moment they belong to", 26.0, CENTER);

    let panel = g.panel(p, fill(160.0, 150.0, 160.0, 60.0), "Album panel");
    let grid = g.grid(panel, tl(24.0, 24.0, 1576.0, 620.0), "Album grid", 382.0, 290.0, 22.0);
    let albums = [("The beginning", "8 pictures"), ("The storm", "12 pictures"), ("The crossing", "5 pictures"), ("Characters", "14 pictures"), ("Places", "9 pictures"), ("Sketchbook", "12 pictures")];
    for (n, d) in albums.iter() {
        let c = g.card_on(grid, sz(382.0, 290.0), &format!("Album {}", n));
        g.art(c, tl(16.0, 16.0, 350.0, 180.0), &format!("Album cover {}", n));
        g.heading_l(c, tl(16.0, 208.0, 350.0, 32.0), &format!("Album name {}", n), n, 26.0);
        g.note(c, tl(16.0, 244.0, 350.0, 28.0), &format!("Album count {}", n), d, 22.0);
    }
    let foot = g.hbox(p, bc(0.0, 80.0, 800.0, 76.0), "Album footer", 16.0);
    g.button2(foot, sz(360.0, 76.0), "Favourites button", "Favourites");
    g.button(foot, sz(360.0, 76.0), "Back albums button", "Back");
}

// ---------------------------------------------------------------------------------------------------------------
// Collection - Collectable viewer
// ---------------------------------------------------------------------------------------------------------------

/// The object on a pedestal in the middle, rotate and zoom controls below, the card on the right.
fn collectable_pedestal(g: &mut G) {
    let p = g.page();
    g.background();
    let stage = g.panel(p, tl(40.0, 40.0, 1180.0, 1000.0), "Stage panel");
    g.heading_l(stage, tl(28.0, 24.0, 900.0, 40.0), "Stage title", "Amber charm", 34.0);
    g.circle(stage, cc(0.0, -40.0, 420.0, 420.0), "Backdrop", alpha(g.t.accent, 50));
    g.art(stage, cc(0.0, -60.0, 420.0, 420.0), "Object picture");
    g.rect(stage, tl(240.0, 640.0, 700.0, 16.0), "Pedestal top", g.t.accent, 0.0);
    g.rect(stage, tl(300.0, 656.0, 580.0, 120.0), "Pedestal body", g.t.panel2, g.t.radius);
    let controls = g.hbox(stage, bl(28.0, 28.0, 1124.0, 90.0), "Controls", 16.0);
    g.button2(controls, sz(260.0, 90.0), "Rotate left button", "Rotate");
    g.button2(controls, sz(260.0, 90.0), "Rotate right button", "Rotate");
    g.button2(controls, sz(260.0, 90.0), "Zoom out button", "Zoom out");
    g.button(controls, sz(260.0, 90.0), "Zoom in button", "Zoom in");

    let card = g.panel(p, right(40.0, 620.0, 40.0, 40.0), "Info card");
    g.heading_l(card, tl(24.0, 24.0, 572.0, 36.0), "Info title", "Amber charm", 30.0);
    g.badge(card, tl(24.0, 72.0, 180.0, 34.0), "Info rarity", "UNCOMMON");
    g.note(card, tl(24.0, 122.0, 572.0, 140.0), "Info text", "A small charm carved from amber. It still smells of the forest where it was left.", 24.0);
    let rows = g.vbox_fill(card, tl(24.0, 280.0, 572.0, 300.0), "Info rows", 10.0);
    for (n, v) in [("Found in", "Ruins"), ("Found on", "Night of the storm"), ("Value", "120 coins"), ("Set", "Wanderer")].iter() {
        let r = g.card(rows, sz(572.0, 64.0), &format!("Info row {}", n));
        g.note(r, tl(16.0, 18.0, 280.0, 28.0), &format!("Info row name {}", n), n, 22.0);
        g.value(r, tr(16.0, 18.0, 240.0, 28.0), &format!("Info row value {}", n), v, 24.0);
    }
    let btns = g.vbox_fill(card, bl(24.0, 24.0, 572.0, 170.0), "Info buttons", 12.0);
    g.button(btns, sz(572.0, 80.0), "Inspect detail button", "Inspect");
    g.button2(btns, sz(572.0, 80.0), "Prev object button", "Previous object");
}

/// One big card, like a trading card: picture on top, text under it and the buttons at the bottom.
fn collectable_card(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_label(p, tc(0.0, 24.0, 1200.0, 32.0), "Screen subtitle", "Wanderer set - card 1 of 8", 26.0, CENTER);
    let card = g.panel(p, cc(0.0, 0.0, 860.0, 900.0), "Card body");
    g.art(card, tc(0.0, 24.0, 812.0, 420.0), "Card picture");
    g.heading(card, tl(24.0, 460.0, 812.0, 44.0), "Card title", "Amber charm", 38.0);
    g.note(card, tl(24.0, 512.0, 812.0, 80.0), "Card text", "A charm that keeps the cold away. It glows a little when the road turns north.", 24.0);
    let tags = g.hbox(card, tl(24.0, 604.0, 812.0, 44.0), "Card tags", 12.0);
    g.badge(tags, sz(200.0, 44.0), "Card set tag", "WANDERER");
    g.badge(tags, sz(160.0, 44.0), "Card rarity tag", "RARE");
    let bars = g.hbox(card, tl(24.0, 664.0, 812.0, 80.0), "Card stats", 20.0);
    g.note(bars, sz(200.0, 80.0), "Card warmth label", "Warmth", 24.0);
    g.bar(bars, sz(560.0, 24.0), "Card warmth bar", 0.8, g.t.accent);
    let btns = g.hbox(card, bl(24.0, 24.0, 812.0, 90.0), "Card buttons", 14.0);
    g.button2(btns, sz(250.0, 90.0), "Card prev button", "<");
    g.button(btns, sz(500.0, 90.0), "Card next button", "Next card");
}

/// A display case: the object in a glass box, arrows on both sides and the index at the bottom.
fn collectable_case(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(40.0, 100.0, 40.0, 40.0), "Header panel");
    g.heading_l(head, tl(28.0, 26.0, 800.0, 40.0), "Screen title", "DISPLAY CASE", 34.0);
    g.note(head, tr(28.0, 34.0, 600.0, 28.0), "Header count", "7 of 24 objects", 24.0);

    let case = g.panel(p, cc(0.0, 20.0, 1000.0, 700.0), "Case panel");
    g.rect(case, tl(32.0, 32.0, 936.0, 520.0), "Glass", alpha(g.t.accent2, 40), g.t.radius);
    g.art(case, cc(0.0, -40.0, 420.0, 420.0), "Case object");
    g.heading(case, tl(32.0, 572.0, 936.0, 40.0), "Case title", "Bone ring", 32.0);
    g.note(case, tl(32.0, 620.0, 936.0, 28.0), "Case text", "Found in the old chapel, under the stone floor.", 24.0);

    let left_arrow = g.panel(p, cl(60.0, 90.0, 420.0, 90.0), "Left arrow plate");
    g.button(left_arrow, sz(90.0, 90.0), "Case prev button", "<");
    let right_arrow = g.panel(p, cr(60.0, 90.0, 420.0, 90.0), "Right arrow plate");
    g.button(right_arrow, sz(90.0, 90.0), "Case next button", ">");

    let info = g.panel(p, bl(60.0, 60.0, 620.0, 160.0), "Case info");
    g.note(info, tl(20.0, 18.0, 580.0, 28.0), "Case info label", "Set progress", 24.0);
    g.bar(info, tl(20.0, 56.0, 580.0, 22.0), "Case info bar", 0.3, g.t.accent);
    g.button(info, tl(20.0, 92.0, 580.0, 56.0), "Equip button", "Show in room");
    let dots = g.hbox(p, bc(0.0, 60.0, 560.0, 30.0), "Case dots", 12.0);
    for i in 0..8usize {
        g.circle(dots, sz(24.0, 24.0), &format!("Case dot {}", i + 1), if i == 2 { g.t.accent } else { g.t.muted });
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Collection - Sound & music player
// ---------------------------------------------------------------------------------------------------------------

/// The track list on the left, a now-playing bar at the bottom with the transport controls.
fn music_list(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(30.0, 100.0, 30.0, 30.0), "Header panel");
    g.heading_l(head, tl(28.0, 24.0, 700.0, 42.0), "Screen title", "SOUNDTRACK", 36.0);
    g.note(head, tl(28.0, 70.0, 700.0, 28.0), "Header note", "12 tracks unlocked", 24.0);
    let head_btns = g.hbox(head, tr(28.0, 30.0, 700.0, 60.0), "Header buttons", 12.0);
    g.button2(head_btns, sz(220.0, 60.0), "Shuffle button", "Shuffle");
    g.button2(head_btns, sz(220.0, 60.0), "Repeat button", "Repeat");

    let panel = g.panel(p, fill(30.0, 150.0, 30.0, 230.0), "Track panel");
    let rows = g.vbox_fill(panel, tl(20.0, 20.0, 1820.0, 660.0), "Track list", 10.0);
    let tracks = ["First light", "The road east", "Storm over the pass", "Quiet service", "The broken bridge", "Ashes and crowns"];
    for (i, t) in tracks.iter().enumerate() {
        let r = if i == 1 { g.card_on(rows, sz(1820.0, 100.0), &format!("Track {}", t)) } else { g.card(rows, sz(1820.0, 100.0), &format!("Track {}", t)) };
        g.icon(r, tl(18.0, 24.0, 52.0, 52.0), &format!("Track icon {}", t), if i == 1 { ">" } else { "#" });
        g.label(r, tl(88.0, 20.0, 600.0, 32.0), &format!("Track name {}", t), t, 26.0);
        g.note(r, tl(88.0, 56.0, 600.0, 28.0), &format!("Track artist {}", t), "Main theme - full version", 22.0);
        g.note(r, tr(400.0, 36.0, 200.0, 28.0), &format!("Track length {}", t), "3:12", 24.0);
        g.bar(r, tr(160.0, 40.0, 200.0, 20.0), &format!("Track bar {}", t), if i == 1 { 0.4 } else { 0.0 }, g.t.accent);
    }

    let player = g.panel(p, bottom(30.0, 160.0, 30.0, 30.0), "Player panel");
    g.art(player, tl(24.0, 24.0, 160.0, 112.0), "Now playing art");
    g.heading_l(player, tl(204.0, 24.0, 700.0, 36.0), "Now playing title", "The road east", 30.0);
    g.note(player, tl(204.0, 64.0, 700.0, 28.0), "Now playing text", "Main theme - full version", 22.0);
    g.bar(player, tl(204.0, 104.0, 900.0, 20.0), "Now playing bar", 0.4, g.t.accent);
    let transport = g.hbox_c(player, tc(0.0, 34.0, 460.0, 76.0), "Transport", 14.0);
    g.button2(transport, sz(100.0, 76.0), "Prev track button", "<");
    g.button(transport, sz(100.0, 76.0), "Play button", ">");
    g.button2(transport, sz(100.0, 76.0), "Next track button", ">");
    g.slider(player, tr(24.0, 44.0, 300.0, 32.0), "Volume slider", 0.7);
    g.value(player, tr(24.0, 84.0, 300.0, 28.0), "Volume value", "Volume", 22.0);
}

/// A jukebox: big cover art, the controls in a metal plate and the queue beside it.
fn music_jukebox(g: &mut G) {
    let p = g.page();
    g.background();
    g.bg_title(p, tc(0.0, 30.0, 1000.0, 56.0), "Screen title", "JUKEBOX", 52.0);
    g.bg_label(p, tc(0.0, 94.0, 1000.0, 30.0), "Screen subtitle", "Drop a coin and pick a song", 26.0, CENTER);

    let box_panel = g.panel(p, left(60.0, 780.0, 150.0, 80.0), "Jukebox panel");
    g.art(box_panel, tl(24.0, 24.0, 732.0, 400.0), "Jukebox art");
    g.heading_l(box_panel, tl(24.0, 444.0, 732.0, 38.0), "Jukebox title", "Ashes and crowns", 32.0);
    g.note(box_panel, tl(24.0, 490.0, 732.0, 28.0), "Jukebox text", "Ending theme", 24.0);
    let metal = g.panel(box_panel, bl(24.0, 24.0, 732.0, 200.0), "Metal plate");
    let row1 = g.hbox(metal, tl(20.0, 20.0, 692.0, 80.0), "Jukebox transport", 14.0);
    g.button2(row1, sz(150.0, 80.0), "Jukebox prev button", "<");
    g.button(row1, sz(300.0, 80.0), "Jukebox play button", "Play");
    g.button2(row1, sz(150.0, 80.0), "Jukebox next button", ">");
    let row2 = g.hbox(metal, bl(20.0, 20.0, 692.0, 60.0), "Jukebox extras", 14.0);
    g.toggle(row2, sz(220.0, 60.0), "Jukebox loop toggle", "Loop", true);
    g.slider(row2, sz(300.0, 60.0), "Jukebox volume", 0.6);

    let queue = g.panel(p, right(60.0, 960.0, 150.0, 80.0), "Queue panel");
    g.heading_l(queue, tl(24.0, 20.0, 912.0, 36.0), "Queue heading", "Up next", 28.0);
    let list = g.vbox_fill(queue, tl(16.0, 66.0, 928.0, 640.0), "Queue list", 10.0);
    let songs = ["First light", "Quiet service", "The broken bridge", "Storm over the pass", "Old roads"];
    for (i, s) in songs.iter().enumerate() {
        let r = g.card(list, sz(928.0, 112.0), &format!("Queue song {}", s));
        g.value(r, tl(18.0, 40.0, 60.0, 32.0), &format!("Queue number {}", s), if i == 0 { "A" } else { "B" }, 26.0);
        g.label(r, tl(84.0, 24.0, 600.0, 32.0), &format!("Queue name {}", s), s, 26.0);
        g.note(r, tl(84.0, 62.0, 600.0, 28.0), &format!("Queue note {}", s), "Unlocked", 22.0);
    }
    let foot = g.hbox(queue, bl(16.0, 16.0, 928.0, 70.0), "Queue footer", 14.0);
    g.button2(foot, sz(300.0, 70.0), "Queue shuffle button", "Shuffle all");
    g.button2(foot, sz(300.0, 70.0), "Queue clear button", "Clear");
}

/// Equalizer bars and the list of songs with their lengths.
fn music_equalizer(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(30.0, 110.0, 30.0, 30.0), "Header panel");
    g.heading_l(head, tl(28.0, 22.0, 700.0, 44.0), "Screen title", "AUDIO", 38.0);
    g.note(head, tr(28.0, 32.0, 900.0, 28.0), "Header note", "Music and sound effects", 24.0);

    let viz = g.panel(p, left(30.0, 700.0, 160.0, 40.0), "Visualizer panel");
    g.heading_l(viz, tl(24.0, 20.0, 652.0, 36.0), "Visualizer title", "Now playing", 28.0);
    g.note(viz, tl(24.0, 60.0, 652.0, 28.0), "Visualizer text", "Ashes and crowns", 24.0);
    let bars = g.hbox(viz, tl(24.0, 120.0, 652.0, 300.0), "Equalizer bars", 8.0);
    for i in 0..16usize {
        let h = 40.0 + ((i * 37) % 200) as f32;
        let col = g.vbox(bars, sz(28.0, 300.0), &format!("Equalizer column {}", i + 1), 0.0);
        g.rect(col, sz(28.0, h), &format!("Equalizer bar {}", i + 1), if i % 4 == 0 { g.t.accent2 } else { g.t.accent }, 0.0);
    }
    let mix = g.vbox_fill(viz, bl(24.0, 24.0, 652.0, 300.0), "Mix sliders", 14.0);
    for (n, v) in [("Music", 0.7), ("Effects", 0.9), ("Voice", 0.6)].iter() {
        let r = g.card(mix, sz(652.0, 90.0), &format!("Mix row {}", n));
        g.note(r, tl(16.0, 16.0, 200.0, 28.0), &format!("Mix name {}", n), n, 24.0);
        g.slider(r, tl(200.0, 26.0, 430.0, 36.0), &format!("Mix slider {}", n), *v);
    }

    let list_panel = g.panel(p, right(30.0, 1130.0, 160.0, 40.0), "Song panel");
    g.heading_l(list_panel, tl(24.0, 20.0, 1082.0, 36.0), "Song heading", "Soundtrack", 28.0);
    let songs = ["First light", "The road east", "Quiet service", "Ashes and crowns", "Old roads", "The long night"];
    for (i, s) in songs.iter().enumerate() {
        let c = if i == 3 { g.card_on(list_panel, sz(1082.0, 96.0), &format!("Song {}", s)) } else { g.card(list_panel, sz(1082.0, 96.0), &format!("Song {}", s)) };
        g.icon(c, tl(16.0, 22.0, 52.0, 52.0), &format!("Song icon {}", s), if i == 3 { ">" } else { "#" });
        g.label(c, tl(84.0, 20.0, 620.0, 32.0), &format!("Song name {}", s), s, 26.0);
        g.note(c, tl(84.0, 56.0, 620.0, 28.0), &format!("Song note {}", s), "Unlocked", 22.0);
        g.badge(c, tr(18.0, 30.0, 160.0, 36.0), &format!("Song length {}", s), "3:12");
    }
    let foot = g.hbox(list_panel, bl(16.0, 16.0, 1098.0, 70.0), "Song footer", 14.0);
    g.button2(foot, sz(340.0, 70.0), "Song play button", "Play");
    g.button2(foot, sz(340.0, 70.0), "Song random button", "Random");
}
