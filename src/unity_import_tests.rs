//! Tests of the Unity -> Lienzo import (see unity_import.rs and docs/UNITY_IMPORT.md): realistic prefab fixtures,
//! Lienzo export round trips and hostile inputs.

use crate::model::{Doc, Element, Kind, LayoutMode, template};
use crate::unity_import::*;
use std::path::PathBuf;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Fixture builders (small Unity-like YAML)
// ---------------------------------------------------------------------------

const G_IMAGE: &str = "fe87c0e1cc204ed48ad3b37840f39efc";
const G_BUTTON: &str = "4e29b1a8efbd4b44bb3f3716e73f07ff";
const G_TMP: &str = "f4688fdb7df04437aeb418b961361dc5";
const G_SCALER: &str = "0cd44c1031e13a943bb63640046fad76";
const G_VLG: &str = "59f8146938fff824cb5fd77236b75775";
const G_SLIDER: &str = "67db9e8f0e2ae9c40bc1e2b64352a6b4";
const G_TOGGLE: &str = "9085046f02f69544eb97fd06b6048fe2";
const G_SCROLL: &str = "1aa08ab6e0800fa44ae55d278d1423e3";
const G_DROPDOWN: &str = "7b743370ac3e4ec2a1668f5455a8ef8a";

const HEADER: &str = "%YAML 1.1\n%TAG !u! tag:unity3d.com,2011:\n";

fn go(id: i64, name: &str, comps: &[i64], active: bool) -> String {
    let mut s = format!("--- !u!1 &{id}\nGameObject:\n  m_ObjectHideFlags: 0\n  m_CorrespondingSourceObject: {{fileID: 0}}\n  m_PrefabInstance: {{fileID: 0}}\n  m_PrefabAsset: {{fileID: 0}}\n  serializedVersion: 6\n  m_Component:\n");
    for c in comps {
        s += &format!("  - component: {{fileID: {c}}}\n");
    }
    s += &format!("  m_Layer: 5\n  m_Name: {name}\n  m_TagString: Untagged\n  m_Icon: {{fileID: 0}}\n  m_NavMeshLayer: 0\n  m_StaticEditorFlags: 0\n  m_IsActive: {}\n", active as i32);
    s
}

#[derive(Clone)]
struct Rt {
    id: i64,
    go: i64,
    father: i64,
    kids: Vec<i64>,
    amin: [f32; 2],
    amax: [f32; 2],
    pos: [f32; 2],
    size: [f32; 2],
    pivot: [f32; 2],
    rot: f32,
    scale: [f32; 2],
}

impl Rt {
    fn new(id: i64, go: i64, father: i64) -> Rt {
        Rt { id, go, father, kids: vec![], amin: [0.5, 0.5], amax: [0.5, 0.5], pos: [0.0, 0.0], size: [100.0, 100.0], pivot: [0.5, 0.5], rot: 0.0, scale: [1.0, 1.0] }
    }
    fn kids(mut self, k: &[i64]) -> Rt {
        self.kids = k.to_vec();
        self
    }
    fn anchors(mut self, min: [f32; 2], max: [f32; 2]) -> Rt {
        self.amin = min;
        self.amax = max;
        self
    }
    fn pos(mut self, p: [f32; 2]) -> Rt {
        self.pos = p;
        self
    }
    fn size(mut self, s: [f32; 2]) -> Rt {
        self.size = s;
        self
    }
    fn pivot(mut self, p: [f32; 2]) -> Rt {
        self.pivot = p;
        self
    }
    fn rot(mut self, deg: f32) -> Rt {
        self.rot = deg;
        self
    }
    fn scale(mut self, s: f32) -> Rt {
        self.scale = [s, s];
        self
    }
    fn stretch(self) -> Rt {
        self.anchors([0.0, 0.0], [1.0, 1.0]).size([0.0, 0.0])
    }
    fn yaml(&self) -> String {
        let (s, c) = (self.rot / 2.0).to_radians().sin_cos();
        let mut y = format!("--- !u!224 &{}\nRectTransform:\n  m_ObjectHideFlags: 0\n  m_GameObject: {{fileID: {}}}\n  m_LocalRotation: {{x: 0, y: 0, z: {}, w: {}}}\n  m_LocalPosition: {{x: 0, y: 0, z: 0}}\n  m_LocalScale: {{x: {}, y: {}, z: 1}}\n", self.id, self.go, s, c, self.scale[0], self.scale[1]);
        if self.kids.is_empty() {
            y += "  m_Children: []\n";
        } else {
            y += "  m_Children:\n";
            for k in &self.kids {
                y += &format!("  - {{fileID: {k}}}\n");
            }
        }
        y += &format!(
            "  m_Father: {{fileID: {}}}\n  m_LocalEulerAnglesHint: {{x: 0, y: 0, z: 0}}\n  m_AnchorMin: {{x: {}, y: {}}}\n  m_AnchorMax: {{x: {}, y: {}}}\n  m_AnchoredPosition: {{x: {}, y: {}}}\n  m_SizeDelta: {{x: {}, y: {}}}\n  m_Pivot: {{x: {}, y: {}}}\n",
            self.father, self.amin[0], self.amin[1], self.amax[0], self.amax[1], self.pos[0], self.pos[1], self.size[0], self.size[1], self.pivot[0], self.pivot[1]
        );
        y
    }
}

fn mb(id: i64, go: i64, guid: &str, ident: &str, fields: &str) -> String {
    format!("--- !u!114 &{id}\nMonoBehaviour:\n  m_ObjectHideFlags: 0\n  m_GameObject: {{fileID: {go}}}\n  m_Enabled: 1\n  m_EditorHideFlags: 0\n  m_Script: {{fileID: 11500000, guid: {guid}, type: 3}}\n  m_Name: \n  m_EditorClassIdentifier: {ident}\n{fields}")
}

fn native(class: u32, id: i64, go: i64, fields: &str) -> String {
    let name = match class {
        223 => "Canvas",
        225 => "CanvasGroup",
        222 => "CanvasRenderer",
        95 => "Animator",
        _ => "Component",
    };
    format!("--- !u!{class} &{id}\n{name}:\n  m_ObjectHideFlags: 0\n  m_GameObject: {{fileID: {go}}}\n{fields}")
}

fn image(id: i64, go: i64, color: [f32; 4], sprite: Option<(i64, &str)>, ty: i32) -> String {
    let sp = match sprite {
        Some((f, g)) => format!("{{fileID: {f}, guid: {g}, type: 3}}"),
        None => "{fileID: 0}".to_string(),
    };
    mb(
        id,
        go,
        G_IMAGE,
        "UnityEngine.UI::UnityEngine.UI.Image",
        &format!("  m_Color: {{r: {}, g: {}, b: {}, a: {}}}\n  m_RaycastTarget: 1\n  m_Sprite: {sp}\n  m_Type: {ty}\n  m_PreserveAspect: 0\n  m_FillCenter: 1\n  m_PixelsPerUnitMultiplier: 1\n", color[0], color[1], color[2], color[3]),
    )
}

fn tmp(id: i64, go: i64, text: &str, size: f32, color: [f32; 4], extra: &str) -> String {
    mb(id, go, G_TMP, "Unity.TextMeshPro::TMPro.TextMeshProUGUI", &format!("  m_Color: {{r: 1, g: 1, b: 1, a: 1}}\n  m_RaycastTarget: 1\n  m_text: {text}\n  m_fontSize: {size}\n  m_fontColor: {{r: {}, g: {}, b: {}, a: {}}}\n{extra}", color[0], color[1], color[2], color[3]))
}

fn selectable_fields(target: i64, extra: &str) -> String {
    format!("  m_Navigation:\n    m_Mode: 3\n  m_Transition: 1\n  m_Interactable: 1\n  m_TargetGraphic: {{fileID: {target}}}\n{extra}")
}

fn prefab(parts: &[String]) -> String {
    let mut s = HEADER.to_string();
    for p in parts {
        s += p;
    }
    s
}

fn import(text: &str) -> Imported {
    let mut a = Assets::none();
    import_prefab_text(text, "Test", &mut a).expect("import")
}

fn tmpdir(tag: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("lienzo_unity_import_{}_{}_{:?}", tag, std::process::id(), std::thread::current().id()).replace(['(', ')', ' '], "_"));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn by_name<'a>(d: &'a Doc, n: &str) -> &'a Element {
    d.elements.iter().find(|e| e.name == n).unwrap_or_else(|| panic!("no element {}: {:?}", n, d.elements.iter().map(|e| &e.name).collect::<Vec<_>>()))
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.05
}

