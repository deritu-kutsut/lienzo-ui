//! In-app updates.
//!
//! GitHub Releases is the only source: the itch.io zip points to the same file, so whoever
//! downloaded the app from GitHub or itch.io updates the same way. The app asks for the latest release,
//! downloads the zip, checks its SHA-256 (the one GitHub publishes) and replaces its own .exe.
//! If anything cannot be verified (another OS, no write permission, no hash), it offers to open
//! the download page instead of installing anything.

use crate::app::EditorApp;
use crate::i18n::t;
use crate::theme::{ACCENT, MUTED, OK};
use eframe::egui::{self, Color32, RichText};
use sha2::{Digest, Sha256};
use std::io::Read;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

pub const VERSION_ACTUAL: &str = env!("CARGO_PKG_VERSION");
const API: &str = "https://api.github.com/repos/deritu-kutsut/lienzo-ui/releases/latest";
const PREFIJO_DESCARGA: &str = "https://github.com/deritu-kutsut/lienzo-ui/releases/download/";
pub const PAGINA_RELEASES: &str = "https://github.com/deritu-kutsut/lienzo-ui/releases/latest";
const MAX_ZIP: u64 = 150 * 1024 * 1024;
const MAX_EXE: u64 = 200 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub struct Novedad {
    pub version: String,
    pub notas: String,
    pub url_zip: String,
    /// SHA-256 in hex; empty if GitHub does not provide it (then it is not installed automatically).
    pub sha256: String,
}

pub enum Estado {
    Nada,
    Buscando,
    AlDia,
    Disponible(Novedad),
    Descargando(Novedad),
    /// Installed version and path of the NEW executable (after renaming, `current_exe()` points to the .viejo).
    Instalada(String, std::path::PathBuf),
    Error(String),
}

pub enum Mensaje {
    Resultado(Result<Option<Novedad>, String>),
    Instalada(Result<(String, std::path::PathBuf), String>),
}

pub struct Actualizador {
    pub estado: Estado,
    pub abierta: bool,
    rx: Option<Receiver<Mensaje>>,
}

impl Actualizador {
    pub fn new() -> Self {
        Self { estado: Estado::Nada, abierta: false, rx: None }
    }
}

// ----------------------------------------------------------------- versions

/// "v1.2.3" or "1.2.3"; versions with a suffix (1.2.3-beta) are not offered.
pub fn parsear_version(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.trim().trim_start_matches('v');
    let mut it = s.split('.');
    let v = (it.next()?.parse().ok()?, it.next()?.parse().ok()?, it.next()?.parse().ok()?);
    if it.next().is_some() { None } else { Some(v) }
}

pub fn es_mas_nueva(candidata: &str, actual: &str) -> bool {
    match (parsear_version(candidata), parsear_version(actual)) {
        (Some(c), Some(a)) => c > a,
        _ => false,
    }
}

// ------------------------------------------------------------------- release

/// Release notes as plain text: no images or Markdown marks, max. 4000 characters.
pub fn limpiar_notas(md: &str) -> String {
    let mut out = String::new();
    for l in md.lines() {
        let t = l.trim();
        if t.starts_with("![") {
            continue;
        }
        let t = t.trim_start_matches('#').trim_start_matches('>').trim();
        let t = t.strip_prefix("- ").map(|r| format!("• {}", r)).unwrap_or_else(|| t.to_string());
        let t = t.replace("**", "").replace('`', "");
        out.push_str(&t);
        out.push('\n');
    }
    out.trim().chars().take(4000).collect()
}

