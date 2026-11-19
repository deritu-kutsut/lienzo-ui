//! Scratch-only tests for the plugins bundled in plugins/ (run through the real plugin engine).
use crate::community_validate::{self, Package};
use crate::model::{Doc, Element, Kind};
use crate::plugins::{self, Action, Exporter, Plugin};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn all_plugin_dirs() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(root().join("plugins")).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.join("plugin.json").exists()).collect();
    v.sort();
    v
}
fn plugin(name: &str) -> Plugin {
    let p = plugins::load(&root().join("plugins").join(name));
    assert!(p.error.is_none(), "{}: {:?}", name, p.error);
    p
}
fn command<'a>(p: &'a Plugin, name: &str) -> &'a Action {
    p.commands.iter().find(|c| c.name == name).unwrap_or_else(|| panic!("no command {}", name))
}
fn exporter<'a>(p: &'a Plugin, name: &str) -> &'a Exporter {
    p.exporters.iter().find(|c| c.name == name).unwrap_or_else(|| panic!("no exporter {}", name))
}
fn run(p: &Plugin, cmd: &str, doc: &Doc) -> (Doc, Vec<String>) {
    plugins::run_command(&p.folder, command(p, cmd), doc).unwrap_or_else(|e| panic!("{} / {}: {}", p.name, cmd, e))
}
fn export(p: &Plugin, name: &str, doc: &Doc) -> String {
    plugins::run_exporter(&p.folder, exporter(p, name), doc).unwrap_or_else(|e| panic!("{} / {}: {}", p.name, name, e))
}

fn sample_docs() -> Vec<(String, Doc)> {
    let mut v = Vec::new();
    let mut files: Vec<PathBuf> = std::fs::read_dir(root().join("designs")).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.to_string_lossy().ends_with(".editorgui.json")).collect();
    files.sort();
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap();
        v.push((f.file_name().unwrap().to_string_lossy().to_string(), Doc::from_json_str(&text).unwrap()));
    }
    for i in 0..crate::model::TEMPLATES.len() {
        v.push((format!("template {}", i), crate::model::template(i, 1920.0, 1080.0)));
    }
    v
}

/// The design must stay valid: unique ids, existing parents, no cycles, finite numbers, and valid against the JSON schema.
fn assert_valid(label: &str, d: &Doc) {
    let ids: HashSet<u64> = d.elements.iter().map(|e| e.id).collect();
    assert_eq!(ids.len(), d.elements.len(), "{}: duplicated ids", label);
    assert!(d.elements.iter().all(|e| e.id < d.next_id), "{}: next_id too small", label);
    for e in &d.elements {
        if let Some(p) = e.parent {
            assert!(ids.contains(&p), "{}: {} has a missing parent", label, e.name);
        }
        assert!([e.x, e.y, e.w, e.h].iter().all(|v| v.is_finite()), "{}: {} has a non finite number", label, e.name);
        assert!(e.w >= 0.0 && e.h >= 0.0, "{}: {} negative size", label, e.name);
        let mut cur = e.parent;
        let mut n = 0;
        while let Some(p) = cur {
            n += 1;
            assert!(n < 200, "{}: parent cycle", label);
            cur = d.elements.iter().find(|x| x.id == p).and_then(|x| x.parent);
        }
    }
    let json = serde_json::to_value(d).unwrap();
    let errs = crate::schema::validate_value(&json);
    assert!(errs.is_empty(), "{}: schema errors {:?}", label, errs);
}

fn diff(a: &serde_json::Value, b: &serde_json::Value, path: &str, out: &mut Vec<String>) {
    match (a, b) {
        (serde_json::Value::Object(x), serde_json::Value::Object(y)) => { for (k, v) in x { diff(v, y.get(k).unwrap_or(&serde_json::Value::Null), &format!("{}.{}", path, k), out); } }
        (serde_json::Value::Number(x), serde_json::Value::Number(y)) => { if (x.as_f64().unwrap() - y.as_f64().unwrap()).abs() > 1e-3 { out.push(format!("{}: {} != {}", path, a, b)); } }
        (serde_json::Value::Array(x), serde_json::Value::Array(y)) if x.len() == y.len() => { for (i, (p, q)) in x.iter().zip(y).enumerate() { diff(p, q, &format!("{}[{}]", path, i), out); } }
        _ => if a != b { out.push(format!("{}: {} != {}", path, a, b)); }
    }
}

