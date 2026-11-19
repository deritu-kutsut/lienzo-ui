//! Tests of the hardening pass: deep / huge hierarchies, id and value bounds, validators, exporter file safety,
//! generated names, the IMGUI round trip and the portrait fold / copy rules.

use crate::docext::*;
use crate::export;
use crate::model::{self, Doc, Element, Kind, MAX_DEPTH};
use std::path::PathBuf;

fn chain(n: usize) -> Doc {
    let mut d = Doc::empty(1920.0, 1080.0);
    d.title = "Chain".into();
    for i in 0..n {
        let mut e = Element::new(0, if i % 7 == 6 { Kind::ScrollView } else { Kind::Panel });
        e.name = format!("N{}", i);
        e.x = (i % 100) as f32;
        e.y = (i % 50) as f32;
        e.w = 200.0;
        e.h = 100.0;
        if i > 0 {
            e.parent = Some(i as u64);
        }
        d.add(e);
    }
    d
}

fn depth_of(d: &Doc) -> usize {
    let idx = d.index_map();
    let mut best = 0;
    for e in &d.elements {
        let (mut k, mut cur) = (0, e.parent);
        while let Some(p) = cur {
            k += 1;
            assert!(k < 100_000, "cycle");
            cur = idx.get(&p).and_then(|i| d.elements[*i].parent);
        }
        best = best.max(k);
    }
    best
}

fn temp_project(tag: &str) -> PathBuf {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!("lienzo-hard-{}-{}-{}", tag, std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("Assets")).unwrap();
    std::fs::create_dir_all(dir.join("ProjectSettings")).unwrap();
    dir
}

#[test]
fn deep_chain_is_flattened_on_load_and_every_walk_survives() {
    let json = serde_json::to_string(&chain(3000)).unwrap();
    let d = Doc::from_json_str(&json).unwrap();
    assert_eq!(d.elements.len(), 3000);
    assert!(depth_of(&d) <= MAX_DEPTH, "depth {}", depth_of(&d));
    // deterministic: loading twice gives the same hierarchy
    let again = Doc::from_json_str(&json).unwrap();
    assert_eq!(d.elements.iter().map(|e| e.parent).collect::<Vec<_>>(), again.elements.iter().map(|e| e.parent).collect::<Vec<_>>());
    assert!(!export::ugui::json(&d, true).is_empty());
    assert!(!export::uitk::uxml(&d).is_empty() && !export::uitk::uss(&d).is_empty());
    assert!(!export::imgui::script(&d).is_empty());
    assert!(!export::view::base_script(&d).is_empty());
    let _ = crate::review::review(&d);
    let mut m = d.clone();
    m.normalize();
    m.reapply_children(None, model::Frame::page(100.0, 100.0));
    assert!(m.shown_all().len() <= 3000);
    let top = m.elements[0].id;
    assert!(m.descendants(top).len() < 3000);
}

#[test]
fn walks_that_run_without_sanitize_do_not_overflow() {
    // a hand-built (never sanitised) 5000-deep chain: only the iterative helpers are used
    let d = chain(5000);
    let last = d.elements.last().unwrap().id;
    assert_eq!(d.ancestors(last).len(), 4999);
    let mut m = d.clone();
    let root = m.elements[0].id;
    assert_eq!(m.descendants(root).len(), 4999);
    m.reapply_children(Some(root), model::Frame { cx: 0.0, cy: 0.0, w: 10.0, h: 10.0, rot: 0.0 });
    m.normalize();
    assert!(depth_of(&m) <= MAX_DEPTH);
    assert!(m.shown_all().len() <= 5000);
    let names = export::unity_names(&m);
    assert_eq!(names.len(), 5000);
    assert_eq!(export::unity_paths(&m, &names).len(), 5000);
}

#[test]
fn too_many_elements_are_rejected_on_load() {
    let mut d = Doc::empty(100.0, 100.0);
    d.elements = (0..20_001).map(|i| Element::new(i + 1, Kind::Panel)).collect();
    d.next_id = 30_000;
    assert!(Doc::from_json_str(&serde_json::to_string(&d).unwrap()).is_err());
    d.sanitize();
    assert_eq!(d.elements.len(), 20_000, "sanitize caps the list too");
    d.elements.truncate(20_000);
    assert!(Doc::from_json_str(&serde_json::to_string(&d).unwrap()).is_ok());
}

