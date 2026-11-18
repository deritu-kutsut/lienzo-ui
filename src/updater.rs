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

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const API: &str = "https://api.github.com/repos/deritu-kutsut/lienzo-ui/releases/latest";
const DOWNLOAD_PREFIX: &str = "https://github.com/deritu-kutsut/lienzo-ui/releases/download/";
pub const RELEASES_PAGE: &str = "https://github.com/deritu-kutsut/lienzo-ui/releases/latest";
const MAX_ZIP: u64 = 150 * 1024 * 1024;
const MAX_EXE: u64 = 200 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq)]
pub struct Release {
    pub version: String,
    pub notes: String,
    pub url_zip: String,
    /// SHA-256 in hex; empty if GitHub does not provide it (then it is not installed automatically).
    pub sha256: String,
}

pub enum State {
    Nothing,
    Searching,
    UpToDate,
    Available(Release),
    Downloading(Release),
    /// Installed version and path of the NEW executable (after renaming, `current_exe()` points to the .old).
    Installed(String, std::path::PathBuf),
    Error(String),
}

pub enum Message {
    Result(Result<Option<Release>, String>),
    Installed(Result<(String, std::path::PathBuf), String>),
}

pub struct Updater {
    pub state: State,
    pub open: bool,
    rx: Option<Receiver<Message>>,
}

impl Updater {
    pub fn new() -> Self {
        Self { state: State::Nothing, open: false, rx: None }
    }
}

// ----------------------------------------------------------------- versions

/// "v1.2.3" or "1.2.3"; versions with a suffix (1.2.3-beta) are not offered.
pub fn parse_version(s: &str) -> Option<(u32, u32, u32)> {
    let s = s.trim().trim_start_matches('v');
    let mut it = s.split('.');
    let v = (it.next()?.parse().ok()?, it.next()?.parse().ok()?, it.next()?.parse().ok()?);
    if it.next().is_some() { None } else { Some(v) }
}

pub fn is_newer(candidate: &str, current: &str) -> bool {
    match (parse_version(candidate), parse_version(current)) {
        (Some(c), Some(a)) => c > a,
        _ => false,
    }
}

// ------------------------------------------------------------------- release

/// Release notes as plain text: no images or Markdown marks, max. 4000 characters.
pub fn clean_notes(md: &str) -> String {
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
pub fn parse_release(json: &str, current: &str) -> Result<Option<Release>, String> {
    let v: serde_json::Value = serde_json::from_str(json).map_err(|_| "Invalid response from GitHub.".to_string())?;
    if v["draft"].as_bool() == Some(true) || v["prerelease"].as_bool() == Some(true) {
        return Ok(None);
    }
    let tag = v["tag_name"].as_str().ok_or("The release has no version.")?;
    if parse_version(tag).is_none() {
        return Err("Unrecognized release version.".into());
    }
    if !is_newer(tag, current) {
        return Ok(None);
    }
    let version = tag.trim_start_matches('v').to_string();
    let name = format!("LienzoUI-windows-{}.zip", version);
    let mut url_zip = String::new();
    let mut sha256 = String::new();
    if let Some(a) = v["assets"].as_array().and_then(|l| l.iter().find(|a| a["name"].as_str() == Some(&name))) {
        let url = a["browser_download_url"].as_str().unwrap_or("");
        if url.starts_with(DOWNLOAD_PREFIX) && url.chars().all(|c| c.is_ascii_graphic()) {
            url_zip = url.to_string();
            let d = a["digest"].as_str().unwrap_or("").strip_prefix("sha256:").unwrap_or("").to_ascii_lowercase();
            if d.len() == 64 && d.chars().all(|c| c.is_ascii_hexdigit()) {
                sha256 = d;
            }
        }
    }
    let notes = clean_notes(v["body"].as_str().unwrap_or(""));
    Ok(Some(Release { version, notes, url_zip, sha256 }))
}

fn agent(seconds: u64) -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(seconds))).user_agent(format!("LienzoUI-Updater/{}", CURRENT_VERSION)).build().into()
}