fn el(id: u64, kind: Kind, name: &str, x: f32, y: f32, w: f32, h: f32) -> Element {
    let mut e = Element::new(id, kind);
    e.name = name.into();
    e.x = x;
    e.y = y;
    e.w = w;
    e.h = h;
    e
}
fn small_doc(elements: Vec<Element>) -> Doc {
    let mut d = Doc::default();
    d.next_id = elements.iter().map(|e| e.id).max().unwrap_or(0) + 1;
    d.elements = elements;
    d
}

#[test]
fn every_plugin_loads_and_packs_for_the_community() {
    let dirs = all_plugin_dirs();
    assert!(dirs.len() >= 13, "only {} plugins", dirs.len());
    for dir in dirs {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        let p = plugins::load(&dir);
        assert!(p.error.is_none(), "{}: {:?}", name, p.error);
        assert_eq!(p.author, "Lienzo UI");
        assert!(!p.version.is_empty() && !p.description.is_empty());
        // same packing as `community::pack_plugin`
        let manifest: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(dir.join("plugin.json")).unwrap()).unwrap();
        let mut files = BTreeMap::new();
        for key in ["commands", "exporters"] {
            for x in manifest[key].as_array().into_iter().flatten() {
                let s = x["script"].as_str().unwrap();
                files.insert(s.to_string(), std::fs::read_to_string(dir.join(s)).unwrap());
            }
        }
        for t in manifest["templates"].as_array().into_iter().flatten() {
            let s = t.as_str().unwrap();
            files.insert(s.to_string(), std::fs::read_to_string(dir.join(s)).unwrap());
        }
        let bytes = serde_json::to_vec(&Package { manifest, files }).unwrap();
        community_validate::validate("plugin", "plugin.json", &bytes).unwrap_or_else(|e| panic!("{}: community validation: {}", name, e));
        assert!(bytes.len() < 200 * 1024, "{}: package too big", name);
        // no stray files in the plugin folder
        for f in std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()) {
            let n = f.file_name().to_string_lossy().to_string();
            assert!(n.ends_with(".rhai") || n.ends_with(".json"), "{}: stray file {}", name, n);
        }
    }
}

#[test]
fn every_command_and_exporter_runs_on_every_sample_design() {
    let docs = sample_docs();
    let mut runs = 0;
    for dir in all_plugin_dirs() {
        let p = plugins::load(&dir);
        for c in &p.commands {
            for (label, d) in &docs {
                let t = std::time::Instant::now();
                let (new, _) = plugins::run_command(&p.folder, c, d).unwrap_or_else(|e| panic!("{} / {} on {}: {}", p.name, c.name, label, e));
                assert_valid(&format!("{} / {} on {}", p.name, c.name, label), &new);
                assert!(t.elapsed().as_secs() < 4, "{} / {} on {} too slow", p.name, c.name, label);
                runs += 1;
            }
        }
        for x in &p.exporters {
            for (label, d) in &docs {
                let text = plugins::run_exporter(&p.folder, x, d).unwrap_or_else(|e| panic!("{} / {} on {}: {}", p.name, x.name, label, e));
                assert!(!text.trim().is_empty() && text.len() <= 64 * 1024, "{} / {} on {}: {} bytes", p.name, x.name, label, text.len());
                assert!(!text.contains('\u{0}'));
                runs += 1;
            }
        }
    }
    println!("{} plugin runs over {} designs", runs, docs.len());
}

