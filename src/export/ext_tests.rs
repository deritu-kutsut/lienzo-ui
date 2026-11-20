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
fn dry_run_lists_the_files_without_touching_the_project() {
    let d = model::template(3, 1920.0, 1080.0);
    let dir = project("dryrun");
    let before = std::fs::read_dir(&dir).unwrap().count();
    let plan = write_unity_package_plan(&d, &dir, true).unwrap();
    assert!(plan.len() > 3 && plan.iter().all(|(_, c)| *c == Change::New), "{plan:?}");
    assert!(plan.iter().any(|(f, _)| f.ends_with(".json")) && plan.iter().any(|(f, _)| f.ends_with("EditorGuiImporter.cs")));
    assert!(!dir.join("Assets/EditorGUI").exists(), "a dry run must not create anything");
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), before);
    // a real export writes exactly what the plan said; a second dry run then reports the same / unchanged files
    let written = write_unity_package(&d, &dir).unwrap();
    assert_eq!(written, plan.iter().map(|(f, _)| f.clone()).collect::<Vec<_>>());
    let again = write_unity_package_plan(&d, &dir, true).unwrap();
    assert!(again.iter().all(|(_, c)| *c == Change::Unchanged), "{again:?}");
    // a changed design is reported as an update and still nothing is written
    let mut d2 = d.clone();
    d2.elements[0].name = "Other name".into();
    let json = dir.join(&plan.iter().find(|(f, _)| f.ends_with(&format!("{}.json", class_name(&d.title)))).unwrap().0);
    let before_json = std::fs::read(&json).unwrap();
    let changed = write_unity_package_plan(&d2, &dir, true).unwrap();
    assert!(changed.iter().any(|(_, c)| *c == Change::Changed), "{changed:?}");
    assert_eq!(std::fs::read(&json).unwrap(), before_json);
    assert_eq!(Change::New.label(), "new");
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
    std::fs::write(dir.join("Assets/Game/Runtime/Game.Runtime.asmdef"), r#"{ "name": "Game.Runtime", "references": ["UnityEngine.UI", "Unity.TextMeshPro"] }"#).unwrap();
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

// ---------------------------------------------------------------------------------------------------------------------
// Audit round 2: image / font names, UXML + USS details, importer lifecycle, assemblies, hostile characters.
// ---------------------------------------------------------------------------------------------------------------------

fn png_bytes(seed: u8) -> Vec<u8> {
    let mut v = Vec::new();
    let mut img = image::RgbaImage::new(2, 2);
    for p in img.pixels_mut() {
        *p = image::Rgba([seed, 10, 20, 255]);
    }
    image::DynamicImage::ImageRgba8(img).write_to(&mut std::io::Cursor::new(&mut v), image::ImageFormat::Png).unwrap();
    v
}

fn doc_with_images(images: &[&str]) -> Doc {
    let mut d = Doc::empty(1920.0, 1080.0);
    d.title = "Pics".into();
    for im in images {
        let mut e = Element::new(0, Kind::Image);
        e.name = format!("Pic {}", im);
        e.image = im.to_string();
        d.add(e);
    }
    d
}

#[test]
fn same_file_name_from_different_folders_gets_different_sprite_files() {
    let dir = project("imgnames");
    let (a, b, c, d2) = (dir.join("a/icon.png"), dir.join("b/icon.png"), dir.join("c/icon.png"), dir.join("d/Icon.PNG"));
    for (p, seed) in [(&a, 1u8), (&b, 2), (&c, 1), (&d2, 3)] {
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, png_bytes(seed)).unwrap();
    }
    let imgs = ["a/icon.png", "b/icon.png", "c/icon.png", "d/Icon.PNG", "a/other.png"];
    let doc = doc_with_images(&imgs);
    let map = |i: &str| Some(dir.join(i));
    let n = ImageNames::with_resolver(&doc, map);
    assert_eq!(n.file("a/icon.png"), "icon.png", "the first picture keeps the plain name");
    assert_eq!(n.file("b/icon.png"), "icon_2.png", "different content, same name");
    assert_eq!(n.file("c/icon.png"), "icon.png", "same content = same file");
    assert_eq!(n.file("d/Icon.PNG"), "Icon_3.PNG", "names are compared case-insensitively (Windows / macOS)");
    assert_eq!(n.file("a/other.png"), "other.png");
    assert_eq!(n.asset_path("b/icon.png"), "Assets/EditorGUI/Sprites/icon_2.png");
    // no clash -> exactly the old name
    let solo = ImageNames::with_resolver(&doc_with_images(&["x/hero.png"]), |_| None);
    assert_eq!(solo.asset_path("x/hero.png"), image_asset_path("x/hero.png"));
    assert_eq!(suffixed_name("noext", 2), "noext_2");
    assert_eq!(suffixed_name(".hidden", 2), ".hidden_2");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn json_uss_and_imgui_use_the_same_image_mapping() {
    // exporters go through ImageNames::new: a design without clashes keeps the plain names everywhere
    let d = doc_with_images(&["Assets/Art/a.png"]);
    assert!(json_of(&d)["elements"][0]["image"]["source"] == "Assets/Art/a.png");
    let mut d = doc_with_images(&["hero.png"]);
    d.elements[0].kind = Kind::Panel;
    let json = json_of(&d);
    let src = json["elements"][0]["image"]["source"].as_str().unwrap().to_string();
    assert_eq!(src, "Assets/EditorGUI/Sprites/hero.png");
    assert!(uitk::uss(&d).contains(&format!("project://database/{}", src)));
    assert!(imgui::script(&d).contains("// hero.png"));
}

#[test]
fn headless_export_copies_fonts_and_warns_about_missing_ones() {
    let arial = Path::new("C:/Windows/Fonts/arial.ttf");
    let mut d = doc_with_button();
    d.elements[0].font = "Arial".into();
    let uss_has_rule = |d: &Doc| uitk::uss(d).contains("-unity-font: url(\"project://database/Assets/EditorGUI/Fonts/Arial.ttf\")");
    let dir = project("fonts");
    let files = write_unity_package(&d, &dir).unwrap();
    if arial.exists() {
        // the GUI never ran here: the table is filled lazily
        assert!(files.iter().any(|f| f == "Assets/EditorGUI/Fonts/Arial.ttf"), "{files:?}");
        assert!(uss_has_rule(&d));
        assert_eq!(font_ext("Arial").as_deref(), Some("ttf"));
    }
    assert!(!uitk::uss(&d).contains("SDF.asset"), "a TextMeshPro asset is not a UI Toolkit font");
    // a font nobody has: nothing is copied and USS keeps the default font
    d.elements[0].font = "Nonexistent Font 123".into();
    let dir2 = project("fonts2");
    let files = write_unity_package(&d, &dir2).unwrap();
    assert!(!files.iter().any(|f| f.contains("/Fonts/")), "{files:?}");
    assert!(!uitk::uss(&d).contains("-unity-font:") && !uitk::uss(&d).contains("-unity-font-definition"));
    assert!(font_file("Nonexistent Font 123").is_none() && font_ext("Nonexistent Font 123").is_none());
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir2);
}

