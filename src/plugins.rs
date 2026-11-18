//! Lienzo UI plugins (add-ons).
//!
//! A plugin is a folder with a `plugin.json` inside one of the plugin folders:
//!   * `plugins/` next to the executable,
//!   * `plugins/` in the working folder (handy when developing from the repository),
//!   * `%APPDATA%/EditorGUI/plugins` (or its equivalent on macOS/Linux).
//!
//! It can provide:
//!   * commands: Rhai scripts with `fn ejecutar(diseno) { ...; diseno }` that receive the design as a
//!     map and return the modified design (can be undone with Ctrl+Z);
//!   * exporters: `fn exportar(diseno) { ... }` that return the text of a new file;
//!   * templates: `.editorgui.json` designs that appear in the Plugins tab.
//!
//! Scripts run sandboxed: no access to files, network or processes, and with operation limits.
//! Full format and examples: docs/PLUGINS.md and the repository's plugins/ folder.

use crate::app::EditorApp;
use crate::i18n::t;
use crate::model::Doc;
use crate::theme::{ACCENT, MUTED, WARN};
use eframe::egui::{self, RichText};
use std::path::{Path, PathBuf};

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct Manifiesto {
    nombre: String,
    version: String,
    autor: String,
    descripcion: String,
    comandos: Vec<Accion>,
    exportadores: Vec<Exportador>,
    plantillas: Vec<String>,
}

#[derive(serde::Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Accion {
    pub nombre: String,
    pub descripcion: String,
    pub script: String,
}

#[derive(serde::Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Exportador {
    pub nombre: String,
    pub extension: String,
    pub script: String,
}

#[derive(Clone, Debug)]
pub struct Plugin {
    pub carpeta: PathBuf,
    pub nombre: String,
    pub version: String,
    pub autor: String,
    pub descripcion: String,
    pub comandos: Vec<Accion>,
    pub exportadores: Vec<Exportador>,
    pub plantillas: Vec<PathBuf>,
    pub error: Option<String>,
}

/// Folders where plugins are searched for (no duplicates).
pub fn carpetas() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(exe) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)) {
        v.push(exe.join("plugins"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        v.push(cwd.join("plugins"));
    }
    if let Some(datos) = crate::library::root_dir().parent() {
        v.push(datos.join("plugins"));
    }
    let mut unicas: Vec<PathBuf> = Vec::new();
    for c in v {
        let canon = c.canonicalize().unwrap_or(c.clone());
        if !unicas.iter().any(|u| u.canonicalize().unwrap_or(u.clone()) == canon) {
            unicas.push(c);
        }
    }
    unicas
}

/// The user's plugins folder (created when opened from the app).
pub fn carpeta_usuario() -> PathBuf {
    crate::library::root_dir().parent().map(|d| d.join("plugins")).unwrap_or_else(|| PathBuf::from("plugins"))
}

/// Path of a plugin file, without leaving its folder.
fn dentro(carpeta: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel_path = Path::new(rel);
    // Only normal names: on Windows "\carpeta" or "C:x" are not absolute but `join` takes them out of the folder.
    let normal = rel_path.components().all(|c| matches!(c, std::path::Component::Normal(_) | std::path::Component::CurDir));
    if rel.is_empty() || rel_path.is_absolute() || rel_path.has_root() || !normal {
        return Err(format!("path not allowed in the plugin: {}", rel));
    }
    Ok(carpeta.join(rel_path))
}

pub fn cargar(carpeta: &Path) -> Plugin {
    let mut p = Plugin {
        carpeta: carpeta.to_path_buf(),
        nombre: carpeta.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
        version: String::new(),
        autor: String::new(),
        descripcion: String::new(),
        comandos: Vec::new(),
        exportadores: Vec::new(),
        plantillas: Vec::new(),
        error: None,
    };
    let m: Manifiesto = match std::fs::read_to_string(carpeta.join("plugin.json")).map_err(|e| e.to_string()).and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string())) {
        Ok(m) => m,
        Err(e) => {
            p.error = Some(format!("plugin.json: {}", e));
            return p;
        }
    };
    if !m.nombre.trim().is_empty() {
        p.nombre = m.nombre;
    }
    p.version = m.version;
    p.autor = m.autor;
    p.descripcion = m.descripcion;
    let mut errores = Vec::new();
    for c in m.comandos {
        match dentro(carpeta, &c.script) {
            Ok(f) if f.exists() => p.comandos.push(c),
            Ok(f) => errores.push(format!("no existe {}", f.display())),
            Err(e) => errores.push(e),
        }
    }
    for x in m.exportadores {
        match dentro(carpeta, &x.script) {
            Ok(f) if f.exists() => p.exportadores.push(x),
            Ok(f) => errores.push(format!("no existe {}", f.display())),
            Err(e) => errores.push(e),
        }
    }
    for pl in m.plantillas {
        match dentro(carpeta, &pl) {
            Ok(f) if f.exists() => p.plantillas.push(f),
            Ok(f) => errores.push(format!("no existe {}", f.display())),
            Err(e) => errores.push(e),
        }
    }
    if !errores.is_empty() {
        p.error = Some(errores.join("; "));
    }
    p
}