/// Parses the `releases/latest` response. `None` = you are already up to date.
pub fn parsear_release(json: &str, actual: &str) -> Result<Option<Novedad>, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|_| "Invalid response from GitHub.".to_string())?;
    if v["draft"].as_bool() == Some(true) || v["prerelease"].as_bool() == Some(true) {
        return Ok(None);
    }
    let tag = v["tag_name"].as_str().ok_or("The release has no version.")?;
    if parsear_version(tag).is_none() {
        return Err("Unrecognized release version.".into());
    }
    if !es_mas_nueva(tag, actual) {
        return Ok(None);
    }
    let version = tag.trim_start_matches('v').to_string();
    let nombre = format!("LienzoUI-windows-{}.zip", version);
    let mut url_zip = String::new();
    let mut sha256 = String::new();
    if let Some(a) = v["assets"].as_array().and_then(|l| l.iter().find(|a| a["name"].as_str() == Some(&nombre))) {
        let url = a["browser_download_url"].as_str().unwrap_or("");
        if url.starts_with(PREFIJO_DESCARGA) && url.chars().all(|c| c.is_ascii_graphic()) {
            url_zip = url.to_string();
            let d = a["digest"].as_str().unwrap_or("").strip_prefix("sha256:").unwrap_or("").to_ascii_lowercase();
            if d.len() == 64 && d.chars().all(|c| c.is_ascii_hexdigit()) {
                sha256 = d;
            }
        }
    }
    let notas = limpiar_notas(v["body"].as_str().unwrap_or(""));
    Ok(Some(Novedad { version, notas, url_zip, sha256 }))
}

fn agente(segundos: u64) -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(segundos))).user_agent(format!("LienzoUI-Actualizador/{}", VERSION_ACTUAL)).build().into()
}

/// A single GET request to GitHub; no user data is sent.
fn buscar() -> Result<Option<Novedad>, String> {
    let mut r = agente(15).get(API).header("Accept", "application/vnd.github+json").call().map_err(|_| "Could not connect to GitHub.".to_string())?;
    if r.status().as_u16() == 404 {
        return Ok(None);
    }
    if !r.status().is_success() {
        return Err(format!("GitHub responded {}.", r.status().as_u16()));
    }
    let cuerpo = r.body_mut().with_config().limit(2 * 1024 * 1024).read_to_string().map_err(|_| "Respuesta de GitHub demasiado grande o ilegible.".to_string())?;
    parsear_release(&cuerpo, VERSION_ACTUAL)
}

// ------------------------------------------------------------------ install

pub fn verificar_sha256(bytes: &[u8], esperado: &str) -> Result<(), String> {
    let h = Sha256::digest(bytes);
    let hex: String = h.iter().map(|b| format!("{:02x}", b)).collect();
    if !esperado.is_empty() && hex.eq_ignore_ascii_case(esperado) { Ok(()) } else { Err("The download doesn't match the published checksum; not installing.".into()) }
}

/// Extracts only the executable from the zip (`LienzoUI-<v>/lienzo-ui.exe`); no other name is written to disk.
pub fn extraer_exe(zip: &[u8], version: &str) -> Result<Vec<u8>, String> {
    let mut a = zip::ZipArchive::new(std::io::Cursor::new(zip)).map_err(|_| "The downloaded zip is not valid.".to_string())?;
    let nombre = format!("LienzoUI-{}/lienzo-ui.exe", version);
    // Compress-Archive in old Windows PowerShell stores paths with "\": they are compared normalized.
    let idx = (0..a.len()).find(|&i| a.by_index_raw(i).is_ok_and(|f| f.name().is_ok_and(|n| n.replace('\\', "/") == nombre))).ok_or("The zip does not contain lienzo-ui.exe.")?;
    let mut f = a.by_index(idx).map_err(|_| "The zip could not be read.".to_string())?;
    if f.size() > MAX_EXE {
        return Err("The executable in the zip is too large.".into());
    }
    let mut out = Vec::with_capacity(f.size() as usize);
    f.by_ref().take(MAX_EXE + 1).read_to_end(&mut out).map_err(|_| "No se pudo descomprimir.".to_string())?;
    if out.len() as u64 > MAX_EXE || out.len() < 4096 || &out[..2] != b"MZ" {
        return Err("The executable in the zip does not look valid.".into());
    }
    Ok(out)
}

fn ruta_con_extension(exe: &std::path::Path, ext: &str) -> std::path::PathBuf {
    let mut s = exe.as_os_str().to_os_string();
    s.push(ext);
    s.into()
}

