//! Plantillas de galería: estadísticas, gestión de recursos y mejoras («Stats & resources»).

use super::*;

pub fn entries() -> Vec<Entry> {
    use Style::*;
    vec![
        // Stats
        Entry { ty: "Player menu", style: Fantasy, build: player_menu_tabs },
        Entry { ty: "Player menu", style: SciFi, build: player_menu_sidebar },
        Entry { ty: "Player menu", style: Cartoon, build: player_menu_tiles },
        Entry { ty: "Unlocked units", style: Retro, build: unlocked_units_grid },
        Entry { ty: "Unlocked units", style: Military, build: unlocked_units_filters },
        Entry { ty: "Unlocked units", style: Elegant, build: unlocked_units_factions },
        Entry { ty: "Team summary", style: Sports, build: team_summary_columns },
        Entry { ty: "Team summary", style: Minimal, build: team_summary_table },
        Entry { ty: "Team summary", style: Neon, build: team_summary_synergy },
        Entry { ty: "Overview & stats", style: Horror, build: overview_portrait_list },
        Entry { ty: "Overview & stats", style: SciFi, build: overview_compare_bars },
        Entry { ty: "Overview & stats", style: Nature, build: overview_two_column_sheet },
        Entry { ty: "Ability list", style: Fantasy, build: ability_list_rows },
        Entry { ty: "Ability list", style: Neon, build: ability_list_tree },
        Entry { ty: "Ability list", style: Minimal, build: ability_list_grid_bar },
        Entry { ty: "Maintenance & management", style: Military, build: maint_base_rooms },
        Entry { ty: "Maintenance & management", style: Horror, build: maint_repair_list },
        Entry { ty: "Maintenance & management", style: Sports, build: maint_staff_table },
        // Resource management
        Entry { ty: "Inventory grid", style: Fantasy, build: inventory_grid_detail },
        Entry { ty: "Inventory grid", style: Minimal, build: inventory_tabbed_list },
        Entry { ty: "Inventory grid", style: Horror, build: inventory_compact_slots },
        Entry { ty: "Equipping", style: Elegant, build: equipping_paperdoll },
        Entry { ty: "Equipping", style: Military, build: equipping_three_col },
        Entry { ty: "Equipping", style: Sports, build: equipping_loadout_perks },
        Entry { ty: "Buying & trading", style: Cartoon, build: trading_merchant_shop },
        Entry { ty: "Buying & trading", style: Retro, build: trading_two_col_barter },
        Entry { ty: "Buying & trading", style: Pastel, build: trading_market_cart },
        Entry { ty: "Change skin or accessory", style: Neon, build: skin_carousel_bottom },
        Entry { ty: "Change skin or accessory", style: Elegant, build: skin_rarity_list },
        Entry { ty: "Change skin or accessory", style: Pastel, build: skin_category_swatches },
        Entry { ty: "Character customization", style: Minimal, build: custom_sliders_preview },
        Entry { ty: "Character customization", style: Cartoon, build: custom_palette_presets },
        Entry { ty: "Character customization", style: Nature, build: custom_step_wizard },
        Entry { ty: "Sub-equip & tweak", style: Military, build: tweak_weapon_attachments },
        Entry { ty: "Sub-equip & tweak", style: Sports, build: tweak_vehicle_sliders },
        Entry { ty: "Sub-equip & tweak", style: Fantasy, build: tweak_gem_socketing },
        Entry { ty: "World customization", style: Pastel, build: world_house_decor },
        Entry { ty: "World customization", style: Cartoon, build: world_base_builder },
        Entry { ty: "World customization", style: Nature, build: world_rules_palette },
        // Upgrades & unlocks
        Entry { ty: "Upgrading & ranking up", style: SciFi, build: upgrade_before_after },
        Entry { ty: "Upgrading & ranking up", style: Sports, build: upgrade_rank_badge },
        Entry { ty: "Upgrading & ranking up", style: Retro, build: upgrade_stat_points },
        Entry { ty: "Unlockable upgrades", style: SciFi, build: unlock_tier_tree },
        Entry { ty: "Unlockable upgrades", style: Horror, build: unlock_req_list },
        Entry { ty: "Unlockable upgrades", style: Pastel, build: unlock_level_rows },
        Entry { ty: "Crafting", style: Nature, build: crafting_three_col },
        Entry { ty: "Crafting", style: Retro, build: crafting_table_3x3 },
        Entry { ty: "Crafting", style: Military, build: crafting_build_queue },
        Entry { ty: "Choose boon / upgrade", style: Fantasy, build: boon_three_cards },
        Entry { ty: "Choose boon / upgrade", style: Neon, build: boon_four_row_reroll },
        Entry { ty: "Choose boon / upgrade", style: Elegant, build: boon_patron_list },
    ]
}

// ---------------------------------------------------------------------------------------------------------------
// Auxiliares comunes
// ---------------------------------------------------------------------------------------------------------------

/// Franja superior con título a la izquierda y texto de estado o recurso a la derecha.
fn header_bar(g: &mut G, title: &str, right_text: &str) -> N {
    let p = g.page();
    let bar = g.panel(p, top(0.0, 96.0, 0.0, 0.0), "Header bar");
    g.heading_l(bar, cl(40.0, 0.0, 760.0, 56.0), "Screen title", title, 40.0);
    if !right_text.is_empty() {
        g.value(bar, cr(40.0, 0.0, 520.0, 48.0), "Header status", right_text, 28.0);
    }
    bar
}

/// Fila con nombre de atributo a la izquierda, valor numérico a la derecha y barra debajo.
fn stat_bar_row(g: &mut G, parent: N, w: f32, name: &str, val_txt: &str, frac: f32, color: Rgba) {
    let row = g.card(parent, sz(w, 72.0), &format!("{} row", name));
    g.label(row, tl(16.0, 10.0, w * 0.55, 30.0), &format!("{} label", name), name, 24.0);
    g.value(row, tr(16.0, 10.0, w * 0.35, 30.0), &format!("{} value", name), val_txt, 24.0);
    g.bar(row, bottom(12.0, 14.0, 16.0, 16.0), &format!("{} bar", name), frac, color);
}

/// Fila compacta de etiqueta + valor numérico dentro de un `vbox`.
fn kv_row(g: &mut G, parent: N, w: f32, h: f32, name: &str, val_txt: &str) {
    let row = g.card(parent, sz(w, h), &format!("{} row", name));
    g.label(row, cl(18.0, 0.0, w * 0.58, 34.0), &format!("{} label", name), name, 24.0);
    g.value(row, cr(18.0, 0.0, w * 0.34, 34.0), &format!("{} value", name), val_txt, 24.0);
}

/// Refuerza tamaño y negrita en controles con relleno `accent` del estilo Cartoon para WCAG en vertical.
fn big_accent(g: &mut G, n: N) {
    let fs = 34.0 * g.s;
    g.edit(n, |e| {
        e.bold = true;
        e.font_size = e.font_size.max(fs);
    });
}

// ===============================================================================================================
// 1. Player menu
// ===============================================================================================================

/// Barra de pestañas arriba, muñeco del personaje centrado y paneles de resumen a ambos lados.
fn player_menu_tabs(g: &mut G) {
    let p = g.page();
    g.background();
    let top_bar = g.panel(p, top(0.0, 104.0, 0.0, 0.0), "Top navigation");
    let tabs = g.hbox_c(top_bar, cc(0.0, 0.0, 1260.0, 72.0), "Menu tabs", 16.0);
    for (i, t) in ["Inventory", "Skills", "Map", "Journal", "Settings"].iter().enumerate() {
        if i == 0 {
            g.button(tabs, sz(232.0, 70.0), &format!("{} tab", t), t);
        } else {
            g.button2(tabs, sz(232.0, 70.0), &format!("{} tab", t), t);
        }
    }
    // Panel izquierdo: resumen rápido del personaje
    let left_p = g.panel(p, left(40.0, 480.0, 134.0, 110.0), "Character summary");
    g.heading_l(left_p, tl(24.0, 24.0, 430.0, 46.0), "Hero name", "Rowan Vale", 36.0);
    g.note(left_p, tl(24.0, 72.0, 430.0, 34.0), "Hero class", "Lv 28 Ranger - Suncrest", 24.0);
    let bars = g.vbox(left_p, tl(24.0, 132.0, 432.0, 340.0), "Vital bars", 14.0);
    stat_bar_row(g, bars, 432.0, "Health", "640 / 700", 0.91, g.t.good);
    stat_bar_row(g, bars, 432.0, "Mana", "210 / 260", 0.81, g.t.accent);
    stat_bar_row(g, bars, 432.0, "Experience", "8,420 / 10,000", 0.84, g.t.warn);
    let stats = g.vbox(left_p, bl(24.0, 24.0, 432.0, 224.0), "Core stats", 10.0);
    for (k, v) in [("Attack", "148"), ("Defense", "96"), ("Crit Chance", "22%")] {
        kv_row(g, stats, 432.0, 64.0, k, v);
    }
    // Centro: figura del personaje
    let stage = g.panel(p, fill(550.0, 134.0, 550.0, 110.0), "Character stage");
    g.art(stage, fill(36.0, 36.0, 36.0, 110.0), "Character model");
    let plate = g.card(stage, bottom(22.0, 72.0, 36.0, 36.0), "Title plate");
    g.heading(plate, full(), "Active title", "Warden of the Amber Woods", 28.0);
    // Derecha: acceso rápido de equipo y monedas
    let right_p = g.panel(p, right(40.0, 480.0, 134.0, 110.0), "Loadout summary");
    g.heading_l(right_p, tl(24.0, 24.0, 430.0, 44.0), "Equipped title", "Active Loadout", 32.0);
    let eq = g.vbox(right_p, tl(24.0, 88.0, 432.0, 420.0), "Loadout list", 14.0);
    for (slot, item) in [("Weapon", "Ashwood Longbow"), ("Armor", "Stalker Coat"), ("Charm", "Amber Compass"), ("Potion", "Sunleaf Tonic x5")] {
        kv_row(g, eq, 432.0, 88.0, slot, item);
    }
    let purse = g.card(right_p, bl(24.0, 24.0, 432.0, 120.0), "Currency box");
    g.label(purse, tl(20.0, 18.0, 200.0, 36.0), "Gold label", "Gold", 26.0);
    g.value(purse, tr(20.0, 18.0, 200.0, 36.0), "Gold value", "14,250", 28.0);
    g.note(purse, bl(20.0, 18.0, 240.0, 32.0), "Carry label", "Carry weight", 22.0);
    g.value(purse, br(20.0, 18.0, 160.0, 32.0), "Carry value", "42 / 70", 24.0);
    // Pie con atajos de teclas
    let foot = g.panel(p, bottom(0.0, 80.0, 0.0, 0.0), "Footer bar");
    let keys = g.hbox(foot, cl(40.0, 0.0, 720.0, 48.0), "Key hints", 16.0);
    g.key(keys, sz(48.0, 46.0), "Prev tab key", "Q");
    g.key(keys, sz(48.0, 46.0), "Next tab key", "E");
    g.note(keys, sz(180.0, 46.0), "Switch hint", "Switch tab", 24.0);
    g.key(keys, sz(64.0, 46.0), "Close key", "Esc");
    g.note(keys, sz(160.0, 46.0), "Close hint", "Resume", 24.0);
}

/// Navegación vertical a la izquierda y vista previa del contenido principal a la derecha.
fn player_menu_sidebar(g: &mut G) {
    let p = g.page();
    g.background();
    let side = g.panel(p, left(0.0, 440.0, 0.0, 0.0), "Sidebar menu");
    let id_card = g.card(side, tl(24.0, 28.0, 392.0, 130.0), "Pilot card");
    g.art(id_card, cl(16.0, 0.0, 96.0, 96.0), "Pilot avatar");
    g.heading_l(id_card, tl(128.0, 24.0, 248.0, 40.0), "Pilot name", "Cmdr. Kael", 30.0);
    g.note(id_card, tl(128.0, 68.0, 248.0, 34.0), "Pilot rank", "Rank 19 - Sector 4", 22.0);
    let nav = g.vbox_fill(side, tl(24.0, 184.0, 392.0, 560.0), "Navigation list", 14.0);
    for (i, item) in ["Overview", "Inventory", "Cybernetics", "Star Map", "Logs", "Settings"].iter().enumerate() {
        g.menu_item(nav, sz(392.0, 74.0), &format!("{} nav", item), item, i == 0);
    }
    g.button2(side, bl(24.0, 28.0, 392.0, 72.0), "Return button", "Return to Bridge");
    // Panel derecho de vista previa
    let main = g.panel(p, fill(470.0, 28.0, 32.0, 28.0), "Preview panel");
    g.heading_l(main, tl(36.0, 30.0, 640.0, 50.0), "Section heading", "Command Overview", 40.0);
    g.badge(main, tr(36.0, 34.0, 180.0, 44.0), "Status badge", "ONLINE");
    g.art(main, tl(36.0, 102.0, 820.0, 440.0), "Suit preview");
    let right_col = g.vbox(main, tr(36.0, 102.0, 480.0, 440.0), "Metrics list", 16.0);
    stat_bar_row(g, right_col, 480.0, "Shield Integrity", "92%", 0.92, g.t.accent);
    stat_bar_row(g, right_col, 480.0, "Core Power", "78%", 0.78, g.t.good);
    stat_bar_row(g, right_col, 480.0, "Cargo Load", "34 / 50", 0.68, g.t.warn);
    kv_row(g, right_col, 480.0, 72.0, "Credits", "48,900 CR");
    kv_row(g, right_col, 480.0, 72.0, "Active Mission", "Signal in Drift 9");
    let bottom_row = g.hbox(main, bottom(30.0, 360.0, 36.0, 36.0), "Summary cards", 20.0);
    for (title, line1, line2) in [
        ("Current Objective", "Reach Relay Beacon B", "Distance: 420m"),
        ("Equipped Kit", "Pulse Carbine Mk.IV", "2 Med-Cells ready"),
        ("Next Milestone", "Unlock Thruster Tier 3", "1,200 XP needed"),
    ] {
        let c = g.card(bottom_row, sz(434.0, 340.0), &format!("{} card", title));
        g.heading_l(c, tl(24.0, 24.0, 386.0, 44.0), &format!("{} heading", title), title, 28.0);
        g.label(c, tl(24.0, 92.0, 386.0, 44.0), &format!("{} text", title), line1, 24.0);
        g.note(c, tl(24.0, 146.0, 386.0, 40.0), &format!("{} detail", title), line2, 22.0);
        g.button2(c, bl(24.0, 24.0, 220.0, 68.0), &format!("Open {}", title), "Open");
    }
}

/// Centro de mando con 6 baldosas grandes y franja superior de perfil.
fn player_menu_tiles(g: &mut G) {
    let p = g.page();
    g.background();
    let bar = header_bar(g, "ADVENTURE HUB", "Coins: 3,420   Gems: 85");
    let lv_b = g.badge(bar, cc(140.0, 0.0, 200.0, 52.0), "Level badge", "LEVEL 14");
    big_accent(g, lv_b);
    let wrap = g.panel(p, fill(80.0, 130.0, 80.0, 110.0), "Hub board");
    let grid = g.grid(wrap, cc(0.0, 0.0, 1660.0, 760.0), "Hub tiles", 530.0, 360.0, 30.0);
    let tiles = [
        ("Inventory", "38 / 60 items", "BAG", true),
        ("Skill Book", "2 points ready", "SKL", false),
        ("World Map", "5 zones cleared", "MAP", false),
        ("Party Roster", "4 active heroes", "PTY", false),
        ("Quest Log", "3 tracked tasks", "QST", false),
        ("Camp Settings", "Audio & controls", "OPT", false),
    ];
    for (title, sub, glyph, active) in tiles {
        let tile = if active {
            g.card_on(grid, sz(530.0, 360.0), &format!("{} tile", title))
        } else {
            g.card(grid, sz(530.0, 360.0), &format!("{} tile", title))
        };
        g.icon(tile, tl(28.0, 28.0, 84.0, 84.0), &format!("{} icon", title), glyph);
        g.heading_l(tile, tl(132.0, 38.0, 370.0, 48.0), &format!("{} title", title), title, 36.0);
        g.note(tile, tl(132.0, 88.0, 370.0, 38.0), &format!("{} subtitle", title), sub, 24.0);
        g.art(tile, tl(28.0, 134.0, 474.0, 120.0), &format!("{} banner", title));
        if active {
            let btn = g.button(tile, bl(28.0, 22.0, 220.0, 68.0), &format!("Open {}", title), "Open");
            big_accent(g, btn);
        } else {
            g.button2(tile, bl(28.0, 22.0, 220.0, 68.0), &format!("Open {}", title), "Open");
        }
    }
    let foot = g.panel(p, bottom(0.0, 82.0, 0.0, 0.0), "Hub footer");
    g.note(foot, cl(40.0, 0.0, 700.0, 38.0), "Tip text", "Tip: Visit the Camp to restore full stamina.", 24.0);
    g.button2(foot, cr(40.0, 0.0, 220.0, 68.0), "Close button", "Resume Game");
}

// ===============================================================================================================
// 2. Unlocked units
// ===============================================================================================================

/// Rejilla de unidades con siluetas bloqueadas y contador global «24/60».
fn unlocked_units_grid(g: &mut G) {
    let p = g.page();
    g.background();
    let bar = header_bar(g, "UNIT COLLECTION", "Unlocked: 24 / 60");
    g.bar(bar, cc(180.0, 0.0, 360.0, 22.0), "Collection progress", 0.40, g.t.good);
    let main = g.panel(p, fill(60.0, 126.0, 60.0, 40.0), "Roster board");
    let grid = g.grid(main, cc(0.0, 0.0, 1720.0, 840.0), "Unit grid", 412.0, 260.0, 24.0);
    let units = [
        ("Pixel Knight", "Lv 18", true, true),
        ("Flame Mage", "Lv 15", true, false),
        ("Moss Archer", "Lv 14", true, false),
        ("Clockwork Golem", "Lv 12", true, false),
        ("Storm Lancer", "Lv 11", true, false),
        ("Frost Cleric", "Lv 10", true, false),
        ("Shadow Rogue", "Lv 9", true, false),
        ("Iron Bard", "Lv 8", true, false),
        ("Locked Unit", "Clear Stage 4", false, false),
        ("Locked Unit", "Clear Stage 5", false, false),
        ("Locked Unit", "Reach Rank B", false, false),
        ("Locked Unit", "Secret Quest", false, false),
    ];
    for (name, sub, unlocked, selected) in units {
        let c = if selected {
            g.card_on(grid, sz(412.0, 260.0), "Unit card")
        } else {
            g.card(grid, sz(412.0, 260.0), "Unit card")
        };
        if unlocked {
            g.art(c, tl(18.0, 18.0, 140.0, 140.0), "Unit portrait");
            g.heading_l(c, tl(176.0, 26.0, 218.0, 42.0), "Unit name", name, 28.0);
            g.note(c, tl(176.0, 74.0, 218.0, 36.0), "Unit level", sub, 24.0);
            g.badge(c, bl(18.0, 20.0, 140.0, 44.0), "Status tag", "READY");
        } else {
            g.art_c(c, tl(18.0, 18.0, 140.0, 140.0), "Locked silhouette", shade(g.t.panel, 0.5), shade(g.t.panel, 0.35));
            g.heading_l(c, tl(176.0, 26.0, 218.0, 42.0), "Locked name", "???", 30.0);
            g.note(c, tl(176.0, 74.0, 218.0, 36.0), "Unlock hint", sub, 22.0);
            g.label(c, bl(18.0, 22.0, 160.0, 36.0), "Locked label", "LOCKED", 24.0);
        }
    }
}

