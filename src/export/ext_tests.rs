//! Golden-ish tests of the Unity export added for transparency, sprites, scripts folder / assembly, canvas options,
//! fonts, first generation, names and the portrait layout.

use super::*;
use crate::docext::{CanvasRenderMode, ElementOverride, GenerateOutputs, ImageMode};
use crate::model::{self, Doc, Element, Kind};
use serde_json::Value;

static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn project(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lienzo-export-{}-{}-{}", tag, std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::SeqCst)));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("Assets")).unwrap();
    std::fs::create_dir_all(dir.join("ProjectSettings")).unwrap();
    dir
}

fn doc_with_button() -> Doc {
    let mut d = Doc::empty(1920.0, 1080.0);
    d.title = "Menu".into();
    let mut b = Element::new(0, Kind::Button);
    b.name = "Play".into();
    b.text = "Play".into();
    b.x = 100.0;
    b.y = 100.0;
    d.add(b);
    d
}

fn json_of(d: &Doc) -> Value {
    serde_json::from_str(&ugui::json(d, true)).unwrap()
}

#[test]
fn transparent_background_emits_no_background_anywhere() {
    let mut d = doc_with_button();
    d.background = [10, 20, 30, 255];
    assert!(d.has_background());
    assert_eq!(json_of(&d)["canvas"]["background"], "#0A141EFF");
    assert!(uitk::uss(&d).contains("background-color: rgba(10, 20, 30, 1)"));
    assert!(imgui::script(&d).contains("drawBackground) Box(root"));
    d.background = [10, 20, 30, 0];
    assert!(!d.has_background());
    assert!(json_of(&d)["canvas"].get("background").is_none(), "no canvas background in the JSON");
    let uss = uitk::uss(&d);
    let root = &uss[..uss.find("}\n").unwrap()];
    assert!(!root.contains("background-color"), "{root}");
    assert!(!imgui::script(&d).contains("drawBackground) Box(root"));
    // the importer creates the background only when the file carries one
    assert!(IMPORTER_CS.contains("if (!string.IsNullOrEmpty(cv.background))"));
}

