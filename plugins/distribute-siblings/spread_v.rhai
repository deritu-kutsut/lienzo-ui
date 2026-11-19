// Spreads siblings vertically: the topmost and bottommost stay, the gaps between all of them become equal.
// Works on groups of siblings (elements with the same parent). Full-canvas backgrounds, locked and hidden elements are left alone.
// Children of a moved element move with it.

fn total_shift(pm, mv, id) {
    let sum = 0.0;
    let cur = id;
    let guard = 0;
    while cur != () && guard < 60 {
        let k = cur.to_string();
        if mv.contains(k) { sum += mv[k]; }
        cur = pm[k];
        guard += 1;
    }
    sum
}

fn run(design) {
    let horizontal = false;
    let spread = true;
    let gap_fixed = 16.0;
    let n = design.elements.len();
    let pm = #{};
    let groups = #{};
    for i in 0..n {
        let e = design.elements[i];
        pm[e.id.to_string()] = e.parent;
        if e.locked || !e.visible { continue; }
        if e.w >= design.width * 0.9 && e.h >= design.height * 0.9 { continue; }
        let k = if e.parent == () { "root" } else { e.parent.to_string() };
        if !groups.contains(k) { groups[k] = []; }
        groups[k].push(i);
    }
    let mv = #{};
    let touched = 0;
    for k in groups.keys() {
        let idxs = groups[k];
        if idxs.len() < (if spread { 3 } else { 2 }) { continue; }
        let order = [];
        for i in idxs {
            let e = design.elements[i];
            order.push([if horizontal { e.x + e.w / 2.0 } else { e.y + e.h / 2.0 }, i]);
        }
        order.sort(|a, b| if a[0] < b[0] { -1 } else if a[0] > b[0] { 1 } else { 0 });
        let first = design.elements[order[0][1]];
        let last = design.elements[order[order.len() - 1][1]];
        let start = if horizontal { first.x } else { first.y };
        let sizes = 0.0;
        for o in order {
            let e = design.elements[o[1]];
            sizes += if horizontal { e.w } else { e.h };
        }
        let gap = gap_fixed;
        if spread {
            let end = if horizontal { last.x + last.w } else { last.y + last.h };
            gap = (end - start - sizes) / (order.len() - 1).to_float();
        }
        let pos = start;
        for o in order {
            let e = design.elements[o[1]];
            let cur = if horizontal { e.x } else { e.y };
            let d = pos - cur;
            if d != 0.0 {
                mv[e.id.to_string()] = d;
                touched += 1;
            }
            pos += (if horizontal { e.w } else { e.h }) + gap;
        }
    }
    for i in 0..n {
        let d = total_shift(pm, mv, design.elements[i].id);
        if d != 0.0 {
            if horizontal { design.elements[i].x += d; } else { design.elements[i].y += d; }
        }
    }
    log(touched.to_string() + " elements moved");
    design
}
