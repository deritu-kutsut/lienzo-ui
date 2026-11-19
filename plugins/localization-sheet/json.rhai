// Exporter: JSON file with one entry per text, { "key": "source text" }, ready for a localization library.
// The keys are the same as in the TSV sheet of this plugin.

fn slug(s) {
    let ok = "abcdefghijklmnopqrstuvwxyz0123456789";
    let out = "";
    let last_us = true;
    for ch in s.to_lower().chars() {
        if ok.contains(ch) {
            out += ch;
            last_us = false;
        } else if !last_us {
            out += "_";
            last_us = true;
        }
    }
    if out.len() > 0 && out.ends_with("_") { out = out.sub_string(0, out.len() - 1); }
    if out == "" { "text" } else { out }
}

fn q(s) {
    let t = s;
    t.replace("\\", "\\\\");
    t.replace("\"", "\\\"");
    t.replace("\n", "\\n");
    t.replace("\r", "");
    t.replace("\t", "\\t");
    // any other control character (below space) must be a \u00XX escape to keep the JSON valid
    let hexd = "0123456789abcdef";
    let out = "";
    for ch in t.chars() {
        let n = ch.to_int();
        if n < 32 { out += "\\u00" + hexd.sub_string(n / 16, 1) + hexd.sub_string(n % 16, 1); } else { out += ch; }
    }
    "\"" + out + "\""
}

fn export_text(design) {
    let lines = [];
    let used = #{};
    for e in design.elements {
        if e.text == "" && e.options.len() == 0 { continue; }
        let base = slug(e.name);
        let key = base;
        if used.contains(key) { key = base + "_" + e.id.to_string(); }
        used[key] = true;
        if e.text != "" && e.kind != "Image" && e.kind != "Panel" && e.kind != "Ellipse" {
            lines.push("  " + q(key) + ": " + q(e.text));
        }
        for k in 0..e.options.len() {
            lines.push("  " + q(key + ".option_" + (k + 1).to_string()) + ": " + q(e.options[k]));
        }
    }
    let out = "{\n";
    for i in 0..lines.len() {
        out += lines[i] + (if i < lines.len() - 1 { "," } else { "" }) + "\n";
        if out.len() > 60000 { break; }
    }
    out + "}\n"
}
