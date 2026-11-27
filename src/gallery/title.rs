//! Gallery: title and system menus (title, settings, modals and overlays).

use super::*;

pub fn entries() -> Vec<Entry> {
    use Style::*;
    vec![
        Entry { ty: "Title screen", style: Fantasy, build: title_centered },
        Entry { ty: "Title screen", style: SciFi, build: title_side },
        Entry { ty: "Title screen", style: Minimal, build: title_press_start },
    ]
}

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
    let foot = g.hbox(col, bl(70.0, 50.0, 520.0, 50.0), "Prompts", 14.0);
    g.key(foot, sz(50.0, 50.0), "Select key", "A");
    g.note(foot, sz(150.0, 50.0), "Select label", "Select", 26.0);
    g.key(foot, sz(50.0, 50.0), "Back key", "B");
    g.note(foot, sz(150.0, 50.0), "Back label", "Back", 26.0);
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
