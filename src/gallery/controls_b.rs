//! Plantillas de controles y superposiciones auxiliares.

use super::*;

pub fn entries() -> Vec<Entry> {
    use Style::*;
    vec![
        Entry { ty: "Button prompts", style: Fantasy, build: prompts_corner },
        Entry { ty: "Button prompts", style: Minimal, build: prompts_hold },
        Entry { ty: "Button prompts", style: Cartoon, build: prompts_tutorial },
        Entry { ty: "Weapon pickup prompt", style: Military, build: pickup_compare },
        Entry { ty: "Weapon pickup prompt", style: SciFi, build: pickup_small },
        Entry { ty: "Weapon pickup prompt", style: Retro, build: pickup_swap },
        Entry { ty: "Quick time event", style: Sports, build: qte_ring },
        Entry { ty: "Quick time event", style: Horror, build: qte_sequence },
        Entry { ty: "Quick time event", style: Neon, build: qte_mash },
        Entry { ty: "Controls reference", style: Pastel, build: reference_pad },
        Entry { ty: "Controls reference", style: Military, build: reference_keyboard },
        Entry { ty: "Controls reference", style: Elegant, build: reference_categories },
        Entry { ty: "Item & ability buttons", style: Cartoon, build: ability_console },
        Entry { ty: "Item & ability buttons", style: Fantasy, build: ability_cooldowns },
        Entry { ty: "Item & ability buttons", style: SciFi, build: ability_touchbar },
        Entry { ty: "Mobile controls", style: Nature, build: mobile_joystick },
        Entry { ty: "Mobile controls", style: Pastel, build: mobile_zones },
        Entry { ty: "Mobile controls", style: Minimal, build: mobile_gestures },
        Entry { ty: "Spectral vision", style: Horror, build: spectral_highlights },
        Entry { ty: "Spectral vision", style: SciFi, build: spectral_scan },
        Entry { ty: "Spectral vision", style: Neon, build: spectral_pulse },
        Entry { ty: "Weapon reticles", style: Military, build: reticle_cross },
        Entry { ty: "Weapon reticles", style: Elegant, build: reticle_scope },
        Entry { ty: "Weapon reticles", style: Sports, build: reticle_ammo },
        Entry { ty: "Object / NPC info", style: Fantasy, build: info_float },
        Entry { ty: "Object / NPC info", style: Nature, build: info_inspect },
        Entry { ty: "Object / NPC info", style: Retro, build: info_nameplate },
        Entry { ty: "Minigame", style: Nature, build: minigame_fishing },
        Entry { ty: "Minigame", style: Neon, build: minigame_rhythm },
        Entry { ty: "Minigame", style: Elegant, build: minigame_cards },
        Entry { ty: "Hacking & lockpicking", style: SciFi, build: hacking_circuit },
        Entry { ty: "Hacking & lockpicking", style: Horror, build: hacking_dial },
        Entry { ty: "Hacking & lockpicking", style: Retro, build: hacking_keypad },
    ]
}

fn key_action(g: &mut G, p: N, x: f32, y: f32, key: &str, action: &str) {
    let row = g.card(p, tl(x, y, 320.0, 75.0), &format!("{} prompt", action));
    g.key(row, cl(12.0, 0.0, 52.0, 52.0), &format!("{} key", action), key);
    g.label(row, cl(80.0, 0.0, 225.0, 48.0), &format!("{} action", action), action, 26.0);
}

fn reticle(g: &mut G, p: N, x: f32, y: f32) {
    for (i, (dx, dy, w, h)) in [(0.0, -54.0, 7.0, 38.0), (0.0, 54.0, 7.0, 38.0), (-54.0, 0.0, 38.0, 7.0), (54.0, 0.0, 38.0, 7.0)].iter().enumerate() {
        g.rect(p, cc(x + dx, y + dy, *w, *h), &format!("Reticle mark {}", i + 1), g.t.accent, 0.0);
    }
    g.circle(p, cc(x, y, 12.0, 12.0), "Reticle center", g.t.accent);
}

/// Indicaciones apiladas en la esquina inferior derecha.
fn prompts_corner(g: &mut G) {
    let p = g.page(); g.scene();
    let box_ = g.panel(p, br(50.0, 50.0, 385.0, 365.0), "Nearby actions panel");
    g.heading(box_, top(16.0, 55.0, 15.0, 15.0), "Actions title", "Nearby actions", 33.0);
    for (i, (key, action)) in [("E", "Open gate"), ("F", "Inspect sign"), ("Q", "Switch tool")].iter().enumerate() { key_action(g, box_, 32.0, 85.0 + i as f32 * 88.0, key, action); }
}

/// Tecla central con barra para mantener pulsado.
fn prompts_hold(g: &mut G) {
    let p = g.page(); g.scene(); g.art(p, cc(0.0, -150.0, 260.0, 220.0), "Supply crate");
    let box_ = g.panel(p, cc(0.0, 140.0, 650.0, 260.0), "Hold action panel");
    g.key(box_, tl(40.0, 42.0, 110.0, 110.0), "Hold key", "E");
    g.heading_l(box_, tl(175.0, 40.0, 440.0, 62.0), "Action title", "Open supply crate", 39.0);
    g.note(box_, tl(175.0, 105.0, 440.0, 48.0), "Hold instruction", "Hold to open", 28.0);
    g.bar(box_, bottom(30.0, 25.0, 40.0, 40.0), "Hold progress", 0.62, g.t.accent);
    let hint = g.panel(p, bc(0.0, 44.0, 440.0, 70.0), "Cancel hint panel");
    g.heading(hint, full(), "Cancel hint", "Release to cancel", 27.0);
}