#[test]
fn unity_image_references_are_not_copied() {
    assert_eq!(unity_image_ref("unity:Assets/Art/ui.png#frame"), Some(("Assets/Art/ui.png".into(), Some("frame".into()))));
    assert_eq!(unity_image_ref("Assets/Art/ui.png"), Some(("Assets/Art/ui.png".into(), None)));
    assert_eq!(unity_image_ref("unity:Assets/../x.png"), None);
    assert_eq!(unity_image_ref("sword.png"), None);
    assert_eq!(unity_image_ref("C:/pics/Assets/a.png"), None);
    let mut d = Doc::empty(1920.0, 1080.0);
    d.title = "Sheet".into();
    let mut e = Element::new(0, Kind::Image);
    e.name = "Icon".into();
    e.image = "unity:Assets/Art/ui.png#frame".into();
    e.slice = Some([8.0, 8.0, 8.0, 8.0]);
    e.ppu = Some(32.0);
    e.preserve_aspect = true;
    d.add(e);
    let v = json_of(&d);
    let el = &v["elements"][0];
    assert_eq!(el["image"]["source"], "Assets/Art/ui.png");
    assert_eq!(el["sprite"], "frame");
    assert_eq!(el["imageMode"], "Sliced");
    assert_eq!(el["slice"], serde_json::json!([8.0, 8.0, 8.0, 8.0]));
    assert_eq!(el["ppu"], 32.0);
    assert_eq!(el["image"]["preserveAspect"], true);
    // exporting does not need (nor create) a copy of the picture
    let dir = project("unityimg");
    let files = write_unity_package(&d, &dir).unwrap();
    assert!(!files.iter().any(|f| f.contains("/Sprites/")), "{files:?}");
    assert!(!dir.join("Assets/EditorGUI/Sprites").exists());
    let uss = uitk::uss(&d);
    assert!(uss.contains("project://database/Assets/Art/ui.png"), "{uss}");
    assert!(uss.contains("-unity-slice-left: 8") && uss.contains("-unity-slice-top: 8"), "{uss}");
    assert!(uss.contains("-unity-background-scale-mode: scale-to-fit"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn uss_slice_only_when_sliced() {
    let mut d = Doc::empty(1920.0, 1080.0);
    let mut e = Element::new(0, Kind::Image);
    e.image = "Assets/Art/frame.png".into();
    e.slice = Some([4.0, 5.0, 6.0, 7.0]);
    e.image_mode = Some(ImageMode::Simple);
    d.add(e);
    let uss = uitk::uss(&d);
    assert!(!uss.contains("-unity-slice"), "an explicit Simple mode has no slice: {uss}");
    d.elements[0].image_mode = None;
    let uss = uitk::uss(&d);
    assert!(uss.contains("-unity-slice-left: 4") && uss.contains("-unity-slice-bottom: 5") && uss.contains("-unity-slice-right: 6") && uss.contains("-unity-slice-top: 7"));
    assert!(uss.contains("stretch-to-fill"));
}

#[test]
fn generate_flags_select_the_outputs() {
    // legacy file (no unity block): everything including IMGUI, as before
    let mut d = doc_with_button();
    d.unity = crate::docext::UnitySettings::default();
    let dir = project("legacy");
    let files = write_unity_package(&d, &dir).unwrap();
    for f in ["Editor/EditorGuiImporter.cs", "Designs/Menu.json", "UI Toolkit/Menu.uxml", "Scripts/MenuOnGUI.cs", "Scripts/MenuViewBase.cs", "Scripts/MenuView.cs"] {
        assert!(files.iter().any(|x| x.ends_with(f)), "{f} missing in {files:?}");
    }
    let _ = std::fs::remove_dir_all(&dir);
    // new design: IMGUI off, nothing else disabled
    let mut d = doc_with_button();
    d.unity.generate = GenerateOutputs::new_design();
    let dir = project("newdesign");
    let files = write_unity_package(&d, &dir).unwrap();
    assert!(!files.iter().any(|x| x.contains("OnGUI")), "{files:?}");
    assert!(files.iter().any(|x| x.ends_with("Scripts/MenuViewBase.cs")));
    let _ = std::fs::remove_dir_all(&dir);
    // only UI Toolkit
    let mut d = doc_with_button();
    d.unity.generate = GenerateOutputs { prefab: false, uitk: true, imgui: Some(false), view_scripts: false };
    let dir = project("uitk");
    let files = write_unity_package(&d, &dir).unwrap();
    assert!(files.iter().all(|x| x.contains("UI Toolkit/")), "{files:?}");
    let _ = std::fs::remove_dir_all(&dir);
    // only the IMGUI script
    let mut d = doc_with_button();
    d.unity.generate = GenerateOutputs { prefab: false, uitk: false, imgui: Some(true), view_scripts: false };
    let dir = project("imgui");
    let files = write_unity_package(&d, &dir).unwrap();
    assert_eq!(files, vec!["Assets/EditorGUI/Scripts/MenuOnGUI.cs".to_string()]);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn scripts_dir_and_assembly_are_honored() {
    let dir = project("asm");
    std::fs::create_dir_all(dir.join("Assets/Game/Runtime")).unwrap();
    std::fs::write(dir.join("Assets/Game/Runtime/Game.Runtime.asmdef"), r#"{ "name": "Game.Runtime", "references": [] }"#).unwrap();
    let mut d = doc_with_button();
    d.unity.generate = GenerateOutputs::new_design();
    // scripts_dir only
    d.unity.scripts_dir = Some("Assets/Game/UI".into());
    assert_eq!(resolve_scripts_dir(&d, &dir), ("Assets/Game/UI".to_string(), None));
    // assembly found, no scripts_dir: under the asmdef folder
    d.unity.scripts_dir = None;
    d.unity.assembly = Some("Game.Runtime".into());
    assert_eq!(resolve_scripts_dir(&d, &dir).0, "Assets/Game/Runtime/LienzoUI");
    // scripts_dir inside the asmdef folder is kept
    d.unity.scripts_dir = Some("Assets/Game/Runtime/Menus".into());
    assert_eq!(resolve_scripts_dir(&d, &dir), ("Assets/Game/Runtime/Menus".to_string(), None));
    // scripts_dir outside: moved into the assembly, with a note
    d.unity.scripts_dir = Some("Assets/Other".into());
    let (p, note) = resolve_scripts_dir(&d, &dir);
    assert_eq!(p, "Assets/Game/Runtime/LienzoUI");
    assert!(note.unwrap().contains("outside"));
    // assembly that does not exist: scripts stay in scripts_dir (Assembly-CSharp), with a note
    d.unity.assembly = Some("Nope".into());
    let (p, note) = resolve_scripts_dir(&d, &dir);
    assert_eq!(p, "Assets/Other");
    assert!(note.unwrap().contains("not found"));
    // an .asmref also counts
    std::fs::create_dir_all(dir.join("Assets/Extra")).unwrap();
    std::fs::write(dir.join("Assets/Extra/Ref.asmref"), r#"{ "reference": "Nope" }"#).unwrap();
    assert_eq!(resolve_scripts_dir(&d, &dir).0, "Assets/Extra/LienzoUI");
    // the export writes there and a ViewBase left in the default folder is removed (it would clash)
    d.unity.assembly = Some("Game.Runtime".into());
    d.unity.scripts_dir = None;
    std::fs::create_dir_all(dir.join("Assets/EditorGUI/Scripts")).unwrap();
    std::fs::write(dir.join("Assets/EditorGUI/Scripts/MenuViewBase.cs"), "// Generated by Lienzo UI\npublic abstract class MenuViewBase {}").unwrap();
    let files = write_unity_package(&d, &dir).unwrap();
    assert!(files.iter().any(|f| f == "Assets/Game/Runtime/LienzoUI/MenuViewBase.cs"), "{files:?}");
    assert!(!dir.join("Assets/EditorGUI/Scripts/MenuViewBase.cs").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn view_base_binds_by_serialized_reference_with_name_fallback() {
    let mut d = doc_with_button();
    d.elements[0].unity_name = Some("PlayButton".into());
    let mut t = Element::new(0, Kind::Text);
    t.name = "Titulo del menú".into();
    t.text = "Hola".into();
    t.y = 300.0;
    d.add(t);
    let cs = view::base_script(&d);
    assert!(cs.contains("public Button PlayButton;"), "{cs}");
    assert!(cs.contains("if (!PlayButton) PlayButton = Find<Button>(\"PlayButton\");"), "{cs}");
    assert!(cs.contains("OnPlayButtonClick"), "{cs}");
    assert!(cs.contains("serialized references"));
    let v = json_of(&d);
    assert_eq!(v["elements"][0]["field"], "PlayButton");
    assert_eq!(v["elements"][1]["field"], "tituloDelMenu");
    assert_eq!(v["elements"][1]["name"], "Titulo del menú");
    // a unity_name that is not a C# identifier falls back to a camelCase member
    d.elements[0].unity_name = Some("Play 1!".into());
    assert_eq!(json_of(&d)["elements"][0]["name"], "Play 1!");
    assert_eq!(json_of(&d)["elements"][0]["field"], "play1");
    // two names that fold to the same member get distinct ones
    let mut dup = Doc::empty(100.0, 100.0);
    for _ in 0..2 {
        let mut b = Element::new(0, Kind::Button);
        b.name = "Same".into();
        dup.add(b);
    }
    let f = view::field_names(&dup);
    let mut vals: Vec<_> = f.values().cloned().collect();
    vals.sort();
    assert_eq!(vals, vec!["same".to_string(), "same2".to_string()]);
}

#[test]
fn generated_children_names_are_prefixed() {
    for n in ["__border", "__text", "__tick", "__canvas_background", "__safe_area"] {
        assert!(IMPORTER_CS.contains(&format!("\"{}\"", n)), "{n}");
    }
    for old in ["\"Borde\"", "\"Texto\"", "\"Tick\"", "\"Fondo del lienzo\""] {
        assert!(!IMPORTER_CS.contains(old), "{old} still used");
    }
    assert!(GEN_NAMES.iter().all(|n| n.starts_with("__")));
}

#[test]
fn importer_keeps_user_sprites_and_separates_pixel_variants() {
    // generated sprites: pixel-art and smooth variants are different files
    assert!(IMPORTER_CS.contains("PixelSuffix => pixelArt ? \"_px\" : \"\""));
    for k in ["\"rr_\" + r + \"_\" + s + PixelSuffix", "\"ellipse_\" + ring + PixelSuffix", "+ PixelSuffix;"] {
        assert!(IMPORTER_CS.contains(k), "{k}");
    }
    // user textures: only what the design sets, guarded by the owner marker
    assert!(IMPORTER_CS.contains("OwnedMarker"));
    assert!(IMPORTER_CS.contains("LoadAllAssetsAtPath"));
    assert!(IMPORTER_CS.contains("pixelsPerUnitMultiplier") && IMPORTER_CS.contains("Image.Type.Sliced"));
}

#[test]
fn canvas_settings_reach_the_json_and_runtime_helpers() {
    let mut d = doc_with_button();
    d.canvas.render_mode = CanvasRenderMode::WorldSpace;
    d.canvas.sort_order = -3;
    d.canvas.plane_distance = 12.5;
    d.canvas.safe_area = true;
    let v = json_of(&d);
    assert_eq!(v["canvas"]["renderMode"], "WorldSpace");
    assert_eq!(v["canvas"]["sortOrder"], -3);
    assert_eq!(v["canvas"]["planeDistance"], 12.5);
    assert_eq!(v["canvas"]["safeArea"], true);
    assert_eq!(v["canvas"]["embed"], false);
    for k in ["ScreenSpaceCamera", "WorldSpace", "canvas.sortingOrder", "SafeAreaName"] {
        assert!(IMPORTER_CS.contains(k), "{k}");
    }
    // the ViewBase fallback path goes through the safe-area container
    let cs = view::base_script(&d);
    assert!(cs.contains("Find<Button>(\"__safe_area/Play\")"), "{cs}");
    // export: the safe-area helper is written once next to the scripts
    let dir = project("safe");
    let files = write_unity_package(&d, &dir).unwrap();
    assert!(files.iter().any(|f| f == "Assets/EditorGUI/Scripts/Runtime/LienzoSafeArea.cs"), "{files:?}");
    assert!(!files.iter().any(|f| f.contains("LienzoOrientationLayout")));
    let cs = std::fs::read_to_string(dir.join("Assets/EditorGUI/Scripts/Runtime/LienzoSafeArea.cs")).unwrap();
    assert!(cs.contains("Screen.safeArea") && cs.contains("class LienzoSafeArea"));
    // a copy that lives somewhere else in the project is respected (no duplicate class)
    let dir2 = project("safe2");
    std::fs::create_dir_all(dir2.join("Assets/Mine")).unwrap();
    std::fs::write(dir2.join("Assets/Mine/LienzoSafeArea.cs"), "public class LienzoSafeArea : UnityEngine.MonoBehaviour {}").unwrap();
    let files = write_unity_package(&d, &dir2).unwrap();
    assert!(!files.iter().any(|f| f.contains("LienzoSafeArea")), "{files:?}");
    // embed: the root has no Canvas (importer) and the flag is in the JSON
    d.canvas.embed = true;
    assert_eq!(json_of(&d)["canvas"]["embed"], true);
    assert!(IMPORTER_CS.contains("if (cv.embed)"));
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir2);
}

#[test]
fn portrait_states_are_real_rect_transform_values() {
    let mut d = Doc::empty(1920.0, 1080.0);
    d.title = "Hud".into();
    let mut e = Element::new(0, Kind::Panel);
    e.name = "Box".into();
    e.x = 0.0;
    e.y = 0.0;
    e.w = 200.0;
    e.h = 100.0;
    let id = d.add(e);
    let mut o = ElementOverride::new(id);
    o.x = Some(100.0);
    o.y = Some(200.0);
    o.w = Some(300.0);
    o.h = Some(400.0);
    o.visible = Some(false);
    o.font_size = Some(20.0);
    assert!(d.set_override(o));
    let v = json_of(&d);
    let ov = &v["portrait"]["overrides"][0];
    assert_eq!(ov["hasRect"], true);
    assert_eq!(ov["hasVisible"], true);
    assert_eq!(ov["hasFontSize"], true);
    assert_eq!(ov["hasLayout"], false);
    let pd = d.resolved_for(crate::docext::Orientation::Portrait);
    let rt = export_rt(&pd, pd.get(id).unwrap());
    assert_eq!(ov["rect"]["sizeDelta"], serde_json::json!([rt.size_delta[0] as f64, rt.size_delta[1] as f64]));
    assert_eq!(ov["rect"]["anchoredPosition"], serde_json::json!([rt.anchored_position[0] as f64, rt.anchored_position[1] as f64]));
    assert_eq!(ov["rect"]["sizeDelta"], serde_json::json!([300.0, 400.0]));
    // landscape RectTransform is untouched
    assert_ne!(v["elements"][0]["rect"]["sizeDelta"], ov["rect"]["sizeDelta"]);
    // an override without geometry has no `rect`
    let mut o2 = ElementOverride::new(id);
    o2.visible = Some(true);
    d.set_override(o2);
    let v = json_of(&d);
    assert!(v["portrait"]["overrides"][0].get("rect").is_none());
    // export writes the orientation helper; the importer fills both states
    d.unity.generate = GenerateOutputs::new_design();
    let dir = project("portrait");
    let files = write_unity_package(&d, &dir).unwrap();
    assert!(files.iter().any(|f| f == "Assets/EditorGUI/Scripts/Runtime/LienzoOrientationLayout.cs"), "{files:?}");
    let cs = std::fs::read_to_string(dir.join("Assets/EditorGUI/Scripts/Runtime/LienzoOrientationLayout.cs")).unwrap();
    assert!(cs.contains("public void Apply(bool portrait)") && cs.contains("Screen.width < Screen.height"));
    assert!(cs.contains("OnRectTransformDimensionsChange") && cs.contains("void Update()"));
    assert!(IMPORTER_CS.contains("AttachPortrait") && IMPORTER_CS.contains("LienzoOrientationLayout"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn first_generation_flow_is_in_the_importer() {
    for k in [
        "[InitializeOnLoad]",
        "CompilationPipeline.compilationFinished",
        "AssemblyReloadEvents.afterAssemblyReload",
        "Library/LienzoUI/pending.txt",
        "go.GetComponent(view) == null",
        "Tools/Lienzo UI/Regenerate all prefabs",
        "Regenerate all prefabs: ",
    ] {
        assert!(IMPORTER_CS.contains(k), "{k}");
    }
}

#[test]
fn fonts_are_resolved_by_name_without_recreating_existing_assets() {
    for k in ["FindFontAsset", "HasCharacters", "fallbackFontAssetTable", "defaultFont", "has no glyph for"] {
        assert!(IMPORTER_CS.contains(k), "{k}");
    }
    // the existing project asset is searched before an SDF asset is created from a TTF
    let find = IMPORTER_CS.find("var fa = FindFontAsset(family);").unwrap();
    let create = IMPORTER_CS.find("TMP_FontAsset.CreateFontAsset(font)").unwrap();
    assert!(find < create);
    let mut d = doc_with_button();
    d.unity.default_font = Some("Inter SDF".into());
    d.unity.fallback_fonts = vec!["Noto".into(), "Emoji".into()];
    let v = json_of(&d);
    assert_eq!(v["unity"]["defaultFont"], "Inter SDF");
    assert_eq!(v["unity"]["fallbackFonts"], serde_json::json!(["Noto", "Emoji"]));
}

#[test]
fn unity_names_are_verbatim_in_json_and_importer_names_gameobjects_with_them() {
    let mut d = doc_with_button();
    d.elements[0].unity_name = Some("Btn Play".into());
    let v = json_of(&d);
    assert_eq!(v["elements"][0]["name"], "Btn Play");
    assert_eq!(v["elements"][0]["unityName"], "Btn Play");
    assert!(IMPORTER_CS.contains("go.name = e.name;"));
}

#[test]
fn all_templates_export_clean_json() {
    for i in 0..model::TEMPLATES.len() {
        let d = model::template(i, 1920.0, 1080.0);
        let s = ugui::json(&d, true);
        assert!(!s.contains("null"), "template {i}");
        let v: Value = serde_json::from_str(&s).unwrap();
        for e in v["elements"].as_array().unwrap() {
            assert!(e["field"].is_string(), "template {i}: every element has a view field");
        }
    }
}