/// Lista con filtros de clase a la izquierda y ficha detallada de la unidad a la derecha.
fn unlocked_units_filters(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "SQUAD BARRACKS", "Active Roster: 18 / 32");
    let left_p = g.panel(p, left(40.0, 820.0, 126.0, 40.0), "Unit list panel");
    let filters = g.hbox(left_p, tl(24.0, 24.0, 772.0, 70.0), "Filter tabs", 12.0);
    for (i, f) in ["All", "Vanguard", "Support", "Recon"].iter().enumerate() {
        if i == 0 {
            g.button(filters, sz(184.0, 68.0), &format!("{} filter", f), f);
        } else {
            g.button2(filters, sz(184.0, 68.0), &format!("{} filter", f), f);
        }
    }
    let list = g.vbox(left_p, tl(24.0, 114.0, 772.0, 760.0), "Units list", 14.0);
    let rows = [
        ("Sgt. Mara Vance", "Vanguard - Lv 24", "OWNED", true),
        ("Cpl. Owen Pike", "Recon - Lv 21", "OWNED", false),
        ("Dr. Lena Cho", "Support - Lv 22", "OWNED", false),
        ("Pvt. Bram Cole", "Vanguard - Lv 19", "OWNED", false),
        ("Spec. Nadia Sol", "Recon - Lv 17", "OWNED", false),
        ("Capt. Reid Frost", "Vanguard - Unlock at Tier 5", "LOCKED", false),
    ];
    for (name, role, tag, sel) in rows {
        let row = if sel {
            g.card_on(list, sz(772.0, 110.0), "Unit row")
        } else {
            g.card(list, sz(772.0, 110.0), "Unit row")
        };
        g.art(row, cl(16.0, 0.0, 76.0, 76.0), "Unit thumb");
        g.heading_l(row, tl(110.0, 20.0, 430.0, 38.0), "Unit row name", name, 28.0);
        g.note(row, tl(110.0, 60.0, 430.0, 32.0), "Unit row role", role, 22.0);
        if tag == "OWNED" {
            g.badge(row, cr(20.0, 0.0, 136.0, 44.0), "Owned tag", tag);
        } else {
            g.value(row, cr(20.0, 0.0, 136.0, 36.0), "Locked tag", tag, 22.0);
        }
    }
    // Panel de detalle derecho
    let detail = g.panel(p, fill(890.0, 126.0, 40.0, 40.0), "Unit detail panel");
    g.art(detail, tl(32.0, 32.0, 340.0, 420.0), "Unit full portrait");
    g.heading_l(detail, tl(400.0, 36.0, 540.0, 50.0), "Selected unit", "Sgt. Mara Vance", 38.0);
    g.badge(detail, tl(400.0, 96.0, 180.0, 44.0), "Role badge", "VANGUARD");
    g.note(detail, tl(400.0, 160.0, 540.0, 96.0), "Unit bio", "Frontline breacher equipped with reactive armor and riot shield.", 24.0);
    let st = g.vbox(detail, tl(400.0, 276.0, 540.0, 240.0), "Unit stats", 12.0);
    stat_bar_row(g, st, 540.0, "Armor", "88 / 100", 0.88, g.t.good);
    stat_bar_row(g, st, 540.0, "Firepower", "74 / 100", 0.74, g.t.accent);
    stat_bar_row(g, st, 540.0, "Mobility", "62 / 100", 0.62, g.t.warn);
    let traits = g.card(detail, tl(32.0, 540.0, 916.0, 220.0), "Traits card");
    g.heading_l(traits, tl(24.0, 20.0, 400.0, 40.0), "Traits heading", "Combat Traits", 28.0);
    g.label(traits, tl(24.0, 74.0, 860.0, 38.0), "Trait 1", "+ Shield Wall: Reduces squad damage taken by 15%.", 24.0);
    g.label(traits, tl(24.0, 120.0, 860.0, 38.0), "Trait 2", "+ Breach Charge: Stuns nearby foes for 2.0 seconds.", 24.0);
    g.button(detail, br(32.0, 32.0, 280.0, 74.0), "Deploy button", "Assign to Squad");
    g.button2(detail, bl(32.0, 32.0, 240.0, 74.0), "Inspect button", "Inspect Gear");
}

/// Pestañas de facción arriba y carrusel horizontal de unidades con la central destacada.
fn unlocked_units_factions(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "HALL OF CHAMPIONS", "Factions: 4   Collected: 19 / 40");
    let fbar = g.panel(p, top(116.0, 96.0, 80.0, 80.0), "Faction bar");
    let ftabs = g.hbox_c(fbar, cc(0.0, 0.0, 1280.0, 70.0), "Faction tabs", 20.0);
    for (i, fac) in ["Sun Court", "Iron Guild", "Mist Order", "Dawn Ward"].iter().enumerate() {
        if i == 0 {
            g.button(ftabs, sz(300.0, 68.0), &format!("{} tab", fac), fac);
        } else {
            g.button2(ftabs, sz(300.0, 68.0), &format!("{} tab", fac), fac);
        }
    }
    let stage = g.panel(p, fill(80.0, 236.0, 80.0, 130.0), "Carousel stage");
    g.button2(stage, cl(28.0, 0.0, 84.0, 84.0), "Previous unit", "<");
    g.button2(stage, cr(28.0, 0.0, 84.0, 84.0), "Next unit", ">");
    let row = g.hbox_c(stage, cc(0.0, 0.0, 1480.0, 620.0), "Unit carousel", 28.0);
    let cards = [
        ("Aurelia Dawn", "Sun Paladin", "Lv 20", false, true),
        ("Solaris Rex", "High Templar", "Lv 30", true, true),
        ("Veiled Seer", "Unlock in Act III", "Locked", false, false),
    ];
    for (name, title, lv, featured, unlocked) in cards {
        let (cw, ch) = if featured { (520.0, 600.0) } else { (440.0, 520.0) };
        let c = if featured {
            g.card_on(row, sz(cw, ch), "Champion card")
        } else {
            g.card(row, sz(cw, ch), "Champion card")
        };
        if unlocked {
            g.art(c, tl(24.0, 24.0, cw - 48.0, ch - 210.0), "Champion art");
        } else {
            g.art_c(c, tl(24.0, 24.0, cw - 48.0, ch - 210.0), "Locked art", shade(g.t.panel, 0.45), shade(g.t.panel, 0.3));
        }
        g.heading(c, bottom(118.0, 44.0, 20.0, 20.0), "Champion name", name, if featured { 34.0 } else { 28.0 });
        g.txt(c, bottom(72.0, 36.0, 20.0, 20.0), "Champion title", title, 22.0, g.t.muted, CENTER, false);
        g.badge(c, bc(0.0, 18.0, 150.0, 44.0), "Champion status", lv);
    }
    let foot = g.panel(p, bottom(28.0, 84.0, 80.0, 80.0), "Faction footer");
    g.label(foot, cl(30.0, 0.0, 820.0, 38.0), "Faction bonus", "Sun Court Bonus: +10% Healing when 2 or more units deploy.", 24.0);
    g.button(foot, cr(24.0, 0.0, 260.0, 68.0), "Select champion", "Select Champion");
}

// ===============================================================================================================
// 3. Team summary
// ===============================================================================================================

/// Cuatro columnas de miembros del equipo con retrato, barras de estado y nivel.
fn team_summary_columns(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "ACTIVE SQUAD", "Team Rating: 4,820");
    let wrap = g.panel(p, fill(50.0, 126.0, 50.0, 120.0), "Squad board");
    let cols = g.hbox_c(wrap, cc(0.0, 0.0, 1760.0, 770.0), "Member columns", 24.0);
    let members = [
        ("Kira Sol", "Striker", "Lv 24", 0.94, 0.80, "1,280", true),
        ("Bram Holt", "Defender", "Lv 23", 0.88, 0.65, "1,190", false),
        ("Nia Park", "Tactician", "Lv 25", 0.72, 0.95, "1,240", false),
        ("Theo Cruz", "Keeper", "Lv 22", 1.00, 0.76, "1,110", false),
    ];
    for (name, role, lv, hp, sp, pwr, leader) in members {
        let col = if leader {
            g.card_on(cols, sz(420.0, 750.0), "Member column")
        } else {
            g.card(cols, sz(420.0, 750.0), "Member column")
        };
        g.art(col, tl(24.0, 24.0, 372.0, 260.0), "Member portrait");
        g.heading_l(col, tl(24.0, 304.0, 250.0, 44.0), "Member name", name, 32.0);
        g.badge(col, tr(24.0, 304.0, 108.0, 44.0), "Member level", lv);
        g.note(col, tl(24.0, 354.0, 372.0, 34.0), "Member role", role, 24.0);
        let bars = g.vbox(col, tl(24.0, 408.0, 372.0, 168.0), "Member bars", 14.0);
        stat_bar_row(g, bars, 372.0, "Stamina", &format!("{}%", (hp * 100.0) as i32), hp, g.t.good);
        stat_bar_row(g, bars, 372.0, "Focus", &format!("{}%", (sp * 100.0) as i32), sp, g.t.accent);
        g.label(col, tl(24.0, 594.0, 180.0, 36.0), "Power label", "Power", 24.0);
        g.value(col, tr(24.0, 594.0, 180.0, 36.0), "Power value", pwr, 28.0);
        g.button2(col, bl(24.0, 24.0, 372.0, 68.0), "Inspect member", "Inspect");
    }
    let foot = g.panel(p, bottom(0.0, 96.0, 0.0, 0.0), "Squad footer");
    g.label(foot, cl(50.0, 0.0, 760.0, 38.0), "Formation text", "Formation: 2-1-1 Balanced Press", 26.0);
    let btns = g.hbox(foot, cr(50.0, 0.0, 540.0, 70.0), "Squad actions", 20.0);
    g.button2(btns, sz(260.0, 68.0), "Swap button", "Change Roster");
    g.button(btns, sz(260.0, 68.0), "Ready button", "Confirm Squad");
}

/// Tabla de estadísticas por fila con columnas alineadas y totales del equipo abajo.
fn team_summary_table(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "PARTY ROSTER TABLE", "Members: 5 / 5");
    let board = g.panel(p, fill(60.0, 126.0, 60.0, 126.0), "Table panel");
    // Encabezado de columnas
    let thead = g.card(board, top(24.0, 64.0, 28.0, 28.0), "Table header");
    let cols = [("Unit", 30.0, 380.0), ("Class", 430.0, 240.0), ("Lv", 700.0, 120.0), ("HP", 860.0, 180.0), ("ATK", 1080.0, 160.0), ("DEF", 1280.0, 160.0), ("SPD", 1480.0, 160.0)];
    for (label, x, w) in cols {
        g.heading_l(thead, tl(x, 14.0, w, 36.0), &format!("{} col", label), label, 24.0);
    }
    let rows_box = g.vbox(board, fill(28.0, 108.0, 28.0, 110.0), "Roster rows", 14.0);
    let data = [
        ("Alden Reed", "Knight", "28", "840 / 840", "142", "168", "64", true),
        ("Mira Thorne", "Arcanist", "27", "510 / 540", "194", "82", "92", false),
        ("Soren Hale", "Scout", "28", "620 / 620", "165", "98", "124", false),
        ("Talia Moss", "Herbalist", "26", "590 / 590", "108", "112", "88", false),
        ("Garrick Stone", "Warden", "27", "920 / 920", "130", "184", "52", false),
    ];
    for (name, cls, lv, hp, atk, def, spd, sel) in data {
        let r = if sel {
            g.card_on(rows_box, sz(1744.0, 96.0), "Member row")
        } else {
            g.card(rows_box, sz(1744.0, 96.0), "Member row")
        };
        g.icon(r, cl(20.0, 0.0, 52.0, 52.0), "Member badge", &name[..2]);
        g.heading_l(r, tl(92.0, 28.0, 320.0, 40.0), "Row name", name, 26.0);
        g.note(r, tl(430.0, 30.0, 240.0, 36.0), "Row class", cls, 24.0);
        g.label(r, tl(700.0, 30.0, 120.0, 36.0), "Row level", lv, 24.0);
        g.label(r, tl(860.0, 30.0, 180.0, 36.0), "Row hp", hp, 24.0);
        g.label(r, tl(1080.0, 30.0, 160.0, 36.0), "Row atk", atk, 24.0);
        g.label(r, tl(1280.0, 30.0, 160.0, 36.0), "Row def", def, 24.0);
        g.label(r, tl(1480.0, 30.0, 160.0, 36.0), "Row spd", spd, 24.0);
    }
    let summary = g.card(board, bottom(20.0, 72.0, 28.0, 28.0), "Totals bar");
    g.heading_l(summary, cl(24.0, 0.0, 480.0, 40.0), "Totals label", "Party Totals   Avg Lv 27.2", 26.0);
    g.value(summary, cr(24.0, 0.0, 760.0, 40.0), "Totals values", "HP 3,510   ATK 739   DEF 644   SPD 420", 26.0);
    let foot = g.panel(p, bottom(0.0, 96.0, 0.0, 0.0), "Table footer");
    g.button2(foot, cl(60.0, 0.0, 240.0, 68.0), "Auto sort", "Sort by Power");
    g.button(foot, cr(60.0, 0.0, 240.0, 68.0), "Save party", "Save Order");
}

/// Tarjeta de equipo principal con poder total, bonos de sinergia y fila de miembros abajo.
fn team_summary_synergy(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "STRIKE TEAM OVERVIEW", "Synergy Rank: S");
    // Bloque superior: poder total + bonos de sinergia
    let hero = g.panel(p, top(126.0, 410.0, 60.0, 60.0), "Team summary card");
    let power_box = g.card_on(hero, tl(32.0, 32.0, 460.0, 346.0), "Power box");
    g.note(power_box, tl(28.0, 28.0, 380.0, 34.0), "Power tag", "TOTAL COMBAT POWER", 22.0);
    g.heading_l(power_box, tl(28.0, 74.0, 400.0, 86.0), "Power number", "18,450", 72.0);
    g.bar(power_box, tl(28.0, 186.0, 404.0, 20.0), "Power meter", 0.86, g.t.accent);
    g.note(power_box, tl(28.0, 224.0, 404.0, 34.0), "Recommended note", "Recommended for Zone 7: 16,000", 22.0);
    g.badge(power_box, bl(28.0, 24.0, 190.0, 46.0), "Ready badge", "OPTIMAL");
    // Bonos de sinergia a la derecha
    let syn_list = g.vbox(hero, fill(524.0, 32.0, 32.0, 32.0), "Synergy bonuses", 16.0);
    let syns = [
        ("Volt Resonance (3/3)", "All squad attacks gain +18% shock damage.", "ACTIVE", true),
        ("Frontline Anchor (2/2)", "Shields regenerate 4% faster during combat.", "ACTIVE", true),
        ("Night Stalkers (1/3)", "Add 2 more stealth units to unlock +12% crit.", "1 / 3", false),
    ];
    for (title, desc, badge_t, on) in syns {
        let c = if on {
            g.card_on(syn_list, sz(1244.0, 104.0), "Synergy card")
        } else {
            g.card(syn_list, sz(1244.0, 104.0), "Synergy card")
        };
        g.heading_l(c, tl(24.0, 18.0, 760.0, 38.0), "Synergy title", title, 28.0);
        g.note(c, tl(24.0, 58.0, 960.0, 34.0), "Synergy desc", desc, 22.0);
        if on {
            g.badge(c, cr(24.0, 0.0, 148.0, 44.0), "Synergy state", badge_t);
        } else {
            g.value(c, cr(24.0, 0.0, 148.0, 36.0), "Synergy count", badge_t, 24.0);
        }
    }
    // Fila inferior: 4 miembros
    let squad = g.panel(p, bottom(36.0, 470.0, 60.0, 60.0), "Squad strip");
    g.heading_l(squad, tl(32.0, 22.0, 480.0, 42.0), "Members heading", "Deployed Operatives", 30.0);
    g.button(squad, tr(32.0, 18.0, 240.0, 68.0), "Edit team", "Edit Team");
    let row = g.hbox_c(squad, bottom(28.0, 340.0, 32.0, 32.0), "Operative cards", 24.0);
    for (name, tag, pwr) in [
        ("Vex Zero", "VOLT / ASSAULT", "4,920"),
        ("Rina Pulse", "VOLT / MEDIC", "4,310"),
        ("Koda Steel", "ANCHOR / TANK", "4,680"),
        ("Lumen Shade", "VOLT / RECON", "4,540"),
    ] {
        let c = g.card(row, sz(416.0, 326.0), "Operative card");
        g.art(c, tl(20.0, 20.0, 376.0, 160.0), "Operative art");
        g.heading_l(c, tl(20.0, 196.0, 376.0, 38.0), "Operative name", name, 28.0);
        g.note(c, tl(20.0, 238.0, 376.0, 32.0), "Operative tag", tag, 22.0);
        g.value(c, br(20.0, 18.0, 200.0, 36.0), "Operative power", &format!("PWR {}", pwr), 24.0);
    }
}

// ===============================================================================================================
// 4. Overview & stats
// ===============================================================================================================

/// Retrato grande del personaje a la izquierda y lista vertical de atributos a la derecha.
fn overview_portrait_list(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "SURVIVOR DOSSIER", "Day 14 - Asylum Wing");
    let left_p = g.panel(p, left(50.0, 640.0, 126.0, 50.0), "Portrait column");
    g.art(left_p, tl(30.0, 30.0, 580.0, 540.0), "Character portrait");
    g.heading_l(left_p, tl(30.0, 594.0, 580.0, 48.0), "Survivor name", "Clara Mercer", 36.0);
    g.note(left_p, tl(30.0, 646.0, 580.0, 36.0), "Condition", "Condition: Wounded (-10% Stamina)", 24.0);
    let vit = g.card(left_p, bl(30.0, 30.0, 580.0, 184.0), "Status summary");
    g.label(vit, tl(20.0, 14.0, 260.0, 32.0), "Vitality label", "Vitality", 24.0);
    g.value(vit, tr(20.0, 14.0, 200.0, 32.0), "Vitality value", "68 / 100", 24.0);
    g.bar(vit, tl(20.0, 52.0, 540.0, 16.0), "Vitality bar", 0.68, g.t.bad);
    g.label(vit, tl(20.0, 80.0, 260.0, 32.0), "Sanity label", "Composure", 24.0);
    g.value(vit, tr(20.0, 80.0, 200.0, 32.0), "Sanity value", "74 / 100", 24.0);
    g.bar(vit, tl(20.0, 118.0, 540.0, 16.0), "Sanity bar", 0.74, g.t.warn);
    g.note(vit, bl(20.0, 12.0, 540.0, 30.0), "Status note", "Carry light source to slow composure drain.", 22.0);
    // Derecha: lista de atributos
    let right_p = g.panel(p, fill(720.0, 126.0, 50.0, 50.0), "Attributes panel");
    g.heading_l(right_p, tl(32.0, 28.0, 600.0, 46.0), "Attr heading", "Core Attributes", 34.0);
    let list = g.vbox(right_p, tl(32.0, 96.0, 1086.0, 670.0), "Attributes list", 14.0);
    let attrs = [
        ("Strength", "Melee impact and heavy carry limit", "14"),
        ("Endurance", "Sprint duration and bleed resistance", "18"),
        ("Agility", "Dodge window and stealth noise", "16"),
        ("Perception", "Trap spotting and flashlight range", "21"),
        ("Resolve", "Resistance against panic and shock", "15"),
        ("Craftsmanship", "Lockpicking speed and bandage yield", "19"),
        ("Medicine", "Healing gained from field kits", "17"),
    ];
    for (name, desc, val) in attrs {
        let row = g.card(list, sz(1086.0, 82.0), &format!("{} attr", name));
        g.heading_l(row, tl(24.0, 12.0, 360.0, 36.0), &format!("{} name", name), name, 26.0);
        g.note(row, tl(24.0, 48.0, 740.0, 28.0), &format!("{} desc", name), desc, 22.0);
        g.value(row, cr(28.0, 0.0, 140.0, 44.0), &format!("{} num", name), val, 34.0);
    }
    g.button2(right_p, br(32.0, 28.0, 240.0, 68.0), "Close dossier", "Back");
}

/// Barras horizontales por atributo con comparación Base frente a Equipo y resumen lateral.
fn overview_compare_bars(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "EXO-SUIT TELEMETRY", "Frame: MK-VI Striker");
    let main = g.panel(p, left(50.0, 1240.0, 126.0, 50.0), "Comparison panel");
    g.heading_l(main, tl(32.0, 26.0, 520.0, 44.0), "Compare title", "Performance Breakdown", 32.0);
    g.note(main, tr(32.0, 34.0, 520.0, 34.0), "Legend note", "Top bar: Base   Bottom bar: With Modules", 22.0);
    let rows = g.vbox(main, tl(32.0, 94.0, 1176.0, 770.0), "Stat comparison list", 16.0);
    let data = [
        ("Kinetic Output", "120 -> 168 (+48)", 0.56, 0.80),
        ("Thermal Shield", "95 -> 140 (+45)", 0.45, 0.70),
        ("Thruster Speed", "110 -> 132 (+22)", 0.55, 0.68),
        ("Sensor Range", "80 -> 150 (+70)", 0.40, 0.78),
        ("Energy Regen", "60 -> 96 (+36)", 0.42, 0.66),
        ("Reactor Stability", "100 -> 88 (-12)", 0.78, 0.68),
    ];
    for (name, delta, base_v, total_v) in data {
        let c = g.card(rows, sz(1176.0, 112.0), &format!("{} compare", name));
        g.heading_l(c, tl(24.0, 14.0, 420.0, 34.0), &format!("{} name", name), name, 26.0);
        g.value(c, tr(24.0, 14.0, 420.0, 34.0), &format!("{} delta", name), delta, 24.0);
        g.bar(c, tl(24.0, 58.0, 1128.0, 16.0), &format!("{} base", name), base_v, g.t.muted);
        g.bar(c, tl(24.0, 82.0, 1128.0, 16.0), &format!("{} equipped", name), total_v, g.t.accent);
    }
    // Panel lateral derecho
    let side = g.panel(p, right(50.0, 550.0, 126.0, 50.0), "Summary column");
    g.art(side, tl(28.0, 28.0, 494.0, 340.0), "Suit diagram");
    let info = g.vbox(side, tl(28.0, 392.0, 494.0, 360.0), "Suit specs", 14.0);
    for (k, v) in [("Power Draw", "84 / 100 MW"), ("Mass", "1.42 Tons"), ("Cooling", "Optimal"), ("Active Set", "Valkyrie 3/4")] {
        kv_row(g, info, 494.0, 76.0, k, v);
    }
    g.button(side, bl(28.0, 28.0, 494.0, 72.0), "Tune modules", "Tune Modules");
}

