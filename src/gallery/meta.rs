//! Galeria: pantallas de meta-juego (servicio y social, creacion de contenido y monetizacion).

use super::*;

pub fn entries() -> Vec<Entry> {
    use Style::*;
    vec![
        // Servicio y social
        Entry { ty: "News & notifications", style: SciFi, build: news_featured_carousel },
        Entry { ty: "News & notifications", style: Minimal, build: news_inbox },
        Entry { ty: "News & notifications", style: Sports, build: news_patch_notes },
        Entry { ty: "Profile customization", style: Cartoon, build: profile_card_editor },
        Entry { ty: "Profile customization", style: Neon, build: profile_avatar_grid },
        Entry { ty: "Profile customization", style: Elegant, build: profile_banners_frames },
        Entry { ty: "Friends & players", style: Minimal, build: friends_list },
        Entry { ty: "Friends & players", style: Fantasy, build: friends_requests },
        Entry { ty: "Friends & players", style: Pastel, build: friends_player_card },
        Entry { ty: "Chat & communication", style: Military, build: chat_channels },
        Entry { ty: "Chat & communication", style: Neon, build: chat_party_overlay },
        Entry { ty: "Chat & communication", style: Retro, build: chat_mail },
        Entry { ty: "Online play", style: Military, build: online_server_browser },
        Entry { ty: "Online play", style: SciFi, build: online_matchmaking },
        Entry { ty: "Online play", style: Sports, build: online_party_invites },
        // Creacion de contenido
        Entry { ty: "Photo mode", style: Nature, build: photo_side_sliders },
        Entry { ty: "Photo mode", style: Pastel, build: photo_frames_stickers },
        Entry { ty: "Photo mode", style: Minimal, build: photo_camera_controls },
        Entry { ty: "Level editor", style: Retro, build: editor_classic_grid },
        Entry { ty: "Level editor", style: Cartoon, build: editor_bottom_library },
        Entry { ty: "Level editor", style: SciFi, build: editor_layers_inspector },
        Entry { ty: "Decals & liveries", style: Sports, build: liveries_layers_color },
        Entry { ty: "Decals & liveries", style: Neon, build: liveries_decal_grid },
        Entry { ty: "Decals & liveries", style: Military, build: liveries_pattern_editor },
        Entry { ty: "Content discovery", style: Nature, build: discovery_grid_filters },
        Entry { ty: "Content discovery", style: Minimal, build: discovery_featured_lanes },
        Entry { ty: "Content discovery", style: Cartoon, build: discovery_item_detail },
        // Monetizacion
        Entry { ty: "Premium store", style: Fantasy, build: store_hero_grid },
        Entry { ty: "Premium store", style: Elegant, build: store_tabs_list },
        Entry { ty: "Premium store", style: Neon, build: store_product_detail },
        Entry { ty: "Offers & bundles", style: Cartoon, build: offers_discount_cards },
        Entry { ty: "Offers & bundles", style: Horror, build: offers_limited_timer },
        Entry { ty: "Offers & bundles", style: Sports, build: offers_pack_compare },
        Entry { ty: "Currency store", style: Fantasy, build: currency_bonus_tiers },
        Entry { ty: "Currency store", style: SciFi, build: currency_vertical_list },
        Entry { ty: "Currency store", style: Pastel, build: currency_best_offer },
        Entry { ty: "Daily rewards", style: Nature, build: daily_7_days },
        Entry { ty: "Daily rewards", style: Pastel, build: daily_28_calendar },
        Entry { ty: "Daily rewards", style: Retro, build: daily_wheel_chests },
        Entry { ty: "Season pass & events", style: Military, build: season_pass_track },
        Entry { ty: "Season pass & events", style: Horror, build: season_event_missions },
        Entry { ty: "Season pass & events", style: Elegant, build: season_chapter_progress },
        Entry { ty: "Gacha & crates", style: Fantasy, build: gacha_banner_odds },
        Entry { ty: "Gacha & crates", style: Horror, build: gacha_crate_open },
        Entry { ty: "Gacha & crates", style: Retro, build: gacha_card_collection },
        Entry { ty: "Upcoming unlocks", style: Horror, build: unlocks_progress_bar },
        Entry { ty: "Upcoming unlocks", style: Nature, build: unlocks_timeline },
        Entry { ty: "Upcoming unlocks", style: Elegant, build: unlocks_next_cards },
    ]
}

// ===============================================================================================================
// Servicio y social
// ===============================================================================================================

/// Carrusel destacado a la izquierda con portada y controles, lista de noticias a la derecha.
fn news_featured_carousel(g: &mut G) {
    let p = g.page();
    g.background();
    // Barra superior de navegacion
    let top_bar = g.panel(p, top(20.0, 92.0, 36.0, 36.0), "Header bar");
    g.heading_l(top_bar, cl(28.0, 0.0, 400.0, 48.0), "Screen title", "NEWS & UPDATES", 40.0);
    let tabs = g.hbox(top_bar, cc(60.0, 0.0, 620.0, 68.0), "Category tabs", 12.0);
    for (i, t) in ["All", "Updates", "Events"].iter().enumerate() {
        if i == 0 {
            g.button(tabs, sz(190.0, 68.0), "Category button", t);
        } else {
            g.button2(tabs, sz(190.0, 68.0), "Category button", t);
        }
    }
    g.button2(top_bar, cr(20.0, 0.0, 170.0, 68.0), "Close button", "Close");

    // Panel destacado izquierdo con carrusel
    let hero = g.panel(p, tl(36.0, 132.0, 1140.0, 824.0), "Featured story panel");
    g.art(hero, top(24.0, 410.0, 24.0, 24.0), "Featured artwork");
    g.badge(hero, tl(44.0, 44.0, 168.0, 44.0), "Featured badge", "FEATURED");
    g.heading_l(hero, tl(28.0, 456.0, 1080.0, 56.0), "Featured headline", "ORBITAL RIFT EXPANSION NOW LIVE", 44.0);
    g.note(hero, tl(28.0, 520.0, 1080.0, 84.0), "Featured summary", "Explore three low-gravity sectors, unlock the Striker frame, and join the weekend relay.", 26.0);
    g.button(hero, bl(28.0, 24.0, 280.0, 72.0), "Read article button", "Read full article");
    let nav = g.hbox(hero, br(28.0, 24.0, 340.0, 72.0), "Carousel controls", 14.0);
    g.button2(nav, sz(84.0, 68.0), "Previous slide button", "<");
    g.heading(nav, sz(120.0, 48.0), "Slide counter", "1 / 4", 26.0);
    g.button2(nav, sz(84.0, 68.0), "Next slide button", ">");

    // Columna derecha con historias recientes
    let list_p = g.panel(p, tr(36.0, 132.0, 684.0, 824.0), "Latest stories panel");
    g.heading_l(list_p, tl(24.0, 22.0, 620.0, 44.0), "Latest heading", "LATEST STORIES", 32.0);
    let list = g.vbox(list_p, fill(24.0, 80.0, 24.0, 24.0), "Story list", 16.0);
    let stories = [
        ("TODAY", "Weekend Double XP Event", "Earn bonus progress in all co-op sectors."),
        ("2D AGO", "Creator Showcase Winners", "Five community maps added to rotation."),
        ("5D AGO", "Server Maintenance Complete", "Improved match stability across regions."),
        ("1W AGO", "Season 4 Roadmap Preview", "New gear tiers and clan trials arriving."),
    ];
    for (i, (date, title, desc)) in stories.iter().enumerate() {
        let c = if i == 0 {
            g.card_on(list, sz(636.0, 164.0), "Story card")
        } else {
            g.card(list, sz(636.0, 164.0), "Story card")
        };
        g.badge(c, tl(18.0, 16.0, 136.0, 40.0), "Story date", date);
        g.heading_l(c, tl(18.0, 66.0, 600.0, 38.0), "Story title", title, 28.0);
        g.note(c, tl(18.0, 108.0, 600.0, 38.0), "Story summary", desc, 22.0);
    }

    // Pie de estado
    let foot = g.panel(p, bottom(20.0, 76.0, 36.0, 36.0), "Status bar");
    g.label(foot, cl(28.0, 0.0, 800.0, 38.0), "Server status", "All regional gateways operational - Ping 24 ms", 24.0);
    let keys = g.hbox(foot, cr(24.0, 0.0, 380.0, 48.0), "Key hints", 12.0);
    g.key(keys, sz(48.0, 44.0), "Select key", "A");
    g.note(keys, sz(110.0, 40.0), "Select hint", "Open", 22.0);
    g.key(keys, sz(48.0, 44.0), "Back key", "B");
    g.note(keys, sz(100.0, 40.0), "Back hint", "Back", 22.0);
}

/// Bandeja de notificaciones centrada con filtros laterales y tarjetas de avisos leidos y sin leer.
fn news_inbox(g: &mut G) {
    let p = g.page();
    g.background();
    let frame = g.panel(p, cc(0.0, 0.0, 1560.0, 940.0), "Inbox window");
    // Cabecera de bandeja
    g.heading_l(frame, tl(32.0, 28.0, 420.0, 52.0), "Inbox title", "NOTIFICATIONS", 42.0);
    g.badge(frame, tl(420.0, 32.0, 140.0, 44.0), "Unread badge", "3 NEW");
    let top_actions = g.hbox(frame, tr(32.0, 22.0, 460.0, 72.0), "Inbox actions", 14.0);
    g.button2(top_actions, sz(260.0, 68.0), "Mark read button", "Mark all read");
    g.button2(top_actions, sz(180.0, 68.0), "Clear button", "Clear");
    g.rect(frame, top(108.0, 2.0, 28.0, 28.0), "Header divider", g.t.line, 0.0);

    // Filtros laterales
    let filters = g.vbox(frame, left(28.0, 300.0, 132.0, 28.0), "Filter menu", 14.0);
    for (i, label) in ["All (12)", "Rewards (3)", "Social (5)", "System (4)"].iter().enumerate() {
        g.menu_item(filters, sz(300.0, 72.0), "Filter button", label, i == 0);
    }

    // Lista de notificaciones
    let feed = g.vbox(frame, fill(356.0, 132.0, 28.0, 28.0), "Notification feed", 16.0);
    let items = [
        (true, "!", "Season Reward Ready", "You unlocked the Summit Emblem from Tier 14.", "5m ago", "Claim"),
        (true, "+", "Friend Request", "RiverKite wants to add you to their friends list.", "22m ago", "Review"),
        (true, ">", "Clan Raid Starting", "Iron Pine raid lobby opens in 15 minutes.", "1h ago", "Join"),
        (false, "i", "Patch 2.4 Installed", "Your game client is up to date.", "Yesterday", "Notes"),
        (false, "*", "Weekly Challenge Done", "Collected 150 bonus credits for 10 assists.", "2d ago", "Details"),
    ];
    for (unread, glyph, title, body, ago, cta) in items {
        let c = if unread {
            g.card_on(feed, sz(1176.0, 138.0), "Unread notification")
        } else {
            g.card(feed, sz(1176.0, 138.0), "Read notification")
        };
        g.icon(c, cl(22.0, 0.0, 60.0, 60.0), "Notification icon", glyph);
        g.heading_l(c, tl(104.0, 24.0, 560.0, 38.0), "Notification title", title, 28.0);
        g.note(c, tl(104.0, 68.0, 740.0, 40.0), "Notification body", body, 23.0);
        g.note(c, tr(230.0, 24.0, 150.0, 34.0), "Timestamp", ago, 22.0);
        if unread {
            g.button(c, cr(20.0, 0.0, 184.0, 68.0), "Action button", cta);
        } else {
            g.button2(c, cr(20.0, 0.0, 184.0, 68.0), "Action button", cta);
        }
    }
}

/// Notas del parche en dos columnas con novedades a la izquierda y ajustes de equilibrio a la derecha.
fn news_patch_notes(g: &mut G) {
    let p = g.page();
    g.background();
    // Cabecera de parche
    let hdr = g.panel(p, top(24.0, 120.0, 40.0, 40.0), "Patch header");
    g.badge(hdr, tl(28.0, 18.0, 170.0, 40.0), "Version badge", "PATCH 2.4");
    g.heading_l(hdr, tl(28.0, 62.0, 760.0, 46.0), "Patch title", "APEX CIRCUIT - BALANCE & CONTENT", 36.0);
    g.dropdown(hdr, cr(224.0, 0.0, 260.0, 68.0), "Version picker", &["v2.4 (Current)", "v2.3", "v2.2"]);
    g.button2(hdr, cr(24.0, 0.0, 176.0, 68.0), "Back button", "Back");

    // Columna izquierda: novedades
    let left_col = g.panel(p, tl(40.0, 168.0, 900.0, 888.0), "Highlights column");
    g.heading_l(left_col, tl(28.0, 24.0, 840.0, 44.0), "Highlights heading", "NEW CONTENT & ARENAS", 32.0);
    g.art(left_col, top(80.0, 220.0, 28.0, 28.0), "Update banner");
    let feats = g.vbox(left_col, fill(28.0, 320.0, 28.0, 24.0), "Feature list", 16.0);
    let fdata = [
        ("ARENA", "Skyline Velodrome", "High-speed banked circuit with three boost lanes and weather shifts."),
        ("MODE", "Relay Sprint 3v3", "Pass the baton across four checkpoints before the shot clock expires."),
        ("GEAR", "Carbon Aero Kit", "New lightweight chassis upgrades earned through ranked divisions."),
    ];
    for (tag, title, desc) in fdata {
        let c = g.card(feats, sz(844.0, 164.0), "Feature card");
        g.badge(c, tl(20.0, 18.0, 130.0, 40.0), "Feature tag", tag);
        g.heading_l(c, tl(168.0, 18.0, 640.0, 40.0), "Feature title", title, 28.0);
        g.note(c, tl(20.0, 76.0, 800.0, 70.0), "Feature summary", desc, 24.0);
    }

    // Columna derecha: equilibrio y correcciones
    let right_col = g.panel(p, tr(40.0, 168.0, 900.0, 888.0), "Changes column");
    g.heading_l(right_col, tl(28.0, 24.0, 840.0, 44.0), "Changes heading", "BALANCE & FIXES", 32.0);
    let bal = g.card(right_col, top(82.0, 360.0, 28.0, 28.0), "Balance section");
    g.heading_l(bal, tl(22.0, 18.0, 780.0, 38.0), "Balance heading", "GAMEPLAY TUNING", 28.0);
    let blist = g.vbox(bal, fill(22.0, 68.0, 22.0, 20.0), "Balance notes", 12.0);
    for line in [
        "+ Turbo recharge rate raised from 8% to 10% per drift.",
        "- Corner assist drag reduced by 5% on wet surfaces.",
        "+ Overtime kickoff timer extended to 15 seconds.",
        "+ Matchmaking tightened for Division 1 squads.",
    ] {
        g.label(blist, sz(800.0, 56.0), "Balance line", line, 24.0);
    }

    let fixes = g.card(right_col, top(464.0, 300.0, 28.0, 28.0), "Bugfix section");
    g.heading_l(fixes, tl(22.0, 18.0, 780.0, 38.0), "Bugfix heading", "STABILITY & BUG FIXES", 28.0);
    let flist = g.vbox(fixes, fill(22.0, 68.0, 22.0, 20.0), "Fix notes", 12.0);
    for line in [
        "- Fixed replay camera clipping near stadium tunnels.",
        "- Fixed squad invites failing after custom lobbies.",
        "- Improved frame pacing on ultrawide displays.",
    ] {
        g.label(flist, sz(800.0, 58.0), "Fix line", line, 24.0);
    }

    g.button(right_col, br(28.0, 24.0, 320.0, 72.0), "Full notes button", "View full changelog");
}

/// Tarjeta de perfil con vista previa de pancarta, avatar y titulo junto a los selectores.
fn profile_card_editor(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 92.0, 36.0, 36.0), "Profile header");
    g.heading_l(hdr, cl(28.0, 0.0, 480.0, 48.0), "Header title", "PROFILE STUDIO", 40.0);
    g.badge(hdr, cc(-60.0, 0.0, 180.0, 44.0), "Level badge", "LEVEL 48");
    g.button2(hdr, cr(236.0, 0.0, 176.0, 68.0), "Back button", "Back");
    g.button(hdr, cr(24.0, 0.0, 192.0, 68.0), "Save button", "Save");

    // Vista previa izquierda
    let prev = g.panel(p, tl(36.0, 132.0, 720.0, 916.0), "Preview panel");
    g.heading_l(prev, tl(28.0, 24.0, 640.0, 42.0), "Preview label", "LIVE PROFILE CARD", 30.0);
    let card = g.card_on(prev, top(80.0, 560.0, 28.0, 28.0), "Player card preview");
    g.art(card, top(18.0, 210.0, 18.0, 18.0), "Banner preview");
    let av = g.panel(card, tl(36.0, 160.0, 148.0, 148.0), "Avatar frame");
    g.icon(av, cc(0.0, 0.0, 108.0, 108.0), "Avatar emblem", "NS");
    g.heading_l(card, tl(204.0, 242.0, 420.0, 46.0), "Player name", "NovaStride", 36.0);
    g.badge(card, tl(204.0, 296.0, 240.0, 44.0), "Title badge", "SKY VANGUARD");
    g.note(card, tl(36.0, 360.0, 580.0, 40.0), "Bio preview", "Always ready for co-op island runs!", 24.0);
    let stats = g.hbox(card, bottom(22.0, 116.0, 24.0, 24.0), "Stats row", 16.0);
    for (k, v) in [("WINS", "184"), ("RANK", "GOLD II"), ("HOURS", "96h")] {
        let sc = g.panel(stats, sz(194.0, 112.0), "Stat box");
        g.note(sc, top(14.0, 30.0, 12.0, 12.0), "Stat label", k, 22.0);
        g.heading_l(sc, bottom(16.0, 44.0, 12.0, 12.0), "Stat value", v, 32.0);
    }
    g.toggle(prev, bl(28.0, 130.0, 460.0, 64.0), "Friend view toggle", "Show public stats", true);
    g.note(prev, bl(28.0, 32.0, 640.0, 72.0), "Preview hint", "Other players see this card in lobbies and leaderboards.", 23.0);

    // Panel derecho de selectores
    let ed = g.panel(p, tr(36.0, 132.0, 1100.0, 916.0), "Editor panel");
    let tabs = g.hbox(ed, tl(28.0, 24.0, 880.0, 68.0), "Customization tabs", 14.0);
    for (i, t) in ["Banner", "Avatar", "Title", "Emblem"].iter().enumerate() {
        if i == 0 {
            g.button(tabs, sz(200.0, 68.0), "Tab button", t);
        } else {
            g.button2(tabs, sz(200.0, 68.0), "Tab button", t);
        }
    }
    g.label(ed, tl(28.0, 116.0, 220.0, 38.0), "Title label", "Display title", 26.0);
    g.dropdown(ed, tl(28.0, 160.0, 490.0, 68.0), "Title selector", &["Sky Vanguard", "Cloud Hopper", "Starlight Ace"]);
    g.label(ed, tl(556.0, 116.0, 240.0, 38.0), "Motto label", "Profile motto", 26.0);
    g.input(ed, tl(556.0, 160.0, 516.0, 68.0), "Motto input", "Always ready for co-op island runs!");

    g.heading_l(ed, tl(28.0, 256.0, 400.0, 40.0), "Grid heading", "SELECT BANNER", 28.0);
    let grid = g.grid(ed, tl(28.0, 312.0, 1044.0, 470.0), "Banner grid", 332.0, 218.0, 24.0);
    let banners = ["Sunrise Cliffs", "Coral Reef", "Cloud Castle", "Lantern Cove", "Frost Peak", "Starlight Bay"];
    for (i, name) in banners.iter().enumerate() {
        let c = if i == 0 {
            g.card_on(grid, sz(332.0, 218.0), "Banner option")
        } else {
            g.card(grid, sz(332.0, 218.0), "Banner option")
        };
        g.art(c, top(14.0, 128.0, 14.0, 14.0), "Banner swatch");
        g.heading_l(c, bottom(16.0, 40.0, 16.0, 16.0), "Banner name", name, 24.0);
    }
    g.button2(ed, bl(28.0, 24.0, 220.0, 68.0), "Reset button", "Reset default");
    g.button(ed, br(28.0, 24.0, 280.0, 72.0), "Apply button", "Apply changes");
}

/// Rejilla amplia de avatares con filtros a la derecha y vista previa destacada a la izquierda.
fn profile_avatar_grid(g: &mut G) {
    let p = g.page();
    g.background();
    // Columna izquierda: avatar elegido
    let side = g.panel(p, tl(36.0, 36.0, 520.0, 1008.0), "Selected avatar panel");
    g.heading_l(side, tl(28.0, 28.0, 460.0, 46.0), "Preview heading", "AVATAR PREVIEW", 34.0);
    let stage = g.card_on(side, tc(0.0, 96.0, 440.0, 440.0), "Avatar stage");
    g.art(stage, fill(20.0, 20.0, 20.0, 20.0), "Avatar backdrop");
    g.icon(stage, cc(0.0, 0.0, 220.0, 220.0), "Avatar icon", "FX");
    g.badge(side, tc(0.0, 560.0, 160.0, 44.0), "Rarity badge", "EPIC");
    g.heading(side, tc(0.0, 616.0, 460.0, 48.0), "Avatar name", "CYBER FOX", 38.0);
    g.note(side, tc(0.0, 672.0, 440.0, 76.0), "Unlock source", "Unlocked by reaching Neon League Tier 20.", 24.0);
    g.button(side, bc(0.0, 114.0, 440.0, 72.0), "Equip button", "Equip avatar");
    g.button2(side, bc(0.0, 28.0, 440.0, 68.0), "Randomize button", "Randomize");

    // Panel derecho: rejilla de avatares
    let main = g.panel(p, tr(36.0, 36.0, 1300.0, 1008.0), "Avatar catalog");
    g.heading_l(main, tl(28.0, 28.0, 480.0, 48.0), "Catalog heading", "CHOOSE AVATAR", 38.0);
    g.badge(main, tr(28.0, 30.0, 240.0, 44.0), "Collection count", "10 / 12 OWNED");
    let filt = g.hbox(main, tl(28.0, 96.0, 780.0, 68.0), "Filter row", 14.0);
    for (i, label) in ["All", "Owned", "Event", "Ranked"].iter().enumerate() {
        if i == 0 {
            g.button(filt, sz(180.0, 68.0), "Filter button", label);
        } else {
            g.button2(filt, sz(180.0, 68.0), "Filter button", label);
        }
    }
    let grid = g.grid(main, tl(28.0, 190.0, 1244.0, 790.0), "Avatar grid", 296.0, 246.0, 20.0);
    let avatars = [
        ("FX", "Cyber Fox", true),
        ("VT", "Volt Cat", true),
        ("NV", "Nova Bot", true),
        ("DR", "Drift Owl", true),
        ("PX", "Pixel Lynx", true),
        ("ZR", "Zero Shark", true),
        ("AR", "Arc Wolf", true),
        ("KR", "Krypt Moth", true),
        ("SL", "Solar Ape", true),
        ("GL", "Glitch Crow", true),
        ("?", "Shadow Elk", false),
        ("?", "Prism Dragon", false),
    ];
    for (i, (glyph, name, owned)) in avatars.iter().enumerate() {
        let c = if i == 0 {
            g.card_on(grid, sz(296.0, 246.0), "Avatar card")
        } else {
            g.card(grid, sz(296.0, 246.0), "Avatar card")
        };
        g.icon(c, tc(0.0, 24.0, 112.0, 112.0), "Avatar glyph", glyph);
        g.heading(c, bc(0.0, 56.0, 260.0, 36.0), "Avatar label", name, 24.0);
        g.note(c, bc(0.0, 16.0, 240.0, 30.0), "Ownership label", if *owned { "Owned" } else { "Locked" }, 22.0);
    }
}

