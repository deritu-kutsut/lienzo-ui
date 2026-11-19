// Creates a row of four tab buttons (Home, Play, Shop, Settings) in the center of the canvas, under an Empty parent with a Horizontal Layout Group.

fn button(id, parent, label, x, y, w, h, fill) {
    #{
        id: id, name: label + " Button", kind: "Button", parent: parent,
        x: x, y: y, w: w, h: h, text: label,
        fill: fill, radius: 18.0, font_size: 30.0, bold: true, text_color: [255, 255, 255, 255]
    }
}

fn run(design) {
    let labels = ["Home", "Play", "Shop", "Settings"];
    let fills = [[124, 92, 255, 255]];
    let vertical = false;
    let bw = 220.0;
    let bh = 72.0;
    let gap = 8.0;
    let n = labels.len();
    let total_w = if vertical { bw } else { n.to_float() * bw + (n - 1).to_float() * gap };
    let total_h = if vertical { n.to_float() * bh + (n - 1).to_float() * gap } else { bh };
    let x0 = ((design.width - total_w) / 2.0).round();
    let y0 = ((design.height - total_h) / 2.0).round();
    let nid = design.next_id;
    for e in design.elements {
        if e.id >= nid { nid = e.id + 1; }
    }
    let parent_id = nid;
    nid += 1;
    design.elements.push(#{
        id: parent_id, name: "Tab Bar Buttons", kind: "Empty",
        x: x0, y: y0, w: total_w, h: total_h, raycast: false,
        layout: #{ mode: if vertical { "Vertical" } else { "Horizontal" }, padding: [0.0, 0.0, 0.0, 0.0], spacing: [gap, gap], align: 4, cell: [bw, bh], expand_w: false, expand_h: false }
    });
    for i in 0..n {
        let x = if vertical { x0 } else { x0 + i.to_float() * (bw + gap) };
        let y = if vertical { y0 + i.to_float() * (bh + gap) } else { y0 };
        design.elements.push(button(nid, parent_id, labels[i], x, y, bw, bh, fills[i % fills.len()]));
        nid += 1;
    }
    design.next_id = nid;
    log(n.to_string() + " buttons created");
    design
}
