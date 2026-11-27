//! Gallery: pre-game, mid-game and game-end states.

use super::*;

pub fn entries() -> Vec<Entry> {
    use Style::*;
    vec![
        Entry { ty: "Character select screen", style: Fantasy, build: character_a },
        Entry { ty: "Character select screen", style: SciFi, build: character_b },
        Entry { ty: "Character select screen", style: Cartoon, build: character_c },
        Entry { ty: "Stage / level select", style: Nature, build: stage_a },
        Entry { ty: "Stage / level select", style: Retro, build: stage_b },
        Entry { ty: "Stage / level select", style: Elegant, build: stage_c },
        Entry { ty: "Game setup", style: Minimal, build: setup_a },
        Entry { ty: "Game setup", style: Military, build: setup_b },
        Entry { ty: "Game setup", style: Pastel, build: setup_c },
        Entry { ty: "Pre-game lobby", style: Sports, build: lobby_a },
        Entry { ty: "Pre-game lobby", style: Cartoon, build: lobby_b },
        Entry { ty: "Pre-game lobby", style: Military, build: lobby_c },
        Entry { ty: "Stage intro", style: Fantasy, build: intro_a },
        Entry { ty: "Stage intro", style: Neon, build: intro_b },
        Entry { ty: "Stage intro", style: Horror, build: intro_c },
        Entry { ty: "Pause screen", style: Minimal, build: pause_a },
        Entry { ty: "Pause screen", style: SciFi, build: pause_b },
        Entry { ty: "Pause screen", style: Nature, build: pause_c },
        Entry { ty: "Change class / party", style: Fantasy, build: party_a },
        Entry { ty: "Change class / party", style: Military, build: party_b },
        Entry { ty: "Change class / party", style: Retro, build: party_c },
        Entry { ty: "Mission accept", style: Elegant, build: mission_a },
        Entry { ty: "Mission accept", style: Horror, build: mission_b },
        Entry { ty: "Mission accept", style: SciFi, build: mission_c },
        Entry { ty: "Loot selection", style: Neon, build: loot_a },
        Entry { ty: "Loot selection", style: Nature, build: loot_b },
        Entry { ty: "Loot selection", style: Minimal, build: loot_c },
        Entry { ty: "Failure & game over", style: Horror, build: failure_a },
        Entry { ty: "Failure & game over", style: Military, build: failure_b },
        Entry { ty: "Failure & game over", style: Cartoon, build: failure_c },
        Entry { ty: "Level complete", style: Fantasy, build: complete_a },
        Entry { ty: "Level complete", style: Sports, build: complete_b },
        Entry { ty: "Level complete", style: Pastel, build: complete_c },
        Entry { ty: "Results screen", style: Sports, build: results_a },
        Entry { ty: "Results screen", style: Elegant, build: results_b },
        Entry { ty: "Results screen", style: SciFi, build: results_c },
        Entry { ty: "Rewards & experience", style: Cartoon, build: rewards_a },
        Entry { ty: "Rewards & experience", style: Minimal, build: rewards_b },
        Entry { ty: "Rewards & experience", style: Neon, build: rewards_c },
        Entry { ty: "Leaderboards & ranking", style: Sports, build: ranks_a },
        Entry { ty: "Leaderboards & ranking", style: Retro, build: ranks_b },
        Entry { ty: "Leaderboards & ranking", style: Fantasy, build: ranks_c },
        Entry { ty: "Post-game menu", style: Nature, build: post_a },
        Entry { ty: "Post-game menu", style: Pastel, build: post_b },
        Entry { ty: "Post-game menu", style: Elegant, build: post_c },
    ]
}

fn base(g: &mut G, title: &str) -> N {
    g.background();
    let p = g.page();
    g.bg_title(p, tc(0.0, 35.0, 1500.0, 85.0), "Screen title", title, 54.0);
    p
}

fn modal(g: &mut G) -> N {
    g.background();
    let p = g.page();
    g.art(p, cc(0.0, 0.0, 1500.0, 840.0), "Scene artwork");
    g.dim(150);
    p
}

fn tile(g: &mut G, p: N, pos: Pos, title: &str, detail: &str) -> N {
    let c = g.card(p, pos, "Item card");
    g.heading(c, top(20.0, 50.0, 10.0, 10.0), "Card title", title, 29.0);
    g.note(c, bottom(15.0, 38.0, 10.0, 10.0), "Card detail", detail, 23.0);
    c
}

fn list_row(g: &mut G, p: N, y: f32, title: &str, detail: &str) -> N {
    let c = g.card(p, tl(22.0, y, 446.0, 78.0), "List row");
    g.heading_l(c, tl(16.0, 7.0, 400.0, 34.0), "Row title", title, 27.0);
    g.note(c, tl(16.0, 44.0, 400.0, 27.0), "Row detail", detail, 22.0);
    c
}

fn strong(g: &mut G, n: N) {
    let fill = g.t.line;
    let text = g.t.on_accent;
    g.edit(n, |e| { e.fill = fill; e.text_color = text; });
}

fn character(g: &mut G, v: u8) {
    let p = base(g, "CHOOSE YOUR HERO");
    match v {
        0 => {
            let roster = g.panel(p, tl(55.0, 150.0, 800.0, 840.0), "Portrait roster");
            for i in 0..8 {
                let c = g.card(roster, tl(25.0 + (i % 4) as f32 * 190.0, 90.0 + (i / 4) as f32 * 300.0, 170.0, 270.0), "Hero portrait card");
                g.art(c, tl(10.0, 10.0, 150.0, 190.0), "Hero portrait");
                g.heading(c, bottom(12.0, 48.0, 5.0, 5.0), "Hero name", &["Arin", "Mira", "Taro", "Lena", "Bram", "Eris", "Niko", "Vela"][i], 24.0);
            }
            let d = g.panel(p, tr(55.0, 150.0, 950.0, 840.0), "Selected hero");
            g.art(d, tl(25.0, 25.0, 430.0, 570.0), "Selected hero artwork");
            g.heading_l(d, tl(495.0, 40.0, 400.0, 60.0), "Selected hero name", "MIRA", 44.0);
            for (i, name) in ["Power", "Speed", "Guard"].iter().enumerate() {
                g.label(d, tl(495.0, 145.0 + i as f32 * 100.0, 400.0, 40.0), "Attribute", name, 26.0);
                g.bar(d, tl(495.0, 195.0 + i as f32 * 100.0, 400.0, 25.0), "Attribute bar", 0.55 + i as f32 * 0.12, g.t.accent);
            }
            g.button(d, br(35.0, 35.0, 400.0, 76.0), "Confirm hero button", "Confirm");
        }
        1 => {
            let strip = g.panel(p, tc(0.0, 190.0, 1760.0, 620.0), "Hero carousel");
            for i in 0..5 {
                let c = if i == 2 { g.card_on(strip, tl(30.0 + i as f32 * 346.0, 65.0, 314.0, 480.0), "Selected hero card") } else { g.card(strip, tl(30.0 + i as f32 * 346.0, 105.0, 314.0, 400.0), "Hero card") };
                g.art(c, tl(16.0, 16.0, 282.0, 270.0), "Hero artwork");
                g.heading(c, bottom(65.0, 46.0, 10.0, 10.0), "Hero name", &["Arin", "Lena", "Mira", "Taro", "Vela"][i], 29.0);
                g.note(c, bottom(20.0, 32.0, 12.0, 12.0), "Hero role", &["Scout", "Guard", "Striker", "Support", "Ranger"][i], 24.0);
            }
            g.button(p, bc(0.0, 92.0, 430.0, 76.0), "Confirm hero button", "Confirm Mira");
            g.bg_label(p, bc(0.0, 42.0, 500.0, 32.0), "Carousel hint", "<  Browse heroes  >", 24.0, CENTER);
        }
        _ => {
            for (x, player, hero) in [(60.0, "PLAYER 1", "ARIN"), (1350.0, "PLAYER 2", "VELA")] {
                let c = g.panel(p, tl(x, 180.0, 510.0, 650.0), "Fighter panel");
                g.heading(c, top(20.0, 55.0, 20.0, 20.0), "Player label", player, 32.0);
                g.art(c, tl(35.0, 95.0, 440.0, 430.0), "Fighter portrait");
                g.heading(c, bottom(30.0, 60.0, 15.0, 15.0), "Fighter name", hero, 40.0);
            }
            let grid = g.panel(p, tc(0.0, 265.0, 700.0, 480.0), "Fighter grid");
            for i in 0..12 {
                let c = g.card(grid, tl(20.0 + (i % 4) as f32 * 170.0, 20.0 + (i / 4) as f32 * 150.0, 150.0, 130.0), "Fighter tile");
                g.art(c, fill(8.0, 8.0, 8.0, 8.0), "Fighter thumbnail");
            }
            let vs = g.badge(p, bc(0.0, 72.0, 180.0, 60.0), "Versus badge", "VS"); strong(g, vs);
            let start = g.button(p, bc(0.0, 12.0, 360.0, 60.0), "Start fight button", "Start fight"); strong(g, start);
        }
    }
}