#[test]
fn ids_are_bounded_and_next_id_saturates() {
    let text = format!(
        r#"{{"next_id":{},"elements":[{{"id":{},"kind":"Panel","name":"big"}},{{"id":3,"parent":{},"kind":"Panel","name":"child"}}],"portrait":{{"overrides":[{{"id":{},"x":5}}]}}}}"#,
        u64::MAX,
        u64::MAX,
        u64::MAX,
        u64::MAX
    );
    let d = Doc::from_json_str(&text).unwrap();
    assert!(d.elements.iter().all(|e| e.id >= 1 && e.id <= model::MAX_ID));
    assert!(d.next_id <= model::MAX_ID * 2 && d.next_id > model::MAX_ID.min(3));
    let big = d.elements.iter().find(|e| e.name == "big").unwrap().id;
    assert_eq!(d.elements.iter().find(|e| e.name == "child").unwrap().parent, Some(big), "parent links follow the renumbering");
    assert_eq!(d.portrait.as_ref().unwrap().overrides[0].id, big);
    let mut e = d.clone();
    e.next_id = u64::MAX;
    let id = e.add(Element::new(0, Kind::Panel));
    assert!(id <= model::MAX_ID * 2);
    let id2 = e.add(Element::new(0, Kind::Panel));
    assert!(id2 >= id);
}

#[test]
fn geometry_and_anchors_are_clamped() {
    let text = r#"{"canvas_match":7,"elements":[{"id":1,"kind":"Panel","x":1e30,"y":-1e30,"w":1e30,"h":3e38,"value":1e30,
        "anchor_min":[1e30,-1e30],"anchor_max":[5e4,0],"pivot":[1e9,2],"shadow":{"on":true,"offset":[1e30,0]}}],
        "portrait":{"overrides":[{"id":1,"anchor_min":[9e9,0],"pivot":[-9e9,0],"w":1e30,"x":1e30}]}}"#;
    let d = Doc::from_json_str(text).unwrap();
    assert_eq!(d.canvas_match, 1.0);
    let e = &d.elements[0];
    for v in [e.x, e.y, e.w, e.h, e.value] {
        assert!(v.is_finite() && v.abs() <= model::COORD_LIMIT);
    }
    for v in e.anchor_min.iter().chain(&e.anchor_max).chain(&e.pivot).chain(&e.shadow.offset) {
        assert!(v.abs() <= model::ANCHOR_LIMIT);
    }
    let o = &d.portrait.as_ref().unwrap().overrides[0];
    assert!(o.anchor_min.unwrap()[0] == model::ANCHOR_LIMIT && o.pivot.unwrap()[0] == -model::ANCHOR_LIMIT);
    assert!(o.w.unwrap() <= model::COORD_LIMIT && o.x.unwrap() <= model::COORD_LIMIT);
    for js in [export::ugui::json(&d, true), export::uitk::uss(&d), export::imgui::script(&d)] {
        for bad in ["NaN", "Infinity", "-inf", " inf"].into_iter().chain(if js.trim_start().starts_with('{') { Some("null") } else { None }) {
            assert!(!js.contains(bad), "non finite number exported ({bad}): {}", js.lines().find(|l| l.contains(bad)).unwrap_or(""));
        }
    }
    let nan = Doc::from_json_str(r#"{"canvas_match":null}"#);
    assert!(nan.is_ok() || nan.is_err());
}

#[test]
fn scripts_dir_rules() {
    for ok in ["Assets/Game/UI", "Assets\\Game\\UI\\", "Assets/My Game/Scripts", "Assets/Com1x"] {
        assert!(normalize_scripts_dir(ok).is_some(), "{}", ok);
    }
    for bad in [
        "Assets/a<b", "Assets/a>b", "Assets/a\"b", "Assets/a|b", "Assets/a?b", "Assets/a*b", "Assets/a./b", "Assets/a /b", "Assets/CON", "Assets/nul/x", "Assets/com1", "Assets/LPT9", "Assets/aux.txt/x", "Assets/x/Prn",
    ] {
        assert!(normalize_scripts_dir(bad).is_none(), "{}", bad);
        assert!(!valid_scripts_dir(bad), "{}", bad);
    }
    assert!(crate::schema::validate_value(&serde_json::json!({"elements":[],"unity":{"scripts_dir":"Assets/CON"}})).iter().any(|e| e.contains("scripts_dir")));
}

