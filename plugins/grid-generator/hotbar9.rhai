// Creates a 1 x 9 grid of slots at the center of the canvas: an Empty parent with a Grid Layout Group
// (cell 96 x 96, gap 10) and one Panel per slot, ready to receive items.

fn run(design) {
    let rows = 1;
    let cols = 9;
    let cw = 96.0;
    let ch = 96.0;
    let gap = 10.0;
    let total_w = cols.to_float() * cw + (cols - 1).to_float() * gap;
    let total_h = rows.to_float() * ch + (rows - 1).to_float() * gap;
    let x0 = ((design.width - total_w) / 2.0).round();
    let y0 = ((design.height - total_h) / 2.0).round();
    let nid = design.next_id;
    for e in design.elements {
        if e.id >= nid { nid = e.id + 1; }
    }
    let parent_id = nid;
    nid += 1;
    design.elements.push(#{
        id: parent_id, name: "Hotbar 9", kind: "Empty",
        x: x0, y: y0, w: total_w, h: total_h,
        raycast: false,
        layout: #{ mode: "Grid", padding: [0.0, 0.0, 0.0, 0.0], spacing: [gap, gap], align: 0, cell: [cw, ch], expand_w: false, expand_h: false }
    });
    for r in 0..rows {
        for c in 0..cols {
            design.elements.push(#{
                id: nid, name: "Slot " + (r + 1).to_string() + "-" + (c + 1).to_string(), kind: "Panel", parent: parent_id,
                x: x0 + c.to_float() * (cw + gap), y: y0 + r.to_float() * (ch + gap), w: cw, h: ch,
                fill: [255, 255, 255, 40], stroke: [255, 255, 255, 120], stroke_w: 2.0, radius: 10.0
            });
            nid += 1;
        }
    }
    design.next_id = nid;
    log((rows * cols).to_string() + " slots created");
    design
}
