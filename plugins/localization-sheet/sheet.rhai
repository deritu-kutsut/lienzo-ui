// Exporter: TSV sheet for translators (opens in Excel or Google Sheets).
// Columns: key, element, kind, source text, max characters (estimate), translation (empty), notes.
// Dropdown options get their own rows. Tabs and line breaks inside texts are escaped.

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

fn clean(s) {
    let t = s;
    t.replace("\t", " ");
    t.replace("\r", "");
    t.replace("\n", "\\n");
    // a cell starting with = + - @ would be run as a formula by Excel / Sheets: prefix it with an apostrophe
    if t.len() > 0 && "=+-@".contains(t.sub_string(0, 1)) { t = "'" + t; }
    t
}

fn max_chars(e) {
    let per_line = (e.w / (e.font_size * 0.55)).floor();
    let lines = (e.h / (e.font_size * 1.25)).floor();
    if per_line < 1.0 { per_line = 1.0; }
    if lines < 1.0 { lines = 1.0; }
    (per_line * lines).to_int()
}

fn export_text(design) {
    let out = "key\telement\tkind\tsource_text\tmax_chars\ttranslation\tnotes\n";
    let used = #{};
    let rows = 0;
    for e in design.elements {
        if e.text == "" && e.options.len() == 0 { continue; }
        let base = slug(e.name);
        let key = base;
        if used.contains(key) { key = base + "_" + e.id.to_string(); }
        used[key] = true;
        if e.text != "" && e.kind != "Image" && e.kind != "Panel" && e.kind != "Ellipse" {
            let note = if e.kind == "Button" { "button label: keep it short" } else { "" };
            out += key + "\t" + clean(e.name) + "\t" + e.kind + "\t" + clean(e.text) + "\t" + max_chars(e).to_string() + "\t\t" + note + "\n";
            rows += 1;
        }
        for k in 0..e.options.len() {
            out += key + ".option_" + (k + 1).to_string() + "\t" + clean(e.name) + "\t" + e.kind + " option\t" + clean(e.options[k]) + "\t" + max_chars(e).to_string() + "\t\t\n";
            rows += 1;
        }
        if out.len() > 60000 { out += "TRUNCATED\t\t\t\t\t\tThe sheet is size limited\n"; break; }
    }
    log(rows.to_string() + " texts exported");
    out
}