#[test]
fn generated_names_and_font_names_are_rejected() {
    for n in ["__border", "__text", "__tick", "__safe_area", "__canvas_background"] {
        assert!(!valid_unity_name(n), "{}", n);
        let mut e = Element::new(1, Kind::Panel);
        e.name = n.into();
        assert_ne!(export::base_unity_name(&e), n, "a display name equal to a generated name is renamed");
    }
    assert!(valid_unity_name("__other"));
    for bad in ["a/b", "a\\b", "C:x", "..", "a..b"] {
        assert!(!valid_font_name(bad), "{}", bad);
    }
    assert!(valid_font_name("Open Sans") && valid_font_name("Inter-Bold.v2"));
    let text = r#"{"elements":[{"id":1,"kind":"Text","font":"../../evil"}],"unity":{"default_font":"a/b","fallback_fonts":["ok","c:d","e\\f"]}}"#;
    let d = Doc::from_json_str(text).unwrap();
    assert_eq!(d.elements[0].font, "");
    assert_eq!(d.unity.default_font, None);
    assert_eq!(d.unity.fallback_fonts, vec!["ok"]);
    let v: serde_json::Value = serde_json::from_str(text).unwrap();
    assert!(crate::schema::validate_value(&v).iter().any(|e| e.contains("font")));
}

#[test]
fn imgui_survives_touching_an_old_design() {
    let old = r#"{"title":"Old","elements":[{"id":1,"kind":"Panel","name":"a"}]}"#;
    let mut d = Doc::from_json_str(old).unwrap();
    assert!(d.unity.generate.imgui_enabled());
    d.unity.assembly = Some("Game".into()); // any Unity setting
    let saved = serde_json::to_string_pretty(&d).unwrap();
    assert!(saved.contains("\"imgui\": true"), "{}", saved);
    let back = Doc::from_json_str(&saved).unwrap();
    assert!(back.unity.generate.imgui_enabled());
    // explicit false stays false, a new design stays off
    let mut n = Doc::empty(10.0, 10.0);
    n.unity.scripts_dir = Some("Assets/X".into());
    let back = Doc::from_json_str(&serde_json::to_string(&n).unwrap()).unwrap();
    assert!(!back.unity.generate.imgui_enabled());
    // an untouched old design does not gain a unity block
    assert!(!serde_json::to_string(&Doc::from_json_str(old).unwrap()).unwrap().contains("unity"));
}

fn menu_doc(elements: &[(&str, Kind, Option<&str>)]) -> Doc {
    let mut d = Doc::empty(1920.0, 1080.0);
    d.title = "Menu".into();
    for (name, kind, un) in elements {
        let mut e = Element::new(0, *kind);
        e.name = (*name).into();
        e.unity_name = un.map(|s| s.to_string());
        e.w = 200.0;
        e.h = 80.0;
        d.add(e);
    }
    d
}

#[test]
fn view_names_explicit_first_and_reserved_identifiers() {
    // the automatic element comes first in the list but must not take the explicit name
    let mut d = menu_doc(&[("Play", Kind::Button, None), ("Other", Kind::Button, Some("play"))]);
    d.elements[0].name = "play".into();
    let f = export::view::field_names(&d);
    let auto = d.elements[0].id;
    let explicit = d.elements[1].id;
    assert_eq!(f[&explicit], "play");
    assert_ne!(f[&auto], "play");

    // reserved: the class name of the ViewBase, a handler name, Debug
    let d = menu_doc(&[("A", Kind::Button, Some("MenuViewBase")), ("B", Kind::Button, Some("OnPlayClick")), ("Play", Kind::Button, None), ("D", Kind::Panel, Some("Debug"))]);
    let f = export::view::field_names(&d);
    let names: Vec<&String> = d.elements.iter().map(|e| &f[&e.id]).collect();
    assert_ne!(names[0], "MenuViewBase");
    assert_ne!(names[1], "OnPlayClick");
    let code = export::view::base_script(&d);
    assert!(code.contains("global::UnityEngine.Debug") || !code.contains("Debug."), "Debug must be fully qualified");
    assert!(!code.contains(" Debug.Log"));
    let mut set = std::collections::HashSet::new();
    assert!(names.iter().all(|n| set.insert(n.as_str())), "fields are unique: {:?}", names);
}