/// Vista previa panoramica superior de placa de nombre con pancartas y marcos divididos abajo.
fn profile_banners_frames(g: &mut G) {
    let p = g.page();
    g.background();
    // Placa panoramica superior
    let stage = g.panel(p, top(28.0, 264.0, 48.0, 48.0), "Nameplate preview");
    g.art(stage, fill(24.0, 24.0, 24.0, 24.0), "Banner backdrop");
    let badge_box = g.panel(stage, cl(56.0, 0.0, 176.0, 176.0), "Frame preview");
    g.icon(badge_box, cc(0.0, 0.0, 118.0, 118.0), "Player emblem", "VK");
    let info = g.panel(stage, cl(264.0, 0.0, 640.0, 156.0), "Identity plate");
    g.heading_l(info, tl(24.0, 22.0, 580.0, 52.0), "Player name", "ValkyrieNine", 44.0);
    g.note(info, tl(24.0, 86.0, 580.0, 40.0), "Player rank", "Grandmaster Division - Season 4", 26.0);
    g.badge(stage, cr(56.0, 0.0, 240.0, 52.0), "Equipped badge", "GILDED CREST");

    // Mitad inferior izquierda: pancartas
    let left_p = g.panel(p, bl(48.0, 28.0, 896.0, 736.0), "Banners panel");
    g.heading_l(left_p, tl(28.0, 24.0, 500.0, 44.0), "Banners heading", "NAMEPLATE BANNERS", 32.0);
    let blist = g.vbox(left_p, fill(28.0, 84.0, 28.0, 28.0), "Banner list", 16.0);
    let banners = [
        ("Royal Velvet", "Equipped - Prestige Tier"),
        ("Obsidian Spire", "Unlocked - Raid Clear"),
        ("Golden Laurel", "Unlocked - Tournament"),
        ("Midnight Arch", "Unlocked - Default"),
    ];
    for (i, (name, sub)) in banners.iter().enumerate() {
        let c = if i == 0 {
            g.card_on(blist, sz(840.0, 142.0), "Banner item")
        } else {
            g.card(blist, sz(840.0, 142.0), "Banner item")
        };
        g.art(c, cl(18.0, 0.0, 240.0, 102.0), "Banner thumb");
        g.heading_l(c, tl(280.0, 28.0, 520.0, 40.0), "Banner title", name, 28.0);
        g.note(c, tl(280.0, 76.0, 520.0, 38.0), "Banner source", sub, 23.0);
    }

    // Mitad inferior derecha: marcos y botones de guardado
    let right_p = g.panel(p, br(48.0, 28.0, 896.0, 736.0), "Frames panel");
    g.heading_l(right_p, tl(28.0, 24.0, 500.0, 44.0), "Frames heading", "AVATAR FRAMES", 32.0);
    let fgrid = g.grid(right_p, tl(28.0, 84.0, 840.0, 520.0), "Frames grid", 266.0, 246.0, 20.0);
    let frames = [("GC", "Gilded"), ("IV", "Ivory"), ("OB", "Obsidian"), ("CR", "Crown"), ("EM", "Emerald"), ("ST", "Star")];
    for (i, (g_txt, name)) in frames.iter().enumerate() {
        let c = if i == 0 {
            g.card_on(fgrid, sz(266.0, 246.0), "Frame card")
        } else {
            g.card(fgrid, sz(266.0, 246.0), "Frame card")
        };
        g.icon(c, tc(0.0, 30.0, 112.0, 112.0), "Frame icon", g_txt);
        g.heading(c, bc(0.0, 26.0, 220.0, 40.0), "Frame name", name, 26.0);
    }
    g.button2(right_p, bl(28.0, 24.0, 220.0, 68.0), "Revert button", "Revert");
    g.button(right_p, br(28.0, 24.0, 280.0, 72.0), "Save profile button", "Save profile");
}

/// Lista principal de amigos con estado y acciones rapidas junto al panel de escuadra actual.
fn friends_list(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 96.0, 36.0, 36.0), "Friends header");
    g.heading_l(hdr, cl(28.0, 0.0, 280.0, 48.0), "Friends title", "FRIENDS", 40.0);
    g.badge(hdr, cl(280.0, 0.0, 180.0, 44.0), "Online count", "4 ONLINE");
    g.input(hdr, cr(260.0, 0.0, 440.0, 64.0), "Search friends", "Filter by player name...");
    g.button(hdr, cr(20.0, 0.0, 216.0, 68.0), "Add friend button", "+ Add friend");

    // Lista izquierda
    let left_p = g.panel(p, tl(36.0, 136.0, 1188.0, 908.0), "Friends list panel");
    let tabs = g.hbox(left_p, tl(24.0, 20.0, 820.0, 68.0), "Friend tabs", 12.0);
    for (i, t) in ["Online (4)", "Offline (9)", "Recent", "Blocked"].iter().enumerate() {
        if i == 0 {
            g.button(tabs, sz(194.0, 68.0), "Tab button", t);
        } else {
            g.button2(tabs, sz(194.0, 68.0), "Tab button", t);
        }
    }
    let rows = g.vbox(left_p, fill(24.0, 108.0, 24.0, 24.0), "Friend rows", 14.0);
    let people = [
        ("MK", "MikaStone", "In Lobby - Ready to play", true),
        ("AR", "ArdenVex", "In Match - Canyon Relay (04:12)", true),
        ("CL", "CloverFin", "Online - Main Menu", true),
        ("TN", "TannerRay", "Away - 12m", true),
        ("JS", "JunoSpark", "Offline - Last seen 2h ago", false),
        ("BR", "BramOak", "Offline - Last seen yesterday", false),
    ];
    for (i, (init, name, status, online)) in people.iter().enumerate() {
        let c = if i == 0 {
            g.card_on(rows, sz(1140.0, 116.0), "Friend row")
        } else {
            g.card(rows, sz(1140.0, 116.0), "Friend row")
        };
        g.icon(c, cl(20.0, 0.0, 64.0, 64.0), "Avatar icon", init);
        g.heading_l(c, tl(104.0, 20.0, 480.0, 38.0), "Friend name", name, 28.0);
        g.note(c, tl(104.0, 62.0, 540.0, 36.0), "Friend status", status, 23.0);
        if *online {
            g.button2(c, cr(206.0, 0.0, 152.0, 68.0), "Chat button", "Chat");
            g.button(c, cr(20.0, 0.0, 170.0, 68.0), "Invite button", "Invite");
        } else {
            g.button2(c, cr(20.0, 0.0, 170.0, 68.0), "Profile button", "Profile");
        }
    }

    // Panel derecho: escuadra actual
    let squad = g.panel(p, tr(36.0, 136.0, 636.0, 908.0), "Squad panel");
    g.heading_l(squad, tl(24.0, 24.0, 560.0, 44.0), "Squad heading", "CURRENT SQUAD (2/4)", 32.0);
    let slots = g.vbox(squad, tl(24.0, 88.0, 588.0, 510.0), "Squad slots", 16.0);
    for (i, (init, name, role)) in [
        ("YO", "You (Leader)", "Ready - Tier 42"),
        ("MK", "MikaStone", "Ready - Tier 39"),
        ("+", "Open Slot", "Invite a friend to join"),
        ("+", "Open Slot", "Invite a friend to join"),
    ]
    .iter()
    .enumerate()
    {
        let c = if i < 2 {
            g.card_on(slots, sz(588.0, 114.0), "Member slot")
        } else {
            g.card(slots, sz(588.0, 114.0), "Empty slot")
        };
        g.icon(c, cl(18.0, 0.0, 58.0, 58.0), "Slot icon", init);
        g.heading_l(c, tl(96.0, 20.0, 440.0, 36.0), "Member name", name, 26.0);
        g.note(c, tl(96.0, 60.0, 440.0, 34.0), "Member detail", role, 22.0);
    }
    g.label(squad, bl(24.0, 198.0, 580.0, 36.0), "Privacy label", "Squad privacy", 24.0);
    g.dropdown(squad, bl(24.0, 122.0, 588.0, 68.0), "Privacy dropdown", &["Friends only", "Open to clan", "Invite only"]);
    g.button(squad, bl(24.0, 24.0, 588.0, 74.0), "Ready button", "Enter Matchmaking");
}

/// Solicitudes pendientes de amistad a la izquierda y busqueda con sugerencias a la derecha.
fn friends_requests(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 96.0, 40.0, 40.0), "Social search bar");
    g.heading_l(hdr, cl(28.0, 0.0, 380.0, 48.0), "Header title", "ADD FRIENDS", 40.0);
    g.input(hdr, cc(40.0, 0.0, 620.0, 64.0), "Player tag input", "Enter player tag (e.g. AshWolf#402)");
    g.button(hdr, cr(216.0, 0.0, 190.0, 68.0), "Search button", "Search");
    g.button2(hdr, cr(24.0, 0.0, 172.0, 68.0), "Back button", "Back");

    // Izquierda: solicitudes recibidas y enviadas
    let left_p = g.panel(p, tl(40.0, 140.0, 900.0, 904.0), "Requests panel");
    g.heading_l(left_p, tl(28.0, 24.0, 500.0, 44.0), "Incoming heading", "INCOMING REQUESTS", 32.0);
    g.badge(left_p, tr(28.0, 24.0, 148.0, 44.0), "Pending badge", "3 PENDING");
    let reqs = g.vbox(left_p, tl(28.0, 88.0, 844.0, 540.0), "Incoming list", 16.0);
    let incoming = [
        ("EL", "ElaraDawn", "3 mutual friends - Sent 14m ago"),
        ("GR", "GarrickOak", "Played Dungeon 4 together - 2h ago"),
        ("SY", "SylviFrost", "Guild ally - Sent yesterday"),
    ];
    for (init, name, meta) in incoming {
        let c = g.card_on(reqs, sz(844.0, 166.0), "Request card");
        g.icon(c, cl(22.0, 0.0, 72.0, 72.0), "Requester icon", init);
        g.heading_l(c, tl(116.0, 28.0, 380.0, 40.0), "Requester name", name, 30.0);
        g.note(c, tl(116.0, 76.0, 420.0, 60.0), "Request detail", meta, 23.0);
        g.button(c, tr(20.0, 16.0, 176.0, 64.0), "Accept button", "Accept");
        g.button2(c, br(20.0, 16.0, 176.0, 64.0), "Decline button", "Decline");
    }
    let sent = g.card(left_p, bottom(24.0, 224.0, 28.0, 28.0), "Sent requests box");
    g.heading_l(sent, tl(22.0, 18.0, 500.0, 38.0), "Sent heading", "SENT REQUESTS (1)", 28.0);
    g.label(sent, tl(22.0, 82.0, 540.0, 40.0), "Sent player", "RowanAsh#819 - Waiting for reply", 25.0);
    g.note(sent, tl(22.0, 128.0, 540.0, 36.0), "Sent time", "Sent 3 days ago", 22.0);
    g.button2(sent, cr(20.0, 16.0, 184.0, 68.0), "Cancel button", "Cancel");

    // Derecha: jugadores sugeridos
    let right_p = g.panel(p, tr(40.0, 140.0, 900.0, 904.0), "Suggestions panel");
    g.heading_l(right_p, tl(28.0, 24.0, 640.0, 44.0), "Suggested heading", "RECENT TEAMMATES", 32.0);
    let sugg = g.vbox(right_p, fill(28.0, 88.0, 28.0, 24.0), "Suggestions list", 16.0);
    let people = [
        ("KN", "KaelenWisp", "Shared raid: Iron Keep - 18m ago"),
        ("MR", "MarenBrook", "Won arena match together - 1h ago"),
        ("TH", "ThornVale", "Same guild: Ember Watch"),
        ("LY", "LyraFawn", "Shared raid: Glass Caverns - 4h ago"),
    ];
    for (init, name, meta) in people {
        let c = g.card(sugg, sz(844.0, 182.0), "Suggested card");
        g.icon(c, cl(24.0, 0.0, 72.0, 72.0), "Player icon", init);
        g.heading_l(c, tl(120.0, 36.0, 460.0, 42.0), "Player name", name, 30.0);
        g.note(c, tl(120.0, 86.0, 480.0, 44.0), "Shared history", meta, 24.0);
        g.button2(c, cr(24.0, 0.0, 188.0, 68.0), "Add button", "+ Add");
    }
}

/// Tarjeta modal detallada de jugador con estadisticas, insignias y botones de interaccion.
fn friends_player_card(g: &mut G) {
    let p = g.page();
    g.background();
    // Fondo de lista desenfocada detras del modal
    let back = g.panel(p, fill(60.0, 60.0, 60.0, 60.0), "Background directory");
    g.heading_l(back, tl(36.0, 28.0, 400.0, 48.0), "Directory title", "PLAYER DIRECTORY", 36.0);
    g.dim(150);

    // Modal central de perfil
    let modal = g.panel(p, cc(0.0, 0.0, 1200.0, 880.0), "Player card modal");
    g.art(modal, top(24.0, 210.0, 24.0, 24.0), "Player banner");
    g.button2(modal, tr(40.0, 40.0, 144.0, 64.0), "Close button", "Close");

    let av_box = g.panel(modal, tl(56.0, 152.0, 168.0, 168.0), "Avatar plate");
    g.icon(av_box, cc(0.0, 0.0, 120.0, 120.0), "Player avatar", "SM");
    g.heading_l(modal, tl(252.0, 248.0, 460.0, 50.0), "Player name", "SunnyMochi", 42.0);
    g.badge(modal, tl(620.0, 252.0, 156.0, 44.0), "Status badge", "ONLINE");
    g.note(modal, tl(252.0, 302.0, 640.0, 38.0), "Guild line", "Clover Club - Playing Garden Rally", 25.0);

    // Cuatro tarjetas de estadisticas
    let stats = g.hbox(modal, top(376.0, 156.0, 40.0, 40.0), "Stats row", 20.0);
    for (label, val) in [("LEVEL", "64"), ("WIN RATE", "62%"), ("MATCHES", "418"), ("RANK", "GOLD II")] {
        let sc = g.card(stats, sz(265.0, 150.0), "Stat card");
        g.note(sc, top(22.0, 34.0, 16.0, 16.0), "Stat name", label, 23.0);
        g.heading_l(sc, bottom(24.0, 52.0, 16.0, 16.0), "Stat number", val, 40.0);
    }

    // Insignias destacadas
    let bwrap = g.card(modal, top(560.0, 168.0, 40.0, 40.0), "Showcase panel");
    g.heading_l(bwrap, tl(24.0, 18.0, 400.0, 36.0), "Showcase title", "FEATURED BADGES", 26.0);
    let badges = g.hbox(bwrap, bottom(18.0, 92.0, 24.0, 24.0), "Badges row", 16.0);
    for (g_txt, name) in [("S4", "Season Vet"), ("CO", "Team Helper"), ("10", "10x Streak"), ("AR", "Art Creator")] {
        let bc = g.panel(badges, sz(256.0, 88.0), "Badge item");
        g.icon(bc, cl(14.0, 0.0, 52.0, 52.0), "Badge icon", g_txt);
        g.label(bc, tl(80.0, 26.0, 166.0, 38.0), "Badge title", name, 24.0);
    }

    // Acciones inferiores
    let acts = g.hbox_c(modal, bottom(30.0, 76.0, 40.0, 40.0), "Action row", 18.0);
    g.button(acts, sz(280.0, 72.0), "Invite button", "Invite to Party");
    g.button2(acts, sz(260.0, 72.0), "Message button", "Send Message");
    g.button2(acts, sz(260.0, 72.0), "Inspect button", "Inspect Gear");
    g.button2(acts, sz(200.0, 72.0), "Remove button", "Remove");
}

/// Ventana completa de chat con columna de canales, registro central y lista de miembros conectados.
fn chat_channels(g: &mut G) {
    let p = g.page();
    g.background();
    let shell = g.panel(p, fill(32.0, 32.0, 32.0, 32.0), "Comms window");

    // Columna de canales
    let ch_col = g.card(shell, left(20.0, 320.0, 20.0, 20.0), "Channels sidebar");
    g.heading_l(ch_col, tl(20.0, 20.0, 280.0, 42.0), "Channels heading", "CHANNELS", 30.0);
    let ch_list = g.vbox(ch_col, tl(16.0, 76.0, 288.0, 480.0), "Channel buttons", 12.0);
    for (i, ch) in ["# Global", "# Squad", "# Regiment", "# Trade", "# Help"].iter().enumerate() {
        g.menu_item(ch_list, sz(288.0, 70.0), "Channel button", ch, i == 0);
    }
    let voice = g.panel(ch_col, bottom(16.0, 200.0, 16.0, 16.0), "Voice panel");
    g.badge(voice, tl(16.0, 16.0, 156.0, 40.0), "Voice badge", "VOICE ON");
    g.label(voice, tl(16.0, 68.0, 250.0, 36.0), "Voice room", "Squad Comms (4)", 24.0);
    g.toggle(voice, bl(16.0, 18.0, 250.0, 64.0), "Mute toggle", "Mute mic", false);

    // Centro: historial y entrada de mensajes
    let main = g.panel(shell, fill(360.0, 20.0, 368.0, 20.0), "Chat log panel");
    let ch_hdr = g.card(main, top(16.0, 76.0, 16.0, 16.0), "Channel header");
    g.heading_l(ch_hdr, cl(20.0, 0.0, 520.0, 42.0), "Active channel", "# GLOBAL - TACTICAL NET", 30.0);
    g.note(ch_hdr, cr(20.0, 0.0, 240.0, 34.0), "Online label", "42 active", 23.0);

    let msgs = g.vbox(main, fill(16.0, 108.0, 16.0, 108.0), "Message stream", 12.0);
    let log = [
        ("Vanguard01", "14:02", "Anyone running Sector 7 defense on Veteran?"),
        ("Rook_88", "14:03", "I can bring heavy support, ready in two minutes."),
        ("Kestrel", "14:03", "Count me in. Sending squad invite now."),
        ("System", "14:04", "Daily supply convoy arrives in 25 minutes."),
        ("IronMule", "14:05", "Trading two optic mods for alloy plates."),
        ("Vanguard01", "14:06", "Squad is full, moving to briefing room."),
    ];
    for (author, time, text) in log {
        let c = g.card(msgs, sz(1096.0, 112.0), "Message row");
        g.heading_l(c, tl(18.0, 14.0, 320.0, 36.0), "Sender name", author, 26.0);
        g.note(c, tr(18.0, 14.0, 120.0, 32.0), "Message time", time, 22.0);
        g.label(c, tl(18.0, 56.0, 1056.0, 38.0), "Message text", text, 25.0);
    }

    g.input(main, bl(16.0, 16.0, 888.0, 70.0), "Chat input", "Transmit message to #Global...");
    g.button(main, br(16.0, 16.0, 192.0, 70.0), "Send button", "Send");

    // Columna derecha: operadores conectados
    let mem = g.card(shell, right(20.0, 328.0, 20.0, 20.0), "Members sidebar");
    g.heading_l(mem, tl(20.0, 20.0, 288.0, 42.0), "Members heading", "ONLINE (12)", 30.0);
    let mlist = g.vbox(mem, fill(16.0, 76.0, 16.0, 20.0), "Member list", 12.0);
    for (tag, name) in [
        ("LD", "Vanguard01"),
        ("MD", "Rook_88"),
        ("SC", "Kestrel"),
        ("EN", "IronMule"),
        ("SN", "EchoNine"),
        ("HV", "Atlas_4"),
    ] {
        let mc = g.panel(mlist, sz(296.0, 88.0), "Member card");
        g.icon(mc, cl(14.0, 0.0, 52.0, 52.0), "Role icon", tag);
        g.label(mc, tl(80.0, 24.0, 200.0, 38.0), "Member name", name, 25.0);
    }
}

