//! Lienzo UI plugins (add-ons).
//!
//! A plugin is a folder with a `plugin.json` inside one of the plugin folders:
//!   * `plugins/` next to the executable,
//!   * `plugins/` in the working folder (handy when developing from the repository),
//!   * `%APPDATA%/EditorGUI/plugins` (or its equivalent on macOS/Linux).
//!
//! It can provide:
//!   * commands: Rhai scripts with `fn run(design) { ...; design }` that receive the design as a
//!     map and return the modified design (can be undone with Ctrl+Z);
//!   * exporters: `fn export_text(design) { ... }` that return the text of a new file;
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

/// `plugin.json`. Manifests written with the old Spanish keys (`nombre`, `autor`, `comandos`…) still load through the aliases.
#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct Manifest {
    #[serde(alias = "nombre")]
    name: String,
    version: String,
    #[serde(alias = "autor")]
    author: String,
    #[serde(alias = "descripcion")]
    description: String,
    #[serde(alias = "comandos")]
    commands: Vec<Action>,
    #[serde(alias = "exportadores")]
    exporters: Vec<Exporter>,
    #[serde(alias = "plantillas")]
    templates: Vec<String>,
}

#[derive(serde::Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Action {
    #[serde(alias = "nombre")]
    pub name: String,
    #[serde(alias = "descripcion")]
    pub description: String,
    pub script: String,
}

#[derive(serde::Deserialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Exporter {
    #[serde(alias = "nombre")]
    pub name: String,
    pub extension: String,
    pub script: String,
}

/// Looks up a manifest key by its English name, falling back to the old Spanish one.
pub fn manifest_key<'a>(m: &'a serde_json::Value, new: &str, old: &str) -> Option<&'a serde_json::Value> {
    m.get(new).or_else(|| m.get(old))
}

#[derive(Clone, Debug)]
pub struct Plugin {
    pub folder: PathBuf,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub commands: Vec<Action>,
    pub exporters: Vec<Exporter>,
    pub templates: Vec<PathBuf>,
    pub error: Option<String>,
}

/// Folders where plugins are searched for (no duplicates).
pub fn folders() -> Vec<PathBuf> {
    let mut v = Vec::new();
    if let Some(exe) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)) {
        v.push(exe.join("plugins"));
    }
    if let Ok(cwd) = std::env::current_dir() {
        v.push(cwd.join("plugins"));
    }
    if let Some(data) = crate::library::root_dir().parent() {
        v.push(data.join("plugins"));
    }
    let mut unique: Vec<PathBuf> = Vec::new();
    for c in v {
        let canon = c.canonicalize().unwrap_or(c.clone());
        if !unique.iter().any(|u| u.canonicalize().unwrap_or(u.clone()) == canon) {
            unique.push(c);
        }
    }
    unique
}

/// The user's plugins folder (created when opened from the app).
pub fn user_folder() -> PathBuf {
    crate::library::root_dir().parent().map(|d| d.join("plugins")).unwrap_or_else(|| PathBuf::from("plugins"))
}

/// Path of a plugin file, without leaving its folder.
fn inside(folder: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel_path = Path::new(rel);
    // Only normal names: on Windows "\folder" or "C:x" are not absolute but `join` takes them out of the folder.
    let normal = rel_path.components().all(|c| matches!(c, std::path::Component::Normal(_) | std::path::Component::CurDir));
    if rel.is_empty() || rel_path.is_absolute() || rel_path.has_root() || !normal {
        return Err(format!("path not allowed in the plugin: {}", rel));
    }
    Ok(folder.join(rel_path))
}

pub fn load(folder: &Path) -> Plugin {
    let mut p = Plugin {
        folder: folder.to_path_buf(),
        name: folder.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(),
        version: String::new(),
        author: String::new(),
        description: String::new(),
        commands: Vec::new(),
        exporters: Vec::new(),
        templates: Vec::new(),
        error: None,
    };
    let m: Manifest = match std::fs::read_to_string(folder.join("plugin.json")).map_err(|e| e.to_string()).and_then(|s| serde_json::from_str(&s).map_err(|e| e.to_string())) {
        Ok(m) => m,
        Err(e) => {
            p.error = Some(format!("plugin.json: {}", e));
            return p;
        }
    };
    if !m.name.trim().is_empty() {
        p.name = m.name;
    }
    p.version = m.version;
    p.author = m.author;
    p.description = m.description;
    let mut errors = Vec::new();
    for c in m.commands {
        match inside(folder, &c.script) {
            Ok(f) if f.exists() => p.commands.push(c),
            Ok(f) => errors.push(format!("{} does not exist", f.display())),
            Err(e) => errors.push(e),
        }
    }
    for x in m.exporters {
        if !crate::community_validate::valid_export_extension(&x.extension) {
            errors.push(format!("exporter “{}”: extension not allowed (allowed: {})", crate::community_validate::terminal_safe(&x.name), crate::community_validate::EXPORT_EXTENSIONS.join(", ")));
            continue;
        }
        match inside(folder, &x.script) {
            Ok(f) if f.exists() => p.exporters.push(x),
            Ok(f) => errors.push(format!("{} does not exist", f.display())),
            Err(e) => errors.push(e),
        }
    }
    for pl in m.templates {
        match inside(folder, &pl) {
            Ok(f) if f.exists() => p.templates.push(f),
            Ok(f) => errors.push(format!("{} does not exist", f.display())),
            Err(e) => errors.push(e),
        }
    }
    if !errors.is_empty() {
        p.error = Some(errors.join("; "));
    }
    p
}