#[test]
fn rename_by_hierarchy() {
    let p = plugin("rename-by-hierarchy");
    let d = small_doc(vec![el(1, Kind::Panel, "Menu", 0.0, 0.0, 500.0, 500.0), { let mut e = el(2, Kind::Panel, "Box", 10.0, 10.0, 100.0, 100.0); e.parent = Some(1); e }, { let mut e = el(3, Kind::Button, "Play", 20.0, 20.0, 50.0, 50.0); e.parent = Some(2); e }]);
    let (a, _) = run(&p, "Prefix names with parents", &d);
    let names: Vec<_> = a.elements.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["Menu", "Menu_Box", "Menu_Box_Play"]);
    let (b, _) = run(&p, "Prefix names with parents", &a);
    assert_eq!(b, a, "idempotent");
    let dup = small_doc(vec![el(1, Kind::Panel, "Cell", 0.0, 0.0, 5.0, 5.0), el(2, Kind::Panel, "Cell", 0.0, 0.0, 5.0, 5.0), el(3, Kind::Panel, "Cell", 0.0, 0.0, 5.0, 5.0), el(4, Kind::Panel, "Cell 2", 0.0, 0.0, 5.0, 5.0)]);
    let (u, _) = run(&p, "Make names unique", &dup);
    let set: HashSet<_> = u.elements.iter().map(|e| e.name.clone()).collect();
    assert_eq!(set.len(), 4, "{:?}", u.elements.iter().map(|e| &e.name).collect::<Vec<_>>());
    assert_eq!(u.elements[0].name, "Cell");
}

#[test]
fn recolor_palette() {
    let p = plugin("recolor-palette");
    let mut a = el(1, Kind::Panel, "A", 0.0, 0.0, 100.0, 100.0);
    a.fill = [10, 10, 10, 255];
    let mut b = el(2, Kind::Button, "B", 0.0, 0.0, 100.0, 100.0);
    b.fill = [240, 240, 240, 200];
    b.text = "X".into();
    let mut d = small_doc(vec![a, b]);
    d.background = [128, 128, 128, 255];
    for cmd in ["Palette: Ocean", "Palette: Sunset", "Palette: Forest", "Palette: Mono"] {
        let (n, _) = run(&p, cmd, &d);
        assert_ne!(n.elements[0].fill, d.elements[0].fill, "{}", cmd);
        assert_eq!(n.elements[1].fill[3], 200, "alpha kept");
        let lum = |c: [u8; 4]| 0.2126 * c[0] as f32 + 0.7152 * c[1] as f32 + 0.0722 * c[2] as f32;
        assert!(lum(n.elements[0].fill) < lum(n.background) && lum(n.background) < lum(n.elements[1].fill), "{}: order of brightness kept", cmd);
        assert_eq!(n.elements[0].fill, if cmd.ends_with("Mono") { [10, 10, 12, 255] } else { n.elements[0].fill });
        // text on the dark/light fill gets readable
        assert!(lum(n.elements[1].fill) >= 140.0 && n.elements[1].text_color[0] < 100, "{}: dark text on light fill", cmd);
    }
    let (i1, _) = run(&p, "Invert colors", &d);
    assert_eq!(i1.elements[0].fill, [245, 245, 245, 255]);
    let (i2, _) = run(&p, "Invert colors", &i1);
    assert_eq!(i2, d);
}