/// Superposicion de chat de grupo sobre la escena de juego junto a un modulo de voz compacto.
fn chat_party_overlay(g: &mut G) {
    let p = g.page();
    g.scene();
    // Panel flotante inferior izquierdo de chat de grupo
    let chat = g.panel(p, bl(36.0, 36.0, 820.0, 560.0), "Party chat overlay");
    let tabs = g.hbox(chat, tl(20.0, 18.0, 540.0, 68.0), "Chat tabs", 12.0);
    for (i, label) in ["Party", "Team", "Whisper"].iter().enumerate() {
        if i == 0 {
            g.button(tabs, sz(168.0, 68.0), "Tab button", label);
        } else {
            g.button2(tabs, sz(168.0, 68.0), "Tab button", label);
        }
    }
    g.key(chat, tr(20.0, 30.0, 88.0, 44.0), "Focus key", "TAB");

    let feed = g.vbox(chat, tl(20.0, 102.0, 780.0, 354.0), "Chat feed", 10.0);
    for (who, msg) in [
        ("Nova:", "Flanking left alley in 5 seconds."),
        ("Pulse:", "Shield drone deployed on point B."),
        ("Koi:", "Copy that, covering the upper bridge."),
        ("System:", "Pulse marked High Ground."),
    ] {
        let row = g.card(feed, sz(780.0, 78.0), "Chat line");
        g.heading_l(row, cl(16.0, 0.0, 160.0, 38.0), "Speaker", who, 25.0);
        g.label(row, cl(180.0, 0.0, 580.0, 38.0), "Line text", msg, 24.0);
    }

    g.input(chat, bl(20.0, 20.0, 596.0, 68.0), "Party input", "Press Enter to message party...");
    g.button(chat, br(20.0, 20.0, 168.0, 68.0), "Send button", "Send");

    // Modulo superior derecho de chat de voz
    let voice = g.panel(p, tr(36.0, 36.0, 460.0, 340.0), "Voice squad widget");
    g.heading_l(voice, tl(20.0, 18.0, 280.0, 40.0), "Voice title", "PARTY VOICE", 28.0);
    g.badge(voice, tr(20.0, 18.0, 110.0, 40.0), "Mic status", "LIVE");
    let vlist = g.vbox(voice, fill(20.0, 74.0, 20.0, 20.0), "Voice members", 12.0);
    for (name, lvl, live) in [("Nova (You)", 0.85, true), ("Pulse", 0.6, true), ("Koi", 0.15, false)] {
        let vc = if live {
            g.card_on(vlist, sz(420.0, 72.0), "Voice row")
        } else {
            g.card(vlist, sz(420.0, 72.0), "Voice row")
        };
        g.label(vc, cl(16.0, 0.0, 200.0, 36.0), "Speaker name", name, 24.0);
        g.bar(vc, cr(16.0, 0.0, 160.0, 22.0), "Voice meter", lvl, g.t.good);
    }
}

/// Correo del juego dividido en bandeja de entrada a la izquierda y lector con adjuntos a la derecha.
fn chat_mail(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(24.0, 96.0, 40.0, 40.0), "Mail header");
    g.heading_l(hdr, cl(28.0, 0.0, 320.0, 48.0), "Mail title", "MAILBOX", 40.0);
    g.badge(hdr, cl(300.0, 0.0, 184.0, 44.0), "Unread count", "3 UNREAD");
    g.button2(hdr, cr(264.0, 0.0, 210.0, 68.0), "Compose button", "Compose");
    g.button(hdr, cr(24.0, 0.0, 220.0, 68.0), "Claim all button", "Claim all");

    // Bandeja izquierda
    let inbox = g.panel(p, tl(40.0, 140.0, 620.0, 904.0), "Inbox column");
    let mlist = g.vbox(inbox, fill(20.0, 20.0, 20.0, 20.0), "Mail list", 14.0);
    let mails = [
        ("Guild Master", "Tournament Payout", "Today", true),
        ("Arena Desk", "Weekly Rank Bonus", "Yesterday", true),
        ("PixelTrader", "Auction Sold: Ruby", "2d ago", true),
        ("Quest Board", "Bounty Completed", "4d ago", false),
        ("Support Team", "Welcome Gift", "6d ago", false),
    ];
    for (i, (from, subj, date, gift)) in mails.iter().enumerate() {
        let c = if i == 0 {
            g.card_on(mlist, sz(580.0, 158.0), "Mail item")
        } else {
            g.card(mlist, sz(580.0, 158.0), "Mail item")
        };
        g.heading_l(c, tl(18.0, 18.0, 380.0, 38.0), "Mail subject", subj, 27.0);
        g.note(c, tr(18.0, 20.0, 140.0, 34.0), "Mail date", date, 22.0);
        g.label(c, tl(18.0, 62.0, 360.0, 36.0), "Mail sender", from, 24.0);
        if *gift {
            g.badge(c, bl(18.0, 14.0, 170.0, 40.0), "Attachment badge", "HAS GIFT");
        } else {
            g.note(c, bl(18.0, 18.0, 200.0, 34.0), "Read status", "Claimed", 22.0);
        }
    }

    // Panel lector derecho
    let view = g.panel(p, tr(40.0, 140.0, 1196.0, 904.0), "Message viewer");
    g.heading_l(view, tl(32.0, 28.0, 860.0, 50.0), "Open subject", "Tournament Payout", 40.0);
    g.note(view, tl(32.0, 86.0, 760.0, 36.0), "Meta line", "From: Guild Master   |   Expires in 14 days", 24.0);
    g.rect(view, top(136.0, 2.0, 32.0, 32.0), "Mail divider", g.t.line, 0.0);

    let body = g.card(view, top(160.0, 320.0, 32.0, 32.0), "Body box");
    let lines = g.vbox(body, fill(24.0, 24.0, 24.0, 24.0), "Body lines", 16.0);
    for line in [
        "Great work in the Starlight Cup qualifier!",
        "Your squad finished in the top 10% across all arenas.",
        "Collect your prize chest and gold coins below before expiry.",
    ] {
        g.label(lines, sz(1084.0, 68.0), "Body line", line, 27.0);
    }

    g.heading_l(view, tl(32.0, 508.0, 400.0, 40.0), "Attachments heading", "ATTACHMENTS", 28.0);
    let gifts = g.hbox(view, tl(32.0, 560.0, 960.0, 184.0), "Attachments row", 20.0);
    for (g_txt, name, qty) in [("G", "Gold Coins", "x500"), ("K", "Arcade Keys", "x2"), ("C", "Cup Crest", "x1")] {
        let gc = g.card(gifts, sz(296.0, 180.0), "Attachment card");
        g.icon(gc, cl(20.0, 0.0, 68.0, 68.0), "Gift icon", g_txt);
        g.heading_l(gc, tl(108.0, 46.0, 174.0, 38.0), "Gift name", name, 24.0);
        g.note(gc, tl(108.0, 90.0, 160.0, 34.0), "Gift quantity", qty, 24.0);
    }

    g.button(view, bl(32.0, 28.0, 300.0, 74.0), "Claim button", "Claim rewards");
    g.button2(view, br(236.0, 28.0, 184.0, 70.0), "Reply button", "Reply");
    g.button2(view, br(32.0, 28.0, 184.0, 70.0), "Delete button", "Delete");
}

/// Navegador de servidores en formato tabla con filtros de region, modo y latencia.
fn online_server_browser(g: &mut G) {
    let p = g.page();
    g.background();
    // Cabecera y busqueda
    let hdr = g.panel(p, top(20.0, 92.0, 36.0, 36.0), "Browser header");
    g.heading_l(hdr, cl(24.0, 0.0, 420.0, 48.0), "Browser title", "SERVER BROWSER", 38.0);
    g.input(hdr, cc(-40.0, 0.0, 440.0, 64.0), "Server search", "Search server name...");
    g.dropdown(hdr, cr(236.0, 0.0, 260.0, 64.0), "Region filter", &["All Regions", "NA East", "EU West", "Asia"]);
    g.button2(hdr, cr(20.0, 0.0, 196.0, 68.0), "Refresh button", "Refresh");

    // Barra de filtros
    let fbar = g.panel(p, top(128.0, 80.0, 36.0, 36.0), "Filter bar");
    let frow = g.hbox(fbar, cl(24.0, 0.0, 1280.0, 64.0), "Filter controls", 24.0);
    g.toggle(frow, sz(220.0, 64.0), "Hide full toggle", "Hide full", true);
    g.toggle(frow, sz(240.0, 64.0), "Hide empty toggle", "Hide empty", true);
    g.toggle(frow, sz(260.0, 64.0), "Open toggle", "No password", false);
    g.dropdown(frow, sz(300.0, 64.0), "Mode dropdown", &["All Modes", "Conquest", "Skirmish", "Escort"]);
    g.note(fbar, cr(24.0, 0.0, 280.0, 36.0), "Count note", "24 servers online", 24.0);

    // Tabla de servidores
    let tbl = g.panel(p, fill(36.0, 224.0, 36.0, 124.0), "Server table");
    let th = g.card(tbl, top(16.0, 56.0, 16.0, 16.0), "Table header");
    g.heading_l(th, cl(20.0, 0.0, 560.0, 34.0), "Col server", "SERVER NAME", 24.0);
    g.heading_l(th, cl(640.0, 0.0, 240.0, 34.0), "Col mode", "MODE", 24.0);
    g.heading_l(th, cl(940.0, 0.0, 300.0, 34.0), "Col map", "MAP", 24.0);
    g.heading_l(th, cl(1320.0, 0.0, 200.0, 34.0), "Col players", "PLAYERS", 24.0);
    g.value(th, cr(24.0, 0.0, 180.0, 34.0), "Col ping", "PING", 24.0);

    let rows = g.vbox(tbl, fill(16.0, 86.0, 16.0, 16.0), "Server rows", 10.0);
    let servers = [
        ("Alpha Outpost #04", "Conquest", "Red Ridge", "28 / 32", "24 ms"),
        ("Bravo Tactical EU", "Escort", "Harbor Yard", "20 / 24", "41 ms"),
        ("Iron Wolves 24/7", "Skirmish", "Snow Depot", "14 / 16", "32 ms"),
        ("Frontier Hardcore", "Conquest", "Dust Valley", "30 / 32", "55 ms"),
        ("Cadet Training Hub", "Skirmish", "Airfield", "10 / 16", "19 ms"),
        ("Night Ops Dedicated", "Escort", "Black Dock", "18 / 24", "48 ms"),
    ];
    for (i, (sname, mode, map, pl, ping)) in servers.iter().enumerate() {
        let r = if i == 0 {
            g.card_on(rows, sz(1816.0, 96.0), "Server row")
        } else {
            g.card(rows, sz(1816.0, 96.0), "Server row")
        };
        g.heading_l(r, cl(20.0, 0.0, 580.0, 38.0), "Server name", sname, 27.0);
        g.label(r, cl(640.0, 0.0, 240.0, 36.0), "Mode value", mode, 25.0);
        g.note(r, cl(940.0, 0.0, 300.0, 36.0), "Map value", map, 25.0);
        g.label(r, cl(1320.0, 0.0, 200.0, 36.0), "Player count", pl, 25.0);
        g.value(r, cr(24.0, 0.0, 180.0, 36.0), "Ping value", ping, 25.0);
    }

    // Pie de conexion
    let foot = g.panel(p, bottom(20.0, 88.0, 36.0, 36.0), "Action footer");
    g.label(foot, cl(24.0, 0.0, 820.0, 38.0), "Selected summary", "Selected: Alpha Outpost #04 (Conquest - Red Ridge)", 25.0);
    g.button2(foot, cr(280.0, 0.0, 220.0, 68.0), "Direct IP button", "Direct IP");
    g.button(foot, cr(20.0, 0.0, 240.0, 68.0), "Join button", "Join server");
}

/// Pantalla centrada de emparejamiento en curso con cronometro, detalles de cola y boton de cancelar.
fn online_matchmaking(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(24.0, 88.0, 48.0, 48.0), "Online status bar");
    g.heading_l(hdr, cl(28.0, 0.0, 420.0, 44.0), "Mode header", "ONLINE MATCHMAKING", 34.0);
    g.badge(hdr, cr(28.0, 0.0, 250.0, 44.0), "Region badge", "NA-EAST  28ms");

    // Tarjeta central de busqueda
    let box_p = g.panel(p, cc(0.0, -30.0, 960.0, 620.0), "Matchmaking panel");
    g.badge(box_p, tc(0.0, 30.0, 220.0, 44.0), "Queue badge", "QUEUE ACTIVE");
    g.heading(box_p, tc(0.0, 96.0, 880.0, 64.0), "Searching heading", "SEARCHING FOR MATCH...", 50.0);

    let timer_box = g.card_on(box_p, tc(0.0, 180.0, 380.0, 132.0), "Timer box");
    g.heading(timer_box, tc(0.0, 16.0, 340.0, 64.0), "Elapsed time", "01:24", 56.0);
    g.note(timer_box, bc(0.0, 14.0, 340.0, 34.0), "Estimate label", "Estimated wait: 01:45", 23.0);

    g.bar(box_p, tc(0.0, 336.0, 760.0, 24.0), "Search progress", 0.68, g.t.accent);

    let info = g.hbox(box_p, tc(0.0, 388.0, 860.0, 110.0), "Queue details", 18.0);
    for (k, v) in [("PLAYLIST", "Ranked 5v5"), ("MAP POOL", "6 Sectors"), ("SQUAD", "2 / 5 Ready")] {
        let ic = g.card(info, sz(274.0, 106.0), "Detail card");
        g.note(ic, top(14.0, 30.0, 14.0, 14.0), "Detail label", k, 22.0);
        g.heading_l(ic, bottom(16.0, 40.0, 14.0, 14.0), "Detail value", v, 28.0);
    }

    g.button(box_p, bc(0.0, 26.0, 340.0, 72.0), "Cancel queue button", "Cancel search");

    // Franja inferior de consejo
    let tip = g.panel(p, bottom(30.0, 120.0, 160.0, 160.0), "Tip bar");
    g.badge(tip, cl(28.0, 0.0, 108.0, 44.0), "Tip badge", "TIP");
    g.label(tip, cl(160.0, 0.0, 1040.0, 44.0), "Tip text", "You can browse your loadout or chat with your squad while waiting.", 26.0);
    g.button2(tip, cr(24.0, 0.0, 250.0, 68.0), "Change mode button", "Change mode");
}

/// Sala de grupo en cuatro columnas de jugadores con huecos de invitacion y barra inferior de amigos.
fn online_party_invites(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 96.0, 40.0, 40.0), "Party header");
    g.heading_l(hdr, cl(28.0, 0.0, 340.0, 48.0), "Party title", "PARTY LOBBY", 40.0);
    g.badge(hdr, cl(360.0, 0.0, 210.0, 44.0), "Code badge", "CODE: AX79");
    g.dropdown(hdr, cr(310.0, 0.0, 300.0, 68.0), "Playlist picker", &["3v3 Arena", "2v2 Duo", "Custom Match"]);
    g.button(hdr, cr(24.0, 0.0, 264.0, 68.0), "Start match button", "Start match");

    // 4 huecos de equipo en fila
    let slots = g.hbox_c(p, hmid(-36.0, 590.0, 40.0, 40.0), "Party slots", 24.0);
    let roster = [
        (true, "YO", "StrikerOne", "Captain - Gold III", "READY"),
        (true, "AL", "AlbaDash", "Wing - Gold II", "READY"),
        (false, "+", "Open Slot", "Invite a friend to fill slot 3", "INVITE"),
        (false, "+", "Open Slot", "Invite a friend to fill slot 4", "INVITE"),
    ];
    for (filled, glyph, name, sub, tag) in roster {
        let c = if filled {
            g.card_on(slots, sz(432.0, 580.0), "Party slot")
        } else {
            g.card(slots, sz(432.0, 580.0), "Empty slot")
        };
        if filled {
            g.art(c, top(20.0, 270.0, 20.0, 20.0), "Player art");
            g.badge(c, tr(34.0, 34.0, 128.0, 42.0), "Ready badge", tag);
            g.heading(c, tc(0.0, 316.0, 380.0, 44.0), "Member name", name, 34.0);
            g.note(c, tc(0.0, 368.0, 380.0, 38.0), "Member rank", sub, 24.0);
            g.button2(c, bc(0.0, 26.0, 340.0, 68.0), "Inspect button", "Change kit");
        } else {
            g.icon(c, tc(0.0, 120.0, 120.0, 120.0), "Invite icon", glyph);
            g.heading(c, tc(0.0, 274.0, 360.0, 44.0), "Empty heading", name, 34.0);
            g.note(c, tc(0.0, 330.0, 360.0, 64.0), "Empty note", sub, 24.0);
            g.button(c, bc(0.0, 26.0, 340.0, 72.0), "Invite slot button", "Invite friend");
        }
    }

    // Pie con opciones y amigos rapidos
    let foot = g.panel(p, bottom(24.0, 152.0, 40.0, 40.0), "Party footer");
    let opts = g.hbox(foot, cl(28.0, 0.0, 560.0, 68.0), "Party toggles", 24.0);
    g.toggle(opts, sz(260.0, 64.0), "Crossplay toggle", "Crossplay", true);
    g.toggle(opts, sz(260.0, 64.0), "Voice toggle", "Party voice", true);

    let quick = g.hbox(foot, cr(28.0, 0.0, 960.0, 108.0), "Quick invites", 16.0);
    for (init, fname) in [("RK", "RicoKick - Online"), ("ZN", "ZinaSprint - Online")] {
        let qc = g.card(quick, sz(472.0, 104.0), "Quick invite card");
        g.icon(qc, cl(16.0, 0.0, 56.0, 56.0), "Friend icon", init);
        g.label(qc, cl(86.0, 0.0, 220.0, 36.0), "Friend label", fname, 23.0);
        g.button2(qc, cr(14.0, 0.0, 144.0, 64.0), "Send invite button", "Invite");
    }
}

// ===============================================================================================================
// Creacion de contenido
// ===============================================================================================================

/// Modo foto con panel lateral de sliders y filtros a la izquierda y guias de teclas abajo.
fn photo_side_sliders(g: &mut G) {
    let p = g.page();
    g.scene();
    // Guias de encuadre sobre la escena
    g.rect(p, cc(240.0, -240.0, 520.0, 4.0), "Frame top", alpha(g.t.line, 180), 0.0);
    g.rect(p, cc(240.0, 240.0, 520.0, 4.0), "Frame bottom", alpha(g.t.line, 180), 0.0);

    // Panel lateral de ajustes
    let side = g.panel(p, left(24.0, 490.0, 24.0, 24.0), "Photo controls panel");
    g.heading_l(side, tl(24.0, 22.0, 420.0, 46.0), "Mode title", "PHOTO MODE", 36.0);
    let tabs = g.hbox(side, tl(24.0, 82.0, 442.0, 68.0), "Photo tabs", 10.0);
    for (i, t) in ["Lens", "Color", "Depth"].iter().enumerate() {
        if i == 0 {
            g.button(tabs, sz(140.0, 68.0), "Tab button", t);
        } else {
            g.button2(tabs, sz(140.0, 68.0), "Tab button", t);
        }
    }

    let sbox = g.vbox(side, tl(24.0, 172.0, 442.0, 480.0), "Slider list", 14.0);
    for (name, val_txt, val) in [
        ("Field of View", "52 deg", 0.52),
        ("Exposure", "+0.4", 0.62),
        ("Contrast", "110%", 0.58),
        ("Saturation", "105%", 0.54),
        ("Depth Blur", "35%", 0.35),
    ] {
        let row = g.card(sbox, sz(442.0, 84.0), "Slider row");
        g.label(row, tl(14.0, 10.0, 260.0, 32.0), "Param name", name, 23.0);
        g.value(row, tr(14.0, 10.0, 130.0, 32.0), "Param value", val_txt, 23.0);
        g.slider(row, bottom(12.0, 26.0, 14.0, 14.0), "Param slider", val);
    }

    g.label(side, tl(24.0, 676.0, 320.0, 34.0), "Filter label", "Color preset", 24.0);
    g.dropdown(side, tl(24.0, 718.0, 442.0, 68.0), "Filter dropdown", &["Natural", "Golden Hour", "Forest Mist", "Noir"]);
    g.toggle(side, tl(24.0, 804.0, 442.0, 64.0), "Grid toggle", "Rule of thirds grid", true);

    g.button2(side, bl(24.0, 22.0, 196.0, 68.0), "Reset button", "Reset");
    g.button(side, br(24.0, 22.0, 228.0, 72.0), "Capture button", "Capture");

    // Barra inferior derecha de atajos
    let hints = g.panel(p, br(24.0, 24.0, 800.0, 76.0), "Shortcuts bar");
    let hrow = g.hbox_c(hints, full(), "Hint row", 10.0);
    for (k, lbl) in [("WASD", "Move"), ("Q/E", "Roll"), ("H", "Hide UI"), ("SPC", "Snap")] {
        g.key(hrow, sz(72.0, 44.0), "Shortcut key", k);
        g.note(hrow, sz(96.0, 38.0), "Shortcut text", lbl, 22.0);
    }
}

/// Modo foto enfocado en marcos decorativos y pegatinas con cajon lateral derecho y barra inferior.
fn photo_frames_stickers(g: &mut G) {
    let p = g.page();
    g.scene();
    // Barra superior de pestanas
    let top_bar = g.panel(p, tc(-200.0, 24.0, 740.0, 84.0), "Mode switch bar");
    let tabs = g.hbox_c(top_bar, fill(12.0, 8.0, 12.0, 8.0), "Mode tabs", 12.0);
    for (i, t) in ["Camera", "Frames", "Stickers", "Logo"].iter().enumerate() {
        if i == 2 {
            g.button(tabs, sz(168.0, 68.0), "Mode button", t);
        } else {
            g.button2(tabs, sz(168.0, 68.0), "Mode button", t);
        }
    }

    // Esquinas decorativas de marco de foto
    g.rect(p, tl(140.0, 180.0, 90.0, 8.0), "Frame corner tl", g.t.accent, 4.0);
    g.rect(p, bl(140.0, 160.0, 90.0, 8.0), "Frame corner bl", g.t.accent, 4.0);

    // Cajon derecho de marcos y pegatinas
    let drawer = g.panel(p, right(24.0, 480.0, 24.0, 126.0), "Stickers drawer");
    g.heading_l(drawer, tl(24.0, 22.0, 420.0, 44.0), "Drawer heading", "FRAMES & STICKERS", 32.0);
    g.note(drawer, tl(24.0, 76.0, 420.0, 32.0), "Frame section", "FRAME STYLE", 22.0);
    let frow = g.hbox(drawer, tl(24.0, 114.0, 432.0, 108.0), "Frame row", 12.0);
    for (i, fname) in ["None", "Postcard", "Film"].iter().enumerate() {
        let fc = if i == 1 {
            g.card_on(frow, sz(136.0, 104.0), "Frame option")
        } else {
            g.card(frow, sz(136.0, 104.0), "Frame option")
        };
        g.heading(fc, full(), "Frame label", fname, 24.0);
    }

    g.note(drawer, tl(24.0, 244.0, 420.0, 32.0), "Sticker section", "STAMP STICKERS", 22.0);
    let sgrid = g.grid(drawer, tl(24.0, 284.0, 432.0, 500.0), "Sticker grid", 136.0, 152.0, 12.0);
    let stamps = [("HI", "Wave"), ("<3", "Heart"), ("!", "Spark"), ("GG", "Cheer"), ("Z", "Sleep"), ("*", "Star")];
    for (i, (gl, name)) in stamps.iter().enumerate() {
        let sc = if i == 0 {
            g.card_on(sgrid, sz(136.0, 152.0), "Sticker card")
        } else {
            g.card(sgrid, sz(136.0, 152.0), "Sticker card")
        };
        g.icon(sc, tc(0.0, 18.0, 68.0, 68.0), "Sticker icon", gl);
        g.label(sc, bc(0.0, 14.0, 116.0, 36.0), "Sticker name", name, 22.0);
    }
    g.toggle(drawer, bl(24.0, 20.0, 420.0, 64.0), "Watermark toggle", "Show game logo", true);

    // Barra inferior de captura
    let dock = g.panel(p, bottom(20.0, 90.0, 24.0, 24.0), "Capture dock");
    g.dropdown(dock, cl(20.0, 0.0, 240.0, 66.0), "Aspect dropdown", &["16:9 Wide", "1:1 Square", "4:3 Classic"]);
    g.button2(dock, cr(264.0, 0.0, 236.0, 68.0), "Clear stamps button", "Clear stickers");
    g.button(dock, cr(20.0, 0.0, 224.0, 68.0), "Save photo button", "Save photo");
}

