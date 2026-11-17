//! Lienzo UI: editor visual de interfaces de juego (lienzo, tiradores, guías
//! inteligentes, capas) que exporta a Unity (uGUI + TextMeshPro, UI Toolkit e IMGUI).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod canvas;
mod export;
mod icons;
mod library;
mod model;
mod panels;
mod render;
mod script;
mod theme;
mod accesibilidad;
mod i18n;
mod plugins;
mod comunidad;
mod comunidad_validar;
mod tutorial;
mod temas;
mod actualizador;
mod revision;

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
/// Con `--espera-capturas` la captura espera más fotogramas (p. ej. a que baje el catálogo de la comunidad).
const SHOT_FRAME_LARGA: u64 = 420;

impl eframe::App for app::EditorApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.frame_no += 1;
        render::promote_pending_fonts();
        render::PIXEL_ART.store(self.doc.pixel_art, Ordering::Relaxed);

        self.atajos_accesibilidad(&ctx);
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
        self.comunidad_poll(&ctx);
        self.comunidad_ventanas(&ctx);
        self.tutorial_window(&ctx);
        self.accesibilidad_window(&ctx);
        self.revision_window(&ctx);
        self.temas_window(&ctx);
        self.actualizador_window(&ctx);
        self.commit(&ctx);

        if let Some(shot) = &self.screenshot {
            ctx.request_repaint();
            match shot.state.load(Ordering::SeqCst) {
                0 if self.frame_no >= if self.captura_larga { SHOT_FRAME_LARGA } else { SHOT_FRAME } => {
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

fn arg_after<'a>(args: &'a [String], flag: &str) -> Option<&'a String> {
    args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1))
}

/// Diseño indicado en la línea de comandos: `--abrir <archivo>` o `--plantilla <n>`.
fn doc_from_args(args: &[String]) -> Result<Option<model::Doc>, String> {
    if let Some(file) = arg_after(args, "--abrir") {
        library::set_carpeta_diseno(std::path::Path::new(file));
        let text = std::fs::read_to_string(file).map_err(|e| format!("{}: {}", file, e))?;
        let mut doc: model::Doc = serde_json::from_str(&text).map_err(|e| format!("{}: {}", file, e))?;
        doc.migrate_groups();
        return Ok(Some(doc));
    }
    if let Some(t) = arg_after(args, "--plantilla").and_then(|v| v.parse::<usize>().ok()) {
        if t >= model::TEMPLATES.len() {
            return Err(format!("la plantilla {} no existe (0-{})", t, model::TEMPLATES.len() - 1));
        }
        return Ok(Some(model::template(t, 1920.0, 1080.0)));
    }
    Ok(None)
}