/// Hoja de estadísticas en dos columnas organizadas por secciones temáticas.
fn overview_two_column_sheet(g: &mut G) {
    let p = g.page();
    g.background();
    let head = g.panel(p, top(28.0, 124.0, 60.0, 60.0), "Character header");
    g.art(head, cl(24.0, 0.0, 88.0, 88.0), "Hero crest");
    g.heading_l(head, tl(132.0, 22.0, 640.0, 46.0), "Sheet hero name", "Linden Oakhart", 36.0);
    g.note(head, tl(132.0, 72.0, 640.0, 34.0), "Sheet hero sub", "Level 32 Druid  |  Grove Keeper", 24.0);
    g.badge(head, cr(28.0, 0.0, 220.0, 48.0), "Playtime badge", "34h 12m PLAYED");
    // Dos columnas de secciones
    let left_c = g.panel(p, left(60.0, 880.0, 176.0, 40.0), "Left stats column");
    let right_c = g.panel(p, right(60.0, 880.0, 176.0, 40.0), "Right stats column");
    // Columna izquierda: Offense & Defense
    g.heading_l(left_c, tl(28.0, 24.0, 400.0, 40.0), "Offense header", "OFFENSE", 28.0);
    let off = g.vbox(left_c, tl(28.0, 76.0, 824.0, 340.0), "Offense rows", 12.0);
    for (k, v) in [("Spell Power", "286"), ("Weapon Damage", "114 - 138"), ("Critical Rate", "19.5%"), ("Cast Speed", "1.24 /s")] {
        kv_row(g, off, 824.0, 74.0, k, v);
    }
    g.heading_l(left_c, tl(28.0, 444.0, 400.0, 40.0), "Defense header", "DEFENSE", 28.0);
    let def = g.vbox(left_c, tl(28.0, 496.0, 824.0, 340.0), "Defense rows", 12.0);
    for (k, v) in [("Max Health", "1,420"), ("Armor Rating", "218"), ("Evasion", "14.0%"), ("Health Regen", "18 /s")] {
        kv_row(g, def, 824.0, 74.0, k, v);
    }
    // Columna derecha: Resistances & Exploration
    g.heading_l(right_c, tl(28.0, 24.0, 400.0, 40.0), "Resist header", "RESISTANCES", 28.0);
    let res = g.vbox(right_c, tl(28.0, 76.0, 824.0, 340.0), "Resistance rows", 12.0);
    for (k, v) in [("Fire Resistance", "35%"), ("Frost Resistance", "42%"), ("Poison Resistance", "68%"), ("Shadow Resistance", "20%")] {
        kv_row(g, res, 824.0, 74.0, k, v);
    }
    g.heading_l(right_c, tl(28.0, 444.0, 400.0, 40.0), "Utility header", "EXPLORATION", 28.0);
    let util = g.vbox(right_c, tl(28.0, 496.0, 824.0, 340.0), "Utility rows", 12.0);
    for (k, v) in [("Move Speed", "+12%"), ("Foraging Yield", "+30%"), ("Carry Capacity", "95 kg"), ("Discovery Bonus", "+15% XP")] {
        kv_row(g, util, 824.0, 74.0, k, v);
    }
}

// ===============================================================================================================
// 5. Ability list
// ===============================================================================================================

/// Lista vertical de habilidades con icono, coste, descripción e insignia «Equipped».
fn ability_list_rows(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "SPELLBOOK & ABILITIES", "Equipped: 4 / 4   Mana: 260");
    let list_p = g.panel(p, left(50.0, 1220.0, 126.0, 44.0), "Ability list panel");
    let rows = g.vbox(list_p, fill(28.0, 28.0, 28.0, 28.0), "Abilities stack", 16.0);
    let abilities = [
        ("EB", "Ember Bolt", "18 Mana  |  2s CD", "Launches a blazing bolt that scorches a single target.", true),
        ("GW", "Gale Ward", "30 Mana  |  10s CD", "Surrounds the caster in wind, deflecting incoming arrows.", true),
        ("SN", "Solar Nova", "55 Mana  |  18s CD", "Releases a ring of light that blinds and burns nearby foes.", true),
        ("MS", "Mist Step", "22 Mana  |  8s CD", "Blink forward a short distance leaving a decoy behind.", true),
        ("IR", "Iron Roots", "28 Mana  |  12s CD", "Binds enemies in place with thorny vines for 3 seconds.", false),
        ("FP", "Frost Pulse", "34 Mana  |  9s CD", "Chills an area and slows enemy movement by 40%.", false),
    ];
    for (gly, name, cost, desc, eq) in abilities {
        let c = if eq {
            g.card_on(rows, sz(1164.0, 128.0), "Ability card")
        } else {
            g.card(rows, sz(1164.0, 128.0), "Ability card")
        };
        g.icon(c, cl(20.0, 0.0, 76.0, 76.0), "Ability icon", gly);
        g.heading_l(c, tl(116.0, 18.0, 440.0, 40.0), "Ability name", name, 30.0);
        g.note(c, tl(116.0, 64.0, 760.0, 38.0), "Ability desc", desc, 22.0);
        g.value(c, tr(200.0, 22.0, 260.0, 36.0), "Ability cost", cost, 22.0);
        if eq {
            g.badge(c, cr(24.0, 0.0, 152.0, 46.0), "Equipped tag", "EQUIPPED");
        } else {
            g.button2(c, cr(24.0, 0.0, 152.0, 68.0), "Equip button", "Equip");
        }
    }
    // Resumen de la barra activa a la derecha
    let side = g.panel(p, right(50.0, 570.0, 126.0, 44.0), "Selected ability panel");
    g.heading_l(side, tl(28.0, 28.0, 514.0, 44.0), "Focus title", "Ember Bolt - Rank 3", 32.0);
    g.art(side, tl(28.0, 92.0, 514.0, 240.0), "Ability preview");
    let specs = g.vbox(side, tl(28.0, 356.0, 514.0, 330.0), "Ability stats", 12.0);
    for (k, v) in [("Base Damage", "145 Fire"), ("Cast Time", "0.6 sec"), ("Range", "24 meters"), ("Upgrade Cost", "1 Skill Point")] {
        kv_row(g, specs, 514.0, 72.0, k, v);
    }
    g.button(side, bl(28.0, 28.0, 514.0, 72.0), "Upgrade ability", "Upgrade to Rank 4");
}

/// Árbol de habilidades con nodos conectados por líneas y detalle del nodo elegido abajo.
fn ability_list_tree(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "NEURAL SKILL TREE", "Skill Points: 4");
    let canvas = g.panel(p, fill(50.0, 126.0, 50.0, 250.0), "Tree canvas");
    // Líneas conectoras horizontales y verticales
    g.rect(canvas, tl(310.0, 344.0, 1220.0, 8.0), "Main branch line", g.t.line, 2.0);
    g.rect(canvas, tl(620.0, 160.0, 8.0, 370.0), "Branch split 1", g.t.line, 2.0);
    g.rect(canvas, tl(1160.0, 160.0, 8.0, 370.0), "Branch split 2", g.t.line, 2.0);
    g.rect(canvas, tl(620.0, 160.0, 910.0, 8.0), "Upper branch line", g.t.line, 2.0);
    g.rect(canvas, tl(620.0, 526.0, 910.0, 8.0), "Lower branch line", g.t.line, 2.0);
    // Nodos del árbol (x, y, código, título, estado, seleccionado)
    let nodes = [
        (80.0, 276.0, "01", "Core Surge", "MAX", true),
        (510.0, 92.0, "A1", "Overclock", "2/3", false),
        (510.0, 276.0, "B1", "Arc Dash", "1/1", false),
        (510.0, 460.0, "C1", "Hard Shell", "1/3", false),
        (1050.0, 92.0, "A2", "Chain Bolt", "0/2", false),
        (1050.0, 276.0, "B2", "Phase Step", "1/2", true),
        (1050.0, 460.0, "C2", "Reflector", "0/2", false),
        (1460.0, 92.0, "A3", "Storm Core", "LOCK", false),
        (1460.0, 276.0, "B3", "Zero Blink", "LOCK", false),
        (1460.0, 460.0, "C3", "Aegis Field", "LOCK", false),
    ];
    for (x, y, code, name, rank, on) in nodes {
        let n = if on {
            g.card_on(canvas, tl(x, y, 270.0, 144.0), &format!("{} node", name))
        } else {
            g.card(canvas, tl(x, y, 270.0, 144.0), &format!("{} node", name))
        };
        g.icon(n, tl(16.0, 20.0, 56.0, 56.0), "Node icon", code);
        g.heading_l(n, tl(86.0, 24.0, 170.0, 38.0), "Node name", name, 24.0);
        g.badge(n, bl(16.0, 16.0, 110.0, 40.0), "Node rank", rank);
    }
    // Panel inferior de detalle
    let foot = g.panel(p, bottom(30.0, 194.0, 50.0, 50.0), "Selected node detail");
    g.heading_l(foot, tl(32.0, 24.0, 520.0, 44.0), "Node detail name", "Phase Step (Rank 1 / 2)", 32.0);
    g.label(foot, tl(32.0, 82.0, 1100.0, 42.0), "Node detail desc", "Dash leaves an ionized trail that deals 80 shock damage and resets on takedown.", 24.0);
    g.note(foot, tl(32.0, 134.0, 760.0, 34.0), "Node req", "Requires: Arc Dash   |   Cost: 1 Skill Point", 22.0);
    g.button(foot, cr(32.0, 0.0, 280.0, 76.0), "Unlock node button", "Unlock Rank 2");
}

/// Rejilla de habilidades arriba con detalle lateral y barra de 4 huecos equipados abajo.
fn ability_list_grid_bar(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "ABILITY LOADOUT", "Active Slots: 4");
    let grid_p = g.panel(p, left(50.0, 1160.0, 126.0, 230.0), "Skills grid panel");
    let grid = g.grid(grid_p, cc(0.0, 0.0, 1080.0, 660.0), "Skills grid", 340.0, 200.0, 26.0);
    let skills = [
        ("SW", "Swift Slash", "0s CD", true),
        ("SH", "Shield Bash", "6s CD", false),
        ("WH", "Whirlwind", "10s CD", false),
        ("LP", "Leap Strike", "12s CD", false),
        ("WR", "War Cry", "18s CD", false),
        ("PR", "Parry Stance", "5s CD", false),
        ("RV", "Rending Cut", "8s CD", false),
        ("GR", "Ground Slam", "14s CD", false),
        ("ST", "Second Wind", "25s CD", false),
    ];
    for (code, name, cd, sel) in skills {
        let c = if sel {
            g.card_on(grid, sz(340.0, 200.0), "Skill tile")
        } else {
            g.card(grid, sz(340.0, 200.0), "Skill tile")
        };
        g.icon(c, tl(20.0, 20.0, 64.0, 64.0), "Skill icon", code);
        g.heading_l(c, tl(100.0, 26.0, 224.0, 40.0), "Skill title", name, 26.0);
        g.note(c, tl(100.0, 70.0, 224.0, 32.0), "Skill cd", cd, 22.0);
        g.badge(c, bl(20.0, 20.0, 120.0, 42.0), "Skill type", "MELEE");
    }
    // Detalle lateral derecho
    let side = g.panel(p, right(50.0, 630.0, 126.0, 230.0), "Skill detail panel");
    g.heading_l(side, tl(28.0, 28.0, 574.0, 46.0), "Detail name", "Swift Slash", 36.0);
    g.note(side, tl(28.0, 80.0, 574.0, 34.0), "Detail meta", "Primary Attack  |  Generates +12 Fury", 24.0);
    g.art(side, tl(28.0, 134.0, 574.0, 220.0), "Skill visual");
    g.label(side, tl(28.0, 378.0, 574.0, 96.0), "Detail text", "Chain three rapid cuts dealing 110% weapon damage each. Final hit cleaves in a cone.", 24.0);
    g.button(side, bl(28.0, 28.0, 270.0, 72.0), "Assign slot", "Assign to Slot 1");
    g.button2(side, br(28.0, 28.0, 270.0, 72.0), "Clear slot", "Clear Slot");
    // Barra inferior de 4 huecos equipados
    let bar = g.panel(p, bottom(30.0, 176.0, 50.0, 50.0), "Hotbar panel");
    g.heading_l(bar, cl(30.0, 0.0, 300.0, 44.0), "Bar title", "Equipped Bar", 30.0);
    let slots = g.hbox_c(bar, cc(140.0, 0.0, 1240.0, 132.0), "Equipped slots", 24.0);
    for (k, name, code) in [("1", "Swift Slash", "SW"), ("2", "Shield Bash", "SH"), ("3", "Leap Strike", "LP"), ("4", "War Cry", "WR")] {
        let s = g.card(slots, sz(290.0, 126.0), "Hotbar slot");
        g.key(s, tl(14.0, 14.0, 46.0, 46.0), "Slot key", k);
        g.icon(s, cl(74.0, 0.0, 56.0, 56.0), "Slot icon", code);
        g.heading_l(s, tl(142.0, 42.0, 136.0, 40.0), "Slot name", name, 22.0);
    }
}

// ===============================================================================================================
// 6. Maintenance & management
// ===============================================================================================================

/// Base con salas en rejilla, barras de estado de integridad y botones de mejora.
fn maint_base_rooms(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "OUTPOST FACILITIES", "Power: 82 / 100   Steel: 1,450   Fuel: 620");
    let wrap = g.panel(p, fill(50.0, 126.0, 50.0, 40.0), "Base map panel");
    let grid = g.grid(wrap, cc(0.0, 0.0, 1760.0, 840.0), "Rooms grid", 566.0, 400.0, 28.0);
    let rooms = [
        ("Command Center", "Lv 4", 0.96, "Operational", "400 Steel", true),
        ("Power Reactor", "Lv 3", 0.72, "High Load", "320 Steel", false),
        ("Med Bay", "Lv 3", 0.88, "2 Beds Free", "250 Steel", false),
        ("Armory Forge", "Lv 2", 0.54, "Needs Repair", "180 Steel", false),
        ("Radar Array", "Lv 2", 0.90, "Scanning Sector", "290 Steel", false),
        ("Hydroponics", "Lv 1", 0.40, "Low Output", "150 Steel", false),
    ];
    for (name, lv, cond, st, cost, sel) in rooms {
        let c = if sel {
            g.card_on(grid, sz(566.0, 400.0), "Room card")
        } else {
            g.card(grid, sz(566.0, 400.0), "Room card")
        };
        g.art(c, tl(20.0, 20.0, 526.0, 140.0), "Room schematic");
        g.heading_l(c, tl(20.0, 176.0, 380.0, 40.0), "Room name", name, 28.0);
        g.badge(c, tr(20.0, 176.0, 108.0, 42.0), "Room level", lv);
        g.note(c, tl(20.0, 224.0, 320.0, 32.0), "Room status", st, 22.0);
        g.value(c, tr(20.0, 224.0, 180.0, 32.0), "Room health", &format!("{}%", (cond * 100.0) as i32), 24.0);
        let col = if cond > 0.75 { g.t.good } else if cond > 0.5 { g.t.warn } else { g.t.bad };
        g.bar(c, tl(20.0, 266.0, 526.0, 18.0), "Condition bar", cond, col);
        g.note(c, bl(20.0, 36.0, 240.0, 34.0), "Upgrade cost", &format!("Next: {}", cost), 22.0);
        g.button(c, br(20.0, 20.0, 220.0, 68.0), "Upgrade room", "Upgrade");
    }
}

/// Pantalla de reparación de equipo: piezas con durabilidad, estado y coste total.
fn maint_repair_list(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "WORKBENCH REPAIR", "Scrap Parts: 420");
    let left_p = g.panel(p, left(50.0, 1220.0, 126.0, 44.0), "Gear list panel");
    let rows = g.vbox(left_p, fill(28.0, 28.0, 28.0, 28.0), "Repair rows", 16.0);
    let items = [
        ("Sawed-Off Shotgun", "CRITICAL", 0.22, "85 Scrap", true),
        ("Reinforced Trenchcoat", "WORN", 0.58, "45 Scrap", false),
        ("Storm Lantern", "DAMAGED", 0.36, "60 Scrap", false),
        ("Iron Crowbar", "GOOD", 0.86, "15 Scrap", false),
        ("Gas Mask Filter", "CRITICAL", 0.14, "95 Scrap", false),
        ("Tactical Boots", "WORN", 0.64, "30 Scrap", false),
    ];
    for (name, cond, frac, cost, sel) in items {
        let r = if sel {
            g.card_on(rows, sz(1164.0, 126.0), "Gear repair row")
        } else {
            g.card(rows, sz(1164.0, 126.0), "Gear repair row")
        };
        g.icon(r, cl(20.0, 0.0, 68.0, 68.0), "Gear icon", &name[..2]);
        g.heading_l(r, tl(108.0, 18.0, 460.0, 38.0), "Gear name", name, 28.0);
        g.note(r, tl(108.0, 58.0, 260.0, 30.0), "Gear state", &format!("Condition: {}", cond), 22.0);
        let col = if frac < 0.35 { g.t.bad } else if frac < 0.7 { g.t.warn } else { g.t.good };
        g.bar(r, tl(108.0, 94.0, 520.0, 16.0), "Durability bar", frac, col);
        g.value(r, tr(240.0, 44.0, 190.0, 38.0), "Repair cost", cost, 26.0);
        g.button2(r, cr(20.0, 0.0, 196.0, 68.0), "Repair single", "Repair");
    }
    let summary = g.panel(p, right(50.0, 570.0, 126.0, 44.0), "Repair summary");
    g.heading_l(summary, tl(28.0, 28.0, 514.0, 46.0), "Summary title", "Repair Estimate", 34.0);
    g.art(summary, tl(28.0, 94.0, 514.0, 250.0), "Workbench preview");
    let kvs = g.vbox(summary, tl(28.0, 368.0, 514.0, 310.0), "Estimate rows", 14.0);
    for (k, v) in [("Items Damaged", "5 / 6"), ("Total Scrap Cost", "330 Scrap"), ("Remaining Scrap", "90 Scrap"), ("Time Required", "Instant")] {
        kv_row(g, kvs, 514.0, 66.0, k, v);
    }
    g.button(summary, bl(28.0, 114.0, 514.0, 74.0), "Repair all button", "Repair All (330 Scrap)");
    g.button2(summary, bl(28.0, 28.0, 514.0, 68.0), "Leave bench", "Leave Workbench");
}

/// Tabla de personal con barras de moral, salario y selector desplegable de asignación.
fn maint_staff_table(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "CLUB STAFF MANAGEMENT", "Weekly Budget: $42,000 / $60,000");
    let top_sum = g.panel(p, top(116.0, 116.0, 50.0, 50.0), "Budget bar");
    let kpis = g.hbox_c(top_sum, cc(0.0, 0.0, 1740.0, 82.0), "Staff metrics", 24.0);
    for (k, v) in [("Active Staff", "5 / 8"), ("Average Morale", "82%"), ("Weekly Payroll", "$42,000"), ("Training Boost", "+18%")] {
        kv_row(g, kpis, 416.0, 76.0, k, v);
    }
    let table = g.panel(p, fill(50.0, 256.0, 50.0, 40.0), "Staff roster panel");
    let head = g.card(table, top(24.0, 60.0, 28.0, 28.0), "Staff table header");
    g.heading_l(head, tl(24.0, 14.0, 340.0, 34.0), "Col staff", "Staff Member", 24.0);
    g.heading_l(head, tl(440.0, 14.0, 360.0, 34.0), "Col morale", "Morale", 24.0);
    g.heading_l(head, tl(860.0, 14.0, 240.0, 34.0), "Col salary", "Salary", 24.0);
    g.heading_l(head, tl(1160.0, 14.0, 460.0, 34.0), "Col assign", "Department Assignment", 24.0);
    let list = g.vbox(table, fill(28.0, 102.0, 28.0, 96.0), "Staff list", 14.0);
    let staff = [
        ("Coach Martin Reyes", 0.92, "$12,500 /wk", &["First Team", "Youth Squad", "Rest"][..]),
        ("Dr. Hannah Osei", 0.84, "$9,200 /wk", &["Recovery Lab", "Field Medic", "Research"][..]),
        ("Lucas Novak", 0.76, "$7,800 /wk", &["Pro Scouting", "Youth Scout", "Analytics"][..]),
        ("Sofia Conti", 0.88, "$6,900 /wk", &["Strength Gym", "Cardio Track", "Recovery Lab"][..]),
        ("Devon Brooks", 0.68, "$5,600 /wk", &["Match Video", "Pro Scouting", "Analytics"][..]),
    ];
    for (name, morale, pay, opts) in staff {
        let r = g.card(list, sz(1764.0, 104.0), "Staff row");
        g.icon(r, cl(20.0, 0.0, 56.0, 56.0), "Staff avatar", &name[..2]);
        g.heading_l(r, tl(96.0, 32.0, 320.0, 40.0), "Staff name", name, 26.0);
        g.bar(r, tl(440.0, 44.0, 270.0, 18.0), "Morale bar", morale, if morale > 0.75 { g.t.good } else { g.t.warn });
        g.value(r, tl(720.0, 34.0, 90.0, 36.0), "Morale pct", &format!("{}%", (morale * 100.0) as i32), 24.0);
        g.label(r, tl(860.0, 34.0, 240.0, 36.0), "Staff salary", pay, 24.0);
        g.dropdown(r, cr(24.0, 0.0, 440.0, 66.0), "Assignment dropdown", opts);
    }
    g.button(table, br(28.0, 18.0, 280.0, 68.0), "Hire button", "Hire Specialist");
}

