//! Lienzo UI: visual editor for game interfaces (canvas, handles, smart
//! guides, layers) that exports to Unity (uGUI + TextMeshPro, UI Toolkit and IMGUI).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod canvas;
mod export;
mod icons;
mod library;
mod history;
mod recovery;
mod model;
mod pack;
mod presets;
mod panels;
mod render;
mod script;
mod theme;
mod accessibility;
mod i18n;
mod plugins;
mod community;
mod community_validate;
mod community_net;
mod tutorial;
mod themes;
mod updater;
mod review;
mod docext;
mod schema;
mod unity_import;
#[cfg(test)]
mod docext_tests;
#[cfg(test)]
mod hardening_tests;
#[cfg(test)]
mod unity_import_tests;

use eframe::egui;
use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

/// `--screenshot <file>` state: 0 = waiting, 1 = requested, 2 = saved, 3 = failed.
pub struct Shot {
    pub path: std::path::PathBuf,
    pub state: Arc<AtomicU8>,
}

const SHOT_FRAME: u64 = 120;
/// With `--screenshot-wait` the screenshot waits more frames (e.g. for the community catalog to download).
const SHOT_FRAME_LONG: u64 = 420;

impl eframe::App for app::EditorApp {
    fn on_exit(&mut self) {
        self.recovery_clean_exit();
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.frame_no += 1;
        render::promote_pending_fonts();
        render::PIXEL_ART.store(self.doc.pixel_art, Ordering::Relaxed);

        self.accessibility_shortcuts(&ctx);
        self.keyboard(&ctx);
        self.header(ui);
        self.rail(ui);
        self.drawer(ui);
        self.inspector(ui);
        self.status_bar(ui);
        self.context_bar(ui);
        egui::CentralPanel::default().frame(egui::Frame::new()).show(ui, |ui| {
            self.canvas_ui(ui);
        });
        self.export_window(&ctx);
        self.save_window(&ctx);
        self.import_window(&ctx);
        self.community_poll(&ctx);
        self.community_windows(&ctx);
        self.tutorial_window(&ctx);
        self.accessibility_window(&ctx);
        self.review_window(&ctx);
        self.themes_window(&ctx);
        self.updater_window(&ctx);
        self.recovery_window(&ctx);
        self.close_prompt_window(&ctx);
        self.commit(&ctx);

        if let Some(shot) = &self.screenshot {
            ctx.request_repaint();
            match shot.state.load(Ordering::SeqCst) {
                0 if self.frame_no >= if self.long_screenshot { SHOT_FRAME_LONG } else { SHOT_FRAME } => {
                    shot.state.store(1, Ordering::SeqCst);
                    let path = shot.path.clone();
                    let state = shot.state.clone();
                    ctx.request_screenshot(move |img| {
                        let [w, h] = img.size;
                        let mut bytes = Vec::with_capacity(w * h * 4);
                        for p in &img.pixels {
                            let [r, g, b, _] = p.to_srgba_unmultiplied();
                            bytes.extend_from_slice(&[r, g, b, 255]);
                        }
                        if let Some(parent) = path.parent() {
                            let _ = std::fs::create_dir_all(parent);
                        }
                        let ok = image::RgbaImage::from_raw(w as u32, h as u32, bytes).map(|im| im.save(&path).is_ok()).unwrap_or(false);
                        state.store(if ok { 2 } else { 3 }, Ordering::SeqCst);
                    });
                }
                2 | 3 => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                _ => {}
            }
        }
    }
}