/// Modo foto minimalista con telemetria de camara arriba y leyenda detallada de teclas abajo.
fn photo_camera_controls(g: &mut G) {
    let p = g.page();
    g.scene();
    // Visor central minimalista
    g.rect(p, cc(0.0, -40.0, 48.0, 4.0), "Reticle horizontal", g.t.line, 0.0);
    g.rect(p, cc(0.0, -40.0, 4.0, 48.0), "Reticle vertical", g.t.line, 0.0);

    // Barra superior de telemetria
    let tel = g.panel(p, tc(0.0, 24.0, 860.0, 76.0), "Camera telemetry");
    let trow = g.hbox_c(tel, full(), "Telemetry items", 28.0);
    for text in ["LENS 50mm", "F/2.8", "ISO 200", "1/500s", "RAW"] {
        g.heading(trow, sz(140.0, 42.0), "Telemetry readout", text, 25.0);
    }

    // Panel inferior izquierdo: rig de camara
    let rig = g.panel(p, bl(28.0, 28.0, 480.0, 316.0), "Camera rig panel");
    g.heading_l(rig, tl(24.0, 20.0, 420.0, 40.0), "Rig heading", "CAMERA RIG", 28.0);
    let tbox = g.vbox(rig, tl(24.0, 72.0, 432.0, 224.0), "Rig toggles", 12.0);
    g.toggle(tbox, sz(432.0, 64.0), "Free cam toggle", "Free orbit camera", true);
    g.toggle(tbox, sz(432.0, 64.0), "Hide player toggle", "Hide character", false);
    g.toggle(tbox, sz(432.0, 64.0), "Freeze toggle", "Freeze world motion", true);

    // Panel inferior derecho: leyenda de controles y disparo
    let ctrl = g.panel(p, br(28.0, 28.0, 840.0, 316.0), "Controls legend");
    g.heading_l(ctrl, tl(24.0, 20.0, 400.0, 40.0), "Controls heading", "CAMERA CONTROLS", 28.0);
    let cgrid = g.grid(ctrl, tl(24.0, 72.0, 792.0, 144.0), "Key grid", 254.0, 64.0, 14.0);
    for (k, desc) in [
        ("WASD", "Pan camera"),
        ("MOUSE", "Aim lens"),
        ("Z / X", "Zoom in/out"),
        ("Q / E", "Tilt roll"),
        ("F", "Auto focus"),
        ("R", "Reset view"),
    ] {
        let kc = g.card(cgrid, sz(254.0, 64.0), "Key item");
        g.key(kc, cl(10.0, 0.0, 82.0, 44.0), "Key cap", k);
        g.label(kc, cl(104.0, 0.0, 140.0, 36.0), "Key action", desc, 22.0);
    }
    g.button2(ctrl, bl(24.0, 20.0, 220.0, 68.0), "Exit photo button", "Exit mode");
    g.button(ctrl, br(24.0, 20.0, 280.0, 72.0), "Shutter button", "Take photo");
}

/// Editor de niveles clasico con barra de herramientas, paleta de bloques, lienzo cuadriculado e inspector.
fn editor_classic_grid(g: &mut G) {
    let p = g.page();
    g.background();
    // Barra de herramientas superior
    let bar = g.panel(p, top(0.0, 92.0, 0.0, 0.0), "Top toolbar");
    g.heading_l(bar, cl(24.0, 0.0, 340.0, 44.0), "Map title", "DUNGEON_04", 34.0);
    let tools = g.hbox(bar, cc(-40.0, 0.0, 660.0, 68.0), "Tool buttons", 12.0);
    for (i, t) in ["Select", "Paint", "Erase", "Fill"].iter().enumerate() {
        if i == 1 {
            g.button(tools, sz(152.0, 68.0), "Tool button", t);
        } else {
            g.button2(tools, sz(152.0, 68.0), "Tool button", t);
        }
    }
    g.button2(bar, cr(232.0, 0.0, 176.0, 68.0), "Save map button", "Save");
    g.button(bar, cr(24.0, 0.0, 192.0, 68.0), "Playtest button", "Test");

    // Paleta izquierda de piezas
    let pal = g.panel(p, left(0.0, 320.0, 92.0, 0.0), "Tile palette");
    g.heading_l(pal, tl(20.0, 18.0, 280.0, 40.0), "Palette heading", "PALETTE", 28.0);
    g.dropdown(pal, tl(20.0, 68.0, 280.0, 64.0), "Palette category", &["Terrain", "Hazards", "Items", "Enemies"]);
    let tgrid = g.grid(pal, tl(20.0, 150.0, 280.0, 780.0), "Tile grid", 132.0, 132.0, 16.0);
    let tiles = ["Wall", "Floor", "Spike", "Door", "Torch", "Chest", "Spawn", "Exit"];
    for (i, name) in tiles.iter().enumerate() {
        let tc_card = if i == 0 {
            g.card_on(tgrid, sz(132.0, 132.0), "Tile button")
        } else {
            g.card(tgrid, sz(132.0, 132.0), "Tile button")
        };
        g.art(tc_card, top(12.0, 68.0, 16.0, 16.0), "Tile sprite");
        g.heading(tc_card, bc(0.0, 10.0, 116.0, 32.0), "Tile name", name, 22.0);
    }

    // Lienzo cuadriculado central
    let canvas = g.panel(p, fill(336.0, 108.0, 376.0, 16.0), "Grid canvas");
    g.badge(canvas, tl(20.0, 18.0, 220.0, 40.0), "Grid badge", "GRID 16x16");
    g.note(canvas, tr(20.0, 22.0, 240.0, 34.0), "Cursor coords", "Cursor: X 4, Y 2", 22.0);
    let cells = g.grid(canvas, cc(0.0, 24.0, 1060.0, 796.0), "Level cells", 250.0, 250.0, 20.0);
    let map_cells = [
        "Wall", "Wall", "Torch", "Wall",
        "Spawn", "Floor", "Floor", "Door",
        "Wall", "Spike", "Chest", "Exit",
    ];
    for (i, label) in map_cells.iter().enumerate() {
        let c = if i == 5 {
            g.card_on(cells, sz(250.0, 250.0), "Map cell")
        } else {
            g.card(cells, sz(250.0, 250.0), "Map cell")
        };
        g.heading(c, cc(0.0, 0.0, 210.0, 40.0), "Cell label", label, 26.0);
    }

    // Inspector derecho de propiedades
    let insp = g.panel(p, right(0.0, 360.0, 92.0, 0.0), "Properties panel");
    g.heading_l(insp, tl(24.0, 18.0, 312.0, 40.0), "Inspector heading", "PROPERTIES", 28.0);
    let sel = g.card(insp, tl(24.0, 72.0, 312.0, 130.0), "Selected tile box");
    g.heading_l(sel, tl(16.0, 16.0, 280.0, 38.0), "Selected name", "Stone Floor", 28.0);
    g.note(sel, tl(16.0, 62.0, 280.0, 34.0), "Selected coords", "Grid X: 1  Y: 1", 23.0);
    g.label(insp, tl(24.0, 228.0, 312.0, 34.0), "Rotation label", "Rotation: 90 deg", 24.0);
    g.slider(insp, tl(24.0, 268.0, 312.0, 28.0), "Rotation slider", 0.25);
    let props = g.vbox(insp, tl(24.0, 326.0, 312.0, 230.0), "Tile flags", 14.0);
    g.toggle(props, sz(312.0, 64.0), "Solid toggle", "Solid collision", true);
    g.toggle(props, sz(312.0, 64.0), "Hazard toggle", "Deals damage", false);
    g.toggle(props, sz(312.0, 64.0), "Hidden toggle", "Secret tile", false);
    g.button2(insp, bc(0.0, 24.0, 312.0, 68.0), "Delete tile button", "Remove tile");
}

/// Editor visual con lienzo panoramico y biblioteca horizontal de piezas anclada abajo.
fn editor_bottom_library(g: &mut G) {
    let p = g.page();
    g.background();
    // Barra superior flotante
    let top_p = g.panel(p, top(20.0, 88.0, 24.0, 24.0), "Builder bar");
    g.heading_l(top_p, cl(24.0, 0.0, 340.0, 44.0), "Builder title", "ISLAND BUILDER", 34.0);
    g.label(top_p, cl(400.0, 0.0, 180.0, 34.0), "Budget label", "Budget 42%", 24.0);
    g.bar(top_p, cl(580.0, 0.0, 260.0, 24.0), "Budget bar", 0.42, g.t.good);
    let acts = g.hbox(top_p, cr(20.0, 0.0, 540.0, 68.0), "Builder actions", 12.0);
    g.button2(acts, sz(152.0, 68.0), "Undo button", "Undo");
    g.button2(acts, sz(152.0, 68.0), "Redo button", "Redo");
    g.button(acts, sz(210.0, 68.0), "Play button", "Playtest");

    // Area de vista previa del nivel
    let view = g.panel(p, fill(24.0, 124.0, 24.0, 296.0), "Stage viewport");
    g.art(view, fill(16.0, 16.0, 16.0, 16.0), "Stage sky preview");
    let p1 = g.card_on(view, tl(180.0, 260.0, 240.0, 110.0), "Placed start");
    g.badge(p1, tc(0.0, 14.0, 128.0, 40.0), "Start tag", "START");
    g.heading(p1, bc(0.0, 16.0, 210.0, 36.0), "Start name", "Launch Gate", 24.0);
    let p2 = g.card(view, cc(0.0, -40.0, 240.0, 110.0), "Placed ramp");
    g.badge(p2, tc(0.0, 14.0, 128.0, 40.0), "Ramp tag", "PROP");
    g.heading(p2, bc(0.0, 16.0, 210.0, 36.0), "Ramp name", "Bounce Pad", 24.0);
    let p3 = g.card(view, tr(180.0, 200.0, 240.0, 110.0), "Placed goal");
    g.badge(p3, tc(0.0, 14.0, 128.0, 40.0), "Goal tag", "GOAL");
    g.heading(p3, bc(0.0, 16.0, 210.0, 36.0), "Goal name", "Finish Arch", 24.0);

    // Biblioteca inferior de piezas
    let dock = g.panel(p, bottom(20.0, 260.0, 24.0, 24.0), "Piece library dock");
    let tabs = g.hbox(dock, tl(20.0, 16.0, 960.0, 68.0), "Piece tabs", 12.0);
    for (i, t) in ["Blocks", "Ramps", "Traps", "Decor", "Goals"].iter().enumerate() {
        if i == 0 {
            g.button(tabs, sz(180.0, 68.0), "Category button", t);
        } else {
            g.button2(tabs, sz(180.0, 68.0), "Category button", t);
        }
    }
    g.input(dock, tr(20.0, 18.0, 420.0, 64.0), "Search pieces", "Search parts...");

    let strip = g.hbox(dock, bottom(16.0, 148.0, 20.0, 20.0), "Pieces strip", 16.0);
    let parts = [
        ("Grass Block", "Cost 2"),
        ("Wood Bridge", "Cost 4"),
        ("Bounce Pad", "Cost 6"),
        ("Spin Gate", "Cost 8"),
        ("Speed Ring", "Cost 5"),
        ("Goal Flag", "Cost 10"),
    ];
    for (i, (pname, cost)) in parts.iter().enumerate() {
        let c = if i == 2 {
            g.card_on(strip, sz(292.0, 144.0), "Part card")
        } else {
            g.card(strip, sz(292.0, 144.0), "Part card")
        };
        g.art(c, cl(14.0, 0.0, 96.0, 108.0), "Part thumb");
        g.heading_l(c, tl(124.0, 32.0, 156.0, 38.0), "Part name", pname, 24.0);
        g.note(c, tl(124.0, 78.0, 156.0, 34.0), "Part cost", cost, 22.0);
    }
}

/// Editor tecnico organizado por capas a la izquierda, plano central e inspector de parametros a la derecha.
fn editor_layers_inspector(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(0.0, 88.0, 0.0, 0.0), "Editor header");
    g.heading_l(hdr, cl(24.0, 0.0, 440.0, 44.0), "Editor title", "SECTOR ARCHITECT v3", 34.0);
    let mid = g.hbox(hdr, cc(-40.0, 0.0, 500.0, 64.0), "Snap toggles", 20.0);
    g.toggle(mid, sz(230.0, 64.0), "Snap toggle", "Snap grid", true);
    g.toggle(mid, sz(230.0, 64.0), "Wire toggle", "Show bounds", true);
    g.button2(hdr, cr(248.0, 0.0, 190.0, 68.0), "Export button", "Export");
    g.button(hdr, cr(24.0, 0.0, 206.0, 68.0), "Simulate button", "Simulate");

    // Columna izquierda: capas
    let lcol = g.panel(p, left(0.0, 380.0, 88.0, 0.0), "Layers panel");
    g.heading_l(lcol, tl(22.0, 20.0, 320.0, 40.0), "Layers heading", "LAYERS", 30.0);
    let lbox = g.vbox(lcol, tl(20.0, 76.0, 340.0, 780.0), "Layers list", 14.0);
    let layers = [
        ("Spawns", "6 nodes", "ON"),
        ("Triggers", "12 nodes", "ON"),
        ("Lighting", "9 nodes", "ON"),
        ("Terrain", "44 nodes", "ON"),
        ("Backdrop", "3 nodes", "OFF"),
    ];
    for (i, (lname, cnt, vis)) in layers.iter().enumerate() {
        let c = if i == 1 {
            g.card_on(lbox, sz(340.0, 128.0), "Layer row")
        } else {
            g.card(lbox, sz(340.0, 128.0), "Layer row")
        };
        g.heading_l(c, tl(18.0, 22.0, 200.0, 38.0), "Layer name", lname, 27.0);
        g.note(c, tl(18.0, 68.0, 200.0, 34.0), "Layer count", cnt, 22.0);
        g.badge(c, cr(16.0, 0.0, 88.0, 42.0), "Vis badge", vis);
    }
    g.button2(lcol, bc(0.0, 20.0, 340.0, 68.0), "Add layer button", "+ New layer");

    // Plano central
    let bp = g.panel(p, fill(396.0, 104.0, 416.0, 16.0), "Blueprint viewport");
    g.badge(bp, tl(20.0, 18.0, 250.0, 40.0), "View badge", "TOP BLUEPRINT");
    let z1 = g.card(bp, tl(80.0, 140.0, 320.0, 220.0), "Zone spawn");
    g.heading(z1, cc(0.0, -18.0, 280.0, 40.0), "Zone name", "SPAWN BAY", 28.0);
    g.note(z1, cc(0.0, 26.0, 260.0, 34.0), "Zone dims", "24m x 16m", 22.0);
    let z2 = g.card_on(bp, cc(20.0, 30.0, 380.0, 260.0), "Zone core");
    g.badge(z2, tc(0.0, 16.0, 160.0, 40.0), "Selected badge", "SELECTED");
    g.heading(z2, cc(0.0, 6.0, 340.0, 42.0), "Core name", "CORE CHAMBER", 30.0);
    g.note(z2, bc(0.0, 22.0, 320.0, 34.0), "Core info", "Trigger Volume #04", 22.0);
    let z3 = g.card(bp, br(80.0, 80.0, 300.0, 200.0), "Zone exit");
    g.heading(z3, cc(0.0, -16.0, 260.0, 38.0), "Exit name", "AIRLOCK B", 26.0);
    g.note(z3, cc(0.0, 24.0, 240.0, 34.0), "Exit info", "Linked to Trigger #04", 22.0);

    // Inspector derecho
    let insp = g.panel(p, right(0.0, 400.0, 88.0, 0.0), "Inspector panel");
    g.heading_l(insp, tl(24.0, 20.0, 340.0, 40.0), "Inspector title", "INSPECTOR", 30.0);
    g.label(insp, tl(24.0, 76.0, 352.0, 34.0), "Obj label", "Object name", 23.0);
    g.input(insp, tl(24.0, 114.0, 352.0, 64.0), "Obj name input", "Core Chamber");
    g.label(insp, tl(24.0, 198.0, 352.0, 34.0), "Tag label", "Volume type", 23.0);
    g.dropdown(insp, tl(24.0, 236.0, 352.0, 64.0), "Tag dropdown", &["Trigger Zone", "Spawn Area", "Safe Room"]);

    let sliders = g.vbox(insp, tl(24.0, 326.0, 352.0, 300.0), "Transform sliders", 14.0);
    for (lbl, val) in [("Scale", 0.72), ("Glow Intensity", 0.45), ("Ceiling Height", 0.60)] {
        let sc = g.card(sliders, sz(352.0, 88.0), "Slider card");
        g.label(sc, tl(14.0, 12.0, 320.0, 32.0), "Slider name", lbl, 23.0);
        g.slider(sc, bottom(14.0, 26.0, 14.0, 14.0), "Slider bar", val);
    }
    g.toggle(insp, tl(24.0, 650.0, 352.0, 64.0), "Lock toggle", "Lock object", false);
    g.button2(insp, bc(0.0, 24.0, 352.0, 68.0), "Duplicate button", "Duplicate node");
}

/// Taller de vinilos con capas a la izquierda, vista del vehiculo al centro y selector de color a la derecha.
fn liveries_layers_color(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 88.0, 28.0, 28.0), "Garage header");
    g.heading_l(hdr, cl(24.0, 0.0, 480.0, 44.0), "Garage title", "LIVERY GARAGE - GT-88", 34.0);
    g.button2(hdr, cr(248.0, 0.0, 188.0, 68.0), "Back button", "Back");
    g.button(hdr, cr(20.0, 0.0, 210.0, 68.0), "Save livery button", "Save livery");

    // Izquierda: capas de calcomanias
    let lcol = g.panel(p, tl(28.0, 124.0, 396.0, 932.0), "Decal layers panel");
    g.heading_l(lcol, tl(22.0, 20.0, 350.0, 40.0), "Layers title", "LAYERS (5 / 50)", 28.0);
    let lbox = g.vbox(lcol, tl(20.0, 76.0, 356.0, 740.0), "Layer list", 14.0);
    let layers = [
        ("01", "Racing Stripe", "Hood - Scale 1.2"),
        ("02", "Number 42", "Door - Scale 0.9"),
        ("03", "Apex Crest", "Roof - Scale 1.0"),
        ("04", "Side Chevron", "Left/Right"),
        ("05", "Carbon Vent", "Front Bumper"),
    ];
    for (i, (num, title, sub)) in layers.iter().enumerate() {
        let c = if i == 0 {
            g.card_on(lbox, sz(356.0, 130.0), "Decal layer")
        } else {
            g.card(lbox, sz(356.0, 130.0), "Decal layer")
        };
        g.icon(c, cl(16.0, 0.0, 56.0, 56.0), "Layer index", num);
        g.heading_l(c, tl(86.0, 24.0, 250.0, 38.0), "Layer label", title, 26.0);
        g.note(c, tl(86.0, 68.0, 250.0, 34.0), "Layer detail", sub, 22.0);
    }
    g.button(lcol, bc(0.0, 20.0, 356.0, 70.0), "Add layer button", "+ Add decal");

    // Centro: vista previa del vehiculo
    let stage = g.panel(p, fill(444.0, 124.0, 476.0, 24.0), "Vehicle preview");
    g.art(stage, fill(24.0, 24.0, 24.0, 124.0), "Vehicle render");
    g.badge(stage, tl(40.0, 40.0, 220.0, 44.0), "Zone badge", "SIDE VIEW");
    let cam = g.hbox_c(stage, bottom(24.0, 76.0, 24.0, 24.0), "Camera angles", 14.0);
    for (i, ang) in ["Front", "Side", "Top", "Rear"].iter().enumerate() {
        if i == 1 {
            g.button(cam, sz(180.0, 68.0), "Angle button", ang);
        } else {
            g.button2(cam, sz(180.0, 68.0), "Angle button", ang);
        }
    }

    // Derecha: selector de color y acabado
    let rcol = g.panel(p, tr(28.0, 124.0, 428.0, 932.0), "Color inspector");
    g.heading_l(rcol, tl(22.0, 20.0, 380.0, 40.0), "Color heading", "PAINT & FINISH", 28.0);
    let swatches = g.grid(rcol, tl(22.0, 76.0, 384.0, 200.0), "Color swatches", 118.0, 88.0, 14.0);
    let colors = [g.t.accent, g.t.accent2, g.t.good, g.t.warn, g.t.bad, g.t.line];
    for (i, col) in colors.iter().enumerate() {
        let sc = if i == 0 {
            g.card_on(swatches, sz(118.0, 88.0), "Swatch cell")
        } else {
            g.card(swatches, sz(118.0, 88.0), "Swatch cell")
        };
        g.rect(sc, fill(12.0, 12.0, 12.0, 12.0), "Color fill", *col, 6.0);
    }

    g.label(rcol, tl(22.0, 298.0, 380.0, 34.0), "Finish label", "Surface finish", 24.0);
    g.dropdown(rcol, tl(22.0, 338.0, 384.0, 64.0), "Finish picker", &["Gloss", "Matte", "Metallic", "Carbon"]);

    let sls = g.vbox(rcol, tl(22.0, 426.0, 384.0, 310.0), "Transform controls", 14.0);
    for (lbl, v) in [("Scale", 0.65), ("Rotation", 0.30), ("Opacity", 0.92)] {
        let row = g.card(sls, sz(384.0, 90.0), "Control row");
        g.label(row, tl(14.0, 12.0, 350.0, 32.0), "Control name", lbl, 23.0);
        g.slider(row, bottom(14.0, 26.0, 14.0, 14.0), "Control slider", v);
    }
    g.toggle(rcol, bl(22.0, 108.0, 384.0, 64.0), "Mirror toggle", "Mirror both sides", true);
    g.button2(rcol, bc(0.0, 20.0, 384.0, 68.0), "Reset layer button", "Reset transform");
}