fn stage(g: &mut G, v: u8) {
    let p = base(g, "SELECT STAGE");
    match v {
        0 => {
            let map = g.panel(p, tl(55.0, 150.0, 1250.0, 840.0), "World map");
            g.art(map, fill(20.0, 20.0, 20.0, 20.0), "Map illustration");
            for i in 0..7 { g.badge(map, tl(70.0 + i as f32 * 155.0, 280.0 + (i % 2) as f32 * 150.0, 125.0, 65.0), "Stage node", &format!("{}", i + 1)); }
            let d = g.panel(p, tr(55.0, 150.0, 530.0, 840.0), "Stage information");
            g.art(d, tl(25.0, 25.0, 480.0, 350.0), "Stage preview");
            g.heading(d, tl(25.0, 400.0, 480.0, 58.0), "Stage name", "Briar Valley", 38.0);
            g.label(d, tl(30.0, 500.0, 460.0, 45.0), "Stage objective", "Reach the old gate", 26.0);
            g.badge(d, tl(30.0, 570.0, 210.0, 55.0), "Stage progress", "3 / 7");
            g.button(d, br(25.0, 30.0, 480.0, 76.0), "Start stage button", "Enter stage");
        }
        1 => {
            let tabs = g.panel(p, top(145.0, 90.0, 70.0, 70.0), "Chapter tabs");
            for (i, name) in ["Chapter I", "Chapter II", "Chapter III"].iter().enumerate() { g.button2(tabs, tl(28.0 + i as f32 * 400.0, 14.0, 375.0, 62.0), "Chapter tab", name); }
            let board = g.panel(p, tl(70.0, 260.0, 1780.0, 730.0), "Stage grid");
            for i in 0..8 {
                let c = g.card(board, tl(28.0 + (i % 4) as f32 * 440.0, 22.0 + (i / 4) as f32 * 350.0, 410.0, 330.0), "Stage card");
                g.art(c, tl(12.0, 12.0, 386.0, 210.0), "Level thumbnail");
                g.heading_l(c, tl(20.0, 232.0, 350.0, 42.0), "Level name", &format!("Stage {:02}", i + 1), 28.0);
                g.note(c, tl(20.0, 282.0, 350.0, 32.0), "Level rating", if i < 5 { "***" } else { "Locked" }, 24.0);
            }
        }
        _ => {
            let list = g.panel(p, tl(70.0, 155.0, 510.0, 830.0), "Stage list");
            for (i, name) in ["Sunken Road", "Old Watch", "Glass Coast", "High Pass", "Final Gate"].iter().enumerate() { list_row(g, list, 70.0 + i as f32 * 140.0, name, &format!("Stage {}", i + 1)); }
            let d = g.panel(p, tr(70.0, 155.0, 1180.0, 830.0), "Selected stage");
            g.art(d, tl(30.0, 25.0, 1120.0, 470.0), "Stage preview");
            g.heading_l(d, tl(36.0, 520.0, 700.0, 60.0), "Selected stage name", "Old Watch", 44.0);
            g.note(d, tl(36.0, 605.0, 600.0, 45.0), "Best time", "Best time  04:32", 28.0);
            g.button(d, br(35.0, 35.0, 420.0, 80.0), "Start stage button", "Start");
        }
    }
}

fn setup(g: &mut G, v: u8) {
    let p = base(g, "GAME SETUP");
    match v {
        0 => {
            let c = g.panel(p, cc(0.0, 25.0, 1500.0, 800.0), "Rules form");
            for (i, name) in ["Game mode", "Map", "Round length", "Difficulty"].iter().enumerate() {
                let x = 45.0 + (i % 2) as f32 * 730.0; let y = 90.0 + (i / 2) as f32 * 210.0;
                g.label(c, tl(x, y, 650.0, 44.0), "Rule label", name, 28.0);
                g.dropdown(c, tl(x, y + 52.0, 650.0, 70.0), "Rule selector", &["Standard", "Custom", "Random"]);
            }
            g.toggle(c, tl(50.0, 570.0, 600.0, 64.0), "Friendly fire toggle", "Friendly fire", false);
            g.toggle(c, tl(780.0, 570.0, 600.0, 64.0), "Teams toggle", "Balanced teams", true);
            g.button(c, br(48.0, 40.0, 400.0, 78.0), "Start match button", "Start match");
        }
        1 => {
            let steps = g.panel(p, tl(65.0, 155.0, 1120.0, 820.0), "Setup wizard");
            for (i, name) in ["1 MODE", "2 MAP", "3 RULES"].iter().enumerate() { g.button2(steps, tl(25.0 + i as f32 * 358.0, 25.0, 335.0, 70.0), "Wizard step", name); }
            g.heading_l(steps, tl(50.0, 145.0, 800.0, 60.0), "Step title", "Choose a mode", 44.0);
            for (i, name) in ["Skirmish", "Survival", "Capture"].iter().enumerate() { tile(g, steps, tl(50.0 + i as f32 * 345.0, 245.0, 325.0, 360.0), name, "2-8 players"); }
            g.button(steps, br(35.0, 35.0, 280.0, 70.0), "Next step button", "Next");
            let d = g.panel(p, tr(65.0, 155.0, 635.0, 820.0), "Match summary");
            g.heading(d, top(40.0, 60.0, 25.0, 25.0), "Summary title", "SUMMARY", 34.0);
            for (i, name) in ["Mode: Skirmish", "Map: Random", "Rules: Standard", "Players: 4"].iter().enumerate() { g.label(d, tl(35.0, 150.0 + i as f32 * 100.0, 565.0, 52.0), "Summary field", name, 28.0); }
        }
        _ => {
            for i in 0..4 {
                let c = g.panel(p, tl(65.0 + i as f32 * 455.0, 195.0, 425.0, 545.0), "Player slot");
                g.art(c, tl(25.0, 25.0, 375.0, 300.0), "Player avatar");
                g.heading(c, tl(20.0, 350.0, 385.0, 54.0), "Player number", &format!("PLAYER {}", i + 1), 32.0);
                g.note(c, tl(20.0, 420.0, 385.0, 60.0), "Join prompt", if i == 0 { "Ready" } else { "Press A to join" }, 26.0);
            }
            let c = g.panel(p, bottom(65.0, 215.0, 65.0, 65.0), "Match options");
            g.label(c, tl(35.0, 35.0, 350.0, 50.0), "Mode label", "Mode: Skirmish", 28.0);
            g.label(c, tl(500.0, 35.0, 350.0, 50.0), "Map label", "Map: Random", 28.0);
            g.button(c, br(35.0, 32.0, 360.0, 76.0), "Continue button", "Continue");
        }
    }
}