#[test]
fn uxml_choices_never_split_and_rich_text_is_off() {
    let mut d = Doc::empty(800.0, 600.0);
    d.title = "Opts".into();
    let mut dd = Element::new(0, Kind::Dropdown);
    dd.name = "Quality".into();
    dd.options = vec!["Low, fast".into(), "High".into(), "A,B,C".into()];
    d.add(dd);
    let mut t = Element::new(0, Kind::Text);
    t.text = "<b>bold?</b>".into();
    d.add(t);
    let mut b = Element::new(0, Kind::Button);
    b.text = "<color=red>x</color>".into();
    d.add(b);
    let mut p = Element::new(0, Kind::Panel);
    p.text = "<i>cap</i>".into();
    d.add(p);
    let uxml = uitk::uxml(&d);
    let line = uxml.lines().find(|l| l.contains("<ui:DropdownField")).unwrap();
    let choices = line.split("choices=\"").nth(1).unwrap().split('"').next().unwrap();
    assert_eq!(choices.split(',').count(), 3, "three choices stay three: {choices}");
    assert!(choices.contains('\u{201A}') && !choices.contains("Low, fast"));
    assert!(uxml.contains("comma inside a choice"), "{uxml}");
    // rich text off on every UI Toolkit text element that supports it
    for l in uxml.lines().filter(|l| l.contains("<ui:Label") || l.contains("<ui:Button")) {
        assert!(l.contains("enable-rich-text=\"false\""), "{l}");
    }
    // a choice list without commas is unchanged
    assert_eq!(uitk::choices_attr(&["A".into(), "B".into()]), "A,B");
    // uGUI importer and IMGUI
    assert!(IMPORTER_CS.contains("t.richText = false;") && IMPORTER_CS.contains("inp.richText = false;") && IMPORTER_CS.contains("dd.itemText.richText = false;"));
    assert!(imgui::script(&d).contains("richText = false"));
}