/// A single GET request to GitHub; no user data is sent.
fn search() -> Result<Option<Release>, String> {
    let mut r = agent(15).get(API).header("Accept", "application/vnd.github+json").call().map_err(|_| "Could not connect to GitHub.".to_string())?;
    if r.status().as_u16() == 404 {
        return Ok(None);
    }
    if !r.status().is_success() {
        return Err(format!("GitHub responded {}.", r.status().as_u16()));
    }
    let body = r.body_mut().with_config().limit(2 * 1024 * 1024).read_to_string().map_err(|_| "Invalid response from GitHub.".to_string())?;
    parse_release(&body, CURRENT_VERSION)
}

// ------------------------------------------------------------------ install

pub fn verify_sha256(bytes: &[u8], expected: &str) -> Result<(), String> {
    let h = Sha256::digest(bytes);
    let hex: String = h.iter().map(|b| format!("{:02x}", b)).collect();
    if !expected.is_empty() && hex.eq_ignore_ascii_case(expected) { Ok(()) } else { Err("The download doesn't match the published checksum; not installing.".into()) }
}

/// Extracts only the executable from the zip (`LienzoUI-<v>/lienzo-ui.exe`); no other name is written to disk.
pub fn extract_exe(zip: &[u8], version: &str) -> Result<Vec<u8>, String> {
    let mut a = zip::ZipArchive::new(std::io::Cursor::new(zip)).map_err(|_| "The downloaded zip is not valid.".to_string())?;
    let name = format!("LienzoUI-{}/lienzo-ui.exe", version);
    // Compress-Archive in old Windows PowerShell stores paths with "\": they are compared normalized.
    let idx = (0..a.len()).find(|&i| a.by_index_raw(i).is_ok_and(|f| f.name().is_ok_and(|n| n.replace('\\', "/") == name))).ok_or("The zip does not contain lienzo-ui.exe.")?;
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

fn path_with_extension(exe: &std::path::Path, ext: &str) -> std::path::PathBuf {
    let mut s = exe.as_os_str().to_os_string();
    s.push(ext);
    s.into()
}

/// Swaps the running executable for the new one. On Windows a running .exe cannot be deleted but can
/// be renamed: the old one stays as ".old" (deleted on startup) and, if anything fails, it is rolled back.
pub fn replace_exe(exe: &std::path::Path, new: &[u8]) -> Result<(), String> {
    let tmp = path_with_extension(exe, ".new");
    let old = path_with_extension(exe, ".old");
    let _ = std::fs::remove_file(&old);
    std::fs::write(&tmp, new).map_err(|_| "No permission to write next to the app. Download the new version manually.".to_string())?;
    if std::fs::rename(exe, &old).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return Err("Could not replace the app (protected folder?). Download the new version manually.".into());
    }
    if std::fs::rename(&tmp, exe).is_err() {
        let _ = std::fs::rename(&old, exe);
        let _ = std::fs::remove_file(&tmp);
        return Err("Could not install the new version; keeping the current one.".into());
    }
    Ok(())
}

fn download_and_install(n: &Release) -> Result<(String, std::path::PathBuf), String> {
    if !cfg!(windows) {
        return Err("Automatic install only exists on Windows. Download the new version manually.".into());
    }
    if n.url_zip.is_empty() || n.sha256.is_empty() {
        return Err("This release can't be verified; download it manually.".into());
    }
    let exe = std::env::current_exe().map_err(|_| "The path of the app could not be found.".to_string())?;
    let mut r = agent(300).get(&n.url_zip).call().map_err(|_| "Could not download the update.".to_string())?;
    if !r.status().is_success() {
        return Err(format!("The download responded {}.", r.status().as_u16()));
    }
    let zip = r.body_mut().with_config().limit(MAX_ZIP).read_to_vec().map_err(|_| "The download is too large or was cut off.".to_string())?;
    verify_sha256(&zip, &n.sha256)?;
    let bytes = extract_exe(&zip, &n.version)?;
    replace_exe(&exe, &bytes)?;
    Ok((n.version.clone(), exe))
}

/// Deletes the old .exe left by the previous update.
pub fn clean_leftovers() {
    if let Ok(exe) = std::env::current_exe() {
        let _ = std::fs::remove_file(path_with_extension(&exe, ".old"));
        let _ = std::fs::remove_file(path_with_extension(&exe, ".viejo")); // left by older versions
        let _ = std::fs::remove_file(path_with_extension(&exe, ".new"));
        let _ = std::fs::remove_file(path_with_extension(&exe, ".nuevo")); // left by older versions
    }
}