fn lobby(g: &mut G, v: u8) {
    let p = base(g, "MATCH LOBBY");
    match v {
        0 => {
            for (x, team) in [(65.0, "BLUE TEAM"), (1000.0, "RED TEAM")] {
                let c = g.panel(p, tl(x, 165.0, 850.0, 600.0), "Team panel");
                g.heading(c, top(28.0, 60.0, 20.0, 20.0), "Team name", team, 34.0);
                for (i, name) in ["Arin", "Mira", "Taro", "Lena"].iter().enumerate() {
                    let r = g.card(c, tl(30.0, 115.0 + i as f32 * 112.0, 790.0, 92.0), "Player row");
                    g.icon(r, tl(16.0, 14.0, 60.0, 60.0), "Player avatar", &format!("{}", i + 1));
                    g.label(r, tl(95.0, 20.0, 400.0, 50.0), "Player name", name, 30.0);
                    g.badge(r, tr(18.0, 20.0, 165.0, 48.0), "Ready badge", if i == 3 { "WAITING" } else { "READY" });
                }
            }
            let c = g.panel(p, bottom(45.0, 220.0, 65.0, 65.0), "Lobby chat");
            g.note(c, tl(25.0, 20.0, 1200.0, 40.0), "Chat message", "Mira: Ready when you are.", 26.0);
            g.input(c, bl(25.0, 22.0, 1300.0, 60.0), "Chat input", "Send a message...");
            g.badge(c, tr(25.0, 25.0, 230.0, 60.0), "Countdown", "00:18");
        }
        1 => {
            let mode = g.panel(p, tl(65.0, 170.0, 440.0, 720.0), "Mode panel");
            g.art(mode, tl(20.0, 20.0, 400.0, 330.0), "Mode artwork");
            g.heading(mode, tl(20.0, 380.0, 400.0, 55.0), "Mode name", "SKIRMISH", 34.0);
            g.note(mode, tl(30.0, 460.0, 380.0, 115.0), "Mode description", "First team to 50 points wins.", 26.0);
            for i in 0..4 {
                let c = g.panel(p, tl(540.0 + i as f32 * 325.0, 250.0, 300.0, 530.0), "Player platform");
                g.art(c, tl(20.0, 20.0, 260.0, 350.0), "Hero artwork");
                g.heading(c, bottom(65.0, 48.0, 10.0, 10.0), "Player name", &["Arin", "Mira", "Taro", "Lena"][i], 28.0);
                let status = g.badge(c, bc(0.0, 18.0, 180.0, 44.0), "Player status", "READY"); strong(g, status);
            }
            let ready = g.button(p, bc(0.0, 68.0, 380.0, 80.0), "Ready button", "Ready up"); strong(g, ready);
        }
        _ => {
            let map = g.panel(p, tl(70.0, 160.0, 1000.0, 645.0), "Briefing map");
            g.art(map, fill(20.0, 20.0, 20.0, 20.0), "Mission map");
            for i in 0..3 { g.badge(map, tl(140.0 + i as f32 * 260.0, 160.0 + i as f32 * 100.0, 100.0, 58.0), "Objective marker", &format!("{}", i + 1)); }
            let c = g.panel(p, tr(70.0, 160.0, 730.0, 645.0), "Objectives panel");
            g.heading_l(c, tl(30.0, 30.0, 650.0, 60.0), "Objectives title", "OBJECTIVES", 36.0);
            for (i, name) in ["Secure the relay", "Hold the ridge", "Extract the team"].iter().enumerate() { g.label(c, tl(35.0, 140.0 + i as f32 * 115.0, 635.0, 65.0), "Objective", &format!("{}. {}", i + 1, name), 28.0); }
            let gear = g.panel(p, bottom(50.0, 170.0, 70.0, 70.0), "Equipment strip");
            for i in 0..4 { g.slot(gear, tl(25.0 + i as f32 * 120.0, 30.0, 96.0, 96.0), "Equipment slot"); }
            g.button(gear, br(30.0, 34.0, 380.0, 76.0), "Deploy button", "Deploy");
        }
    }
}

fn intro(g: &mut G, v: u8) {
    let p = modal(g);
    match v {
        0 => {
            let c = g.panel(p, cc(0.0, 0.0, 1350.0, 450.0), "World title plate");
            g.heading(c, tl(40.0, 65.0, 1270.0, 130.0), "World number", "WORLD 2-3", 96.0);
            g.heading(c, tl(40.0, 220.0, 1270.0, 70.0), "Stage name", "THE OLD WATCH", 48.0);
            g.note(c, tl(60.0, 330.0, 1230.0, 50.0), "Stage objective", "Objective: reach the beacon", 28.0);
            for i in 0..3 { g.circle(c, tl(575.0 + i as f32 * 55.0, 405.0, 22.0, 22.0), "Progress dot", g.t.accent); }
        }
        1 => {
            let c = g.panel(p, hmid(0.0, 385.0, 0.0, 0.0), "Versus banner");
            let stage = g.card(c, tl(65.0, 45.0, 1080.0, 295.0), "Stage title card");
            g.heading_l(stage, tl(35.0, 35.0, 950.0, 85.0), "Stage name", "NEON CITADEL", 64.0);
            g.note(stage, tl(35.0, 150.0, 950.0, 55.0), "Stage subtitle", "Round 04  /  Final approach", 28.0);
            g.badge(c, tl(1190.0, 140.0, 140.0, 90.0), "Versus badge", "VS");
            let boss = g.card(c, tr(65.0, 45.0, 490.0, 295.0), "Boss card");
            g.art(boss, tl(20.0, 20.0, 180.0, 250.0), "Boss portrait");
            g.heading(boss, tl(210.0, 70.0, 250.0, 120.0), "Boss name", "WARDEN", 38.0);
        }
        _ => {
            let c = g.panel(p, bl(75.0, 85.0, 940.0, 310.0), "Location title strip");
            g.badge(c, tl(30.0, 25.0, 180.0, 52.0), "Chapter badge", "CHAPTER 03");
            g.heading_l(c, tl(30.0, 105.0, 850.0, 85.0), "Location name", "ASHEN HARBOR", 62.0);
            g.note(c, tl(35.0, 210.0, 850.0, 52.0), "Location time", "Nightfall  /  21:45", 30.0);
            g.badge(p, tr(75.0, 85.0, 210.0, 56.0), "Area status", "NEW AREA");
            g.circle(p, tr(100.0, 190.0, 110.0, 110.0), "Area emblem", g.t.accent);
            g.rect(c, bottom(20.0, 5.0, 30.0, 30.0), "Accent rule", g.t.accent, 0.0);
        }
    }
}