/// Lección lateral que señala una tecla de herramienta.
fn prompts_tutorial(g: &mut G) {
    let p = g.page(); g.scene();
    let lesson = g.panel(p, cl(50.0, 0.0, 620.0, 420.0), "Tutorial lesson panel");
    g.badge(lesson, tl(30.0, 25.0, 150.0, 46.0), "Step badge", "STEP 2/4");
    g.heading_l(lesson, tl(30.0, 95.0, 555.0, 70.0), "Lesson heading", "Use your tools", 45.0);
    g.label(lesson, tl(30.0, 180.0, 555.0, 110.0), "Lesson text", "Press X to equip the scanner, then point it at the marked door.", 29.0);
    g.bar(lesson, bottom(65.0, 17.0, 30.0, 30.0), "Lesson progress", 0.5, g.t.accent);
    g.button(lesson, br(30.0, 17.0, 190.0, 60.0), "Skip lesson button", "Skip");
    let action = g.panel(p, cr(150.0, 0.0, 390.0, 210.0), "Scanner action panel");
    g.key(action, tc(0.0, 23.0, 100.0, 100.0), "Scanner key", "X");
    g.heading(action, bottom(22.0, 55.0, 18.0, 18.0), "Scanner action", "Equip scanner", 31.0);
}

/// Compara el arma equipada y la encontrada en dos tarjetas.
fn pickup_compare(g: &mut G) {
    let p = g.page(); g.scene();
    let panel = g.panel(p, cc(0.0, 110.0, 910.0, 470.0), "Weapon comparison panel");
    g.heading(panel, top(18.0, 60.0, 25.0, 25.0), "Comparison title", "Weapon found", 42.0);
    for (i, (name, damage, rate)) in [("Equipped carbine", "Damage 48", "Rate 72"), ("Pulse carbine", "Damage 62", "Rate 60")].iter().enumerate() {
        let card = g.card(panel, tl(28.0 + i as f32 * 452.0, 100.0, 420.0, 265.0), &format!("{} card", name));
        g.art(card, tl(20.0, 20.0, 120.0, 115.0), &format!("{} art", name));
        g.heading_l(card, tl(155.0, 23.0, 250.0, 65.0), &format!("{} name", name), name, 28.0);
        g.label(card, tl(20.0, 150.0, 370.0, 44.0), &format!("{} damage", name), damage, 27.0);
        g.label(card, tl(20.0, 202.0, 370.0, 44.0), &format!("{} rate", name), rate, 27.0);
    }
    g.button(panel, bl(30.0, 18.0, 300.0, 65.0), "Pick up button", "Pick up");
    g.button2(panel, br(30.0, 18.0, 300.0, 65.0), "Leave button", "Leave");
}

/// Aviso compacto bajo un arma del suelo.
fn pickup_small(g: &mut G) {
    let p = g.page(); g.scene(); g.art(p, cc(0.0, -70.0, 340.0, 180.0), "Ground rifle");
    let panel = g.panel(p, cc(0.0, 135.0, 580.0, 150.0), "Pickup prompt panel");
    g.key(panel, cl(24.0, 0.0, 90.0, 90.0), "Pickup key", "E");
    g.heading_l(panel, tl(140.0, 22.0, 410.0, 55.0), "Weapon name", "Compact rifle", 37.0);
    g.note(panel, tl(140.0, 82.0, 410.0, 43.0), "Weapon ammunition", "Ammo: 24 / 96", 27.0);
    let current = g.panel(p, br(48.0, 45.0, 320.0, 100.0), "Current weapon panel");
    g.note(current, top(10.0, 37.0, 17.0, 17.0), "Current label", "CURRENT WEAPON", 23.0);
    g.heading(current, bottom(13.0, 45.0, 17.0, 17.0), "Current name", "Sidearm", 30.0);
}

/// Dos armas a los lados de una tecla de intercambio.
fn pickup_swap(g: &mut G) {
    let p = g.page(); g.scene();
    let panel = g.panel(p, bc(0.0, 55.0, 1160.0, 310.0), "Weapon swap panel");
    g.heading(panel, top(15.0, 53.0, 20.0, 20.0), "Swap title", "Swap weapon", 39.0);
    for (i, name) in ["Old blaster", "Arc launcher"].iter().enumerate() {
        let card = g.card(panel, tl(25.0 + i as f32 * 650.0, 92.0, 480.0, 180.0), &format!("{} card", name));
        g.art(card, tl(18.0, 20.0, 145.0, 135.0), &format!("{} art", name));
        g.heading_l(card, tl(182.0, 25.0, 275.0, 54.0), &format!("{} name", name), name, 32.0);
        g.note(card, tl(182.0, 91.0, 275.0, 45.0), &format!("{} power", name), if i == 0 { "Power 42" } else { "Power 68" }, 27.0);
    }
    g.key(panel, cc(0.0, 35.0, 90.0, 90.0), "Swap key", "E");
    g.badge(panel, bc(0.0, 12.0, 190.0, 42.0), "Swap hint", "HOLD TO SWAP");
}