pub fn cargar_todos() -> Vec<Plugin> {
    let mut v = Vec::new();
    for base in carpetas() {
        let Ok(rd) = std::fs::read_dir(&base) else { continue };
        let mut dirs: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.join("plugin.json").exists()).collect();
        dirs.sort();
        v.extend(dirs.iter().map(|d| cargar(d)));
    }
    v
}

/// Sandboxed Rhai engine with limits, plus a message output (`log("…")`).
fn motor() -> (rhai::Engine, std::rc::Rc<std::cell::RefCell<Vec<String>>>) {
    let mut e = rhai::Engine::new();
    e.set_max_operations(20_000_000);
    e.set_max_call_levels(64);
    e.set_max_string_size(2_000_000);
    e.set_max_array_size(200_000);
    e.set_max_map_size(200_000);
    e.set_module_resolver(rhai::module_resolvers::DummyModuleResolver::new());
    let log = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let l2 = log.clone();
    e.register_fn("log", move |s: &str| l2.borrow_mut().push(s.to_string()));
    let l3 = log.clone();
    e.on_print(move |s| l3.borrow_mut().push(s.to_string()));
    (e, log)
}

fn llamar(carpeta: &Path, script: &str, funcion: &str, doc: &Doc) -> Result<(rhai::Dynamic, Vec<String>), String> {
    let ruta = dentro(carpeta, script)?;
    let codigo = std::fs::read_to_string(&ruta).map_err(|e| format!("{}: {}", ruta.display(), e))?;
    let (engine, log) = motor();
    let ast = engine.compile(&codigo).map_err(|e| format!("{}: {}", script, e))?;
    let arg = rhai::serde::to_dynamic(doc).map_err(|e| e.to_string())?;
    let r = engine.call_fn::<rhai::Dynamic>(&mut rhai::Scope::new(), &ast, funcion, (arg,)).map_err(|e| format!("{}: {}", script, e))?;
    let msgs = log.borrow().clone();
    Ok((r, msgs))
}

/// Runs a command: returns the new design and the script's messages.
pub fn ejecutar_comando(carpeta: &Path, c: &Accion, doc: &Doc) -> Result<(Doc, Vec<String>), String> {
    let (r, msgs) = llamar(carpeta, &c.script, "ejecutar", doc)?;
    // Rhai works with f64 and the model with f32: it goes through JSON, which converts the numbers.
    let json: serde_json::Value = rhai::serde::from_dynamic(&r).map_err(|e| format!("{}: the returned design is not valid ({})", c.script, e))?;
    let mut nuevo: Doc = serde_json::from_value(json).map_err(|e| format!("{}: the returned design is not valid ({})", c.script, e))?;
    nuevo.migrate_groups();
    Ok((nuevo, msgs))
}