#[test]
fn placeholder_text() {
    let p = plugin("placeholder-text");
    let mut t = el(1, Kind::Text, "T", 0.0, 0.0, 400.0, 100.0);
    t.text = String::new();
    t.font_size = 32.0;
    let mut real = el(2, Kind::Text, "Real", 0.0, 0.0, 400.0, 100.0);
    real.text = "Keep me".into();
    let mut btn = el(3, Kind::Button, "B", 0.0, 0.0, 100.0, 50.0);
    btn.text = String::new();
    let mut dd = el(4, Kind::Dropdown, "D", 0.0, 0.0, 100.0, 50.0);
    dd.options.clear();
    let d = small_doc(vec![t, real, btn, dd]);
    let (n, _) = run(&p, "Fill empty texts", &d);
    assert!(n.elements[0].text.starts_with("Lorem ipsum"), "{}", n.elements[0].text);
    assert!(n.elements[0].text.len() > 20 && n.elements[0].text.len() <= 400);
    assert_eq!(n.elements[1].text, "Keep me");
    assert_eq!(n.elements[2].text, "Button");
    assert_eq!(n.elements[3].options.len(), 3);
    let (c, _) = run(&p, "Remove lorem ipsum", &n);
    assert_eq!(c.elements[0].text, "");
    assert_eq!(c.elements[1].text, "Keep me");
    // single line box gets a short text
    let mut s = el(1, Kind::Text, "S", 0.0, 0.0, 300.0, 40.0);
    s.text = String::new();
    s.font_size = 30.0;
    let (n2, _) = run(&p, "Fill empty texts", &small_doc(vec![s]));
    assert!(n2.elements[0].text.len() < 25, "{}", n2.elements[0].text);
}

#[test]
fn grid_generator() {
    let p = plugin("grid-generator");
    let base = crate::model::template(0, 1920.0, 1080.0);
    for (cmd, rows, cols) in [("Grid 3 x 3 panels", 3, 3), ("Inventory 6 x 4", 4, 6), ("Hotbar 9 slots", 1, 9), ("Card grid 3 x 2", 2, 3)] {
        let (n, _) = run(&p, cmd, &base);
        assert_eq!(n.elements.len(), base.elements.len() + rows * cols + 1, "{}", cmd);
        let parent = &n.elements[base.elements.len()];
        assert_eq!(parent.kind, Kind::Empty);
        assert_eq!(parent.layout.mode, crate::model::LayoutMode::Grid);
        let slots: Vec<_> = n.elements.iter().filter(|e| e.parent == Some(parent.id)).collect();
        assert_eq!(slots.len(), rows * cols);
        for s in &slots {
            assert!(s.x >= 0.0 && s.y >= 0.0 && s.x + s.w <= n.width && s.y + s.h <= n.height, "{}: slot outside canvas", cmd);
            assert!(s.x >= parent.x - 0.5 && s.x + s.w <= parent.x + parent.w + 0.5);
        }
        assert_valid(cmd, &n);
        // twice: ids still unique
        let (n2, _) = run(&p, cmd, &n);
        assert_valid(cmd, &n2);
    }
}

#[test]
fn mirror_layout() {
    let p = plugin("mirror-layout");
    let docs = sample_docs();
    for (label, d) in &docs {
        let (h1, _) = run(&p, "Mirror horizontally (RTL)", d);
        assert_valid(label, &h1);
        let (h2, _) = run(&p, "Mirror horizontally (RTL)", &h1);
        { let mut o = vec![]; diff(&serde_json::to_value(&h2).unwrap(), &serde_json::to_value(d).unwrap(), "", &mut o); assert!(o.is_empty(), "{}: mirror twice: {:?}", label, o); }
        let (v1, _) = run(&p, "Mirror vertically", d);
        let (v2, _) = run(&p, "Mirror vertically", &v1);
        { let mut o = vec![]; diff(&serde_json::to_value(&v2).unwrap(), &serde_json::to_value(d).unwrap(), "", &mut o); assert!(o.is_empty(), "{}: vertical twice: {:?}", label, o); }
    }
    let mut e = el(1, Kind::Text, "T", 100.0, 50.0, 300.0, 40.0);
    e.align = 0;
    let d = small_doc(vec![e]);
    let (m, _) = run(&p, "Mirror horizontally (RTL)", &d);
    assert_eq!(m.elements[0].x, 1920.0 - 100.0 - 300.0);
    assert_eq!(m.elements[0].align, 2);
    assert_eq!(m.elements[0].y, 50.0);
}