/// Swaps the running executable for the new one. On Windows a running .exe cannot be deleted but can
/// be renamed: the old one stays as ".viejo" (deleted on startup) and, if anything fails, it is rolled back.
pub fn reemplazar_exe(exe: &std::path::Path, nuevo: &[u8]) -> Result<(), String> {
    let tmp = ruta_con_extension(exe, ".nuevo");
    let viejo = ruta_con_extension(exe, ".viejo");
    let _ = std::fs::remove_file(&viejo);
    std::fs::write(&tmp, nuevo).map_err(|_| "No permission to write next to the app. Download the new version manually.".to_string())?;
    if std::fs::rename(exe, &viejo).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return Err("Could not replace the app (protected folder?). Download the new version manually.".into());
    }
    if std::fs::rename(&tmp, exe).is_err() {
        let _ = std::fs::rename(&viejo, exe);
        let _ = std::fs::remove_file(&tmp);
        return Err("Could not install the new version; keeping the current one.".into());
    }
    Ok(())
}

fn descargar_e_instalar(n: &Novedad) -> Result<(String, std::path::PathBuf), String> {
    if !cfg!(windows) {
        return Err("Automatic install only exists on Windows. Download the new version manually.".into());
    }
    if n.url_zip.is_empty() || n.sha256.is_empty() {
        return Err("This release can't be verified; download it manually.".into());
    }
    let exe = std::env::current_exe().map_err(|_| "No se encuentra la ruta de la app.".to_string())?;
    let mut r = agente(300).get(&n.url_zip).call().map_err(|_| "Could not download the update.".to_string())?;
    if !r.status().is_success() {
        return Err(format!("The download responded {}.", r.status().as_u16()));
    }
    let zip = r.body_mut().with_config().limit(MAX_ZIP).read_to_vec().map_err(|_| "The download is too large or was cut off.".to_string())?;
    verificar_sha256(&zip, &n.sha256)?;
    let bytes = extraer_exe(&zip, &n.version)?;
    reemplazar_exe(&exe, &bytes)?;
    Ok((n.version.clone(), exe))
}

/// Deletes the old .exe left by the previous update.
pub fn limpiar_residuos() {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::fs::remove_file(ruta_con_extension(&exe, ".viejo"));
        let _ = std::fs::remove_file(ruta_con_extension(&exe, ".nuevo"));
    }
}