/// Botón central dentro de un anillo de tiempo.
fn qte_ring(g: &mut G) {
    let p = g.page(); g.scene();
    let panel = g.panel(p, cc(0.0, 0.0, 510.0, 510.0), "Timing panel");
    g.circle(panel, cc(0.0, -25.0, 300.0, 300.0), "Timing ring", g.t.line);
    g.circle(panel, cc(0.0, -25.0, 240.0, 240.0), "Timing inner", g.t.panel2);
    g.key(panel, cc(0.0, -25.0, 130.0, 130.0), "Timing key", "A");
    g.heading(panel, bottom(36.0, 58.0, 20.0, 20.0), "Timing instruction", "Press now!", 40.0);
    g.bar(panel, bottom(15.0, 12.0, 65.0, 65.0), "Time remaining", 0.38, g.t.accent);
    let hint = g.panel(p, bc(0.0, 50.0, 490.0, 72.0), "Event hint panel");
    g.heading(hint, full(), "Event hint", "Catch the falling crate", 27.0);
}

/// Secuencia horizontal de teclas con paso activo.
fn qte_sequence(g: &mut G) {
    let p = g.page(); g.scene();
    let panel = g.panel(p, cc(0.0, 50.0, 990.0, 340.0), "Sequence panel");
    g.heading(panel, top(22.0, 60.0, 20.0, 20.0), "Sequence heading", "Escape the grasp", 42.0);
    for (i, key) in ["W", "A", "S", "D", "E"].iter().enumerate() {
        let card = if i == 2 { g.card_on(panel, tl(38.0 + i as f32 * 186.0, 115.0, 164.0, 140.0), &format!("Step {} card", i + 1)) } else { g.card(panel, tl(38.0 + i as f32 * 186.0, 115.0, 164.0, 140.0), &format!("Step {} card", i + 1)) };
        g.key(card, tc(0.0, 17.0, 88.0, 88.0), &format!("Step {} key", i + 1), key);
        g.note(card, bottom(8.0, 29.0, 12.0, 12.0), &format!("Step {} status", i + 1), if i < 2 { "DONE" } else if i == 2 { "NEXT" } else { "WAIT" }, 22.0);
    }
    g.bar(panel, bottom(27.0, 18.0, 40.0, 40.0), "Sequence timer", 0.58, g.t.accent);
}

/// Barra de pulsaciones con tecla grande y contador.
fn qte_mash(g: &mut G) {
    let p = g.page(); g.scene();
    let panel = g.panel(p, cc(0.0, 80.0, 710.0, 395.0), "Mash panel");
    g.badge(panel, tc(0.0, 28.0, 240.0, 48.0), "Urgency badge", "HURRY");
    g.heading(panel, top(86.0, 56.0, 25.0, 25.0), "Mash instruction", "Break free!", 44.0);
    g.key(panel, tc(0.0, 151.0, 112.0, 112.0), "Mash key", "X");
    g.bar(panel, bottom(63.0, 28.0, 56.0, 56.0), "Mash progress", 0.72, g.t.accent);
    g.note(panel, bottom(20.0, 38.0, 56.0, 56.0), "Mash count", "18 / 25 presses", 28.0);
    let timer = g.panel(p, tc(0.0, 54.0, 330.0, 75.0), "Countdown panel");
    g.heading(timer, full(), "Seconds left", "03.2", 44.0);
}

/// Mando centrado con llamadas de botones a izquierda y derecha.
fn reference_pad(g: &mut G) {
    let p = g.page(); g.background();
    let panel = g.panel(p, cc(0.0, 0.0, 1640.0, 900.0), "Controller guide panel");
    g.heading(panel, top(25.0, 70.0, 30.0, 30.0), "Guide heading", "Controller guide", 50.0);
    g.art(panel, cc(0.0, 0.0, 620.0, 450.0), "Controller diagram");
    for (i, (key, action)) in [("L", "Move"), ("LT", "Aim"), ("LB", "Tools"), ("R", "Look")].iter().enumerate() { key_action(g, panel, 40.0, 150.0 + i as f32 * 130.0, key, action); }
    for (i, (key, action)) in [("A", "Jump"), ("B", "Dodge"), ("X", "Interact"), ("Y", "Reload")].iter().enumerate() { key_action(g, panel, 1270.0, 150.0 + i as f32 * 130.0, key, action); }
    g.button(panel, bc(0.0, 25.0, 250.0, 65.0), "Close guide button", "Close");
}

/// Lista de teclado en dos columnas temáticas.
fn reference_keyboard(g: &mut G) {
    let p = g.page(); g.background();
    let panel = g.panel(p, cc(0.0, 0.0, 1380.0, 900.0), "Keyboard guide panel");
    g.heading(panel, top(30.0, 65.0, 35.0, 35.0), "Keyboard heading", "Keyboard controls", 48.0);
    for (col, items) in [[("W", "Forward"), ("S", "Back"), ("A", "Left"), ("D", "Right")], [("Space", "Jump"), ("Shift", "Sprint"), ("E", "Interact"), ("Tab", "Inventory")]].iter().enumerate() {
        let side = g.card(panel, tl(40.0 + col as f32 * 680.0, 130.0, 640.0, 650.0), &format!("Column {}", col + 1));
        g.heading_l(side, tl(25.0, 20.0, 580.0, 55.0), &format!("Column {} heading", col + 1), if col == 0 { "Movement" } else { "Actions" }, 35.0);
        for (i, (key, action)) in items.iter().enumerate() { key_action(g, side, 25.0, 110.0 + i as f32 * 125.0, key, action); }
    }
    g.button(panel, bc(0.0, 20.0, 250.0, 65.0), "Back button", "Back");
}