// ===============================================================================================================
// 7. Inventory grid
// ===============================================================================================================

/// Rejilla de huecos de inventario, detalle del objeto a la derecha y barra de peso abajo.
fn inventory_grid_detail(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "BACKPACK INVENTORY", "Gold: 8,640");
    let grid_p = g.panel(p, left(50.0, 1140.0, 126.0, 124.0), "Grid panel");
    let grid = g.grid(grid_p, cc(0.0, 0.0, 1060.0, 760.0), "Item slots", 196.0, 174.0, 20.0);
    let items = [
        ("SW", "x1", true), ("SH", "x1", false), ("HP", "x6", false), ("MP", "x4", false), ("KY", "x2", false),
        ("OR", "x12", false), ("HR", "x9", false), ("RG", "x1", false), ("AR", "x24", false), ("GM", "x3", false),
        ("TR", "x5", false), ("FD", "x8", false), ("", "", false), ("", "", false), ("", "", false),
        ("", "", false), ("", "", false), ("", "", false), ("", "", false), ("", "", false),
    ];
    for (gly, qty, sel) in items {
        let s = if sel {
            g.card_on(grid, sz(196.0, 174.0), "Inventory slot")
        } else {
            g.slot(grid, sz(196.0, 174.0), "Inventory slot")
        };
        if !gly.is_empty() {
            g.icon(s, cc(0.0, -16.0, 68.0, 68.0), "Item icon", gly);
            g.badge(s, br(10.0, 10.0, 74.0, 40.0), "Item qty", qty);
        }
    }
    // Panel de detalle derecho
    let detail = g.panel(p, right(50.0, 650.0, 126.0, 124.0), "Item detail");
    g.art(detail, tl(28.0, 28.0, 594.0, 240.0), "Item art");
    g.heading_l(detail, tl(28.0, 290.0, 420.0, 44.0), "Item name", "Sunforged Blade", 34.0);
    g.badge(detail, tr(28.0, 290.0, 130.0, 44.0), "Rarity badge", "RARE");
    g.note(detail, tl(28.0, 344.0, 594.0, 96.0), "Item desc", "Balanced steel sword tempered with ember dust. Deals bonus fire damage at dusk.", 24.0);
    let specs = g.vbox(detail, tl(28.0, 456.0, 594.0, 240.0), "Item specs", 12.0);
    for (k, v) in [("Damage", "68 - 84"), ("Weight", "4.5 kg"), ("Sell Value", "1,250 Gold")] {
        kv_row(g, specs, 594.0, 70.0, k, v);
    }
    let acts = g.hbox(detail, bl(28.0, 28.0, 594.0, 72.0), "Item actions", 18.0);
    g.button(acts, sz(288.0, 70.0), "Equip item", "Equip");
    g.button2(acts, sz(288.0, 70.0), "Drop item", "Drop");
    // Barra de peso abajo
    let wbar = g.panel(p, bottom(30.0, 76.0, 50.0, 50.0), "Weight bar panel");
    g.heading_l(wbar, cl(28.0, 0.0, 260.0, 38.0), "Weight title", "Carry Weight", 26.0);
    g.bar(wbar, cc(20.0, 0.0, 1140.0, 22.0), "Weight meter", 0.64, g.t.warn);
    g.value(wbar, cr(28.0, 0.0, 240.0, 38.0), "Weight value", "48.2 / 75.0 kg", 26.0);
}

/// Lista de objetos organizada por pestañas con menú desplegable de ordenamiento.
fn inventory_tabbed_list(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "STORAGE CATALOG", "Slots Used: 19 / 80");
    let ctrl = g.panel(p, top(116.0, 102.0, 50.0, 50.0), "Filter toolbar");
    let tabs = g.hbox(ctrl, cl(24.0, 0.0, 1020.0, 70.0), "Category tabs", 14.0);
    for (i, t) in ["All", "Weapons", "Armor", "Consumables", "Materials"].iter().enumerate() {
        if i == 0 {
            g.button(tabs, sz(192.0, 68.0), &format!("{} tab", t), t);
        } else {
            g.button2(tabs, sz(192.0, 68.0), &format!("{} tab", t), t);
        }
    }
    g.dropdown(ctrl, cr(24.0, 0.0, 380.0, 68.0), "Sort dropdown", &["Sort: Weight", "Sort: Value", "Sort: Name", "Sort: Quantity"]);
    let list_p = g.panel(p, fill(50.0, 238.0, 50.0, 40.0), "Inventory list panel");
    let rows = g.vbox(list_p, fill(28.0, 24.0, 28.0, 24.0), "Item rows", 14.0);
    let data = [
        ("Field Medkit", "Consumable", "x4", "0.8 kg", "180 G", true),
        ("Carbon Repeater", "Weapon", "x1", "3.4 kg", "1,400 G", false),
        ("Alloy Vest", "Armor", "x1", "5.2 kg", "950 G", false),
        ("Energy Cell", "Material", "x18", "1.8 kg", "360 G", false),
        ("Signal Flare", "Consumable", "x6", "0.6 kg", "120 G", false),
        ("Titanium Plate", "Material", "x9", "4.5 kg", "540 G", false),
    ];
    for (name, cat, qty, wt, val, sel) in data {
        let r = if sel {
            g.card_on(rows, sz(1764.0, 110.0), "List item row")
        } else {
            g.card(rows, sz(1764.0, 110.0), "List item row")
        };
        g.icon(r, cl(20.0, 0.0, 60.0, 60.0), "Row icon", &name[..2]);
        g.heading_l(r, tl(100.0, 20.0, 440.0, 38.0), "Row item name", name, 28.0);
        g.note(r, tl(100.0, 62.0, 320.0, 30.0), "Row category", cat, 22.0);
        g.label(r, tl(740.0, 36.0, 140.0, 36.0), "Row qty", qty, 26.0);
        g.note(r, tl(940.0, 36.0, 180.0, 36.0), "Row weight", wt, 24.0);
        g.value(r, tl(1160.0, 36.0, 220.0, 36.0), "Row value", val, 26.0);
        g.button2(r, cr(24.0, 0.0, 190.0, 68.0), "Use button", "Use");
    }
}

/// Inventario compacto de 2x4 huecos grandes, equipo activo a la izquierda y vista previa abajo.
fn inventory_compact_slots(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "FIELD PACK", "8 Pockets  |  Low Space");
    // Izquierda: equipado en manos
    let eq_p = g.panel(p, left(50.0, 480.0, 126.0, 44.0), "Equipped panel");
    g.heading_l(eq_p, tl(24.0, 24.0, 432.0, 44.0), "Hands title", "In Hands", 32.0);
    let eq_list = g.vbox(eq_p, tl(24.0, 88.0, 432.0, 560.0), "Equipped slots", 18.0);
    for (slot_t, item_t, gly) in [("Main Hand", "Revolver .38 (5/6)", "RV"), ("Off Hand", "Brass Flashlight", "FL"), ("Body", "Canvas Coat", "CT")] {
        let c = g.card(eq_list, sz(432.0, 170.0), &format!("{} card", slot_t));
        g.note(c, tl(20.0, 16.0, 220.0, 30.0), "Slot type", slot_t, 22.0);
        g.icon(c, tl(20.0, 66.0, 74.0, 74.0), "Equipped icon", gly);
        g.heading_l(c, tl(112.0, 82.0, 300.0, 42.0), "Equipped name", item_t, 26.0);
    }
    g.button2(eq_p, bl(24.0, 24.0, 432.0, 72.0), "Combine button", "Combine Selected");
    // Derecha arriba: 2x4 huecos grandes
    let pack = g.panel(p, fill(560.0, 126.0, 50.0, 360.0), "Pack pockets panel");
    let grid = g.grid(pack, cc(0.0, 0.0, 1220.0, 520.0), "Pocket grid", 286.0, 242.0, 24.0);
    let pockets = [
        ("BD", "Bandage x2", true),
        ("AM", "Shells x12", false),
        ("KY", "Cellar Key", false),
        ("BT", "Battery x1", false),
        ("NT", "Torn Note", false),
        ("HR", "Red Herb", false),
        ("", "Empty", false),
        ("", "Empty", false),
    ];
    for (gly, label, sel) in pockets {
        let c = if sel {
            g.card_on(grid, sz(286.0, 242.0), "Pocket slot")
        } else {
            g.card(grid, sz(286.0, 242.0), "Pocket slot")
        };
        if !gly.is_empty() {
            g.icon(c, tc(0.0, 34.0, 84.0, 84.0), "Pocket icon", gly);
            g.heading(c, bottom(28.0, 44.0, 16.0, 16.0), "Pocket label", label, 26.0);
        } else {
            g.txt(c, cc(0.0, 0.0, 220.0, 40.0), "Empty pocket", "Empty", 24.0, g.t.muted, CENTER, false);
        }
    }
    // Derecha abajo: vista previa del objeto
    let prev = g.panel(p, bottom(44.0, 290.0, 560.0, 50.0), "Item preview panel");
    g.art(prev, cl(28.0, 0.0, 230.0, 230.0), "Selected item art");
    g.heading_l(prev, tl(284.0, 28.0, 540.0, 44.0), "Preview item title", "Clean Bandage (x2)", 34.0);
    g.note(prev, tl(284.0, 84.0, 680.0, 80.0), "Preview item desc", "Stops bleeding immediately and restores 35 health. Takes 3 seconds to apply.", 24.0);
    let acts = g.hbox(prev, bl(284.0, 28.0, 560.0, 72.0), "Preview actions", 20.0);
    g.button(acts, sz(260.0, 70.0), "Use bandage", "Use Now");
    g.button2(acts, sz(260.0, 70.0), "Discard item", "Discard");
}

// ===============================================================================================================
// 8. Equipping
// ===============================================================================================================

/// Muñeco centrado con huecos alrededor y panel de cambios de atributos a la derecha.
fn equipping_paperdoll(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "EQUIPMENT & ATTIRE", "Gear Score: 642");
    let doll = g.panel(p, left(50.0, 1180.0, 126.0, 44.0), "Paperdoll panel");
    g.art(doll, cc(0.0, 0.0, 460.0, 760.0), "Character silhouette");
    // 3 huecos izquierda y 3 derecha
    let left_slots = g.vbox(doll, cl(36.0, 0.0, 300.0, 620.0), "Left gear slots", 28.0);
    for (slot_t, item_t, on) in [("HEAD", "Silver Circlet", true), ("CHEST", "Velvet Cuirass", false), ("HANDS", "Duelist Gloves", false)] {
        let c = if on { g.card_on(left_slots, sz(300.0, 186.0), "Left slot") } else { g.card(left_slots, sz(300.0, 186.0), "Left slot") };
        g.badge(c, tl(16.0, 16.0, 120.0, 40.0), "Slot tag", slot_t);
        g.heading_l(c, tl(16.0, 76.0, 268.0, 40.0), "Gear item", item_t, 26.0);
        g.note(c, bl(16.0, 16.0, 268.0, 32.0), "Gear tier", "Tier IV  +18", 22.0);
    }
    let right_slots = g.vbox(doll, cr(36.0, 0.0, 300.0, 620.0), "Right gear slots", 28.0);
    for (slot_t, item_t) in [("WEAPON", "Gilded Rapier"), ("FEET", "Strider Boots"), ("CHARM", "Opal Locket")] {
        let c = g.card(right_slots, sz(300.0, 186.0), "Right slot");
        g.badge(c, tl(16.0, 16.0, 124.0, 40.0), "Slot tag", slot_t);
        g.heading_l(c, tl(16.0, 76.0, 268.0, 40.0), "Gear item", item_t, 26.0);
        g.note(c, bl(16.0, 16.0, 268.0, 32.0), "Gear tier", "Tier IV  +22", 22.0);
    }
    // Panel derecho de cambios de stats
    let stats_p = g.panel(p, right(50.0, 610.0, 126.0, 44.0), "Stat changes panel");
    g.heading_l(stats_p, tl(28.0, 28.0, 554.0, 46.0), "Preview title", "Attribute Impact", 34.0);
    g.note(stats_p, tl(28.0, 80.0, 554.0, 34.0), "Swapping note", "Comparing: Crown of Dusk vs Silver Circlet", 22.0);
    let rows = g.vbox(stats_p, tl(28.0, 134.0, 554.0, 530.0), "Impact list", 14.0);
    for (k, v) in [
        ("Armor", "240 -> 262 (+22)"),
        ("Magic Power", "188 -> 215 (+27)"),
        ("Crit Chance", "16% -> 14% (-2%)"),
        ("Mana Regen", "12/s -> 15/s (+3)"),
        ("Weight", "32.0 -> 34.5 (+2.5)"),
        ("Poise", "55 -> 60 (+5)"),
    ] {
        kv_row(g, rows, 554.0, 76.0, k, v);
    }
    let btns = g.hbox(stats_p, bl(28.0, 28.0, 554.0, 72.0), "Equip actions", 18.0);
    g.button(btns, sz(268.0, 70.0), "Equip confirm", "Equip Crown");
    g.button2(btns, sz(268.0, 70.0), "Unequip button", "Unequip");
}

/// Tres columnas: huecos equipados, candidatos disponibles y comparación (+/-).
fn equipping_three_col(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "ARMORY LOADOUT", "Slot: Primary Weapon");
    // Columna 1: Huecos
    let c1 = g.panel(p, left(40.0, 440.0, 126.0, 40.0), "Slots column");
    g.heading_l(c1, tl(24.0, 22.0, 392.0, 40.0), "Slots heading", "1. Gear Slots", 28.0);
    let s_list = g.vbox(c1, tl(24.0, 78.0, 392.0, 790.0), "Slot buttons", 14.0);
    for (i, (slot_t, cur)) in [
        ("Primary", "M4 Carbine"),
        ("Sidearm", "K9 Pistol"),
        ("Helmet", "Tactical Visor"),
        ("Vest", "Plate Carrier"),
        ("Gadget", "Recon Drone"),
        ("Grenade", "Flashbang x2"),
    ]
    .iter()
    .enumerate()
    {
        let card = if i == 0 { g.card_on(s_list, sz(392.0, 118.0), "Slot card") } else { g.card(s_list, sz(392.0, 118.0), "Slot card") };
        g.note(card, tl(18.0, 16.0, 356.0, 30.0), "Slot label", slot_t, 22.0);
        g.heading_l(card, tl(18.0, 54.0, 356.0, 40.0), "Current gear", cur, 26.0);
    }
    // Columna 2: Candidatos
    let c2 = g.panel(p, fill(506.0, 126.0, 686.0, 40.0), "Candidates column");
    g.heading_l(c2, tl(24.0, 22.0, 600.0, 40.0), "Candidates heading", "2. Available Weapons", 28.0);
    let cand = g.vbox(c2, tl(24.0, 78.0, 680.0, 790.0), "Candidate list", 14.0);
    for (i, (name, sub, pwr)) in [
        ("VX-7 Marksman Rifle", "Semi-Auto  |  Long Range", "PWR 84"),
        ("M4 Carbine (Equipped)", "Full-Auto  |  Balanced", "PWR 78"),
        ("Kestrel SMG", "High RPM  |  Close Range", "PWR 74"),
        ("Bulldog Breacher", "Pump  |  High Impact", "PWR 80"),
        ("Valkyrie LMG", "Heavy  |  100-Round Belt", "PWR 82"),
    ]
    .iter()
    .enumerate()
    {
        let card = if i == 0 { g.card_on(cand, sz(680.0, 142.0), "Candidate card") } else { g.card(cand, sz(680.0, 142.0), "Candidate card") };
        g.icon(card, cl(18.0, 0.0, 68.0, 68.0), "Weapon icon", &name[..2]);
        g.heading_l(card, tl(104.0, 26.0, 420.0, 40.0), "Candidate name", name, 26.0);
        g.note(card, tl(104.0, 72.0, 420.0, 34.0), "Candidate sub", sub, 22.0);
        g.badge(card, cr(18.0, 0.0, 130.0, 44.0), "Candidate power", pwr);
    }
    // Columna 3: Comparación
    let c3 = g.panel(p, right(40.0, 620.0, 126.0, 40.0), "Comparison column");
    g.heading_l(c3, tl(28.0, 22.0, 564.0, 40.0), "Compare heading", "3. Comparison (+/-)", 28.0);
    g.art(c3, tl(28.0, 78.0, 564.0, 200.0), "Weapon render");
    let cmp = g.vbox(c3, tl(28.0, 300.0, 564.0, 460.0), "Compare rows", 14.0);
    for (k, v) in [
        ("Damage", "42 -> 64 (+22)"),
        ("Fire Rate", "750 -> 420 (-330)"),
        ("Accuracy", "72 -> 91 (+19)"),
        ("Range", "55 -> 88 (+33)"),
        ("Mobility", "80 -> 66 (-14)"),
    ] {
        kv_row(g, cmp, 564.0, 80.0, k, v);
    }
    g.button(c3, bl(28.0, 24.0, 564.0, 72.0), "Equip weapon", "Equip VX-7 Rifle");
}

/// Dos paneles de armas (principal y secundaria) arriba y fila de cuatro ventajas abajo.
fn equipping_loadout_perks(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "MATCH LOADOUT", "Class: Striker Kit #2");
    let weapons = g.hbox_c(p, top(126.0, 450.0, 50.0, 50.0), "Weapons row", 28.0);
    for (slot_t, name, desc, dmg, acc, spd) in [
        ("PRIMARY GEAR", "Aero Carbon Racket", "Lightweight frame built for fast top-spin volleys.", 0.84, 0.90, 0.78),
        ("SECONDARY GEAR", "Grip Pro Footwear", "High-traction sole for rapid lateral court cuts.", 0.72, 0.82, 0.94),
    ] {
        let wp = g.panel(weapons, sz(896.0, 444.0), &format!("{} panel", slot_t));
        g.badge(wp, tl(28.0, 24.0, 220.0, 44.0), "Weapon slot tag", slot_t);
        g.button2(wp, tr(28.0, 20.0, 180.0, 68.0), "Change gear", "Swap");
        g.art(wp, tl(28.0, 100.0, 320.0, 220.0), "Gear preview");
        g.heading_l(wp, tl(372.0, 102.0, 496.0, 44.0), "Gear title", name, 32.0);
        g.note(wp, tl(372.0, 152.0, 496.0, 70.0), "Gear note", desc, 22.0);
        let bars = g.vbox(wp, tl(372.0, 232.0, 496.0, 186.0), "Gear bars", 10.0);
        stat_bar_row(g, bars, 496.0, "Power", &format!("{}", (dmg * 100.0) as i32), dmg, g.t.accent);
        stat_bar_row(g, bars, 496.0, "Control", &format!("{}", (acc * 100.0) as i32), acc, g.t.good);
        g.bar(wp, bl(28.0, 32.0, 320.0, 18.0), "Speed bar", spd, g.t.warn);
        g.note(wp, bl(28.0, 58.0, 320.0, 30.0), "Speed note", &format!("Agility Rating: {}", (spd * 100.0) as i32), 22.0);
    }
    // Ventajas abajo
    let perks_p = g.panel(p, bottom(116.0, 340.0, 50.0, 50.0), "Perks panel");
    g.heading_l(perks_p, tl(28.0, 22.0, 500.0, 40.0), "Perks heading", "Equipped Perks (4 / 4)", 30.0);
    let prow = g.hbox_c(perks_p, bottom(24.0, 240.0, 28.0, 28.0), "Perk cards", 24.0);
    for (code, name, desc) in [
        ("SP", "Quick Sprint", "+12% burst speed after a serve."),
        ("ST", "Iron Stamina", "Lose 20% less stamina on long rallies."),
        ("FC", "Clutch Focus", "+15% accuracy on match points."),
        ("RC", "Fast Recovery", "Restore energy faster between sets."),
    ] {
        let c = g.card(prow, sz(422.0, 230.0), "Perk card");
        g.icon(c, tl(20.0, 20.0, 64.0, 64.0), "Perk icon", code);
        g.heading_l(c, tl(100.0, 30.0, 304.0, 40.0), "Perk name", name, 26.0);
        g.note(c, tl(20.0, 104.0, 382.0, 96.0), "Perk desc", desc, 22.0);
    }
    let foot = g.panel(p, bottom(0.0, 94.0, 0.0, 0.0), "Loadout footer");
    g.button2(foot, cl(50.0, 0.0, 240.0, 68.0), "Reset loadout", "Reset Defaults");
    g.button(foot, cr(50.0, 0.0, 280.0, 68.0), "Confirm loadout", "Confirm Loadout");
}