/// Rejilla de biblioteca de calcomanias con panel lateral de detalle y colocacion.
fn liveries_decal_grid(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 92.0, 32.0, 32.0), "Decal library bar");
    g.heading_l(hdr, cl(24.0, 0.0, 380.0, 44.0), "Library title", "DECAL LIBRARY", 36.0);
    let tabs = g.hbox(hdr, cc(-60.0, 0.0, 740.0, 68.0), "Decal categories", 12.0);
    for (i, t) in ["Shapes", "Stripes", "Numbers", "Badges"].iter().enumerate() {
        if i == 0 {
            g.button(tabs, sz(174.0, 68.0), "Category button", t);
        } else {
            g.button2(tabs, sz(174.0, 68.0), "Category button", t);
        }
    }
    g.input(hdr, cr(20.0, 0.0, 340.0, 64.0), "Search decals", "Search decals...");

    // Rejilla izquierda de calcomanias
    let catalog = g.panel(p, tl(32.0, 132.0, 1244.0, 916.0), "Decal grid panel");
    let grid = g.grid(catalog, fill(24.0, 24.0, 24.0, 24.0), "Decals grid", 284.0, 276.0, 20.0);
    let decals = [
        ("Z", "Cyber Bolt"),
        ("/\\", "Apex Peak"),
        ("<>", "Prism Core"),
        ("++", "Grid Cross"),
        ("42", "Race Num 42"),
        ("<<", "Aero Arrow"),
        ("O", "Halo Ring"),
        ("X", "Strike Mark"),
        ("V", "Wing Crest"),
        ("=", "Twin Bars"),
        ("#", "Mesh Patch"),
        ("*", "Nova Star"),
    ];
    for (i, (gl, name)) in decals.iter().enumerate() {
        let c = if i == 0 {
            g.card_on(grid, sz(284.0, 276.0), "Decal card")
        } else {
            g.card(grid, sz(284.0, 276.0), "Decal card")
        };
        g.icon(c, tc(0.0, 32.0, 128.0, 128.0), "Decal glyph", gl);
        g.heading(c, bc(0.0, 52.0, 250.0, 38.0), "Decal title", name, 26.0);
        g.note(c, bc(0.0, 16.0, 240.0, 30.0), "Decal group", "Unlocked", 22.0);
    }

    // Panel derecho de seleccion
    let det = g.panel(p, tr(32.0, 132.0, 588.0, 916.0), "Decal detail panel");
    g.heading_l(det, tl(28.0, 24.0, 520.0, 44.0), "Detail heading", "SELECTED DECAL", 32.0);
    let stage = g.card_on(det, tc(0.0, 88.0, 532.0, 360.0), "Decal stage");
    g.icon(stage, cc(0.0, 0.0, 196.0, 196.0), "Selected glyph", "Z");
    g.heading_l(det, tl(28.0, 474.0, 520.0, 46.0), "Selected title", "CYBER BOLT", 38.0);
    g.note(det, tl(28.0, 526.0, 520.0, 40.0), "Selected info", "Geometric pack - Supports custom tint", 24.0);
    g.label(det, tl(28.0, 594.0, 520.0, 36.0), "Scale label", "Initial scale", 24.0);
    g.slider(det, tl(28.0, 636.0, 532.0, 28.0), "Scale slider", 0.55);
    g.button(det, bc(0.0, 114.0, 532.0, 74.0), "Place decal button", "Place on vehicle");
    g.button2(det, bc(0.0, 28.0, 532.0, 68.0), "Favorite button", "Add to favorites");
}

/// Editor de patrones de camuflaje con vista del casco a la izquierda y canales de color y desgaste a la derecha.
fn liveries_pattern_editor(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 88.0, 32.0, 32.0), "Pattern studio bar");
    g.heading_l(hdr, cl(24.0, 0.0, 540.0, 44.0), "Studio title", "CAMO & PATTERN STUDIO", 34.0);
    g.dropdown(hdr, cr(284.0, 0.0, 300.0, 64.0), "Pattern preset", &["Woodland", "Desert Hex", "Urban Slate", "Arctic"]);
    g.button(hdr, cr(20.0, 0.0, 240.0, 68.0), "Apply pattern button", "Apply pattern");

    // Izquierda: vista del vehiculo y zonas
    let prev = g.panel(p, tl(32.0, 128.0, 1120.0, 920.0), "Hull preview panel");
    g.art(prev, fill(24.0, 24.0, 24.0, 128.0), "Camo preview art");
    g.badge(prev, tl(44.0, 44.0, 240.0, 44.0), "Pattern tag", "WOODLAND MK-II");
    let zones = g.hbox_c(prev, bottom(24.0, 76.0, 24.0, 24.0), "Zone buttons", 16.0);
    for (i, z) in ["Hull", "Turret", "Wings", "Trim"].iter().enumerate() {
        if i == 0 {
            g.button(zones, sz(230.0, 68.0), "Zone button", z);
        } else {
            g.button2(zones, sz(230.0, 68.0), "Zone button", z);
        }
    }

    // Derecha: canales de color y parametros de patron
    let ctrl = g.panel(p, tr(32.0, 128.0, 712.0, 920.0), "Pattern parameters");
    g.heading_l(ctrl, tl(24.0, 22.0, 640.0, 42.0), "Params heading", "COLOR CHANNELS", 30.0);
    let ch = g.hbox(ctrl, tl(24.0, 76.0, 664.0, 118.0), "Channel cards", 14.0);
    for (name, col) in [("Base", g.t.panel2), ("Patch A", g.t.accent), ("Patch B", g.t.good), ("Wear", g.t.warn)] {
        let cc_card = g.card(ch, sz(155.0, 114.0), "Channel card");
        g.rect(cc_card, top(12.0, 48.0, 12.0, 12.0), "Channel swatch", col, 4.0);
        g.heading(cc_card, bc(0.0, 10.0, 140.0, 34.0), "Channel label", name, 22.0);
    }

    g.heading_l(ctrl, tl(24.0, 220.0, 640.0, 40.0), "Tuning heading", "PATTERN TUNING", 28.0);
    let sl = g.vbox(ctrl, tl(24.0, 272.0, 664.0, 420.0), "Pattern sliders", 14.0);
    for (lbl, txt, val) in [
        ("Pattern Scale", "125%", 0.62),
        ("Pattern Angle", "45 deg", 0.25),
        ("Paint Wear", "38%", 0.38),
        ("Field Dirt", "20%", 0.20),
    ] {
        let r = g.card(sl, sz(664.0, 92.0), "Tuning row");
        g.label(r, tl(16.0, 12.0, 380.0, 32.0), "Tuning name", lbl, 24.0);
        g.value(r, tr(16.0, 12.0, 180.0, 32.0), "Tuning value", txt, 24.0);
        g.slider(r, bottom(14.0, 26.0, 16.0, 16.0), "Tuning slider", val);
    }
    g.toggle(ctrl, bl(24.0, 116.0, 664.0, 64.0), "Sync toggle", "Apply to all hull zones", true);
    g.button2(ctrl, bl(24.0, 24.0, 310.0, 68.0), "Randomize button", "Randomize");
    g.button(ctrl, br(24.0, 24.0, 330.0, 70.0), "Save preset button", "Save preset");
}

/// Galeria de creaciones de la comunidad con filtros laterales y rejilla valorada.
fn discovery_grid_filters(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 92.0, 36.0, 36.0), "Workshop header");
    g.heading_l(hdr, cl(24.0, 0.0, 440.0, 44.0), "Workshop title", "COMMUNITY WORKSHOP", 34.0);
    g.input(hdr, cc(-40.0, 0.0, 460.0, 64.0), "Search workshop", "Search maps, quests, skins...");
    g.dropdown(hdr, cr(248.0, 0.0, 264.0, 64.0), "Sort picker", &["Most Popular", "Top Rated", "Newest"]);
    g.button(hdr, cr(20.0, 0.0, 208.0, 68.0), "Upload button", "+ Upload");

    // Barra lateral de filtros
    let side = g.panel(p, tl(36.0, 132.0, 332.0, 916.0), "Filters sidebar");
    g.heading_l(side, tl(22.0, 20.0, 280.0, 40.0), "Filters heading", "CATEGORIES", 28.0);
    let cats = g.vbox(side, tl(20.0, 76.0, 292.0, 420.0), "Category menu", 12.0);
    for (i, c) in ["All Creations", "Adventure Maps", "Puzzles", "Skins & Packs", "Co-op Trials"].iter().enumerate() {
        g.menu_item(cats, sz(292.0, 68.0), "Category item", c, i == 0);
    }
    let opts = g.vbox(side, tl(20.0, 540.0, 292.0, 160.0), "Filter toggles", 14.0);
    g.toggle(opts, sz(292.0, 64.0), "Verified toggle", "Verified only", true);
    g.toggle(opts, sz(292.0, 64.0), "Coop toggle", "Co-op ready", false);
    g.button2(side, bc(0.0, 24.0, 292.0, 68.0), "Clear filters button", "Reset filters");

    // Rejilla de 6 creaciones
    let main = g.panel(p, tr(36.0, 132.0, 1492.0, 916.0), "Creations grid panel");
    let grid = g.grid(main, fill(24.0, 24.0, 24.0, 24.0), "Creations grid", 468.0, 424.0, 20.0);
    let mods = [
        ("Mossy Citadel", "by FernWeaver", "4.9/5", "18.4k plays"),
        ("River Crossing", "by CedarFox", "4.8/5", "12.1k plays"),
        ("Sunlit Ruins", "by LarkSong", "4.7/5", "9.6k plays"),
        ("Canopy Glide", "by BriarOwl", "4.9/5", "15.0k plays"),
        ("Pebble Puzzle", "by MossCat", "4.6/5", "6.2k plays"),
        ("Autumn Village", "by BirchLeaf", "4.8/5", "11.3k plays"),
    ];
    for (i, (title, author, rate, plays)) in mods.iter().enumerate() {
        let c = if i == 0 {
            g.card_on(grid, sz(468.0, 424.0), "Creation card")
        } else {
            g.card(grid, sz(468.0, 424.0), "Creation card")
        };
        g.art(c, top(16.0, 224.0, 16.0, 16.0), "Creation preview");
        g.badge(c, tr(28.0, 28.0, 112.0, 40.0), "Rating badge", rate);
        g.heading_l(c, tl(18.0, 256.0, 430.0, 40.0), "Creation title", title, 28.0);
        g.note(c, tl(18.0, 300.0, 430.0, 34.0), "Author line", author, 23.0);
        g.label(c, bl(18.0, 20.0, 220.0, 34.0), "Play count", plays, 22.0);
        g.button2(c, br(16.0, 14.0, 152.0, 64.0), "Play button", "Play");
    }
}

/// Portada de descubrimiento con banner destacado superior y dos carriles horizontales de creaciones.
fn discovery_featured_lanes(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 86.0, 40.0, 40.0), "Discover header");
    g.heading_l(hdr, cl(24.0, 0.0, 260.0, 44.0), "Discover title", "DISCOVER", 36.0);
    let tabs = g.hbox(hdr, cl(300.0, 0.0, 740.0, 68.0), "Discover tabs", 12.0);
    for (i, t) in ["Featured", "Trending", "Library", "Following"].iter().enumerate() {
        if i == 0 {
            g.button(tabs, sz(174.0, 68.0), "Nav tab", t);
        } else {
            g.button2(tabs, sz(174.0, 68.0), "Nav tab", t);
        }
    }
    g.input(hdr, cr(20.0, 0.0, 420.0, 64.0), "Discover search", "Search creations...");

    // Hero destacado
    let hero = g.panel(p, top(122.0, 326.0, 40.0, 40.0), "Spotlight card");
    g.art(hero, left(22.0, 680.0, 22.0, 22.0), "Spotlight art");
    g.badge(hero, tl(730.0, 24.0, 168.0, 42.0), "Pick badge", "STAFF PICK");
    g.heading_l(hero, tl(730.0, 78.0, 1060.0, 52.0), "Spotlight title", "FLOATING GARDENS OF ORIA", 42.0);
    g.note(hero, tl(730.0, 136.0, 1040.0, 68.0), "Spotlight desc", "By MapleCraft  |  4.9 Rating  |  A peaceful puzzle expedition across sky terraces.", 25.0);
    g.button(hero, bl(730.0, 24.0, 240.0, 72.0), "Play now button", "Play now");
    g.button2(hero, bl(990.0, 24.0, 220.0, 70.0), "Bookmark button", "Bookmark");

    // Carril 1: tendencias
    let lane1 = g.panel(p, top(468.0, 286.0, 40.0, 40.0), "Trending lane");
    g.heading_l(lane1, tl(24.0, 16.0, 500.0, 38.0), "Lane 1 title", "TRENDING THIS WEEK", 28.0);
    let r1 = g.hbox(lane1, bottom(18.0, 206.0, 24.0, 24.0), "Trending row", 20.0);
    for (title, sub, rate) in [
        ("Clockwork Tower", "by GearMint", "4.9"),
        ("Mirror Maze", "by GlassFox", "4.7"),
        ("Frost Circuit", "by NorthRun", "4.8"),
        ("Harbor Heist", "by DockNine", "4.8"),
    ] {
        let c = g.card(r1, sz(432.0, 202.0), "Trending card");
        g.art(c, top(12.0, 102.0, 12.0, 12.0), "Card art");
        g.badge(c, tr(20.0, 20.0, 82.0, 40.0), "Rate badge", rate);
        g.heading_l(c, tl(14.0, 122.0, 400.0, 34.0), "Card title", title, 25.0);
        g.note(c, tl(14.0, 158.0, 400.0, 30.0), "Card author", sub, 22.0);
    }

    // Carril 2: nuevos creadores
    let lane2 = g.panel(p, bottom(20.0, 286.0, 40.0, 40.0), "New lane");
    g.heading_l(lane2, tl(24.0, 16.0, 500.0, 38.0), "Lane 2 title", "FRESH FROM CREATORS", 28.0);
    let r2 = g.hbox(lane2, bottom(18.0, 206.0, 24.0, 24.0), "New row", 20.0);
    for (title, sub, tag) in [
        ("Starlight Drift", "by VelaRun", "RACE"),
        ("Tiny Bakery", "by MuffinDev", "COZY"),
        ("Laser Vault", "by RedDot", "PUZZLE"),
        ("Cloud Arena", "by WindUp", "PVP"),
    ] {
        let c = g.card(r2, sz(432.0, 202.0), "New card");
        g.art(c, top(12.0, 102.0, 12.0, 12.0), "Card art");
        g.badge(c, tr(20.0, 20.0, 118.0, 40.0), "Mode badge", tag);
        g.heading_l(c, tl(14.0, 122.0, 400.0, 34.0), "Card title", title, 25.0);
        g.note(c, tl(14.0, 158.0, 400.0, 30.0), "Card author", sub, 22.0);
    }
}

/// Ficha detallada de una creacion con galeria de capturas, datos del autor y boton de descarga.
fn discovery_item_detail(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 88.0, 40.0, 40.0), "Detail header");
    g.button2(hdr, cl(20.0, 0.0, 240.0, 68.0), "Back button", "< Workshop");
    g.label(hdr, cl(286.0, 0.0, 740.0, 38.0), "Breadcrumb", "Maps  /  Parkour  /  Sky Bounce Arena", 26.0);
    g.button2(hdr, cr(20.0, 0.0, 176.0, 68.0), "Share button", "Share");

    // Columna izquierda: galeria y descripcion
    let left_p = g.panel(p, tl(40.0, 128.0, 1184.0, 916.0), "Media panel");
    g.art(left_p, top(24.0, 430.0, 24.0, 24.0), "Main screenshot");
    let thumbs = g.hbox(left_p, tl(24.0, 474.0, 1136.0, 128.0), "Thumbnail strip", 18.0);
    for i in 0..4 {
        let tc_card = if i == 0 {
            g.card_on(thumbs, sz(270.0, 124.0), "Thumb card")
        } else {
            g.card(thumbs, sz(270.0, 124.0), "Thumb card")
        };
        g.art(tc_card, fill(10.0, 10.0, 10.0, 10.0), "Thumb image");
    }
    g.heading_l(left_p, tl(24.0, 624.0, 900.0, 48.0), "Item title", "SKY BOUNCE ARENA", 38.0);
    g.note(left_p, tl(24.0, 680.0, 1130.0, 116.0), "Item description", "Leap across 12 floating trampolines, dodge spinning wind vanes, and race up to 4 friends to the golden bell at the summit.", 26.0);
    let tags = g.hbox(left_p, bl(24.0, 24.0, 760.0, 46.0), "Tag row", 14.0);
    for t in ["PARKOUR", "4 PLAYERS", "FAST RUN"] {
        g.badge(tags, sz(180.0, 44.0), "Tag badge", t);
    }

    // Columna derecha: autor, metricas y descarga
    let right_p = g.panel(p, tr(40.0, 128.0, 632.0, 916.0), "Download panel");
    let creator = g.card(right_p, top(24.0, 120.0, 24.0, 24.0), "Creator card");
    g.icon(creator, cl(18.0, 0.0, 68.0, 68.0), "Creator icon", "PP");
    g.heading_l(creator, tl(104.0, 22.0, 280.0, 38.0), "Creator name", "PixelPaws", 28.0);
    g.note(creator, tl(104.0, 64.0, 280.0, 34.0), "Creator stats", "14 maps published", 22.0);
    g.button2(creator, cr(16.0, 0.0, 152.0, 64.0), "Follow button", "Follow");

    let stats = g.grid(right_p, tl(24.0, 168.0, 584.0, 290.0), "Stats grid", 282.0, 134.0, 20.0);
    for (k, v) in [("RATING", "4.8 / 5"), ("DOWNLOADS", "14.2k"), ("FILE SIZE", "18 MB"), ("VERSION", "v1.4")] {
        let sc = g.card(stats, sz(282.0, 134.0), "Metric card");
        g.note(sc, top(18.0, 32.0, 16.0, 16.0), "Metric name", k, 22.0);
        g.heading_l(sc, bottom(20.0, 46.0, 16.0, 16.0), "Metric val", v, 34.0);
    }

    let acts = g.vbox(right_p, bottom(24.0, 246.0, 24.0, 24.0), "Download actions", 16.0);
    g.button(acts, sz(584.0, 78.0), "Download button", "Download & Play");
    g.button2(acts, sz(584.0, 68.0), "Favorite button", "Add to Favorites");
    g.button2(acts, sz(584.0, 68.0), "Report button", "Report Content");
}

// ===============================================================================================================
// Monetizacion
// ===============================================================================================================

/// Tienda premium con articulo heroe destacado a la izquierda y rejilla de rotacion diaria a la derecha.
fn store_hero_grid(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 92.0, 36.0, 36.0), "Store header");
    g.heading_l(hdr, cl(28.0, 0.0, 440.0, 46.0), "Store title", "ROYAL EMPORIUM", 38.0);
    g.badge(hdr, cr(480.0, 0.0, 210.0, 44.0), "Gold balance", "14,200 GOLD");
    g.badge(hdr, cr(250.0, 0.0, 210.0, 44.0), "Gem balance", "2,450 GEMS");
    g.button(hdr, cr(20.0, 0.0, 210.0, 68.0), "Topup button", "+ Buy Gems");

    // Heroe destacado izquierdo
    let hero = g.card_on(p, tl(36.0, 132.0, 692.0, 912.0), "Featured item card");
    g.art(hero, top(24.0, 490.0, 24.0, 24.0), "Featured art");
    g.badge(hero, tl(40.0, 40.0, 176.0, 44.0), "Featured tag", "FEATURED");
    g.heading_l(hero, tl(28.0, 540.0, 636.0, 50.0), "Hero title", "ASTRAL PALADIN SET", 40.0);
    g.note(hero, tl(28.0, 598.0, 636.0, 72.0), "Hero contents", "Includes full armor, starlit cloak, and matching steed barding.", 25.0);
    let price = g.panel(hero, bl(28.0, 120.0, 636.0, 76.0), "Hero price strip");
    g.label(price, cl(20.0, 0.0, 280.0, 36.0), "Price label", "Bundle price", 25.0);
    g.value(price, cr(20.0, 0.0, 280.0, 44.0), "Price amount", "1,800 Gems", 34.0);
    g.button2(hero, bl(28.0, 24.0, 250.0, 72.0), "Inspect hero button", "Inspect");
    g.button(hero, br(28.0, 24.0, 360.0, 72.0), "Buy hero button", "Purchase");

    // Rejilla derecha de rotacion diaria
    let right_p = g.panel(p, tr(36.0, 132.0, 1132.0, 912.0), "Daily rotation panel");
    g.heading_l(right_p, tl(24.0, 22.0, 520.0, 42.0), "Rotation heading", "DAILY ROTATION", 32.0);
    g.note(right_p, tr(24.0, 26.0, 320.0, 36.0), "Refresh timer", "Refreshes in 14:22:10", 24.0);
    let grid = g.grid(right_p, tl(24.0, 80.0, 1084.0, 808.0), "Items grid", 348.0, 392.0, 20.0);
    let items = [
        ("EPIC", "Ember Greatsword", "800 Gems"),
        ("RARE", "Lantern Staff", "500 Gems"),
        ("EPIC", "Frost Wolf Mount", "1,200 Gems"),
        ("RARE", "Gilded Cape", "450 Gems"),
        ("COMMON", "Tavern Emote", "200 Gems"),
        ("RARE", "Rune Banner", "350 Gems"),
    ];
    for (rar, name, cost) in items {
        let c = g.card(grid, sz(348.0, 392.0), "Store item card");
        g.art(c, top(14.0, 188.0, 14.0, 14.0), "Item art");
        g.badge(c, tl(24.0, 24.0, 132.0, 40.0), "Rarity tag", rar);
        g.heading_l(c, tl(16.0, 218.0, 316.0, 40.0), "Item name", name, 26.0);
        g.note(c, tl(16.0, 262.0, 316.0, 34.0), "Item price", cost, 25.0);
        g.button2(c, bc(0.0, 16.0, 316.0, 64.0), "Buy item button", "Buy");
    }
}

