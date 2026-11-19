// Adds translucent red guide panels over the areas of a phone that the notch, the rounded corners and the home bar can cover.
// Landscape: 5.5% on the left and right and 5.5% at the bottom. Portrait: 5.5% at the top and 4.5% at the bottom.
// The guides are locked, ignore clicks and are named "Guide Safe ...": use "Remove guides" to delete them.

fn guide(id, name, x, y, w, h) {
    #{
        id: id, name: name, kind: "Panel", x: x, y: y, w: w, h: h,
        fill: [255, 60, 60, 80], stroke_w: 0.0, radius: 0.0,
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
    let landscape = w >= h;
    let list = [];
    if landscape {
        let side = (w * 0.055).round();
        let bottom = (h * 0.055).round();
        list.push(["Guide Safe Left", 0.0, 0.0, side, h]);
        list.push(["Guide Safe Right", w - side, 0.0, side, h]);
        list.push(["Guide Safe Bottom", side, h - bottom, w - 2.0 * side, bottom]);
    } else {
        let top = (h * 0.055).round();
        let bottom = (h * 0.045).round();
        list.push(["Guide Safe Top", 0.0, 0.0, w, top]);
        list.push(["Guide Safe Bottom", 0.0, h - bottom, w, bottom]);
    }
    for g in list {
        design.elements.push(guide(nid, g[0], g[1], g[2], g[3], g[4]));
        nid += 1;
    }
    design.next_id = nid;
    log(list.len().to_string() + " safe-area guides added (" + (if landscape { "landscape" } else { "portrait" }) + ")");
    design
}
