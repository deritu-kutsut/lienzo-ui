// Fills empty texts with lorem ipsum sized to the box, so layouts can be judged with realistic content.
// Text: lorem ipsum sized to the box. Button and Toggle: a short label. InputField: a hint. Dropdown without options: three options.

fn lorem(chars, offset) {
    let words = ["lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit", "sed", "do", "eiusmod", "tempor", "incididunt", "ut", "labore", "et", "dolore", "magna", "aliqua", "ut", "enim", "ad", "minim", "veniam", "quis", "nostrud", "exercitation"];
    let s = "";
    let i = offset % words.len();
    while s.len() < chars {
        if s != "" { s += " "; }
        s += words[i];
        i = (i + 1) % words.len();
    }
    if s.len() > chars && chars >= 12 {
        // cut at the last whole word
        let cut = s.sub_string(0, chars);
        let sp = cut.len() - 1;
        while sp > 0 && cut.sub_string(sp, 1) != " " { sp -= 1; }
        if sp > 0 { s = cut.sub_string(0, sp); } else { s = cut; }
    }
    let first = s.sub_string(0, 1).to_upper();
    first + s.sub_string(1)
}

fn run(design) {
    let filled = 0;
    for i in 0..design.elements.len() {
        let e = design.elements[i];
        if e.text != "" && e.kind != "Dropdown" { continue; }
        if e.kind == "Text" {
            let per_line = (e.w / (e.font_size * 0.55)).floor().to_int();
            let lines = (e.h / (e.font_size * 1.25)).floor().to_int();
            if per_line < 1 { per_line = 1; }
            if lines < 1 { lines = 1; }
            let chars = per_line * lines;
            if chars > 400 { chars = 400; }
            if lines == 1 && chars > 3 { chars = chars - 1; }
            e.text = lorem(chars, i * 3);
            filled += 1;
        } else if e.kind == "Button" {
            e.text = "Button";
            filled += 1;
        } else if e.kind == "Toggle" {
            e.text = "Option";
            filled += 1;
        } else if e.kind == "InputField" {
            e.text = "Enter text...";
            filled += 1;
        } else if e.kind == "Dropdown" && e.options.len() == 0 {
            e.options = ["Option 1", "Option 2", "Option 3"];
            filled += 1;
        }
        design.elements[i] = e;
    }
    log(filled.to_string() + " empty elements filled");
    design
}