/// Tabla por categorías con pares de tecla y acción.
fn reference_categories(g: &mut G) {
    let p = g.page(); g.background();
    let panel = g.panel(p, cc(0.0, 0.0, 1510.0, 900.0), "Controls table panel");
    g.heading_l(panel, tl(45.0, 32.0, 1000.0, 70.0), "Table heading", "Controls reference", 49.0);
    for (i, (heading, rows)) in [("Movement", [("W A S D", "Move"), ("Space", "Jump")]), ("Combat", [("Mouse 1", "Attack"), ("R", "Reload")]), ("Other", [("E", "Interact"), ("M", "Map")])].iter().enumerate() {
        let section = g.card(panel, tl(55.0, 135.0 + i as f32 * 225.0, 1400.0, 200.0), &format!("{} section", heading));
        g.heading_l(section, tl(25.0, 20.0, 320.0, 50.0), &format!("{} heading", heading), heading, 34.0);
        for (j, (key, action)) in rows.iter().enumerate() { key_action(g, section, 375.0 + j as f32 * 490.0, 76.0, key, action); }
    }
    g.button(panel, br(55.0, 25.0, 240.0, 65.0), "Close table button", "Close");
}

/// Dos grupos de botones de consola a ambos lados.
fn ability_console(g: &mut G) {
    let p = g.page(); g.scene();
    for (side, items) in [[("X", "Attack"), ("Y", "Heavy"), ("B", "Dodge")], [("LB", "Potion"), ("RB", "Skill"), ("A", "Jump")]].iter().enumerate() {
        let x = if side == 0 { 45.0 } else { 1455.0 };
        let panel = g.panel(p, tl(x, 650.0, 420.0, 350.0), &format!("Action group {}", side + 1));
        g.heading(panel, top(15.0, 50.0, 20.0, 20.0), &format!("Group {} heading", side + 1), if side == 0 { "Combat" } else { "Utility" }, 31.0);
        for (i, (key, action)) in items.iter().enumerate() { key_action(g, panel, 35.0, 75.0 + i as f32 * 88.0, key, action); }
    }
    let status = g.panel(p, bc(0.0, 50.0, 430.0, 90.0), "Ability status panel");
    g.heading(status, full(), "Ability status", "All skills ready", 30.0);
}

/// Cuatro habilidades con barras individuales de enfriamiento.
fn ability_cooldowns(g: &mut G) {
    let p = g.page(); g.scene();
    let panel = g.panel(p, bc(0.0, 35.0, 930.0, 215.0), "Cooldown bar panel");
    for (i, (key, name, level)) in [("1", "Flame", 1.0), ("2", "Ward", 0.7), ("3", "Blink", 0.3), ("4", "Nova", 0.0)].iter().enumerate() {
        let card = g.card(panel, tl(22.0 + i as f32 * 226.0, 22.0, 204.0, 165.0), &format!("{} ability card", name));
        g.icon(card, tc(0.0, 15.0, 75.0, 75.0), &format!("{} icon", name), key);
        g.heading(card, top(96.0, 39.0, 12.0, 12.0), &format!("{} name", name), name, 25.0);
        g.bar(card, bottom(12.0, 12.0, 16.0, 16.0), &format!("{} cooldown", name), *level, g.t.accent);
    }
    let note = g.panel(p, tc(0.0, 35.0, 430.0, 76.0), "Ready ability panel");
    g.heading(note, full(), "Ready ability", "Flame is ready", 30.0);
}

/// Barra táctil ancha con cinco acciones.
fn ability_touchbar(g: &mut G) {
    let p = g.page(); g.scene();
    let panel = g.panel(p, bottom(25.0, 210.0, 170.0, 170.0), "Touch action bar");
    for (i, (glyph, name)) in [("+", "Heal"), ("!", "Shield"), ("x", "Pulse"), (">", "Dash"), ("1", "Item")].iter().enumerate() {
        let card = g.card(panel, tl(25.0 + i as f32 * 305.0, 20.0, 280.0, 165.0), &format!("{} touch card", name));
        g.icon(card, tc(0.0, 15.0, 88.0, 88.0), &format!("{} icon", name), glyph);
        g.heading(card, bottom(12.0, 46.0, 10.0, 10.0), &format!("{} label", name), name, 28.0);
    }
    let selected = g.panel(p, cr(45.0, -135.0, 330.0, 105.0), "Selected item panel");
    g.heading(selected, top(10.0, 46.0, 15.0, 15.0), "Selected item", "Med kit x3", 30.0);
    g.note(selected, bottom(10.0, 36.0, 15.0, 15.0), "Item hint", "Tap to switch", 24.0);
}