fn restart(ctx: &egui::Context, exe: &std::path::Path) {
    if std::process::Command::new(exe).spawn().is_ok() {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
    }
}

// ----------------------------------------------------------------------- UI

impl EditorApp {
    /// Starts the check in the background (does nothing if one is already running).
    pub fn updater_search(&mut self) {
        if matches!(self.updater.state, State::Searching | State::Downloading(_)) {
            return;
        }
        let (tx, rx) = channel();
        self.updater.rx = Some(rx);
        self.updater.state = State::Searching;
        std::thread::spawn(move || {
            let _ = tx.send(Message::Result(search()));
        });
    }

    /// Automatic check on startup (can be disabled in the updates window).
    pub fn updater_start(&mut self) {
        clean_leftovers();
        if !self.settings.read_only && !self.settings.skip_update_check && self.screenshot.is_none() {
            self.updater_search();
        }
    }

    fn updater_install(&mut self, n: Release) {
        let (tx, rx) = channel();
        self.updater.rx = Some(rx);
        self.updater.state = State::Downloading(n.clone());
        std::thread::spawn(move || {
            let _ = tx.send(Message::Installed(download_and_install(&n)));
        });
    }

    /// New version to announce in the header (None if there is none or it was skipped).
    pub fn update_for_button(&self) -> Option<&Release> {
        match &self.updater.state {
            State::Available(n) if n.version != self.settings.skipped_version => Some(n),
            _ => None,
        }
    }

    /// Header button: only shown when there is a new version.
    pub fn updater_button(&mut self, ui: &mut egui::Ui) {
        let label = match &self.updater.state {
            State::Installed(..) => Some((t("Restart to update"), OK())),
            _ => self.update_for_button().map(|n| (format!("{} {}", t("Update to"), n.version), ACCENT())),
        };
        if let Some((text, color)) = label {
            let b = egui::Button::new(RichText::new(text).strong().color(Color32::WHITE)).fill(color).corner_radius(6.0);
            if ui.add(b).on_hover_text(t("A new version of Lienzo UI is available")).clicked() {
                self.updater.open = true;
            }
        }
    }