// ===============================================================================================================
// 9. Buying & trading
// ===============================================================================================================

/// Tienda con retrato del mercader, globo de diálogo, lista con precios y botón de compra.
fn trading_merchant_shop(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "PIP'S GENERAL STORE", "Your Gold: 2,480 G");
    // Izquierda: mercader
    let merch = g.panel(p, left(50.0, 560.0, 126.0, 44.0), "Merchant panel");
    g.art(merch, tl(28.0, 28.0, 504.0, 430.0), "Merchant portrait");
    let speech = g.card(merch, tl(28.0, 480.0, 504.0, 180.0), "Merchant dialog");
    g.heading_l(speech, tl(20.0, 16.0, 464.0, 38.0), "Merchant name", "Pip the Tinker", 28.0);
    g.label(speech, tl(20.0, 62.0, 464.0, 98.0), "Merchant quote", "\"Fresh potions and sturdy boots! Pick what you need before dusk.\"", 24.0);
    let mode_row = g.hbox(merch, bl(28.0, 28.0, 504.0, 72.0), "Shop mode tabs", 16.0);
    let buy_tab = g.button(mode_row, sz(244.0, 70.0), "Buy tab", "Buy");
    big_accent(g, buy_tab);
    g.button2(mode_row, sz(244.0, 70.0), "Sell tab", "Sell");
    // Derecha: lista de mercancía
    let shop = g.panel(p, fill(640.0, 126.0, 50.0, 44.0), "Goods panel");
    let list = g.vbox(shop, tl(28.0, 28.0, 1174.0, 710.0), "Shop items", 14.0);
    let wares = [
        ("HP", "Berry Potion", "Restores 120 Health", "120 G", "Stock: 9", true),
        ("BT", "Bouncy Boots", "+15% Jump Height", "640 G", "Stock: 1", false),
        ("LP", "Brass Lockpick", "Opens locked chests", "85 G", "Stock: 5", false),
        ("CR", "Camp Ration", "Full party stamina", "150 G", "Stock: 4", false),
        ("BM", "Spark Bomb", "Stuns foes in radius", "210 G", "Stock: 3", false),
    ];
    for (gly, name, desc, price, stock, sel) in wares {
        let r = if sel { g.card_on(list, sz(1174.0, 128.0), "Ware row") } else { g.card(list, sz(1174.0, 128.0), "Ware row") };
        g.icon(r, cl(20.0, 0.0, 78.0, 78.0), "Ware icon", gly);
        g.heading_l(r, tl(118.0, 22.0, 440.0, 40.0), "Ware name", name, 28.0);
        g.note(r, tl(118.0, 68.0, 480.0, 34.0), "Ware desc", desc, 22.0);
        g.note(r, tr(390.0, 48.0, 160.0, 34.0), "Ware stock", stock, 22.0);
        g.value(r, tr(210.0, 44.0, 160.0, 40.0), "Ware price", price, 28.0);
        let buy_b = g.button(r, cr(20.0, 0.0, 164.0, 68.0), "Buy button", "Buy");
        big_accent(g, buy_b);
    }
    let foot = g.card(shop, bl(28.0, 24.0, 1174.0, 96.0), "Selected summary");
    g.label(foot, cl(24.0, 0.0, 680.0, 40.0), "Purchase info", "Selected: Berry Potion (x1)  -  Cost: 120 G", 26.0);
    let buy5 = g.button(foot, cr(20.0, 0.0, 280.0, 68.0), "Buy five", "Buy x5 (600 G)");
    big_accent(g, buy5);
}

/// Intercambio en dos columnas («Your offer» / «Their offer») con balance de valor y confirmación.
fn trading_two_col_barter(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "CARAVAN BARTER", "Deal Fairness: Balanced");
    // Columna izquierda: tu oferta
    let yours = g.panel(p, left(50.0, 880.0, 126.0, 140.0), "Your offer panel");
    g.heading_l(yours, tl(28.0, 24.0, 400.0, 42.0), "Your offer title", "YOUR OFFER", 32.0);
    g.value(yours, tr(28.0, 24.0, 300.0, 42.0), "Your value", "Total: 740 G", 28.0);
    let ygrid = g.grid(yours, tl(28.0, 88.0, 824.0, 560.0), "Your offer grid", 190.0, 170.0, 20.0);
    for (gly, qty) in [("OR", "x8"), ("WD", "x12"), ("PL", "x3"), ("GM", "x1"), ("", ""), ("", ""), ("", ""), ("", ""), ("", ""), ("", ""), ("", ""), ("", "")] {
        let s = g.slot(ygrid, sz(190.0, 170.0), "Your slot");
        if !gly.is_empty() {
            g.icon(s, cc(0.0, -14.0, 64.0, 64.0), "Offer icon", gly);
            g.badge(s, br(10.0, 10.0, 72.0, 40.0), "Offer qty", qty);
        }
    }
    let ygold = g.card(yours, bottom(24.0, 86.0, 28.0, 28.0), "Your gold bar");
    g.label(ygold, cl(20.0, 0.0, 340.0, 38.0), "Gold offered", "Gold Added: +120 G", 26.0);
    g.button2(ygold, cr(16.0, 0.0, 200.0, 68.0), "Add gold", "+50 Gold");
    // Columna derecha: su oferta
    let theirs = g.panel(p, right(50.0, 880.0, 126.0, 140.0), "Their offer panel");
    g.heading_l(theirs, tl(28.0, 24.0, 400.0, 42.0), "Their offer title", "THEIR OFFER", 32.0);
    g.value(theirs, tr(28.0, 24.0, 300.0, 42.0), "Their value", "Total: 720 G", 28.0);
    let tgrid = g.grid(theirs, tl(28.0, 88.0, 824.0, 560.0), "Their offer grid", 190.0, 170.0, 20.0);
    for (gly, qty) in [("SW", "x1"), ("MP", "x4"), ("KY", "x1"), ("", ""), ("", ""), ("", ""), ("", ""), ("", ""), ("", ""), ("", ""), ("", ""), ("", "")] {
        let s = g.slot(tgrid, sz(190.0, 170.0), "Their slot");
        if !gly.is_empty() {
            g.icon(s, cc(0.0, -14.0, 64.0, 64.0), "Their icon", gly);
            g.badge(s, br(10.0, 10.0, 72.0, 40.0), "Their qty", qty);
        }
    }
    let tmood = g.card(theirs, bottom(24.0, 86.0, 28.0, 28.0), "Trader mood bar");
    g.label(tmood, cl(20.0, 0.0, 620.0, 38.0), "Trader mood", "Trader Status: Willing to accept", 26.0);
    // Pie de confirmación
    let foot = g.panel(p, bottom(0.0, 108.0, 0.0, 0.0), "Trade footer");
    g.button2(foot, cl(50.0, 0.0, 240.0, 70.0), "Clear trade", "Clear Offer");
    g.bar(foot, cc(0.0, 0.0, 520.0, 24.0), "Fairness meter", 0.52, g.t.good);
    g.button(foot, cr(50.0, 0.0, 280.0, 70.0), "Confirm trade", "Confirm Trade");
}

/// Mercado en rejilla de productos con precios a la izquierda y carrito de compra a la derecha.
fn trading_market_cart(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "VILLAGE MARKETPLACE", "Wallet: 1,920 Coins");
    let catalog = g.panel(p, left(50.0, 1220.0, 126.0, 44.0), "Catalog panel");
    let grid = g.grid(catalog, cc(0.0, 0.0, 1140.0, 840.0), "Market grid", 362.0, 404.0, 24.0);
    let goods = [
        ("Honey Tea", "Restores energy", "90 C", true),
        ("Starlight Seed", "Rare crop", "240 C", false),
        ("Wool Scarf", "Warm accessory", "310 C", false),
        ("Clay Planter", "Home decor", "140 C", false),
        ("Sugar Tart", "+10% Speed", "110 C", false),
        ("Lantern Kit", "Garden light", "180 C", false),
    ];
    for (name, sub, price, sel) in goods {
        let c = if sel { g.card_on(grid, sz(362.0, 404.0), "Product card") } else { g.card(grid, sz(362.0, 404.0), "Product card") };
        g.art(c, tl(20.0, 20.0, 322.0, 174.0), "Product image");
        g.heading_l(c, tl(20.0, 208.0, 322.0, 38.0), "Product name", name, 26.0);
        g.note(c, tl(20.0, 248.0, 322.0, 30.0), "Product sub", sub, 22.0);
        g.value(c, bl(20.0, 34.0, 120.0, 38.0), "Product price", price, 26.0);
        g.button(c, br(20.0, 20.0, 170.0, 68.0), "Add to cart", "Add +");
    }
    // Carrito derecho
    let cart = g.panel(p, right(50.0, 570.0, 126.0, 44.0), "Cart panel");
    g.heading_l(cart, tl(28.0, 26.0, 340.0, 44.0), "Cart title", "Shopping Basket", 32.0);
    g.badge(cart, tr(28.0, 28.0, 120.0, 42.0), "Cart count", "3 ITEMS");
    let clist = g.vbox(cart, tl(28.0, 92.0, 514.0, 480.0), "Cart items", 14.0);
    for (item, qty, cost) in [("Honey Tea", "x2", "180 C"), ("Starlight Seed", "x1", "240 C"), ("Clay Planter", "x1", "140 C")] {
        let r = g.card(clist, sz(514.0, 108.0), "Cart row");
        g.heading_l(r, tl(20.0, 18.0, 300.0, 36.0), "Cart item name", item, 26.0);
        g.note(r, tl(20.0, 58.0, 160.0, 30.0), "Cart item qty", qty, 22.0);
        g.value(r, cr(20.0, 0.0, 140.0, 38.0), "Cart item cost", cost, 26.0);
    }
    let tot = g.card(cart, bl(28.0, 130.0, 514.0, 118.0), "Cart totals");
    g.label(tot, tl(20.0, 18.0, 220.0, 34.0), "Subtotal label", "Total Cost", 24.0);
    g.value(tot, tr(20.0, 18.0, 200.0, 40.0), "Total amount", "560 Coins", 30.0);
    g.note(tot, bl(20.0, 18.0, 474.0, 30.0), "Balance after", "Balance after purchase: 1,360 Coins", 22.0);
    g.button(cart, bl(28.0, 28.0, 514.0, 74.0), "Checkout button", "Complete Purchase");
}

// ===============================================================================================================
// 10. Change skin or accessory
// ===============================================================================================================

/// Personaje grande centrado y carrusel horizontal de skins abajo.
fn skin_carousel_bottom(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "WARDROBE & SKINS", "Unlocked: 9 / 16");
    let stage = g.panel(p, fill(80.0, 120.0, 80.0, 330.0), "Preview stage");
    g.art(stage, cc(0.0, -10.0, 620.0, 560.0), "Skin full preview");
    let info = g.card(stage, tl(28.0, 28.0, 420.0, 210.0), "Skin info card");
    g.badge(info, tl(20.0, 20.0, 150.0, 42.0), "Skin rarity", "EPIC SKIN");
    g.heading_l(info, tl(20.0, 76.0, 380.0, 44.0), "Skin name", "Cyber Mirage", 34.0);
    g.note(info, tl(20.0, 128.0, 380.0, 60.0), "Skin note", "Includes custom visor glow and neon trail.", 22.0);
    g.button(stage, br(28.0, 28.0, 280.0, 74.0), "Equip skin", "Equip Skin");
    g.button2(stage, tr(28.0, 28.0, 220.0, 68.0), "Rotate button", "Rotate 90");
    // Carrusel inferior de 5 skins
    let shelf = g.panel(p, bottom(30.0, 276.0, 80.0, 80.0), "Skin carousel panel");
    g.button2(shelf, cl(22.0, 0.0, 76.0, 76.0), "Prev skin", "<");
    g.button2(shelf, cr(22.0, 0.0, 76.0, 76.0), "Next skin", ">");
    let row = g.hbox_c(shelf, cc(0.0, 0.0, 1500.0, 224.0), "Skin thumbnails", 22.0);
    for (name, st, sel) in [
        ("Default Suit", "OWNED", false),
        ("Midnight Ops", "OWNED", false),
        ("Cyber Mirage", "EQUIPPED", true),
        ("Solar Flare", "OWNED", false),
        ("Phantom Zero", "LOCKED", false),
    ] {
        let c = if sel { g.card_on(row, sz(280.0, 216.0), "Skin card") } else { g.card(row, sz(280.0, 216.0), "Skin card") };
        g.art(c, tl(16.0, 16.0, 248.0, 108.0), "Skin thumb");
        g.heading_l(c, tl(16.0, 134.0, 248.0, 34.0), "Skin label", name, 24.0);
        g.note(c, bl(16.0, 12.0, 248.0, 30.0), "Skin state", st, 22.0);
    }
}

/// Lista vertical de skins con insignia de rareza a la izquierda y vista previa grande a la derecha.
fn skin_rarity_list(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "APPEARANCE ARCHIVE", "Active Outfit: Gilded Raven");
    let list_p = g.panel(p, left(50.0, 740.0, 126.0, 44.0), "Skin list panel");
    let rows = g.vbox(list_p, fill(28.0, 28.0, 28.0, 28.0), "Skins stack", 16.0);
    let skins = [
        ("Gilded Raven", "LEGENDARY", "Equipped", true),
        ("Ivory Duelist", "EPIC", "Owned", false),
        ("Velvet Nocturne", "RARE", "Owned", false),
        ("Ashen Coat", "COMMON", "Owned", false),
        ("Royal Masque", "EPIC", "Unlock at Lv 30", false),
        ("Obsidian Crown", "LEGENDARY", "Season Reward", false),
    ];
    for (name, rarity, state, sel) in skins {
        let c = if sel { g.card_on(rows, sz(684.0, 126.0), "Skin row") } else { g.card(rows, sz(684.0, 126.0), "Skin row") };
        g.art(c, cl(16.0, 0.0, 88.0, 88.0), "Skin mini art");
        g.heading_l(c, tl(122.0, 22.0, 360.0, 40.0), "Skin row name", name, 28.0);
        g.note(c, tl(122.0, 68.0, 320.0, 32.0), "Skin row state", state, 22.0);
        g.badge(c, cr(20.0, 0.0, 174.0, 44.0), "Rarity tag", rarity);
    }
    let prev = g.panel(p, fill(820.0, 126.0, 50.0, 44.0), "Skin preview panel");
    g.art(prev, fill(32.0, 32.0, 32.0, 160.0), "Outfit preview");
    let foot = g.card(prev, bottom(28.0, 108.0, 32.0, 32.0), "Preview footer");
    g.heading_l(foot, tl(24.0, 16.0, 480.0, 38.0), "Preview name", "Gilded Raven", 30.0);
    g.note(foot, tl(24.0, 58.0, 540.0, 32.0), "Preview detail", "Hand-stitched cloak with brass filigree trim.", 22.0);
    g.button(foot, cr(20.0, 0.0, 240.0, 70.0), "Apply outfit", "Equipped");
}

/// Pestañas Head / Body / Back con rejilla de muestras a la izquierda y vista previa a la derecha.
fn skin_category_swatches(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "ACCESSORY LOCKER", "Style Preset: Pastel Breeze");
    let left_p = g.panel(p, left(50.0, 1020.0, 126.0, 44.0), "Accessory picker");
    let tabs = g.hbox(left_p, tl(28.0, 24.0, 964.0, 70.0), "Slot tabs", 16.0);
    for (i, t) in ["Head", "Body", "Back", "Trail"].iter().enumerate() {
        if i == 0 {
            g.button(tabs, sz(228.0, 68.0), &format!("{} tab", t), t);
        } else {
            g.button2(tabs, sz(228.0, 68.0), &format!("{} tab", t), t);
        }
    }
    let grid = g.grid(left_p, tl(28.0, 116.0, 964.0, 660.0), "Swatch grid", 226.0, 204.0, 20.0);
    let items = [
        ("BR", "Beret", true), ("CR", "Flower Crown", false), ("GL", "Round Glasses", false), ("RB", "Ribbon", false),
        ("HD", "Headphones", false), ("CP", "Sun Cap", false), ("HR", "Cloud Pin", false), ("MK", "Sleep Mask", false),
        ("ST", "Star Clip", false), ("BN", "Beanie", false), ("VS", "Visor", false), ("NO", "None", false),
    ];
    for (code, name, sel) in items {
        let c = if sel { g.card_on(grid, sz(226.0, 204.0), "Accessory swatch") } else { g.card(grid, sz(226.0, 204.0), "Accessory swatch") };
        g.icon(c, tc(0.0, 28.0, 76.0, 76.0), "Accessory icon", code);
        g.heading(c, bottom(24.0, 40.0, 12.0, 12.0), "Accessory name", name, 24.0);
    }
    g.note(left_p, bl(28.0, 32.0, 640.0, 34.0), "Tint note", "Tip: Accessories inherit your primary outfit tint.", 22.0);
    // Vista previa derecha
    let right_p = g.panel(p, right(50.0, 770.0, 126.0, 44.0), "Avatar preview panel");
    g.art(right_p, tl(28.0, 28.0, 714.0, 650.0), "Character preview");
    g.heading_l(right_p, tl(28.0, 696.0, 440.0, 42.0), "Equipped hat", "Head: Soft Beret", 30.0);
    let btns = g.hbox(right_p, bl(28.0, 28.0, 714.0, 72.0), "Accessory actions", 18.0);
    g.button2(btns, sz(348.0, 70.0), "Random look", "Randomize");
    g.button(btns, sz(348.0, 70.0), "Save look", "Save Look");
}

// ===============================================================================================================
// 11. Character customization
// ===============================================================================================================

/// Deslizadores de rasgos a la izquierda y vista previa del personaje a la derecha.
fn custom_sliders_preview(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "CHARACTER CREATOR", "Step: Sculpt Features");
    let left_p = g.panel(p, left(50.0, 780.0, 126.0, 44.0), "Sliders panel");
    g.heading_l(left_p, tl(28.0, 24.0, 500.0, 42.0), "Sculpt heading", "Body & Face Proportions", 30.0);
    let sl = g.vbox(left_p, tl(28.0, 84.0, 724.0, 690.0), "Feature sliders", 14.0);
    let sliders = [
        ("Height", "178 cm", 0.62),
        ("Build", "Athletic", 0.54),
        ("Jaw Width", "52%", 0.52),
        ("Eye Spacing", "48%", 0.48),
        ("Nose Bridge", "60%", 0.60),
        ("Brow Depth", "40%", 0.40),
        ("Voice Pitch", "Medium", 0.50),
    ];
    for (name, val, v) in sliders {
        let c = g.card(sl, sz(724.0, 86.0), &format!("{} slider card", name));
        g.label(c, tl(20.0, 12.0, 340.0, 30.0), &format!("{} label", name), name, 24.0);
        g.value(c, tr(20.0, 12.0, 220.0, 30.0), &format!("{} val", name), val, 24.0);
        g.slider(c, bottom(14.0, 24.0, 20.0, 20.0), &format!("{} slider", name), v);
    }
    let acts = g.hbox(left_p, bl(28.0, 24.0, 724.0, 72.0), "Sculpt actions", 18.0);
    g.button2(acts, sz(353.0, 70.0), "Randomize button", "Randomize");
    g.button2(acts, sz(353.0, 70.0), "Reset button", "Reset");
    // Vista previa derecha
    let right_p = g.panel(p, fill(860.0, 126.0, 50.0, 44.0), "Preview panel");
    g.art(right_p, tl(32.0, 32.0, 946.0, 720.0), "Character 3D preview");
    g.button2(right_p, bl(32.0, 28.0, 240.0, 72.0), "Zoom button", "Toggle Close-Up");
    g.button(right_p, br(32.0, 28.0, 280.0, 72.0), "Confirm character", "Confirm Hero");
}