/// Runs an exporter: returns the file's text.
pub fn ejecutar_exportador(carpeta: &Path, x: &Exportador, doc: &Doc) -> Result<String, String> {
    let (r, _) = llamar(carpeta, &x.script, "exportar", doc)?;
    r.into_string().map_err(|tipo| format!("{}: exportar() must return text, it returned {}", x.script, tipo))
}

impl EditorApp {
    pub fn tab_plugins(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(t("Plugins")).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button(t("Reload")).clicked() {
                    self.plugins = cargar_todos();
                    self.status = format!("{}: {}", t("Loaded plugins"), self.plugins.len());
                }
                if ui.small_button(t("Open folder")).on_hover_text(carpeta_usuario().display().to_string()).clicked() {
                    let c = carpeta_usuario();
                    let _ = std::fs::create_dir_all(&c);
                    let _ = std::process::Command::new(if cfg!(windows) { "explorer" } else if cfg!(target_os = "macos") { "open" } else { "xdg-open" }).arg(&c).spawn();
                }
            });
        });
        ui.label(RichText::new(t("They add commands, exporters and templates. Guide: docs/PLUGINS.md")).small().color(MUTED()));
        ui.add_space(6.0);
        if self.plugins.is_empty() {
            ui.label(RichText::new(t("No plugins installed. Copy a folder with plugin.json into the plugins folder and press Reload.")).color(MUTED()));
            return;
        }
        let mut accion: Option<(usize, u8, usize)> = None; // (plugin, kind 0 command / 1 exporter / 2 template, index)
        for (i, p) in self.plugins.iter().enumerate() {
            egui::Frame::new().fill(crate::theme::PANEL_DARK()).corner_radius(6.0).inner_margin(8.0).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(format!("{}  {}", p.nombre, p.version)).strong());
                if !p.autor.is_empty() {
                    ui.label(RichText::new(format!("{} {}", t("by"), p.autor)).small().color(MUTED()));
                }
                if !p.descripcion.is_empty() {
                    ui.label(RichText::new(&p.descripcion).small());
                }
                if let Some(e) = &p.error {
                    ui.label(RichText::new(e).small().color(WARN()));
                }
                for (j, c) in p.comandos.iter().enumerate() {
                    let b = ui.add_sized([ui.available_width(), 24.0], crate::icons::btn("play", c.nombre.as_str()));
                    if b.on_hover_text(&c.descripcion).clicked() {
                        accion = Some((i, 0, j));
                    }
                }
                for (j, x) in p.exportadores.iter().enumerate() {
                    if ui.add_sized([ui.available_width(), 24.0], crate::icons::btn("download", format!("{} (.{})", x.nombre, x.extension))).clicked() {
                        accion = Some((i, 1, j));
                    }
                }
                for (j, f) in p.plantillas.iter().enumerate() {
                    let nombre = f.file_stem().map(|s| s.to_string_lossy().replace(".editorgui", "")).unwrap_or_default();
                    if ui.add_sized([ui.available_width(), 24.0], egui::Button::new(RichText::new(format!("▦ {}", nombre)).color(ACCENT()))).clicked() {
                        accion = Some((i, 2, j));
                    }
                }
            });
            ui.add_space(6.0);
        }
        if let Some((i, tipo, j)) = accion {
            let p = self.plugins[i].clone();
            match tipo {
                0 => match ejecutar_comando(&p.carpeta, &p.comandos[j], &self.doc) {
                    Ok((nuevo, msgs)) => {
                        self.doc = nuevo;
                        self.sel.clear();
                        self.status = format!("{} “{}”{}", t("Plugin executed:"), p.comandos[j].nombre, if msgs.is_empty() { String::new() } else { format!(" · {}", msgs.join(" · ")) });
                    }
                    Err(e) => self.status = format!("{} {}", t("Plugin error:"), e),
                },
                1 => match ejecutar_exportador(&p.carpeta, &p.exportadores[j], &self.doc) {
                    Ok(texto) => {
                        let x = &p.exportadores[j];
                        let nombre = format!("{}.{}", crate::export::class_name(&self.doc.title), x.extension);
                        self.save_text_file(&x.nombre, &nombre, &x.extension, &texto);
                    }
                    Err(e) => self.status = format!("{} {}", t("Plugin error:"), e),
                },
                _ => {
                    let f = &p.plantillas[j];
                    match std::fs::read_to_string(f).map_err(|e| e.to_string()).and_then(|s| serde_json::from_str::<Doc>(&s).map_err(|e| e.to_string())) {
                        Ok(d) => {
                            // Same as a custom template: leaves test mode and locks the backgrounds.
                            self.load_user_template(&d);
                            self.status = format!("{} {}", t("Plugin template applied (Ctrl+Z to undo):"), f.display());
                        }
                        Err(e) => self.status = format!("{} {}", t("Plugin error:"), e),
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn carpeta_ejemplos() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("plugins")
    }

    #[test]
    fn carga_los_plugins_de_ejemplo() {
        for nombre in ["ajustar-cuadricula", "textos-csv"] {
            let p = cargar(&carpeta_ejemplos().join(nombre));
            assert!(p.error.is_none(), "{}: {:?}", nombre, p.error);
        }
    }

    #[test]
    fn comando_ajusta_a_cuadricula() {
        let p = cargar(&carpeta_ejemplos().join("ajustar-cuadricula"));
        let mut doc = crate::model::template(0, 1920.0, 1080.0);
        doc.elements[1].x = 13.0;
        let (nuevo, _) = ejecutar_comando(&p.carpeta, &p.comandos[0], &doc).expect("the command must work");
        assert_eq!(nuevo.elements.len(), doc.elements.len());
        assert!(nuevo.elements.iter().all(|e| e.x % 8.0 == 0.0 && e.y % 8.0 == 0.0));
    }

    #[test]
    fn exportador_csv() {
        let p = cargar(&carpeta_ejemplos().join("textos-csv"));
        let doc = crate::model::template(0, 1920.0, 1080.0);
        let csv = ejecutar_exportador(&p.carpeta, &p.exportadores[0], &doc).expect("the exporter must work");
        assert!(csv.starts_with("elemento,texto"));
        assert!(csv.contains("Play"));
    }

    #[test]
    fn rutas_fuera_del_plugin_rechazadas() {
        assert!(dentro(Path::new("x"), "../fuera.rhai").is_err());
        assert!(dentro(Path::new("x"), "C:/Windows/a.rhai").is_err() || cfg!(not(windows)));
        assert!(dentro(Path::new("x"), "ok.rhai").is_ok());
    }

    #[test]
    fn script_sin_limite_se_corta() {
        let dir = std::env::temp_dir().join("lienzo-plugin-bucle");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("bucle.rhai"), "fn ejecutar(d) { loop { } }").unwrap();
        let c = Accion { nombre: "bucle".into(), descripcion: String::new(), script: "bucle.rhai".into() };
        let doc = crate::model::template(5, 1920.0, 1080.0);
        assert!(ejecutar_comando(&dir, &c, &doc).is_err());
    }
}

#[cfg(test)]
mod rutas_tests {
    use super::*;

    #[test]
    fn rutas_que_salen_de_la_carpeta() {
        let c = Path::new("plugins/x");
        for mala in ["", "../a.rhai", "a/../../b.rhai", "/etc/a.rhai", "\\Users\\a.rhai", "C:a.rhai", "C:\\a.rhai"] {
            assert!(dentro(c, mala).is_err(), "should reject {:?}", mala);
        }
        for buena in ["a.rhai", "sub/a.rhai", "./a.rhai"] {
            assert!(dentro(c, buena).is_ok(), "should accept {:?}", buena);
        }
    }
}
