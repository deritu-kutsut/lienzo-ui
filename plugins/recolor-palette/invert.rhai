// Inverts the RGB of every fill, stroke, text color and the background (alpha is kept): quick light/dark variant.

fn inv(c) {
    [255 - c[0], 255 - c[1], 255 - c[2], c[3]]
}

fn run(design) {
    design.background = inv(design.background);
    for i in 0..design.elements.len() {
        let e = design.elements[i];
        e.fill = inv(e.fill);
        e.fill2 = inv(e.fill2);
        e.stroke = inv(e.stroke);
        e.text_color = inv(e.text_color);
        design.elements[i] = e;
    }
    log("colors inverted in " + design.elements.len().to_string() + " elements");
    design
}