/// Command-line flags that used to have Spanish names. They still work: `normalize_args` rewrites them to the English ones,
/// so the rest of the code only knows the English flags (and the help text only shows those).
const LEGACY_FLAGS: &[(&str, &str)] = &[
    ("--abrir", "--open"),
    ("--exportar", "--export"),
    ("--guardar", "--save"),
    ("--plantilla", "--template"),
    ("--plantillas", "--templates"),
    ("--revisar", "--review"),
    ("--idioma", "--language"),
    ("--escala", "--scale"),
    ("--tema", "--theme"),
    ("--pestana", "--tab"),
    ("--alto-contraste", "--high-contrast"),
    ("--sin-tutorial", "--no-tutorial"),
    ("--probar", "--test"),
    ("--profesional", "--pro"),
    ("--comunidad-subir", "--community-upload"),
    ("--comunidad-corazon", "--community-heart"),
    ("--comunidad-seguir", "--community-follow"),
    ("--comunidad-yo", "--community-me"),
    ("--comunidad-revisar", "--community-review"),
    ("--comunidad-reintentar", "--community-retry"),
    ("--abrir-revision", "--open-review"),
    ("--abrir-actualizacion", "--open-update"),
    ("--abrir-perfil", "--open-profile"),
    ("--abrir-temas", "--open-themes"),
    ("--abrir-subida", "--open-upload"),
    ("--demo-actualizacion", "--demo-update"),
    ("--demo-comunidad", "--demo-community"),
    ("--espera-capturas", "--screenshot-wait"),
    ("--titulo", "--title"),
    ("--etiquetas", "--labels"),
    ("--descripcion", "--description"),
    ("--icono", "--icon"),
    ("--vista", "--preview"),
];

/// Replaces the old Spanish flags with their English names (the arguments after a flag are left alone).
fn normalize_args(args: Vec<String>) -> Vec<String> {
    args.into_iter()
        .map(|a| match LEGACY_FLAGS.iter().find(|(old, _)| *old == a) {
            Some((_, new)) => new.to_string(),
            None => a,
        })
        .collect()
}

fn arg_after<'a>(args: &'a [String], flag: &str) -> Option<&'a String> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1))
}

/// Design given on the command line: `--open <file>`, `--import-unity <file>` or `--template <n>`.
fn doc_from_args(args: &[String]) -> Result<Option<model::Doc>, String> {
    if let Some(file) = arg_after(args, "--import-unity") {
        let imp = unity_import::import_file(std::path::Path::new(file))?;
        for w in &imp.warnings {
            eprintln!("{}", i18n::tf("Warning: {}", &[w]));
        }
        println!("{}", i18n::tf("Imported {} elements from {} ({} warnings)", &[&imp.doc.elements.len().to_string(), file, &imp.warnings.len().to_string()]));
        return Ok(Some(imp.doc));
    }
    if let Some(file) = arg_after(args, "--open") {
        library::set_design_folder(std::path::Path::new(file));
        let text = std::fs::read_to_string(file).map_err(|e| format!("{}: {}", file, e))?;
        let doc = model::Doc::from_json_str(&text).map_err(|e| format!("{}: {}", file, e))?;
        return Ok(Some(doc));
    }
    if let Some(t) = arg_after(args, "--template").and_then(|v| v.parse::<usize>().ok()) {
        if t >= model::TEMPLATES.len() {
            return Err(format!("template {} does not exist (0-{})", t, model::TEMPLATES.len() - 1));
        }
        return Ok(Some(model::template(t, 1920.0, 1080.0)));
    }
    Ok(None)
}