/// Fila de categorías arriba, muestras de color y caras predefinidas junto a la vista previa.
fn custom_palette_presets(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "CREATE YOUR HERO", "Cartoon Style");
    let cats = g.panel(p, top(116.0, 96.0, 50.0, 50.0), "Category strip");
    let crow = g.hbox_c(cats, cc(0.0, 0.0, 1460.0, 70.0), "Category buttons", 18.0);
    for (i, cat) in ["Face", "Hair", "Eyes", "Outfit", "Colors", "Voice"].iter().enumerate() {
        if i == 0 {
            let b = g.button(crow, sz(228.0, 68.0), &format!("{} cat", cat), cat);
            big_accent(g, b);
        } else {
            g.button2(crow, sz(228.0, 68.0), &format!("{} cat", cat), cat);
        }
    }
    // Izquierda: presets + paleta de color
    let left_p = g.panel(p, left(50.0, 960.0, 232.0, 40.0), "Presets panel");
    g.heading_l(left_p, tl(28.0, 22.0, 440.0, 40.0), "Presets heading", "Face Presets", 30.0);
    let fgrid = g.grid(left_p, tl(28.0, 76.0, 904.0, 390.0), "Face grid", 210.0, 182.0, 20.0);
    for i in 1..=8 {
        let c = if i == 2 { g.card_on(fgrid, sz(210.0, 182.0), "Face preset") } else { g.card(fgrid, sz(210.0, 182.0), "Face preset") };
        g.icon(c, tc(0.0, 24.0, 76.0, 76.0), "Face icon", &format!("F{}", i));
        g.heading(c, bottom(20.0, 38.0, 12.0, 12.0), "Face name", &format!("Preset {}", i), 24.0);
    }
    g.heading_l(left_p, tl(28.0, 492.0, 440.0, 40.0), "Color heading", "Hair & Accent Color", 30.0);
    let sw_row = g.hbox(left_p, tl(28.0, 548.0, 904.0, 96.0), "Color swatches", 18.0);
    let palette = [g.t.accent, g.t.accent2, g.t.good, g.t.warn, g.t.bad, g.t.line, shade(g.t.accent, 0.7), shade(g.t.accent2, 0.7)];
    for (i, col) in palette.iter().enumerate() {
        let box_n = if i == 0 { g.card_on(sw_row, sz(96.0, 90.0), "Swatch frame") } else { g.card(sw_row, sz(96.0, 90.0), "Swatch frame") };
        g.rect(box_n, fill(12.0, 12.0, 12.0, 12.0), "Color chip", *col, 14.0);
    }
    g.button2(left_p, bl(28.0, 24.0, 280.0, 70.0), "Shuffle look", "Surprise Me");
    // Derecha: vista previa
    let right_p = g.panel(p, right(50.0, 830.0, 232.0, 40.0), "Hero preview");
    g.art(right_p, tl(28.0, 28.0, 774.0, 640.0), "Avatar render");
    let start_b = g.button(right_p, bl(28.0, 24.0, 774.0, 76.0), "Start adventure", "Start Adventure");
    big_accent(g, start_b);
}

/// Asistente por pasos (Race > Appearance > Name) con tarjetas de selección y campo de nombre.
fn custom_step_wizard(g: &mut G) {
    let p = g.page();
    g.background();
    // Barra de pasos arriba
    let step_bar = g.panel(p, top(28.0, 108.0, 60.0, 60.0), "Wizard steps");
    let steps = g.hbox_c(step_bar, cc(0.0, 0.0, 1260.0, 70.0), "Steps row", 26.0);
    g.button2(steps, sz(360.0, 68.0), "Step 1", "1. Race (Done)");
    g.button2(steps, sz(360.0, 68.0), "Step 2", "2. Appearance (Done)");
    g.button(steps, sz(360.0, 68.0), "Step 3", "3. Name & Origin");
    // Panel central principal
    let main = g.panel(p, fill(60.0, 160.0, 60.0, 130.0), "Wizard content");
    // Izquierda: origen y nombre
    let form = g.card(main, tl(36.0, 36.0, 980.0, 718.0), "Identity form");
    g.heading_l(form, tl(32.0, 28.0, 640.0, 48.0), "Name heading", "Choose Your Name", 36.0);
    g.note(form, tl(32.0, 82.0, 760.0, 34.0), "Name hint", "Enter a name for your traveler (3 to 18 characters).", 24.0);
    g.input(form, tl(32.0, 136.0, 640.0, 74.0), "Character name input", "Rowan Greenleaf");
    g.button2(form, tl(692.0, 136.0, 256.0, 74.0), "Generate name", "Random Name");
    g.heading_l(form, tl(32.0, 248.0, 500.0, 42.0), "Origin heading", "Select Homeland", 30.0);
    let origins = g.vbox(form, tl(32.0, 306.0, 916.0, 380.0), "Origin cards", 16.0);
    for (title, bonus, sel) in [
        ("Whispering Vale", "+10% Foraging Yield & Calm Aura", true),
        ("Highland Crags", "+15 Max Health & Cold Resistance", false),
        ("River Delta", "+12% Swim Speed & Trade Discount", false),
    ] {
        let o = if sel { g.card_on(origins, sz(916.0, 114.0), "Origin card") } else { g.card(origins, sz(916.0, 114.0), "Origin card") };
        g.heading_l(o, tl(24.0, 18.0, 600.0, 38.0), "Origin name", title, 28.0);
        g.note(o, tl(24.0, 62.0, 760.0, 34.0), "Origin bonus", bonus, 22.0);
        if sel {
            g.badge(o, cr(24.0, 0.0, 148.0, 44.0), "Selected tag", "CHOSEN");
        }
    }
    // Derecha: resumen del personaje
    let prev = g.card(main, tr(36.0, 36.0, 710.0, 718.0), "Summary preview");
    g.art(prev, tl(28.0, 28.0, 654.0, 450.0), "Traveler portrait");
    g.heading_l(prev, tl(28.0, 500.0, 654.0, 44.0), "Preview hero", "Rowan Greenleaf", 34.0);
    g.note(prev, tl(28.0, 550.0, 654.0, 36.0), "Preview race", "Race: Sylvan   |   Homeland: Whispering Vale", 24.0);
    g.note(prev, tl(28.0, 594.0, 654.0, 36.0), "Preview kit", "Starter Kit: Cedar Staff & Travel Cloak", 22.0);
    // Pie de navegación
    let foot = g.panel(p, bottom(0.0, 102.0, 0.0, 0.0), "Wizard footer");
    g.button2(foot, cl(60.0, 0.0, 260.0, 70.0), "Back step", "< Appearance");
    g.button(foot, cr(60.0, 0.0, 300.0, 70.0), "Finish wizard", "Begin Journey >");
}

// ===============================================================================================================
// 12. Sub-equip & tweak
// ===============================================================================================================

/// Arma centrada con cuatro huecos de accesorios alrededor y barras de rendimiento a la derecha.
fn tweak_weapon_attachments(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "GUNSMITH BENCH", "Weapon: Vektor-9 Rifle   |   4 / 4 Attachments");
    let bench = g.panel(p, left(50.0, 1240.0, 126.0, 44.0), "Weapon bench");
    g.art(bench, cc(0.0, 0.0, 740.0, 280.0), "Weapon blueprint");
    // 4 huecos conectados visualmente alrededor del arma
    let slots = [
        (tl(40.0, 40.0, 360.0, 160.0), "SCOPE", "2x Holo Sight", "+14 Accuracy", true),
        (tr(40.0, 40.0, 360.0, 160.0), "BARREL", "Suppressor Mk2", "-20 Noise / -4 Range", false),
        (bl(40.0, 40.0, 360.0, 160.0), "GRIP", "Angled Foregrip", "+12 Recoil Control", false),
        (br(40.0, 40.0, 360.0, 160.0), "MAGAZINE", "Extended 40-Rd", "+10 Capacity / -6 ADS", false),
    ];
    for (pos, tag, name, effect, sel) in slots {
        let c = if sel { g.card_on(bench, pos, &format!("{} slot", tag)) } else { g.card(bench, pos, &format!("{} slot", tag)) };
        g.badge(c, tl(18.0, 16.0, 146.0, 40.0), "Attach tag", tag);
        g.heading_l(c, tl(18.0, 68.0, 324.0, 38.0), "Attach name", name, 26.0);
        g.note(c, bl(18.0, 16.0, 324.0, 30.0), "Attach effect", effect, 22.0);
    }
    // Panel derecho de estadísticas
    let stats_p = g.panel(p, right(50.0, 550.0, 126.0, 44.0), "Weapon stats panel");
    g.heading_l(stats_p, tl(28.0, 28.0, 494.0, 44.0), "Stats title", "Ballistics Profile", 32.0);
    let bars = g.vbox(stats_p, tl(28.0, 92.0, 494.0, 560.0), "Ballistics bars", 16.0);
    stat_bar_row(g, bars, 494.0, "Damage", "68", 0.68, g.t.accent);
    stat_bar_row(g, bars, 494.0, "Accuracy", "86 (+14)", 0.86, g.t.good);
    stat_bar_row(g, bars, 494.0, "Range", "62 (-4)", 0.62, g.t.warn);
    stat_bar_row(g, bars, 494.0, "Control", "79 (+12)", 0.79, g.t.good);
    stat_bar_row(g, bars, 494.0, "Handling", "71 (-6)", 0.71, g.t.bad);
    stat_bar_row(g, bars, 494.0, "Fire Rate", "82", 0.82, g.t.accent);
    let btns = g.hbox(stats_p, bl(28.0, 28.0, 494.0, 72.0), "Bench actions", 16.0);
    g.button2(btns, sz(239.0, 70.0), "Test range", "Test Fire");
    g.button(btns, sz(239.0, 70.0), "Save blueprint", "Save Build");
}

/// Ajustes de vehículo con vista previa, deslizadores de puesta a punto y barras de rendimiento.
fn tweak_vehicle_sliders(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "GARAGE TUNING", "Car: Apex GT-4   |   Class S-820");
    // Izquierda: sliders de puesta a punto
    let tune = g.panel(p, left(50.0, 920.0, 126.0, 44.0), "Tuning panel");
    g.heading_l(tune, tl(28.0, 24.0, 520.0, 44.0), "Tuning title", "Setup Parameters", 32.0);
    g.dropdown(tune, tr(28.0, 18.0, 320.0, 66.0), "Preset dropdown", &["Preset: Track Dry", "Preset: Wet Grip", "Preset: Top Speed"]);
    let sl = g.vbox(tune, tl(28.0, 102.0, 864.0, 680.0), "Tuning sliders", 16.0);
    for (name, val, v) in [
        ("Final Gear Ratio", "3.42  (Accel <-> Speed)", 0.64),
        ("Front Downforce", "185 kg", 0.72),
        ("Rear Downforce", "240 kg", 0.80),
        ("Suspension Stiffness", "Firm +4", 0.68),
        ("Brake Balance", "54% Front", 0.54),
        ("Tire Pressure", "29.5 PSI", 0.50),
    ] {
        let c = g.card(sl, sz(864.0, 98.0), &format!("{} tune", name));
        g.heading_l(c, tl(20.0, 12.0, 420.0, 34.0), &format!("{} name", name), name, 24.0);
        g.value(c, tr(20.0, 12.0, 380.0, 34.0), &format!("{} value", name), val, 22.0);
        g.slider(c, bottom(16.0, 26.0, 20.0, 20.0), &format!("{} slider", name), v);
    }
    let acts = g.hbox(tune, bl(28.0, 24.0, 864.0, 72.0), "Tune actions", 20.0);
    g.button2(acts, sz(260.0, 70.0), "Reset tune", "Reset Setup");
    g.button(acts, sz(300.0, 70.0), "Apply tune", "Apply & Test");
    // Derecha: coche + telemetría
    let right_p = g.panel(p, right(50.0, 870.0, 126.0, 44.0), "Telemetry panel");
    g.art(right_p, tl(28.0, 28.0, 814.0, 340.0), "Vehicle preview");
    g.heading_l(right_p, tl(28.0, 390.0, 500.0, 42.0), "Perf title", "Projected Performance", 30.0);
    let bars = g.vbox(right_p, tl(28.0, 448.0, 814.0, 430.0), "Performance bars", 14.0);
    stat_bar_row(g, bars, 814.0, "Top Speed", "318 km/h", 0.86, g.t.accent);
    stat_bar_row(g, bars, 814.0, "0-100 Acceleration", "2.8 sec", 0.91, g.t.good);
    stat_bar_row(g, bars, 814.0, "Cornering Grip", "1.42 G", 0.84, g.t.good);
    stat_bar_row(g, bars, 814.0, "Braking Distance", "28.4 m", 0.78, g.t.warn);
    stat_bar_row(g, bars, 814.0, "Aero Drag", "Low-Med", 0.45, g.t.accent2);
}

/// Engaste de gemas: objeto con tres huecos arriba e inventario de gemas abajo.
fn tweak_gem_socketing(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "RUNE & GEM SOCKETING", "Item: Runeblade of Dawn");
    // Mitad superior: objeto + 3 huecos
    let top_p = g.panel(p, top(126.0, 410.0, 50.0, 50.0), "Item sockets panel");
    g.art(top_p, cl(32.0, 0.0, 320.0, 340.0), "Weapon art");
    g.heading_l(top_p, tl(384.0, 30.0, 540.0, 46.0), "Socket item name", "Runeblade of Dawn (+6)", 36.0);
    g.note(top_p, tl(384.0, 82.0, 680.0, 34.0), "Socket bonus", "Set Bonus (2/3 Fire): +25 Burn Damage on critical hits.", 24.0);
    let srow = g.hbox(top_p, tl(384.0, 146.0, 1390.0, 230.0), "Gem sockets", 24.0);
    for (idx, name, bonus, filled) in [
        ("Socket 1", "Ruby of Ash", "+18 Fire DMG", true),
        ("Socket 2", "Amber Shard", "+12% Crit Rate", true),
        ("Socket 3", "Empty Socket", "Select a gem below", false),
    ] {
        let c = if !filled { g.card_on(srow, sz(446.0, 220.0), "Gem socket") } else { g.card(srow, sz(446.0, 220.0), "Gem socket") };
        g.badge(c, tl(20.0, 18.0, 150.0, 42.0), "Socket badge", idx);
        g.icon(c, cl(20.0, 26.0, 70.0, 70.0), "Gem icon", if filled { "GM" } else { "+" });
        g.heading_l(c, tl(108.0, 92.0, 318.0, 38.0), "Socketed gem", name, 26.0);
        g.note(c, tl(108.0, 136.0, 318.0, 32.0), "Socket effect", bonus, 22.0);
    }
    // Mitad inferior: inventario de gemas
    let bag = g.panel(p, bottom(36.0, 480.0, 50.0, 50.0), "Gem pouch panel");
    g.heading_l(bag, tl(32.0, 24.0, 460.0, 42.0), "Pouch heading", "Available Gems", 30.0);
    g.button(bag, tr(32.0, 18.0, 260.0, 68.0), "Socket button", "Socket Selected");
    let grid = g.grid(bag, bottom(28.0, 356.0, 32.0, 32.0), "Gems grid", 418.0, 166.0, 22.0);
    let gems = [
        ("RB", "Crimson Ruby", "+22 Fire DMG", "x2", true),
        ("SP", "Frost Sapphire", "+18 Ice DMG", "x3", false),
        ("EM", "Vital Emerald", "+90 Max HP", "x1", false),
        ("TP", "Storm Topaz", "+10% Attack Speed", "x4", false),
        ("AM", "Arcane Amethyst", "+14% Mana Regen", "x2", false),
        ("OX", "Shadow Onyx", "+15% Armor Pierce", "x1", false),
        ("OP", "Sun Opal", "+8% XP Gain", "x3", false),
        ("PR", "Mist Pearl", "+12% Evasion", "x2", false),
    ];
    for (gly, name, stat, qty, sel) in gems {
        let c = if sel { g.card_on(grid, sz(418.0, 166.0), "Gem card") } else { g.card(grid, sz(418.0, 166.0), "Gem card") };
        g.icon(c, cl(18.0, 0.0, 68.0, 68.0), "Pouch gem icon", gly);
        g.heading_l(c, tl(104.0, 28.0, 220.0, 38.0), "Gem name", name, 24.0);
        g.note(c, tl(104.0, 74.0, 290.0, 34.0), "Gem stat", stat, 22.0);
        g.badge(c, tr(16.0, 18.0, 68.0, 40.0), "Gem qty", qty);
    }
}

// ===============================================================================================================
// 13. World customization
// ===============================================================================================================

/// Decoración de casa: vista amplia de la habitación arriba y catálogo horizontal de muebles abajo.
fn world_house_decor(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "COTTAGE DECORATOR", "Comfort Score: 420   |   Budget: 3,200 C");
    // Vista de la habitación
    let view = g.panel(p, fill(50.0, 120.0, 50.0, 360.0), "Room viewport");
    g.art(view, fill(24.0, 24.0, 24.0, 24.0), "Room preview");
    let hud = g.card(view, tl(44.0, 44.0, 380.0, 152.0), "Room info overlay");
    g.heading_l(hud, tl(20.0, 18.0, 340.0, 38.0), "Room name", "Sunlit Living Room", 28.0);
    g.note(hud, tl(20.0, 62.0, 340.0, 32.0), "Item count", "Placed Items: 14 / 30", 22.0);
    g.note(hud, tl(20.0, 100.0, 340.0, 32.0), "Grid snap", "Snap to Grid: ON", 22.0);
    let tools = g.hbox(view, tr(44.0, 44.0, 480.0, 72.0), "Room tools", 14.0);
    g.button2(tools, sz(150.0, 68.0), "Rotate decor", "Rotate");
    g.button2(tools, sz(150.0, 68.0), "Store decor", "Store");
    g.button(tools, sz(150.0, 68.0), "Save room", "Save");
    // Catálogo inferior
    let cat = g.panel(p, bottom(30.0, 306.0, 50.0, 50.0), "Furniture catalog");
    let tabs = g.hbox(cat, tl(28.0, 18.0, 960.0, 68.0), "Decor tabs", 14.0);
    for (i, t) in ["Seating", "Tables", "Lighting", "Plants", "Rugs"].iter().enumerate() {
        if i == 0 {
            g.button(tabs, sz(180.0, 68.0), &format!("{} tab", t), t);
        } else {
            g.button2(tabs, sz(180.0, 68.0), &format!("{} tab", t), t);
        }
    }
    let items = g.hbox_c(cat, bottom(20.0, 192.0, 28.0, 28.0), "Catalog items", 20.0);
    for (name, price, sel) in [
        ("Plush Armchair", "280 C", true),
        ("Oak Bench", "190 C", false),
        ("Bean Bag", "140 C", false),
        ("Window Sofa", "420 C", false),
        ("Wicker Stool", "95 C", false),
        ("Rocking Chair", "310 C", false),
    ] {
        let c = if sel { g.card_on(items, sz(274.0, 184.0), "Decor card") } else { g.card(items, sz(274.0, 184.0), "Decor card") };
        g.art(c, tl(14.0, 14.0, 246.0, 86.0), "Decor thumb");
        g.heading_l(c, tl(14.0, 108.0, 246.0, 32.0), "Decor name", name, 22.0);
        g.note(c, bl(14.0, 10.0, 140.0, 28.0), "Decor price", price, 22.0);
    }
}

/// Constructor de base con categorías a la izquierda, terreno al centro y catálogo con coste a la derecha.
fn world_base_builder(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "ISLAND BASE BUILDER", "Wood: 640   Stone: 410   Energy: 38 / 50");
    // Categorías izquierda
    let cats = g.panel(p, left(40.0, 320.0, 126.0, 40.0), "Build categories");
    g.heading_l(cats, tl(24.0, 22.0, 272.0, 40.0), "Cat heading", "Categories", 28.0);
    let clist = g.vbox_fill(cats, tl(24.0, 78.0, 272.0, 560.0), "Category list", 14.0);
    for (i, c) in ["Defense", "Power", "Housing", "Farms", "Roads", "Decor"].iter().enumerate() {
        let item = g.menu_item(clist, sz(272.0, 72.0), &format!("{} cat", c), c, i == 0);
        if i == 0 {
            big_accent(g, item);
        }
    }
    g.button2(cats, bl(24.0, 24.0, 272.0, 70.0), "Demolish mode", "Demolish");
    // Terreno al centro
    let map = g.panel(p, fill(384.0, 126.0, 610.0, 40.0), "Terrain map");
    g.art(map, fill(24.0, 24.0, 24.0, 96.0), "Terrain grid preview");
    let map_bar = g.card(map, bottom(20.0, 68.0, 24.0, 24.0), "Map status");
    g.label(map_bar, cl(20.0, 0.0, 520.0, 36.0), "Placement info", "Placing: Watch Tower (2x2) - Valid Tile", 24.0);
    let valid_b = g.badge(map_bar, cr(18.0, 0.0, 156.0, 50.0), "Valid badge", "VALID");
    big_accent(g, valid_b);
    // Catálogo + coste derecha
    let right_p = g.panel(p, right(40.0, 546.0, 126.0, 40.0), "Structures panel");
    g.heading_l(right_p, tl(24.0, 22.0, 498.0, 40.0), "Struct heading", "Defense Structures", 28.0);
    let sgrid = g.grid(right_p, tl(24.0, 76.0, 498.0, 460.0), "Structure grid", 238.0, 214.0, 20.0);
    for (gly, name, sel) in [("TW", "Watch Tower", true), ("WL", "Stone Wall", false), ("GT", "Iron Gate", false), ("CN", "Spark Turret", false)] {
        let c = if sel { g.card_on(sgrid, sz(238.0, 214.0), "Struct card") } else { g.card(sgrid, sz(238.0, 214.0), "Struct card") };
        g.icon(c, tc(0.0, 28.0, 76.0, 76.0), "Struct icon", gly);
        g.heading(c, bottom(24.0, 40.0, 12.0, 12.0), "Struct name", name, 24.0);
    }
    let cost = g.card(right_p, tl(24.0, 558.0, 498.0, 224.0), "Cost card");
    g.heading_l(cost, tl(20.0, 16.0, 458.0, 38.0), "Cost title", "Watch Tower Cost", 26.0);
    g.label(cost, tl(20.0, 68.0, 240.0, 34.0), "Wood label", "Timber", 24.0);
    g.value(cost, tr(20.0, 68.0, 180.0, 34.0), "Wood cost", "120 / 640", 24.0);
    g.label(cost, tl(20.0, 114.0, 240.0, 34.0), "Stone label", "Stone", 24.0);
    g.value(cost, tr(20.0, 114.0, 180.0, 34.0), "Stone cost", "80 / 410", 24.0);
    g.label(cost, tl(20.0, 160.0, 240.0, 34.0), "Power label", "Energy Draw", 24.0);
    g.value(cost, tr(20.0, 160.0, 180.0, 34.0), "Power cost", "4 / 12 Free", 24.0);
    let build_b = g.button(right_p, bl(24.0, 24.0, 498.0, 74.0), "Build button", "Place Watch Tower");
    big_accent(g, build_b);
}

