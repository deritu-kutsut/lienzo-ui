//! Tests of the format extensions (see docext.rs and docs/FORMAT.md).

use crate::docext::*;
use crate::model::{Doc, Element, Kind, template};
use std::collections::HashSet;

const NEW_DOC_KEYS: [&str; 3] = ["canvas", "unity", "portrait"];
const NEW_EL_KEYS: [&str; 6] = ["unity_name", "slice", "ppu", "sprite", "image_mode", "preserve_aspect"];

#[test]
fn minimal_hand_written_files_load() {
    let text = r#"{"title":"T","width":800,"height":600,"background":[10,20,30,255],
        "elements":[{"id":5,"name":"a","kind":"Panel","x":1,"y":2,"w":30,"h":40},
                    {"name":"b","kind":"Text","x":0,"y":0,"w":10,"h":10},
                    {"id":0,"name":"c","kind":"Button","x":0,"y":0,"w":10,"h":10},
                    {"id":5,"name":"dup","kind":"Image","x":0,"y":0,"w":10,"h":10}]}"#;
    let d = Doc::from_json_str(text).unwrap();
    assert_eq!(d.canvas_match, 0.5);
    assert_eq!(d.elements.len(), 4);
    let ids: HashSet<u64> = d.elements.iter().map(|e| e.id).collect();
    assert_eq!(ids.len(), 4, "missing / zero / duplicate ids are assigned");
    assert!(!ids.contains(&0));
    assert!(d.next_id > *ids.iter().max().unwrap());
    assert_eq!(d.elements[0].id, 5);
    assert!(d.elements[0].visible && d.elements[0].interactable && d.elements[0].raycast);
    assert_eq!(d.elements[0].pivot, [0.5, 0.5]);
    assert!(d.unity.generate.imgui_enabled(), "no unity block keeps the old behaviour");
}

#[test]
fn file_with_only_elements_or_nothing_loads() {
    let d = Doc::from_json_str("{}").unwrap();
    assert_eq!((d.next_id, d.canvas_match, d.elements.len()), (1, 0.5, 0));
    let d = Doc::from_json_str("\u{feff}{\"elements\":[{\"id\":7,\"kind\":\"Slider\"}]}").unwrap();
    assert_eq!(d.next_id, 8);
    assert_eq!((d.width, d.height), (1920.0, 1080.0));
    assert!(Doc::from_json_str("[1]").is_err());
}

#[test]
fn old_designs_round_trip_without_new_keys() {
    for f in std::fs::read_dir("designs").unwrap().flatten() {
        if f.path().extension().is_none_or(|e| e != "json") {
            continue;
        }
        let text = std::fs::read_to_string(f.path()).unwrap();
        let d = Doc::from_json_str(&text).unwrap();
        let out = serde_json::to_string_pretty(&d).unwrap();
        let v: serde_json::Value = serde_json::from_str(&out).unwrap();
        for k in NEW_DOC_KEYS {
            assert!(v.get(k).is_none(), "{:?} gained {}", f.path(), k);
        }
        for e in v["elements"].as_array().unwrap() {
            for k in NEW_EL_KEYS {
                assert!(e.get(k).is_none(), "{:?} gained {}", f.path(), k);
            }
        }
        // idempotent: load(save(load(x))) saves the same bytes
        let again = serde_json::to_string_pretty(&Doc::from_json_str(&out).unwrap()).unwrap();
        assert_eq!(out, again);
        assert!(d.unity.generate.imgui_enabled());
    }
}

#[test]
fn transparent_background() {
    let mut d = Doc::empty(100.0, 100.0);
    assert!(d.has_background());
    d.background = [10, 20, 30, 0];
    assert!(!d.has_background());
    let back = Doc::from_json_str(&serde_json::to_string(&d).unwrap()).unwrap();
    assert_eq!(back.background, [10, 20, 30, 0]);
    assert!(!back.has_background());
}