#[test]
fn distribute_siblings() {
    let p = plugin("distribute-siblings");
    let mk = |xs: &[f32]| {
        let mut v = vec![];
        for (i, x) in xs.iter().enumerate() {
            v.push(el(i as u64 + 1, Kind::Button, &format!("B{}", i), *x, 100.0 + i as f32 * 10.0, 100.0, 50.0));
        }
        small_doc(v)
    };
    let d = mk(&[100.0, 150.0, 600.0, 1000.0]);
    let (n, _) = run(&p, "Spread evenly (horizontal)", &d);
    let e = &n.elements;
    assert_eq!(e[0].x, 100.0);
    assert_eq!(e[3].x, 1000.0);
    let g1 = e[1].x - (e[0].x + e[0].w);
    let g2 = e[2].x - (e[1].x + e[1].w);
    let g3 = e[3].x - (e[2].x + e[2].w);
    assert!((g1 - g2).abs() < 0.01 && (g2 - g3).abs() < 0.01, "{} {} {}", g1, g2, g3);
    let (s, _) = run(&p, "Stack in a row (16 px gap)", &d);
    assert_eq!(s.elements[1].x - (s.elements[0].x + s.elements[0].w), 16.0);
    assert_eq!(s.elements[0].x, 100.0);
    let (v, _) = run(&p, "Stack in a column (16 px gap)", &d);
    assert_eq!(v.elements[1].y - (v.elements[0].y + v.elements[0].h), 16.0);
    let (sv, _) = run(&p, "Spread evenly (vertical)", &d);
    assert_eq!(sv.elements[0].y, 100.0);
    // children travel with the parent
    let mut a = el(1, Kind::Panel, "A", 0.0, 100.0, 100.0, 100.0);
    a.parent = None;
    let b = el(2, Kind::Panel, "B", 150.0, 100.0, 100.0, 100.0);
    let c = el(3, Kind::Panel, "C", 900.0, 100.0, 100.0, 100.0);
    let mut kid = el(4, Kind::Button, "Kid", 160.0, 110.0, 20.0, 20.0);
    kid.parent = Some(2);
    let d2 = small_doc(vec![a, b, c, kid]);
    let (n2, _) = run(&p, "Spread evenly (horizontal)", &d2);
    let shift = n2.elements[1].x - 150.0;
    assert!(shift > 100.0);
    let kid_x = n2.elements.iter().find(|e| e.name == "Kid").unwrap().x;
    assert!((kid_x - (160.0 + shift)).abs() < 0.01, "kid moved with parent: shift {} kid x {}", shift, kid_x);
    // a full-canvas background is ignored
    let mut bg = el(5, Kind::Image, "Background", 0.0, 0.0, 1920.0, 1080.0);
    bg.locked = false;
    let mut d3 = d.clone();
    d3.elements.push(bg);
    d3.next_id = 10;
    let (n3, _) = run(&p, "Stack in a row (16 px gap)", &d3);
    assert_eq!(n3.elements[4].x, 0.0);
}

#[test]
fn size_normalizer() {
    let p = plugin("size-normalizer");
    let d = small_doc(vec![el(1, Kind::Button, "A", 100.0, 100.0, 200.0, 60.0), el(2, Kind::Button, "B", 400.0, 100.0, 300.0, 80.0), el(3, Kind::Button, "C", 800.0, 100.0, 250.0, 70.0)]);
    let (n, _) = run(&p, "Equalize all buttons", &d);
    for (i, e) in n.elements.iter().enumerate() {
        assert_eq!((e.w, e.h), (300.0, 80.0));
        let o = &d.elements[i];
        assert!(((e.x + e.w / 2.0) - (o.x + o.w / 2.0)).abs() < 0.01 && ((e.y + e.h / 2.0) - (o.y + o.h / 2.0)).abs() < 0.01, "centered");
    }
    let (w, _) = run(&p, "Equalize sibling widths", &d);
    assert!(w.elements.iter().all(|e| e.w == 300.0));
    assert_eq!(w.elements[0].h, 60.0);
    let (h, _) = run(&p, "Equalize sibling heights", &d);
    assert!(h.elements.iter().all(|e| e.h == 80.0));
    let (s, _) = run(&p, "Equalize sibling sizes", &d);
    assert!(s.elements.iter().all(|e| (e.w, e.h) == (300.0, 80.0)));
    let odd = small_doc(vec![el(1, Kind::Panel, "A", 10.0, 10.0, 101.0, 55.0)]);
    let (ev, _) = run(&p, "Make sizes even", &odd);
    assert!(ev.elements[0].w % 2.0 == 0.0 && ev.elements[0].h % 2.0 == 0.0);
}

