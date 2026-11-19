// Rounds the width and height of every element to an even number of pixels, so centered children never land on half pixels.

fn even(v) {
    let r = (v / 2.0).round() * 2.0;
    if r < 2.0 { 2.0 } else { r }
}

fn run(design) {
    let changed = 0;
    for i in 0..design.elements.len() {
        let e = design.elements[i];
        let w = even(e.w);
        let h = even(e.h);
        if w != e.w || h != e.h {
            e.x -= (w - e.w) / 2.0;
            e.y -= (h - e.h) / 2.0;
            e.w = w;
            e.h = h;
            design.elements[i] = e;
            changed += 1;
        }
    }
    log(changed.to_string() + " elements adjusted to even sizes");
    design
}