/// Ajustes del mundo: interruptores de reglas, paleta de bioma y vista previa en vivo.
fn world_rules_palette(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "WORLD SETTINGS", "Seed: 482-OAK-991");
    let left_p = g.panel(p, left(50.0, 860.0, 126.0, 44.0), "World rules panel");
    g.heading_l(left_p, tl(28.0, 24.0, 480.0, 42.0), "Rules heading", "Environment Rules", 30.0);
    let toggles = g.vbox(left_p, tl(28.0, 82.0, 804.0, 360.0), "Rule toggles", 8.0);
    for (label, on) in [
        ("Dynamic Day / Night Cycle", true),
        ("Seasonal Weather & Storms", true),
        ("Ambient Wildlife Spawns", true),
        ("Morning Valley Fog", false),
        ("Resource Regrowth", true),
    ] {
        let r = g.card(toggles, sz(804.0, 64.0), "Toggle row");
        g.toggle(r, cl(20.0, 0.0, 760.0, 64.0), &format!("{} toggle", label), label, on);
    }
    g.heading_l(left_p, tl(28.0, 466.0, 480.0, 40.0), "Palette heading", "Biome Color Palette", 30.0);
    let pals = g.hbox(left_p, tl(28.0, 522.0, 804.0, 130.0), "Biome palettes", 18.0);
    for (name, c1, c2, sel) in [
        ("Emerald", g.t.good, g.t.accent, true),
        ("Autumn", g.t.warn, g.t.bad, false),
        ("Highland", g.t.sky, g.t.ground, false),
        ("Twilight", g.t.bg2, g.t.accent2, false),
    ] {
        let c = if sel { g.card_on(pals, sz(186.0, 124.0), "Palette card") } else { g.card(pals, sz(186.0, 124.0), "Palette card") };
        g.art_c(c, tl(14.0, 14.0, 158.0, 56.0), "Palette swatch", c1, c2);
        g.heading(c, bottom(14.0, 36.0, 10.0, 10.0), "Palette name", name, 22.0);
    }
    g.input(left_p, tl(28.0, 680.0, 520.0, 70.0), "World seed input", "482-OAK-991");
    g.button2(left_p, tl(568.0, 680.0, 264.0, 70.0), "New seed", "New Seed");
    g.button(left_p, bl(28.0, 24.0, 804.0, 72.0), "Apply world", "Generate World");
    // Derecha: vista previa
    let right_p = g.panel(p, right(50.0, 930.0, 126.0, 44.0), "World preview panel");
    g.heading_l(right_p, tl(28.0, 24.0, 500.0, 42.0), "Preview title", "Biome Preview", 30.0);
    g.art_c(right_p, tl(28.0, 82.0, 874.0, 580.0), "Landscape preview", g.t.sky, g.t.ground);
    let sum = g.card(right_p, bl(28.0, 28.0, 874.0, 196.0), "World summary");
    g.heading_l(sum, tl(24.0, 20.0, 540.0, 38.0), "Biome title", "Emerald Valley (Temperate)", 28.0);
    g.note(sum, tl(24.0, 68.0, 820.0, 40.0), "Biome desc", "Lush pine forests, mild rainfall, abundant timber and clear rivers.", 24.0);
    g.label(sum, bl(24.0, 20.0, 820.0, 36.0), "Difficulty note", "Survival Rating: Moderate   |   Day Length: 24 min", 22.0);
}

// ===============================================================================================================
// 14. Upgrading & ranking up
// ===============================================================================================================

/// Comparativa antes/después del objeto con flecha central, costes y probabilidad de éxito.
fn upgrade_before_after(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "FORGE ENHANCEMENT", "Credits: 24,500   Alloy Cores: 18");
    let stage = g.panel(p, top(126.0, 520.0, 60.0, 60.0), "Before after stage");
    // Tarjeta ANTES
    let before = g.card(stage, cl(40.0, 0.0, 740.0, 440.0), "Before card");
    g.badge(before, tl(24.0, 24.0, 180.0, 44.0), "Current tag", "CURRENT +5");
    g.art(before, tl(24.0, 86.0, 240.0, 240.0), "Current item art");
    g.heading_l(before, tl(288.0, 92.0, 420.0, 44.0), "Before name", "Plasma Lance +5", 32.0);
    let bstats = g.vbox(before, tl(288.0, 152.0, 420.0, 250.0), "Before stats", 12.0);
    for (k, v) in [("Attack", "312"), ("Energy Crit", "14%"), ("Heat Cap", "100")] {
        kv_row(g, bstats, 420.0, 72.0, k, v);
    }
    // Flecha central
    g.icon(stage, cc(0.0, 0.0, 110.0, 110.0), "Upgrade arrow", ">");
    // Tarjeta DESPUÉS
    let after = g.card_on(stage, cr(40.0, 0.0, 740.0, 440.0), "After card");
    g.badge(after, tl(24.0, 24.0, 180.0, 44.0), "Next tag", "NEXT +6");
    g.art(after, tl(24.0, 86.0, 240.0, 240.0), "Upgraded item art");
    g.heading_l(after, tl(288.0, 92.0, 420.0, 44.0), "After name", "Plasma Lance +6", 32.0);
    let astats = g.vbox(after, tl(288.0, 152.0, 420.0, 250.0), "After stats", 12.0);
    for (k, v) in [("Attack", "358 (+46)"), ("Energy Crit", "17% (+3%)"), ("Heat Cap", "115 (+15)")] {
        kv_row(g, astats, 420.0, 72.0, k, v);
    }
    // Panel inferior: materiales, probabilidad y botón
    let mat_p = g.panel(p, bottom(40.0, 360.0, 60.0, 60.0), "Materials and action");
    g.heading_l(mat_p, tl(32.0, 24.0, 460.0, 42.0), "Mat heading", "Required Materials", 30.0);
    let mrow = g.hbox(mat_p, tl(32.0, 84.0, 1080.0, 160.0), "Materials row", 22.0);
    for (gly, name, qty) in [("AC", "Alloy Core", "4 / 18"), ("PC", "Plasma Cell", "2 / 7"), ("CR", "Credits", "3,200")] {
        let m = g.card(mrow, sz(344.0, 150.0), "Material card");
        g.icon(m, cl(18.0, 0.0, 64.0, 64.0), "Mat icon", gly);
        g.heading_l(m, tl(100.0, 34.0, 226.0, 38.0), "Mat name", name, 26.0);
        g.note(m, tl(100.0, 78.0, 226.0, 34.0), "Mat count", qty, 24.0);
    }
    let chance = g.card(mat_p, tr(32.0, 32.0, 560.0, 200.0), "Chance card");
    g.note(chance, tl(24.0, 20.0, 320.0, 34.0), "Chance label", "ENHANCEMENT CHANCE", 22.0);
    g.heading_l(chance, tl(24.0, 64.0, 260.0, 70.0), "Chance value", "85%", 58.0);
    g.bar(chance, tl(24.0, 150.0, 512.0, 20.0), "Chance bar", 0.85, g.t.good);
    g.button(mat_p, br(32.0, 28.0, 560.0, 76.0), "Enhance button", "Enhance to +6");
    g.note(mat_p, bl(32.0, 34.0, 960.0, 34.0), "Safety note", "Item will not break on failure; bonus +10% chance next try.", 22.0);
}

/// Insignia central de rango con estrellas, barra de ascenso y materiales requeridos.
fn upgrade_rank_badge(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "ATHLETE RANK UP", "Tokens: 14   Medals: 6");
    let center = g.panel(p, cc(0.0, 36.0, 1280.0, 880.0), "Rank up panel");
    g.icon(center, tc(0.0, 40.0, 196.0, 196.0), "Rank emblem", "III");
    g.heading(center, tc(0.0, 256.0, 820.0, 54.0), "Rank title", "GOLD TIER III  ->  GOLD TIER IV", 40.0);
    // Fila de 5 estrellas en ASCII
    let stars = g.hbox_c(center, tc(0.0, 324.0, 520.0, 62.0), "Rank stars", 18.0);
    for i in 0..5 {
        if i < 3 {
            g.badge(stars, sz(84.0, 52.0), "Filled star", "*");
        } else {
            let s = g.card(stars, sz(84.0, 52.0), "Empty star");
            g.heading(s, full(), "Star dash", "-", 28.0);
        }
    }
    g.bar(center, tc(0.0, 408.0, 860.0, 24.0), "Rank xp bar", 0.78, g.t.accent);
    g.txt(center, tc(0.0, 444.0, 600.0, 34.0), "Rank xp label", "Rank Progress: 3,900 / 5,000 RP", 24.0, g.t.muted, CENTER, false);
    // Materiales y recompensas
    let row = g.hbox_c(center, tc(0.0, 504.0, 1180.0, 220.0), "Rank requirements", 24.0);
    for (title, val, sub) in [
        ("Pro Tokens", "8 / 14", "Required to promote"),
        ("League Medals", "3 / 6", "Earned in weekly cups"),
        ("Promotion Reward", "+12 All Stats", "Unlocks Tier IV Jersey"),
    ] {
        let c = g.card(row, sz(376.0, 210.0), "Rank req card");
        g.note(c, tl(24.0, 22.0, 328.0, 32.0), "Req title", title, 22.0);
        g.heading_l(c, tl(24.0, 70.0, 328.0, 48.0), "Req value", val, 34.0);
        g.note(c, bl(24.0, 22.0, 328.0, 32.0), "Req note", sub, 22.0);
    }
    let btns = g.hbox_c(center, bc(0.0, 36.0, 720.0, 78.0), "Rank actions", 24.0);
    g.button2(btns, sz(320.0, 74.0), "Cancel rank", "Not Now");
    g.button(btns, sz(360.0, 74.0), "Promote button", "Promote to Tier IV");
}

/// Reparto de puntos de atributos con botones -/+ y contador de puntos restantes.
fn upgrade_stat_points(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "LEVEL UP: ALLOCATE POINTS", "Hero Level 16");
    let left_p = g.panel(p, left(60.0, 1140.0, 126.0, 44.0), "Allocation panel");
    let pool = g.card_on(left_p, tl(28.0, 24.0, 1084.0, 96.0), "Points pool");
    g.heading_l(pool, cl(28.0, 0.0, 540.0, 44.0), "Points heading", "Unspent Attribute Points", 32.0);
    g.badge(pool, cr(28.0, 0.0, 200.0, 54.0), "Points left", "6 LEFT");
    let list = g.vbox(left_p, tl(28.0, 140.0, 1084.0, 640.0), "Stat rows", 16.0);
    let stats = [
        ("Vigor", "Max Health & Stamina", "18", "+2"),
        ("Might", "Physical Attack Power", "22", "+1"),
        ("Focus", "Spell Power & Mana", "14", "+0"),
        ("Haste", "Speed & Dodge Window", "16", "+1"),
        ("Luck", "Crit Rate & Loot Drops", "12", "+0"),
    ];
    for (name, desc, base_v, added) in stats {
        let r = g.card(list, sz(1084.0, 114.0), &format!("{} alloc row", name));
        g.heading_l(r, tl(24.0, 20.0, 320.0, 38.0), "Alloc name", name, 28.0);
        g.note(r, tl(24.0, 62.0, 420.0, 32.0), "Alloc desc", desc, 22.0);
        g.value(r, tr(360.0, 34.0, 120.0, 44.0), "Alloc val", base_v, 34.0);
        g.badge(r, tr(240.0, 34.0, 96.0, 44.0), "Alloc delta", added);
        g.button2(r, cr(120.0, 0.0, 74.0, 72.0), &format!("Minus {}", name), "-");
        g.button(r, cr(24.0, 0.0, 74.0, 72.0), &format!("Plus {}", name), "+");
    }
    let acts = g.hbox(left_p, bl(28.0, 24.0, 1084.0, 72.0), "Alloc actions", 20.0);
    g.button2(acts, sz(260.0, 70.0), "Reset points", "Reset");
    g.button(acts, sz(320.0, 70.0), "Confirm points", "Confirm Changes");
    // Derecha: impacto derivado
    let right_p = g.panel(p, right(60.0, 630.0, 126.0, 44.0), "Derived stats panel");
    g.heading_l(right_p, tl(28.0, 28.0, 574.0, 44.0), "Derived title", "Stat Preview", 32.0);
    let derived = g.vbox(right_p, tl(28.0, 94.0, 574.0, 680.0), "Derived list", 16.0);
    for (k, v) in [
        ("Max Health", "620 -> 670 (+50)"),
        ("Attack Power", "94 -> 99 (+5)"),
        ("Spell Power", "64 -> 64"),
        ("Crit Chance", "12% -> 12%"),
        ("Move Speed", "108 -> 112 (+4)"),
        ("Carry Limit", "60 -> 64 (+4)"),
    ] {
        kv_row(g, derived, 574.0, 94.0, k, v);
    }
}

// ===============================================================================================================
// 15. Unlockable upgrades
// ===============================================================================================================

/// Árbol de mejoras organizado en tres columnas de nivel con líneas conectoras y detalle abajo.
fn unlock_tier_tree(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "TECH RESEARCH TREE", "Tech Points: 5");
    let tree = g.panel(p, fill(50.0, 126.0, 50.0, 240.0), "Tech tree board");
    // Encabezados de columnas Tier 1, Tier 2, Tier 3
    for (i, label) in ["TIER I - BASICS", "TIER II - ADVANCED", "TIER III - PROTOTYPE"].iter().enumerate() {
        let x = 60.0 + i as f32 * 580.0;
        let h = g.card(tree, tl(x, 22.0, 520.0, 54.0), "Tier header");
        g.heading(h, full(), "Tier title", label, 24.0);
    }
    // Conectores entre columnas
    g.rect(tree, tl(580.0, 194.0, 60.0, 8.0), "Link 1-2 top", g.t.line, 2.0);
    g.rect(tree, tl(1160.0, 194.0, 60.0, 8.0), "Link 2-3 top", g.t.line, 2.0);
    g.rect(tree, tl(580.0, 394.0, 60.0, 8.0), "Link 1-2 mid", g.t.line, 2.0);
    g.rect(tree, tl(1160.0, 394.0, 60.0, 8.0), "Link 2-3 mid", g.t.line, 2.0);
    g.rect(tree, tl(580.0, 594.0, 60.0, 8.0), "Link 1-2 bot", g.t.line, 2.0);
    g.rect(tree, tl(1160.0, 594.0, 60.0, 8.0), "Link 2-3 bot", g.t.line, 2.0);
    let nodes = [
        (0, 0, "Solar Cells", "+20% Power", "UNLOCKED", false),
        (1, 0, "Plasma Grid", "+40% Power", "2 TP", true),
        (2, 0, "Fusion Core", "Zero Drain", "LOCKED", false),
        (0, 1, "Composite Hull", "+150 Armor", "UNLOCKED", false),
        (1, 1, "Nano Repair", "Auto-Heal", "3 TP", false),
        (2, 1, "Phase Shield", "Block Hit", "LOCKED", false),
        (0, 2, "Wide Scanner", "+30% Range", "UNLOCKED", false),
        (1, 2, "Deep Probe", "Reveal Ore", "2 TP", false),
        (2, 2, "Orbital Link", "Map Reveal", "LOCKED", false),
    ];
    for (col, row, name, desc, status, sel) in nodes {
        let x = 60.0 + col as f32 * 580.0;
        let y = 114.0 + row as f32 * 200.0;
        let c = if sel { g.card_on(tree, tl(x, y, 520.0, 166.0), "Tech node") } else { g.card(tree, tl(x, y, 520.0, 166.0), "Tech node") };
        g.heading_l(c, tl(24.0, 22.0, 320.0, 38.0), "Tech name", name, 28.0);
        g.note(c, tl(24.0, 68.0, 320.0, 34.0), "Tech bonus", desc, 24.0);
        g.badge(c, cr(20.0, 0.0, 152.0, 44.0), "Tech state", status);
    }
    let foot = g.panel(p, bottom(30.0, 186.0, 50.0, 50.0), "Selected tech footer");
    g.heading_l(foot, tl(32.0, 24.0, 540.0, 44.0), "Focus tech", "Plasma Grid (Tier II)", 32.0);
    g.label(foot, tl(32.0, 80.0, 1140.0, 40.0), "Focus desc", "Upgrades all station relays for +40% energy capacity and unlocks Fusion Core.", 24.0);
    g.note(foot, tl(32.0, 128.0, 680.0, 32.0), "Focus req", "Requires: Solar Cells (Met)   |   Cost: 2 Tech Points", 22.0);
    g.button(foot, cr(32.0, 0.0, 280.0, 74.0), "Research button", "Unlock (2 TP)");
}

/// Lista con estado bloqueado/desbloqueado y requisitos a la izquierda, descripción a la derecha.
fn unlock_req_list(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "ALTAR OF RITES", "Blood Shards: 640");
    let left_p = g.panel(p, left(50.0, 1120.0, 126.0, 44.0), "Upgrades list panel");
    let rows = g.vbox(left_p, fill(28.0, 28.0, 28.0, 28.0), "Upgrades stack", 16.0);
    let list = [
        ("Iron Lungs", "Unlocked", "Sprint 25% longer in dark zones", true, false),
        ("Steady Hands", "Cost: 250 Shards", "Requires: Iron Lungs (Met)", false, true),
        ("Lantern Oil+", "Cost: 300 Shards", "Requires: Clear Crypt Wing (Met)", false, false),
        ("Bone Charm", "Locked", "Requires: Defeat Gatekeeper", false, false),
        ("Quiet Treads", "Locked", "Requires: Reach Survivor Lv 12", false, false),
        ("Last Breath", "Locked", "Requires: 3 Rites Unlocked", false, false),
    ];
    for (name, status, req, done, sel) in list {
        let c = if sel { g.card_on(rows, sz(1064.0, 128.0), "Rite card") } else { g.card(rows, sz(1064.0, 128.0), "Rite card") };
        g.icon(c, cl(20.0, 0.0, 64.0, 64.0), "Rite icon", if done { "OK" } else { "RT" });
        g.heading_l(c, tl(104.0, 22.0, 440.0, 40.0), "Rite name", name, 28.0);
        g.note(c, tl(104.0, 68.0, 560.0, 34.0), "Rite req", req, 22.0);
        if done || sel {
            g.badge(c, cr(24.0, 0.0, 220.0, 46.0), "Rite status", status);
        } else {
            g.value(c, cr(24.0, 0.0, 240.0, 38.0), "Rite status text", status, 22.0);
        }
    }
    let detail = g.panel(p, right(50.0, 670.0, 126.0, 44.0), "Rite detail panel");
    g.art(detail, tl(28.0, 28.0, 614.0, 260.0), "Rite emblem art");
    g.heading_l(detail, tl(28.0, 310.0, 614.0, 46.0), "Selected rite", "Steady Hands", 36.0);
    g.note(detail, tl(28.0, 366.0, 614.0, 110.0), "Selected desc", "Reduces aim sway by 40% while injured and speeds up reload by 15%.", 24.0);
    let reqs = g.vbox(detail, tl(28.0, 500.0, 614.0, 250.0), "Requirements list", 14.0);
    for (k, v) in [("Prerequisite", "Iron Lungs (Met)"), ("Shard Cost", "250 / 640"), ("Permanent", "Yes")] {
        kv_row(g, reqs, 614.0, 72.0, k, v);
    }
    g.button(detail, bl(28.0, 28.0, 614.0, 76.0), "Unlock rite", "Unlock Rite (250 Shards)");
}

