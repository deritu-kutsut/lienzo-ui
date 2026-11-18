// Example: receives the design (a map with title, width, height, elements…) and returns it modified.
// Every element has x, y, w, h (canvas pixels), name, kind, text, fill [r,g,b,a]…

fn snap_to(v, step) {
    (v / step).round() * step
}

fn run(design) {
    let step = 8.0;
    let changed = 0;
    for i in 0..design.elements.len() {
        let e = design.elements[i];
        let before = [e.x, e.y, e.w, e.h];
        e.x = snap_to(e.x, step);
        e.y = snap_to(e.y, step);
        e.w = snap_to(e.w, step);
        e.h = snap_to(e.h, step);
        if e.w < step { e.w = step; }
        if e.h < step { e.h = step; }
        if before != [e.x, e.y, e.w, e.h] { changed += 1; }
        design.elements[i] = e;
    }
    log(`${changed} elements snapped to ${step} px`);
    design
}