// A canvas "Menu" with a panel "Window" (400x300 at +50 y) holding a Button "Play" with a TMP label.
fn menu_prefab() -> String {
    prefab(&[
        go(1, "Menu", &[11, 12, 13, 14, 15], true),
        Rt::new(11, 1, 0).kids(&[21]).stretch().yaml(),
        native(223, 12, 1, "  m_Enabled: 1\n  serializedVersion: 3\n  m_RenderMode: 1\n  m_PlaneDistance: 50\n  m_PixelPerfect: 0\n  m_SortingOrder: 7\n"),
        mb(13, 1, G_SCALER, "", "  m_UiScaleMode: 1\n  m_ReferencePixelsPerUnit: 100\n  m_ScaleFactor: 1\n  m_ReferenceResolution: {x: 1280, y: 720}\n  m_ScreenMatchMode: 0\n  m_MatchWidthOrHeight: 0.3\n"),
        native(222, 14, 1, ""),
        mb(15, 1, "dc42784cf147c0c48a680349fa168899", "", "  m_IgnoreReversedGraphics: 1\n  m_BlockingObjects: 0\n"),
        go(2, "Window", &[21, 22], true),
        Rt::new(21, 2, 11).kids(&[31]).pos([0.0, 50.0]).size([400.0, 300.0]).yaml(),
        image(22, 2, [1.0, 0.0, 0.0, 1.0], None, 0),
        go(3, "Play", &[31, 32, 33], true),
        Rt::new(31, 3, 21).kids(&[41]).pos([0.0, -60.0]).size([160.0, 40.0]).yaml(),
        image(32, 3, [0.0, 0.0, 1.0, 1.0], None, 0),
        mb(33, 3, G_BUTTON, "UnityEngine.UI::UnityEngine.UI.Button", &selectable_fields(32, "  m_OnClick:\n    m_PersistentCalls:\n      m_Calls:\n      - m_Target: {fileID: 0}\n        m_MethodName: Go\n")),
        go(4, "Text (TMP)", &[41, 42], true),
        Rt::new(41, 4, 31).stretch().yaml(),
        tmp(42, 4, "Play", 40.0, [0.0, 1.0, 0.0, 1.0], "  m_fontStyle: 1\n  m_HorizontalAlignment: 2\n  m_VerticalAlignment: 512\n"),
    ])
}

// ---------------------------------------------------------------------------
// YAML reader
// ---------------------------------------------------------------------------

#[test]
fn yaml_reader_handles_unity_forms() {
    let t = "GameObject:\n  m_Component:\n  - component: {fileID: 5}\n  - component: {fileID: 6}\n  m_Name: \"A \\\"quoted\\\" \\u00e9 name\"\n  m_Long: \"first line\n    second line\"\n  m_Empty: \n  m_Flow: {a: 1, b: {c: [1, 2, 3], d: 'it''s'}}\n  m_Items:\n  - first:\n      213: 21300000\n    second: icon\n  - {x: 1}\n  m_List: []\n  m_Plain: hello world: again\n";
    let y = parse_plain(t);
    let g = y.get("GameObject").unwrap();
    assert_eq!(g.get("m_Component").unwrap().seq().len(), 2);
    assert_eq!(g.get("m_Name").unwrap().s().unwrap(), "A \"quoted\" \u{e9} name");
    assert_eq!(g.get("m_Long").unwrap().s().unwrap(), "first line second line");
    assert!(matches!(g.get("m_Empty"), Some(crate::unity_import::Y::Null)));
    let flow = g.get("m_Flow").unwrap();
    assert_eq!(flow.get("b").unwrap().get("c").unwrap().seq().len(), 3);
    assert_eq!(flow.get("b").unwrap().get("d").unwrap().s().unwrap(), "it's");
    let items = g.get("m_Items").unwrap().seq();
    assert_eq!(items[0].get("second").unwrap().s().unwrap(), "icon");
    assert_eq!(items[0].get("first").unwrap().get("213").unwrap().s().unwrap(), "21300000");
    assert_eq!(items[1].get("x").unwrap().s().unwrap(), "1");
    assert_eq!(g.get("m_Plain").unwrap().s().unwrap(), "hello world: again");
}

// ---------------------------------------------------------------------------
// Prefab import
// ---------------------------------------------------------------------------

#[test]
fn canvas_panel_and_button_with_tmp_label() {
    let imp = import(&menu_prefab());
    let d = &imp.doc;
    // page and canvas
    assert_eq!((d.width, d.height), (1280.0, 720.0));
    assert!(close(d.canvas_match, 0.3));
    assert_eq!(d.canvas.render_mode, crate::docext::CanvasRenderMode::ScreenSpaceCamera);
    assert_eq!(d.canvas.sort_order, 7);
    assert!(close(d.canvas.plane_distance, 50.0));
    assert!(!d.canvas.embed);
    assert!(!d.has_background(), "a Unity canvas has no background colour");
    // the label child of the button is absorbed: two elements
    assert_eq!(d.elements.len(), 2, "{:?}", d.elements.iter().map(|e| &e.name).collect::<Vec<_>>());
    let win = by_name(d, "Window");
    let play = by_name(d, "Play");
    assert_eq!(win.kind, Kind::Panel);
    assert_eq!(win.parent, None);
    assert_eq!(play.parent, Some(win.id));
    assert_eq!(play.kind, Kind::Button);
    // 400x300 centred, +50 up: x = 640-200, y = 360-50-150
    assert!(close(win.x, 440.0) && close(win.y, 160.0) && close(win.w, 400.0) && close(win.h, 300.0), "{:?}", (win.x, win.y, win.w, win.h));
    assert_eq!(win.anchor_min, [0.5, 0.5]);
    assert_eq!(win.fill, [255, 0, 0, 255]);
    // button 160x40 at -60 inside the window (centre 640,310): centre y 370
    assert!(close(play.x + play.w / 2.0, 640.0) && close(play.y + play.h / 2.0, 370.0));
    assert_eq!(play.text, "Play");
    assert!(close(play.font_size, 40.0));
    assert_eq!(play.text_color, [0, 255, 0, 255]);
    assert!(play.bold && !play.italic);
    assert_eq!((play.align, play.valign), (1, 1));
    assert_eq!(play.fill, [0, 0, 255, 255]);
    // names are kept verbatim for game code
    assert_eq!(win.unity_name.as_deref(), Some("Window"));
    assert_eq!(play.unity_name.as_deref(), Some("Play"));
    assert!(imp.warnings.iter().any(|w| w.contains("UnityEvent")), "{:?}", imp.warnings);
    // the design is a valid, repaired document
    let json = serde_json::to_string(d).unwrap();
    assert!(Doc::from_json_str(&json).is_ok());
}

#[test]
fn inactive_objects_are_not_visible_and_unsupported_components_warn_once() {
    let p = prefab(&[
        go(1, "Root", &[11, 12], true),
        Rt::new(11, 1, 0).kids(&[21, 31, 41]).size([800.0, 600.0]).yaml(),
        native(223, 12, 1, "  m_RenderMode: 0\n"),
        go(2, "Hidden", &[21, 22, 23], false),
        Rt::new(21, 2, 11).size([50.0, 50.0]).yaml(),
        image(22, 2, [1.0, 1.0, 1.0, 1.0], None, 0),
        native(95, 23, 2, ""),
        go(3, "Walker", &[31, 32], true),
        Rt::new(31, 3, 11).size([50.0, 50.0]).yaml(),
        native(95, 32, 3, ""),
        go(4, "Fitter", &[41, 42], true),
        Rt::new(41, 4, 11).size([50.0, 50.0]).yaml(),
        mb(42, 4, "3245ec927659c4140ac4f8d17403cc18", "", "  m_HorizontalFit: 0\n  m_VerticalFit: 2\n"),
    ]);
    let imp = import(&p);
    assert!(!by_name(&imp.doc, "Hidden").visible);
    assert!(by_name(&imp.doc, "Walker").visible);
    let animator: Vec<_> = imp.warnings.iter().filter(|w| w.contains("Animator")).collect();
    assert_eq!(animator.len(), 1, "one warning per component type: {:?}", imp.warnings);
    assert!(animator[0].contains("Hidden") && animator[0].contains("1 more"), "{}", animator[0]);
    assert!(imp.warnings.iter().any(|w| w.contains("ContentSizeFitter") && w.contains("Fitter")), "{:?}", imp.warnings);
}

#[test]
fn vertical_layout_group_with_children() {
    let lg = "  m_Padding:\n    m_Left: 10\n    m_Right: 12\n    m_Top: 14\n    m_Bottom: 16\n  m_ChildAlignment: 4\n  m_Spacing: 8\n  m_ChildForceExpandWidth: 1\n  m_ChildForceExpandHeight: 0\n  m_ChildControlWidth: 1\n  m_ChildControlHeight: 0\n  m_ChildScaleWidth: 0\n  m_ChildScaleHeight: 0\n  m_ReverseArrangement: 0\n";
    let p = prefab(&[
        go(1, "Root", &[11, 12], true),
        Rt::new(11, 1, 0).kids(&[21]).size([1000.0, 800.0]).yaml(),
        native(223, 12, 1, "  m_RenderMode: 0\n"),
        go(2, "List", &[21, 22], true),
        Rt::new(21, 2, 11).kids(&[31, 41, 51]).anchors([0.0, 0.0], [0.0, 1.0]).pos([0.0, 0.0]).size([300.0, 0.0]).pivot([0.0, 0.5]).yaml(),
        mb(22, 2, G_VLG, "UnityEngine.UI::UnityEngine.UI.VerticalLayoutGroup", lg),
        go(3, "A", &[31], true),
        Rt::new(31, 3, 21).anchors([0.0, 1.0], [1.0, 1.0]).pivot([0.5, 1.0]).pos([0.0, -14.0]).size([-22.0, 60.0]).yaml(),
        go(4, "B", &[41], true),
        Rt::new(41, 4, 21).anchors([0.0, 1.0], [1.0, 1.0]).pivot([0.5, 1.0]).pos([0.0, -82.0]).size([-22.0, 60.0]).yaml(),
        go(5, "C", &[51], true),
        Rt::new(51, 5, 21).anchors([0.0, 1.0], [1.0, 1.0]).pivot([0.5, 1.0]).pos([0.0, -150.0]).size([-22.0, 60.0]).yaml(),
    ]);
    let imp = import(&p);
    let d = &imp.doc;
    let list = by_name(d, "List");
    assert_eq!(list.kind, Kind::Empty);
    assert_eq!(list.layout.mode, LayoutMode::Vertical);
    assert_eq!(list.layout.padding, [10.0, 12.0, 14.0, 16.0]);
    assert_eq!(list.layout.spacing[1], 8.0);
    assert_eq!(list.layout.align, 4);
    assert!(list.layout.expand_w && !list.layout.expand_h);
    assert_eq!(d.children(Some(list.id)).len(), 3, "children keep their order");
    let names: Vec<&str> = d.children(Some(list.id)).iter().map(|i| d.get(*i).unwrap().name.as_str()).collect();
    assert_eq!(names, ["A", "B", "C"]);
    let a = by_name(d, "A");
    assert!(close(a.w, 300.0 - 22.0) && close(a.h, 60.0));
    assert!(close(a.y, list.y + 14.0), "{} vs {}", a.y, list.y);
}