/// Tienda por pestanas de categoria con lista seleccionable a la izquierda y compra a la derecha.
fn store_tabs_list(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 92.0, 40.0, 40.0), "Boutique header");
    g.heading_l(hdr, cl(24.0, 0.0, 300.0, 46.0), "Boutique title", "BOUTIQUE", 38.0);
    let tabs = g.hbox(hdr, cc(-40.0, 0.0, 760.0, 68.0), "Store tabs", 12.0);
    for (i, t) in ["Outfits", "Weapons", "Mounts", "Boosts"].iter().enumerate() {
        if i == 1 {
            g.button(tabs, sz(180.0, 68.0), "Store tab", t);
        } else {
            g.button2(tabs, sz(180.0, 68.0), "Store tab", t);
        }
    }
    g.badge(hdr, cr(24.0, 0.0, 240.0, 44.0), "Credits badge", "3,100 CREDITS");

    // Lista izquierda
    let list_p = g.panel(p, tl(40.0, 132.0, 1084.0, 912.0), "Catalog list panel");
    let rows = g.vbox(list_p, fill(24.0, 24.0, 24.0, 24.0), "Catalog rows", 16.0);
    let catalog = [
        ("Obsidian Blade", "Weapon Skin - 3 Finishes", "MYTHIC", "950 CR"),
        ("Ivory Rapier", "Weapon Skin - Animated Hilt", "EPIC", "700 CR"),
        ("Gilded Longbow", "Weapon Skin - Gold Trail", "EPIC", "750 CR"),
        ("Velvet Scabbard", "Back Accessory", "RARE", "400 CR"),
        ("Noir Edge", "Weapon Skin - Matte Finish", "RARE", "500 CR"),
    ];
    for (i, (name, sub, rar, price)) in catalog.iter().enumerate() {
        let c = if i == 0 {
            g.card_on(rows, sz(1036.0, 158.0), "Catalog row")
        } else {
            g.card(rows, sz(1036.0, 158.0), "Catalog row")
        };
        g.art(c, cl(18.0, 0.0, 132.0, 118.0), "Item thumb");
        g.heading_l(c, tl(172.0, 28.0, 480.0, 42.0), "Item title", name, 30.0);
        g.note(c, tl(172.0, 76.0, 500.0, 36.0), "Item subtitle", sub, 24.0);
        g.badge(c, cr(220.0, 0.0, 144.0, 42.0), "Item rarity", rar);
        g.value(c, cr(24.0, 0.0, 176.0, 42.0), "Item cost", price, 30.0);
    }

    // Panel derecho de vista previa y compra
    let det = g.panel(p, tr(40.0, 132.0, 732.0, 912.0), "Purchase detail");
    g.art(det, top(24.0, 390.0, 24.0, 24.0), "Large preview");
    g.badge(det, tl(40.0, 40.0, 144.0, 42.0), "Selected rarity", "MYTHIC");
    g.heading_l(det, tl(28.0, 440.0, 676.0, 50.0), "Selected name", "OBSIDIAN BLADE", 40.0);
    g.note(det, tl(28.0, 498.0, 676.0, 96.0), "Selected desc", "Forged with smoked crystal inlays. Includes custom draw audio and three metallic color variants.", 25.0);
    let summary = g.card(det, tl(28.0, 616.0, 676.0, 88.0), "Price summary");
    g.label(summary, cl(20.0, 0.0, 300.0, 38.0), "Total label", "Total price", 26.0);
    g.value(summary, cr(20.0, 0.0, 300.0, 44.0), "Total value", "950 Credits", 34.0);
    g.button(det, bc(0.0, 114.0, 676.0, 74.0), "Buy now button", "Buy now - 950 Credits");
    g.button2(det, bc(0.0, 28.0, 676.0, 68.0), "Gift button", "Gift to a friend");
}

/// Ficha de producto a pantalla completa con visor de variantes a la izquierda y desglose a la derecha.
fn store_product_detail(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 88.0, 40.0, 40.0), "Product header");
    g.button2(hdr, cl(20.0, 0.0, 224.0, 68.0), "Back button", "< Store");
    g.label(hdr, cl(276.0, 0.0, 540.0, 38.0), "Collection label", "CYBER COLLECTION  /  SUITS", 26.0);
    g.badge(hdr, cr(24.0, 0.0, 240.0, 44.0), "Wallet badge", "1,650 CREDITS");

    // Escenario izquierdo con variantes
    let stage = g.panel(p, tl(40.0, 128.0, 980.0, 916.0), "Product stage");
    g.art(stage, fill(24.0, 24.0, 24.0, 190.0), "Product model art");
    g.badge(stage, tl(44.0, 44.0, 180.0, 44.0), "View tag", "360 PREVIEW");
    let vars = g.hbox_c(stage, bottom(24.0, 144.0, 24.0, 24.0), "Variant selector", 18.0);
    for (i, (vname, col)) in [("Pulse Pink", g.t.accent), ("Volt Cyan", g.t.accent2), ("Lime Surge", g.t.good)].iter().enumerate() {
        let vc = if i == 0 {
            g.card_on(vars, sz(296.0, 136.0), "Variant card")
        } else {
            g.card(vars, sz(296.0, 136.0), "Variant card")
        };
        g.rect(vc, top(14.0, 56.0, 18.0, 18.0), "Variant swatch", *col, 6.0);
        g.heading(vc, bc(0.0, 16.0, 260.0, 36.0), "Variant name", vname, 24.0);
    }

    // Hoja derecha de compra
    let sheet = g.panel(p, tr(40.0, 128.0, 836.0, 916.0), "Product sheet");
    g.badge(sheet, tl(28.0, 28.0, 152.0, 44.0), "Tier badge", "MYTHIC");
    g.heading_l(sheet, tl(28.0, 88.0, 780.0, 56.0), "Product title", "NEON DRIFTER SUIT", 46.0);
    g.note(sheet, tl(28.0, 154.0, 780.0, 76.0), "Product summary", "Reactive holo-weave suit that pulses with your combo meter and sprint boosts.", 25.0);

    let inc = g.card(sheet, tl(28.0, 250.0, 780.0, 316.0), "Included box");
    g.heading_l(inc, tl(20.0, 18.0, 400.0, 38.0), "Included heading", "INCLUDED IN PACK", 26.0);
    let ilist = g.vbox(inc, fill(20.0, 68.0, 20.0, 20.0), "Included list", 10.0);
    for item in ["+ Neon Drifter Exo-Suit", "+ Reactive Visor Helmet", "+ Light-Trail Glider", "+ 3 Unlockable Colorways"] {
        g.label(ilist, sz(740.0, 48.0), "Included item", item, 25.0);
    }

    let pbox = g.card_on(sheet, tl(28.0, 590.0, 780.0, 108.0), "Price block");
    g.badge(pbox, cl(20.0, 0.0, 138.0, 44.0), "Discount badge", "-25% OFF");
    g.note(pbox, cl(180.0, 0.0, 240.0, 34.0), "Old price", "Was 1,600 CR", 24.0);
    g.value(pbox, cr(24.0, 0.0, 320.0, 50.0), "New price", "1,200 Credits", 38.0);

    g.button(sheet, bc(0.0, 114.0, 780.0, 74.0), "Purchase button", "Purchase now");
    g.button2(sheet, bc(0.0, 28.0, 780.0, 68.0), "Try button", "Test in firing range");
}

/// Tres tarjetas verticales de paquetes promocionales con descuentos y etiqueta de mejor valor.
fn offers_discount_cards(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(24.0, 96.0, 40.0, 40.0), "Offers header");
    g.heading_l(hdr, cl(28.0, 0.0, 480.0, 48.0), "Offers title", "SPECIAL BUNDLES", 40.0);
    g.badge(hdr, cc(0.0, 0.0, 240.0, 44.0), "Promo badge", "UP TO 60% OFF");
    g.button2(hdr, cr(24.0, 0.0, 180.0, 68.0), "Back button", "Back");

    // Tres tarjetas de paquete
    let row = g.hbox_c(p, hmid(48.0, 880.0, 40.0, 40.0), "Bundles row", 28.0);
    let packs = [
        (false, "-35% OFF", "Starter Kit", "500 Coins + Hero Hat + 2 Keys", "Was $7.99", "$4.99"),
        (true, "-60% BEST VALUE", "Mega Hero Pack", "2,400 Coins + 2 Skins + Pet + 10 Keys", "Was $24.99", "$9.99"),
        (false, "-45% OFF", "Builder Bundle", "1,200 Coins + Island Theme + 5 Keys", "Was $14.99", "$7.99"),
    ];
    for (best, tag, title, items, was, price) in packs {
        let c = if best {
            g.card_on(row, sz(580.0, 860.0), "Best bundle card")
        } else {
            g.card(row, sz(580.0, 820.0), "Bundle card")
        };
        g.badge(c, tc(0.0, 22.0, 260.0, 44.0), "Discount badge", tag);
        g.art(c, top(82.0, 280.0, 24.0, 24.0), "Bundle art");
        g.heading(c, tc(0.0, 386.0, 520.0, 50.0), "Bundle name", title, 40.0);
        let inc = g.panel(c, tc(0.0, 456.0, 520.0, 150.0), "Items box");
        g.label(inc, fill(20.0, 22.0, 20.0, 22.0), "Items list", items, 26.0);
        g.note(c, bc(0.0, 168.0, 400.0, 34.0), "Old price", was, 24.0);
        g.heading(c, bc(0.0, 114.0, 400.0, 50.0), "Current price", price, 42.0);
        if best {
            g.button(c, bc(0.0, 26.0, 500.0, 74.0), "Buy bundle button", "Claim best deal");
        } else {
            g.button2(c, bc(0.0, 26.0, 500.0, 70.0), "Buy bundle button", "Buy bundle");
        }
    }
}

/// Oferta modal por tiempo limitado con cuenta atras destacada, contenido incluido y descuento.
fn offers_limited_timer(g: &mut G) {
    let p = g.page();
    g.background();
    let bg_p = g.panel(p, fill(80.0, 80.0, 80.0, 80.0), "Store backdrop");
    g.heading_l(bg_p, tl(36.0, 28.0, 400.0, 44.0), "Backdrop title", "RELIC MARKET", 34.0);
    g.dim(155);

    let modal = g.panel(p, cc(0.0, 0.0, 1380.0, 860.0), "Limited offer modal");
    // Arte izquierdo
    g.art(modal, left(28.0, 540.0, 28.0, 28.0), "Offer key art");
    g.badge(modal, tl(48.0, 48.0, 230.0, 44.0), "Limited tag", "LIMITED OFFER");

    // Detalle derecho con temporizador
    let timer = g.card_on(modal, tr(28.0, 28.0, 756.0, 88.0), "Countdown bar");
    g.label(timer, cl(24.0, 0.0, 320.0, 38.0), "Expires label", "OFFER EXPIRES IN", 26.0);
    g.value(timer, cr(24.0, 0.0, 300.0, 46.0), "Timer readout", "02:18:44", 38.0);

    g.heading_l(modal, tl(596.0, 140.0, 740.0, 54.0), "Offer title", "BLOOD MOON RELIC PACK", 44.0);
    g.note(modal, tl(596.0, 200.0, 740.0, 46.0), "Offer sub", "One-time seasonal bundle available only during the Eclipse.", 24.0);

    let rewards = g.vbox(modal, tl(596.0, 268.0, 756.0, 344.0), "Included rewards", 14.0);
    for (g_txt, name, desc) in [
        ("L", "Cursed Lantern Skin", "Exclusive glowing relic appearance"),
        ("S", "1,500 Blood Shards", "Spend in the Eclipse event shop"),
        ("K", "3x Crypt Keys", "Open high-tier treasure crates"),
    ] {
        let rc = g.card(rewards, sz(756.0, 104.0), "Reward line");
        g.icon(rc, cl(18.0, 0.0, 60.0, 60.0), "Reward icon", g_txt);
        g.heading_l(rc, tl(98.0, 18.0, 620.0, 36.0), "Reward name", name, 26.0);
        g.note(rc, tl(98.0, 58.0, 620.0, 32.0), "Reward desc", desc, 22.0);
    }

    let pcard = g.card(modal, tl(596.0, 636.0, 756.0, 84.0), "Offer price row");
    g.badge(pcard, cl(20.0, 0.0, 146.0, 44.0), "Off badge", "-50% OFF");
    g.note(pcard, cl(188.0, 0.0, 220.0, 34.0), "Old price", "Was $9.99", 24.0);
    g.value(pcard, cr(24.0, 0.0, 240.0, 46.0), "Deal price", "$4.99", 38.0);

    g.button2(modal, bl(596.0, 28.0, 250.0, 70.0), "Dismiss button", "No thanks");
    g.button(modal, br(28.0, 28.0, 480.0, 74.0), "Claim deal button", "Claim offer - $4.99");
}

/// Tabla comparativa de tres ediciones de temporada con filas de beneficios y seleccion directa.
fn offers_pack_compare(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 96.0, 40.0, 40.0), "Editions header");
    g.heading_l(hdr, cl(28.0, 0.0, 520.0, 48.0), "Editions title", "COMPARE SEASON PACKS", 38.0);
    g.note(hdr, cc(120.0, 0.0, 540.0, 36.0), "Editions note", "All editions unlock instantly on your account.", 24.0);
    g.button2(hdr, cr(24.0, 0.0, 176.0, 68.0), "Back button", "Back");

    // Columna de nombres de beneficio a la izquierda + 3 columnas de packs
    let wrap = g.panel(p, fill(40.0, 136.0, 40.0, 28.0), "Comparison table");
    let feat_col = g.card(wrap, left(24.0, 440.0, 180.0, 116.0), "Feature labels column");
    let flist = g.vbox(feat_col, fill(18.0, 20.0, 18.0, 20.0), "Feature names", 16.0);
    for feat in ["Season Pass Access", "Instant Tier Skips", "Bonus Currency", "Exclusive Jersey", "VIP Match Badge"] {
        g.heading_l(flist, sz(404.0, 100.0), "Feature label", feat, 27.0);
    }

    let cols = g.hbox(wrap, fill(488.0, 24.0, 24.0, 24.0), "Tier columns", 20.0);
    let tiers = [
        (false, "STANDARD", "$9.99", ["Included", "-", "500 CR", "-", "-"]),
        (true, "PRO EDITION", "$19.99", ["Included", "+15 Tiers", "1,500 CR", "Included", "-"]),
        (false, "ULTIMATE", "$34.99", ["Included", "+30 Tiers", "3,500 CR", "Included", "Included"]),
    ];
    for (hi, name, price, vals) in tiers {
        let col = if hi {
            g.card_on(cols, sz(424.0, 868.0), "Highlighted tier")
        } else {
            g.card(cols, sz(424.0, 868.0), "Tier column")
        };
        if hi {
            g.badge(col, tc(0.0, 16.0, 210.0, 40.0), "Popular badge", "MOST POPULAR");
        }
        g.heading(col, tc(0.0, 64.0, 380.0, 42.0), "Tier name", name, 32.0);
        g.heading(col, tc(0.0, 110.0, 380.0, 44.0), "Tier price", price, 36.0);

        let vlist = g.vbox(col, tl(18.0, 176.0, 388.0, 564.0), "Tier perks", 16.0);
        for v in vals {
            let cell = g.panel(vlist, sz(388.0, 100.0), "Perk cell");
            g.heading(cell, full(), "Perk value", v, 26.0);
        }
        if hi {
            g.button(col, bc(0.0, 20.0, 360.0, 72.0), "Select tier button", "Choose Pro");
        } else {
            g.button2(col, bc(0.0, 20.0, 360.0, 70.0), "Select tier button", "Select");
        }
    }
}

/// Tienda de moneda en rejilla de seis niveles de paquetes con insignias de bonificacion.
fn currency_bonus_tiers(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 96.0, 48.0, 48.0), "Vault header");
    g.heading_l(hdr, cl(28.0, 0.0, 440.0, 48.0), "Vault title", "TREASURE VAULT", 40.0);
    g.badge(hdr, cr(236.0, 0.0, 240.0, 46.0), "Balance badge", "480 GEMS");
    g.button2(hdr, cr(24.0, 0.0, 184.0, 68.0), "Back button", "Back");

    // Rejilla 3x2 de paquetes de gemas
    let grid_p = g.panel(p, fill(48.0, 136.0, 48.0, 108.0), "Packages panel");
    let grid = g.grid(grid_p, fill(28.0, 24.0, 28.0, 24.0), "Gem packs grid", 572.0, 376.0, 22.0);
    let packs = [
        (false, "STARTER", "100 Gems", "No bonus", "$0.99"),
        (false, "+10% BONUS", "550 Gems", "500 + 50 Bonus", "$4.99"),
        (true, "+20% POPULAR", "1,440 Gems", "1,200 + 240 Bonus", "$9.99"),
        (false, "+30% BONUS", "3,250 Gems", "2,500 + 750 Bonus", "$19.99"),
        (false, "+40% BONUS", "7,000 Gems", "5,000 + 2,000 Bonus", "$39.99"),
        (false, "+50% BEST", "15,000 Gems", "10,000 + 5,000 Bonus", "$79.99"),
    ];
    for (hi, tag, amt, sub, price) in packs {
        let c = if hi {
            g.card_on(grid, sz(572.0, 376.0), "Featured pack")
        } else {
            g.card(grid, sz(572.0, 376.0), "Gem pack")
        };
        g.badge(c, tr(20.0, 18.0, 200.0, 42.0), "Bonus tag", tag);
        g.icon(c, tc(0.0, 56.0, 108.0, 108.0), "Gem icon", "G");
        g.heading(c, tc(0.0, 182.0, 520.0, 48.0), "Gem amount", amt, 38.0);
        g.note(c, tc(0.0, 236.0, 520.0, 36.0), "Bonus detail", sub, 24.0);
        if hi {
            g.button(c, bc(0.0, 20.0, 480.0, 72.0), "Buy gems button", price);
        } else {
            g.button2(c, bc(0.0, 20.0, 480.0, 70.0), "Buy gems button", price);
        }
    }

    let foot = g.panel(p, bottom(20.0, 72.0, 48.0, 48.0), "Vault notice");
    g.heading(foot, full(), "First buy note", "First purchase on any tier grants double bonus gems.", 25.0);
}

/// Tienda de creditos en lista vertical centrada con filas limpias y boton de compra por fila.
fn currency_vertical_list(g: &mut G) {
    let p = g.page();
    g.background();
    let frame = g.panel(p, cc(0.0, 0.0, 1320.0, 960.0), "Credits exchange panel");
    g.heading_l(frame, tl(32.0, 28.0, 520.0, 48.0), "Exchange title", "CREDITS EXCHANGE", 40.0);
    g.badge(frame, tr(264.0, 30.0, 224.0, 44.0), "Balance badge", "1,240 CR");
    g.button2(frame, tr(32.0, 20.0, 212.0, 68.0), "Redeem button", "Redeem code");
    g.rect(frame, top(108.0, 2.0, 32.0, 32.0), "Top divider", g.t.line, 0.0);

    let list = g.vbox(frame, fill(32.0, 130.0, 32.0, 92.0), "Credit tiers", 16.0);
    let tiers = [
        (false, "C1", "Micro Pack - 200 Credits", "Standard rate", "STANDARD", "$1.99"),
        (false, "C2", "Starter Pack - 600 Credits", "Includes +50 bonus credits", "+10% BONUS", "$4.99"),
        (true, "C3", "Commander Pack - 1,400 Credits", "Includes +250 bonus credits", "MOST POPULAR", "$9.99"),
        (false, "C4", "Fleet Pack - 3,100 Credits", "Includes +700 bonus credits", "+25% BONUS", "$19.99"),
        (false, "C5", "Orbital Reserve - 7,500 Credits", "Includes +2,500 bonus credits", "+40% BONUS", "$49.99"),
    ];
    for (hi, code, name, desc, tag, price) in tiers {
        let r = if hi {
            g.card_on(list, sz(1256.0, 134.0), "Popular tier row")
        } else {
            g.card(list, sz(1256.0, 134.0), "Tier row")
        };
        g.icon(r, cl(22.0, 0.0, 72.0, 72.0), "Credit icon", code);
        g.heading_l(r, tl(118.0, 24.0, 580.0, 40.0), "Tier title", name, 30.0);
        g.note(r, tl(118.0, 72.0, 580.0, 36.0), "Tier bonus", desc, 24.0);
        g.badge(r, cr(264.0, 0.0, 216.0, 44.0), "Tier badge", tag);
        if hi {
            g.button(r, cr(24.0, 0.0, 212.0, 72.0), "Purchase tier button", price);
        } else {
            g.button2(r, cr(24.0, 0.0, 212.0, 70.0), "Purchase tier button", price);
        }
    }

    g.note(frame, bottom(22.0, 40.0, 32.0, 32.0), "Delivery note", "Credits are delivered immediately and shared across linked platforms.", 23.0);
}

/// Tienda de moneda con la mejor oferta destacada en grande a la izquierda y cuatro packs a la derecha.
fn currency_best_offer(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(24.0, 92.0, 40.0, 40.0), "Currency header");
    g.heading_l(hdr, cl(28.0, 0.0, 440.0, 48.0), "Shop title", "STAR CRYSTALS", 40.0);
    g.badge(hdr, cr(232.0, 0.0, 224.0, 44.0), "Stars balance", "320 STARS");
    g.button2(hdr, cr(24.0, 0.0, 180.0, 68.0), "Back button", "Back");

    // Gran tarjeta destacada izquierda
    let hero = g.card_on(p, tl(40.0, 140.0, 700.0, 904.0), "Best value panel");
    g.badge(hero, tc(0.0, 28.0, 280.0, 46.0), "Best tag", "BEST VALUE +50%");
    g.art(hero, top(94.0, 340.0, 32.0, 32.0), "Crystal hoard art");
    g.icon(hero, tc(0.0, 190.0, 148.0, 148.0), "Star emblem", "*");
    g.heading(hero, tc(0.0, 462.0, 640.0, 56.0), "Hero pack title", "9,750 STAR CRYSTALS", 44.0);
    g.note(hero, tc(0.0, 526.0, 620.0, 40.0), "Hero pack sub", "6,500 Base + 3,250 Free Bonus", 26.0);
    let perks = g.panel(hero, tc(0.0, 592.0, 620.0, 144.0), "Bonus perks box");
    g.label(perks, tl(24.0, 24.0, 570.0, 40.0), "Perk 1", "+ Includes Exclusive Ribbon Avatar Frame", 25.0);
    g.label(perks, tl(24.0, 76.0, 570.0, 40.0), "Perk 2", "+ Instant VIP support and gift token", 25.0);
    g.button(hero, bc(0.0, 32.0, 620.0, 80.0), "Buy best button", "Purchase - $49.99");

    // Rejilla 2x2 derecha
    let right_p = g.panel(p, tr(40.0, 140.0, 1112.0, 904.0), "Standard packs panel");
    let grid = g.grid(right_p, fill(28.0, 28.0, 28.0, 28.0), "Standard grid", 516.0, 412.0, 24.0);
    let stds = [
        ("SMALL", "300 Stars", "Base pouch", "$2.99"),
        ("+15% BONUS", "1,150 Stars", "1,000 + 150 Bonus", "$9.99"),
        ("+25% BONUS", "2,500 Stars", "2,000 + 500 Bonus", "$19.99"),
        ("+35% BONUS", "4,725 Stars", "3,500 + 1,225 Bonus", "$34.99"),
    ];
    for (tag, amt, note_txt, price) in stds {
        let c = g.card(grid, sz(516.0, 412.0), "Standard pack");
        g.badge(c, tr(20.0, 18.0, 184.0, 42.0), "Pack tag", tag);
        g.icon(c, tc(0.0, 68.0, 100.0, 100.0), "Crystal icon", "*");
        g.heading(c, tc(0.0, 190.0, 460.0, 46.0), "Pack amount", amt, 36.0);
        g.note(c, tc(0.0, 244.0, 460.0, 36.0), "Pack detail", note_txt, 24.0);
        g.button2(c, bc(0.0, 24.0, 430.0, 70.0), "Buy pack button", price);
    }
}