/// Joystick a la izquierda y tres botones de acción a la derecha.
fn mobile_joystick(g: &mut G) {
    let p = g.page(); g.scene();
    let stick = g.panel(p, bl(55.0, 55.0, 340.0, 340.0), "Joystick panel");
    g.circle(stick, cc(0.0, 0.0, 270.0, 270.0), "Joystick ring", alpha(g.t.accent, 100));
    g.circle(stick, cc(20.0, -15.0, 110.0, 110.0), "Joystick thumb", g.t.accent);
    let actions = g.panel(p, br(55.0, 55.0, 520.0, 355.0), "Mobile actions panel");
    for (i, (key, name)) in [("A", "Attack"), ("J", "Jump"), ("D", "Dash")].iter().enumerate() {
        let card = g.card(actions, tl(25.0 + i as f32 * 165.0, 85.0, 150.0, 170.0), &format!("{} action card", name));
        g.icon(card, tc(0.0, 20.0, 95.0, 95.0), &format!("{} icon", name), key);
        g.heading(card, bottom(15.0, 38.0, 8.0, 8.0), &format!("{} label", name), name, 24.0);
    }
    let hint = g.panel(p, tc(0.0, 50.0, 460.0, 85.0), "Mobile hint panel");
    g.heading(hint, full(), "Mobile hint", "Drag to move", 31.0);
}

/// Zonas grandes de toque para mover y mirar.
fn mobile_zones(g: &mut G) {
    let p = g.page(); g.scene();
    for (i, (name, hint)) in [("Move zone", "Drag left thumb"), ("Look zone", "Drag right thumb")].iter().enumerate() {
        let zone = g.panel(p, tl(55.0 + i as f32 * 965.0, 250.0, 850.0, 600.0), name);
        g.heading(zone, top(28.0, 60.0, 25.0, 25.0), &format!("{} heading", name), name, 42.0);
        g.circle(zone, cc(0.0, 20.0, 220.0, 220.0), &format!("{} ring", name), alpha(g.t.accent, 120));
        g.circle(zone, cc(0.0, 20.0, 75.0, 75.0), &format!("{} point", name), g.t.accent);
        g.note(zone, bottom(32.0, 48.0, 40.0, 40.0), &format!("{} hint", name), hint, 28.0);
    }
    let action = g.panel(p, bc(0.0, 30.0, 390.0, 100.0), "Tap action panel");
    g.heading(action, full(), "Tap action", "Tap to interact", 31.0);
}

/// Tres gestos en tarjetas y un botón de pausa aislado.
fn mobile_gestures(g: &mut G) {
    let p = g.page(); g.scene();
    let head = g.panel(p, tc(0.0, 50.0, 650.0, 95.0), "Gesture heading panel");
    g.heading(head, full(), "Gesture heading", "Touch gestures", 46.0);
    for (i, (glyph, title, detail)) in [("<", "Swipe left", "Change target"), (">", "Swipe right", "Switch item"), ("+", "Double tap", "Quick dodge")].iter().enumerate() {
        let card = g.panel(p, tl(170.0 + i as f32 * 535.0, 440.0, 485.0, 280.0), &format!("{} card", title));
        g.icon(card, tc(0.0, 25.0, 105.0, 105.0), &format!("{} icon", title), glyph);
        g.heading(card, top(145.0, 55.0, 20.0, 20.0), &format!("{} title", title), title, 37.0);
        g.note(card, bottom(22.0, 42.0, 20.0, 20.0), &format!("{} detail", title), detail, 26.0);
    }
    g.button2(p, tr(44.0, 40.0, 180.0, 72.0), "Pause button", "Pause");
}

/// Tinte espectral con tres objetos resaltados y leyenda.
fn spectral_highlights(g: &mut G) {
    let p = g.page(); g.scene();
    g.el(p, full(), Kind::Panel, "Spectral tint", |e| { e.fill = [55, 20, 85, 95]; e.raycast = false; });
    for (i, (x, y, name)) in [(280.0, 280.0, "Hidden gate"), (880.0, 390.0, "Trace"), (1440.0, 260.0, "Relic")].iter().enumerate() {
        g.circle(p, tl(*x, *y, 170.0, 170.0), &format!("Spectral halo {}", i + 1), alpha(g.t.accent, 105));
        let tag = g.panel(p, tl(*x - 40.0, *y + 190.0, 250.0, 62.0), &format!("{} label panel", name));
        g.heading(tag, full(), &format!("{} label", name), name, 28.0);
    }
    let legend = g.panel(p, bc(0.0, 45.0, 680.0, 100.0), "Vision legend panel");
    g.heading(legend, top(8.0, 43.0, 15.0, 15.0), "Vision heading", "Spectral sight", 34.0);
    g.note(legend, bottom(8.0, 38.0, 15.0, 15.0), "Vision hint", "Follow the bright traces", 26.0);
}

/// Escaneo lateral con ficha del objetivo y progreso.
fn spectral_scan(g: &mut G) {
    let p = g.page(); g.scene();
    g.el(p, full(), Kind::Panel, "Scan tint", |e| { e.fill = [0, 65, 85, 70]; e.raycast = false; });
    let target = g.art(p, cc(-210.0, -50.0, 370.0, 390.0), "Scanned device");
    g.rect(target, top(12.0, 6.0, 20.0, 20.0), "Device scan edge", g.t.accent, 0.0);
    let scan = g.panel(p, tr(55.0, 110.0, 525.0, 665.0), "Scan details panel");
    g.badge(scan, tl(30.0, 28.0, 170.0, 45.0), "Scan status", "SCANNING");
    g.heading_l(scan, tl(30.0, 100.0, 455.0, 65.0), "Scan target", "Power relay", 42.0);
    for (i, (name, value)) in [("Signal", "Strong"), ("Status", "Offline"), ("Access", "Restricted")].iter().enumerate() {
        let row = g.card(scan, tl(30.0, 195.0 + i as f32 * 110.0, 465.0, 87.0), &format!("{} scan row", name));
        g.label(row, cl(20.0, 0.0, 220.0, 46.0), &format!("{} label", name), name, 27.0);
        g.value(row, cr(20.0, 0.0, 190.0, 46.0), &format!("{} value", name), value, 26.0);
    }
    g.bar(scan, bottom(76.0, 20.0, 30.0, 30.0), "Scan progress", 0.76, g.t.accent);
    g.button(scan, bc(0.0, 20.0, 270.0, 62.0), "Analyze button", "Analyze");
}