/// Cartas de ventaja agrupadas en filas horizontales por nivel requerido.
fn unlock_level_rows(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "PERK MILESTONES", "Level 8   |   Unlocks Available: 2");
    let board = g.panel(p, fill(50.0, 126.0, 50.0, 44.0), "Milestone board");
    let rows = g.vbox(board, fill(28.0, 24.0, 28.0, 24.0), "Level rows", 22.0);
    let tiers = [
        ("LV 1", [("Quick Feet", "OWNED", false), ("Bright Eyes", "OWNED", false), ("Soft Landing", "UNLOCK", false)]),
        ("LV 5", [("Lucky Clover", "OWNED", false), ("Double Harvest", "READY", true), ("Fast Craft", "READY", false)]),
        ("LV 10", [("Cloud Glider", "LOCKED", false), ("Master Baker", "LOCKED", false), ("Star Magnet", "LOCKED", false)]),
    ];
    for (lv, perks) in tiers {
        let band = g.card(rows, sz(1764.0, 268.0), &format!("{} tier row", lv));
        g.badge(band, cl(24.0, 0.0, 140.0, 58.0), "Tier level", lv);
        let cards = g.hbox(band, fill(196.0, 22.0, 24.0, 22.0), "Tier perk cards", 24.0);
        for (name, st, sel) in perks {
            let c = if sel { g.card_on(cards, sz(498.0, 224.0), "Perk unlock card") } else { g.card(cards, sz(498.0, 224.0), "Perk unlock card") };
            g.icon(c, tl(20.0, 22.0, 64.0, 64.0), "Perk glyph", &name[..2]);
            g.heading_l(c, tl(102.0, 28.0, 376.0, 40.0), "Perk title", name, 28.0);
            g.note(c, tl(102.0, 74.0, 376.0, 34.0), "Perk sub", "Passive bonus active everywhere", 22.0);
            if st == "READY" {
                g.button(c, bl(20.0, 20.0, 220.0, 68.0), "Claim perk", "Unlock Now");
            } else {
                g.badge(c, bl(20.0, 24.0, 150.0, 44.0), "Perk state", st);
            }
        }
    }
}

// ===============================================================================================================
// 16. Crafting
// ===============================================================================================================

/// Tres columnas: lista de recetas, ingredientes con cantidades y resultado con botón Craft.
fn crafting_three_col(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "HERBALIST WORKBENCH", "Crafting Lv 12");
    // Columna 1: Recetas
    let c1 = g.panel(p, left(40.0, 520.0, 126.0, 40.0), "Recipes column");
    g.heading_l(c1, tl(24.0, 22.0, 472.0, 40.0), "Recipes heading", "1. Recipes", 28.0);
    let rlist = g.vbox(c1, tl(24.0, 78.0, 472.0, 790.0), "Recipe list", 14.0);
    for (i, (name, tag)) in [
        ("Sunleaf Elixir", "POTION"),
        ("Antidote Salve", "SALVE"),
        ("Oakwood Bow", "WEAPON"),
        ("Traveler Tent", "CAMP"),
        ("Lantern Oil", "SUPPLY"),
        ("Spiced Trail Mix", "FOOD"),
    ]
    .iter()
    .enumerate()
    {
        let r = if i == 0 { g.card_on(rlist, sz(472.0, 118.0), "Recipe row") } else { g.card(rlist, sz(472.0, 118.0), "Recipe row") };
        g.heading_l(r, tl(20.0, 20.0, 300.0, 38.0), "Recipe name", name, 26.0);
        g.note(r, tl(20.0, 64.0, 260.0, 32.0), "Recipe craftable", "Can craft: x2", 22.0);
        g.badge(r, cr(18.0, 0.0, 124.0, 42.0), "Recipe tag", tag);
    }
    // Columna 2: Ingredientes
    let c2 = g.panel(p, fill(584.0, 126.0, 644.0, 40.0), "Ingredients column");
    g.heading_l(c2, tl(24.0, 22.0, 600.0, 40.0), "Ing heading", "2. Ingredients Required", 28.0);
    let ilist = g.vbox(c2, tl(24.0, 78.0, 644.0, 660.0), "Ingredient list", 16.0);
    for (gly, name, have, need) in [
        ("SL", "Sunleaf Herb", "6", "3"),
        ("CW", "Clear Spring Water", "5", "2"),
        ("HN", "Wild Honey", "4", "1"),
        ("GV", "Glass Vial", "8", "1"),
    ] {
        let r = g.card(ilist, sz(644.0, 148.0), "Ingredient row");
        g.icon(r, cl(20.0, 0.0, 72.0, 72.0), "Ing icon", gly);
        g.heading_l(r, tl(112.0, 30.0, 340.0, 40.0), "Ing name", name, 26.0);
        g.note(r, tl(112.0, 76.0, 320.0, 32.0), "Ing status", "In backpack", 22.0);
        g.badge(r, cr(24.0, 0.0, 124.0, 46.0), "Ing qty", &format!("{} / {}", have, need));
    }
    g.note(c2, bl(24.0, 36.0, 644.0, 34.0), "Craft xp", "Crafting grants +35 Herbalism XP.", 24.0);
    // Columna 3: Resultado + Craft
    let c3 = g.panel(p, right(40.0, 580.0, 126.0, 40.0), "Result column");
    g.heading_l(c3, tl(28.0, 22.0, 524.0, 40.0), "Result heading", "3. Output Preview", 28.0);
    g.art(c3, tl(28.0, 80.0, 524.0, 290.0), "Crafted item art");
    g.heading_l(c3, tl(28.0, 392.0, 524.0, 44.0), "Result name", "Sunleaf Elixir (x1)", 34.0);
    g.note(c3, tl(28.0, 448.0, 524.0, 100.0), "Result desc", "Restores 350 health immediately and grants +15% stamina regen for 60 seconds.", 24.0);
    let qty_card = g.card(c3, bl(28.0, 124.0, 524.0, 92.0), "Quantity selector");
    g.label(qty_card, cl(20.0, 0.0, 200.0, 38.0), "Qty label", "Craft Amount", 26.0);
    g.button2(qty_card, cr(188.0, 0.0, 68.0, 68.0), "Decrease qty", "-");
    g.heading(qty_card, cr(108.0, 0.0, 72.0, 44.0), "Qty count", "1", 30.0);
    g.button2(qty_card, cr(20.0, 0.0, 68.0, 68.0), "Increase qty", "+");
    g.button(c3, bl(28.0, 28.0, 524.0, 76.0), "Craft button", "Craft Sunleaf Elixir");
}

/// Mesa de fabricación 3x3 a la izquierda, flecha y panel de resultado grande a la derecha.
fn crafting_table_3x3(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "CRAFTING TABLE", "Pattern Match: Steel Pickaxe");
    let top_p = g.panel(p, top(126.0, 600.0, 60.0, 60.0), "Crafting board");
    // Rejilla 3x3
    let grid = g.grid(top_p, cl(60.0, 0.0, 520.0, 520.0), "Crafting 3x3 grid", 160.0, 160.0, 18.0);
    let cells = ["IR", "IR", "IR", "", "ST", "", "", "ST", ""];
    for gly in cells {
        let s = g.slot(grid, sz(160.0, 160.0), "Table slot");
        if !gly.is_empty() {
            g.icon(s, cc(0.0, 0.0, 74.0, 74.0), "Placed mat", gly);
        }
    }
    // Flecha central
    g.icon(top_p, cc(-140.0, 0.0, 100.0, 100.0), "Result arrow", ">");
    // Panel de resultado grande a la derecha
    let out = g.card_on(top_p, cr(50.0, 0.0, 860.0, 520.0), "Output panel");
    g.art(out, tl(32.0, 32.0, 300.0, 300.0), "Output art");
    g.badge(out, tl(360.0, 36.0, 180.0, 44.0), "Output tier", "TOOL TIER II");
    g.heading_l(out, tl(360.0, 96.0, 460.0, 48.0), "Output name", "Steel Pickaxe", 38.0);
    g.note(out, tl(360.0, 156.0, 460.0, 90.0), "Output desc", "Mines iron, cobalt and quartz veins 40% faster. Durability: 250.", 24.0);
    g.label(out, tl(360.0, 262.0, 460.0, 38.0), "Output recipe", "Uses: 3x Iron Ingot, 2x Oak Stick", 24.0);
    let btns = g.hbox(out, bl(32.0, 32.0, 796.0, 74.0), "Craft actions", 20.0);
    g.button2(btns, sz(260.0, 72.0), "Clear table", "Clear Grid");
    g.button(btns, sz(516.0, 72.0), "Craft item", "Craft Steel Pickaxe");
    // Franja inferior de materiales disponibles
    let tray = g.panel(p, bottom(36.0, 290.0, 60.0, 60.0), "Materials tray");
    g.heading_l(tray, tl(28.0, 20.0, 420.0, 40.0), "Tray heading", "Quick Materials", 28.0);
    let mrow = g.hbox_c(tray, bottom(24.0, 196.0, 28.0, 28.0), "Quick mat slots", 22.0);
    for (gly, name, qty) in [("IR", "Iron Ingot", "x14"), ("ST", "Oak Stick", "x22"), ("WD", "Planks", "x30"), ("CL", "Coal", "x19"), ("CB", "Cobalt", "x5"), ("LT", "Leather", "x8")] {
        let c = g.card(mrow, sz(270.0, 186.0), "Material slot");
        g.icon(c, tc(0.0, 20.0, 64.0, 64.0), "Tray icon", gly);
        g.heading(c, tc(0.0, 94.0, 240.0, 36.0), "Tray name", name, 24.0);
        g.badge(c, bc(0.0, 14.0, 88.0, 40.0), "Tray qty", qty);
    }
}

/// Cola de fabricación con temporizadores y barras de progreso junto a recetas rápidas.
fn crafting_build_queue(g: &mut G) {
    let p = g.page();
    g.background();
    header_bar(g, "FABRICATION BAY", "Queue: 4 / 5   Power: 100%");
    // Izquierda: cola activa
    let q_p = g.panel(p, left(50.0, 1120.0, 126.0, 44.0), "Build queue panel");
    g.heading_l(q_p, tl(28.0, 24.0, 480.0, 42.0), "Queue title", "Active Production Queue", 32.0);
    let qlist = g.vbox(q_p, tl(28.0, 84.0, 1064.0, 780.0), "Queue jobs", 18.0);
    let jobs = [
        ("AM", "Armor Plate x5", "IN PROGRESS", "01:14 left", 0.68, true),
        ("MD", "Field Med-Pack x3", "QUEUED", "02:30", 0.0, false),
        ("DR", "Scout Drone x1", "QUEUED", "04:00", 0.0, false),
        ("FL", "Thermal Cell x10", "QUEUED", "01:45", 0.0, false),
    ];
    for (gly, name, st, time_s, prog, active) in jobs {
        let c = if active { g.card_on(qlist, sz(1064.0, 176.0), "Queue card") } else { g.card(qlist, sz(1064.0, 176.0), "Queue card") };
        g.icon(c, cl(22.0, 0.0, 76.0, 76.0), "Job icon", gly);
        g.heading_l(c, tl(118.0, 22.0, 460.0, 40.0), "Job name", name, 30.0);
        g.badge(c, tl(118.0, 70.0, 180.0, 42.0), "Job state", st);
        g.value(c, tr(220.0, 28.0, 220.0, 40.0), "Job timer", time_s, 26.0);
        g.bar(c, tl(118.0, 130.0, 720.0, 20.0), "Job progress", prog, if active { g.t.good } else { g.t.muted });
        g.button2(c, cr(22.0, 0.0, 176.0, 68.0), "Cancel job", if active { "Rush" } else { "Cancel" });
    }
    // Derecha: añadir a la cola
    let add_p = g.panel(p, right(50.0, 670.0, 126.0, 44.0), "Quick blueprints panel");
    g.heading_l(add_p, tl(28.0, 24.0, 520.0, 42.0), "Quick title", "Add Blueprint to Queue", 30.0);
    let blist = g.vbox(add_p, tl(28.0, 84.0, 614.0, 680.0), "Blueprint list", 16.0);
    for (name, cost, dur) in [
        ("Rifle Ammo Box", "40 Steel", "00:45"),
        ("Smoke Grenade x2", "25 Chem", "01:00"),
        ("Repair Kit", "60 Steel", "01:30"),
        ("Sensor Mine x2", "50 Alloy", "02:15"),
    ] {
        let c = g.card(blist, sz(614.0, 152.0), "Blueprint card");
        g.heading_l(c, tl(22.0, 20.0, 360.0, 38.0), "Bp name", name, 28.0);
        g.note(c, tl(22.0, 66.0, 340.0, 34.0), "Bp meta", &format!("Cost: {}   |   Time: {}", cost, dur), 22.0);
        g.button(c, cr(20.0, 0.0, 176.0, 68.0), "Queue button", "Queue +");
    }
    g.note(add_p, bl(28.0, 34.0, 614.0, 34.0), "Slot note", "Upgrade Fabrication Bay to unlock 8 queue slots.", 22.0);
}

// ===============================================================================================================
// 17. Choose boon / upgrade
// ===============================================================================================================

/// Tres cartas verticales grandes estilo roguelite para elegir una bendición.
fn boon_three_cards(g: &mut G) {
    let p = g.page();
    g.background();
    g.dim(150);
    let banner = g.panel(p, tc(0.0, 42.0, 980.0, 124.0), "Boon header");
    g.heading(banner, tl(24.0, 18.0, 932.0, 48.0), "Boon title", "CHOOSE A BLESSING", 42.0);
    g.txt(banner, bottom(16.0, 36.0, 24.0, 24.0), "Boon subtitle", "Select one permanent upgrade for this run", 24.0, g.t.muted, CENTER, false);
    let row = g.hbox_c(p, cc(0.0, 56.0, 1720.0, 740.0), "Boon cards row", 36.0);
    let boons = [
        ("FL", "RARE", "Ember Blade", "Your strikes ignite foes, dealing 28 burn damage over 3 seconds.", "+28 Burn DMG", false),
        ("WD", "EPIC", "Gale Dash", "Dodging unleashes a shockwave that pushes enemies back and resets attack speed.", "+30% Dash Speed", true),
        ("ST", "RARE", "Aegis Heart", "Gain +120 Max Health immediately and restore 25 health after every room.", "+120 Max HP", false),
    ];
    for (gly, rarity, name, desc, stat, feat) in boons {
        let c = if feat { g.card_on(row, sz(548.0, 720.0), "Boon card") } else { g.card(row, sz(548.0, 720.0), "Boon card") };
        g.badge(c, tc(0.0, 28.0, 170.0, 46.0), "Boon rarity", rarity);
        g.art(c, tc(0.0, 94.0, 468.0, 220.0), "Boon artwork");
        g.icon(c, tc(0.0, 274.0, 88.0, 88.0), "Boon emblem", gly);
        g.heading(c, tc(0.0, 384.0, 480.0, 48.0), "Boon name", name, 36.0);
        g.txt(c, tc(0.0, 444.0, 468.0, 110.0), "Boon desc", desc, 24.0, g.t.muted, CENTER, false);
        let pill = g.panel(c, tc(0.0, 564.0, 360.0, 52.0), "Boon highlight");
        g.heading(pill, full(), "Boon stat", stat, 24.0);
        if feat {
            g.button(c, bc(0.0, 28.0, 380.0, 74.0), &format!("Choose {}", name), "Choose Blessing");
        } else {
            g.button2(c, bc(0.0, 28.0, 380.0, 74.0), &format!("Choose {}", name), "Choose Blessing");
        }
    }
}

/// Cuatro opciones compactas en fila horizontal con iconos y botón Reroll abajo.
fn boon_four_row_reroll(g: &mut G) {
    let p = g.page();
    g.background();
    g.dim(150);
    let head = g.panel(p, top(36.0, 114.0, 80.0, 80.0), "Augment header");
    g.heading_l(head, cl(32.0, 0.0, 720.0, 48.0), "Augment title", "SELECT MODULE UPGRADE", 38.0);
    g.badge(head, cr(32.0, 0.0, 220.0, 48.0), "Wave badge", "WAVE 6 CLEAR");
    let wrap = g.panel(p, fill(80.0, 178.0, 80.0, 160.0), "Options frame");
    let row = g.hbox_c(wrap, cc(0.0, 0.0, 1680.0, 660.0), "Augment row", 24.0);
    let opts = [
        ("OV", "Overdrive", "+20% Fire Rate", "Weapon shots heat up faster.", false),
        ("PL", "Pulse Shield", "+80 Shield Cap", "Recharges 2 seconds sooner.", true),
        ("CR", "Crit Optics", "+15% Crit Rate", "Weakspot hits deal +25%.", false),
        ("MG", "Magnet Core", "+50% Pickup Range", "Pulls energy shards faster.", false),
    ];
    for (gly, name, main_s, sub_s, sel) in opts {
        let c = if sel { g.card_on(row, sz(400.0, 620.0), "Augment card") } else { g.card(row, sz(400.0, 620.0), "Augment card") };
        g.icon(c, tc(0.0, 44.0, 112.0, 112.0), "Augment icon", gly);
        g.heading(c, tc(0.0, 184.0, 352.0, 44.0), "Augment name", name, 32.0);
        g.badge(c, tc(0.0, 244.0, 260.0, 48.0), "Augment bonus", main_s);
        g.txt(c, tc(0.0, 324.0, 344.0, 120.0), "Augment desc", sub_s, 24.0, g.t.muted, CENTER, false);
        if sel {
            g.button(c, bc(0.0, 32.0, 320.0, 74.0), &format!("Pick {}", name), "Install");
        } else {
            g.button2(c, bc(0.0, 32.0, 320.0, 74.0), &format!("Pick {}", name), "Install");
        }
    }
    let foot = g.panel(p, bottom(36.0, 96.0, 80.0, 80.0), "Reroll bar");
    g.note(foot, cl(32.0, 0.0, 660.0, 38.0), "Reroll hint", "Reroll replaces all 4 modules with fresh options.", 24.0);
    let acts = g.hbox(foot, cr(28.0, 0.0, 560.0, 70.0), "Reroll actions", 20.0);
    g.button2(acts, sz(270.0, 68.0), "Skip button", "Skip (+50 Credits)");
    g.button(acts, sz(270.0, 68.0), "Reroll button", "Reroll (2 Tokens)");
}

/// Retrato del benefactor con cita a la izquierda y tres opciones en lista vertical a la derecha.
fn boon_patron_list(g: &mut G) {
    let p = g.page();
    g.background();
    g.dim(150);
    let modal = g.panel(p, cc(0.0, 0.0, 1660.0, 880.0), "Patron offering modal");
    // Izquierda: benefactor
    let patron = g.card(modal, tl(36.0, 36.0, 560.0, 808.0), "Patron column");
    g.badge(patron, tl(24.0, 24.0, 210.0, 44.0), "Patron tag", "ORACLE OF DUSK");
    g.art(patron, tl(24.0, 88.0, 512.0, 460.0), "Patron portrait");
    g.heading_l(patron, tl(24.0, 570.0, 512.0, 44.0), "Patron name", "Lady Vespera", 34.0);
    g.note(patron, tl(24.0, 624.0, 512.0, 150.0), "Patron quote", "\"Three threads of fate lie before you. Claim one gift, and walk on with grace.\"", 24.0);
    // Derecha: 3 opciones en lista vertical
    g.heading_l(modal, tl(632.0, 36.0, 720.0, 48.0), "Offer heading", "Choose Your Gift", 38.0);
    g.note(modal, tl(632.0, 90.0, 720.0, 34.0), "Offer sub", "Select one boon to empower your current expedition.", 24.0);
    let list = g.vbox(modal, tl(632.0, 150.0, 992.0, 600.0), "Gift options", 22.0);
    let gifts = [
        ("VS", "Veil of Silver", "EPIC", "Gain a barrier that absorbs the first hit in every encounter.", true),
        ("CH", "Chalice of Vigor", "RARE", "Restore 50% Health now and increase potion healing by 25%.", false),
        ("ST", "Starlit Step", "RARE", "Your dash distance increases by 20% and costs zero stamina.", false),
    ];
    for (gly, name, rarity, desc, sel) in gifts {
        let c = if sel { g.card_on(list, sz(992.0, 184.0), "Gift option") } else { g.card(list, sz(992.0, 184.0), "Gift option") };
        g.icon(c, cl(24.0, 0.0, 84.0, 84.0), "Gift icon", gly);
        g.heading_l(c, tl(132.0, 24.0, 460.0, 40.0), "Gift name", name, 30.0);
        g.badge(c, tr(220.0, 24.0, 130.0, 42.0), "Gift rarity", rarity);
        g.note(c, tl(132.0, 76.0, 620.0, 82.0), "Gift desc", desc, 24.0);
        if sel {
            g.button(c, cr(24.0, 0.0, 174.0, 72.0), &format!("Accept {}", name), "Accept");
        } else {
            g.button2(c, cr(24.0, 0.0, 174.0, 72.0), &format!("Accept {}", name), "Accept");
        }
    }
    g.note(modal, bl(632.0, 44.0, 660.0, 34.0), "Decline hint", "Declining grants 150 Gold instead of a blessing.", 22.0);
    g.button2(modal, br(36.0, 32.0, 240.0, 68.0), "Decline button", "Take 150 Gold");
}
