// Exporter: USS (Unity UI Toolkit) file with the colors of the design as variables,
// most used first: --lz-color-1, --lz-color-2... plus the background.

fn css(c) {
    if c[3] == 255 {
        "rgb(" + c[0].to_string() + ", " + c[1].to_string() + ", " + c[2].to_string() + ")"
    } else {
        "rgba(" + c[0].to_string() + ", " + c[1].to_string() + ", " + c[2].to_string() + ", " + ((c[3].to_float() / 255.0 * 100.0).round() / 100.0).to_string() + ")"
    }
}

fn key(c) {
    c[0].to_string() + "," + c[1].to_string() + "," + c[2].to_string() + "," + c[3].to_string()
}

// Replaces control characters (below space) with a space.
fn strip_ctrl(s) {
    let out = "";
    for ch in s.chars() {
        if ch.to_int() >= 32 { out += ch; } else { out += " "; }
    }
    out
}

// The title goes inside a CSS comment: it must not be able to close it.
fn safe_title(s) {
    let t = strip_ctrl(s);
    while t.contains("*/") { t.replace("*/", "* /"); }
    t
}

fn export_text(design) {
    let counts = #{};
    let colors = #{};
    let add = |c| {
        if c[3] == 0 { return; }
        let k = key(c);
        counts[k] = (if counts.contains(k) { counts[k] } else { 0 }) + 1;
        colors[k] = c;
    };
    for e in design.elements {
        if e.kind != "Text" && e.kind != "Empty" { add.call(e.fill); }
        if e.stroke_w > 0.0 { add.call(e.stroke); }
        if e.text != "" { add.call(e.text_color); }
    }
    let list = [];
    for k in counts.keys() { list.push([counts[k], k]); }
    list.sort(|a, b| if a[0] > b[0] { -1 } else if a[0] < b[0] { 1 } else { 0 });
    let out = "/* Colors of " + safe_title(design.title) + " */\n:root {\n";
    out += "    --lz-background: " + css(design.background) + ";\n";
    let n = 0;
    for item in list {
        n += 1;
        if n > 64 { break; }
        out += "    --lz-color-" + n.to_string() + ": " + css(colors[item[1]]) + ";  /* used " + item[0].to_string() + " times */\n";
    }
    out += "}\n";
    out
}