    pub fn updater_window(&mut self, ctx: &egui::Context) {
        // Messages from the background thread.
        let messages: Vec<Message> = self.updater.rx.as_ref().map(|rx| rx.try_iter().collect()).unwrap_or_default();
        for m in messages {
            match m {
                Message::Result(Ok(Some(n))) => self.updater.state = State::Available(n),
                Message::Result(Ok(None)) => self.updater.state = State::UpToDate,
                Message::Result(Err(e)) => self.updater.state = State::Error(e),
                Message::Installed(Ok((v, exe))) => {
                    self.updater.state = State::Installed(v, exe);
                    self.updater.open = true;
                }
                Message::Installed(Err(e)) => self.updater.state = State::Error(e),
            }
        }
        if matches!(self.updater.state, State::Searching | State::Downloading(_)) {
            ctx.request_repaint_after(Duration::from_millis(200));
        }
        if !self.updater.open {
            return;
        }
        let mut open = true;
        let mut install: Option<Release> = None;
        let mut search_other = false;
        let mut skip: Option<String> = None;
        let mut restart_exe: Option<std::path::PathBuf> = None;
        let mut change = false;
        egui::Window::new(t("Updates")).open(&mut open).collapsible(false).resizable(false).default_width(440.0).show(ctx, |ui| {
            ui.label(format!("{} {}", t("Installed version:"), CURRENT_VERSION));
            ui.add_space(6.0);
            match &self.updater.state {
                State::Nothing => {
                    if ui.button(t("Check for updates")).clicked() {
                        search_other = true;
                    }
                }
                State::Searching => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(t("Checking for updates…"));
                    });
                }
                State::UpToDate => {
                    ui.label(RichText::new(t("You have the latest version.")).color(OK()));
                    if ui.button(t("Check again")).clicked() {
                        search_other = true;
                    }
                }
                State::Available(n) => {
                    ui.label(RichText::new(format!("{} {}", t("A new version is available:"), n.version)).strong().color(ACCENT()));
                    if !n.notes.is_empty() {
                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            ui.label(RichText::new(&n.notes).small().color(MUTED()));
                        });
                    }
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        let auto = cfg!(windows) && !n.url_zip.is_empty() && !n.sha256.is_empty();
                        if auto {
                            if ui.add(egui::Button::new(RichText::new(t("Update now")).strong().color(Color32::WHITE)).fill(ACCENT())).clicked() {
                                install = Some(n.clone());
                            }
                        }
                        if ui.button(t(if auto { "Manual download" } else { "Open download page" })).clicked() {
                            let _ = webbrowser::open(RELEASES_PAGE);
                        }
                        if ui.button(t("Skip this version")).clicked() {
                            skip = Some(n.version.clone());
                        }
                    });
                    if !(cfg!(windows) && !n.sha256.is_empty()) {
                        ui.label(RichText::new(t("This version must be downloaded manually (it can't be verified from the app).")).small().color(MUTED()));
                    }
                }
                State::Downloading(n) => {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(format!("{} {}…", t("Downloading and installing version"), n.version));
                    });
                    ui.label(RichText::new(t("Don't close the app in the meantime.")).small().color(MUTED()));
                }
                State::Installed(v, exe) => {
                    ui.label(RichText::new(format!("{} {}", t("Version installed successfully:"), v)).strong().color(OK()));
                    ui.label(RichText::new(t("Save your design: restarting closes the current window.")).small().color(MUTED()));
                    if ui.add(egui::Button::new(RichText::new(t("Restart now")).strong().color(Color32::WHITE)).fill(OK())).clicked() {
                        restart_exe = Some(exe.clone());
                    }
                }
                State::Error(e) => {
                    ui.label(RichText::new(t(e)).color(crate::theme::WARN()));
                    ui.horizontal(|ui| {
                        if ui.button(t("Retry")).clicked() {
                            search_other = true;
                        }
                        if ui.button(t("Open download page")).clicked() {
                            let _ = webbrowser::open(RELEASES_PAGE);
                        }
                    });
                }
            }
            ui.add_space(8.0);
            ui.separator();
            let mut check_on_open = !self.settings.skip_update_check;
            if ui.checkbox(&mut check_on_open, t("Check for updates on startup")).changed() {
                self.settings.skip_update_check = !check_on_open;
                change = true;
            }
            ui.label(RichText::new(t("Only github.com is queried; none of your data is sent.")).small().color(MUTED()));
        });
        if change {
            self.settings.save();
        }
        if let Some(v) = skip {
            self.settings.skipped_version = v;
            self.settings.save();
            open = false;
        }
        if let Some(n) = install {
            self.updater_install(n);
        }
        if search_other {
            self.updater.state = State::Nothing;
            self.updater_search();
        }
        if let Some(exe) = restart_exe {
            restart(ctx, &exe);
        }
        self.updater.open = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn zip_with(name: &str, data: &[u8]) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            z.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
            z.write_all(data).unwrap();
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    fn fake_exe() -> Vec<u8> {
        let mut v = b"MZ".to_vec();
        v.resize(8192, 7);
        v
    }

    #[test]
    fn cleans_the_notes() {
        let n = clean_notes("![x](https://a/b.png)

## Español
**Hola** `app`
- punto
> aviso");
        assert_eq!(n, "Español
Hola app
• punto
aviso");
        assert!(clean_notes(&"a".repeat(9000)).chars().count() <= 4000);
    }

    #[test]
    fn compares_versions() {
        assert!(is_newer("v1.2.0", "1.1.9"));
        assert!(is_newer("1.10.0", "1.9.9"));
        assert!(!is_newer("1.1.1", "1.1.1"));
        assert!(!is_newer("1.0.9", "1.1.1"));
        assert!(!is_newer("1.2.0-beta", "1.1.1"));
        assert!(!is_newer("basura", "1.1.1"));
        assert!(!is_newer("1.2", "1.1.1"));
    }

    #[test]
    fn parses_the_release() {
        let h = "a".repeat(64);
        let json = format!(
            r#"{{"tag_name":"v1.2.0","body":"Novedades","assets":[{{"name":"LienzoUI-windows-1.2.0.zip","browser_download_url":"{}v1.2.0/LienzoUI-windows-1.2.0.zip","digest":"sha256:{}"}}]}}"#,
            DOWNLOAD_PREFIX, h
        );
        let n = parse_release(&json, "1.1.1").unwrap().unwrap();
        assert_eq!((n.version.as_str(), n.sha256.as_str()), ("1.2.0", h.as_str()));
        assert!(parse_release(&json, "1.2.0").unwrap().is_none());
        assert!(parse_release("no es json", "1.1.1").is_err());
    }

    #[test]
    fn rejects_downloads_from_other_origin_or_without_fingerprint() {
        let json = r#"{"tag_name":"v1.2.0","assets":[{"name":"LienzoUI-windows-1.2.0.zip","browser_download_url":"https://malo.example/x.zip","digest":"sha256:abc"}]}"#;
        let n = parse_release(json, "1.1.1").unwrap().unwrap();
        assert!(n.url_zip.is_empty() && n.sha256.is_empty());
        let draft = r#"{"tag_name":"v9.0.0","draft":true}"#;
        assert!(parse_release(draft, "1.1.1").unwrap().is_none());
        let pre = r#"{"tag_name":"v9.0.0","prerelease":true}"#;
        assert!(parse_release(pre, "1.1.1").unwrap().is_none());
    }

    #[test]
    fn verifies_the_fingerprint() {
        let data = b"hola";
        let hex: String = Sha256::digest(data).iter().map(|b| format!("{:02x}", b)).collect();
        assert!(verify_sha256(data, &hex).is_ok());
        assert!(verify_sha256(data, &hex.to_uppercase()).is_ok());
        assert!(verify_sha256(b"otra cosa", &hex).is_err());
        assert!(verify_sha256(data, "").is_err());
    }

    #[test]
    fn extracts_only_the_expected_exe() {
        let ok = zip_with("LienzoUI-1.2.0/lienzo-ui.exe", &fake_exe());
        assert_eq!(extract_exe(&ok, "1.2.0").unwrap().len(), 8192);
        assert!(extract_exe(&zip_with("LienzoUI-1.2.0/otro.exe", &fake_exe()), "1.2.0").is_err());
        assert_eq!(extract_exe(&zip_with("LienzoUI-1.2.0\\lienzo-ui.exe", &fake_exe()), "1.2.0").unwrap().len(), 8192);
        assert!(extract_exe(&zip_with("../lienzo-ui.exe", &fake_exe()), "1.2.0").is_err());
        assert!(extract_exe(&zip_with("LienzoUI-1.2.0/lienzo-ui.exe", b"MZ corto"), "1.2.0").is_err());
        assert!(extract_exe(&zip_with("LienzoUI-1.2.0/lienzo-ui.exe", &vec![1u8; 8192]), "1.2.0").is_err());
        assert!(extract_exe(b"no es un zip", "1.2.0").is_err());
    }

    #[test]
    fn replaces_the_exe_and_keeps_the_old_one() {
        let dir = std::env::temp_dir().join(format!("lienzo-act-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("Lienzo UI.exe");
        std::fs::write(&exe, b"old version").unwrap();
        replace_exe(&exe, &fake_exe()).unwrap();
        assert_eq!(std::fs::read(&exe).unwrap().len(), 8192);
        assert_eq!(std::fs::read(path_with_extension(&exe, ".old")).unwrap(), b"old version");
        assert!(!path_with_extension(&exe, ".new").exists());
        // a second update overwrites the .old without failing
        replace_exe(&exe, &fake_exe()).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Needs network: downloads the real GitHub release, checks its hash and extracts the .exe (`cargo test -- --ignored`).
    #[test]
    #[ignore]
    fn with_the_real_github_release() {
        let mut r = agent(15).get(API).header("Accept", "application/vnd.github+json").call().unwrap();
        let json = r.body_mut().read_to_string().unwrap();
        let n = parse_release(&json, "0.0.1").unwrap().expect("there should be a version newer than 0.0.1");
        assert!(!n.sha256.is_empty() && !n.url_zip.is_empty(), "the release must have a URL and a fingerprint: {:?}", n);
        let mut d = agent(300).get(&n.url_zip).call().unwrap();
        let zip = d.body_mut().with_config().limit(MAX_ZIP).read_to_vec().unwrap();
        verify_sha256(&zip, &n.sha256).unwrap();
        let exe = extract_exe(&zip, &n.version).unwrap();
        println!("release {} · zip {} B · exe {} B", n.version, zip.len(), exe.len());
    }
}