#[test]
fn accessibility_report() {
    let p = plugin("accessibility-report");
    let mut bad = el(1, Kind::Text, "Faint", 100.0, 100.0, 400.0, 60.0);
    bad.text = "Hard to read".into();
    bad.text_color = [200, 200, 200, 255];
    bad.font_size = 20.0;
    let mut good = el(2, Kind::Text, "Good", 100.0, 200.0, 400.0, 60.0);
    good.text = "Easy to read".into();
    good.text_color = [0, 0, 0, 255];
    good.font_size = 30.0;
    let mut tiny = el(3, Kind::Button, "Tiny", 100.0, 300.0, 30.0, 30.0);
    tiny.text = "x".into();
    let mut d = small_doc(vec![bad, good, tiny]);
    d.background = [255, 255, 255, 255];
    let r = export(&p, "Accessibility report", &d);
    assert!(r.starts_with("# Accessibility report"));
    assert!(r.contains("| Error | Faint | Contrast"), "{}", r);
    assert!(!r.contains("| Error | Good"), "{}", r);
    assert!(r.contains("Tiny | Touch target is 30x30"), "{}", r);
    assert!(r.contains("Errors: 1"), "{}", r);
    println!("{}", r);
    let hud = export(&p, "Accessibility report", &sample_docs().into_iter().find(|(n, _)| n.starts_with("hud.")).unwrap().1);
    println!("{}", hud.lines().take(12).collect::<Vec<_>>().join("\n"));
}

#[test]
fn design_spec_markdown() {
    let p = plugin("design-spec-markdown");
    for (label, d) in sample_docs() {
        let s = export(&p, "Design spec", &d);
        assert!(s.contains("## Element tree") && s.contains(&d.title), "{}", label);
        for e in d.elements.iter().take(5) {
            assert!(s.contains(&format!("**{}**", e.name)), "{}: {}", label, e.name);
        }
        assert!(s.contains("## Colors in use"));
    }
    let inv = sample_docs().into_iter().find(|(n, _)| n.starts_with("inventory")).unwrap().1;
    let s = export(&p, "Design spec", &inv);
    // children are indented under their parent
    assert!(s.contains("\n  - **Title**"), "{}", s.lines().skip(12).take(30).collect::<Vec<_>>().join("\n"));
}

#[test]
fn localization_sheet() {
    let p = plugin("localization-sheet");
    let mut t = el(1, Kind::Text, "Welcome Title!", 0.0, 0.0, 400.0, 60.0);
    t.text = "Hello\tworld \"quoted\"\nline 2".into();
    let mut t2 = el(2, Kind::Text, "Welcome Title!", 0.0, 0.0, 400.0, 60.0);
    t2.text = "Second".into();
    let mut dd = el(3, Kind::Dropdown, "Language", 0.0, 0.0, 200.0, 50.0);
    dd.options = vec!["English".into(), "Espa\u{f1}ol".into()];
    let d = small_doc(vec![t, t2, dd]);
    let tsv = export(&p, "Translator sheet", &d);
    for line in tsv.lines() {
        assert_eq!(line.split('\t').count(), 7, "{:?}", line);
    }
    assert!(tsv.contains("welcome_title\tWelcome Title!\tText\tHello world \"quoted\"\\nline 2"), "{}", tsv);
    assert!(tsv.contains("welcome_title_2\t"), "{}", tsv);
    assert!(tsv.contains("language.option_2\tLanguage\tDropdown option\tEspa\u{f1}ol"));
    let js = export(&p, "Texts as JSON keys", &d);
    let v: serde_json::Value = serde_json::from_str(&js).unwrap_or_else(|e| panic!("{}: {}", e, js));
    assert_eq!(v["welcome_title"], "Hello\tworld \"quoted\"\nline 2");
    assert_eq!(v["language.option_1"], "English");
    // every sample parses as JSON
    for (label, d) in sample_docs() {
        let js = export(&p, "Texts as JSON keys", &d);
        serde_json::from_str::<serde_json::Value>(&js).unwrap_or_else(|e| panic!("{}: {}", label, e));
    }
}

