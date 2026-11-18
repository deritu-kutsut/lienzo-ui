// Example exporter: receives the design and returns the text of the file that will be saved.

fn cell(s) {
    let x = s;
    x.replace("\"", "\"\"");
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