#[test]
fn importer_default_font_ownership_marker_and_fallback_rules() {
    // default font names (the app writes "Monospace"), misses are cached, .ttc is looked up
    for k in ["IsDefaultFontName", "\"Monospace\"", "\"Monoespaciada\"", "fontCache[family] = fa;", "\".ttc\""] {
        assert!(IMPORTER_CS.contains(k), "{k}");
    }
    // sprites: the marker carries a hash of what Lienzo wrote; a different state = the user's change = hands off
    for k in ["SettingsHash", "Ownership.Modified", "OwnedMarker + \":\" + SettingsHash(ti)", "was changed by hand"] {
        assert!(IMPORTER_CS.contains(k), "{k}");
    }
    // other texture types are never retyped
    assert!(IMPORTER_CS.contains("ti.textureType == TextureImporterType.Default") && IMPORTER_CS.contains("left untouched"));
    // fallbacks: logged, only with a configured list, never in a package
    let i = IMPORTER_CS.find("static void EnsureFallbacks").unwrap();
    let body = &IMPORTER_CS[i..i + 2200];
    assert!(body.contains("fallbackFontNames.Length == 0") && body.contains("Debug.Log(") && body.contains("Packages/"), "{body}");
}

#[test]
fn pending_list_retries_hashes_designs_and_warns() {
    for k in ["const int MaxAttempts = 5;", "if (retry) Schedule();", "is still not compiled after", "DesignHash", "lienzo:design:", "StoredDesignHash", "Where(b => b != 13)"] {
        assert!(IMPORTER_CS.contains(k), "{k}");
    }
    // the rebuild decision is the stored content hash, not "json newer than prefab"
    let i = IMPORTER_CS.find("static bool NeedsRebuild").unwrap();
    let body = &IMPORTER_CS[i..i + 900];
    assert!(body.contains("stored != EditorGuiImporter.DesignHash(json)"), "{body}");
    assert!(body.contains("AddMinutes(1)"), "legacy prefabs only: a clearly newer design");
    // BuildPrefab records the hash
    assert!(IMPORTER_CS.contains("StampDesignHash(prefabPath, DesignHash(jsonPath));"));
}

#[test]
fn safe_area_only_acts_on_a_full_screen_overlay_canvas_and_uses_driven_rects() {
    for k in ["DrivenRectTransformTracker", "tracker.Add(", "tracker.Clear()", "RenderMode.ScreenSpaceOverlay", "InFullScreenOverlay", "PrefabStageUtility", "nested Canvas", "Debug.LogWarning"] {
        assert!(SAFE_AREA_CS.contains(k), "{k}");
    }
    // braces balance (cheap syntax check; the real compile is done against the Unity DLLs)
    assert_eq!(SAFE_AREA_CS.matches('{').count(), SAFE_AREA_CS.matches('}').count());
}