/// Pulso radial con anillos y ecos detectados.
fn spectral_pulse(g: &mut G) {
    let p = g.page(); g.scene();
    g.el(p, full(), Kind::Panel, "Pulse tint", |e| { e.fill = [15, 0, 60, 85]; e.raycast = false; });
    for (i, size) in [610.0, 460.0, 300.0].iter().enumerate() { g.circle(p, cc(0.0, 0.0, *size, *size), &format!("Pulse ring {}", i + 1), alpha(g.t.accent, 35 + i as u8 * 18)); }
    g.icon(p, cc(0.0, 0.0, 105.0, 105.0), "Pulse source", "+");
    for (i, (x, y)) in [(-210.0, -150.0), (165.0, -70.0), (135.0, 180.0)].iter().enumerate() { g.circle(p, cc(*x, *y, 30.0, 30.0), &format!("Echo {}", i + 1), g.t.warn); }
    let panel = g.panel(p, bc(0.0, 40.0, 535.0, 95.0), "Pulse status panel");
    g.heading(panel, top(9.0, 40.0, 12.0, 12.0), "Pulse status", "3 echoes nearby", 32.0);
    g.note(panel, bottom(8.0, 32.0, 12.0, 12.0), "Pulse range", "Range: 25 m", 24.0);
}

/// Cruz de tiro con marcador de impacto y munición.
fn reticle_cross(g: &mut G) {
    let p = g.page(); g.scene(); reticle(g, p, 0.0, 0.0);
    for (i, (x, y)) in [(-24.0, -24.0), (24.0, -24.0), (-24.0, 24.0), (24.0, 24.0)].iter().enumerate() { g.rect(p, cc(*x, *y, 14.0, 5.0), &format!("Hit marker {}", i + 1), g.t.warn, 0.0); }
    let hit = g.panel(p, cc(0.0, 120.0, 270.0, 62.0), "Hit notice panel");
    g.heading(hit, full(), "Hit notice", "TARGET HIT", 28.0);
    let ammo = g.panel(p, br(50.0, 50.0, 280.0, 95.0), "Ammo panel");
    g.heading(ammo, full(), "Ammo count", "24 / 120", 41.0);
}

/// Mira telescópica con ejes y marcas de distancia.
fn reticle_scope(g: &mut G) {
    let p = g.page(); g.scene();
    g.el(p, full(), Kind::Panel, "Scope shade", |e| { e.fill = [0, 0, 0, 105]; e.raycast = false; });
    g.circle(p, cc(0.0, 0.0, 780.0, 780.0), "Scope rim", g.t.line);
    g.circle(p, cc(0.0, 0.0, 750.0, 750.0), "Scope glass", alpha(g.t.panel, 95));
    g.rect(p, cc(0.0, 0.0, 3.0, 720.0), "Vertical crosshair", g.t.accent, 0.0);
    g.rect(p, cc(0.0, 0.0, 720.0, 3.0), "Horizontal crosshair", g.t.accent, 0.0);
    for (i, y) in [-210.0, -105.0, 105.0, 210.0].iter().enumerate() { g.rect(p, cc(0.0, *y, 36.0, 5.0), &format!("Range tick {}", i + 1), g.t.accent, 0.0); }
    let zoom = g.panel(p, br(70.0, 70.0, 260.0, 95.0), "Zoom panel");
    g.heading(zoom, full(), "Zoom value", "ZOOM 4x", 36.0);
}

/// Retícula desplazada con placa de munición contigua.
fn reticle_ammo(g: &mut G) {
    let p = g.page(); g.scene(); reticle(g, p, -120.0, -35.0);
    let ammo = g.panel(p, cc(170.0, -35.0, 230.0, 130.0), "Adjacent ammo panel");
    g.heading(ammo, top(12.0, 60.0, 12.0, 12.0), "Magazine count", "08 / 32", 44.0);
    g.note(ammo, bottom(13.0, 38.0, 12.0, 12.0), "Ammo type", "PRECISION", 24.0);
    let range = g.panel(p, bc(0.0, 50.0, 360.0, 80.0), "Range panel");
    g.heading(range, full(), "Range estimate", "Target 38 m", 32.0);
}

/// Tarjeta flotante sobre un objeto con rareza e interacción.
fn info_float(g: &mut G) {
    let p = g.page(); g.scene(); g.art(p, cc(0.0, 95.0, 340.0, 300.0), "Ancient chest");
    let card = g.panel(p, cc(0.0, -185.0, 560.0, 250.0), "Floating object card");
    g.badge(card, tl(25.0, 22.0, 155.0, 44.0), "Object rarity", "RARE");
    g.heading_l(card, tl(25.0, 75.0, 510.0, 61.0), "Object name", "Sealed chest", 42.0);
    g.note(card, tl(25.0, 140.0, 510.0, 45.0), "Object clue", "An old lock guards its contents.", 26.0);
    g.key(card, bl(25.0, 20.0, 54.0, 54.0), "Interact key", "E");
    g.label(card, bl(95.0, 22.0, 260.0, 48.0), "Interact action", "Inspect", 29.0);
}