pub fn load_all() -> Vec<Plugin> {
    let mut v = Vec::new();
    for base in folders() {
        let Ok(rd) = std::fs::read_dir(&base) else { continue };
        let mut dirs: Vec<PathBuf> = rd.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.join("plugin.json").exists()).collect();
        dirs.sort();
        v.extend(dirs.iter().map(|d| load(d)));
    }
    v
}

/// Sandboxed Rhai engine with limits, plus a bounded message output (`log("…")`, `print`, `debug`).
fn engine() -> (rhai::Engine, crate::script::Log) {
    let mut e = rhai::Engine::new();
    let log = crate::script::OutLog::new_shared();
    // 5 s of wall-clock time and 5M operations, strings of 64 KB and arrays/maps of 10k items (nested data counts):
    // plenty for a command, never a hang nor gigabytes of memory. No imports. Output is capped (see `script::OutLog`).
    crate::script::harden_with_log(&mut e, 5_000_000, PLUGIN_MAX_STRING, PLUGIN_MAX_ITEMS, 5000, Some(log.clone()));
    let l2 = log.clone();
    e.register_fn("log", move |s: &str| l2.borrow_mut().push(s));
    let l3 = log.clone();
    e.on_print(move |s| l3.borrow_mut().push(s));
    let l4 = log.clone();
    e.on_debug(move |s, _, _| l4.borrow_mut().push(s));
    (e, log)
}

/// Limits of a plugin script (see `engine`).
pub const PLUGIN_MAX_STRING: usize = 64 * 1024;
pub const PLUGIN_MAX_ITEMS: usize = 10_000;

/// Runs the first of `functions` that the script defines (the English name first, then the old Spanish one).
fn call(folder: &Path, script: &str, functions: &[&str], doc: &Doc) -> Result<(rhai::Dynamic, Vec<String>), String> {
    let path = inside(folder, script)?;
    let code = std::fs::read_to_string(&path).map_err(|e| format!("{}: {}", path.display(), e))?;
    let (engine, log) = engine();
    let ast = engine.compile(&code).map_err(|e| format!("{}: {}", script, e))?;
    let arg = rhai::serde::to_dynamic(doc).map_err(|e| e.to_string())?;
    let function = functions.iter().copied().find(|f| ast.iter_functions().any(|d| d.name == *f)).unwrap_or(functions[0]);
    let r = engine.call_fn::<rhai::Dynamic>(&mut rhai::Scope::new(), &ast, function, (arg,)).map_err(|e| format!("{}: {}", script, e))?;
    if log.borrow().overflow {
        return Err(format!("{}: too much output (message limit exceeded)", script));
    }
    let msgs = log.borrow().lines.clone();
    Ok((r, msgs))
}

/// Runs a command: returns the new design and the script's messages.
pub fn run_command(folder: &Path, c: &Action, doc: &Doc) -> Result<(Doc, Vec<String>), String> {
    let (r, msgs) = call(folder, &c.script, &["run", "ejecutar"], doc)?;
    // Rhai works with f64 and the model with f32: it goes through JSON, which converts the numbers.
    let json: serde_json::Value = rhai::serde::from_dynamic(&r).map_err(|e| format!("{}: the returned design is not valid ({})", c.script, e))?;
    let mut new: Doc = serde_json::from_value(json).map_err(|e| format!("{}: the returned design is not valid ({})", c.script, e))?;
    // The output of a plugin is untrusted: new strings and image references must be safe (see `check_plugin_output`).
    let input = serde_json::to_value(doc).map_err(|e| e.to_string())?;
    let output = serde_json::to_value(&new).map_err(|e| e.to_string())?;
    crate::community_validate::check_plugin_output(&input, &output).map_err(|e| format!("{}: {}", c.script, e))?;
    new.migrate_groups();
    Ok((new, msgs))
}