#[test]
fn all_outputs_off_is_a_clear_error() {
    let mut d = doc_with_button();
    d.unity.generate = GenerateOutputs { prefab: false, uitk: false, imgui: Some(false), view_scripts: false };
    let dir = project("nothing");
    let err = write_unity_package(&d, &dir).unwrap_err();
    assert!(err.contains("every output set to false"), "{err}");
    assert!(!dir.join("Assets/EditorGUI").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

fn count_cs(p: &Path) -> usize {
    std::fs::read_dir(p).unwrap().flatten().map(|e| if e.path().is_dir() { count_cs(&e.path()) } else { e.path().extension().is_some_and(|x| x == "cs") as usize }).sum()
}

#[test]
fn every_cs_file_is_read_once_per_export() {
    let dir = project("reads");
    std::fs::create_dir_all(dir.join("Assets/Game")).unwrap();
    for i in 0..7 {
        std::fs::write(dir.join(format!("Assets/Game/C{i}.cs")), format!("public class C{i} {{}}")).unwrap();
    }
    let mut d = doc_with_button();
    d.canvas.safe_area = true; // the helper scan, the ViewBase scan and the View scan all run
    d.unity.generate = GenerateOutputs { prefab: true, uitk: true, imgui: Some(false), view_scripts: true };
    CS_READS.with(|c| c.set(0));
    write_unity_package(&d, &dir).unwrap();
    // the 7 + the importer the export wrote just before (the first scan loads the cache)
    assert_eq!(CS_READS.with(|c| c.get()), 8, "three scans share one read per file");
    // a second export finds the 7 + what the first one generated: still one read per file
    let before = count_cs(&dir.join("Assets"));
    assert!(before > 8);
    CS_READS.with(|c| c.set(0));
    write_unity_package(&d, &dir).unwrap();
    assert_eq!(CS_READS.with(|c| c.get()), before);
    let _ = std::fs::remove_dir_all(&dir);
}

fn asm_project(tag: &str, asmdef: &str, meta: Option<&str>) -> (PathBuf, Doc) {
    let dir = project(tag);
    std::fs::create_dir_all(dir.join("Assets/Game/Runtime")).unwrap();
    std::fs::write(dir.join("Assets/Game/Runtime/Game.Runtime.asmdef"), asmdef).unwrap();
    if let Some(g) = meta {
        std::fs::write(dir.join("Assets/Game/Runtime/Game.Runtime.asmdef.meta"), format!("fileFormatVersion: 2\nguid: {}\nAssemblyDefinitionImporter:\n  userData: \n", g)).unwrap();
    }
    let mut d = doc_with_button();
    d.unity.generate = GenerateOutputs::new_design();
    d.unity.assembly = Some("Game.Runtime".into());
    (dir, d)
}

#[test]
fn assembly_without_ui_references_or_editor_only_falls_back_with_a_warning() {
    // references missing: Assembly-CSharp + a note that says which ones
    let (dir, d) = asm_project("asm-missing", r#"{ "name": "Game.Runtime", "references": ["UnityEngine.UI"] }"#, None);
    let (p, note) = resolve_scripts_dir(&d, &dir);
    assert_eq!(p, DEFAULT_SCRIPTS_DIR);
    let note = note.unwrap();
    assert!(note.contains("Unity.TextMeshPro") && note.contains("Assembly-CSharp") && note.contains("NOT"), "{note}");
    let files = write_unity_package(&d, &dir).unwrap();
    assert!(files.iter().any(|f| f == "Assets/EditorGUI/Scripts/MenuViewBase.cs"), "{files:?}");
    // the user asked for a folder inside the failing assembly: still outside it
    let mut d2 = d.clone();
    d2.unity.scripts_dir = Some("Assets/Game/Runtime/UI".into());
    assert_eq!(resolve_scripts_dir(&d2, &dir).0, DEFAULT_SCRIPTS_DIR);
    // IMGUI only needs no UI package: the assembly is fine
    let mut d3 = d.clone();
    d3.unity.generate = GenerateOutputs { prefab: false, uitk: false, imgui: Some(true), view_scripts: false };
    assert_eq!(resolve_scripts_dir(&d3, &dir), ("Assets/Game/Runtime/LienzoUI".to_string(), None));
    let _ = std::fs::remove_dir_all(&dir);
    // Editor-only assembly
    let (dir, d) = asm_project("asm-editor", r#"{ "name": "Game.Runtime", "includePlatforms": ["Editor"], "references": ["UnityEngine.UI", "Unity.TextMeshPro"] }"#, None);
    let mut d_imgui = d.clone();
    d_imgui.unity.generate = GenerateOutputs { prefab: false, uitk: false, imgui: Some(true), view_scripts: false };
    for doc in [&d, &d_imgui] {
        let (p, note) = resolve_scripts_dir(doc, &dir);
        assert_eq!(p, DEFAULT_SCRIPTS_DIR);
        assert!(note.unwrap().contains("Editor-only"));
    }
    let _ = std::fs::remove_dir_all(&dir);
    // references by GUID (Unity's default) are resolved: the well-known UnityEngine.UI / Unity.TextMeshPro GUIDs
    let refs = r#"{ "name": "Game.Runtime", "references": ["GUID:2bafac87e7f4b9b418d9448d219b01ab", "GUID:6055be8ebefd69e48b49212b09b47b2f"] }"#;
    let (dir, d) = asm_project("asm-guid-refs", refs, None);
    assert_eq!(resolve_scripts_dir(&d, &dir), ("Assets/Game/Runtime/LienzoUI".to_string(), None));
    let _ = std::fs::remove_dir_all(&dir);
    // an unresolvable GUID: cannot be sure -> kept, with a note
    let (dir, d) = asm_project("asm-guid-unknown", r#"{ "name": "Game.Runtime", "references": ["GUID:ffffffffffffffffffffffffffffffff"] }"#, None);
    let (p, note) = resolve_scripts_dir(&d, &dir);
    assert_eq!(p, "Assets/Game/Runtime/LienzoUI");
    assert!(note.unwrap().contains("could not confirm"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn asmref_by_guid_resolves_through_the_asmdef_meta() {
    // the asmdef lives in an embedded package (not in Assets); Assets/Extra has an .asmref by GUID
    let dir = project("asmref-guid");
    let pkg = dir.join("Packages/com.game.core/Runtime");
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::write(pkg.join("Game.Core.asmdef"), r#"{ "name": "Game.Core", "references": ["UnityEngine.UI", "Unity.TextMeshPro"] }"#).unwrap();
    std::fs::write(pkg.join("Game.Core.asmdef.meta"), "fileFormatVersion: 2\nguid: ABCDEF0123456789abcdef0123456789\n").unwrap();
    std::fs::create_dir_all(dir.join("Assets/Extra")).unwrap();
    std::fs::write(dir.join("Assets/Extra/Core.asmref"), r#"{ "reference": "GUID:abcdef0123456789abcdef0123456789" }"#).unwrap();
    let mut d = doc_with_button();
    d.unity.generate = GenerateOutputs::new_design();
    d.unity.assembly = Some("Game.Core".into());
    assert_eq!(find_asmdef_folder(&dir, "Game.Core").as_deref(), Some("Assets/Extra"));
    assert_eq!(resolve_scripts_dir(&d, &dir), ("Assets/Extra/LienzoUI".to_string(), None));
    // the same asmref, but the package asmdef lacks the references: fallback, never silent
    std::fs::write(pkg.join("Game.Core.asmdef"), r#"{ "name": "Game.Core", "references": [] }"#).unwrap();
    let (p, note) = resolve_scripts_dir(&d, &dir);
    assert_eq!(p, DEFAULT_SCRIPTS_DIR);
    assert!(note.unwrap().contains("does not reference"));
    // a GUID nobody has: the assembly is "not found"
    std::fs::write(dir.join("Assets/Extra/Core.asmref"), r#"{ "reference": "GUID:00000000000000000000000000000000" }"#).unwrap();
    assert!(find_asmdef_folder(&dir, "Game.Core").is_none());
    assert!(resolve_scripts_dir(&d, &dir).1.unwrap().contains("not found"));
    let _ = std::fs::remove_dir_all(&dir);
}

const BIDI: &str = "a\u{202E}b\u{202A}c\u{202B}\u{202C}\u{202D}\u{2066}\u{2067}\u{2068}\u{2069}\u{200E}\u{200F}\u{061C}\u{200B}\u{200C}\u{200D}\u{2060}\u{FEFF}z";

fn assert_no_invisible(name: &str, text: &str) {
    for c in text.chars() {
        assert!(!is_invisible_ctrl(c), "{name}: raw U+{:04X}", c as u32);
    }
}

#[test]
fn bidi_and_zero_width_characters_are_escaped_everywhere() {
    let mut d = Doc::empty(800.0, 600.0);
    d.title = format!("T{}", BIDI);
    for k in [Kind::Button, Kind::Text, Kind::Toggle, Kind::InputField, Kind::Dropdown, Kind::Panel] {
        let mut e = Element::new(0, k);
        e.name = format!("N{}", BIDI);
        e.text = format!("X{}", BIDI);
        e.on_click = format!("C{}", BIDI);
        e.options = vec![format!("O{}", BIDI), "P".into()];
        e.font = format!("F{}", BIDI);
        d.add(e);
    }
    // note: `Doc::sanitize` is NOT called: the exporters must be safe on their own
    let json = ugui::json(&d, true);
    assert_no_invisible("json", &json);
    let v: Value = serde_json::from_str(&json).unwrap();
    // same values once parsed (emoji ZWJ sequences, Persian ZWNJ... keep working)
    assert_eq!(v["elements"][1]["text"]["value"], format!("X{}", BIDI));
    assert_no_invisible("uxml", &uitk::uxml(&d));
    assert_no_invisible("uss", &uitk::uss(&d));
    assert_no_invisible("imgui", &imgui::script(&d));
    assert_no_invisible("view_base", &view::base_script(&d));
    assert_no_invisible("view_user", &view::user_script(&d));
    // numeric character reference in UXML: it parses back to the same character
    assert!(uitk::uxml(&d).contains("&#x202E;"));
    assert_eq!(xml_attr("a\u{200B}b"), "a&#x200B;b");
    assert_eq!(css_url_part("a\u{202E}b"), "ab");
    assert_eq!(sanitize_line("a\u{200F}b"), "a b");
}

#[test]
fn non_latin_titles_get_stable_distinct_class_names() {
    let titles = ["主菜单", "设置", "Главное меню", "Настройки", "🎮", "メニュー"];
    let names: Vec<String> = titles.iter().map(|t| class_name(t)).collect();
    for (t, n) in titles.iter().zip(&names) {
        assert!(n.starts_with("Design_") && n.len() == "Design_".len() + 8, "{t} -> {n}");
        assert_eq!(&class_name(t), n, "stable");
    }
    let mut uniq = names.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(uniq.len(), titles.len(), "no two titles share a name: {names:?}");
    // ASCII and Latin-with-accents titles keep their old names; a blank title is plain Design
    assert_eq!(class_name("Menú principal"), "MenuPrincipal");
    assert_eq!(class_name("   "), "Design");
    assert_eq!(class_name(""), "Design");
    // two designs with such titles export to two sets of files
    let dir = project("cjk");
    for t in ["主菜单", "设置"] {
        let mut d = doc_with_button();
        d.title = t.into();
        write_unity_package(&d, &dir).unwrap();
    }
    let designs: Vec<_> = std::fs::read_dir(dir.join("Assets/EditorGUI/Designs")).unwrap().flatten().collect();
    assert_eq!(designs.len(), 2);
    // view fields of two non-Latin element names do not collapse into one
    let mut d = Doc::empty(100.0, 100.0);
    for n in ["按钮一", "按钮二"] {
        let mut b = Element::new(0, Kind::Button);
        b.name = n.into();
        d.add(b);
    }
    let f = view::field_names(&d);
    let mut vals: Vec<_> = f.values().cloned().collect();
    vals.sort();
    vals.dedup();
    assert_eq!(vals.len(), 2);
    assert!(vals.iter().all(|v| v.starts_with("design_")), "{vals:?}");
    let _ = std::fs::remove_dir_all(&dir);
}
