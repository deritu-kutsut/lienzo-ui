// Within each group of same-kind siblings, makes all of them as tall as the tallest.
// Resizes around the center of each element. Full-canvas backgrounds, locked and hidden elements are left alone.

fn run(design) {
    let mode = "heights";
    let n = design.elements.len();
    let groups = #{};
    for i in 0..n {
        let e = design.elements[i];
        if e.locked || !e.visible { continue; }
        if e.w >= design.width * 0.9 && e.h >= design.height * 0.9 { continue; }
        let k = "";
        if mode == "buttons" {
            if e.kind != "Button" { continue; }
            k = "all";
        } else {
            if e.kind == "Text" || e.kind == "Empty" { continue; }
            k = (if e.parent == () { "root" } else { e.parent.to_string() }) + "|" + e.kind;
        }
        if !groups.contains(k) { groups[k] = []; }
        groups[k].push(i);
    }
    let changed = 0;
    for k in groups.keys() {
        let idxs = groups[k];
        if idxs.len() < 2 { continue; }
        let mw = 0.0;
        let mh = 0.0;
        for i in idxs {
            if design.elements[i].w > mw { mw = design.elements[i].w; }
            if design.elements[i].h > mh { mh = design.elements[i].h; }
        }
        for i in idxs {
            let e = design.elements[i];
            let nw = if mode == "heights" { e.w } else { mw };
            let nh = if mode == "widths" { e.h } else { mh };
            if nw != e.w || nh != e.h {
                e.x -= (nw - e.w) / 2.0;
                e.y -= (nh - e.h) / 2.0;
                e.w = nw;
                e.h = nh;
                design.elements[i] = e;
                changed += 1;
            }
        }
    }
    log(changed.to_string() + " elements resized");
    design
}
