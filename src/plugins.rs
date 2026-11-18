//! Plugins (complementos) de Lienzo UI.
//!
//! Un plugin es una carpeta con un `plugin.json` dentro de alguna carpeta de plugins:
//!   * `plugins/` junto al ejecutable,
//!   * `plugins/` en la carpeta de trabajo (útil al desarrollar desde el repositorio),
//!   * `%APPDATA%/EditorGUI/plugins` (o su equivalente en macOS/Linux).
//!
//! Puede aportar:
//!   * comandos: scripts Rhai con `fn ejecutar(diseno) { ...; diseno }` que reciben el diseño como
//!     mapa y devuelven el diseño modificado (se puede deshacer con Ctrl+Z);
//!   * exportadores: `fn exportar(diseno) { ... }` que devuelven el texto de un archivo nuevo;
//!   * plantillas: diseños `.editorgui.json` que aparecen en la pestaña Plugins.
//!
//! Los scripts corren aislados: sin acceso a archivos, red ni procesos, y con límites de operaciones.
//! Formato completo y ejemplos: docs/PLUGINS.md y la carpeta plugins/ del repositorio.

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

/// Carpetas donde se buscan plugins (sin repetir).
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

/// Carpeta de plugins del usuario (se crea al abrirla desde la app).
pub fn carpeta_usuario() -> PathBuf {
    crate::library::root_dir().parent().map(|d| d.join("plugins")).unwrap_or_else(|| PathBuf::from("plugins"))
}

/// Ruta de un archivo del plugin, sin salir de su carpeta.
fn dentro(carpeta: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel_path = Path::new(rel);
    // Solo nombres normales: en Windows «\carpeta» o «C:x» no son absolutas pero `join` las saca de la carpeta.
    let normal = rel_path.components().all(|c| matches!(c, std::path::Component::Normal(_) | std::path::Component::CurDir));
    if rel.is_empty() || rel_path.is_absolute() || rel_path.has_root() || !normal {
        return Err(format!("ruta no permitida en el plugin: {}", rel));
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

/// Motor Rhai aislado y con límites, más una salida de mensajes (`log("…")`).
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

/// Ejecuta un comando: devuelve el diseño nuevo y los mensajes del script.
pub fn ejecutar_comando(carpeta: &Path, c: &Accion, doc: &Doc) -> Result<(Doc, Vec<String>), String> {
    let (r, msgs) = llamar(carpeta, &c.script, "ejecutar", doc)?;
    // Rhai trabaja con f64 y el modelo con f32: se pasa por JSON, que convierte los números.
    let json: serde_json::Value = rhai::serde::from_dynamic(&r).map_err(|e| format!("{}: el diseño devuelto no es válido ({})", c.script, e))?;
    let mut nuevo: Doc = serde_json::from_value(json).map_err(|e| format!("{}: el diseño devuelto no es válido ({})", c.script, e))?;
    nuevo.migrate_groups();
    Ok((nuevo, msgs))
}

/// Ejecuta un exportador: devuelve el texto del archivo.
pub fn ejecutar_exportador(carpeta: &Path, x: &Exportador, doc: &Doc) -> Result<String, String> {
    let (r, _) = llamar(carpeta, &x.script, "exportar", doc)?;
    r.into_string().map_err(|tipo| format!("{}: exportar() debe devolver texto, devolvió {}", x.script, tipo))
}

impl EditorApp {
    pub fn tab_plugins(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(RichText::new(t("Plugins")).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button(t("Recargar")).clicked() {
                    self.plugins = cargar_todos();
                    self.status = format!("{}: {}", t("Plugins cargados"), self.plugins.len());
                }
                if ui.small_button(t("Abrir carpeta")).on_hover_text(carpeta_usuario().display().to_string()).clicked() {
                    let c = carpeta_usuario();
                    let _ = std::fs::create_dir_all(&c);
                    let _ = std::process::Command::new(if cfg!(windows) { "explorer" } else if cfg!(target_os = "macos") { "open" } else { "xdg-open" }).arg(&c).spawn();
                }
            });
        });
        ui.label(RichText::new(t("Añaden comandos, exportadores y plantillas. Guía: docs/PLUGINS.md")).small().color(MUTED()));
        ui.add_space(6.0);
        if self.plugins.is_empty() {
            ui.label(RichText::new(t("No hay plugins instalados. Copia una carpeta con plugin.json en la carpeta de plugins y pulsa Recargar.")).color(MUTED()));
            return;
        }
        let mut accion: Option<(usize, u8, usize)> = None; // (plugin, tipo 0 comando / 1 exportador / 2 plantilla, índice)
        for (i, p) in self.plugins.iter().enumerate() {
            egui::Frame::new().fill(crate::theme::PANEL_DARK()).corner_radius(6.0).inner_margin(8.0).show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label(RichText::new(format!("{}  {}", p.nombre, p.version)).strong());
                if !p.autor.is_empty() {
                    ui.label(RichText::new(format!("{} {}", t("por"), p.autor)).small().color(MUTED()));
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
                        self.status = format!("{} «{}»{}", t("Plugin ejecutado:"), p.comandos[j].nombre, if msgs.is_empty() { String::new() } else { format!(" · {}", msgs.join(" · ")) });
                    }
                    Err(e) => self.status = format!("{} {}", t("Error del plugin:"), e),
                },
                1 => match ejecutar_exportador(&p.carpeta, &p.exportadores[j], &self.doc) {
                    Ok(texto) => {
                        let x = &p.exportadores[j];
                        let nombre = format!("{}.{}", crate::export::class_name(&self.doc.title), x.extension);
                        self.save_text_file(&x.nombre, &nombre, &x.extension, &texto);
                    }
                    Err(e) => self.status = format!("{} {}", t("Error del plugin:"), e),
                },
                _ => {
                    let f = &p.plantillas[j];
                    match std::fs::read_to_string(f).map_err(|e| e.to_string()).and_then(|s| serde_json::from_str::<Doc>(&s).map_err(|e| e.to_string())) {
                        Ok(d) => {
                            // Igual que una plantilla propia: sale del modo prueba y bloquea los fondos.
                            self.load_user_template(&d);
                            self.status = format!("{} {}", t("Plantilla del plugin aplicada (Ctrl+Z para deshacer):"), f.display());
                        }
                        Err(e) => self.status = format!("{} {}", t("Error del plugin:"), e),
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
        let (nuevo, _) = ejecutar_comando(&p.carpeta, &p.comandos[0], &doc).expect("el comando debe funcionar");
        assert_eq!(nuevo.elements.len(), doc.elements.len());
        assert!(nuevo.elements.iter().all(|e| e.x % 8.0 == 0.0 && e.y % 8.0 == 0.0));
    }

    #[test]
    fn exportador_csv() {
        let p = cargar(&carpeta_ejemplos().join("textos-csv"));
        let doc = crate::model::template(0, 1920.0, 1080.0);
        let csv = ejecutar_exportador(&p.carpeta, &p.exportadores[0], &doc).expect("el exportador debe funcionar");
        assert!(csv.starts_with("elemento,texto"));
        assert!(csv.contains("Jugar"));
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
            assert!(dentro(c, mala).is_err(), "debería rechazar {:?}", mala);
        }
        for buena in ["a.rhai", "sub/a.rhai", "./a.rhai"] {
            assert!(dentro(c, buena).is_ok(), "debería aceptar {:?}", buena);
        }
    }
}