fn pause(g: &mut G, v: u8) {
    let p = modal(g);
    match v {
        0 => {
            let c = g.panel(p, cc(0.0, 0.0, 580.0, 770.0), "Pause menu card");
            g.heading(c, top(45.0, 90.0, 25.0, 25.0), "Pause heading", "PAUSED", 58.0);
            for (i, name) in ["Resume", "Restart", "Options", "Quit"].iter().enumerate() { g.menu_item(c, tl(55.0, 180.0 + i as f32 * 112.0, 470.0, 80.0), "Pause option", name, i == 0); }
            g.note(c, bottom(34.0, 42.0, 55.0, 55.0), "Pause hint", "Take a breath.", 24.0);
        }
        1 => {
            let side = g.panel(p, left(0.0, 600.0, 0.0, 0.0), "Pause sidebar");
            g.heading(side, top(75.0, 90.0, 35.0, 35.0), "Pause heading", "PAUSED", 54.0);
            for (i, name) in ["Resume", "Map", "Objectives", "Options", "Quit"].iter().enumerate() { g.menu_item(side, tl(55.0, 210.0 + i as f32 * 118.0, 490.0, 82.0), "Pause option", name, i == 0); }
            let c = g.panel(p, tr(70.0, 100.0, 1100.0, 810.0), "Current objective panel");
            g.art(c, tl(25.0, 25.0, 1050.0, 480.0), "Current map");
            g.heading_l(c, tl(30.0, 550.0, 1000.0, 55.0), "Objective title", "Reach the northern gate", 38.0);
            g.note(c, tl(30.0, 635.0, 1000.0, 55.0), "Objective detail", "The path opens beyond the old tower.", 27.0);
        }
        _ => {
            let c = g.panel(p, cc(0.0, 0.0, 1570.0, 850.0), "Tabbed pause panel");
            for (i, name) in ["Map", "Inventory", "Quests", "Settings"].iter().enumerate() { g.button2(c, tl(25.0 + i as f32 * 380.0, 25.0, 355.0, 68.0), "Pause tab", name); }
            let d = g.card(c, tl(35.0, 125.0, 1500.0, 540.0), "Pause summary");
            g.art(d, tl(25.0, 25.0, 620.0, 490.0), "Area map");
            g.heading_l(d, tl(700.0, 55.0, 720.0, 70.0), "Summary title", "CURRENT AREA", 42.0);
            g.label(d, tl(700.0, 170.0, 720.0, 60.0), "Area name", "Briar Valley", 32.0);
            g.note(d, tl(700.0, 260.0, 720.0, 100.0), "Quest summary", "Find the eastern passage.", 27.0);
            g.button(c, bl(40.0, 45.0, 330.0, 75.0), "Resume button", "Resume");
            g.button2(c, br(40.0, 45.0, 330.0, 75.0), "Quit button", "Quit");
        }
    }
}

fn party(g: &mut G, v: u8) {
    let p = base(g, "MANAGE PARTY");
    match v {
        0 => {
            let active = g.panel(p, top(155.0, 300.0, 70.0, 70.0), "Active party");
            for i in 0..4 {
                let c = g.card(active, tl(25.0 + i as f32 * 445.0, 25.0, 420.0, 250.0), "Party member");
                g.art(c, tl(15.0, 15.0, 170.0, 220.0), "Member portrait");
                g.heading_l(c, tl(200.0, 30.0, 200.0, 45.0), "Member name", &["Arin", "Mira", "Taro", "Lena"][i], 28.0);
                g.bar(c, tl(200.0, 125.0, 190.0, 25.0), "Health bar", 0.7, g.t.good);
                g.note(c, tl(200.0, 175.0, 190.0, 40.0), "Member level", "Level 18", 24.0);
            }
            let available = g.panel(p, tl(70.0, 480.0, 980.0, 510.0), "Available members");
            for (i, name) in ["Eris", "Bram", "Niko"].iter().enumerate() { list_row(g, available, 60.0 + i as f32 * 145.0, name, "Available"); }
            let d = g.panel(p, tr(70.0, 480.0, 760.0, 510.0), "Stat comparison");
            g.heading(d, top(28.0, 58.0, 20.0, 20.0), "Comparison title", "COMPARISON", 32.0);
            for (i, name) in ["Power  68  >  74", "Guard  52  >  61", "Speed  80  >  73"].iter().enumerate() { g.label(d, tl(42.0, 120.0 + i as f32 * 90.0, 680.0, 60.0), "Compared stat", name, 28.0); }
            g.button(d, br(35.0, 28.0, 300.0, 70.0), "Swap member button", "Swap member");
        }
        1 => {
            let list = g.panel(p, tl(70.0, 155.0, 510.0, 840.0), "Class list");
            for (i, name) in ["Vanguard", "Medic", "Ranger", "Engineer", "Scout"].iter().enumerate() { let r = list_row(g, list, 65.0 + i as f32 * 143.0, name, "Class available"); g.icon(r, tr(12.0, 12.0, 50.0, 50.0), "Class icon", "+"); }
            let d = g.panel(p, tr(70.0, 155.0, 1220.0, 840.0), "Class detail");
            g.art(d, tl(35.0, 35.0, 430.0, 610.0), "Class artwork");
            g.heading_l(d, tl(510.0, 50.0, 660.0, 60.0), "Class name", "VANGUARD", 48.0);
            g.note(d, tl(510.0, 130.0, 650.0, 45.0), "Class role", "Frontline defender", 28.0);
            for (i, name) in ["Shield rush", "Guard stance", "Last stand"].iter().enumerate() { g.label(d, tl(520.0, 240.0 + i as f32 * 110.0, 600.0, 60.0), "Class ability", name, 28.0); }
            g.button(d, br(35.0, 35.0, 370.0, 78.0), "Equip class button", "Equip class");
        }
        _ => {
            let formation = g.panel(p, tl(70.0, 160.0, 1160.0, 655.0), "Formation grid");
            for i in 0..9 {
                let c = g.slot(formation, tl(80.0 + (i % 3) as f32 * 350.0, 30.0 + (i / 3) as f32 * 210.0, 270.0, 175.0), "Formation position");
                if i < 4 { g.icon(c, tc(0.0, 28.0, 70.0, 70.0), "Unit icon", &format!("{}", i + 1)); g.heading(c, bottom(18.0, 42.0, 12.0, 12.0), "Unit name", &["Arin", "Mira", "Taro", "Lena"][i], 25.0); }
            }
            let orders = g.panel(p, tr(70.0, 160.0, 560.0, 655.0), "Formation orders");
            g.heading(orders, top(35.0, 60.0, 20.0, 20.0), "Orders title", "FORMATION", 34.0);
            for (i, name) in ["Front: defend", "Middle: support", "Back: ranged"].iter().enumerate() { g.label(orders, tl(35.0, 150.0 + i as f32 * 110.0, 490.0, 55.0), "Formation order", name, 27.0); }
            let bench = g.panel(p, bottom(45.0, 155.0, 70.0, 70.0), "Reserve bench");
            for (i, name) in ["Eris", "Bram", "Niko", "Vela"].iter().enumerate() { tile(g, bench, tl(20.0 + i as f32 * 240.0, 20.0, 220.0, 110.0), name, "Reserve"); }
            g.button(bench, br(25.0, 30.0, 350.0, 76.0), "Save formation button", "Save formation");
        }
    }
}