/// Windowless mode (automation, MCP, CI). Returns Some(exit code) if it was used.
///   --templates                lists the templates
///   --validate <file>          checks a design file against docs/editorgui.schema.json (errors with JSON paths, exit 1)
///   --review                   reviews the design (--open or --template): legibility, contrast, touch size
///   --save <file>              saves the design (--open or --template) as a project and exits
///   --export <Unity project>   exports the design to Unity and exits
///   --import-unity <file>      imports a Unity uGUI .prefab or a Lienzo Designs/*.json as the design (combine with --save / --export / --review)
///   --import-unity-project <Unity project> --out <folder>   converts every UI prefab of the project to .editorgui.json files
/// (The old Spanish flags, like --abrir or --exportar, are rewritten to these by `normalize_args`.)
///   --community-upload <type> <file> --title "…"  uploads content to the community with your saved session
fn headless(args: &[String]) -> Option<i32> {
    if args.iter().any(|a| a == "--templates") {
        for (i, t) in model::TEMPLATES.iter().enumerate() {
            println!("{}\t{}", i, t);
        }
        return Some(0);
    }
    if let Some(i) = args.iter().position(|a| a == "--community-upload") {
        return Some(community::upload_cli(&args[i + 1..]));
    }
    for (flag, what) in [("--community-heart", "heart"), ("--community-follow", "follow")] {
        if let Some(i) = args.iter().position(|a| a == flag) {
            return Some(community::social_cli(what, args.get(i + 1).map(|s| s.as_str()).unwrap_or("")));
        }
    }
    if args.iter().any(|a| a == "--community-me") {
        return Some(community::me_cli());
    }
    if let Some(i) = args.iter().position(|a| a == "--community-review") {
        return Some(community::review_cli(&args[i + 1..]));
    }
    if args.iter().any(|a| a == "--community-retry") {
        return Some(community::retry_cli());
    }
    if let Some(i) = args.iter().position(|a| a == "--validate") {
        return Some(match args.get(i + 1) {
            Some(f) => schema::validate_file(f),
            None => {
                eprintln!("Usage: --validate <file.editorgui.json>");
                2
            }
        });
    }
    if let Some(i) = args.iter().position(|a| a == "--import-unity-project") {
        return Some(import_project_cli(args, i));
    }
    if args.iter().any(|a| a == "--import-unity") && arg_after(args, "--import-unity").is_none_or(|v| v.starts_with("--")) {
        eprintln!("{}", i18n::t("Usage: --import-unity <file.prefab|file.json> [--save <file.editorgui.json>] [--export <Unity project>]"));
        return Some(2);
    }
    if args.iter().any(|a| a == "--review") {
        return Some(match doc_from_args(args) {
            Ok(Some(d)) => {
                let (text, errors) = review::report(&d);
                println!("{}", text);
                if errors { 1 } else { 0 }
            }
            Ok(None) => {
                eprintln!("Specify the design with --open <file> or --template <n>");
                2
            }
            Err(e) => {
                eprintln!("Could not load the design: {}", e);
                1
            }
        });
    }
    let export = arg_after(args, "--export");
    let save = arg_after(args, "--save");
    let importing = args.iter().any(|a| a == "--import-unity");
    if export.is_none() && save.is_none() && !importing {
        return None;
    }
    let doc = match doc_from_args(args) {
        Ok(Some(d)) => d,
        Ok(None) => {
            eprintln!("Specify the design with --open <file> or --template <n>");
            return Some(2);
        }
        Err(e) => {
            eprintln!("Could not load the design: {}", e);
            return Some(1);
        }
    };
    if let Some(file) = save {
        let r = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string()).and_then(|s| std::fs::write(file, s).map_err(|e| e.to_string()));
        match r {
            Ok(()) => println!("Saved: {}", file),
            Err(e) => {
                eprintln!("Could not save {}: {}", file, e);
                return Some(1);
            }
        }
    }
    if let Some(dest) = export {
        match export::write_unity_package(&doc, std::path::Path::new(dest)) {
            Ok(files) => files.iter().for_each(|f| println!("{}", f)),
            Err(e) => {
                eprintln!("Could not export: {}", e);
                return Some(1);
            }
        }
    }
    Some(0)
}