/// Panel lateral con arte, descripción y atributos del objeto.
fn info_inspect(g: &mut G) {
    let p = g.page(); g.scene();
    let panel = g.panel(p, right(35.0, 620.0, 65.0, 65.0), "Inspection panel");
    g.heading_l(panel, tl(30.0, 25.0, 560.0, 65.0), "Inspection title", "Field generator", 42.0);
    g.art(panel, top(110.0, 300.0, 30.0, 30.0), "Generator art");
    g.label(panel, tl(30.0, 430.0, 560.0, 85.0), "Generator description", "A portable shield source with a worn power cell.", 28.0);
    for (i, (key, value)) in [("Condition", "72%"), ("Charge", "Low"), ("Weight", "8 kg")].iter().enumerate() {
        let row = g.card(panel, tl(30.0, 540.0 + i as f32 * 80.0, 560.0, 66.0), &format!("{} detail row", key));
        g.label(row, cl(16.0, 0.0, 265.0, 40.0), &format!("{} label", key), key, 26.0);
        g.value(row, cr(16.0, 0.0, 200.0, 40.0), &format!("{} value", key), value, 27.0);
    }
    g.button(panel, bc(0.0, 26.0, 300.0, 66.0), "Take generator button", "Take");
}

/// Placa de NPC con rol, nivel y acción de hablar.
fn info_nameplate(g: &mut G) {
    let p = g.page(); g.scene(); g.art(p, cc(0.0, 120.0, 300.0, 390.0), "NPC silhouette");
    let plate = g.panel(p, cc(0.0, -210.0, 500.0, 165.0), "NPC nameplate");
    g.heading(plate, top(15.0, 55.0, 15.0, 15.0), "NPC name", "Mira, Trader", 41.0);
    g.note(plate, top(75.0, 40.0, 20.0, 20.0), "NPC role", "Market guide - Lv 12", 27.0);
    g.badge(plate, bc(0.0, 10.0, 155.0, 38.0), "NPC relationship", "FRIENDLY");
    let talk = g.panel(p, bc(0.0, 46.0, 350.0, 84.0), "Talk prompt panel");
    g.key(talk, cl(20.0, 0.0, 56.0, 56.0), "Talk key", "E");
    g.label(talk, cl(92.0, 0.0, 230.0, 50.0), "Talk action", "Talk", 31.0);
}

/// Pesca con barra de tensión vertical y botón de recoger.
fn minigame_fishing(g: &mut G) {
    let p = g.page(); g.scene(); g.art(p, cc(-80.0, 10.0, 470.0, 250.0), "Fish silhouette");
    let meter = g.panel(p, cr(110.0, 0.0, 300.0, 670.0), "Fishing tension panel");
    g.heading(meter, top(20.0, 58.0, 15.0, 15.0), "Tension heading", "Tension", 36.0);
    g.rect(meter, tl(112.0, 103.0, 76.0, 430.0), "Tension track", g.t.line, 12.0);
    g.rect(meter, tl(112.0, 277.0, 76.0, 256.0), "Tension fill", g.t.accent, 12.0);
    g.badge(meter, bc(0.0, 78.0, 165.0, 50.0), "Tension level", "60%");
    g.button(meter, bc(0.0, 16.0, 220.0, 62.0), "Reel button", "Reel in");
    let goal = g.panel(p, tc(0.0, 48.0, 530.0, 95.0), "Fishing goal panel");
    g.heading(goal, full(), "Fishing goal", "Keep the line steady", 33.0);
}

/// Tres carriles de ritmo con notas y línea de acierto.
fn minigame_rhythm(g: &mut G) {
    let p = g.page(); g.background();
    let board = g.panel(p, cc(0.0, 0.0, 1060.0, 940.0), "Rhythm board");
    g.heading(board, top(20.0, 55.0, 18.0, 18.0), "Rhythm heading", "Keep the beat", 43.0);
    for (i, key) in ["A", "S", "D"].iter().enumerate() {
        let lane = g.card(board, tl(70.0 + i as f32 * 315.0, 100.0, 290.0, 690.0), &format!("Lane {}", i + 1));
        for (j, y) in [115.0, 310.0, 500.0].iter().enumerate() { g.circle(lane, tc(0.0, *y, 70.0, 70.0), &format!("Lane {} note {}", i + 1, j + 1), g.t.accent); }
        g.rect(lane, bottom(108.0, 8.0, 20.0, 20.0), &format!("Lane {} hit line", i + 1), g.t.warn, 0.0);
        g.key(lane, bc(0.0, 18.0, 72.0, 72.0), &format!("Lane {} key", i + 1), key);
    }
    let combo = g.card(board, bc(0.0, 30.0, 380.0, 75.0), "Combo panel");
    g.heading(combo, full(), "Combo count", "COMBO 18", 37.0);
}