fn mission(g: &mut G, v: u8) {
    let p = if v == 2 { modal(g) } else { base(g, "AVAILABLE MISSIONS") };
    match v {
        0 => {
            let c = g.panel(p, cc(0.0, 30.0, 1450.0, 790.0), "Mission conversation");
            g.art(c, tl(30.0, 30.0, 400.0, 570.0), "Quest giver portrait");
            g.heading_l(c, tl(470.0, 45.0, 930.0, 60.0), "Quest giver name", "THE CARTOGRAPHER", 42.0);
            g.label(c, tl(470.0, 135.0, 900.0, 120.0), "Mission description", "The beacon has gone quiet. Find its keeper and restore the signal.", 30.0);
            g.badge(c, tl(470.0, 300.0, 280.0, 60.0), "Reward badge", "400 GOLD");
            g.badge(c, tl(780.0, 300.0, 280.0, 60.0), "Experience badge", "800 XP");
            g.button2(c, bl(45.0, 38.0, 320.0, 76.0), "Decline mission button", "Decline");
            g.button(c, br(45.0, 38.0, 320.0, 76.0), "Accept mission button", "Accept");
        }
        1 => {
            let board = g.panel(p, tl(70.0, 155.0, 1000.0, 840.0), "Contract board");
            for (i, name) in ["Beacon run", "Guard the pass", "Missing supplies", "Silent tower"].iter().enumerate() {
                let c = g.card(board, tl(28.0 + (i % 2) as f32 * 485.0, 30.0 + (i / 2) as f32 * 390.0, 455.0, 360.0), "Contract card");
                g.art(c, tl(15.0, 15.0, 425.0, 175.0), "Contract artwork");
                g.heading_l(c, tl(20.0, 210.0, 405.0, 45.0), "Contract name", name, 28.0);
                g.note(c, tl(20.0, 275.0, 405.0, 45.0), "Contract pay", "Risk: Medium  /  Pay: 400", 23.0);
            }
            let d = g.panel(p, tr(70.0, 155.0, 700.0, 840.0), "Contract detail");
            g.heading(d, top(35.0, 70.0, 25.0, 25.0), "Detail title", "BEACON RUN", 40.0);
            g.label(d, tl(40.0, 150.0, 620.0, 180.0), "Contract description", "Reach the beacon before nightfall and restore the relay.", 29.0);
            g.note(d, tl(40.0, 420.0, 620.0, 65.0), "Contract difficulty", "Difficulty: Medium", 27.0);
            g.badge(d, tl(40.0, 520.0, 230.0, 58.0), "Contract reward", "400 GOLD");
            g.button(d, br(35.0, 35.0, 330.0, 78.0), "Accept contract button", "Accept");
        }
        _ => {
            let c = g.panel(p, cc(0.0, 0.0, 940.0, 760.0), "Mission modal");
            g.heading(c, top(35.0, 70.0, 30.0, 30.0), "Mission title", "RESTORE THE RELAY", 44.0);
            g.note(c, tl(45.0, 135.0, 850.0, 75.0), "Recommended level", "Recommended level: 18", 28.0);
            for (i, name) in ["Find the tower", "Repair the relay", "Return to camp"].iter().enumerate() {
                let r = g.card(c, tl(45.0, 240.0 + i as f32 * 105.0, 850.0, 84.0), "Mission objective row");
                g.label(r, tl(20.0, 17.0, 800.0, 50.0), "Mission objective", &format!("{}. {}", i + 1, name), 28.0);
            }
            g.button2(c, bl(45.0, 38.0, 300.0, 76.0), "Cancel mission button", "Later");
            g.button(c, br(45.0, 38.0, 300.0, 76.0), "Accept mission button", "Accept");
        }
    }
}

fn loot(g: &mut G, v: u8) {
    let p = base(g, "LOOT FOUND");
    match v {
        0 => {
            for (i, (name, rarity)) in [("Ember blade", "RARE"), ("Moon guard", "EPIC"), ("Swift boots", "UNCOMMON")].iter().enumerate() {
                let c = g.panel(p, tl(105.0 + i as f32 * 590.0, 195.0, 540.0, 690.0), "Loot choice card");
                g.badge(c, tc(0.0, 24.0, 210.0, 54.0), "Rarity badge", rarity);
                g.art(c, tl(45.0, 100.0, 450.0, 365.0), "Item artwork");
                g.heading(c, tl(30.0, 490.0, 480.0, 55.0), "Item name", name, 34.0);
                g.note(c, tl(35.0, 555.0, 470.0, 45.0), "Item stat", "+12 power", 26.0);
                g.button(c, bc(0.0, 25.0, 340.0, 66.0), "Choose item button", "Choose");
            }
        }
        1 => {
            let chest = g.panel(p, tl(70.0, 165.0, 560.0, 825.0), "Chest panel");
            g.art(chest, tl(35.0, 35.0, 490.0, 550.0), "Chest artwork");
            g.heading(chest, tl(20.0, 625.0, 520.0, 60.0), "Chest label", "SUPPLY CHEST", 36.0);
            let list = g.panel(p, tr(70.0, 165.0, 1170.0, 825.0), "Chest contents");
            g.heading_l(list, tl(30.0, 30.0, 1000.0, 60.0), "Contents title", "CONTENTS", 42.0);
            for (i, name) in ["Iron ore x12", "Healing tonic x3", "Old key x1", "Silver coins x80"].iter().enumerate() {
                let r = g.card(list, tl(32.0, 115.0 + i as f32 * 120.0, 1105.0, 90.0), "Loot row");
                g.toggle(r, tl(18.0, 16.0, 1000.0, 58.0), "Take item toggle", name, true);
            }
            g.button(list, br(35.0, 35.0, 350.0, 76.0), "Take all button", "Take all");
        }
        _ => {
            for (x, title) in [(70.0, "YOUR INVENTORY"), (750.0, "CONTAINER")] {
                let c = g.panel(p, tl(x, 165.0, 620.0, 825.0), "Inventory column");
                g.heading(c, top(25.0, 55.0, 18.0, 18.0), "Inventory title", title, 32.0);
                for i in 0..6 {
                    let r = g.card(c, tl(25.0, 95.0 + i as f32 * 112.0, 570.0, 88.0), "Inventory item");
                    g.icon(r, tl(14.0, 14.0, 60.0, 60.0), "Item icon", "+");
                    g.label(r, tl(90.0, 18.0, 440.0, 50.0), "Item name", &format!("Supply item {}", i + 1), 26.0);
                }
            }
            let d = g.panel(p, tr(70.0, 165.0, 410.0, 825.0), "Selected item detail");
            g.art(d, tl(25.0, 25.0, 360.0, 320.0), "Selected item artwork");
            g.heading(d, tl(25.0, 380.0, 360.0, 62.0), "Selected item name", "IRON ORE", 30.0);
            g.note(d, tl(30.0, 475.0, 350.0, 90.0), "Selected item description", "A useful crafting material.", 25.0);
            g.button(d, br(25.0, 30.0, 360.0, 72.0), "Transfer item button", "Transfer");
        }
    }
}

fn failure(g: &mut G, v: u8) {
    let p = modal(g);
    match v {
        0 => {
            let c = g.panel(p, cc(0.0, -135.0, 1200.0, 300.0), "Game over title plate");
            g.heading(c, tl(25.0, 65.0, 1150.0, 145.0), "Game over title", "GAME OVER", 104.0);
            g.note(c, bottom(25.0, 55.0, 20.0, 20.0), "Failure subtitle", "The journey ends here", 28.0);
            g.badge(p, tc(0.0, 75.0, 210.0, 56.0), "Attempt badge", "RUN 04");
            g.badge(p, bc(0.0, 310.0, 260.0, 56.0), "Final score badge", "SCORE  2,400");
            g.button(p, bc(-220.0, 170.0, 390.0, 86.0), "Retry button", "Retry");
            g.button2(p, bc(220.0, 170.0, 390.0, 86.0), "Quit button", "Quit");
        }
        1 => {
            let c = g.panel(p, cc(0.0, 0.0, 1120.0, 820.0), "Mission failed panel");
            g.heading(c, top(42.0, 85.0, 30.0, 30.0), "Failure title", "MISSION FAILED", 60.0);
            g.label(c, tl(55.0, 165.0, 1000.0, 60.0), "Failure cause", "Cause: Relay destroyed", 34.0);
            let stats = g.card(c, tl(50.0, 280.0, 1020.0, 245.0), "Failure statistics");
            for (i, name) in ["Time survived  08:42", "Objectives  2 / 3", "Enemies defeated  24"].iter().enumerate() { g.label(stats, tl(25.0, 20.0 + i as f32 * 70.0, 950.0, 55.0), "Failure stat", name, 28.0); }
            g.note(c, tl(55.0, 555.0, 1000.0, 60.0), "Retry tip", "Tip: defend the relay from the ridge.", 27.0);
            g.button(c, bc(0.0, 42.0, 650.0, 78.0), "Retry checkpoint button", "Retry from checkpoint");
        }
        _ => {
            let c = g.panel(p, cc(0.0, 0.0, 760.0, 760.0), "Continue panel");
            g.heading(c, top(36.0, 85.0, 25.0, 25.0), "Continue title", "CONTINUE?", 56.0);
            g.circle(c, tc(0.0, 165.0, 310.0, 310.0), "Countdown ring", g.t.accent);
            let num = g.card(c, tc(0.0, 195.0, 250.0, 250.0), "Countdown number plate");
            g.heading(num, full(), "Countdown number", "9", 120.0);
            let coin = g.badge(c, tc(0.0, 505.0, 250.0, 60.0), "Coin cost", "1 COIN"); strong(g, coin);
            let yes = g.button(c, bl(42.0, 42.0, 295.0, 76.0), "Continue yes button", "Yes"); strong(g, yes);
            g.button2(c, br(42.0, 42.0, 295.0, 76.0), "Continue no button", "No");
        }
    }
}