/// `--import-unity-project <Unity project> --out <folder>`: converts every UI prefab of the project. Exit 0 / 1 / 2.
fn import_project_cli(args: &[String], i: usize) -> i32 {
    let dir = args.get(i + 1).filter(|v| !v.starts_with("--"));
    let out = arg_after(args, "--out").filter(|v| !v.starts_with("--"));
    let (Some(dir), Some(out)) = (dir, out) else {
        eprintln!("{}", i18n::t("Usage: --import-unity-project <Unity project folder> --out <folder for the .editorgui.json files>"));
        return 2;
    };
    match unity_import::import_unity_project_report(std::path::Path::new(dir), std::path::Path::new(out)) {
        Ok(r) => {
            for (file, n) in &r.written {
                println!("{}", i18n::tf("{}  ({} elements)", &[file, &n.to_string()]));
            }
            for (prefab, w) in &r.warnings {
                eprintln!("{}", i18n::tf("Warning in {}: {}", &[prefab, w]));
            }
            for (prefab, e) in &r.failed {
                eprintln!("{}", i18n::tf("Could not import {}: {}", &[prefab, e]));
            }
            println!("{}", i18n::tf("Converted {} prefabs, skipped {} (not UI), failed {}, {} warnings", &[&r.written.len().to_string(), &r.skipped.len().to_string(), &r.failed.len().to_string(), &r.warnings.len().to_string()]));
            if r.failed.is_empty() { 0 } else { 1 }
        }
        Err(e) => {
            eprintln!("{}", i18n::tf("Could not import: {}", &[&e]));
            1
        }
    }
}