#[test]
fn many_equal_names_get_unique_suffixes_quickly() {
    let mut d = Doc::empty(100.0, 100.0);
    for _ in 0..4000 {
        let mut e = Element::new(0, Kind::Button);
        e.name = "Btn".into();
        d.add(e);
    }
    let names = export::unity_names(&d);
    let set: std::collections::HashSet<&String> = names.values().collect();
    assert_eq!(set.len(), 4000);
    assert_eq!(names[&d.elements[0].id], "Btn");
    assert_eq!(names[&d.elements[1].id], "Btn (2)");
    assert_eq!(names[&d.elements[2].id], "Btn (3)");
    let f = export::view::field_names(&d);
    assert_eq!(f.values().collect::<std::collections::HashSet<_>>().len(), 4000);
}

#[test]
fn exporter_never_overwrites_a_user_file_without_the_marker() {
    let dir = temp_project("user");
    let mut d = menu_doc(&[("Play", Kind::Button, None)]);
    d.unity.generate.imgui = Some(true);
    d.canvas.safe_area = true;
    let scripts = dir.join("Assets/EditorGUI/Scripts");
    std::fs::create_dir_all(scripts.join("Runtime")).unwrap();
    std::fs::write(scripts.join("MenuOnGUI.cs"), "// my own file\nclass Mine {}\n").unwrap();
    std::fs::write(scripts.join("Runtime/LienzoSafeArea.cs"), "// not generated\npublic class LienzoSafeArea {}\n").unwrap();
    std::fs::write(scripts.join("MenuViewBase.cs"), "// mine\npublic abstract class MenuViewBase : UnityEngine.MonoBehaviour {}\n").unwrap();
    let written = export::write_unity_package(&d, &dir).unwrap();
    assert!(!written.iter().any(|w| w.ends_with("MenuOnGUI.cs") || w.ends_with("LienzoSafeArea.cs") || w.ends_with("MenuViewBase.cs")), "{:?}", written);
    assert_eq!(std::fs::read_to_string(scripts.join("MenuOnGUI.cs")).unwrap(), "// my own file\nclass Mine {}\n");
    assert!(std::fs::read_to_string(scripts.join("Runtime/LienzoSafeArea.cs")).unwrap().starts_with("// not generated"));
    assert!(std::fs::read_to_string(scripts.join("MenuViewBase.cs")).unwrap().starts_with("// mine"));
    // the files that carry the marker are updated
    let dir2 = temp_project("gen");
    export::write_unity_package(&d, &dir2).unwrap();
    let again = export::write_unity_package(&d, &dir2).unwrap();
    assert!(again.iter().any(|w| w.ends_with("MenuOnGUI.cs")) && again.iter().any(|w| w.ends_with("MenuViewBase.cs")));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir2);
}