fn complete(g: &mut G, v: u8) {
    let p = base(g, "LEVEL COMPLETE");
    match v {
        0 => {
            let c = g.panel(p, cc(0.0, 30.0, 1100.0, 800.0), "Completion panel");
            for i in 0..3 { g.badge(c, tl(180.0 + i as f32 * 260.0, 110.0, 210.0, 145.0), "Star badge", "*"); }
            g.heading(c, tl(100.0, 330.0, 900.0, 100.0), "Score", "SCORE  24,800", 56.0);
            g.note(c, tl(100.0, 455.0, 900.0, 60.0), "Score detail", "New personal best!", 30.0);
            g.button2(c, bl(55.0, 50.0, 400.0, 82.0), "Replay level button", "Replay");
            g.button(c, br(55.0, 50.0, 400.0, 82.0), "Next level button", "Next level");
        }
        1 => {
            let banner = g.panel(p, top(180.0, 180.0, 0.0, 0.0), "Stage clear banner");
            g.heading(banner, full(), "Stage clear title", "STAGE CLEAR", 94.0);
            let stats = g.panel(p, cc(0.0, 95.0, 1380.0, 550.0), "Clear stats");
            for (i, (name, value)) in [("TIME", "04:32"), ("COLLECTED", "18 / 20"), ("RANK", "A")].iter().enumerate() {
                let c = g.card(stats, tl(35.0 + i as f32 * 445.0, 70.0, 420.0, 390.0), "Clear stat card");
                g.heading(c, top(65.0, 55.0, 15.0, 15.0), "Stat name", name, 30.0);
                g.heading(c, cc(0.0, 40.0, 370.0, 110.0), "Stat value", value, 56.0);
            }
            g.button(p, bc(0.0, 65.0, 440.0, 78.0), "Continue button", "Continue");
        }
        _ => {
            let c = g.panel(p, cc(0.0, 25.0, 1250.0, 830.0), "Objective recap");
            g.heading(c, top(35.0, 65.0, 25.0, 25.0), "Recap title", "OBJECTIVES COMPLETE", 44.0);
            for (i, name) in ["Reach the tower", "Restore the signal", "Find the cache", "Return safely"].iter().enumerate() {
                let r = g.card(c, tl(55.0, 135.0 + i as f32 * 105.0, 1140.0, 80.0), "Completed objective");
                g.badge(r, tl(18.0, 17.0, 55.0, 46.0), "Objective check", "+");
                g.label(r, tl(100.0, 15.0, 980.0, 50.0), "Objective name", name, 28.0);
            }
            g.badge(c, bl(55.0, 45.0, 350.0, 65.0), "Total reward", "TOTAL  1,200 XP");
            g.button(c, br(55.0, 45.0, 350.0, 75.0), "Continue button", "Continue");
        }
    }
}

fn results(g: &mut G, v: u8) {
    let p = base(g, "MATCH RESULTS");
    match v {
        0 => {
            let c = g.panel(p, cc(0.0, 25.0, 1610.0, 830.0), "Results table");
            g.heading_l(c, tl(30.0, 30.0, 1500.0, 55.0), "Table columns", "RANK     PLAYER                 KILLS       DEATHS       SCORE", 30.0);
            for i in 0..6 {
                let r = if i == 1 { g.card_on(c, tl(30.0, 110.0 + i as f32 * 110.0, 1550.0, 88.0), "Your results row") } else { g.card(c, tl(30.0, 110.0 + i as f32 * 110.0, 1550.0, 88.0), "Player results row") };
                g.label(r, tl(22.0, 20.0, 1450.0, 50.0), "Player results", &format!("{:02}        {:<14}             {:02}                {:02}             {}", i + 1, ["Arin", "You", "Mira", "Taro", "Lena", "Bram"][i], 18 - i, i + 2, 2800 - i * 300), 27.0);
            }
        }
        1 => {
            let podium = g.panel(p, top(175.0, 520.0, 190.0, 190.0), "Winner podium");
            for (i, (name, height)) in [("Mira", 300.0), ("Arin", 400.0), ("Taro", 230.0)].iter().enumerate() {
                let c = g.card(podium, bl(115.0 + i as f32 * 460.0, 30.0, 370.0, *height), "Podium place");
                g.art(c, tc(0.0, 25.0, 150.0, 145.0), "Winner portrait");
                g.heading(c, bottom(70.0, 50.0, 10.0, 10.0), "Winner name", name, 32.0);
                g.badge(c, bc(0.0, 17.0, 140.0, 46.0), "Place badge", &["2ND", "1ST", "3RD"][i]);
            }
            let list = g.panel(p, bottom(55.0, 260.0, 190.0, 190.0), "Remaining players");
            for (i, name) in ["4  Lena", "5  Bram", "6  Vela"].iter().enumerate() { g.label(list, tl(50.0, 30.0 + i as f32 * 70.0, 1200.0, 50.0), "Player rank", name, 28.0); }
        }
        _ => {
            let chart = g.panel(p, tl(85.0, 165.0, 1160.0, 825.0), "Performance chart");
            g.heading_l(chart, tl(40.0, 35.0, 1000.0, 60.0), "Chart title", "PERFORMANCE", 42.0);
            for (i, (name, val)) in [("Accuracy", 0.74), ("Damage", 0.86), ("Support", 0.52), ("Survival", 0.68)].iter().enumerate() {
                g.label(chart, tl(45.0, 145.0 + i as f32 * 145.0, 300.0, 45.0), "Stat name", name, 28.0);
                g.bar(chart, tl(340.0, 155.0 + i as f32 * 145.0, 720.0, 35.0), "Stat bar", *val, g.t.accent);
            }
            let d = g.panel(p, tr(85.0, 165.0, 545.0, 825.0), "Personal records");
            g.heading(d, top(35.0, 65.0, 25.0, 25.0), "Records title", "NEW RECORDS", 34.0);
            for (i, name) in ["Best accuracy", "Longest streak", "Most assists"].iter().enumerate() { tile(g, d, tl(30.0, 135.0 + i as f32 * 180.0, 485.0, 145.0), name, "Personal best"); }
        }
    }
}

