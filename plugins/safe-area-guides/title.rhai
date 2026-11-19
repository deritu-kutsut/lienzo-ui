// Adds four translucent guide panels that mark a 5% title-safe margin on every side (important text and buttons should stay inside).
// The guides are locked, ignore clicks and are named "Guide Title ...": use "Remove guides" to delete them.

fn guide(id, name, x, y, w, h) {
    #{
        id: id, name: name, kind: "Panel", x: x, y: y, w: w, h: h,
        fill: [255, 190, 40, 70], stroke_w: 0.0, radius: 0.0,
        raycast: false, interactable: false, locked: true
    }
}

fn run(design) {
    let nid = design.next_id;
    for e in design.elements {
        if e.id >= nid { nid = e.id + 1; }
    }
    let w = design.width;
    let h = design.height;
    let mx = (w * 0.05).round();
    let my = (h * 0.05).round();
    let list = [
        ["Guide Title Top", 0.0, 0.0, w, my],
        ["Guide Title Bottom", 0.0, h - my, w, my],
        ["Guide Title Left", 0.0, my, mx, h - 2.0 * my],
        ["Guide Title Right", w - mx, my, mx, h - 2.0 * my]
    ];
    for g in list {
        design.elements.push(guide(nid, g[0], g[1], g[2], g[3], g[4]));
        nid += 1;
    }
    design.next_id = nid;
    log("title-safe guides added (5% margins)");
    design
}