#[test]
fn canvas_settings_round_trip_and_clamp() {
    let d = Doc::from_json_str(r#"{"canvas":{"render_mode":"WorldSpace","sort_order":99999999999,"safe_area":true,"embed":true,"plane_distance":-5}}"#).unwrap();
    assert_eq!(d.canvas.render_mode, CanvasRenderMode::WorldSpace);
    assert_eq!(d.canvas.sort_order, 32767);
    assert!(d.canvas.safe_area && d.canvas.embed);
    assert_eq!(d.canvas.plane_distance, 0.01);
    let d2 = Doc::from_json_str(&serde_json::to_string(&d).unwrap()).unwrap();
    assert_eq!(d, d2);
    let d = Doc::from_json_str(r#"{"canvas":{"sort_order":-99999}}"#).unwrap();
    assert_eq!(d.canvas.sort_order, -32768);
    assert_eq!(d.canvas.render_mode, CanvasRenderMode::ScreenSpaceOverlay);
    assert_eq!(d.canvas.plane_distance, 100.0);
    let mut c = CanvasSettings { plane_distance: f32::NAN, ..Default::default() };
    c.sanitize();
    assert_eq!(c.plane_distance, 100.0);
}

#[test]
fn unity_settings_defaults_and_validation() {
    assert!(Doc::from_json_str("{}").unwrap().unity.generate.imgui_enabled());
    assert!(!Doc::empty(10.0, 10.0).unity.generate.imgui_enabled(), "new designs: IMGUI off");
    let d = Doc::from_json_str(r#"{"unity":{}}"#).unwrap();
    assert!(!d.unity.generate.imgui_enabled() && d.unity.generate.prefab && d.unity.generate.uitk && d.unity.generate.view_scripts);
    let d = Doc::from_json_str(r#"{"unity":{"generate":{"imgui":true,"prefab":false}}}"#).unwrap();
    assert!(d.unity.generate.imgui_enabled() && !d.unity.generate.prefab);
    let d = Doc::from_json_str(r#"{"unity":{"scripts_dir":"Assets\\Game\\UI\\","assembly":"Game.UI","default_font":"Inter SDF","fallback_fonts":["A","\u0007","B"]}}"#).unwrap();
    assert_eq!(d.unity.scripts_dir.as_deref(), Some("Assets/Game/UI"));
    assert_eq!(d.unity.assembly.as_deref(), Some("Game.UI"));
    assert_eq!(d.unity.fallback_fonts, vec!["A", "B"]);
    for bad in ["../x", "Assets/../x", "/abs", "C:/x", "Other/x", "Assets//x", "Assets/a:b", ""] {
        assert!(normalize_scripts_dir(bad).is_none(), "{}", bad);
    }
    assert!(valid_assembly("Game_UI-2.Runtime") && !valid_assembly("1abc") && !valid_assembly("a b") && !valid_assembly("") && !valid_assembly("a/b"));
    let d = Doc::from_json_str(r#"{"unity":{"scripts_dir":"C:/evil","assembly":"a b"}}"#).unwrap();
    assert!(d.unity.scripts_dir.is_none() && d.unity.assembly.is_none());
    let d2 = Doc::from_json_str(&serde_json::to_string(&Doc::from_json_str(r#"{"unity":{"assembly":"X"}}"#).unwrap()).unwrap()).unwrap();
    assert_eq!(d2.unity.assembly.as_deref(), Some("X"));
    assert!(!d2.unity.generate.imgui_enabled());
}

fn doc3() -> Doc {
    let mut d = Doc::empty(1920.0, 1080.0);
    for i in 0..3 {
        let mut e = Element::new(0, Kind::Button);
        e.x = 100.0 * i as f32;
        e.y = 10.0;
        e.w = 80.0;
        e.h = 40.0;
        d.add(e);
    }
    d
}

#[test]
fn portrait_round_trip_and_inherit() {
    let mut d = doc3();
    let ids: Vec<u64> = d.elements.iter().map(|e| e.id).collect();
    let mut o = ElementOverride::new(ids[1]);
    o.x = Some(5.0);
    o.y = Some(500.0);
    o.visible = Some(false);
    o.font_size = Some(50.0);
    o.anchor_min = Some([0.0, 1.0]);
    assert!(d.set_override(o));
    assert!(!d.set_override(ElementOverride::new(9999)), "unknown element");
    let p = d.portrait.clone().unwrap();
    assert_eq!((p.width, p.height), (1080.0, 1920.0));
    let back = Doc::from_json_str(&serde_json::to_string(&d).unwrap()).unwrap();
    assert_eq!(back, d);

    let port = d.resolved_for(Orientation::Portrait);
    assert_eq!((port.width, port.height), (1080.0, 1920.0));
    assert!(port.portrait.is_none());
    let e = port.get(ids[1]).unwrap();
    assert_eq!((e.x, e.y, e.w, e.visible, e.font_size, e.anchor_min), (5.0, 500.0, 80.0, false, 50.0, [0.0, 1.0]));
    // no override: keeps its landscape RectTransform (centre anchors: the same offset from the page centre)
    let (b, lb) = (port.get(ids[0]).unwrap(), d.get(ids[0]).unwrap());
    assert_eq!((b.x + b.w / 2.0 - 540.0, b.y + b.h / 2.0 - 960.0), (lb.x + lb.w / 2.0 - 960.0, lb.y + lb.h / 2.0 - 540.0), "no override: inherits its anchors");
    assert_eq!((b.w, b.h), (lb.w, lb.h));
    let land = d.resolved_for(Orientation::Landscape);
    assert_eq!((land.width, land.get(ids[1]).unwrap().x), (1920.0, 100.0));
    assert!(d.get(ids[1]).unwrap().visible, "original untouched");
}

#[test]
fn portrait_set_replace_clear_and_delete() {
    let mut d = doc3();
    let ids: Vec<u64> = d.elements.iter().map(|e| e.id).collect();
    let mut o = ElementOverride::new(ids[0]);
    o.w = Some(10.0);
    d.set_override(o.clone());
    o.w = Some(20.0);
    d.set_override(o);
    assert_eq!(d.portrait.as_ref().unwrap().overrides.len(), 1);
    assert_eq!(d.override_of(ids[0]).unwrap().w, Some(20.0));
    let mut o2 = ElementOverride::new(ids[2]);
    o2.rot = Some(90.0);
    d.set_override(o2);
    assert!(d.clear_override(ids[0]) && !d.clear_override(ids[0]));
    // deleting an element drops its override
    d.remove_subtrees(&[ids[2]]);
    assert!(d.override_of(ids[2]).is_none());
    // ... also when the file brings overrides of missing ids or duplicates
    let mut d = doc3();
    d.portrait = Some(PortraitLayout { width: 10.0, height: 20.0, overrides: vec![ElementOverride::new(4242), ElementOverride::new(ids[0]), ElementOverride::new(ids[0])] });
    d.sanitize();
    assert_eq!(d.portrait.as_ref().unwrap().overrides.len(), 1);
    d.elements.retain(|e| e.id != ids[0]);
    d.normalize();
    assert!(d.portrait.as_ref().unwrap().overrides.is_empty());
    // an empty override clears
    let mut d = doc3();
    let mut o = ElementOverride::new(ids[0]);
    o.x = Some(1.0);
    d.set_override(o);
    d.set_override(ElementOverride::new(ids[0]));
    assert!(d.override_of(ids[0]).is_none());
}

#[test]
fn hostile_portrait_values_do_not_panic() {
    let mut d = doc3();
    let id = d.elements[0].id;
    d.portrait = Some(PortraitLayout {
        width: f32::NAN,
        height: 1.0e30,
        overrides: vec![ElementOverride { id, x: Some(f32::INFINITY), w: Some(-5.0), rot: Some(1.0e9), font_size: Some(f32::NAN), pivot: Some([f32::NAN, 0.0]), ..Default::default() }],
    });
    d.sanitize();
    let p = d.portrait.clone().unwrap();
    assert_eq!((p.width, p.height), (1080.0, 16384.0));
    let o = &p.overrides[0];
    assert_eq!((o.x, o.w, o.rot, o.font_size, o.pivot), (None, Some(0.0), Some(360.0), None, None));
    let r = d.resolved_for(Orientation::Portrait);
    assert!(r.get(id).unwrap().x.is_finite());
    assert!(Doc::from_json_str(r#"{"portrait":null}"#).unwrap().portrait.is_none());
    assert!(Doc::from_json_str(r#"{"portrait":{}}"#).unwrap().portrait.is_some());
}

#[test]
fn element_new_fields_skip_default_and_carry_over() {
    let e = Element::new(1, Kind::Image);
    let v = serde_json::to_value(&e).unwrap();
    for k in NEW_EL_KEYS {
        assert!(v.get(k).is_none(), "{}", k);
    }
    let mut d = Doc::empty(100.0, 100.0);
    let mut e = Element::new(0, Kind::Image);
    e.unity_name = Some("Icon".into());
    e.slice = Some([4.0, 4.0, 4.0, 4.0]);
    e.ppu = Some(64.0);
    e.sprite = Some("icon_0".into());
    e.image_mode = Some(ImageMode::Tiled);
    e.preserve_aspect = true;
    let id = d.add(e.clone());
    let back = Doc::from_json_str(&serde_json::to_string(&d).unwrap()).unwrap();
    assert_eq!(back.get(id).unwrap().slice, Some([4.0; 4]));
    assert_eq!(back.get(id).unwrap(), d.get(id).unwrap());
    // duplicate / paste path
    let items = d.collect_subtrees(&[id]);
    let new = d.insert_items(&items, None);
    let c = d.get(new[0]).unwrap();
    assert_eq!((c.unity_name.as_deref(), c.ppu, c.sprite.as_deref(), c.image_mode, c.preserve_aspect), (Some("Icon"), Some(64.0), Some("icon_0"), Some(ImageMode::Tiled), true));
    assert_eq!(c.slice, e.slice);
    // image mode
    let mut e = Element::new(0, Kind::Image);
    assert_eq!(e.effective_image_mode(), ImageMode::Simple);
    e.slice = Some([1.0; 4]);
    assert_eq!(e.effective_image_mode(), ImageMode::Sliced);
    e.image_mode = Some(ImageMode::Filled);
    assert_eq!(e.effective_image_mode(), ImageMode::Filled);
}

#[test]
fn element_new_fields_are_sanitized() {
    let mut d = Doc::empty(100.0, 100.0);
    let mut e = Element::new(0, Kind::Image);
    e.unity_name = Some("Ok\u{2028}Name\u{0}".into());
    e.slice = Some([-4.0, f32::NAN, 1.0e12, 2.0]);
    e.ppu = Some(1.0e9);
    e.sprite = Some("\u{7}".into());
    d.add(e);
    let mut e2 = Element::new(0, Kind::Image);
    e2.ppu = Some(f32::NAN);
    e2.slice = Some([0.0; 4]);
    e2.unity_name = Some("   ".into());
    d.add(e2);
    d.sanitize();
    let a = &d.elements[0];
    assert_eq!(a.unity_name.as_deref(), Some("OkName"));
    assert_eq!(a.slice, Some([0.0, 0.0, 100_000.0, 2.0]));
    assert_eq!(a.ppu, Some(10_000.0));
    assert!(a.sprite.is_none());
    let b = &d.elements[1];
    assert!(b.ppu.is_none() && b.slice.is_none() && b.unity_name.is_none());
    let mut e3 = Element::new(0, Kind::Image);
    e3.ppu = Some(0.0);
    d.add(e3);
    d.sanitize();
    assert_eq!(d.elements[2].ppu, Some(0.01));
}

#[test]
fn templates_have_no_new_keys() {
    for t in 0..crate::model::TEMPLATES.len() {
        let v = serde_json::to_value(template(t, 1920.0, 1080.0)).unwrap();
        for e in v["elements"].as_array().unwrap() {
            for k in NEW_EL_KEYS {
                assert!(e.get(k).is_none());
            }
        }
    }
}

#[test]
fn unity_name_is_used_verbatim_and_collisions_are_reported() {
    use crate::export::{unity_name_collisions, unity_names};
    let mut d = Doc::empty(100.0, 100.0);
    let mut auto = Element::new(0, Kind::Button);
    auto.name = "Play".into();
    let a = d.add(auto);
    let mut x = Element::new(0, Kind::Button);
    x.name = "Label in editor".into();
    x.unity_name = Some("Play".into());
    let b = d.add(x);
    let mut y = Element::new(0, Kind::Button);
    y.unity_name = Some("Play".into());
    let c = d.add(y);
    let mut bad = Element::new(0, Kind::Button);
    bad.name = "Fallback".into();
    bad.unity_name = Some("a/b".into());
    let e = d.add(bad);
    let n = unity_names(&d);
    assert_eq!(n[&b], "Play", "explicit name wins and is verbatim");
    assert_eq!(n[&c], "Play (2)");
    assert_eq!(n[&a], "Play (3)", "the automatic name gives way");
    assert_eq!(n[&e], "Fallback", "invalid unity_name falls back to the display name");
    assert_eq!(unity_name_collisions(&d), vec![c]);
    // children of different parents never collide
    let mut kid = Element::new(0, Kind::Text);
    kid.unity_name = Some("Play".into());
    kid.parent = Some(a);
    let k = d.add(kid);
    d.normalize();
    assert_eq!(unity_names(&d)[&k], "Play");
}

#[test]
fn review_flags_name_problems() {
    let mut d = Doc::empty(1920.0, 1080.0);
    for (name, un) in [("A", Some("Same")), ("B", Some("Same")), ("C", Some("bad/name")), ("D", None)] {
        let mut e = Element::new(0, Kind::Empty);
        e.name = name.into();
        e.unity_name = un.map(String::from);
        e.x = 10.0;
        e.y = 10.0;
        d.add(e);
    }
    let ps = crate::review::review(&d);
    let msgs: Vec<String> = ps.iter().map(|p| format!("{}: {}", p.element, crate::review::describe(p))).collect();
    assert!(msgs.iter().any(|m| m.starts_with("B: Two siblings resolve to the same Unity name")), "{:?}", msgs);
    assert!(msgs.iter().any(|m| m.starts_with("C: Invalid Unity name")), "{:?}", msgs);
    assert!(!msgs.iter().any(|m| m.starts_with("A: Two siblings") || m.starts_with("D:")), "{:?}", msgs);
}

#[test]
fn export_json_carries_new_fields_only_when_used() {
    let plain = crate::export::ugui::json(&Doc::from_json_str(&std::fs::read_to_string("designs/hud.editorgui.json").unwrap()).unwrap(), true);
    for k in ["unityName", "\"slice\"", "\"ppu\"", "\"sprite\"", "imageMode", "renderMode", "\"unity\"", "\"portrait\""] {
        assert!(!plain.contains(k), "{} in plain export", k);
    }
    let mut d = Doc::empty(1920.0, 1080.0);
    let mut e = Element::new(0, Kind::Image);
    e.name = "Editor label".into();
    e.unity_name = Some("Exact".into());
    e.slice = Some([1.0, 2.0, 3.0, 4.0]);
    e.ppu = Some(64.0);
    e.sprite = Some("frame".into());
    e.preserve_aspect = true;
    let id = d.add(e);
    d.canvas.render_mode = CanvasRenderMode::ScreenSpaceCamera;
    d.canvas.sort_order = 7;
    d.canvas.embed = true;
    d.unity.scripts_dir = Some("Assets/Game/UI".into());
    d.unity.default_font = Some("Inter SDF".into());
    let mut o = ElementOverride::new(id);
    o.x = Some(3.0);
    o.visible = Some(false);
    d.set_override(o);
    let s = crate::export::ugui::json(&d, true);
    assert!(!s.contains("null"));
    let v: serde_json::Value = serde_json::from_str(&s).unwrap();
    let el = &v["elements"][0];
    assert_eq!(el["name"], "Exact");
    assert_eq!(el["unityName"], "Exact");
    assert_eq!(el["slice"], serde_json::json!([1.0, 2.0, 3.0, 4.0]));
    assert_eq!(el["ppu"], 64.0);
    assert_eq!(el["sprite"], "frame");
    assert_eq!(el["imageMode"], "Sliced");
    assert_eq!(el["image"]["preserveAspect"], true);
    assert_eq!(v["canvas"]["renderMode"], "ScreenSpaceCamera");
    assert_eq!(v["canvas"]["sortOrder"], 7);
    assert_eq!(v["canvas"]["embed"], true);
    assert_eq!(v["unity"]["scriptsDir"], "Assets/Game/UI");
    assert_eq!(v["unity"]["generate"]["imgui"], false);
    assert_eq!(v["unity"]["defaultFont"], "Inter SDF");
    assert_eq!(v["portrait"]["width"], 1080.0);
    assert_eq!(v["portrait"]["overrides"][0]["x"], 3.0);
    assert_eq!(v["portrait"]["overrides"][0]["visible"], false);
}