/// Mesa con carta rival, mano de cuatro cartas y dos acciones.
fn minigame_cards(g: &mut G) {
    let p = g.page(); g.background();
    let table = g.panel(p, cc(0.0, 0.0, 1590.0, 940.0), "Card table");
    let rival = g.card(table, tc(0.0, 35.0, 360.0, 130.0), "Rival status card");
    g.heading(rival, top(15.0, 50.0, 15.0, 15.0), "Rival name", "Dealer", 34.0);
    g.note(rival, bottom(16.0, 37.0, 15.0, 15.0), "Rival score", "Score 14", 25.0);
    g.art(table, tc(0.0, 195.0, 155.0, 225.0), "Dealer card art");
    for (i, name) in ["Copper 7", "Silver 3", "Gold 5", "Copper 9"].iter().enumerate() {
        let card = g.card(table, tl(250.0 + i as f32 * 285.0, 500.0, 250.0, 300.0), &format!("{} hand card", name));
        g.art(card, top(20.0, 170.0, 20.0, 20.0), &format!("{} art", name));
        g.heading(card, bottom(35.0, 55.0, 15.0, 15.0), &format!("{} name", name), name, 29.0);
        g.badge(card, bc(0.0, 10.0, 140.0, 37.0), &format!("{} action", name), "PLAY");
    }
    g.button(table, bl(40.0, 30.0, 260.0, 66.0), "Play card button", "Play card");
    g.button2(table, br(40.0, 30.0, 260.0, 66.0), "Pass button", "Pass");
}

/// Circuito de cuatro por cuatro con origen, destino y temporizador.
fn hacking_circuit(g: &mut G) {
    let p = g.page(); g.background();
    let main = g.panel(p, cc(0.0, 0.0, 1300.0, 940.0), "Circuit puzzle panel");
    g.heading(main, top(22.0, 64.0, 25.0, 25.0), "Circuit heading", "Route the signal", 45.0);
    let grid = g.el(main, cc(0.0, 5.0, 700.0, 700.0), Kind::Empty, "Circuit grid", |_| {});
    for i in 0..16 {
        let pos = tl(12.0 + (i % 4) as f32 * 172.0, 12.0 + (i / 4) as f32 * 172.0, 160.0, 160.0);
        let cell = if [0, 1, 5, 9, 10, 14, 15].contains(&i) { g.card_on(grid, pos, &format!("Circuit tile {}", i + 1)) } else { g.card(grid, pos, &format!("Circuit tile {}", i + 1)) };
        g.icon(cell, cc(0.0, 0.0, 68.0, 68.0), &format!("Circuit node {}", i + 1), if i == 0 { "S" } else if i == 15 { "E" } else { "+" });
    }
    g.bar(main, bottom(45.0, 22.0, 90.0, 90.0), "Circuit timer", 0.48, g.t.accent);
    g.note(main, bottom(10.0, 32.0, 90.0, 90.0), "Circuit hint", "Rotate tiles to connect S and E", 25.0);
}

/// Dial circular con aguja y zona válida de ganzúa.
fn hacking_dial(g: &mut G) {
    let p = g.page(); g.background();
    let main = g.panel(p, cc(0.0, 0.0, 1110.0, 900.0), "Lockpick panel");
    g.heading(main, top(30.0, 65.0, 25.0, 25.0), "Lock heading", "Find the angle", 45.0);
    g.circle(main, cc(0.0, 5.0, 540.0, 540.0), "Lock dial rim", g.t.line);
    g.circle(main, cc(0.0, 5.0, 430.0, 430.0), "Lock dial face", g.t.panel2);
    g.circle(main, cc(0.0, 5.0, 105.0, 105.0), "Lock dial center", g.t.accent);
    g.rect(main, cc(75.0, -150.0, 14.0, 245.0), "Lockpick needle", g.t.warn, 5.0);
    g.badge(main, cc(275.0, -155.0, 140.0, 50.0), "Sweet spot", "TARGET");
    key_action(g, main, 70.0, 695.0, "A", "Turn pick");
    key_action(g, main, 700.0, 695.0, "D", "Adjust angle");
    g.bar(main, bottom(20.0, 18.0, 70.0, 70.0), "Pick durability", 0.64, g.t.accent);
}

/// Teclado numérico con código incompleto e intentos restantes.
fn hacking_keypad(g: &mut G) {
    let p = g.page(); g.background();
    let main = g.panel(p, cc(0.0, 0.0, 760.0, 960.0), "Code keypad panel");
    g.heading(main, top(25.0, 62.0, 20.0, 20.0), "Keypad heading", "Enter access code", 43.0);
    let display = g.card(main, top(115.0, 112.0, 65.0, 65.0), "Code display");
    g.heading(display, full(), "Entered code", "4 7 _ _", 53.0);
    let keys = g.grid(main, cc(0.0, 105.0, 540.0, 570.0), "Number keys", 170.0, 170.0, 15.0);
    for i in 1..=9 { g.button2(keys, sz(170.0, 170.0), &format!("Digit {} button", i), &i.to_string()); }
    g.button2(main, bl(80.0, 68.0, 170.0, 70.0), "Clear code button", "Clear");
    g.button2(main, bc(0.0, 68.0, 170.0, 70.0), "Digit zero button", "0");
    g.button(main, br(80.0, 68.0, 170.0, 70.0), "Enter code button", "Enter");
    g.note(main, bottom(18.0, 38.0, 80.0, 80.0), "Attempts remaining", "Attempts left: 3", 26.0);
}