#[test]
fn safe_area_guides() {
    let p = plugin("safe-area-guides");
    let d = crate::model::template(0, 1920.0, 1080.0);
    let (a, _) = run(&p, "Add phone safe-area guides", &d);
    assert_eq!(a.elements.len(), d.elements.len() + 3);
    let g = a.elements.last().unwrap();
    assert!(g.locked && !g.raycast && g.name.starts_with("Guide Safe"));
    assert_valid("guides", &a);
    let (t, _) = run(&p, "Add title-safe margin guides", &a);
    assert_eq!(t.elements.len(), a.elements.len() + 4);
    let (r, _) = run(&p, "Remove guides", &t);
    assert_eq!(r.elements, d.elements);
    let mut portrait = d.clone();
    portrait.width = 1080.0;
    portrait.height = 1920.0;
    let (pa, _) = run(&p, "Add phone safe-area guides", &portrait);
    assert_eq!(pa.elements.len(), d.elements.len() + 2);
}

#[test]
fn button_set_generator() {
    let p = plugin("button-set-generator");
    let d = crate::model::template(0, 1920.0, 1080.0);
    for (cmd, n) in [("Main menu column", 3), ("OK / Cancel pair", 2), ("Yes / No / Cancel row", 3), ("Tab bar (4 tabs)", 4)] {
        let (r, _) = run(&p, cmd, &d);
        assert_eq!(r.elements.len(), d.elements.len() + n + 1, "{}", cmd);
        let parent = &r.elements[d.elements.len()];
        let kids: Vec<_> = r.elements.iter().filter(|e| e.parent == Some(parent.id)).collect();
        assert_eq!(kids.len(), n);
        assert!(kids.iter().all(|k| k.kind == Kind::Button && !k.text.is_empty()));
        assert!(kids.iter().all(|k| k.x >= 0.0 && k.x + k.w <= r.width && k.y >= 0.0 && k.y + k.h <= r.height));
        assert_valid(cmd, &r);
    }
}

#[test]
fn text_case_tools() {
    let p = plugin("text-case-tools");
    let mut t = el(1, Kind::Text, "T", 0.0, 0.0, 400.0, 60.0);
    t.text = "hello wORLD of-ui".into();
    let mut dd = el(2, Kind::Dropdown, "D", 0.0, 0.0, 200.0, 50.0);
    dd.options = vec!["easy mode".into()];
    let d = small_doc(vec![t, dd]);
    assert_eq!(run(&p, "UPPERCASE all texts", &d).0.elements[0].text, "HELLO WORLD OF-UI");
    assert_eq!(run(&p, "lowercase all texts", &d).0.elements[0].text, "hello world of-ui");
    assert_eq!(run(&p, "Title Case all texts", &d).0.elements[0].text, "Hello World Of-Ui");
    assert_eq!(run(&p, "Sentence case all texts", &d).0.elements[0].text, "Hello world of-ui");
    assert_eq!(run(&p, "Title Case all texts", &d).0.elements[1].options[0], "Easy Mode");
}

