// Converts every text to Sentence case: first letter in capitals, the rest in lowercase.
// Changes the text of every element and the options of dropdowns.

fn convert(s) {
    if s.len() < 1 { s } else { s.sub_string(0, 1).to_upper() + s.sub_string(1).to_lower() }
}

fn run(design) {
    let changed = 0;
    for i in 0..design.elements.len() {
        let e = design.elements[i];
        let touched = false;
        if e.text != "" {
            let t = convert(e.text);
            if t != e.text { e.text = t; touched = true; }
        }
        for k in 0..e.options.len() {
            let o = convert(e.options[k]);
            if o != e.options[k] { e.options[k] = o; touched = true; }
        }
        if touched {
            design.elements[i] = e;
            changed += 1;
        }
    }
    log(changed.to_string() + " elements changed");
    design
}