fn rewards(g: &mut G, v: u8) {
    let p = base(g, "REWARDS & EXPERIENCE");
    match v {
        0 => {
            let xp = g.panel(p, top(170.0, 300.0, 120.0, 120.0), "Experience panel");
            let level = g.badge(xp, tl(35.0, 35.0, 260.0, 72.0), "Level badge", "LEVEL 19"); strong(g, level);
            g.heading_l(xp, tl(335.0, 42.0, 900.0, 65.0), "Level up message", "LEVEL UP!", 52.0);
            g.bar(xp, tl(335.0, 155.0, 1180.0, 45.0), "Experience bar", 0.76, g.t.accent);
            g.note(xp, tl(335.0, 220.0, 900.0, 40.0), "Experience progress", "7,600 / 10,000 XP", 27.0);
            let items = g.panel(p, bottom(90.0, 470.0, 120.0, 120.0), "Earned items");
            for (i, name) in ["Ember blade", "Supply pack", "Silver key", "Lucky charm"].iter().enumerate() {
                let c = tile(g, items, tl(30.0 + i as f32 * 420.0, 40.0, 390.0, 365.0), name, "New item");
                g.art(c, tl(40.0, 95.0, 310.0, 180.0), "Item artwork");
            }
        }
        1 => {
            let c = g.panel(p, cc(0.0, 20.0, 980.0, 830.0), "Rewards ledger");
            g.heading(c, top(35.0, 65.0, 25.0, 25.0), "Ledger title", "EARNED THIS MATCH", 40.0);
            for (i, (name, amount)) in [("Coins", "+ 480"), ("Gems", "+ 12"), ("Experience", "+ 1,250")].iter().enumerate() {
                let r = g.card(c, tl(45.0, 150.0 + i as f32 * 145.0, 890.0, 112.0), "Reward line");
                g.label(r, tl(30.0, 25.0, 450.0, 60.0), "Reward name", name, 32.0);
                g.value(r, tr(30.0, 25.0, 340.0, 60.0), "Reward amount", amount, 36.0);
            }
            g.badge(c, bl(50.0, 45.0, 410.0, 68.0), "Reward total", "TOTAL  1,742");
            g.button(c, br(45.0, 40.0, 330.0, 76.0), "Collect rewards button", "Collect");
        }
        _ => {
            let c = g.panel(p, cc(0.0, 30.0, 1700.0, 780.0), "Season pass track");
            g.heading_l(c, tl(45.0, 35.0, 1100.0, 60.0), "Pass title", "SEASON PROGRESS", 46.0);
            g.bar(c, tl(50.0, 130.0, 1590.0, 38.0), "Pass progress", 0.62, g.t.accent);
            for (i, level) in ["18", "19", "20", "21", "22"].iter().enumerate() {
                let r = g.card(c, tl(45.0 + i as f32 * 330.0, 230.0, 305.0, 360.0), "Pass reward tier");
                g.badge(r, tc(0.0, 20.0, 150.0, 52.0), "Pass level badge", level);
                g.art(r, tl(22.0, 100.0, 260.0, 175.0), "Pass reward artwork");
                g.heading(r, bottom(25.0, 46.0, 10.0, 10.0), "Reward state", if i < 3 { "UNLOCKED" } else { "LOCKED" }, 27.0);
            }
            g.button(c, br(45.0, 38.0, 360.0, 76.0), "Continue button", "Continue");
        }
    }
}

fn ranks(g: &mut G, v: u8) {
    let p = base(g, "LEADERBOARDS");
    match v {
        0 => {
            let c = g.panel(p, cc(0.0, 20.0, 1530.0, 850.0), "Global leaderboard");
            for (i, name) in ["Global", "Friends", "Weekly"].iter().enumerate() { g.button2(c, tl(30.0 + i as f32 * 370.0, 25.0, 345.0, 66.0), "Ranking tab", name); }
            g.heading_l(c, tl(45.0, 125.0, 1400.0, 45.0), "Leaderboard columns", "RANK                PLAYER                      SCORE", 28.0);
            for i in 0..6 {
                let r = if i == 3 { g.card_on(c, tl(38.0, 190.0 + i as f32 * 95.0, 1450.0, 78.0), "Your ranking row") } else { g.card(c, tl(38.0, 190.0 + i as f32 * 95.0, 1450.0, 78.0), "Ranking row") };
                g.label(r, tl(25.0, 15.0, 1390.0, 48.0), "Ranking entry", &format!("#{:<5}             {:<16}                 {}", i + 1, ["Mira", "Arin", "Taro", "You", "Lena", "Bram"][i], 9850 - i * 720), 28.0);
            }
        }
        1 => {
            let podium = g.panel(p, top(165.0, 460.0, 155.0, 155.0), "Top three podium");
            for (i, name) in ["Arin", "Mira", "Taro"].iter().enumerate() {
                let c = g.card(podium, tl(55.0 + i as f32 * 520.0, if i == 1 { 25.0 } else { 95.0 }, 480.0, if i == 1 { 400.0 } else { 330.0 }), "Podium card");
                g.art(c, tl(20.0, 20.0, 180.0, 220.0), "Player portrait");
                g.heading(c, tl(220.0, 95.0, 235.0, 60.0), "Player name", name, 34.0);
                g.badge(c, bl(20.0, 25.0, 150.0, 52.0), "Podium rank", &format!("#{}", [2, 1, 3][i]));
            }
            let list = g.panel(p, bottom(65.0, 330.0, 155.0, 155.0), "Other rankings");
            for i in 0..3 { g.label(list, tl(45.0, 40.0 + i as f32 * 85.0, 1350.0, 55.0), "Other player", &format!("#{}    {}                         {} pts", i + 4, ["You", "Lena", "Bram"][i], 7150 - i * 500), 28.0); }
        }
        _ => {
            let ladder = g.panel(p, tl(120.0, 160.0, 980.0, 825.0), "League ladder");
            for (i, name) in ["DIAMOND", "GOLD", "SILVER", "BRONZE"].iter().enumerate() {
                let c = if i == 2 { g.card_on(ladder, tl(42.0, 40.0 + i as f32 * 185.0, 895.0, 150.0), "Current league") } else { g.card(ladder, tl(42.0, 40.0 + i as f32 * 185.0, 895.0, 150.0), "League tier") };
                g.icon(c, tl(25.0, 30.0, 90.0, 90.0), "League emblem", &format!("{}", 4 - i));
                g.heading_l(c, tl(150.0, 35.0, 600.0, 60.0), "League name", name, 36.0);
                g.note(c, tl(150.0, 93.0, 600.0, 35.0), "League status", if i == 2 { "Your current league" } else { "Rank tier" }, 24.0);
            }
            let d = g.panel(p, tr(120.0, 160.0, 630.0, 825.0), "Promotion progress");
            g.heading(d, top(45.0, 80.0, 25.0, 25.0), "Current rank", "SILVER II", 50.0);
            g.art(d, tc(0.0, 160.0, 350.0, 320.0), "Rank emblem artwork");
            g.label(d, tl(45.0, 550.0, 540.0, 55.0), "Promotion points", "740 / 1,000 points", 29.0);
            g.bar(d, tl(45.0, 635.0, 540.0, 42.0), "Promotion bar", 0.74, g.t.accent);
        }
    }
}