fn reiniciar(ctx: &egui::Context, exe: &std::path::Path) {
    if std::process::Command::new(exe).spawn().is_ok() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

// ----------------------------------------------------------------------- UI

impl EditorApp {
    /// Starts the check in the background (does nothing if one is already running).
    pub fn actualizador_buscar(&mut self) {
        if matches!(self.actualizador.estado, Estado::Buscando | Estado::Descargando(_)) {
            return;
        }
        let (tx, rx) = channel();
        self.actualizador.rx = Some(rx);
        self.actualizador.estado = Estado::Buscando;
        std::thread::spawn(move || {
            let _ = tx.send(Mensaje::Resultado(buscar()));
        });
    }

    /// Automatic check on startup (can be disabled in the updates window).
    pub fn actualizador_inicio(&mut self) {
        limpiar_residuos();
        if !self.settings.solo_lectura && !self.settings.no_buscar_actualizaciones && self.screenshot.is_none() {
            self.actualizador_buscar();
        }
    }

    fn actualizador_instalar(&mut self, n: Novedad) {
        let (tx, rx) = channel();
        self.actualizador.rx = Some(rx);
        self.actualizador.estado = Estado::Descargando(n.clone());
        std::thread::spawn(move || {
            let _ = tx.send(Mensaje::Instalada(descargar_e_instalar(&n)));
        });
    }

    /// New version to announce in the header (None if there is none or it was skipped).
    pub fn actualizacion_para_boton(&self) -> Option<&Novedad> {
        match &self.actualizador.estado {
            Estado::Disponible(n) if n.version != self.settings.version_omitida => Some(n),
            _ => None,
        }
    }

    /// Header button: only shown when there is a new version.
    pub fn actualizador_boton(&mut self, ui: &mut egui::Ui) {
        let etiqueta = match &self.actualizador.estado {
            Estado::Instalada(..) => Some((t("Restart to update"), OK())),
            _ => self.actualizacion_para_boton().map(|n| (format!("{} {}", t("Update to"), n.version), ACCENT())),
        };
        if let Some((texto, color)) = etiqueta {
            let b = egui::Button::new(RichText::new(texto).strong().color(Color32::WHITE)).fill(color).corner_radius(6.0);
            if ui.add(b).on_hover_text(t("A new version of Lienzo UI is available")).clicked() {
                self.actualizador.abierta = true;
            }
        }
    }

    pub fn actualizador_window(&mut self, ctx: &egui::Context) {
        // Messages from the background thread.
        let mensajes: Vec<Mensaje> = self.actualizador.rx.as_ref().map(|rx| rx.try_iter().collect()).unwrap_or_default();
        for m in mensajes {
            match m {
                Mensaje::Resultado(Ok(Some(n))) => self.actualizador.estado = Estado::Disponible(n),
                Mensaje::Resultado(Ok(None)) => self.actualizador.estado = Estado::AlDia,
                Mensaje::Resultado(Err(e)) => self.actualizador.estado = Estado::Error(e),
                Mensaje::Instalada(Ok((v, exe))) => {
                    self.actualizador.estado = Estado::Instalada(v, exe);
                    self.actualizador.abierta = true;
                }
                Mensaje::Instalada(Err(e)) => self.actualizador.estado = Estado::Error(e),
            }
        }
        if matches!(self.actualizador.estado, Estado::Buscando | Estado::Descargando(_)) {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
        if !self.actualizador.abierta {
            return;
        }
        let mut abierta = true;
        let mut instalar: Option<Novedad> = None;
        let mut buscar_otra = false;
        let mut omitir: Option<String> = None;
        let mut reinicio: Option<std::path::PathBuf> = None;
        let mut cambio = false;
        egui::Window::new(t("Updates")).open(&mut abierta).collapsible(false).resizable(false).default_width(440.0).show(ctx, |ui| {
            ui.label(format!("{} {}", t("Installed version:"), VERSION_ACTUAL));
            ui.add_space(6.0);
            match &self.actualizador.estado {
                Estado::Nada => {
                    if ui.button(t("Check for updates")).clicked() {
                        buscar_otra = true;
                    }
                }
                Estado::Buscando => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(t("Checking for updates…"));
                    });
                }
                Estado::AlDia => {
                    ui.label(RichText::new(t("You have the latest version.")).color(OK()));
                    if ui.button(t("Check again")).clicked() {
                        buscar_otra = true;
                    }
                }
                Estado::Disponible(n) => {
                    ui.label(RichText::new(format!("{} {}", t("A new version is available:"), n.version)).strong().color(ACCENT()));
                    if !n.notas.is_empty() {
                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            ui.label(RichText::new(&n.notas).small().color(MUTED()));
                        });
                    }
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let auto = cfg!(windows) && !n.url_zip.is_empty() && !n.sha256.is_empty();
                        if auto {
                            if ui.add(egui::Button::new(RichText::new(t("Update now")).strong().color(Color32::WHITE)).fill(ACCENT())).clicked() {
                                instalar = Some(n.clone());
                            }
                        }
                        if ui.button(t(if auto { "Manual download" } else { "Open download page" })).clicked() {
                            let _ = webbrowser::open(PAGINA_RELEASES);
                        }
                        if ui.button(t("Skip this version")).clicked() {
                            omitir = Some(n.version.clone());
                        }
                    });
                    if !(cfg!(windows) && !n.sha256.is_empty()) {
                        ui.label(RichText::new(t("This version must be downloaded manually (it can't be verified from the app).")).small().color(MUTED()));
                    }
                }
                Estado::Descargando(n) => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(format!("{} {}…", t("Downloading and installing version"), n.version));
                    });
                    ui.label(RichText::new(t("Don't close the app in the meantime.")).small().color(MUTED()));
                }
                Estado::Instalada(v, exe) => {
                    ui.label(RichText::new(format!("{} {}", t("Version installed successfully:"), v)).strong().color(OK()));
                    ui.label(RichText::new(t("Save your design: restarting closes the current window.")).small().color(MUTED()));
                    if ui.add(egui::Button::new(RichText::new(t("Restart now")).strong().color(Color32::WHITE)).fill(OK())).clicked() {
                        reinicio = Some(exe.clone());
                    }
                }
                Estado::Error(e) => {
                    ui.label(RichText::new(t(e)).color(crate::theme::WARN()));
                    ui.horizontal(|ui| {
                        if ui.button(t("Retry")).clicked() {
                            buscar_otra = true;
                        }
                        if ui.button(t("Open download page")).clicked() {
                            let _ = webbrowser::open(PAGINA_RELEASES);
                        }
                    });
                }
            }
            ui.add_space(8.0);
            ui.separator();
            let mut buscar_al_abrir = !self.settings.no_buscar_actualizaciones;
            if ui.checkbox(&mut buscar_al_abrir, t("Check for updates on startup")).changed() {
                self.settings.no_buscar_actualizaciones = !buscar_al_abrir;
                cambio = true;
            }
            ui.label(RichText::new(t("Only github.com is queried; none of your data is sent.")).small().color(MUTED()));
        });
        if cambio {
            self.settings.save();
        }
        if let Some(v) = omitir {
            self.settings.version_omitida = v;
            self.settings.save();
            abierta = false;
        }
        if let Some(n) = instalar {
            self.actualizador_instalar(n);
        }
        if buscar_otra {
            self.actualizador.estado = Estado::Nada;
            self.actualizador_buscar();
        }
        if let Some(exe) = reinicio {
            reiniciar(ctx, &exe);
        }
        self.actualizador.abierta = abierta;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn zip_con(nombre: &str, datos: &[u8]) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            z.start_file(nombre, zip::write::SimpleFileOptions::default()).unwrap();
            z.write_all(datos).unwrap();
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    fn exe_falso() -> Vec<u8> {
        let mut v = b"MZ".to_vec();
        v.resize(8192, 7);
        v
    }

    #[test]
    fn limpia_las_notas() {
        let n = limpiar_notas("![x](https://a/b.png)

## Español
**Hola** `app`
- punto
> aviso");
        assert_eq!(n, "Español
Hola app
• punto
aviso");
        assert!(limpiar_notas(&"a".repeat(9000)).chars().count() <= 4000);
    }

    #[test]
    fn compara_versiones() {
        assert!(es_mas_nueva("v1.2.0", "1.1.9"));
        assert!(es_mas_nueva("1.10.0", "1.9.9"));
        assert!(!es_mas_nueva("1.1.1", "1.1.1"));
        assert!(!es_mas_nueva("1.0.9", "1.1.1"));
        assert!(!es_mas_nueva("1.2.0-beta", "1.1.1"));
        assert!(!es_mas_nueva("basura", "1.1.1"));
        assert!(!es_mas_nueva("1.2", "1.1.1"));
    }

    #[test]
    fn interpreta_la_release() {
        let h = "a".repeat(64);
        let json = format!(
            r#"{{"tag_name":"v1.2.0","body":"Novedades","assets":[{{"name":"LienzoUI-windows-1.2.0.zip","browser_download_url":"{}v1.2.0/LienzoUI-windows-1.2.0.zip","digest":"sha256:{}"}}]}}"#,
            PREFIJO_DESCARGA, h
        );
        let n = parsear_release(&json, "1.1.1").unwrap().unwrap();
        assert_eq!((n.version.as_str(), n.sha256.as_str()), ("1.2.0", h.as_str()));
        assert!(parsear_release(&json, "1.2.0").unwrap().is_none());
        assert!(parsear_release("no es json", "1.1.1").is_err());
    }

    #[test]
    fn rechaza_descargas_de_otro_origen_o_sin_huella() {
        let json = r#"{"tag_name":"v1.2.0","assets":[{"name":"LienzoUI-windows-1.2.0.zip","browser_download_url":"https://malo.example/x.zip","digest":"sha256:abc"}]}"#;
        let n = parsear_release(json, "1.1.1").unwrap().unwrap();
        assert!(n.url_zip.is_empty() && n.sha256.is_empty());
        let draft = r#"{"tag_name":"v9.0.0","draft":true}"#;
        assert!(parsear_release(draft, "1.1.1").unwrap().is_none());
        let pre = r#"{"tag_name":"v9.0.0","prerelease":true}"#;
        assert!(parsear_release(pre, "1.1.1").unwrap().is_none());
    }

    #[test]
    fn verifica_la_huella() {
        let datos = b"hola";
        let hex: String = Sha256::digest(datos).iter().map(|b| format!("{:02x}", b)).collect();
        assert!(verificar_sha256(datos, &hex).is_ok());
        assert!(verificar_sha256(datos, &hex.to_uppercase()).is_ok());
        assert!(verificar_sha256(b"otra cosa", &hex).is_err());
        assert!(verificar_sha256(datos, "").is_err());
    }

    #[test]
    fn extrae_solo_el_exe_esperado() {
        let ok = zip_con("LienzoUI-1.2.0/lienzo-ui.exe", &exe_falso());
        assert_eq!(extraer_exe(&ok, "1.2.0").unwrap().len(), 8192);
        assert!(extraer_exe(&zip_con("LienzoUI-1.2.0/otro.exe", &exe_falso()), "1.2.0").is_err());
        assert_eq!(extraer_exe(&zip_con("LienzoUI-1.2.0\\lienzo-ui.exe", &exe_falso()), "1.2.0").unwrap().len(), 8192);
        assert!(extraer_exe(&zip_con("../lienzo-ui.exe", &exe_falso()), "1.2.0").is_err());
        assert!(extraer_exe(&zip_con("LienzoUI-1.2.0/lienzo-ui.exe", b"MZ corto"), "1.2.0").is_err());
        assert!(extraer_exe(&zip_con("LienzoUI-1.2.0/lienzo-ui.exe", &vec![1u8; 8192]), "1.2.0").is_err());
        assert!(extraer_exe(b"no es un zip", "1.2.0").is_err());
    }

    #[test]
    fn reemplaza_el_exe_y_deja_el_viejo() {
        let dir = std::env::temp_dir().join(format!("lienzo-act-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("Lienzo UI.exe");
        std::fs::write(&exe, b"version vieja").unwrap();
        reemplazar_exe(&exe, &exe_falso()).unwrap();
        assert_eq!(std::fs::read(&exe).unwrap().len(), 8192);
        assert_eq!(std::fs::read(ruta_con_extension(&exe, ".viejo")).unwrap(), b"version vieja");
        assert!(!ruta_con_extension(&exe, ".nuevo").exists());
        // a second update overwrites the .viejo without failing
        reemplazar_exe(&exe, &exe_falso()).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Needs network: downloads the real GitHub release, checks its hash and extracts the .exe (`cargo test -- --ignored`).
    #[test]
    #[ignore]
    fn con_la_release_real_de_github() {
        let mut r = agente(15).get(API).header("Accept", "application/vnd.github+json").call().unwrap();
        let json = r.body_mut().read_to_string().unwrap();
        let n = parsear_release(&json, "0.0.1").unwrap().expect("there should be a version newer than 0.0.1");
        assert!(!n.sha256.is_empty() && !n.url_zip.is_empty(), "la release debe traer URL y huella: {:?}", n);
        let mut d = agente(300).get(&n.url_zip).call().unwrap();
        let zip = d.body_mut().with_config().limit(MAX_ZIP).read_to_vec().unwrap();
        verificar_sha256(&zip, &n.sha256).unwrap();
        let exe = extraer_exe(&zip, &n.version).unwrap();
        println!("release {} · zip {} B · exe {} B", n.version, zip.len(), exe.len());
    }
}
