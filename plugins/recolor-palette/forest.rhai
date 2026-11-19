// Recolors the whole design with a palette (dark to light). Every distinct color of the design, background included,
// is ranked by brightness and replaced by the palette color of the same rank, so the look of the layout is kept.
// Text on buttons and other filled elements is switched to white or dark for good contrast.

fn lum(c) {
    0.2126 * c[0].to_float() + 0.7152 * c[1].to_float() + 0.0722 * c[2].to_float()
}

fn key(c) {
    c[0].to_string() + "," + c[1].to_string() + "," + c[2].to_string()
}

// color of the palette at position t (0.0 = darkest, 1.0 = lightest)
fn at(pal, t) {
    let n = pal.len() - 1;
    let f = t * n.to_float();
    let i = f.floor().to_int();
    if i >= n { i = n - 1; }
    if i < 0 { i = 0; }
    let k = f - i.to_float();
    let a = pal[i];
    let b = pal[i + 1];
    [
        (a[0].to_float() + (b[0] - a[0]).to_float() * k).round().to_int(),
        (a[1].to_float() + (b[1] - a[1]).to_float() * k).round().to_int(),
        (a[2].to_float() + (b[2] - a[2]).to_float() * k).round().to_int()
    ]
}

fn run(design) {
    let pal = [[10, 32, 20], [24, 78, 44], [60, 140, 70], [150, 200, 110], [232, 245, 205]];
    let found = #{};
    let list = [];
    let all = [design.background];
    for e in design.elements {
        if e.kind != "Text" && e.fill[3] > 0 { all.push(e.fill); }
        if (e.kind == "Image" || e.kind == "Slider" || e.kind == "ScrollView") && e.fill2[3] > 0 { all.push(e.fill2); }
    }
    for c in all {
        let k = key(c);
        if !found.contains(k) {
            found[k] = 0;
            list.push([lum(c), k, c]);
        }
    }
    list.sort(|a, b| if a[0] < b[0] { -1 } else if a[0] > b[0] { 1 } else { 0 });
    let map = #{};
    let count = list.len();
    for r in 0..count {
        let t = if count > 1 { r.to_float() / (count - 1).to_float() } else { 0.5 };
        map[list[r][1]] = at(pal, t);
    }
    let swap = |c| {
        let m = map[key(c)];
        if m == () { c } else { [m[0], m[1], m[2], c[3]] }
    };
    design.background = swap.call(design.background);
    let changed = 0;
    for i in 0..design.elements.len() {
        let e = design.elements[i];
        if e.kind != "Text" && e.fill[3] > 0 {
            e.fill = swap.call(e.fill);
            if e.text != "" || e.kind == "Button" {
                e.text_color = if lum(e.fill) < 140.0 { [255, 255, 255, e.text_color[3]] } else { [20, 20, 30, e.text_color[3]] };
            }
        }
        if (e.kind == "Image" || e.kind == "Slider" || e.kind == "ScrollView") && e.fill2[3] > 0 { e.fill2 = swap.call(e.fill2); }
        design.elements[i] = e;
        changed += 1;
    }
    log(count.to_string() + " colors remapped in " + changed.to_string() + " elements");
    design
}