/// Modo sin ventana (automatización, MCP, CI). Devuelve Some(código de salida) si se usó.
///   --plantillas                 lista las plantillas
///   --revisar                    revisa el diseño (--abrir o --plantilla): legibilidad, contraste, tamaño táctil
///   --guardar <archivo>          guarda el diseño (--abrir o --plantilla) como proyecto y termina
///   --exportar <proyecto Unity>  exporta el diseño a Unity y termina
///   --comunidad-subir <tipo> <archivo> --titulo "…"  sube contenido a la comunidad con tu sesión guardada
fn headless(args: &[String]) -> Option<i32> {
    if args.iter().any(|a| a == "--plantillas") {
        for (i, t) in model::TEMPLATES.iter().enumerate() {
            println!("{}\t{}", i, t);
        }
        return Some(0);
    }
    if let Some(i) = args.iter().position(|a| a == "--comunidad-subir") {
        return Some(comunidad::subir_cli(&args[i + 1..]));
    }
    for (flag, que) in [("--comunidad-corazon", "corazon"), ("--comunidad-seguir", "seguir")] {
        if let Some(i) = args.iter().position(|a| a == flag) {
            return Some(comunidad::social_cli(que, args.get(i + 1).map(|s| s.as_str()).unwrap_or("")));
        }
    }
    if args.iter().any(|a| a == "--comunidad-yo") {
        return Some(comunidad::yo_cli());
    }
    if let Some(i) = args.iter().position(|a| a == "--comunidad-revisar") {
        return Some(comunidad::revisar_cli(&args[i + 1..]));
    }
    if args.iter().any(|a| a == "--comunidad-reintentar") {
        return Some(comunidad::reintentar_cli());
    }
    if args.iter().any(|a| a == "--revisar") {
        return Some(match doc_from_args(args) {
            Ok(Some(d)) => {
                let (texto, errores) = revision::informe(&d);
                println!("{}", texto);
                if errores { 1 } else { 0 }
            }
            Ok(None) => {
                eprintln!("Indica el diseño con --abrir <archivo> o --plantilla <n>");
                2
            }
            Err(e) => {
                eprintln!("No se pudo cargar el diseño: {}", e);
                1
            }
        });
    }
    let exportar = arg_after(args, "--exportar");
    let guardar = arg_after(args, "--guardar");
    if exportar.is_none() && guardar.is_none() {
        return None;
    }
    let doc = match doc_from_args(args) {
        Ok(Some(d)) => d,
        Ok(None) => {
            eprintln!("Indica el diseño con --abrir <archivo> o --plantilla <n>");
            return Some(2);
        }
        Err(e) => {
            eprintln!("No se pudo cargar el diseño: {}", e);
            return Some(1);
        }
    };
    if let Some(file) = guardar {
        let r = serde_json::to_string_pretty(&doc).map_err(|e| e.to_string()).and_then(|s| std::fs::write(file, s).map_err(|e| e.to_string()));
        match r {
            Ok(()) => println!("Guardado: {}", file),
            Err(e) => {
                eprintln!("No se pudo guardar {}: {}", file, e);
                return Some(1);
            }
        }
    }
    if let Some(dest) = exportar {
        match export::write_unity_package(&doc, std::path::Path::new(dest)) {
            Ok(files) => files.iter().for_each(|f| println!("{}", f)),
            Err(e) => {
                eprintln!("No se pudo exportar: {}", e);
                return Some(1);
            }
        }
    }
    Some(0)
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if let Some(code) = headless(&args) {
        std::process::exit(code);
    }
    let shot_path = args.iter().position(|a| a == "--screenshot").and_then(|i| args.get(i + 1)).map(std::path::PathBuf::from);

    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Lienzo UI · diseña interfaces para Unity")
        .with_inner_size([1500.0, 900.0])
        .with_min_inner_size([960.0, 600.0]);
    // La ventana arranca maximizada (salvo en capturas automáticas, que necesitan un tamaño fijo).
    if shot_path.is_none() {
        viewport = viewport.with_maximized(true);
    }
    if let Ok(img) = image::load_from_memory(include_bytes!("../assets/marca/icono.png")) {
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
            if let Some(p) = arg_after(&args, "--pestana") {
                app.tab = match p.as_str() {
                    "plantillas" => app::Tab::Templates,
                    "elementos" => app::Tab::Elements,
                    "texto" => app::Tab::Text,
                    "imagenes" => app::Tab::Images,
                    "plugins" => app::Tab::Plugins,
                    "comunidad" => app::Tab::Comunidad,
                    _ => app::Tab::Layers,
                };
            }
            // --abrir-revision: abre la ventana «Revisar diseño» (capturas).
            app.revision_abierta = args.iter().any(|a| a == "--abrir-revision");
            // Las capturas automáticas no guardan ajustes.
            if app.screenshot.is_some() {
                app.settings.solo_lectura = true;
            }
            // Opciones de accesibilidad solo para esta sesión (se aplican sobre una copia y no se guardan):
            // --idioma en, --escala 1.5, --alto-contraste
            app.actualizador_inicio();
            // --demo-actualizacion [--abrir-actualizacion]: simula una versión nueva (capturas); no consulta GitHub.
            if args.iter().any(|a| a == "--demo-actualizacion") {
                app.actualizador.estado = actualizador::Estado::Disponible(actualizador::Novedad {
                    version: "1.2.0".into(),
                    notas: "Actualizaciones desde la app: aparece un botón en la cabecera cuando hay una versión nueva.".into(),
                    url_zip: "demo".into(),
                    sha256: "demo".into(),
                });
                app.actualizador.abierta = args.iter().any(|a| a == "--abrir-actualizacion");
            }
            let mut sesion = app.settings.clone();
            if let Some(i) = arg_after(&args, "--idioma") {
                sesion.idioma = i.clone();
            }
            if let Some(e) = arg_after(&args, "--escala").and_then(|v| v.parse::<f32>().ok()) {
                sesion.escala_ui = e;
            }
            if args.iter().any(|a| a == "--alto-contraste") {
                sesion.alto_contraste = true;
            }
            // --tema "Lienzo claro": un tema de la app solo para esta sesión (capturas, pruebas)
            if let Some(n) = arg_after(&args, "--tema") {
                if let Some(t) = theme::predeterminados().into_iter().find(|t| t.nombre == *n) {
                    app.settings.tema = Some(t.clone()); // solo en memoria: en esta sesión no se guardan ajustes
                    sesion.tema = Some(t);
                }
            }
            accesibilidad::aplicar(&cc.egui_ctx, &sesion);
            // Solo para la documentación (capturas del README): una sesión FALSA y sin red, más el catálogo real y anónimo.
            //   --demo-comunidad [--abrir-perfil] [--abrir-temas] [--espera-capturas]
            if args.iter().any(|a| a == "--demo-comunidad") {
                app.comunidad_demo(&cc.egui_ctx, args.iter().any(|a| a == "--abrir-perfil"), args.iter().any(|a| a == "--abrir-subida"));
            }
            if args.iter().any(|a| a == "--abrir-temas") {
                app.temas_abierto = true;
            }
            app.captura_larga = args.iter().any(|a| a == "--espera-capturas");
            // --probar 1080x2400 : start in test mode simulating that screen
            if let Some(spec) = args.iter().position(|a| a == "--probar").and_then(|i| args.get(i + 1)) {
                app.set_preview(true);
                let wh: Vec<f32> = spec.split('x').filter_map(|v| v.trim().parse().ok()).collect();
                if wh.len() == 2 {
                    app.preview_screen = Some([wh[0], wh[1]]);
                }
            }
            // El tutorial se abre solo la primera vez (nunca en capturas automáticas, salvo --tutorial).
            let forzar = args.iter().any(|a| a == "--tutorial");
            let sin = args.iter().any(|a| a == "--sin-tutorial");
            if forzar || (!sin && app.screenshot.is_none() && !app.settings.tutorial_visto) {
                let pista = if args.iter().any(|a| a == "--profesional") { tutorial::Pista::Profesional } else { tutorial::Pista::Novato };
                app.abrir_tutorial(pista);
            }
            if let Some(path) = arg_after(&args, "--abrir") {
                app.project_path = path.clone();
                app.open_project();
            } else if let Some(t) = arg_after(&args, "--plantilla").and_then(|v| v.parse::<usize>().ok()) {
                if t < model_templates() {
                    app.load_template(t);
                }
            }
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
