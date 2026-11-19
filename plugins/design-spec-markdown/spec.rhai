// Exporter: Markdown design spec for developers. The canvas settings, the element tree with kind, position, size,
// colors and text, and a list of the colors in use.

fn hex(c) {
    let d = "0123456789abcdef";
    let s = "#";
    for k in 0..3 {
        s += d.sub_string(c[k] / 16, 1) + d.sub_string(c[k] % 16, 1);
    }
    if c[3] < 255 { s += d.sub_string(c[3] / 16, 1) + d.sub_string(c[3] % 16, 1); }
    s
}

fn px(v) {
    v.round().to_int().to_string()
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
    let n = design.elements.len();
    let out = "# Design spec: " + md_safe(design.title) + "\n\n";
    out += "| Property | Value |\n|---|---|\n";
    out += "| Reference resolution | " + px(design.width) + " x " + px(design.height) + " |\n";
    out += "| Background | `" + hex(design.background) + "` |\n";
    out += "| Canvas scaler match | " + design.canvas_match.to_string() + " (0 width, 1 height) |\n";
    out += "| Pixel art | " + (if design.pixel_art { "yes" } else { "no" }) + " |\n";
    out += "| Elements | " + n.to_string() + " |\n\n";
    out += "## Element tree\n\n";
    out += "Format: name (Kind) `x,y size WxH` fill, text. Positions are in design pixels from the top-left corner.\n\n";

    let ids = #{};
    for i in 0..n { ids[design.elements[i].id.to_string()] = i; }
    let kids = #{};
    let roots = [];
    for i in 0..n {
        let p = design.elements[i].parent;
        if p != () && ids.contains(p.to_string()) {
            let k = p.to_string();
            if !kids.contains(k) { kids[k] = []; }
            kids[k].push(i);
        } else {
            roots.push(i);
        }
    }
    let colors = #{};
    let stack = [];
    let r = roots.len() - 1;
    while r >= 0 {
        stack.push([roots[r], 0]);
        r -= 1;
    }
    let lines = 0;
    let shown = 0;
    while stack.len() > 0 {
        let item = stack.pop();
        let e = design.elements[item[0]];
        if out.len() > 56000 { break; }
        let line = "";
        for d in 0..item[1] { line += "  "; }
        line += "- **" + md_safe(e.name) + "** (" + e.kind + ") `" + px(e.x) + "," + px(e.y) + " size " + px(e.w) + "x" + px(e.h) + "`";
        if e.kind != "Text" && e.kind != "Empty" && e.fill[3] > 0 {
            line += " fill `" + hex(e.fill) + "`";
            let k = hex(e.fill);
            colors[k] = (if colors.contains(k) { colors[k] } else { 0 }) + 1;
        }
        if e.stroke_w > 0.0 { line += " stroke `" + hex(e.stroke) + "` " + px(e.stroke_w) + " px"; }
        if e.radius > 0.0 { line += " radius " + px(e.radius); }
        if e.text != "" {
            let t = md_safe(e.text);
            if t.len() > 60 { t = t.sub_string(0, 60) + "..."; }
            line += " text \"" + t + "\" (" + px(e.font_size) + " px, `" + hex(e.text_color) + "`)";
            let k = hex(e.text_color);
            colors[k] = (if colors.contains(k) { colors[k] } else { 0 }) + 1;
        }
        if e.layout.mode != "None" { line += " layout " + e.layout.mode; }
        if !e.visible { line += " (hidden)"; }
        out += line + "\n";
        shown += 1;
        let key = e.id.to_string();
        if kids.contains(key) {
            let list = kids[key];
            let j = list.len() - 1;
            while j >= 0 {
                stack.push([list[j], item[1] + 1]);
                j -= 1;
            }
        }
    }
    if shown < n { out += "\n(" + (n - shown).to_string() + " more elements not listed: the file is size limited)\n"; }
    out += "\n## Colors in use\n\n";
    for k in colors.keys() {
        out += "- `" + k + "` x" + colors[k].to_string() + "\n";
    }
    out
}