#[test]
fn palette_export() {
    let p = plugin("palette-export");
    let mut a = el(1, Kind::Panel, "A", 0.0, 0.0, 10.0, 10.0);
    a.fill = [255, 0, 0, 255];
    let mut b = el(2, Kind::Panel, "B", 0.0, 0.0, 10.0, 10.0);
    b.fill = [255, 0, 0, 255];
    let mut c = el(3, Kind::Text, "C", 0.0, 0.0, 10.0, 10.0);
    c.text = "x".into();
    c.text_color = [0, 0, 255, 128];
    let d = small_doc(vec![a, b, c]);
    let uss = export(&p, "Palette as USS variables", &d);
    assert!(uss.contains(":root {") && uss.contains("--lz-color-1: rgb(255, 0, 0);  /* used 2 times */"), "{}", uss);
    assert!(uss.contains("rgba(0, 0, 255, 0.5)"), "{}", uss);
    let md = export(&p, "Palette table", &d);
    assert!(md.contains("| `#ff0000` | 255, 0, 0 | 255 | 2 | fill |"), "{}", md);
}

#[test]
fn big_designs_stay_within_the_script_limits() {
    // How many elements can a plugin handle before the 10k nested item limit of the script engine stops it?
    let p = plugin("rename-by-hierarchy");
    let mut last_ok = 0;
    for n in [50usize, 100, 150, 200, 250, 300, 400] {
        let els: Vec<Element> = (0..n).map(|i| el(i as u64 + 1, Kind::Panel, &format!("P{}", i), i as f32, 0.0, 10.0, 10.0)).collect();
        let d = small_doc(els);
        match plugins::run_command(&p.folder, command(&p, "Make names unique"), &d) {
            Ok(_) => last_ok = n,
            Err(e) => {
                println!("limit reached at {} elements: {}", n, e);
                break;
            }
        }
    }
    println!("largest design tried OK: {}", last_ok);
}

#[test]
fn baseline_snap_to_grid_limit() {
    let p = plugin("snap-to-grid");
    for n in [200usize, 250, 300] {
        let els: Vec<Element> = (0..n).map(|i| el(i as u64 + 1, Kind::Panel, &format!("P{}", i), i as f32, 0.0, 10.0, 10.0)).collect();
        let r = plugins::run_command(&p.folder, &p.commands[0], &small_doc(els));
        println!("snap-to-grid with {} elements: {}", n, if r.is_ok() { "ok".to_string() } else { r.unwrap_err() });
    }
}

#[test]
fn community_pack_fonts_pass_the_app_validator_and_load_in_egui() {
    use eframe::egui;
    let pack = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("community-pack");
    let j: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(pack.join("fonts.json")).unwrap()).unwrap();
    let mut defs = egui::FontDefinitions::default();
    let mut names = vec![];
    for f in j["fonts"].as_array().unwrap() {
        let file = f["file"].as_str().unwrap();
        let bytes = std::fs::read(pack.join(file)).unwrap();
        let name = std::path::Path::new(file).file_name().unwrap().to_string_lossy().to_string();
        community_validate::validate("font", &name, &bytes).unwrap_or_else(|e| panic!("{}: {}", name, e));
        assert_eq!(community_validate::sha256_hex(&bytes), f["sha256"].as_str().unwrap());
        assert_eq!(bytes.len() as u64, f["size_bytes"].as_u64().unwrap());
        assert!(bytes.len() <= 600 * 1024);
        assert!(pack.join(f["license_file"].as_str().unwrap()).exists());
        defs.font_data.insert(name.clone(), std::sync::Arc::new(egui::FontData::from_owned(bytes)));
        defs.families.insert(egui::FontFamily::Name(name.clone().into()), vec![name.clone()]);
        names.push(name);
    }
    let ctx = egui::Context::default();
    ctx.set_fonts(defs);
    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            for n in &names {
                let galley = ui.painter().layout_no_wrap("Play 123".into(), egui::FontId::new(24.0, egui::FontFamily::Name(n.clone().into())), egui::Color32::WHITE);
                assert!(galley.size().x > 10.0, "{}: text did not lay out", n);
            }
    });
    out.textures_delta.clear();
    println!("{} fonts validated and laid out", names.len());
}