fn main() -> eframe::Result<()> {
    let args = normalize_args(std::env::args().collect());
    if let Some(code) = headless(&args) {
        std::process::exit(code);
    }
    let shot_path = args.iter().position(|a| a == "--screenshot").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from);

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Lienzo UI · design interfaces for Unity")
        .with_inner_size([1500.0, 900.0])
        .with_min_inner_size([960.0, 600.0]);
    // The window starts maximized (except in automatic screenshots, which need a fixed size).
    if shot_path.is_none() {
        viewport = viewport.with_maximized(true);
    }
    if let Ok(img) = image::load_from_memory(include_bytes!("../assets/brand/icon.png")) {
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();
        viewport = viewport.with_icon(egui::IconData { rgba: rgba.into_raw(), width, height });
    }
    if shot_path.is_some() {
        viewport = viewport.with_active(false);
        #[cfg(target_os = "windows")]
        {
            viewport = viewport.with_window_level(egui::WindowLevel::AlwaysOnBottom);
        }
    }
    #[allow(unused_mut)]
    let mut options = eframe::NativeOptions { viewport, ..Default::default() };
    // On Windows skip the OpenGL fallback probe (its helper thread can overflow its stack);
    // DirectX 12 / Vulkan are used. WGPU_BACKEND still overrides this.
    #[cfg(target_os = "windows")]
    if std::env::var_os("WGPU_BACKEND").is_none() {
        use eframe::egui_wgpu::{WgpuSetup, WgpuSetupCreateNew, wgpu};
        let mut create = WgpuSetupCreateNew::without_display_handle();
        create.instance_descriptor.backends = wgpu::Backends::PRIMARY;
        options.wgpu_options.wgpu_setup = WgpuSetup::CreateNew(create);
    }
    #[cfg(target_os = "macos")]
    if shot_path.is_some() {
        options.event_loop_builder = Some(Box::new(|b| {
            use winit::platform::macos::EventLoopBuilderExtMacOS;
            b.with_activate_ignoring_other_apps(false);
        }));
    }

    let shot = shot_path.map(|path| Shot { path, state: Arc::new(AtomicU8::new(0)) });
    let state = shot.as_ref().map(|s| s.state.clone());
    let result = eframe::run_native(
        "editor_gui",
        options,
        Box::new(move |cc| {
            setup_style(&cc.egui_ctx);
            let mut app = app::EditorApp::new(shot);
            if let Some(p) = arg_after(&args, "--tab") {
                app.tab = match p.as_str() {
                    "templates" | "plantillas" => app::Tab::Templates,
                    "elements" => app::Tab::Elements,
                    "text" | "texto" => app::Tab::Text,
                    "images" | "imagenes" => app::Tab::Images,
                    "plugins" => app::Tab::Plugins,
                    "community" | "comunidad" => app::Tab::Community,
                    _ => app::Tab::Layers,
                };
            }
            // --open-review: opens the "Review design" window (screenshots).
            app.review_open = args.iter().any(|a| a == "--open-review");
            // Automatic screenshots do not save settings.
            if app.screenshot.is_some() {
                app.settings.read_only = true;
            }
            // Accessibility options for this session only (applied to a copy and not saved):
            // --language en, --scale 1.5, --high-contrast
            app.updater_start();
            // --demo-update [--open-update]: simulates a new version (screenshots); does not query GitHub.
            if args.iter().any(|a| a == "--demo-update") {
                app.updater.state = updater::State::Available(updater::Release {
                    version: "1.2.0".into(),
                    notes: "In-app updates: a button appears in the header when there is a new version.".into(),
                    url_zip: "demo".into(),
                    sha256: "demo".into(),
                });
                app.updater.open = args.iter().any(|a| a == "--open-update");
            }
            let mut session = app.settings.clone();
            if let Some(i) = arg_after(&args, "--language") {
                session.language = i.clone();
            }
            if let Some(e) = arg_after(&args, "--scale").and_then(|v| v.parse::<f32>().ok()) {
                session.ui_scale = e;
            }
            if args.iter().any(|a| a == "--high-contrast") {
                session.high_contrast = true;
            }
            // --theme "Lienzo Light": an app theme for this session only (screenshots, tests)
            if let Some(n) = arg_after(&args, "--theme") {
                if let Some(t) = theme::find_default(n) {
                    app.settings.theme = Some(t.clone()); // in memory only: settings are not saved in this session
                    session.theme = Some(t);
                }
            }
            accessibility::apply(&cc.egui_ctx, &session);
            // Documentation only (README screenshots): a FAKE, offline session, plus the real anonymous catalog.
            //   --demo-community [--open-profile] [--open-themes] [--screenshot-wait]
            if args.iter().any(|a| a == "--demo-community") {
                app.community_demo(&cc.egui_ctx, args.iter().any(|a| a == "--open-profile"), args.iter().any(|a| a == "--open-upload"));
            }
            if args.iter().any(|a| a == "--open-themes") {
                app.themes_open = true;
            }
            app.long_screenshot = args.iter().any(|a| a == "--screenshot-wait");
            // --test 1080x2400 : start in test mode simulating that screen
            if let Some(spec) = args.iter().position(|a| a == "--test").and_then(|i| args.get(i + 1)) {
                app.set_preview(true);
                let wh: Vec<f32> = spec.split('x').filter_map(|v| v.trim().parse().ok()).collect();
                if wh.len() == 2 {
                    app.preview_screen = Some([wh[0], wh[1]]);
                }
            }
            // The tutorial opens by itself the first time (never in automatic screenshots, except --tutorial).
            let force = args.iter().any(|a| a == "--tutorial");
            let skip = args.iter().any(|a| a == "--no-tutorial");
            if force || (!skip && app.screenshot.is_none() && !app.settings.tutorial_seen) {
                let track = if args.iter().any(|a| a == "--pro") { tutorial::Track::Pro } else { tutorial::Track::Beginner };
                app.open_tutorial(track);
            }
            if let Some(path) = arg_after(&args, "--open") {
                app.project_path = path.clone();
                app.open_project();
            } else if let Some(t) = arg_after(&args, "--template").and_then(|v| v.parse::<usize>().ok()) {
                if t < model_templates() {
                    app.load_template(t);
                }
            }
            let cli_project = arg_after(&args, "--open").cloned();
            app.recovery_startup(cli_project.as_deref());
            Ok(Box::new(app))
        }),
    );
    if let Some(s) = state {
        if s.load(Ordering::SeqCst) != 2 {
            eprintln!("screenshot was not saved");
            std::process::exit(1);
        }
    }
    result
}

fn model_templates() -> usize {
    model::TEMPLATES.len()
}