/// Extension for the Save dialog: the one the plugin declares when it is allowed, otherwise `txt`.
pub fn safe_extension(ext: &str) -> String {
    let e = ext.trim().trim_start_matches('.').to_lowercase();
    if crate::community_validate::valid_export_extension(&e) { e } else { "txt".into() }
}

/// Marks designs loaded while it lives as coming from an untrusted source (plugin output, plugin or community
/// templates): absolute image paths are refused. Restores the previous state on drop.
pub struct UntrustedSource(bool);

impl UntrustedSource {
    pub fn begin() -> Self {
        let before = crate::library::trusted_source();
        crate::library::set_trusted_source(false);
        UntrustedSource(before)
    }
}

impl Drop for UntrustedSource {
    fn drop(&mut self) {
        crate::library::set_trusted_source(self.0);
    }
}

/// Runs an exporter: returns the file's text.
pub fn run_exporter(folder: &Path, x: &Exporter, doc: &Doc) -> Result<String, String> {
    let (r, _) = call(folder, &x.script, &["export_text", "exportar"], doc)?;
    r.into_string().map_err(|kind| format!("{}: export_text() must return text, it returned {}", x.script, kind))
}

impl EditorApp {
    pub fn tab_plugins(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(t("Plugins")).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button(t("Reload")).clicked() {
                    self.plugins = load_all();
                    self.status = format!("{}: {}", t("Loaded plugins"), self.plugins.len());
                }
                if ui.small_button(t("Open folder")).on_hover_text(user_folder().display().to_string()).clicked() {
                    let c = user_folder();
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
        let mut action: Option<(usize, u8, usize)> = None; // (plugin, kind 0 command / 1 exporter / 2 template, index)
        for (i, p) in self.plugins.iter().enumerate() {
            egui::Frame::new().fill(crate::theme::PANEL_DARK()).corner_radius(6.0).inner_margin(8.0).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(format!("{}  {}", p.name, p.version)).strong());
                if !p.author.is_empty() {
                    ui.label(RichText::new(format!("{} {}", t("by"), p.author)).small().color(MUTED()));
                }
                if !p.description.is_empty() {
                    ui.label(RichText::new(&p.description).small());
                }
                if let Some(e) = &p.error {
                    ui.label(RichText::new(e).small().color(WARN()));
                }
                for (j, c) in p.commands.iter().enumerate() {
                    let b = ui.add_sized([ui.available_width(), 24.0], crate::icons::btn("play", c.name.as_str()));
                    if b.on_hover_text(&c.description).clicked() {
                        action = Some((i, 0, j));
                    }
                }
                for (j, x) in p.exporters.iter().enumerate() {
                    if ui.add_sized([ui.available_width(), 24.0], crate::icons::btn("download", format!("{} (.{})", x.name, safe_extension(&x.extension)))).clicked() {
                        action = Some((i, 1, j));
                    }
                }
                for (j, f) in p.templates.iter().enumerate() {
                    let name = f.file_stem().map(|s| s.to_string_lossy().replace(".editorgui", "")).unwrap_or_default();
                    if ui.add_sized([ui.available_width(), 24.0], egui::Button::new(RichText::new(format!("▦ {}", name)).color(ACCENT()))).clicked() {
                        action = Some((i, 2, j));
                    }
                }
            });
            ui.add_space(6.0);
        }
        if let Some((i, kind, j)) = action {
            let p = self.plugins[i].clone();
            match kind {
                0 => match run_command(&p.folder, &p.commands[j], &self.doc) {
                    Ok((new, msgs)) => {
                        let _untrusted = UntrustedSource::begin();
                        self.doc = new;
                        self.sel.clear();
                        let shown = crate::script::bounded_join(&msgs, " · ", 400);
                        self.status = format!("{} “{}”{}", t("Plugin executed:"), p.commands[j].name, if shown.is_empty() { String::new() } else { format!(" · {}", shown) });
                    }
                    Err(e) => self.status = format!("{} {}", t("Plugin error:"), e),
                },
                1 => match run_exporter(&p.folder, &p.exporters[j], &self.doc) {
                    Ok(text) => {
                        let x = &p.exporters[j];
                        let ext = safe_extension(&x.extension);
                        let name = format!("{}.{}", crate::export::class_name(&self.doc.title), ext);
                        self.save_text_file(&x.name, &name, &ext, &text);
                    }
                    Err(e) => self.status = format!("{} {}", t("Plugin error:"), e),
                },
                _ => {
                    let f = &p.templates[j];
                    match std::fs::read_to_string(f).map_err(|e| e.to_string()).and_then(|s| serde_json::from_str::<Doc>(&s).map_err(|e| e.to_string())) {
                        Ok(d) => {
                            // Same as a custom template: leaves test mode and locks the backgrounds.
                            let _untrusted = UntrustedSource::begin();
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

    fn examples_folder() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("plugins")
    }

    #[test]
    fn loads_the_example_plugins() {
        for name in ["snap-to-grid", "text-to-csv", "extra-templates"] {
            let p = load(&examples_folder().join(name));
            assert!(p.error.is_none(), "{}: {:?}", name, p.error);
        }
    }

    #[test]
    fn command_snaps_to_grid() {
        let p = load(&examples_folder().join("snap-to-grid"));
        let mut doc = crate::model::template(0, 1920.0, 1080.0);
        doc.elements[1].x = 13.0;
        let (new, _) = run_command(&p.folder, &p.commands[0], &doc).expect("the command must work");
        assert_eq!(new.elements.len(), doc.elements.len());
        assert!(new.elements.iter().all(|e| e.x % 8.0 == 0.0 && e.y % 8.0 == 0.0));
    }

    #[test]
    fn csv_exporter() {
        let p = load(&examples_folder().join("text-to-csv"));
        let doc = crate::model::template(0, 1920.0, 1080.0);
        let csv = run_exporter(&p.folder, &p.exporters[0], &doc).expect("the exporter must work");
        assert!(csv.starts_with("element,text"));
        assert!(csv.contains("Play"));
    }

    #[test]
    fn paths_outside_plugin_rejected() {
        assert!(inside(Path::new("x"), "../fuera.rhai").is_err());
        assert!(inside(Path::new("x"), "C:/Windows/a.rhai").is_err() || cfg!(not(windows)));
        assert!(inside(Path::new("x"), "ok.rhai").is_ok());
    }

    #[test]
    fn unbounded_script_is_cut() {
        let dir = std::env::temp_dir().join("lienzo-plugin-bucle");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("bucle.rhai"), "fn run(d) { loop { } }").unwrap();
        let c = Action { name: "bucle".into(), description: String::new(), script: "bucle.rhai".into() };
        let doc = crate::model::template(5, 1920.0, 1080.0);
        assert!(run_command(&dir, &c, &doc).is_err());
    }

    #[test]
    fn old_spanish_manifest_and_function_names_still_work() {
        let dir = std::env::temp_dir().join(format!("lienzo-plugin-viejo-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("plugin.json"),
            r#"{ "nombre": "Plugin viejo", "version": "1.0.0", "autor": "yo", "descripcion": "d",
                 "comandos": [ { "nombre": "Cmd", "descripcion": "x", "script": "a.rhai" } ],
                 "exportadores": [ { "nombre": "Exp", "extension": "txt", "script": "b.rhai" } ],
                 "plantillas": [] }"#,
        )
        .unwrap();
        std::fs::write(dir.join("a.rhai"), "fn ejecutar(diseno) { diseno.title = \"Hecho\"; diseno }").unwrap();
        std::fs::write(dir.join("b.rhai"), "fn exportar(diseno) { diseno.title }").unwrap();
        let p = load(&dir);
        assert!(p.error.is_none(), "{:?}", p.error);
        assert_eq!((p.name.as_str(), p.author.as_str(), p.description.as_str()), ("Plugin viejo", "yo", "d"));
        assert_eq!((p.commands.len(), p.exporters.len()), (1, 1));
        assert_eq!((p.commands[0].name.as_str(), p.exporters[0].extension.as_str()), ("Cmd", "txt"));
        let doc = crate::model::template(0, 1920.0, 1080.0);
        let (new, _) = run_command(&p.folder, &p.commands[0], &doc).expect("the old function name `ejecutar` must be found");
        assert_eq!(new.title, "Hecho");
        assert_eq!(run_exporter(&p.folder, &p.exporters[0], &doc).expect("the old function name `exportar` must be found"), doc.title);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn manifests_with_the_old_spanish_keys_are_read_like_the_english_ones() {
        // Community items uploaded by app versions 1.0 - 1.3.3 still carry Spanish manifests.
        let m: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(examples_folder().join("snap-to-grid/plugin.json")).unwrap()).unwrap();
        let old = serde_json::json!({ "nombre": m["name"], "comandos": [{ "nombre": m["commands"][0]["name"], "script": m["commands"][0]["script"] }] });
        assert_eq!(manifest_key(&old, "name", "nombre"), manifest_key(&m, "name", "nombre"));
        assert_eq!(manifest_key(&old, "commands", "comandos").and_then(|c| c[0]["nombre"].as_str()), m["commands"][0]["name"].as_str());
        let parsed: Manifest = serde_json::from_value(old).unwrap();
        assert_eq!((parsed.name.as_str(), parsed.commands.len(), parsed.commands[0].name.as_str()), (m["name"].as_str().unwrap(), 1, m["commands"][0]["name"].as_str().unwrap()));
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn paths_escaping_the_folder() {
        let c = Path::new("plugins/x");
        for bad in ["", "../a.rhai", "a/../../b.rhai", "/etc/a.rhai", "\\Users\\a.rhai", "C:a.rhai", "C:\\a.rhai"] {
            assert!(inside(c, bad).is_err(), "should reject {:?}", bad);
        }
        for good in ["a.rhai", "sub/a.rhai", "./a.rhai"] {
            assert!(inside(c, good).is_ok(), "should accept {:?}", good);
        }
    }
}

#[cfg(test)]
mod sandbox_tests {
    use super::*;

    #[test]
    fn hostile_plugin_scripts_fail_fast() {
        let dir = std::env::temp_dir().join(format!("lienzo-plugin-sandbox-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let doc = crate::model::template(0, 1920.0, 1080.0);
        for (i, body) in ["let a = []; a.pad(300000000, 0); d", "let b = blob(1000000000); d", "import \"C:\\x\\m\" as m; d", "loop { } d"].into_iter().enumerate() {
            std::fs::write(dir.join(format!("h{}.rhai", i)), format!("fn run(d) {{ {} }}", body)).unwrap();
            let t = std::time::Instant::now();
            let r = call(&dir, &format!("h{}.rhai", i), &["run"], &doc);
            assert!(r.is_err(), "{}", body);
            assert!(t.elapsed() < std::time::Duration::from_secs(15), "{}", body);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod audit_tests {
    use super::*;

    fn temp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("lienzo-plugin-audit-{}-{}", tag, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn local_plugin_with_dangerous_extension_is_refused() {
        let dir = temp("ext");
        std::fs::write(dir.join("plugin.json"), r#"{"name":"E","exporters":[{"name":"ok","extension":"csv","script":"a.rhai"},{"name":"bad","extension":"bat","script":"a.rhai"},{"name":"bad2","extension":"../x","script":"a.rhai"}]}"#).unwrap();
        std::fs::write(dir.join("a.rhai"), "fn export_text(d){ \"\" }").unwrap();
        let p = load(&dir);
        assert_eq!(p.exporters.len(), 1);
        assert_eq!(p.exporters[0].extension, "csv");
        assert!(p.error.as_deref().is_some_and(|e| e.contains("extension not allowed")), "{:?}", p.error);
        assert_eq!(safe_extension("BAT"), "txt");
        assert_eq!(safe_extension(".CSV"), "csv");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn print_flood_and_nested_growth_in_plugins_fail() {
        let dir = temp("flood");
        let doc = crate::model::template(0, 1920.0, 1080.0);
        for (i, body) in ["loop { print(\"x\"); } d", "loop { log(\"x\"); } d", "let a = []; loop { a.push(a.clone()); } d", "let s = \"abc\"; loop { s += s; } d"].into_iter().enumerate() {
            std::fs::write(dir.join(format!("f{}.rhai", i)), format!("fn run(d) {{ {} }}", body)).unwrap();
            let t = std::time::Instant::now();
            let r = call(&dir, &format!("f{}.rhai", i), &["run"], &doc);
            assert!(r.is_err(), "{}", body);
            assert!(t.elapsed() < std::time::Duration::from_secs(8), "{}", body);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn command_output_with_unsafe_image_is_rejected() {
        let dir = temp("out");
        std::fs::write(dir.join("a.rhai"), "fn run(d) { d.elements[0].image = \"C:/Users/x/secret.png\"; d }").unwrap();
        std::fs::write(dir.join("b.rhai"), "fn run(d) { d.elements[0].text = \"fine\"; d }").unwrap();
        let doc = crate::model::template(0, 1920.0, 1080.0);
        let bad = Action { name: "a".into(), description: String::new(), script: "a.rhai".into() };
        let good = Action { name: "b".into(), description: String::new(), script: "b.rhai".into() };
        assert!(run_command(&dir, &bad, &doc).is_err());
        assert!(run_command(&dir, &good, &doc).is_ok());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