/// Calendario modal de 7 dias con racha actual, tarjetas diarias y premio mayor del dia 7.
fn daily_7_days(g: &mut G) {
    let p = g.page();
    g.background();
    let back = g.panel(p, fill(64.0, 64.0, 64.0, 64.0), "Lobby backdrop");
    g.heading_l(back, tl(36.0, 28.0, 400.0, 44.0), "Lobby title", "CAMP HUB", 34.0);
    g.dim(140);

    let modal = g.panel(p, cc(0.0, 0.0, 1560.0, 880.0), "Daily checkin modal");
    g.heading_l(modal, tl(36.0, 28.0, 520.0, 50.0), "Checkin title", "DAILY REWARDS", 42.0);
    g.badge(modal, tc(0.0, 32.0, 236.0, 46.0), "Streak badge", "DAY 4 STREAK");
    g.button2(modal, tr(36.0, 22.0, 168.0, 68.0), "Close button", "Close");

    // Dias 1 a 6 en fila
    let days = g.hbox_c(modal, top(116.0, 370.0, 36.0, 36.0), "Week row", 18.0);
    let data = [
        ("DAY 1", "G", "200 Gold", "Claimed", false),
        ("DAY 2", "E", "2x Elixir", "Claimed", false),
        ("DAY 3", "K", "1x Key", "Claimed", false),
        ("DAY 4", "D", "150 Gems", "TODAY", true),
        ("DAY 5", "B", "XP Boost", "Locked", false),
        ("DAY 6", "G", "600 Gold", "Locked", false),
    ];
    for (day, gl, reward, status, today) in data {
        let c = if today {
            g.card_on(days, sz(232.0, 360.0), "Today card")
        } else {
            g.card(days, sz(232.0, 350.0), "Day card")
        };
        g.badge(c, tc(0.0, 18.0, 136.0, 40.0), "Day badge", day);
        g.icon(c, tc(0.0, 88.0, 100.0, 100.0), "Reward icon", gl);
        g.heading(c, tc(0.0, 212.0, 210.0, 40.0), "Reward title", reward, 26.0);
        g.note(c, bc(0.0, 24.0, 190.0, 34.0), "Status note", status, 23.0);
    }

    // Tarjeta especial del dia 7
    let d7 = g.card_on(modal, bottom(124.0, 220.0, 36.0, 36.0), "Day 7 grand prize");
    g.art(d7, left(22.0, 280.0, 22.0, 22.0), "Grand prize art");
    g.badge(d7, tl(328.0, 24.0, 240.0, 44.0), "Day 7 badge", "DAY 7 GRAND PRIZE");
    g.heading_l(d7, tl(328.0, 82.0, 740.0, 48.0), "Grand prize title", "FOREST SPIRIT COMPANION + 500 GEMS", 36.0);
    g.note(d7, tl(328.0, 142.0, 620.0, 36.0), "Grand prize note", "Log in 3 more consecutive days to unlock.", 24.0);
    g.bar(d7, cr(28.0, 20.0, 320.0, 28.0), "Week progress", 4.0 / 7.0, g.t.good);
    g.value(d7, cr(28.0, -24.0, 240.0, 36.0), "Week counter", "4 / 7 Days", 26.0);

    g.button(modal, bc(0.0, 28.0, 460.0, 74.0), "Claim daily button", "Claim Day 4 Reward");
}

/// Calendario mensual de 28 dias en rejilla 7x4 junto a una barra lateral de hitos acumulados.
fn daily_28_calendar(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 92.0, 36.0, 36.0), "Calendar header");
    g.heading_l(hdr, cl(28.0, 0.0, 560.0, 48.0), "Month title", "MONTHLY CHECK-IN - NOVEMBER", 36.0);
    g.badge(hdr, cc(140.0, 0.0, 220.0, 44.0), "Month progress", "12 / 28 DAYS");
    g.button(hdr, cr(24.0, 0.0, 240.0, 68.0), "Claim today button", "Claim Day 12");

    // Rejilla 7x4 de 28 dias (dos elementos por celda para mantener conteo compacto)
    let cal = g.panel(p, tl(36.0, 132.0, 1344.0, 912.0), "Month calendar panel");
    let grid = g.grid(cal, fill(24.0, 24.0, 24.0, 24.0), "Days grid", 172.0, 202.0, 14.0);
    let rewards = [
        "50 G", "1x K", "100 G", "Boost", "150 G", "2x K", "50 Gem",
        "80 G", "Card", "120 G", "Boost", "100 Gem", "200 G", "Chest",
        "100 G", "1x K", "180 G", "Boost", "220 G", "2x K", "150 Gem",
        "120 G", "Card", "250 G", "Boost", "300 G", "3x K", "MYTHIC",
    ];
    for (i, rew) in rewards.iter().enumerate() {
        let day = i + 1;
        let c = if day == 12 {
            g.card_on(grid, sz(172.0, 202.0), "Today cell")
        } else {
            g.card(grid, sz(172.0, 202.0), "Day cell")
        };
        let status = if day < 12 {
            format!("D{} OK", day)
        } else if day == 12 {
            format!("DAY {}!", day)
        } else {
            format!("Day {}", day)
        };
        g.heading(c, tc(0.0, 22.0, 156.0, 40.0), "Day number", &status, 24.0);
        g.note(c, bc(0.0, 28.0, 156.0, 42.0), "Day reward", rew, 25.0);
    }

    // Barra lateral derecha de hitos
    let side = g.panel(p, tr(36.0, 132.0, 480.0, 912.0), "Milestones sidebar");
    g.heading_l(side, tl(24.0, 24.0, 430.0, 44.0), "Milestones title", "MONTHLY MILESTONES", 30.0);
    g.bar(side, tl(24.0, 82.0, 432.0, 24.0), "Monthly bar", 12.0 / 28.0, g.t.accent);
    g.note(side, tl(24.0, 114.0, 432.0, 34.0), "Checkin count", "12 check-ins completed this month", 22.0);

    let mlist = g.vbox(side, tl(24.0, 174.0, 432.0, 560.0), "Milestone cards", 18.0);
    for (done, title, rew, tag) in [
        (true, "7 Days Bonus", "250 Star Coins", "CLAIMED"),
        (false, "14 Days Bonus", "Pastel Ribbon Cape", "2 TO GO"),
        (false, "28 Days Bonus", "Mythic Wish Box", "16 TO GO"),
    ] {
        let mc = if done {
            g.card(mlist, sz(432.0, 168.0), "Milestone card")
        } else {
            g.card_on(mlist, sz(432.0, 168.0), "Milestone card")
        };
        g.badge(mc, tr(16.0, 16.0, 146.0, 40.0), "Milestone state", tag);
        g.heading_l(mc, tl(18.0, 20.0, 250.0, 38.0), "Milestone name", title, 26.0);
        g.label(mc, tl(18.0, 82.0, 396.0, 38.0), "Milestone prize", rew, 25.0);
    }
    g.note(side, bl(24.0, 28.0, 432.0, 36.0), "Reset timer", "Next daily reset in 09:42:10", 23.0);
}

/// Ruleta diaria de fortuna a la izquierda combinada con tres cofres de actividad a la derecha.
fn daily_wheel_chests(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 92.0, 40.0, 40.0), "Fortune header");
    g.heading_l(hdr, cl(28.0, 0.0, 420.0, 48.0), "Fortune title", "DAILY FORTUNE", 40.0);
    g.badge(hdr, cc(0.0, 0.0, 224.0, 44.0), "Spins badge", "1 FREE SPIN");
    g.button2(hdr, cr(24.0, 0.0, 176.0, 68.0), "Back button", "Back");

    // Izquierda: ruleta de premios
    let wheel_p = g.panel(p, tl(40.0, 132.0, 916.0, 912.0), "Prize wheel panel");
    g.heading_l(wheel_p, tl(28.0, 24.0, 500.0, 44.0), "Wheel heading", "LUCKY PRIZE WHEEL", 32.0);
    g.circle(wheel_p, cc(0.0, -34.0, 540.0, 540.0), "Wheel outer ring", g.t.panel2);
    g.circle(wheel_p, cc(0.0, -34.0, 460.0, 460.0), "Wheel inner ring", alpha(g.t.line, 90));
    g.badge(wheel_p, tc(0.0, 92.0, 148.0, 44.0), "Wheel pointer", "WIN !");

    // Cuatro tarjetas de premios dispuestas en cruz alrededor de la ruleta
    let prizes = [
        (cc(0.0, -206.0, 240.0, 96.0), "500 Gold", true),
        (cc(220.0, -34.0, 240.0, 96.0), "100 Gems", false),
        (cc(0.0, 138.0, 240.0, 96.0), "2x XP Card", false),
        (cc(-220.0, -34.0, 240.0, 96.0), "Arcade Key", false),
    ];
    for (pos, label, top_hit) in prizes {
        let pc = if top_hit {
            g.card_on(wheel_p, pos, "Prize slice")
        } else {
            g.card(wheel_p, pos, "Prize slice")
        };
        g.heading(pc, full(), "Prize text", label, 26.0);
    }
    g.icon(wheel_p, cc(0.0, -34.0, 112.0, 112.0), "Wheel hub", "GO");
    g.button(wheel_p, bc(0.0, 28.0, 460.0, 76.0), "Spin button", "SPIN WHEEL (FREE)");

    // Derecha: cofres diarios por partidas
    let chests_p = g.panel(p, tr(40.0, 132.0, 896.0, 912.0), "Daily chests panel");
    g.heading_l(chests_p, tl(28.0, 24.0, 620.0, 44.0), "Chests heading", "DAILY ACTIVITY CHESTS", 32.0);
    g.note(chests_p, tl(28.0, 74.0, 760.0, 36.0), "Chests subtitle", "Play matches today to unlock up to three bonus chests.", 24.0);

    let clist = g.vbox(chests_p, fill(28.0, 132.0, 28.0, 28.0), "Chest list", 22.0);
    let chests = [
        (true, "B", "Bronze Chest", "Win 1 match (1 / 1)", 1.0, "Open now"),
        (false, "S", "Silver Chest", "Win 3 matches (2 / 3)", 0.66, "2 / 3"),
        (false, "G", "Gold Chest", "Win 5 matches (2 / 5)", 0.40, "2 / 5"),
    ];
    for (ready, gl, name, req, prog, btn) in chests {
        let cc_card = if ready {
            g.card_on(clist, sz(840.0, 232.0), "Ready chest")
        } else {
            g.card(clist, sz(840.0, 232.0), "Locked chest")
        };
        g.icon(cc_card, cl(28.0, 0.0, 108.0, 108.0), "Chest icon", gl);
        g.heading_l(cc_card, tl(164.0, 34.0, 420.0, 42.0), "Chest name", name, 32.0);
        g.note(cc_card, tl(164.0, 84.0, 420.0, 36.0), "Chest requirement", req, 24.0);
        g.bar(cc_card, tl(164.0, 146.0, 400.0, 26.0), "Chest progress", prog, if ready { g.t.good } else { g.t.accent });
        if ready {
            g.button(cc_card, cr(24.0, 0.0, 210.0, 74.0), "Open chest button", btn);
        } else {
            g.button2(cc_card, cr(24.0, 0.0, 210.0, 70.0), "Chest status button", btn);
        }
    }
}

/// Pista horizontal de pase de temporada con filas paralelas gratuita y premium por nivel.
fn season_pass_track(g: &mut G) {
    let p = g.page();
    g.background();
    // Cabecera de temporada y barra de XP
    let hdr = g.panel(p, top(20.0, 108.0, 36.0, 36.0), "Season header");
    g.heading_l(hdr, tl(24.0, 16.0, 560.0, 44.0), "Season title", "SEASON 4: IRON FRONT", 36.0);
    g.note(hdr, tl(24.0, 64.0, 460.0, 32.0), "Season timer", "Ends in 28 days  |  Tier 18 / 50", 23.0);
    g.bar(hdr, cc(60.0, 14.0, 440.0, 24.0), "Tier XP bar", 0.64, g.t.accent);
    g.label(hdr, cc(60.0, -22.0, 440.0, 32.0), "Tier XP text", "6,400 / 10,000 XP to Tier 19", 23.0);
    g.button(hdr, cr(24.0, 0.0, 320.0, 72.0), "Upgrade pass button", "Unlock Premium Pass");

    // Pista central con columna de etiquetas + 5 niveles visibles
    let track = g.panel(p, fill(36.0, 148.0, 36.0, 144.0), "Pass track panel");
    let labels = g.vbox(track, left(20.0, 210.0, 96.0, 20.0), "Track labels", 24.0);
    let free_lbl = g.card(labels, sz(210.0, 310.0), "Free track plate");
    g.heading(free_lbl, full(), "Free heading", "FREE\nTRACK", 30.0);
    let prem_lbl = g.card_on(labels, sz(210.0, 310.0), "Premium track plate");
    g.heading(prem_lbl, full(), "Premium heading", "PREMIUM\nTRACK", 30.0);

    let cols = g.hbox(track, fill(250.0, 20.0, 20.0, 20.0), "Tier columns", 18.0);
    let tiers = [
        ("TIER 17", "200 Credits", "Tactical Visor", false),
        ("TIER 18", "XP Boost", "Desert Camo", true),
        ("TIER 19", "Emblem", "Rifle Skin", false),
        ("TIER 20", "300 Credits", "Commander Suit", false),
        ("TIER 21", "Supply Key", "500 Credits", false),
    ];
    for (tname, free_r, prem_r, current) in tiers {
        let col = g.panel(cols, sz(298.0, 744.0), "Tier column");
        g.badge(col, tc(0.0, 16.0, 180.0, 44.0), "Tier badge", tname);
        let fc = if current {
            g.card_on(col, tc(0.0, 76.0, 270.0, 310.0), "Free reward")
        } else {
            g.card(col, tc(0.0, 76.0, 270.0, 310.0), "Free reward")
        };
        g.art(fc, top(16.0, 160.0, 16.0, 16.0), "Free art");
        g.heading(fc, bc(0.0, 62.0, 240.0, 38.0), "Free reward name", free_r, 25.0);
        g.note(fc, bc(0.0, 18.0, 220.0, 32.0), "Free state", if current { "Ready" } else { "Tier reward" }, 22.0);

        let pc = if current {
            g.card_on(col, bc(0.0, 20.0, 270.0, 310.0), "Premium reward")
        } else {
            g.card(col, bc(0.0, 20.0, 270.0, 310.0), "Premium reward")
        };
        g.art(pc, top(16.0, 160.0, 16.0, 16.0), "Premium art");
        g.heading(pc, bc(0.0, 62.0, 240.0, 38.0), "Premium reward name", prem_r, 25.0);
        g.note(pc, bc(0.0, 18.0, 220.0, 32.0), "Premium state", "Premium", 22.0);
    }

    // Controles inferiores
    let foot = g.panel(p, bottom(20.0, 104.0, 36.0, 36.0), "Track footer");
    let nav = g.hbox(foot, cl(24.0, 0.0, 440.0, 68.0), "Page buttons", 14.0);
    g.button2(nav, sz(208.0, 68.0), "Prev tiers button", "< Prev 5");
    g.button2(nav, sz(208.0, 68.0), "Next tiers button", "Next 5 >");
    g.button2(foot, cr(332.0, 0.0, 280.0, 68.0), "Buy tier button", "Buy 1 Tier (150 CR)");
    g.button(foot, cr(24.0, 0.0, 288.0, 72.0), "Claim all button", "Claim unlocked (3)");
}

/// Pantalla de evento temporal con banner superior, misiones activas a la izquierda e hitos a la derecha.
fn season_event_missions(g: &mut G) {
    let p = g.page();
    g.background();
    // Banner de evento
    let hdr = g.panel(p, top(20.0, 168.0, 40.0, 40.0), "Event banner");
    g.art(hdr, left(20.0, 380.0, 20.0, 20.0), "Event artwork");
    g.badge(hdr, tl(428.0, 22.0, 240.0, 42.0), "Countdown badge", "ENDS IN 6D 14H");
    g.heading_l(hdr, tl(428.0, 74.0, 820.0, 52.0), "Event title", "ECLIPSE HARVEST EVENT", 42.0);
    g.badge(hdr, cr(224.0, 0.0, 220.0, 46.0), "Event currency", "340 EMBERS");
    g.button2(hdr, cr(24.0, 0.0, 176.0, 68.0), "Back button", "Back");

    // Izquierda: misiones del evento
    let left_p = g.panel(p, tl(40.0, 208.0, 1068.0, 836.0), "Event missions panel");
    g.heading_l(left_p, tl(24.0, 22.0, 520.0, 44.0), "Missions heading", "EVENT MISSIONS", 32.0);
    let mlist = g.vbox(left_p, fill(24.0, 84.0, 24.0, 24.0), "Mission list", 16.0);
    let missions = [
        (true, "Light 5 Moon Braziers", "+100 Embers", 1.0, "5 / 5"),
        (false, "Defeat 12 Dusk Stalkers", "+120 Embers", 0.75, "9 / 12"),
        (false, "Survive 3 Night Waves", "+150 Embers", 0.33, "1 / 3"),
        (false, "Play 4 Co-op Hunts", "+180 Embers", 0.50, "2 / 4"),
    ];
    for (done, title, rew, prog, cnt) in missions {
        let mc = if done {
            g.card_on(mlist, sz(1020.0, 168.0), "Completed mission")
        } else {
            g.card(mlist, sz(1020.0, 168.0), "Active mission")
        };
        g.heading_l(mc, tl(22.0, 22.0, 620.0, 40.0), "Mission title", title, 28.0);
        g.note(mc, tl(22.0, 68.0, 420.0, 34.0), "Mission reward", rew, 24.0);
        g.bar(mc, tl(22.0, 118.0, 680.0, 24.0), "Mission bar", prog, if done { g.t.good } else { g.t.accent });
        if done {
            g.button(mc, cr(22.0, 0.0, 220.0, 72.0), "Claim mission button", "Claim");
        } else {
            g.value(mc, cr(26.0, 0.0, 180.0, 42.0), "Mission count", cnt, 30.0);
        }
    }

    // Derecha: hitos de recompensas del evento
    let right_p = g.panel(p, tr(40.0, 208.0, 748.0, 836.0), "Event milestones panel");
    g.heading_l(right_p, tl(24.0, 22.0, 520.0, 44.0), "Milestones heading", "MILESTONE REWARDS", 32.0);
    g.bar(right_p, tl(24.0, 82.0, 700.0, 26.0), "Total embers bar", 0.34, g.t.accent);
    g.note(right_p, tl(24.0, 116.0, 520.0, 34.0), "Embers total", "340 / 1,000 Embers collected", 23.0);

    let rlist = g.vbox(right_p, tl(24.0, 172.0, 700.0, 540.0), "Milestone list", 14.0);
    for (reached, pts, prize) in [
        (true, "250", "Ashen Sigil Banner"),
        (false, "500", "2x Crypt Keys"),
        (false, "750", "Hollow Mask Hood"),
        (false, "1,000", "Eclipse Scythe Weapon"),
    ] {
        let rc = if reached {
            g.card_on(rlist, sz(700.0, 122.0), "Reached milestone")
        } else {
            g.card(rlist, sz(700.0, 122.0), "Pending milestone")
        };
        g.badge(rc, cl(18.0, 0.0, 130.0, 44.0), "Points badge", pts);
        g.heading_l(rc, cl(168.0, 0.0, 360.0, 40.0), "Prize title", prize, 26.0);
        g.note(rc, cr(20.0, 0.0, 140.0, 34.0), "Prize state", if reached { "Unlocked" } else { "Locked" }, 22.0);
    }
    g.button(right_p, bc(0.0, 24.0, 700.0, 74.0), "Event shop button", "Open Event Shop");
}

