// Flips the whole layout top <-> bottom: positions, anchors, pivots, vertical text alignment, rotation, shadows and layout groups.

fn run(design) {
    let h = design.height;
    for i in 0..design.elements.len() {
        let e = design.elements[i];
        e.y = h - e.y - e.h;
        let amin = e.anchor_min;
        let amax = e.anchor_max;
        e.anchor_min = [amin[0], 1.0 - amax[1]];
        e.anchor_max = [amax[0], 1.0 - amin[1]];
        e.pivot = [e.pivot[0], 1.0 - e.pivot[1]];
        if e.valign == 0 { e.valign = 2; } else if e.valign == 2 { e.valign = 0; }
        if e.rot != 0.0 { e.rot = (360.0 - e.rot) % 360.0; }
        e.shadow.offset = [e.shadow.offset[0], -e.shadow.offset[1]];
        let pad = e.layout.padding;
        e.layout.padding = [pad[0], pad[1], pad[3], pad[2]];
        let a = e.layout.align;
        if a >= 0 && a <= 8 { e.layout.align = (2 - a / 3) * 3 + a % 3; }
        design.elements[i] = e;
    }
    let reversed = 0;
    for p in 0..design.elements.len() {
        if design.elements[p].layout.mode != "Vertical" { continue; }
        let pid = design.elements[p].id;
        let slots = [];
        for i in 0..design.elements.len() {
            if design.elements[i].parent == pid { slots.push(i); }
        }
        if slots.len() < 2 { continue; }
        let kids = [];
        for s in slots { kids.push(design.elements[s]); }
        kids.reverse();
        for k in 0..slots.len() { design.elements[slots[k]] = kids[k]; }
        reversed += 1;
    }
    log("layout mirrored vertically (" + reversed.to_string() + " vertical groups reversed)");
    design
}