fn slider_parts(extra_color: bool) -> Vec<String> {
    let mut v = vec![
        go(1, "Root", &[11, 12], true),
        Rt::new(11, 1, 0).kids(&[21]).size([1000.0, 800.0]).yaml(),
        native(223, 12, 1, "  m_RenderMode: 0\n"),
        go(2, "Volume", &[21, 22], true),
        Rt::new(21, 2, 11).kids(&[31, 41, 51]).size([600.0, 40.0]).yaml(),
        mb(22, 2, G_SLIDER, "UnityEngine.UI::UnityEngine.UI.Slider", &format!("{}  m_FillRect: {{fileID: 61}}\n  m_HandleRect: {{fileID: 71}}\n  m_Direction: 0\n  m_MinValue: 0\n  m_MaxValue: 10\n  m_WholeNumbers: 0\n  m_Value: 2.5\n", selectable_fields(72, ""))),
        go(3, "Background", &[31, 32], true),
        Rt::new(31, 3, 21).stretch().yaml(),
        image(32, 3, [0.2, 0.2, 0.2, 1.0], None, 0),
        go(4, "Fill Area", &[41], true),
        Rt::new(41, 4, 21).kids(&[61]).stretch().yaml(),
        go(5, "Handle Slide Area", &[51], true),
        Rt::new(51, 5, 21).kids(&[71]).stretch().yaml(),
        go(6, "Fill", &[61, 62], true),
        Rt::new(61, 6, 41).size([10.0, 0.0]).yaml(),
        image(62, 6, if extra_color { [1.0, 0.5, 0.0, 1.0] } else { [1.0, 1.0, 1.0, 1.0] }, None, 0),
        go(7, "Handle", &[71, 72], true),
        Rt::new(71, 7, 51).size([20.0, 0.0]).yaml(),
        image(72, 7, [1.0, 1.0, 1.0, 1.0], None, 0),
    ];
    v.shrink_to_fit();
    v
}

#[test]
fn slider_collapses_its_standard_children() {
    let imp = import(&prefab(&slider_parts(true)));
    let d = &imp.doc;
    assert_eq!(d.elements.len(), 1, "Background / Fill Area / Handle Slide Area belong to the slider: {:?}", d.elements.iter().map(|e| &e.name).collect::<Vec<_>>());
    let s = &d.elements[0];
    assert_eq!(s.kind, Kind::Slider);
    assert!(close(s.value, 0.25), "value is normalised: {}", s.value);
    assert_eq!(s.fill, [255, 128, 0, 255], "fill colour comes from the Fill image");
    assert_eq!(s.fill2, [51, 51, 51, 255], "track colour comes from Background");
    assert!(close(s.w, 600.0) && close(s.h, 40.0));
}

#[test]
fn toggle_dropdown_and_scrollview_controls() {
    let p = prefab(&[
        go(1, "Root", &[11, 12], true),
        Rt::new(11, 1, 0).kids(&[21, 31, 41]).size([1000.0, 800.0]).yaml(),
        native(223, 12, 1, "  m_RenderMode: 0\n"),
        // toggle with Background / Checkmark / Label
        go(2, "Sound", &[21, 22], true),
        Rt::new(21, 2, 11).kids(&[24, 26]).pos([-300.0, 200.0]).size([300.0, 40.0]).yaml(),
        mb(22, 2, G_TOGGLE, "UnityEngine.UI::UnityEngine.UI.Toggle", &format!("{}  toggleTransition: 1\n  graphic: {{fileID: 27}}\n  m_Group: {{fileID: 0}}\n  m_IsOn: 1\n", selectable_fields(25, ""))),
        go(3, "Background", &[24, 25], true),
        Rt::new(24, 3, 21).kids(&[28]).anchors([0.0, 1.0], [0.0, 1.0]).size([20.0, 20.0]).yaml(),
        image(25, 3, [1.0, 1.0, 1.0, 1.0], None, 0),
        go(8, "Checkmark", &[28, 27], true),
        Rt::new(28, 8, 24).size([20.0, 20.0]).yaml(),
        image(27, 8, [0.0, 0.5, 1.0, 1.0], None, 0),
        go(4, "Label", &[26, 29], true),
        Rt::new(26, 4, 21).stretch().yaml(),
        tmp(29, 4, "Sound on", 24.0, [0.1, 0.1, 0.1, 1.0], ""),
        // dropdown with options
        go(5, "Quality", &[31, 32, 33], true),
        Rt::new(31, 5, 11).kids(&[34, 36]).pos([0.0, 100.0]).size([300.0, 50.0]).yaml(),
        image(32, 5, [0.9, 0.9, 0.9, 1.0], None, 0),
        mb(
            33,
            5,
            G_DROPDOWN,
            "Unity.TextMeshPro::TMPro.TMP_Dropdown",
            &format!("{}  m_Template: {{fileID: 37}}\n  m_CaptionText: {{fileID: 35}}\n  m_Value: 1\n  m_Options:\n    m_Options:\n    - m_Text: Low\n      m_Image: {{fileID: 0}}\n    - m_Text: High\n      m_Image: {{fileID: 0}}\n", selectable_fields(32, "")),
        ),
        go(6, "Label", &[34, 35], true),
        Rt::new(34, 6, 31).stretch().yaml(),
        tmp(35, 6, "High", 22.0, [0.0, 0.0, 0.0, 1.0], ""),
        go(7, "Template", &[36, 37], false),
        Rt::new(36, 7, 31).size([300.0, 150.0]).yaml(),
        native(222, 37, 7, ""),
        // scroll view
        go(210, "Items", &[41, 42, 43], true),
        Rt::new(41, 210, 11).kids(&[51]).pos([0.0, -150.0]).size([400.0, 300.0]).yaml(),
        image(42, 210, [0.5, 0.5, 0.5, 1.0], None, 0),
        mb(43, 210, G_SCROLL, "UnityEngine.UI::UnityEngine.UI.ScrollRect", "  m_Content: {fileID: 61}\n  m_Horizontal: 0\n  m_Vertical: 1\n  m_Viewport: {fileID: 51}\n"),
        go(211, "Viewport", &[51, 52], true),
        Rt::new(51, 211, 41).kids(&[61]).stretch().yaml(),
        mb(52, 211, "3312d7739989d2b4e91e6319e9a96d76", "", "  m_Padding: {x: 0, y: 0, z: 0, w: 0}\n  m_Softness: {x: 0, y: 0}\n"),
        go(212, "Content", &[61], true),
        Rt::new(61, 212, 51).kids(&[71]).anchors([0.0, 1.0], [1.0, 1.0]).pivot([0.0, 1.0]).pos([0.0, 0.0]).size([0.0, 500.0]).yaml(),
        go(213, "Item", &[71, 72], true),
        Rt::new(71, 213, 61).anchors([0.0, 1.0], [0.0, 1.0]).pivot([0.0, 1.0]).pos([20.0, -30.0]).size([100.0, 40.0]).yaml(),
        image(72, 213, [1.0, 0.0, 1.0, 1.0], None, 0),
    ]);
    let imp = import(&p);
    let d = &imp.doc;
    let names: Vec<&str> = d.elements.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["Sound", "Quality", "Items", "Item"], "{:?}", names);
    let t = by_name(d, "Sound");
    assert_eq!(t.kind, Kind::Toggle);
    assert_eq!(t.value, 1.0);
    assert_eq!(t.text, "Sound on");
    assert_eq!(t.fill, [0, 128, 255, 255], "the checkmark colour is the toggle colour");
    let q = by_name(d, "Quality");
    assert_eq!(q.kind, Kind::Dropdown);
    assert_eq!(q.options, ["Low", "High"]);
    assert_eq!(q.value, 1.0);
    assert!(close(q.font_size, 22.0));
    let sv = by_name(d, "Items");
    assert_eq!(sv.kind, Kind::ScrollView);
    assert!(sv.mask, "RectMask2D on the viewport");
    let item = by_name(d, "Item");
    assert_eq!(item.parent, Some(sv.id), "children of Content hang from the ScrollView");
    // content is top-aligned and as wide as the view: item at (+20, +30) from the top-left corner
    assert!(close(item.x, sv.x + 20.0) && close(item.y, sv.y + 30.0), "{:?} vs {:?}", (item.x, item.y), (sv.x, sv.y));
    assert!(close(item.w, 100.0) && close(item.h, 40.0));
}