/// Progreso de temporada por capitulos narrativos con lista a la izquierda y tareas del capitulo a la derecha.
fn season_chapter_progress(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 92.0, 40.0, 40.0), "Chronicle header");
    g.heading_l(hdr, cl(28.0, 0.0, 560.0, 48.0), "Chronicle title", "CHRONICLE PASS - ACT II", 38.0);
    g.badge(hdr, cc(140.0, 0.0, 236.0, 44.0), "Completion badge", "68% COMPLETE");
    g.button2(hdr, cr(24.0, 0.0, 236.0, 68.0), "Rewards button", "All rewards");

    // Izquierda: lista de capitulos
    let ch_p = g.panel(p, tl(40.0, 132.0, 620.0, 912.0), "Chapters list panel");
    g.heading_l(ch_p, tl(24.0, 22.0, 420.0, 42.0), "Chapters heading", "STORY CHAPTERS", 30.0);
    let clist = g.vbox(ch_p, fill(24.0, 84.0, 24.0, 24.0), "Chapter items", 18.0);
    let chapters = [
        ("CH 1", "The Awakening", 1.0, "Complete", false),
        ("CH 2", "The Crossing", 0.75, "Active - 3/4", true),
        ("CH 3", "High Spire", 0.30, "In progress", false),
        ("CH 4", "Crown Finale", 0.0, "Unlocks next week", false),
    ];
    for (num, name, prog, st, sel) in chapters {
        let c = if sel {
            g.card_on(clist, sz(572.0, 184.0), "Active chapter")
        } else {
            g.card(clist, sz(572.0, 184.0), "Chapter card")
        };
        g.badge(c, tl(18.0, 18.0, 112.0, 40.0), "Chapter tag", num);
        g.heading_l(c, tl(148.0, 18.0, 400.0, 40.0), "Chapter name", name, 28.0);
        g.note(c, tl(18.0, 78.0, 420.0, 34.0), "Chapter state", st, 23.0);
        g.bar(c, bottom(22.0, 24.0, 18.0, 18.0), "Chapter bar", prog, if prog >= 1.0 { g.t.good } else { g.t.accent });
    }

    // Derecha: detalle del capitulo seleccionado
    let det = g.panel(p, tr(40.0, 132.0, 1196.0, 912.0), "Chapter detail panel");
    g.art(det, top(24.0, 270.0, 24.0, 24.0), "Chapter banner");
    g.badge(det, tl(44.0, 44.0, 196.0, 42.0), "Active tag", "CHAPTER 2");
    g.heading_l(det, tl(28.0, 316.0, 820.0, 50.0), "Active title", "THE CROSSING", 40.0);
    g.note(det, tl(28.0, 372.0, 1120.0, 62.0), "Active story", "Complete all chapter trials to advance the chronicle and earn the Gilded Compass cloak.", 25.0);

    let objs = g.hbox(det, tl(28.0, 456.0, 1140.0, 250.0), "Chapter tasks", 20.0);
    for (title, sub, prog) in [
        ("Bridge Skirmish", "Complete 3 matches (3/3)", 1.0),
        ("Relic Hunt", "Find 6 tokens (5/6)", 0.83),
        ("Clean Escape", "Finish under 8 min (1/1)", 1.0),
    ] {
        let oc = g.card(objs, sz(366.0, 242.0), "Task card");
        g.heading_l(oc, tl(18.0, 20.0, 330.0, 38.0), "Task name", title, 26.0);
        g.note(oc, tl(18.0, 68.0, 330.0, 68.0), "Task goal", sub, 23.0);
        g.bar(oc, bottom(26.0, 24.0, 18.0, 18.0), "Task bar", prog, if prog >= 1.0 { g.t.good } else { g.t.accent });
    }

    let rew = g.card_on(det, bottom(24.0, 152.0, 28.0, 28.0), "Chapter reward box");
    g.icon(rew, cl(24.0, 0.0, 84.0, 84.0), "Bonus icon", "C");
    g.heading_l(rew, tl(132.0, 34.0, 580.0, 40.0), "Bonus title", "Chapter Bonus: Gilded Compass Cloak", 28.0);
    g.note(rew, tl(132.0, 82.0, 580.0, 36.0), "Bonus progress", "Finish 1 more task to claim chapter bonus", 24.0);
    g.button(rew, cr(24.0, 0.0, 310.0, 74.0), "Track task button", "Play Relic Hunt");
}

/// Banner de invocacion con botones 1x y 10x, selector lateral de banners y tabla de probabilidades.
fn gacha_banner_odds(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 92.0, 36.0, 36.0), "Summon header");
    g.heading_l(hdr, cl(28.0, 0.0, 460.0, 48.0), "Summon title", "SUMMON SANCTUARY", 38.0);
    g.badge(hdr, cr(480.0, 0.0, 210.0, 44.0), "Ticket count", "14 TICKETS");
    g.badge(hdr, cr(244.0, 0.0, 216.0, 44.0), "Gems count", "2,400 GEMS");
    g.button2(hdr, cr(20.0, 0.0, 200.0, 68.0), "History button", "History");

    // Izquierda: selector de banners y probabilidades
    let side = g.panel(p, tl(36.0, 132.0, 400.0, 912.0), "Banner list panel");
    let blist = g.vbox(side, tl(20.0, 20.0, 360.0, 460.0), "Banners", 16.0);
    for (i, (bname, sub)) in [
        ("Solar Empress", "Rate Up - 9d left"),
        ("Ancient Relics", "Weapon Pool"),
        ("Standard Pool", "Always open"),
    ]
    .iter()
    .enumerate()
    {
        let c = if i == 0 {
            g.card_on(blist, sz(360.0, 136.0), "Selected banner")
        } else {
            g.card(blist, sz(360.0, 136.0), "Banner option")
        };
        g.heading_l(c, tl(18.0, 24.0, 320.0, 40.0), "Banner name", bname, 28.0);
        g.note(c, tl(18.0, 74.0, 320.0, 34.0), "Banner info", sub, 23.0);
    }

    let odds = g.card(side, bottom(20.0, 396.0, 20.0, 20.0), "Drop rates card");
    g.heading_l(odds, tl(18.0, 18.0, 320.0, 38.0), "Odds heading", "DROP RATES & PITY", 26.0);
    let olist = g.vbox(odds, tl(18.0, 68.0, 324.0, 224.0), "Rate lines", 12.0);
    for line in ["5-Star Hero:  2.0%", "4-Star Hero:  12.0%", "3-Star Gear:  86.0%", "Guaranteed 5-Star at 80"] {
        g.label(olist, sz(324.0, 46.0), "Rate row", line, 23.0);
    }
    g.label(odds, bl(18.0, 56.0, 324.0, 34.0), "Pity text", "Pity counter: 42 / 80", 23.0);
    g.bar(odds, bottom(20.0, 24.0, 18.0, 18.0), "Pity bar", 42.0 / 80.0, g.t.warn);

    // Derecha: escenario principal del banner y botones 1x/10x
    let stage = g.panel(p, tr(36.0, 132.0, 1424.0, 912.0), "Banner showcase");
    g.art(stage, fill(24.0, 24.0, 24.0, 220.0), "Hero banner art");
    g.badge(stage, tl(48.0, 48.0, 210.0, 46.0), "Rateup badge", "5-STAR RATE UP");
    let info = g.panel(stage, bl(48.0, 248.0, 680.0, 176.0), "Banner title plate");
    g.heading_l(info, tl(24.0, 20.0, 630.0, 54.0), "Hero banner title", "SOLAR EMPRESS AURELIA", 42.0);
    g.note(info, tl(24.0, 86.0, 630.0, 68.0), "Hero banner desc", "Featured 5-star caster with boosted 50% rate on any 5-star summon.", 24.0);

    let dock = g.hbox_c(stage, bottom(28.0, 148.0, 24.0, 24.0), "Summon actions", 28.0);
    let b1 = g.card(dock, sz(580.0, 140.0), "Single pull box");
    g.label(b1, tl(24.0, 22.0, 280.0, 36.0), "Single cost", "Cost: 160 Gems", 24.0);
    g.note(b1, tl(24.0, 64.0, 280.0, 34.0), "Single sub", "1x Summon", 23.0);
    g.button2(b1, cr(20.0, 0.0, 240.0, 74.0), "Summon 1x button", "Summon 1x");

    let b10 = g.card_on(dock, sz(640.0, 140.0), "Ten pull box");
    g.label(b10, tl(24.0, 22.0, 320.0, 36.0), "Ten cost", "Cost: 1,600 Gems", 24.0);
    g.note(b10, tl(24.0, 64.0, 320.0, 34.0), "Ten sub", "Guaranteed 4-Star+", 23.0);
    g.button(b10, cr(20.0, 0.0, 268.0, 76.0), "Summon 10x button", "Summon 10x");
}

/// Pantalla de apertura de caja revelando tres objetos obtenidos con rareza central destacada.
fn gacha_crate_open(g: &mut G) {
    let p = g.page();
    g.background();
    // Cabecera de revelacion
    let top_p = g.panel(p, tc(0.0, 32.0, 960.0, 128.0), "Reveal header");
    g.badge(top_p, tc(0.0, 16.0, 248.0, 42.0), "Drop badge", "LEGENDARY DROP!");
    g.heading(top_p, bc(0.0, 16.0, 880.0, 52.0), "Crate title", "CRYPT CRATE OPENED", 44.0);

    // Tres cartas reveladas al centro
    let cards = g.hbox_c(p, cc(0.0, -10.0, 1660.0, 640.0), "Loot reveal row", 34.0);
    let drops = [
        (false, "RARE", "Bone Charm", "Trinket - New", "Equip"),
        (true, "LEGENDARY", "SHADOW SCYTHE", "Mythic Weapon - New!", "Equip now"),
        (false, "EPIC", "Raven Cloak", "Duplicate -> +120 Dust", "Details"),
    ];
    for (hi, rar, name, sub, cta) in drops {
        let c = if hi {
            g.card_on(cards, sz(560.0, 624.0), "Legendary drop")
        } else {
            g.card(cards, sz(490.0, 560.0), "Crate drop")
        };
        g.badge(c, tc(0.0, 22.0, 210.0, 44.0), "Drop rarity", rar);
        g.art(c, top(86.0, if hi { 280.0 } else { 236.0 }, 24.0, 24.0), "Drop artwork");
        g.heading(c, bc(0.0, 166.0, 440.0, 48.0), "Drop name", name, if hi { 38.0 } else { 32.0 });
        g.note(c, bc(0.0, 116.0, 440.0, 36.0), "Drop note", sub, 24.0);
        if hi {
            g.button(c, bc(0.0, 24.0, 440.0, 74.0), "Equip drop button", cta);
        } else {
            g.button2(c, bc(0.0, 24.0, 380.0, 68.0), "Inspect drop button", cta);
        }
    }

    // Acciones inferiores
    let foot = g.panel(p, bc(0.0, 32.0, 1140.0, 116.0), "Reveal footer");
    let acts = g.hbox_c(foot, full(), "Crate actions", 24.0);
    g.button2(acts, sz(260.0, 72.0), "Inventory button", "Go to Armory");
    g.button(acts, sz(420.0, 76.0), "Open next button", "Open another (3 left)");
    g.button2(acts, sz(260.0, 72.0), "Continue button", "Continue");
}

/// Coleccion de cartas obtenidas en rejilla 4x2 con resumen de progreso por rareza a la derecha.
fn gacha_card_collection(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 92.0, 36.0, 36.0), "Archive header");
    g.heading_l(hdr, cl(28.0, 0.0, 380.0, 48.0), "Archive title", "CARD ARCHIVE", 38.0);
    g.badge(hdr, cl(380.0, 0.0, 250.0, 44.0), "Collected count", "42 / 120 OWNED");
    let filt = g.hbox(hdr, cr(24.0, 0.0, 720.0, 68.0), "Rarity filters", 12.0);
    for (i, t) in ["All", "5-Star", "4-Star", "3-Star"].iter().enumerate() {
        if i == 0 {
            g.button(filt, sz(168.0, 68.0), "Rarity button", t);
        } else {
            g.button2(filt, sz(168.0, 68.0), "Rarity button", t);
        }
    }

    // Rejilla 4x2 de cartas obtenidas
    let grid_p = g.panel(p, tl(36.0, 132.0, 1284.0, 912.0), "Cards grid panel");
    let grid = g.grid(grid_p, fill(24.0, 24.0, 24.0, 24.0), "Obtained cards", 294.0, 420.0, 20.0);
    let cards = [
        ("*****", "Pixel Knight", "x1"),
        ("*****", "Star Mage", "x2"),
        ("****", "Turbo Bot", "x3"),
        ("****", "Frost Archer", "x1"),
        ("****", "Flame Bard", "x2"),
        ("***", "Iron Slime", "x6"),
        ("***", "Cave Bat", "x5"),
        ("***", "Wood Golem", "x4"),
    ];
    for (i, (stars, name, cnt)) in cards.iter().enumerate() {
        let c = if i == 0 {
            g.card_on(grid, sz(294.0, 420.0), "Collected card")
        } else {
            g.card(grid, sz(294.0, 420.0), "Collected card")
        };
        g.art(c, top(16.0, 236.0, 16.0, 16.0), "Card art");
        g.badge(c, tr(26.0, 26.0, 86.0, 40.0), "Copy badge", cnt);
        g.heading(c, bc(0.0, 96.0, 260.0, 38.0), "Card name", name, 26.0);
        g.note(c, bc(0.0, 52.0, 260.0, 34.0), "Card stars", stars, 24.0);
        g.label(c, bc(0.0, 14.0, 260.0, 32.0), "Card status", "Collected", 22.0);
    }

    // Panel derecho de estadisticas y fabricacion
    let side = g.panel(p, tr(36.0, 132.0, 540.0, 912.0), "Collection stats");
    g.heading_l(side, tl(24.0, 24.0, 490.0, 44.0), "Stats heading", "COMPLETION STATS", 32.0);
    let bars = g.vbox(side, tl(24.0, 92.0, 492.0, 370.0), "Rarity bars", 18.0);
    for (lbl, txt, prog) in [
        ("5-Star Cards", "4 / 15", 4.0 / 15.0),
        ("4-Star Cards", "14 / 35", 14.0 / 35.0),
        ("3-Star Cards", "24 / 70", 24.0 / 70.0),
    ] {
        let rc = g.card(bars, sz(492.0, 108.0), "Rarity progress");
        g.label(rc, tl(18.0, 16.0, 280.0, 34.0), "Rarity name", lbl, 25.0);
        g.value(rc, tr(18.0, 16.0, 160.0, 34.0), "Rarity count", txt, 25.0);
        g.bar(rc, bottom(20.0, 24.0, 18.0, 18.0), "Rarity bar", prog, g.t.accent);
    }

    let dust = g.card_on(side, tl(24.0, 492.0, 492.0, 160.0), "Dust summary");
    g.heading_l(dust, tl(20.0, 22.0, 450.0, 40.0), "Dust heading", "DUPLICATE DUST", 28.0);
    g.heading_l(dust, tl(20.0, 74.0, 450.0, 52.0), "Dust amount", "380 Shards", 42.0);

    g.button(side, bc(0.0, 114.0, 492.0, 74.0), "Craft card button", "Craft missing card");
    g.button2(side, bc(0.0, 28.0, 492.0, 68.0), "Back summon button", "Back to Summon");
}

/// Barra superior de progreso de nivel acompañada de una lista vertical de proximos desbloqueos.
fn unlocks_progress_bar(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(20.0, 92.0, 40.0, 40.0), "Unlocks header");
    g.heading_l(hdr, cl(28.0, 0.0, 500.0, 48.0), "Unlocks title", "UPCOMING UNLOCKS", 38.0);
    g.badge(hdr, cc(140.0, 0.0, 180.0, 44.0), "Rank badge", "LEVEL 24");
    g.button2(hdr, cr(24.0, 0.0, 176.0, 68.0), "Back button", "Back");

    // Panel de barra de XP actual
    let xp = g.panel(p, top(132.0, 196.0, 40.0, 40.0), "XP progress panel");
    g.heading_l(xp, tl(28.0, 24.0, 760.0, 46.0), "XP heading", "NEXT UNLOCK IN 450 XP", 36.0);
    g.badge(xp, tr(28.0, 24.0, 236.0, 44.0), "Boost badge", "+25% XP ACTIVE");
    g.bar(xp, hmid(16.0, 34.0, 28.0, 28.0), "Level XP bar", 0.78, g.t.accent);
    g.label(xp, bl(28.0, 22.0, 320.0, 34.0), "Current level", "Level 24 (1,550 / 2,000 XP)", 24.0);
    g.value(xp, br(28.0, 22.0, 220.0, 34.0), "Next level", "Level 25", 26.0);

    // Lista inferior de proximos premios
    let list_p = g.panel(p, fill(40.0, 348.0, 40.0, 28.0), "Upcoming list panel");
    g.heading_l(list_p, tl(28.0, 22.0, 480.0, 42.0), "List heading", "REWARD QUEUE", 30.0);
    let rows = g.vbox(list_p, fill(28.0, 80.0, 28.0, 24.0), "Unlock rows", 16.0);
    let queue = [
        (true, "LVL 25", "Serrated Fang Blade", "Melee Weapon Blueprint", "450 XP left", 0.78),
        (false, "LVL 26", "Crypt Map Access", "New Survival Arena", "2 levels away", 0.40),
        (false, "LVL 28", "Blood Lantern Charm", "Passive Perk Slot +1", "4 levels away", 0.20),
        (false, "LVL 30", "Reaper Coat", "Mastery Outfit Set", "6 levels away", 0.10),
    ];
    for (next, lvl, name, cat, dist, prog) in queue {
        let r = if next {
            g.card_on(rows, sz(1784.0, 134.0), "Next unlock row")
        } else {
            g.card(rows, sz(1784.0, 134.0), "Future unlock row")
        };
        g.badge(r, cl(22.0, 0.0, 136.0, 46.0), "Level badge", lvl);
        g.art(r, cl(180.0, 0.0, 96.0, 96.0), "Unlock thumb");
        g.heading_l(r, tl(304.0, 24.0, 620.0, 40.0), "Unlock name", name, 30.0);
        g.note(r, tl(304.0, 70.0, 620.0, 34.0), "Unlock category", cat, 24.0);
        g.label(r, tr(28.0, 24.0, 360.0, 34.0), "Distance label", dist, 24.0);
        g.bar(r, br(28.0, 28.0, 360.0, 22.0), "Unlock progress", prog, if next { g.t.good } else { g.t.accent });
    }
}

/// Linea de tiempo horizontal de desbloqueos por rango con pista continua y cinco nodos de recompensa.
fn unlocks_timeline(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(24.0, 92.0, 40.0, 40.0), "Roadmap header");
    g.heading_l(hdr, cl(28.0, 0.0, 560.0, 48.0), "Roadmap title", "PROGRESSION ROADMAP", 38.0);
    g.badge(hdr, cc(140.0, 0.0, 196.0, 44.0), "Current rank", "RANK 14");
    g.button2(hdr, cr(24.0, 0.0, 210.0, 68.0), "All ranks button", "Full ladder");

    // Panel central con pista horizontal y 5 nodos
    let track = g.panel(p, hmid(0.0, 668.0, 40.0, 40.0), "Timeline panel");
    g.bar(track, top(78.0, 22.0, 60.0, 60.0), "Roadmap track bar", 0.42, g.t.good);

    let nodes = g.hbox_c(track, fill(28.0, 28.0, 28.0, 28.0), "Milestone nodes", 20.0);
    let steps = [
        (false, "RANK 12", "Herbal Pouch", "Unlocked", "DONE"),
        (true, "RANK 14", "Glider Wings", "Current Rank", "ACTIVE"),
        (false, "RANK 16", "Oak Longbow", "Next Unlock", "NEXT"),
        (false, "RANK 18", "Campfire Pet", "4 Ranks Away", "LOCKED"),
        (false, "RANK 20", "Grove Crown", "Master Reward", "GRAND"),
    ];
    for (cur, rk, title, desc, tag) in steps {
        let nc = if cur {
            g.card_on(nodes, sz(340.0, 592.0), "Current node")
        } else {
            g.card(nodes, sz(340.0, 564.0), "Milestone node")
        };
        g.badge(nc, tc(0.0, 22.0, 168.0, 44.0), "Rank tag", rk);
        g.art(nc, top(92.0, 250.0, 22.0, 22.0), "Reward art");
        g.heading(nc, tc(0.0, 368.0, 300.0, 42.0), "Reward title", title, 28.0);
        g.note(nc, tc(0.0, 418.0, 300.0, 36.0), "Reward desc", desc, 23.0);
        g.badge(nc, bc(0.0, 28.0, 152.0, 42.0), "Node state", tag);
    }

    // Franja inferior con siguiente objetivo
    let foot = g.panel(p, bottom(24.0, 116.0, 40.0, 40.0), "Goal footer");
    g.label(foot, cl(28.0, 0.0, 1200.0, 40.0), "Next goal note", "Complete 2 daily field quests to earn +600 XP and reach Rank 15.", 26.0);
    g.button(foot, cr(24.0, 0.0, 280.0, 72.0), "Quests button", "View Daily Quests");
}

/// Tres tarjetas destacadas de «Next reward» que muestran el premio mas cercano en cada modo.
fn unlocks_next_cards(g: &mut G) {
    let p = g.page();
    g.background();
    let hdr = g.panel(p, top(24.0, 96.0, 40.0, 40.0), "Next rewards header");
    g.heading_l(hdr, cl(28.0, 0.0, 460.0, 48.0), "Header title", "NEXT REWARDS", 40.0);
    g.note(hdr, cc(40.0, 0.0, 680.0, 36.0), "Header subtitle", "Your closest unlocks across Ranked, Mastery, and Season tracks.", 24.0);
    g.button2(hdr, cr(24.0, 0.0, 180.0, 68.0), "Back button", "Back");

    // Tres tarjetas grandes de siguiente recompensa
    let row = g.hbox_c(p, hmid(12.0, 772.0, 40.0, 40.0), "Next reward cards", 28.0);
    let items = [
        (true, "RANKED", "Gilded Victor Frame", "Win 2 more ranked matches", "8 / 10 Wins", 0.80, "Play Ranked"),
        (false, "MASTERY", "Silver Bow Finish", "Reach Bow Mastery Level 15", "13 / 15 Levels", 0.86, "Open Mastery"),
        (false, "SEASON", "300 Bonus Credits", "Earn 600 more Season XP", "72% Complete", 0.72, "Open Season Pass"),
    ];
    for (hi, mode, title, req, pct, prog, cta) in items {
        let c = if hi {
            g.card_on(row, sz(580.0, 756.0), "Featured next reward")
        } else {
            g.card(row, sz(580.0, 736.0), "Next reward card")
        };
        g.badge(c, tl(24.0, 24.0, 168.0, 42.0), "Track badge", mode);
        g.badge(c, tr(24.0, 24.0, 196.0, 42.0), "Next badge", "NEXT REWARD");
        g.art(c, top(88.0, 300.0, 24.0, 24.0), "Next reward art");
        g.heading_l(c, tl(24.0, 414.0, 532.0, 46.0), "Reward title", title, 34.0);
        g.note(c, tl(24.0, 468.0, 532.0, 38.0), "Reward requirement", req, 24.0);
        g.label(c, tl(24.0, 530.0, 532.0, 34.0), "Progress text", pct, 24.0);
        g.bar(c, tl(24.0, 574.0, 532.0, 26.0), "Progress bar", prog, if hi { g.t.good } else { g.t.accent });
        if hi {
            g.button(c, bc(0.0, 28.0, 532.0, 74.0), "Go track button", cta);
        } else {
            g.button2(c, bc(0.0, 28.0, 532.0, 70.0), "Go track button", cta);
        }
    }

    let foot = g.panel(p, bottom(24.0, 76.0, 40.0, 40.0), "Delivery notice");
    g.heading(foot, full(), "Auto unlock note", "Rewards unlock automatically as soon as their goal reaches 100%.", 24.0);
}
