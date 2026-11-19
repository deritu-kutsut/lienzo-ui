// Exporter: Markdown table with every color of the design (hex, RGB, alpha and usage), most used first.

fn hex(c) {
    let d = "0123456789abcdef";
    let s = "#";
    for k in 0..3 {
        s += d.sub_string(c[k] / 16, 1) + d.sub_string(c[k] % 16, 1);
    }
    s
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
// Text that goes inside a Markdown table cell, a heading or inline code: no line breaks, pipes or backticks.
fn md_safe(s) {
    let t = s;
    t.replace("|", "/");
    t.replace("`", "'");
    strip_ctrl(t)
}

fn export_text(design) {
    let counts = #{};
    let colors = #{};
    let uses = #{};
    let add = |c, what| {
        if c[3] == 0 { return; }
        let k = key(c);
        counts[k] = (if counts.contains(k) { counts[k] } else { 0 }) + 1;
        colors[k] = c;
        if !uses.contains(k) { uses[k] = what; } else if !uses[k].contains(what) { uses[k] += ", " + what; }
    };
    add.call(design.background, "background");
    for e in design.elements {
        if e.kind != "Text" && e.kind != "Empty" { add.call(e.fill, "fill"); }
        if e.stroke_w > 0.0 { add.call(e.stroke, "stroke"); }
        if e.text != "" { add.call(e.text_color, "text"); }
    }
    let list = [];
    for k in counts.keys() { list.push([counts[k], k]); }
    list.sort(|a, b| if a[0] > b[0] { -1 } else if a[0] < b[0] { 1 } else { 0 });
    let out = "# Color palette: " + md_safe(design.title) + "\n\n";
    out += "| Hex | RGB | Alpha | Used | Where |\n|---|---|---|---|---|\n";
    let n = 0;
    for item in list {
        n += 1;
        if n > 200 { break; }
        let c = colors[item[1]];
        out += "| `" + hex(c) + "` | " + c[0].to_string() + ", " + c[1].to_string() + ", " + c[2].to_string() + " | " + c[3].to_string() + " | " + item[0].to_string() + " | " + uses[item[1]] + " |\n";
    }
    out
}