fn post(g: &mut G, v: u8) {
    let p = base(g, "AFTER THE MATCH");
    match v {
        0 => {
            let menu = g.panel(p, tl(105.0, 180.0, 790.0, 760.0), "Post-game choices");
            g.heading(menu, top(50.0, 70.0, 25.0, 25.0), "Menu title", "WHAT NEXT?", 48.0);
            for (i, name) in ["Play again", "Return to lobby", "Change loadout", "Main menu"].iter().enumerate() { g.menu_item(menu, tl(60.0, 175.0 + i as f32 * 125.0, 670.0, 84.0), "Post-game option", name, i == 0); }
            let d = g.panel(p, tr(105.0, 180.0, 790.0, 760.0), "Match summary");
            g.art(d, tl(25.0, 25.0, 740.0, 370.0), "Match artwork");
            g.heading(d, tl(30.0, 430.0, 730.0, 60.0), "Match result", "VICTORY", 48.0);
            g.note(d, tl(40.0, 520.0, 700.0, 55.0), "Match score", "Final score  50 - 42", 30.0);
            g.badge(d, bc(0.0, 55.0, 320.0, 65.0), "Experience earned", "+ 1,250 XP");
        }
        1 => {
            let c = g.panel(p, cc(0.0, 20.0, 1680.0, 830.0), "Next map vote");
            g.heading(c, top(25.0, 70.0, 25.0, 25.0), "Vote title", "VOTE FOR THE NEXT MAP", 44.0);
            for (i, (name, votes)) in [("Glass Coast", "4 VOTES"), ("Old Quarry", "2 VOTES"), ("Night Market", "1 VOTE")].iter().enumerate() {
                let r = g.card(c, tl(35.0 + i as f32 * 545.0, 125.0, 510.0, 465.0), "Map vote card");
                g.art(r, tl(20.0, 20.0, 470.0, 290.0), "Map artwork");
                g.heading(r, tl(20.0, 335.0, 470.0, 55.0), "Map name", name, 32.0);
                g.badge(r, bc(0.0, 20.0, 230.0, 46.0), "Vote count", votes);
            }
            let player = g.card(c, bl(35.0, 30.0, 830.0, 155.0), "Player summary card");
            g.icon(player, tl(25.0, 28.0, 90.0, 90.0), "Player avatar", "P");
            g.heading_l(player, tl(145.0, 28.0, 630.0, 55.0), "Player name", "YOUR MATCH", 34.0);
            g.note(player, tl(145.0, 85.0, 630.0, 40.0), "Player match stat", "18 kills  /  6 assists", 26.0);
            g.button(c, br(35.0, 62.0, 380.0, 76.0), "Confirm vote button", "Confirm vote");
        }
        _ => {
            let replay = g.panel(p, top(160.0, 470.0, 180.0, 180.0), "Replay preview");
            g.art(replay, fill(18.0, 18.0, 18.0, 18.0), "Replay thumbnail");
            let caption = g.panel(replay, bc(0.0, 28.0, 620.0, 78.0), "Replay caption plate");
            g.heading(caption, full(), "Replay caption", "MATCH HIGHLIGHTS", 32.0);
            for (i, (name, detail)) in [("SHARE", "Send your result"), ("REPLAY", "Watch highlights"), ("CONTINUE", "Return to lobby")].iter().enumerate() {
                let c = g.panel(p, tl(180.0 + i as f32 * 535.0, 690.0, 500.0, 285.0), "Post-game action card");
                g.icon(c, tc(0.0, 25.0, 80.0, 80.0), "Action icon", &format!("{}", i + 1));
                g.heading(c, tl(20.0, 120.0, 460.0, 65.0), "Action title", name, 38.0);
                g.note(c, tl(25.0, 205.0, 450.0, 42.0), "Action detail", detail, 25.0);
            }
        }
    }
}

/// Rejilla de retratos y atributos del héroe elegido.
fn character_a(g: &mut G) { character(g, 0); }
/// Carrusel de cinco héroes con el central destacado.
fn character_b(g: &mut G) { character(g, 1); }
/// Dos luchadores enfrentados y rejilla de selección central.
fn character_c(g: &mut G) { character(g, 2); }
/// Mapa con nodos y ficha del nivel elegido.
fn stage_a(g: &mut G) { stage(g, 0); }
/// Pestañas de capítulos sobre una rejilla de niveles.
fn stage_b(g: &mut G) { stage(g, 1); }
/// Lista de etapas junto a una vista previa grande.
fn stage_c(g: &mut G) { stage(g, 2); }
/// Formulario de reglas en dos columnas.
fn setup_a(g: &mut G) { setup(g, 0); }
/// Asistente por pasos con resumen lateral.
fn setup_b(g: &mut G) { setup(g, 1); }
/// Cuatro huecos de jugadores y opciones inferiores.
fn setup_c(g: &mut G) { setup(g, 2); }
/// Dos equipos con insignias de preparación y chat.
fn lobby_a(g: &mut G) { lobby(g, 0); }
/// Cuatro personajes sobre plataformas y panel de modo.
fn lobby_b(g: &mut G) { lobby(g, 1); }
/// Mapa de briefing, objetivos y equipo.
fn lobby_c(g: &mut G) { lobby(g, 2); }
/// Rótulo central de mundo, nombre y objetivo.
fn intro_a(g: &mut G) { intro(g, 0); }
/// Franja horizontal con escenario contra jefe.
fn intro_b(g: &mut G) { intro(g, 1); }
/// Rótulo cinematográfico inferior con lugar y hora.
fn intro_c(g: &mut G) { intro(g, 2); }
/// Menú vertical centrado sobre una escena atenuada.
fn pause_a(g: &mut G) { pause(g, 0); }
/// Barra lateral de pausa junto al mapa.
fn pause_b(g: &mut G) { pause(g, 1); }
/// Pestañas superiores, resumen y acciones inferiores.
fn pause_c(g: &mut G) { pause(g, 2); }
/// Miembros activos sobre disponibles y comparación.
fn party_a(g: &mut G) { party(g, 0); }
/// Lista de clases y detalle de habilidades.
fn party_b(g: &mut G) { party(g, 1); }
/// Formación de tres por tres con banca inferior.
fn party_c(g: &mut G) { party(g, 2); }
/// Conversación con retrato, recompensas y decisión.
fn mission_a(g: &mut G) { mission(g, 0); }
/// Tablón de contratos y detalle seleccionado.
fn mission_b(g: &mut G) { mission(g, 1); }
/// Modal con objetivos y nivel recomendado.
fn mission_c(g: &mut G) { mission(g, 2); }
/// Tres cartas de botín para elegir una.
fn loot_a(g: &mut G) { loot(g, 0); }
/// Cofre ilustrado con lista seleccionable.
fn loot_b(g: &mut G) { loot(g, 1); }
/// Inventario y contenedor con detalle de objeto.
fn loot_c(g: &mut G) { loot(g, 2); }
/// Gran rótulo de derrota con dos acciones.
fn failure_a(g: &mut G) { failure(g, 0); }
/// Informe de fallo con causa, cifras y consejo.
fn failure_b(g: &mut G) { failure(g, 1); }
/// Cuenta atrás circular para continuar.
fn failure_c(g: &mut G) { failure(g, 2); }
/// Estrellas y puntuación con repetir o avanzar.
fn complete_a(g: &mut G) { complete(g, 0); }
/// Franja de victoria con tiempo y rango.
fn complete_b(g: &mut G) { complete(g, 1); }
/// Objetivos completados y recompensa acumulada.
fn complete_c(g: &mut G) { complete(g, 2); }
/// Tabla de jugadores con fila propia destacada.
fn results_a(g: &mut G) { results(g, 0); }
/// Podio de tres con una lista adicional.
fn results_b(g: &mut G) { results(g, 1); }
/// Barras de desempeño y récords personales.
fn results_c(g: &mut G) { results(g, 2); }
/// Subida de nivel y objetos obtenidos.
fn rewards_a(g: &mut G) { rewards(g, 0); }
/// Libro de monedas, gemas y experiencia.
fn rewards_b(g: &mut G) { rewards(g, 1); }
/// Pista horizontal de premios de temporada.
fn rewards_c(g: &mut G) { rewards(g, 2); }
/// Tabla mundial con pestañas y posición propia.
fn ranks_a(g: &mut G) { ranks(g, 0); }
/// Podio de líderes con clasificación secundaria.
fn ranks_b(g: &mut G) { ranks(g, 1); }
/// Escalera de ligas y barra de ascenso.
fn ranks_c(g: &mut G) { ranks(g, 2); }
/// Opciones posteriores junto al resumen del match.
fn post_a(g: &mut G) { post(g, 0); }
/// Votación de mapas y rendimiento propio.
fn post_b(g: &mut G) { post(g, 1); }
/// Miniatura de repetición sobre tres acciones.
fn post_c(g: &mut G) { post(g, 2); }