#[test]
fn moved_viewbase_is_updated_in_place_and_never_deleted() {
    let dir = temp_project("moved");
    let mut d = menu_doc(&[("Play", Kind::Button, None)]);
    export::write_unity_package(&d, &dir).unwrap();
    let old = dir.join("Assets/EditorGUI/Scripts/MenuViewBase.cs");
    let moved_dir = dir.join("Assets/Game/UI");
    std::fs::create_dir_all(&moved_dir).unwrap();
    let moved = moved_dir.join("MenuViewBase.cs");
    std::fs::rename(&old, &moved).unwrap();
    d.elements[0].name = "Start".into();
    let written = export::write_unity_package(&d, &dir).unwrap();
    assert!(written.iter().any(|w| w == "Assets/Game/UI/MenuViewBase.cs"), "{:?}", written);
    assert!(!old.exists(), "no second copy is created");
    assert!(std::fs::read_to_string(&moved).unwrap().contains("start"));
    // a scripts_dir change updates the copy where it is as well
    d.unity.scripts_dir = Some("Assets/Other".into());
    export::write_unity_package(&d, &dir).unwrap();
    assert!(moved.exists() && !dir.join("Assets/Other/MenuViewBase.cs").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn stale_ongui_copy_is_removed_only_when_generated_and_replaced() {
    let dir = temp_project("ongui");
    let mut d = menu_doc(&[("Play", Kind::Button, None)]);
    d.unity.generate.imgui = Some(true);
    export::write_unity_package(&d, &dir).unwrap();
    let first = dir.join("Assets/EditorGUI/Scripts/MenuOnGUI.cs");
    assert!(first.exists());
    d.unity.scripts_dir = Some("Assets/Moved".into());
    export::write_unity_package(&d, &dir).unwrap();
    assert!(dir.join("Assets/Moved/MenuOnGUI.cs").exists());
    assert!(!first.exists(), "the stale generated copy is gone");
    // a user's file that merely mentions the class is not removed
    let user = dir.join("Assets/Moved/Sub");
    std::fs::create_dir_all(&user).unwrap();
    std::fs::write(user.join("Mine.cs"), "public class MyGui : MenuOnGUI {}\n").unwrap();
    export::write_unity_package(&d, &dir).unwrap();
    assert!(user.join("Mine.cs").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn class_matcher_and_scanner_read_odd_files() {
    let src = "public\tabstract\nclass\n\tMenuViewBase : X {}";
    assert!(export::declares_abstract_class_for_tests(src, "MenuViewBase"));
    assert!(!export::declares_abstract_class_for_tests("public abstract class MenuViewBaseX {}", "MenuViewBase"));
    assert!(!export::declares_abstract_class_for_tests("public class MenuViewBase {}", "MenuViewBase"));
    assert!(!export::declares_abstract_class_for_tests("subclass MenuViewBase", "MenuViewBase"));
    // UTF-16 with BOM and Latin-1 are decoded
    let mut u16le = vec![0xFF, 0xFE];
    for c in "// Generated by Lienzo UI\nabstract class MenuViewBase {}".encode_utf16() {
        u16le.extend_from_slice(&c.to_le_bytes());
    }
    assert!(export::decode_cs(&u16le).contains("abstract class MenuViewBase"));
    assert!(export::decode_cs(b"// caf\xe9\nclass A {}").contains("class A"));
    // a UTF-16 generated ViewBase is found and updated in place
    let dir = temp_project("utf16");
    let d = menu_doc(&[("Play", Kind::Button, None)]);
    std::fs::create_dir_all(dir.join("Assets/Elsewhere")).unwrap();
    std::fs::write(dir.join("Assets/Elsewhere/MenuViewBase.cs"), &u16le).unwrap();
    let written = export::write_unity_package(&d, &dir).unwrap();
    assert!(written.iter().any(|w| w == "Assets/Elsewhere/MenuViewBase.cs"), "{:?}", written);
    assert!(!dir.join("Assets/EditorGUI/Scripts/MenuViewBase.cs").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn only_real_pictures_are_copied() {
    let dir = temp_project("img");
    let src = temp_project("imgsrc");
    let png: Vec<u8> = vec![0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0];
    std::fs::write(src.join("good.png"), &png).unwrap();
    std::fs::write(src.join("fake.png"), b"MZ this is an executable").unwrap();
    std::fs::write(src.join("win.ini"), b"[fonts]").unwrap();
    let mut d = menu_doc(&[("A", Kind::Image, None), ("B", Kind::Image, None), ("C", Kind::Image, None)]);
    d.elements[0].image = src.join("good.png").to_string_lossy().to_string();
    d.elements[1].image = src.join("fake.png").to_string_lossy().to_string();
    d.elements[2].image = src.join("win.ini").to_string_lossy().to_string();
    let prev = crate::library::DESIGN_FOLDER.lock().unwrap().clone();
    crate::library::set_design_folder(&src.join("x.editorgui.json"));
    let written = export::write_unity_package(&d, &dir).unwrap();
    *crate::library::DESIGN_FOLDER.lock().unwrap() = prev;
    assert!(dir.join("Assets/EditorGUI/Sprites/good.png").exists(), "{:?}", written);
    assert!(!dir.join("Assets/EditorGUI/Sprites/fake.png").exists());
    assert!(!dir.join("Assets/EditorGUI/Sprites/win.ini").exists());
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&src);
}

#[test]
fn portrait_review_does_not_repeat_layout_independent_checks() {
    let mut d = Doc::empty(1920.0, 1080.0);
    for _ in 0..2 {
        let mut b = Element::new(0, Kind::Button);
        b.name = "Play".into();
        b.x = 100.0;
        b.y = 100.0;
        b.w = 300.0;
        b.h = 100.0;
        d.add(b);
    }
    let mut img = Element::new(0, Kind::Image);
    img.image = "definitely-missing-image.png".into();
    img.w = 100.0;
    img.h = 100.0;
    d.add(img);
    let n = |ps: &[crate::review::Problem], t: &str| ps.iter().filter(|p| p.template.contains(t)).count();
    let landscape = crate::review::review(&d);
    let (dup, miss) = (n(&landscape, "GameObject.Find"), n(&landscape, "Missing image"));
    assert!(dup >= 1 && miss == 1);
    let id = d.elements[0].id;
    d.set_override(ElementOverride { x: Some(5.0), ..ElementOverride::new(id) });
    let both = crate::review::review(&d);
    assert_eq!(n(&both, "GameObject.Find"), dup);
    assert_eq!(n(&both, "Missing image"), miss);
}

#[test]
fn portrait_resolved_design_applies_layout_groups() {
    use crate::model::LayoutMode;
    let mut d = Doc::empty(1920.0, 1080.0);
    let mut row = Element::new(0, Kind::Panel);
    row.x = 0.0;
    row.y = 0.0;
    row.w = 800.0;
    row.h = 100.0;
    row.layout.mode = LayoutMode::Horizontal;
    row.layout.padding = [0.0; 4];
    row.layout.spacing = [10.0, 0.0];
    let rid = d.add(row);
    for _ in 0..2 {
        let mut c = Element::new(0, Kind::Panel);
        c.w = 100.0;
        c.h = 50.0;
        c.parent = Some(rid);
        d.add(c);
    }
    d.normalize();
    d.apply_layouts();
    d.set_override(ElementOverride { w: Some(300.0), ..ElementOverride::new(rid) });
    let p = d.resolved_for(Orientation::Portrait);
    let kids = p.children(Some(rid));
    let (a, b) = (p.get(kids[0]).unwrap(), p.get(kids[1]).unwrap());
    // the children follow the narrower portrait row (200 px of content centred in 300)
    assert!((b.x - a.x - 110.0).abs() < 0.5, "{} {}", a.x, b.x);
    assert!(a.x >= 0.0 && b.x + b.w <= 300.0 + 0.5, "{} {}", a.x, b.x + b.w);
}

#[test]
fn portrait_content_height_is_exported_per_scrollview() {
    let mut d = Doc::empty(1920.0, 1080.0);
    let mut sv = Element::new(0, Kind::ScrollView);
    sv.x = 0.0;
    sv.y = 0.0;
    sv.w = 400.0;
    sv.h = 300.0;
    let sid = d.add(sv);
    let mut c = Element::new(0, Kind::Panel);
    c.x = 10.0;
    c.y = 10.0;
    c.w = 100.0;
    c.h = 100.0;
    c.parent = Some(sid);
    let cid = d.add(c);
    d.normalize();
    d.set_override(ElementOverride { y: Some(900.0), ..ElementOverride::new(cid) });
    let v: serde_json::Value = serde_json::from_str(&export::ugui::json(&d, true)).unwrap();
    let hs = v["portrait"]["contentHeights"].as_array().expect("portrait content heights");
    assert_eq!(hs[0]["id"].as_u64(), Some(sid));
    assert!(hs[0]["contentHeight"].as_f64().unwrap() > 900.0);
    assert!(export::ORIENTATION_CS.contains("contentHeight") && export::ORIENTATION_CS.contains("content.sizeDelta"));
}

// ------------------------------------------------------------------ portrait fold / copy rules (app.rs)

mod portrait {
    use super::*;
    use crate::app::{fold_portrait_edit_with, EditorApp};
    use std::collections::{HashMap, HashSet};

    fn base() -> Doc {
        let mut d = Doc::empty(1920.0, 1080.0);
        let mut a = Element::new(0, Kind::Text);
        a.name = "Label".into();
        a.x = 100.0;
        a.y = 100.0;
        a.w = 300.0;
        a.h = 60.0;
        a.font_size = 40.0;
        a.rot = 0.0;
        d.add(a);
        d.portrait = Some(PortraitLayout { width: 1080.0, height: 1920.0, overrides: vec![] });
        d
    }

    #[test]
    fn element_created_in_portrait_keeps_landscape_defaults() {
        let real0 = base();
        let old = real0.resolved_for(Orientation::Portrait);
        let mut new = old.clone();
        let mut e = Element::new(0, Kind::Text);
        e.name = "New".into();
        e.x = 100.0;
        e.y = 1500.0; // below the landscape page
        e.w = 300.0;
        e.h = 60.0;
        e.font_size = 64.0;
        e.rot = 30.0;
        e.visible = false;
        e.anchor_min = [0.0, 0.0];
        e.anchor_max = [1.0, 1.0];
        let id = new.add(e);
        let mut created = HashSet::new();
        created.insert(id);
        let mut real = real0.clone();
        fold_portrait_edit_with(&mut real, &old, &new, &HashMap::new(), &created);
        let l = real.get(id).unwrap();
        let def = Element::new(id, Kind::Text);
        assert_eq!((l.rot, l.visible, l.font_size, l.anchor_min, l.anchor_max), (def.rot, def.visible, def.font_size, def.anchor_min, def.anchor_max));
        assert!(l.y + l.h <= 1080.0 + 0.5, "landscape box is inside the landscape page");
        let o = real.override_of(id).expect("portrait values are stored as an override");
        assert_eq!((o.rot, o.visible, o.font_size), (Some(30.0), Some(false), Some(64.0)));
        assert_eq!(o.anchor_max, Some([1.0, 1.0]));
        assert_eq!(o.y, Some(1500.0));
        // and the portrait view shows exactly what was drawn
        let back = real.resolved_for(Orientation::Portrait);
        assert_eq!(back.get(id).unwrap().y, 1500.0);
    }

    #[test]
    fn duplicate_in_portrait_starts_from_the_landscape_source() {
        let mut real0 = base();
        let src_id = real0.elements[0].id;
        real0.set_override(ElementOverride { x: Some(10.0), font_size: Some(20.0), visible: Some(true), ..ElementOverride::new(src_id) });
        let old = real0.resolved_for(Orientation::Portrait);
        let mut new = old.clone();
        let mut copy = new.get(src_id).unwrap().clone();
        copy.x += 20.0;
        copy.y += 20.0;
        let id = {
            let mut c = copy;
            c.name = "Label (copia)".into();
            new.add(c)
        };
        let mut src = HashMap::new();
        src.insert(id, src_id);
        let mut real = real0.clone();
        fold_portrait_edit_with(&mut real, &old, &new, &src, &HashSet::new());
        let l = real.get(id).unwrap();
        assert_eq!((l.font_size, l.x), (40.0, 120.0), "landscape: source landscape values plus the same shift");
        let o = real.override_of(id).unwrap();
        assert_eq!((o.font_size, o.x), (Some(20.0), Some(30.0)));
    }

    #[test]
    fn group_in_portrait_takes_the_landscape_box_of_its_children() {
        let mut real0 = base();
        let mut b = Element::new(0, Kind::Panel);
        b.x = 600.0;
        b.y = 300.0;
        b.w = 100.0;
        b.h = 100.0;
        real0.add(b);
        let ids: Vec<u64> = real0.elements.iter().map(|e| e.id).collect();
        for id in &ids {
            real0.set_override(ElementOverride { x: Some(20.0), y: Some(1000.0 + *id as f32 * 100.0), ..ElementOverride::new(*id) });
        }
        let old = real0.resolved_for(Orientation::Portrait);
        let mut new = old.clone();
        let mut g = Element::new(0, Kind::Empty);
        g.x = 20.0;
        g.y = 1100.0;
        g.w = 400.0;
        g.h = 300.0;
        let gid = new.add(g);
        for id in &ids {
            new.get_mut(*id).unwrap().parent = Some(gid);
        }
        new.normalize();
        let mut created = HashSet::new();
        created.insert(gid);
        let mut real = real0.clone();
        fold_portrait_edit_with(&mut real, &old, &new, &HashMap::new(), &created);
        let g = real.get(gid).unwrap();
        assert_eq!((g.x, g.y), (100.0, 100.0), "landscape group box wraps the landscape boxes");
        assert_eq!((g.x + g.w, g.y + g.h), (700.0, 400.0));
        assert_eq!(real.override_of(gid).and_then(|o| o.y), Some(1100.0));
    }

    #[test]
    fn app_duplicate_and_paste_in_landscape_carry_the_overrides() {
        let mut a = EditorApp::new(None);
        a.doc = base();
        let id = a.doc.elements[0].id;
        a.doc.set_override(ElementOverride { x: Some(10.0), font_size: Some(20.0), ..ElementOverride::new(id) });
        a.sel = vec![id];
        a.duplicate_selection(30.0);
        let dup = a.sel[0];
        assert_ne!(dup, id);
        let o = a.doc.override_of(dup).expect("the copy has the portrait override");
        assert_eq!((o.x, o.font_size), (Some(40.0), Some(20.0)));
        assert!(a.doc.override_of(id).is_some());
        a.sel = vec![id];
        a.copy();
        a.paste();
        let pasted = a.sel[0];
        assert!(pasted != id && pasted != dup);
        assert_eq!(a.doc.override_of(pasted).and_then(|o| o.font_size), Some(20.0));
        assert_eq!(a.doc.override_of(pasted).and_then(|o| o.x), Some(50.0));
    }
}

/// `EDITORGUI_DUMP=<folder> cargo test dump_for_csc -- --ignored`: writes one Unity project per case (every template with
/// all outputs on, the bundled designs and designs with hostile names) so the generated C# can be compiled with csc.
#[test]
#[ignore]
fn dump_for_csc() {
    let out = PathBuf::from(std::env::var("EDITORGUI_DUMP").expect("EDITORGUI_DUMP"));
    let mut cases: Vec<(String, Doc)> = Vec::new();
    crate::library::set_design_folder(&std::path::Path::new("designs").join("x.editorgui.json"));
    for i in 0..model::TEMPLATES.len() {
        cases.push((format!("template{}", i), model::template(i, 1920.0, 1080.0)));
    }
    for f in std::fs::read_dir("designs").unwrap().flatten() {
        if f.path().extension().is_some_and(|e| e == "json") {
            let d = Doc::from_json_str(&std::fs::read_to_string(f.path()).unwrap()).unwrap();
            cases.push((format!("design_{}", f.file_name().to_string_lossy().replace('.', "_")), d));
        }
    }
    let mut hostile = menu_doc(&[
        ("Debug", Kind::Button, Some("Debug")),
        ("Menu", Kind::Panel, Some("Menu")),
        ("MenuViewBase", Kind::Button, Some("MenuViewBase")),
        ("Play", Kind::Button, None),
        ("PlayClick", Kind::Panel, Some("OnPlayClick")),
        ("play", Kind::Button, Some("play")),
        ("UnityEngine", Kind::Text, Some("UnityEngine")),
        ("Button", Kind::Button, Some("Button")),
        ("Image", Kind::Image, Some("Image")),
        ("TMPro", Kind::Text, Some("TMPro")),
        ("class", Kind::Toggle, Some("class")),
        ("T", Kind::Slider, Some("T")),
        ("Component", Kind::Panel, Some("Component")),
        ("__text", Kind::Panel, None),
        ("Bind", Kind::Button, Some("Bind")),
        ("Find", Kind::Button, Some("Find")),
        ("Awake", Kind::Panel, Some("Awake")),
        ("OnEnable", Kind::Panel, Some("OnEnable")),
        ("MonoBehaviour", Kind::Panel, Some("MonoBehaviour")),
        ("Transform", Kind::Panel, Some("transform")),
    ]);
    hostile.unity.generate.imgui = Some(true);
    cases.push(("hostile_menu".into(), hostile));
    let mut debug_title = menu_doc(&[("A", Kind::Button, Some("Debug")), ("Button", Kind::Button, None)]);
    debug_title.title = "Debug".into();
    cases.push(("hostile_debug".into(), debug_title));
    for (name, mut d) in cases {
        d.unity.generate = GenerateOutputs { prefab: true, uitk: true, imgui: Some(true), view_scripts: true };
        d.canvas.safe_area = true;
        if d.portrait.is_none() {
            if let Some(first) = d.elements.first().map(|e| e.id) {
                d.set_override(ElementOverride { x: Some(5.0), ..ElementOverride::new(first) });
            }
        }
        let dir = out.join(&name);
        std::fs::create_dir_all(dir.join("Assets")).unwrap();
        std::fs::create_dir_all(dir.join("ProjectSettings")).unwrap();
        export::write_unity_package(&d, &dir).unwrap();
    }
}
