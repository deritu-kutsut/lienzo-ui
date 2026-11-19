// Exporter: Markdown accessibility report. Checks text contrast (WCAG 2.1), tiny text, small touch targets,
// empty button labels, faint text and texts that probably overflow their box.
// Sizes are in design pixels (reference resolution of the design).

fn channel(v) {
    let c = v.to_float() / 255.0;
    if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055) ** 2.4 }
}

fn luminance(c) {
    0.2126 * channel(c[0]) + 0.7152 * channel(c[1]) + 0.0722 * channel(c[2])
}

fn ratio(a, b) {
    let la = luminance(a);
    let lb = luminance(b);
    let hi = if la > lb { la } else { lb };
    let lo = if la > lb { lb } else { la };
    (hi + 0.05) / (lo + 0.05)
}

fn hex(c) {
    let d = "0123456789abcdef";
    let s = "#";
    for k in 0..3 {
        s += d.sub_string(c[k] / 16, 1) + d.sub_string(c[k] % 16, 1);
    }
    s
}

fn round1(v) {
    (v * 10.0).round() / 10.0
}

// background behind the text of element i: its own fill for buttons and fields, otherwise the top-most
// filled element drawn before it that contains its center, otherwise the design background
fn background_of(design, i) {
    let e = design.elements[i];
    if e.kind != "Text" && e.kind != "Empty" && e.fill[3] >= 128 { return e.fill; }
    let cx = e.x + e.w / 2.0;
    let cy = e.y + e.h / 2.0;
    let j = i - 1;
    while j >= 0 {
        let o = design.elements[j];
        if o.visible && o.kind != "Text" && o.kind != "Empty" && o.fill[3] >= 128 && cx >= o.x && cx <= o.x + o.w && cy >= o.y && cy <= o.y + o.h {
            return o.fill;
        }
        j -= 1;
    }
    design.background
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
    let rows = [];
    let errors = 0;
    let warns = 0;
    let checked = 0;
    let touch = ["Button", "Toggle", "Slider", "InputField", "Dropdown"];
    for i in 0..design.elements.len() {
        let e = design.elements[i];
        if !e.visible { continue; }
        let has_text = e.text != "" && e.kind != "Image" && e.kind != "Panel" && e.kind != "Ellipse" && e.kind != "Empty";
        if has_text {
            checked += 1;
            let bg = background_of(design, i);
            let r = ratio(e.text_color, bg);
            let large = e.font_size >= 24.0 || (e.bold && e.font_size >= 18.66);
            let need = if large { 3.0 } else { 4.5 };
            if r < need {
                errors += 1;
                rows.push("| Error | " + md_safe(e.name) + " | Contrast " + round1(r).to_string() + ":1 is below " + need.to_string() + ":1 (text " + hex(e.text_color) + " on " + hex(bg) + ") |");
            }
            if e.font_size < 12.0 {
                warns += 1;
                rows.push("| Warning | " + md_safe(e.name) + " | Text is only " + e.font_size.to_int().to_string() + " px tall: hard to read on phones |");
            }
            if e.opacity < 0.5 || e.text_color[3] < 128 {
                warns += 1;
                rows.push("| Warning | " + md_safe(e.name) + " | Text is very transparent |");
            }
            let est_w = e.text.len().to_float() * e.font_size * 0.55;
            let lines = (est_w / (if e.w > 1.0 { e.w } else { 1.0 })).ceiling();
            if lines * e.font_size * 1.2 > e.h * 1.15 + 2.0 && e.kind != "ScrollView" {
                warns += 1;
                rows.push("| Warning | " + md_safe(e.name) + " | Text may not fit its box (" + e.w.to_int().to_string() + "x" + e.h.to_int().to_string() + " px) |");
            }
        }
        if touch.contains(e.kind) {
            if e.w < 44.0 || e.h < 44.0 {
                warns += 1;
                rows.push("| Warning | " + md_safe(e.name) + " | Touch target is " + e.w.to_int().to_string() + "x" + e.h.to_int().to_string() + " px: at least 44x44 is recommended |");
            }
        }
        if e.kind == "Button" && e.text == "" && e.image == "" {
            warns += 1;
            rows.push("| Warning | " + md_safe(e.name) + " | Button without a label or an image |");
        }
        if rows.len() > 400 { break; }
    }
    let out = "# Accessibility report: " + md_safe(design.title) + "\n\n";
    out += "- Reference resolution: " + design.width.to_int().to_string() + " x " + design.height.to_int().to_string() + "\n";
    out += "- Texts checked: " + checked.to_string() + "\n";
    out += "- Errors: " + errors.to_string() + "\n";
    out += "- Warnings: " + warns.to_string() + "\n\n";
    if rows.len() == 0 {
        out += "No issues found.\n";
    } else {
        out += "| Level | Element | Issue |\n|---|---|---|\n";
        for r in rows {
            if out.len() > 60000 { out += "| Info | - | Report truncated |\n"; break; }
            out += r + "\n";
        }
    }
    out += "\nRules: text contrast 4.5:1 (3:1 for large text, 24 px or bold 18.7 px and up), touch targets 44 px, text 12 px or more.\n";
    out
}
