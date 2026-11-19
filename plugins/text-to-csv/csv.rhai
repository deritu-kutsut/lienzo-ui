// Example exporter: receives the design and returns the text of the file that will be saved.

fn cell(s) {
    let x = s;
    x.replace("\"", "\"\"");
    // a cell starting with = + - @ would be run as a formula by Excel / Sheets
    if x.len() > 0 && "=+-@".contains(x.sub_string(0, 1)) { x = "'" + x; }
    "\"" + x + "\""
}

fn export_text(design) {
    let output = "element,text\n";
    for e in design.elements {
        if e.text != "" {
            output += cell(e.name) + "," + cell(e.text) + "\n";
        }
    }
    output
}
