// Flips the whole layout left <-> right (for right-to-left languages such as Arabic or Hebrew):
// positions, anchors, pivots, text alignment, rotation, shadows and layout groups. Horizontal layout groups also reverse their children order.

fn run(design) {
    let w = design.width;
    for i in 0..design.elements.len() {
        let e = design.elements[i];
        e.x = w - e.x - e.w;
        let amin = e.anchor_min;
        let amax = e.anchor_max;
        e.anchor_min = [1.0 - amax[0], amin[1]];
        e.anchor_max = [1.0 - amin[0], amax[1]];
        e.pivot = [1.0 - e.pivot[0], e.pivot[1]];
        if e.align == 0 { e.align = 2; } else if e.align == 2 { e.align = 0; }
        if e.rot != 0.0 { e.rot = (360.0 - e.rot) % 360.0; }
        e.shadow.offset = [-e.shadow.offset[0], e.shadow.offset[1]];
        let pad = e.layout.padding;
        e.layout.padding = [pad[1], pad[0], pad[2], pad[3]];
        let a = e.layout.align;
        if a >= 0 && a <= 8 { e.layout.align = (a / 3) * 3 + (2 - a % 3); }
        design.elements[i] = e;
    }
    // reverse the order of the children of every Horizontal Layout Group, so the visual order is mirrored too
    let reversed = 0;
    for p in 0..design.elements.len() {
        if design.elements[p].layout.mode != "Horizontal" { continue; }
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
    log("layout mirrored horizontally (" + reversed.to_string() + " horizontal groups reversed)");
    design
}