fn setup_style(ctx: &egui::Context) {
    egui_extras::install_image_loaders(ctx);
    render::load_system_fonts(ctx);
    ctx.options_mut(|o| o.zoom_with_keyboard = false);
    theme::apply(ctx, false);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn old_spanish_flags_are_rewritten_to_the_english_ones() {
        let old = normalize_args(args(&["lienzo-ui", "--abrir", "a.editorgui.json", "--exportar", "Proyecto", "--guardar", "b.json", "--plantilla", "3", "--idioma", "en", "--escala", "1.5",
            "--tema", "Lienzo claro", "--pestana", "plantillas", "--alto-contraste", "--sin-tutorial", "--probar", "1080x2400", "--profesional", "--comunidad-subir", "--titulo", "T",
            "--espera-capturas", "--abrir-revision", "--demo-comunidad", "--comunidad-yo"]));
        let new = args(&["lienzo-ui", "--open", "a.editorgui.json", "--export", "Proyecto", "--save", "b.json", "--template", "3", "--language", "en", "--scale", "1.5",
            "--theme", "Lienzo claro", "--tab", "plantillas", "--high-contrast", "--no-tutorial", "--test", "1080x2400", "--pro", "--community-upload", "--title", "T",
            "--screenshot-wait", "--open-review", "--demo-community", "--community-me"]);
        assert_eq!(old, new, "values (including Spanish ones like the tab name or theme name) are left alone");
        // English flags pass through untouched
        assert_eq!(normalize_args(new.clone()), new);
        // every old flag maps to a distinct English one
        let mut seen = std::collections::HashSet::new();
        assert!(LEGACY_FLAGS.iter().all(|(old, new)| old != new && old.starts_with("--") && new.starts_with("--") && seen.insert(*new)));
    }

    #[test]
    fn headless_import_unity_modes_and_exit_codes() {
        let dir = std::env::temp_dir().join(format!("lienzo_cli_import_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("Design.json");
        std::fs::write(&src, export::ugui::json(&model::template(2, 1920.0, 1080.0), true)).unwrap();
        let out = dir.join("out.editorgui.json");
        let (s, o) = (src.to_string_lossy().to_string(), out.to_string_lossy().to_string());
        assert_eq!(headless(&args(&["lienzo-ui", "--import-unity", &s, "--save", &o])), Some(0));
        let d = model::Doc::from_json_str(&std::fs::read_to_string(&out).unwrap()).unwrap();
        assert_eq!(d.elements.len(), model::template(2, 1920.0, 1080.0).elements.len());
        // only importing is also a headless run
        assert_eq!(headless(&args(&["lienzo-ui", "--import-unity", &s])), Some(0));
        // missing value / missing file / project without --out
        assert_eq!(headless(&args(&["lienzo-ui", "--import-unity"])), Some(2));
        assert_eq!(headless(&args(&["lienzo-ui", "--import-unity", "--save", &o])), Some(2));
        assert_eq!(headless(&args(&["lienzo-ui", "--import-unity", &dir.join("none.prefab").to_string_lossy(), "--save", &o])), Some(1));
        assert_eq!(headless(&args(&["lienzo-ui", "--import-unity-project", &dir.to_string_lossy()])), Some(2));
        assert_eq!(headless(&args(&["lienzo-ui", "--import-unity-project", &dir.to_string_lossy(), "--out", &dir.join("o").to_string_lossy()])), Some(1), "no Assets folder");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn headless_accepts_old_flags() {
        // --plantillas (old) lists the templates and exits with 0, exactly like --templates
        assert_eq!(headless(&normalize_args(args(&["lienzo-ui", "--plantillas"]))), Some(0));
        assert_eq!(headless(&normalize_args(args(&["lienzo-ui", "--templates"]))), Some(0));
        // --revisar --plantilla 0 (old) reviews the first template
        assert_eq!(headless(&normalize_args(args(&["lienzo-ui", "--revisar", "--plantilla", "0"]))).map(|c| c < 2), Some(true));
    }
}