#[test]
fn rotation_scale_and_pivot() {
    let p = prefab(&[
        go(1, "Root", &[11, 12], true),
        Rt::new(11, 1, 0).kids(&[21, 31]).size([1000.0, 800.0]).yaml(),
        native(223, 12, 1, "  m_RenderMode: 0\n"),
        go(2, "Turned", &[21, 22], true),
        Rt::new(21, 2, 11).size([200.0, 100.0]).rot(30.0).yaml(),
        image(22, 2, [1.0, 1.0, 1.0, 1.0], None, 0),
        go(3, "Big", &[31, 32], true),
        Rt::new(31, 3, 11).size([100.0, 50.0]).scale(2.0).pivot([0.0, 1.0]).pos([-400.0, 300.0]).yaml(),
        image(32, 3, [1.0, 1.0, 1.0, 1.0], None, 0),
    ]);
    let imp = import(&p);
    let t = by_name(&imp.doc, "Turned");
    assert!(close(t.rot, -30.0), "counter-clockwise 30 in Unity is -30 clockwise: {}", t.rot);
    assert!(close(t.w, 200.0) && close(t.h, 100.0));
    let b = by_name(&imp.doc, "Big");
    assert!(close(b.w, 200.0) && close(b.h, 100.0), "scale 2 is folded into the size: {:?}", (b.w, b.h));
    // pivot top-left at (-400, +300) from the page centre (500, 400): top-left corner stays at (100, 100)
    assert!(close(b.x, 100.0) && close(b.y, 100.0), "{:?}", (b.x, b.y));
    assert!(imp.warnings.iter().any(|w| w.contains("scale")));
}

#[test]
fn prefab_without_canvas_is_an_embedded_page() {
    let p = prefab(&[
        go(1, "HealthBar", &[11], true),
        Rt::new(11, 1, 0).kids(&[21]).size([640.0, 96.0]).yaml(),
        go(2, "Label", &[21, 22], true),
        Rt::new(21, 2, 11).stretch().yaml(),
        tmp(22, 2, "HP", 30.0, [1.0, 1.0, 1.0, 1.0], ""),
    ]);
    let imp = import(&p);
    let d = &imp.doc;
    assert!(d.canvas.embed);
    assert_eq!((d.width, d.height), (640.0, 96.0));
    assert_eq!(d.elements.len(), 1);
    let l = &d.elements[0];
    assert_eq!(l.kind, Kind::Text);
    assert_eq!(l.text, "HP");
    assert!(close(l.w, 640.0) && close(l.h, 96.0) && close(l.x, 0.0));
}

#[test]
fn tmp_and_legacy_alignment_and_font_decoding() {
    let p = prefab(&[
        go(1, "Root", &[11, 12], true),
        Rt::new(11, 1, 0).kids(&[21, 31]).size([1000.0, 800.0]).yaml(),
        native(223, 12, 1, "  m_RenderMode: 0\n"),
        go(2, "BottomRight", &[21, 22], true),
        Rt::new(21, 2, 11).size([200.0, 50.0]).yaml(),
        tmp(22, 2, "x", 20.0, [1.0, 1.0, 1.0, 1.0], "  m_HorizontalAlignment: 4\n  m_VerticalAlignment: 1024\n  m_fontStyle: 6\n"),
        go(3, "TopCenterOld", &[31, 32], true),
        Rt::new(31, 3, 11).size([200.0, 50.0]).yaml(),
        tmp(32, 3, "y", 18.0, [1.0, 1.0, 1.0, 1.0], "  m_textAlignment: 258\n  m_enableAutoSizing: 1\n  m_fontSizeMax: 30\n"),
    ]);
    let d = import(&p).doc;
    let a = by_name(&d, "BottomRight");
    assert_eq!((a.align, a.valign), (2, 2));
    assert!(!a.bold && a.italic && a.underline);
    let b = by_name(&d, "TopCenterOld");
    assert_eq!((b.align, b.valign), (1, 0));
    assert!(close(b.font_size, 30.0), "auto size takes the maximum");
}

#[test]
fn lienzo_exported_structure_is_recognised() {
    // "Fondo del lienzo" background, "Borde" ring, "Texto" caption and generated sprites (rr_ / ellipse_) of Lienzo's own prefabs
    let dir = tmpdir("lienzo_struct");
    let sprites = dir.join("Assets/EditorGUI/Sprites/Generated");
    std::fs::create_dir_all(&sprites).unwrap();
    for (n, g) in [("rr_24_0", "11111111111111111111111111111111"), ("rr_24_3", "22222222222222222222222222222222"), ("ellipse_0", "33333333333333333333333333333333")] {
        std::fs::write(sprites.join(format!("{n}.png")), b"png").unwrap();
        std::fs::write(sprites.join(format!("{n}.png.meta")), format!("fileFormatVersion: 2\nguid: {g}\nTextureImporter:\n  spriteMode: 1\n  spriteBorder: {{x: 26, y: 26, z: 26, w: 26}}\n")).unwrap();
    }
    let pref_dir = dir.join("Assets/EditorGUI/Prefabs");
    std::fs::create_dir_all(&pref_dir).unwrap();
    let text = prefab(&[
        go(1, "Menu", &[11, 12, 13], true),
        Rt::new(11, 1, 0).kids(&[21, 31, 51]).stretch().yaml(),
        native(223, 12, 1, "  m_RenderMode: 0\n"),
        mb(13, 1, G_SCALER, "", "  m_UiScaleMode: 1\n  m_ReferenceResolution: {x: 1920, y: 1080}\n  m_ScreenMatchMode: 0\n  m_MatchWidthOrHeight: 0.5\n"),
        go(2, "Fondo del lienzo", &[21, 22], true),
        Rt::new(21, 2, 11).stretch().yaml(),
        image(22, 2, [0.1, 0.2, 0.3, 1.0], None, 0),
        go(3, "Card", &[31, 32], true),
        Rt::new(31, 3, 11).kids(&[41, 42]).size([500.0, 300.0]).yaml(),
        image(32, 3, [1.0, 1.0, 1.0, 1.0], Some((21300000, "11111111111111111111111111111111")), 1),
        go(4, "Borde", &[41, 43], true),
        Rt::new(41, 4, 31).stretch().yaml(),
        image(43, 4, [0.0, 0.0, 0.0, 1.0], Some((21300000, "22222222222222222222222222222222")), 1),
        go(5, "Texto", &[42, 44], true),
        Rt::new(42, 5, 31).stretch().yaml(),
        tmp(44, 5, "Hello", 28.0, [0.0, 0.0, 0.0, 1.0], ""),
        go(6, "Dot", &[51, 52], true),
        Rt::new(51, 6, 11).size([80.0, 80.0]).pos([300.0, 0.0]).yaml(),
        image(52, 6, [1.0, 0.0, 0.0, 1.0], Some((21300000, "33333333333333333333333333333333")), 0),
    ]);
    let pf = pref_dir.join("Menu.prefab");
    std::fs::write(&pf, text).unwrap();
    let imp = import_file(&pf).unwrap();
    let d = &imp.doc;
    assert_eq!(d.background, [26, 51, 77, 255], "the 'Fondo del lienzo' child is the page background: {:?}", d.background);
    assert_eq!(d.elements.len(), 2, "{:?}", d.elements.iter().map(|e| &e.name).collect::<Vec<_>>());
    let card = by_name(d, "Card");
    assert_eq!(card.kind, Kind::Panel, "a generated rounded sprite is a plain panel");
    assert_eq!(card.radius, 24.0);
    assert_eq!(card.stroke_w, 3.0, "ring sprite rr_24_3 gives the border width");
    assert_eq!(card.stroke, [0, 0, 0, 255]);
    assert_eq!(card.text, "Hello");
    assert!(card.image.is_empty() && card.slice.is_none());
    assert_eq!(by_name(d, "Dot").kind, Kind::Ellipse);
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Sprites through .meta
// ---------------------------------------------------------------------------

#[test]
fn sprites_resolve_through_meta_with_slice_ppu_and_sub_sprites() {
    let dir = tmpdir("sprites");
    let ui = dir.join("Assets/UI");
    let prefabs = dir.join("Assets/Prefabs");
    std::fs::create_dir_all(&ui).unwrap();
    std::fs::create_dir_all(&prefabs).unwrap();
    std::fs::write(ui.join("frame.png"), b"png").unwrap();
    std::fs::write(ui.join("frame.png.meta"), "fileFormatVersion: 2\nguid: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\nTextureImporter:\n  spriteMode: 1\n  spritePixelsToUnits: 64\n  spriteBorder: {x: 8, y: 6, z: 8, w: 4}\n").unwrap();
    std::fs::write(ui.join("icons.png"), b"png").unwrap();
    std::fs::write(
        ui.join("icons.png.meta"),
        "fileFormatVersion: 2\nguid: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\nTextureImporter:\n  internalIDToNameTable:\n  - first:\n      213: 21300000\n    second: icon_a\n  - first:\n      213: 21300002\n    second: icon_b\n  spriteMode: 2\n  spritePixelsToUnits: 32\n  spriteBorder: {x: 0, y: 0, z: 0, w: 0}\n  spriteSheet:\n    serializedVersion: 2\n    sprites:\n    - serializedVersion: 2\n      name: icon_a\n      border: {x: 0, y: 0, z: 0, w: 0}\n      internalID: 21300000\n    - serializedVersion: 2\n      name: icon_b\n      border: {x: 3, y: 2, z: 3, w: 1}\n      internalID: 21300002\n",
    )
    .unwrap();
    let text = prefab(&[
        go(1, "Root", &[11, 12], true),
        Rt::new(11, 1, 0).kids(&[21, 31, 41, 51]).size([1000.0, 800.0]).yaml(),
        native(223, 12, 1, "  m_RenderMode: 0\n"),
        go(2, "Frame", &[21, 22], true),
        Rt::new(21, 2, 11).size([300.0, 200.0]).yaml(),
        image(22, 2, [1.0, 1.0, 1.0, 1.0], Some((21300000, "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")), 1),
        go(3, "IconB", &[31, 32], true),
        Rt::new(31, 3, 11).size([64.0, 64.0]).yaml(),
        image(32, 3, [1.0, 1.0, 1.0, 0.5], Some((21300002, "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")), 0),
        go(4, "Gone", &[41, 42], true),
        Rt::new(41, 4, 11).size([64.0, 64.0]).yaml(),
        image(42, 4, [1.0, 1.0, 1.0, 1.0], Some((21300000, "cccccccccccccccccccccccccccccccc")), 0),
        go(5, "Builtin", &[51, 52], true),
        Rt::new(51, 5, 11).size([64.0, 64.0]).yaml(),
        image(52, 5, [1.0, 1.0, 1.0, 1.0], Some((10905, "0000000000000000f000000000000000")), 1),
    ]);
    let pf = prefabs.join("Sprites.prefab");
    std::fs::write(&pf, text).unwrap();
    let imp = import_file(&pf).unwrap();
    let d = &imp.doc;
    let f = by_name(d, "Frame");
    assert_eq!(f.kind, Kind::Image);
    assert_eq!(f.image, "unity:Assets/UI/frame.png");
    assert_eq!(f.slice, Some([8.0, 6.0, 8.0, 4.0]));
    assert_eq!(f.ppu, Some(64.0));
    assert_eq!(f.effective_image_mode(), crate::docext::ImageMode::Sliced);
    assert_eq!(f.sprite, None);
    let i = by_name(d, "IconB");
    assert_eq!(i.image, "unity:Assets/UI/icons.png");
    assert_eq!(i.sprite.as_deref(), Some("icon_b"));
    assert_eq!(i.slice, Some([3.0, 2.0, 3.0, 1.0]), "border of the sub-sprite");
    assert_eq!(i.ppu, Some(32.0));
    assert!(close(i.opacity, 0.5), "tint alpha becomes the opacity of a picture");
    let g = by_name(d, "Gone");
    assert!(g.image.is_empty() && g.kind == Kind::Panel);
    assert!(imp.warnings.iter().any(|w| w.contains("could not be resolved") && w.contains("cccccccc")), "{:?}", imp.warnings);
    assert!(imp.warnings.iter().any(|w| w.contains("built-in sprites")), "{:?}", imp.warnings);
    assert_eq!(by_name(d, "Builtin").effective_image_mode(), crate::docext::ImageMode::Sliced);
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Lienzo export JSON
// ---------------------------------------------------------------------------

fn parent_index(d: &Doc, e: &Element) -> i64 {
    e.parent.and_then(|p| d.index_of(p)).map(|i| i as i64).unwrap_or(-1)
}

fn assert_same_design(a: &Doc, b: &Doc, what: &str) {
    assert_eq!(a.elements.len(), b.elements.len(), "{what}: element count");
    assert!(close(a.width, b.width) && close(a.height, b.height), "{what}: page");
    for (x, y) in a.elements.iter().zip(&b.elements) {
        let tag = format!("{what} / {}", x.name);
        assert_eq!(x.kind, y.kind, "{tag}: kind");
        assert_eq!(parent_index(a, x), parent_index(b, y), "{tag}: parent");
        assert_eq!(x.visible, y.visible, "{tag}: visible");
        assert_eq!(x.layout.mode, y.layout.mode, "{tag}: layout");
        assert_eq!(x.text, y.text, "{tag}: text");
        assert_eq!(x.options, y.options, "{tag}: options");
        for (n, p, q) in [("x", x.x, y.x), ("y", x.y, y.y), ("w", x.w, y.w), ("h", x.h, y.h), ("rot", x.rot, y.rot)] {
            assert!((p - q).abs() <= 1.0, "{tag}: {n} {p} vs {q}");
        }
        assert_eq!(y.unity_name.as_deref(), Some(crate::export::base_unity_name(x).as_str()), "{tag}: unity_name");
    }
}

#[test]
fn every_template_round_trips_through_the_exported_json() {
    for i in 0..crate::model::TEMPLATES.len() {
        let d = template(i, 1920.0, 1080.0);
        let json = crate::export::ugui::json(&d, true);
        let imp = import_export_json_for_tests(&json);
        assert_same_design(&d, &imp.doc, &format!("template {}", i));
        // exporting what was imported gives the same elements again
        let again = crate::export::ugui::json(&imp.doc, true);
        let imp2 = import_export_json_for_tests(&again);
        assert_same_design(&imp.doc, &imp2.doc, &format!("template {} twice", i));
    }
}

fn import_export_json_for_tests(json: &str) -> Imported {
    let dir = tmpdir("json");
    let f = dir.join("Design.json");
    std::fs::write(&f, json).unwrap();
    let r = import_file(&f).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    r
}

#[test]
fn exported_json_keeps_canvas_unity_portrait_and_image_fields() {
    let mut d = template(3, 1920.0, 1080.0);
    d.canvas.render_mode = crate::docext::CanvasRenderMode::WorldSpace;
    d.canvas.sort_order = 12;
    d.canvas.safe_area = true;
    d.canvas_match = 0.75;
    d.unity.scripts_dir = Some("Assets/Game/UI".into());
    d.unity.default_font = Some("Mono SDF".into());
    d.background = [10, 20, 30, 255];
    let id = d.elements.iter().find(|e| e.kind == Kind::Panel).map(|e| e.id).unwrap();
    {
        let e = d.get_mut(id).unwrap();
        e.unity_name = Some("MainWindow".into());
        e.slice = Some([4.0, 5.0, 6.0, 7.0]);
        e.ppu = Some(48.0);
        e.sprite = Some("frame_a".into());
        e.image = "x.png".into();
    }
    let mut ov = crate::docext::ElementOverride::new(id);
    ov.x = Some(40.0);
    ov.visible = Some(false);
    assert!(d.set_override(ov));
    let imp = import_export_json_for_tests(&crate::export::ugui::json(&d, true));
    let r = &imp.doc;
    assert_eq!(r.canvas.render_mode, crate::docext::CanvasRenderMode::WorldSpace);
    assert_eq!(r.canvas.sort_order, 12);
    assert!(r.canvas.safe_area);
    assert!(close(r.canvas_match, 0.75));
    assert_eq!(r.unity.scripts_dir.as_deref(), Some("Assets/Game/UI"));
    assert_eq!(r.unity.default_font.as_deref(), Some("Mono SDF"));
    assert_eq!(r.background, [10, 20, 30, 255]);
    let e = r.elements.iter().find(|e| e.unity_name.as_deref() == Some("MainWindow")).unwrap();
    assert_eq!(e.slice, Some([4.0, 5.0, 6.0, 7.0]));
    assert_eq!(e.ppu, Some(48.0));
    assert_eq!(e.sprite.as_deref(), Some("frame_a"));
    assert_eq!(e.image, "unity:Assets/EditorGUI/Sprites/x.png");
    let ov = r.override_of(e.id).expect("portrait override survives");
    assert_eq!((ov.x, ov.visible), (Some(40.0), Some(false)));
}

#[test]
fn a_design_file_given_to_the_importer_is_loaded_as_is() {
    let d = template(0, 1920.0, 1080.0);
    let dir = tmpdir("passthrough");
    let f = dir.join("m.json");
    std::fs::write(&f, serde_json::to_string(&d).unwrap()).unwrap();
    let imp = import_file(&f).unwrap();
    assert_eq!(imp.doc.elements.len(), d.elements.len());
    assert!(imp.warnings.iter().any(|w| w.contains("already a Lienzo design")));
    std::fs::write(&f, "{\"hello\": 1}").unwrap();
    assert!(import_file(&f).is_err());
    std::fs::write(&f, "not json").unwrap();
    assert!(import_file(&f).is_err());
    assert!(import_file(&dir.join("a.txt")).is_err());
    assert!(import_file(&dir.join("missing.prefab")).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Project import
// ---------------------------------------------------------------------------

#[test]
fn file_names_are_sanitised() {
    assert_eq!(sanitize_file_name("Main Menu"), "Main Menu");
    assert_eq!(sanitize_file_name("a/b\\c:d*e?f\"g<h>i|j"), "a_b_c_d_e_f_g_h_i_j");
    assert_eq!(sanitize_file_name("CON"), "_CON");
    assert_eq!(sanitize_file_name("nul.txt"), "_nul.txt");
    assert_eq!(sanitize_file_name("COM1"), "_COM1");
    assert_eq!(sanitize_file_name("lpt9"), "_lpt9");
    assert_eq!(sanitize_file_name("COM0"), "COM0");
    let evil = sanitize_file_name("..\\..\\evil");
    assert!(!evil.contains('\\') && !evil.contains('/') && !evil.starts_with('.'), "{evil}");
    assert_eq!(sanitize_file_name("name. "), "name");
    assert_eq!(sanitize_file_name(""), "prefab");
    assert_eq!(sanitize_file_name("..."), "prefab");
    assert!(sanitize_file_name(&"x".repeat(500)).chars().count() <= 100);
}

#[test]
fn project_import_writes_only_inside_the_output_folder() {
    let dir = tmpdir("project");
    let proj = dir.join("proj");
    let out = dir.join("out");
    for d in ["Assets/UI", "Assets/Other/Deep", "Assets/Models"] {
        std::fs::create_dir_all(proj.join(d)).unwrap();
    }
    let ui = menu_prefab();
    std::fs::write(proj.join("Assets/UI/Menu.prefab"), &ui).unwrap();
    std::fs::write(proj.join("Assets/Other/Deep/menu.prefab"), &ui).unwrap(); // same name, other case
    std::fs::write(proj.join("Assets/UI/CON.prefab"), &ui).unwrap(); // reserved Windows name
    std::fs::write(proj.join("Assets/UI/Evil Name.prefab"), &ui).unwrap();
    // not UI: a GameObject with a plain Transform
    let model = prefab(&[go(1, "Crate", &[2], true), "--- !u!4 &2\nTransform:\n  m_GameObject: {fileID: 1}\n  m_Children: []\n  m_Father: {fileID: 0}\n".to_string()]);
    std::fs::write(proj.join("Assets/Models/Crate.prefab"), model).unwrap();
    std::fs::write(proj.join("Assets/Models/Broken.prefab"), "garbage \n--- !u!224 &x\n").unwrap();
    let rep = import_unity_project_report(&proj, &out).unwrap();
    let names: Vec<&str> = rep.written.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(rep.written.len(), 4, "{:?}", names);
    assert!(names.iter().all(|n| n.ends_with(".editorgui.json")));
    assert!(names.contains(&"_CON.editorgui.json"), "{:?}", names);
    let lower: std::collections::HashSet<String> = names.iter().map(|n| n.to_lowercase()).collect();
    assert_eq!(lower.len(), 4, "names are unique even ignoring case: {:?}", names);
    assert!(rep.written.iter().all(|(_, n)| *n == 2));
    assert!(rep.skipped.iter().any(|s| s.ends_with("Crate.prefab")));
    // everything that was written is inside `out`, nothing next to it
    let mut seen = 0;
    for e in std::fs::read_dir(&out).unwrap() {
        let e = e.unwrap();
        assert!(names.contains(&e.file_name().to_string_lossy().as_ref()));
        let d = Doc::from_json_str(&std::fs::read_to_string(e.path()).unwrap()).unwrap();
        assert_eq!(d.elements.len(), 2);
        seen += 1;
    }
    assert_eq!(seen, 4);
    let top: Vec<String> = std::fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().to_string()).collect();
    assert_eq!(top.len(), 2, "only proj and out exist: {:?}", top);
    // the convenience wrapper
    let v = import_unity_project(&proj, &dir.join("out2")).unwrap();
    assert_eq!(v.len(), 4);
    assert!(import_unity_project(&dir.join("nowhere"), &out).is_err());
    assert!(import_unity_project(&out, &out).is_err(), "no Assets folder");
    let _ = std::fs::remove_dir_all(&dir);
}

// ---------------------------------------------------------------------------
// Hostile inputs: never panic, never hang
// ---------------------------------------------------------------------------

fn fast<T>(label: &str, f: impl FnOnce() -> T) -> T {
    let t = Instant::now();
    let r = f();
    assert!(t.elapsed() < Duration::from_secs(20), "{label} took {:?}", t.elapsed());
    r
}

#[test]
fn cycles_missing_ids_and_duplicates_do_not_break_the_import() {
    // m_Father cycle with no root
    let cyc = prefab(&[
        go(1, "A", &[11], true),
        Rt::new(11, 1, 12).kids(&[12]).yaml(),
        go(2, "B", &[12], true),
        Rt::new(12, 2, 11).kids(&[11]).yaml(),
    ]);
    let mut a = Assets::none();
    assert!(import_prefab_text(&cyc, "x", &mut a).is_err(), "no root: a clean error");
    // a cycle below a real root: the cyclic part is simply not repeated
    let below = prefab(&[
        go(1, "Root", &[11, 12], true),
        Rt::new(11, 1, 0).kids(&[21]).size([100.0, 100.0]).yaml(),
        native(223, 12, 1, ""),
        go(2, "C1", &[21], true),
        Rt::new(21, 2, 11).kids(&[31, 21, 11]).yaml(),
        go(3, "C2", &[31], true),
        Rt::new(31, 3, 21).kids(&[21, 31]).yaml(),
    ]);
    let imp = import(&below);
    assert_eq!(imp.doc.elements.len(), 2);
    // a child that points at itself, children that do not exist, components that do not exist
    let broken = prefab(&[
        go(1, "Root", &[11, 99, 12], true),
        Rt::new(11, 1, 0).kids(&[11, 555, 21]).size([100.0, 100.0]).yaml(),
        native(223, 12, 1, ""),
        go(2, "Kid", &[21, 77], true),
        Rt::new(21, 2, 11).yaml(),
    ]);
    assert_eq!(import(&broken).doc.elements.len(), 1);
    // duplicate fileIDs: the first one wins
    let dup = prefab(&[
        go(1, "Root", &[11, 12], true),
        Rt::new(11, 1, 0).kids(&[21]).size([100.0, 100.0]).yaml(),
        native(223, 12, 1, ""),
        go(2, "First", &[21], true),
        Rt::new(21, 2, 11).yaml(),
        go(2, "Second", &[21], true),
    ]);
    let imp = import(&dup);
    assert_eq!(imp.doc.elements.len(), 1);
    assert_eq!(imp.doc.elements[0].name, "First");
    assert!(imp.warnings.iter().any(|w| w.contains("duplicate")));
    // only prefab instances
    let variant = format!("{}--- !u!1001 &1\nPrefabInstance:\n  m_Modification: {{}}\n", HEADER);
    assert!(import_prefab_text(&variant, "x", &mut Assets::none()).is_err());
}

#[test]
fn huge_and_non_finite_numbers_are_tamed() {
    let p = prefab(&[
        go(1, "Root", &[11, 12], true),
        Rt::new(11, 1, 0).kids(&[21, 31]).size([1000.0, 800.0]).yaml(),
        native(223, 12, 1, ""),
        go(2, "Huge", &[21, 22], true),
        "--- !u!224 &21\nRectTransform:\n  m_GameObject: {fileID: 2}\n  m_LocalRotation: {x: 1e300, y: nan, z: inf, w: -inf}\n  m_LocalScale: {x: 1e38, y: -5, z: 1}\n  m_Children: []\n  m_Father: {fileID: 11}\n  m_AnchorMin: {x: -1e30, y: 1e30}\n  m_AnchorMax: {x: 5e300, y: .nan}\n  m_AnchoredPosition: {x: 1e308, y: -1e308}\n  m_SizeDelta: {x: 1e39, y: Infinity}\n  m_Pivot: {x: 1e20, y: -1e20}\n".to_string(),
        image(22, 2, [1e30, -1e30, f32::NAN, 5.0], None, 0),
        go(3, "Text", &[31, 32], true),
        Rt::new(31, 3, 11).yaml(),
        tmp(32, 3, "x", 1e30, [1.0, 1.0, 1.0, 1.0], "  m_fontStyle: 99999999999999999999\n"),
    ]);
    let imp = fast("huge numbers", || import(&p));
    for e in &imp.doc.elements {
        for v in [e.x, e.y, e.w, e.h, e.rot, e.font_size, e.opacity] {
            assert!(v.is_finite(), "{}: {:?}", e.name, (e.x, e.y, e.w, e.h, e.rot, e.font_size));
        }
    }
    // the repaired document serialises and loads back
    let json = serde_json::to_string(&imp.doc).unwrap();
    assert!(Doc::from_json_str(&json).is_ok());
}

#[test]
fn truncated_garbage_and_random_bytes_never_panic() {
    let full = menu_prefab();
    for cut in (0..full.len()).step_by(37) {
        let Some(t) = full.get(..cut) else { continue };
        let _ = import_prefab_text(t, "cut", &mut Assets::none());
    }
    // random-ish bytes (deterministic)
    let mut x = 0x2545F4914F6CDD1Du64;
    for round in 0..40 {
        let mut s = String::new();
        for _ in 0..2000 {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            let c = match x % 9 {
                0 => '\n',
                1 => ' ',
                2 => ':',
                3 => '-',
                4 => '{',
                5 => '"',
                6 => '[',
                7 => '\u{e9}',
                _ => (b'a' + (x % 26) as u8) as char,
            };
            s.push(c);
        }
        let with_header = format!("{}--- !u!{} &{}\nGameObject:\n{}", HEADER, 1 + round % 3, round, s);
        let _ = import_prefab_text(&with_header, "r", &mut Assets::none());
        let _ = parse_plain(&s);
    }
    for t in ["", "---", "--- !u!1 &1", "--- !u!1 &1\n", "\u{feff}", "%YAML 1.1\n", "--- !u!224 &5 stripped\nRectTransform:\n"] {
        let _ = import_prefab_text(t, "e", &mut Assets::none());
    }
}

#[test]
fn deep_nesting_in_yaml_and_in_the_hierarchy_is_bounded() {
    // flow nesting 100 000 deep
    let deep = format!("{}--- !u!1 &1\nGameObject:\n  m_Name: {}{}\n", HEADER, "[".repeat(100_000), "]".repeat(100_000));
    fast("deep flow", || {
        let _ = import_prefab_text(&deep, "d", &mut Assets::none());
    });
    // block nesting
    let mut blk = String::from("A:\n");
    for i in 1..5000 {
        blk += &format!("{}k{}:\n", "  ".repeat(i.min(400)), i);
    }
    fast("deep block", || {
        let _ = parse_plain(&blk);
    });
    // a hierarchy 400 levels deep: the part below the limit is dropped with a warning
    let mut parts = vec![go(1, "Root", &[11, 12], true), native(223, 12, 1, "")];
    let n = 400i64;
    parts.push(Rt::new(11, 1, 0).kids(&[100]).size([500.0, 500.0]).yaml());
    for i in 0..n {
        let id = 100 + i;
        let kids = if i + 1 < n { vec![id + 1] } else { vec![] };
        parts.push(go(1000 + i, &format!("L{i}"), &[id], true));
        parts.push(Rt::new(id, 1000 + i, if i == 0 { 11 } else { id - 1 }).kids(&kids).size([10.0, 10.0]).yaml());
    }
    let imp = fast("deep hierarchy", || import(&prefab(&parts)));
    assert!(imp.doc.elements.len() < 200 && imp.warnings.iter().any(|w| w.contains("deeper")), "{} {:?}", imp.doc.elements.len(), imp.warnings);
}

#[test]
fn a_flat_hierarchy_with_thousands_of_children_is_capped() {
    let n = 7000i64;
    let kids: Vec<i64> = (0..n).map(|i| 100 + i).collect();
    let mut parts = vec![go(1, "Root", &[11, 12], true), native(223, 12, 1, ""), Rt::new(11, 1, 0).kids(&kids).size([500.0, 500.0]).yaml()];
    for i in 0..n {
        parts.push(go(10_000 + i, &format!("E{i}"), &[100 + i], true));
        parts.push(Rt::new(100 + i, 10_000 + i, 11).size([10.0, 10.0]).yaml());
    }
    let imp = fast("7000 children", || import(&prefab(&parts)));
    assert_eq!(imp.doc.elements.len(), 5000);
    assert!(imp.warnings.iter().any(|w| w.contains("More than")));
}

#[test]
fn oversized_files_and_too_many_documents_are_refused() {
    let dir = tmpdir("big");
    // 50 MB: refused from the file size, without reading it
    let f = dir.join("huge.prefab");
    let file = std::fs::File::create(&f).unwrap();
    file.set_len(50 * 1024 * 1024).unwrap();
    drop(file);
    let r = fast("50 MB", || import_file(&f));
    assert!(r.is_err() && r.err().unwrap().contains("too large"));
    let j = dir.join("huge.json");
    std::fs::File::create(&j).unwrap().set_len(50 * 1024 * 1024).unwrap();
    assert!(import_file(&j).is_err());
    // more documents than allowed
    let mut many = String::from(HEADER);
    for i in 0..100_050 {
        many += &format!("--- !u!114 &{i}\n");
    }
    let r = fast("many docs", || import_prefab_text(&many, "m", &mut Assets::none()));
    assert!(r.is_err());
    // a large (but allowed) file of junk is processed in bounded time
    let junk = format!("{}--- !u!1 &1\nGameObject:\n  m_Name: {}\n", HEADER, "x".repeat(10 * 1024 * 1024));
    let _ = fast("10 MB line", || import_prefab_text(&junk, "j", &mut Assets::none()));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn hostile_export_json_does_not_panic() {
    for t in [
        "{\"format\":\"editor-gui/ugui/2\"}",
        "{\"format\":\"editor-gui/ugui/2\",\"elements\":[null,1,\"x\",[],{\"parent\":99999999999,\"size\":[1e400,2],\"rect\":{\"anchorMin\":[\"a\"],\"rotationZ\":1e300}}]}",
        "{\"format\":\"editor-gui/ugui/2\",\"elements\":[{\"id\":18446744073709551615,\"parent\":0,\"type\":\"Nope\",\"name\":\"a/b\"},{\"id\":18446744073709551615,\"parent\":-5}],\"canvas\":{\"referenceResolution\":[0,-4],\"background\":\"#zz\"},\"portrait\":{\"overrides\":[{\"id\":1,\"x\":1e999}]}}",
    ] {
        let dir = tmpdir("hjson");
        let f = dir.join("h.json");
        std::fs::write(&f, t).unwrap();
        let r = import_file(&f);
        if let Ok(i) = r {
            assert!(Doc::from_json_str(&serde_json::to_string(&i.doc).unwrap()).is_ok());
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}

// ---------------------------------------------------------------------------
// Security audit regressions
// ---------------------------------------------------------------------------

#[test]
fn unclosed_quote_and_flow_do_not_rescan_quadratically() {
    let mut q = String::from("k: \"never closed\n");
    for _ in 0..200_000 {
        q += "  more text here\n";
    }
    let t = Instant::now();
    let _ = parse_plain(&q);
    assert!(t.elapsed() < Duration::from_secs(2), "unclosed quote took {:?}", t.elapsed());
    let mut f = String::from("k: {\n");
    for _ in 0..300_000 {
        f += "{\n";
    }
    let t = Instant::now();
    let _ = parse_plain(&f);
    assert!(t.elapsed() < Duration::from_secs(2), "unclosed flow took {:?}", t.elapsed());
    // a legitimate multi-line quoted scalar and flow still work
    let y = parse_plain("a: \"one\n  two\"\nb: {x: 1,\n  y: 2}\nc: after\n");
    assert_eq!(y.get("a").and_then(|v| v.s()), Some("one two"));
    assert_eq!(y.get("b").and_then(|v| v.get("y")).and_then(|v| v.s()), Some("2"));
    assert_eq!(y.get("c").and_then(|v| v.s()), Some("after"));
}

#[test]
fn overlong_scalar_is_cut_with_a_warning() {
    let big = "x".repeat(200_000);
    let p = prefab(&[
        go(1, "Root", &[11, 12], true),
        Rt::new(11, 1, 0).kids(&[21]).size([1000.0, 800.0]).yaml(),
        native(223, 12, 1, "  m_RenderMode: 0\n"),
        go(2, "T", &[21, 22], true),
        Rt::new(21, 2, 11).yaml(),
        tmp(22, 2, &format!("\"{big}"), 20.0, [1.0, 1.0, 1.0, 1.0], ""),
    ]);
    let imp = import(&p);
    assert!(imp.warnings.iter().any(|w| w.contains("cut")), "{:?}", imp.warnings);
}

#[test]
fn text_is_capped_per_element_and_per_import() {
    let shared = "y".repeat(60_000);
    let n = 5000i64;
    let kids: Vec<i64> = (0..n).map(|i| 1000 + i * 10).collect();
    let mut parts = vec![go(1, "Root", &[11, 12], true), Rt::new(11, 1, 0).kids(&kids).size([1000.0, 800.0]).yaml(), native(223, 12, 1, "  m_RenderMode: 0\n")];
    // one shared placeholder text of 60 KB
    parts.push(go(2, "Shared", &[21, 22], true));
    parts.push(Rt::new(21, 2, 11).yaml());
    parts.push(tmp(22, 2, &shared, 20.0, [1.0, 1.0, 1.0, 1.0], ""));
    for i in 0..n {
        let b = 1000 + i * 10;
        parts.push(go(b, &format!("I{i}"), &[b + 1, b + 2], true));
        parts.push(Rt::new(b + 1, b, 11).yaml());
        parts.push(mb(b + 2, b, "d199490a83bb2b844b9695cbf13b01ef", "", &format!("{}  m_Text: \n  m_TextComponent: {{fileID: 21}}\n  m_Placeholder: {{fileID: 21}}\n", selectable_fields(0, ""))));
    }
    let p = prefab(&parts);
    let t = Instant::now();
    let imp = import(&p);
    assert!(t.elapsed() < Duration::from_secs(10), "{:?}", t.elapsed());
    let json = serde_json::to_string(&imp.doc).unwrap();
    assert!(json.len() < 20 * 1024 * 1024, "output {} bytes", json.len());
    assert!(imp.doc.elements.iter().all(|e| e.text.len() <= 4096));
    assert!(imp.warnings.iter().any(|w| w.contains("cut or dropped")), "{:?}", imp.warnings);
}

#[test]
fn absurd_component_lists_are_bounded() {
    let rts = 400i64;
    let kids: Vec<i64> = (0..rts).map(|i| 100_000 + i * 10 + 1).collect();
    let mut parts = vec![go(1, "Root", &[11, 12], true), Rt::new(11, 1, 0).kids(&kids).size([1000.0, 800.0]).yaml(), native(223, 12, 1, "  m_RenderMode: 0\n")];
    for i in 0..rts {
        let b = 100_000 + i * 10;
        // 600 component references per GameObject (cap is 256) (all pointing at the same real component)
        let comps: Vec<i64> = std::iter::repeat(b + 1).take(600).collect();
        parts.push(go(b, &format!("N{i}"), &comps, true));
        parts.push(Rt::new(b + 1, b, 11).yaml());
    }
    let p = prefab(&parts);
    let t = Instant::now();
    let _ = import(&p);
    assert!(t.elapsed() < Duration::from_secs(2), "{:?}", t.elapsed());
}

#[test]
fn project_import_never_overwrites_existing_files() {
    let dir = tmpdir("noover");
    let proj = dir.join("proj");
    let out = dir.join("out");
    std::fs::create_dir_all(proj.join("Assets/UI")).unwrap();
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(proj.join("Assets/UI/Menu.prefab"), menu_prefab()).unwrap();
    std::fs::write(out.join("Menu.editorgui.json"), "precious").unwrap();
    let rep = import_unity_project_report(&proj, &out).unwrap();
    assert_eq!(std::fs::read_to_string(out.join("Menu.editorgui.json")).unwrap(), "precious");
    assert_eq!(rep.written.len(), 1);
    assert_eq!(rep.written[0].0, "Menu_2.editorgui.json");
    assert!(rep.warnings.iter().any(|(_, w)| w.contains("already existed")), "{:?}", rep.warnings);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn json_image_sources_are_restricted() {
    let mk = |src: &str| {
        let j = serde_json::json!({"format":"editor-gui/ugui/2","elements":[{"id":1,"type":"Image","name":"I","image":{"source":src}}]}).to_string();
        import_export_json(&j, "t").unwrap()
    };
    assert_eq!(mk("Assets/UI/a.png").doc.elements[0].image, "unity:Assets/UI/a.png");
    assert_eq!(mk("Packages/x/a.png").doc.elements[0].image, "unity:Packages/x/a.png");
    assert_eq!(mk("sword.png").doc.elements[0].image, "sword.png");
    for bad in ["C:\\Windows\\win.ini", "/etc/passwd", "\\\\server\\share\\a.png", "//server/share/a.png", "../x.png", "Assets/../../x.png", "a/b.png", "C:foo.png", "AssetsEvil/a.png", "a.png:s"] {
        let r = mk(bad);
        assert_eq!(r.doc.elements[0].image, "", "{bad}");
        assert!(r.warnings.iter().any(|w| w.contains("dropped")), "{bad}: {:?}", r.warnings);
    }
}

#[test]
fn warnings_are_clean_and_mirroring_and_name_absorption_are_reported() {
    let p = prefab(&[
        go(1, "Root", &[11, 12], true),
        Rt::new(11, 1, 0).kids(&[21, 31]).size([1000.0, 800.0]).yaml(),
        native(223, 12, 1, "  m_RenderMode: 0\n"),
        go(2, "\"Evil\\x1b[31mred\\x07\"", &[21, 22], true),
        Rt::new(21, 2, 11).scale(-1.0).yaml(),
        image(22, 2, [1.0, 1.0, 1.0, 1.0], None, 0),
        go(3, "Cap", &[31, 32], true),
        Rt::new(31, 3, 11).kids(&[41]).yaml(),
        image(32, 3, [1.0, 1.0, 1.0, 1.0], None, 0),
        go(4, "Texto", &[41, 42], true),
        Rt::new(41, 4, 31).stretch().yaml(),
        tmp(42, 4, "Hello", 20.0, [0.0, 0.0, 0.0, 1.0], ""),
    ]);
    let imp = import(&p);
    for w in &imp.warnings {
        assert!(!w.chars().any(|c| c.is_control()), "{w:?}");
    }
    assert!(imp.warnings.iter().any(|w| w.contains("mirrored")), "{:?}", imp.warnings);
    assert!(imp.warnings.iter().any(|w| w.contains("'Texto' absorbed")), "{:?}", imp.warnings);
    assert!(imp.warnings.iter().any(|w| w.contains("Evil")), "the name is still reported: {:?}", imp.warnings);
}

#[test]
fn scrollview_content_keeps_sibling_order() {
    // m_Children of the root: Before, Items (scroll view), After
    let p = prefab(&[
        go(1, "Root", &[11, 12], true),
        Rt::new(11, 1, 0).kids(&[21, 41, 31]).size([1000.0, 800.0]).yaml(),
        native(223, 12, 1, "  m_RenderMode: 0\n"),
        go(2, "Before", &[21, 22], true),
        Rt::new(21, 2, 11).yaml(),
        image(22, 2, [1.0, 1.0, 1.0, 1.0], None, 0),
        go(3, "After", &[31, 32], true),
        Rt::new(31, 3, 11).yaml(),
        image(32, 3, [1.0, 1.0, 1.0, 1.0], None, 0),
        go(210, "Items", &[41, 42, 43], true),
        Rt::new(41, 210, 11).kids(&[51]).size([400.0, 300.0]).yaml(),
        image(42, 210, [0.5, 0.5, 0.5, 1.0], None, 0),
        mb(43, 210, G_SCROLL, "UnityEngine.UI::UnityEngine.UI.ScrollRect", "  m_Content: {fileID: 61}\n  m_Viewport: {fileID: 51}\n"),
        go(211, "Viewport", &[51], true),
        Rt::new(51, 211, 41).kids(&[61]).stretch().yaml(),
        go(212, "Content", &[61], true),
        Rt::new(61, 212, 51).kids(&[71]).anchors([0.0, 1.0], [1.0, 1.0]).pivot([0.0, 1.0]).size([0.0, 500.0]).yaml(),
        go(213, "Item", &[71, 72], true),
        Rt::new(71, 213, 61).size([100.0, 40.0]).yaml(),
        image(72, 213, [1.0, 0.0, 1.0, 1.0], None, 0),
    ]);
    let imp = import(&p);
    let names: Vec<&str> = imp.doc.elements.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, ["Before", "Items", "Item", "After"], "{names:?}");
}

// ---- Lienzo-generated helper names, new form (marked block: the old form is covered by the test above)
#[test]
fn lienzo_exported_structure_is_recognised_with_the_new_generated_names() {
    // (names of exports from v1.5 on) "__canvas_background" background, "__border" ring, "__text" caption and generated sprites (rr_ / ellipse_) of Lienzo's own prefabs
    let dir = tmpdir("lienzo_struct_new");
    let sprites = dir.join("Assets/EditorGUI/Sprites/Generated");
    std::fs::create_dir_all(&sprites).unwrap();
    for (n, g) in [("rr_24_0", "11111111111111111111111111111111"), ("rr_24_3", "22222222222222222222222222222222"), ("ellipse_0", "33333333333333333333333333333333")] {
        std::fs::write(sprites.join(format!("{n}.png")), b"png").unwrap();
        std::fs::write(sprites.join(format!("{n}.png.meta")), format!("fileFormatVersion: 2\nguid: {g}\nTextureImporter:\n  spriteMode: 1\n  spriteBorder: {{x: 26, y: 26, z: 26, w: 26}}\n")).unwrap();
    }
    let pref_dir = dir.join("Assets/EditorGUI/Prefabs");
    std::fs::create_dir_all(&pref_dir).unwrap();
    let text = prefab(&[
        go(1, "Menu", &[11, 12, 13], true),
        Rt::new(11, 1, 0).kids(&[21, 31, 51]).stretch().yaml(),
        native(223, 12, 1, "  m_RenderMode: 0\n"),
        mb(13, 1, G_SCALER, "", "  m_UiScaleMode: 1\n  m_ReferenceResolution: {x: 1920, y: 1080}\n  m_ScreenMatchMode: 0\n  m_MatchWidthOrHeight: 0.5\n"),
        go(2, "__canvas_background", &[21, 22], true),
        Rt::new(21, 2, 11).stretch().yaml(),
        image(22, 2, [0.1, 0.2, 0.3, 1.0], None, 0),
        go(3, "Card", &[31, 32], true),
        Rt::new(31, 3, 11).kids(&[41, 42]).size([500.0, 300.0]).yaml(),
        image(32, 3, [1.0, 1.0, 1.0, 1.0], Some((21300000, "11111111111111111111111111111111")), 1),
        go(4, "__border", &[41, 43], true),
        Rt::new(41, 4, 31).stretch().yaml(),
        image(43, 4, [0.0, 0.0, 0.0, 1.0], Some((21300000, "22222222222222222222222222222222")), 1),
        go(5, "__text", &[42, 44], true),
        Rt::new(42, 5, 31).stretch().yaml(),
        tmp(44, 5, "Hello", 28.0, [0.0, 0.0, 0.0, 1.0], ""),
        go(6, "Dot", &[51, 52], true),
        Rt::new(51, 6, 11).size([80.0, 80.0]).pos([300.0, 0.0]).yaml(),
        image(52, 6, [1.0, 0.0, 0.0, 1.0], Some((21300000, "33333333333333333333333333333333")), 0),
    ]);
    let pf = pref_dir.join("Menu.prefab");
    std::fs::write(&pf, text).unwrap();
    let imp = import_file(&pf).unwrap();
    let d = &imp.doc;
    assert_eq!(d.background, [26, 51, 77, 255], "the 'Fondo del lienzo' child is the page background: {:?}", d.background);
    assert_eq!(d.elements.len(), 2, "{:?}", d.elements.iter().map(|e| &e.name).collect::<Vec<_>>());
    let card = by_name(d, "Card");
    assert_eq!(card.kind, Kind::Panel, "a generated rounded sprite is a plain panel");
    assert_eq!(card.radius, 24.0);
    assert_eq!(card.stroke_w, 3.0, "ring sprite rr_24_3 gives the border width");
    assert_eq!(card.stroke, [0, 0, 0, 255]);
    assert_eq!(card.text, "Hello");
    assert!(card.image.is_empty() && card.slice.is_none());
    assert_eq!(by_name(d, "Dot").kind, Kind::Ellipse);
    let _ = std::fs::remove_dir_all(&dir);
}
