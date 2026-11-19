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

fn le16(b: &[u8], i: usize) -> Option<u16> {
    Some(u16::from_le_bytes(b.get(i..i.checked_add(2)?)?.try_into().ok()?))
}

fn le32(b: &[u8], i: usize) -> Option<u32> {
    Some(u32::from_le_bytes(b.get(i..i.checked_add(4)?)?.try_into().ok()?))
}

/// Checks the PE (Windows executable) headers and returns the file version of its version resource
/// (`major, minor, patch, build`), or None when the program has no readable version resource.
/// Err when it is not a valid PE file.
pub fn pe_version(exe: &[u8]) -> Result<Option<[u16; 4]>, String> {
    let bad = || "The executable in the zip does not look valid.".to_string();
    if exe.get(..2) != Some(b"MZ") {
        return Err(bad());
    }
    let pe = le32(exe, 0x3c).ok_or_else(bad)? as usize;
    if exe.get(pe..pe.checked_add(4).ok_or_else(bad)?) != Some(b"PE\0\0") {
        return Err(bad());
    }
    let machine = le16(exe, pe + 4).ok_or_else(bad)?;
    let nsec = le16(exe, pe + 6).ok_or_else(bad)? as usize;
    let opt_size = le16(exe, pe + 20).ok_or_else(bad)? as usize;
    let opt = pe + 24;
    let magic = le16(exe, opt).ok_or_else(bad)?;
    if !matches!(machine, 0x14c | 0x8664 | 0xaa64) || !matches!(magic, 0x10b | 0x20b) || nsec == 0 || nsec > 96 || opt_size < 96 {
        return Err(bad());
    }
    let plus = magic == 0x20b;
    let n_dirs = le32(exe, opt + if plus { 108 } else { 92 }).ok_or_else(bad)? as usize;
    let dirs = opt + if plus { 112 } else { 96 };
    let sections = opt + opt_size;
    if exe.len() < sections + nsec * 40 {
        return Err(bad());
    }
    let rva_to_off = |rva: u32| -> Option<usize> {
        for i in 0..nsec {
            let s = sections + i * 40;
            let (va, raw_size, raw_ptr) = (le32(exe, s + 12)?, le32(exe, s + 16)?, le32(exe, s + 20)?);
            if rva >= va && rva - va < raw_size {
                return Some(raw_ptr as usize + (rva - va) as usize);
            }
        }
        None
    };
    if n_dirs <= 2 {
        return Ok(None);
    }
    let Some(res_rva) = le32(exe, dirs + 16).filter(|r| *r != 0) else { return Ok(None) };
    let Some(base) = rva_to_off(res_rva) else { return Ok(None) };
    // resource directory: entry `idx` of the directory at `dir` -> (id, target offset, target is a directory)
    let entry = |dir: usize, idx: usize| -> Option<(u32, usize, bool)> {
        let named = le16(exe, dir + 12)? as usize;
        let ids = le16(exe, dir + 14)? as usize;
        if idx >= named + ids {
            return None;
        }
        let e = dir + 16 + idx * 8;
        let (id, off) = (le32(exe, e)?, le32(exe, e + 4)?);
        Some((id, base + (off & 0x7fff_ffff) as usize, off & 0x8000_0000 != 0))
    };
    let count = |dir: usize| le16(exe, dir + 12).unwrap_or(0) as usize + le16(exe, dir + 14).unwrap_or(0) as usize;
    // type RT_VERSION (16) -> first name -> first language -> data entry
    let Some((_, d1, true)) = (0..count(base).min(64)).filter_map(|i| entry(base, i)).find(|(id, _, dir)| *id == 16 && *dir) else { return Ok(None) };
    let Some((_, d2, true)) = entry(d1, 0) else { return Ok(None) };
    let Some((_, leaf, false)) = entry(d2, 0) else { return Ok(None) };
    let (Some(data_rva), Some(size)) = (le32(exe, leaf), le32(exe, leaf + 4)) else { return Ok(None) };
    let Some(off) = rva_to_off(data_rva) else { return Ok(None) };
    let data = exe.get(off..off.saturating_add((size as usize).min(1024))).unwrap_or(&[]);
    // VS_FIXEDFILEINFO starts with the signature 0xFEEF04BD, a few bytes after the "VS_VERSION_INFO" key
    let Some(sig) = (0..data.len().saturating_sub(16)).step_by(2).find(|&i| le32(data, i) == Some(0xFEEF_04BD)) else { return Ok(None) };
    let (Some(ms), Some(ls)) = (le32(data, sig + 8), le32(data, sig + 12)) else { return Ok(None) };
    Ok(Some([(ms >> 16) as u16, ms as u16, (ls >> 16) as u16, ls as u16]))
}

/// Extracts the executable from the zip. The zip must contain exactly ONE `.exe` (`LienzoUI-<v>/lienzo-ui.exe`), it must be a
/// valid PE file and its version resource (required) must match the release; no other name is written to disk.
pub fn extract_exe(zip: &[u8], version: &str) -> Result<Vec<u8>, String> {
    let mut a = zip::ZipArchive::new(std::io::Cursor::new(zip)).map_err(|_| "The downloaded zip is not valid.".to_string())?;
    let name = format!("LienzoUI-{}/lienzo-ui.exe", version);
    // Compress-Archive in old Windows PowerShell stores paths with "\": they are compared normalized.
    let exes: Vec<usize> = (0..a.len()).filter(|&i| a.by_index_raw(i).is_ok_and(|f| !f.is_dir() && f.name().is_ok_and(|n| n.to_ascii_lowercase().ends_with(".exe")))).collect();
    if exes.len() > 1 {
        return Err("The zip must contain a single executable.".into());
    }
    let idx = exes.first().copied().filter(|&i| a.by_index_raw(i).is_ok_and(|f| f.name().is_ok_and(|n| n.replace('\\', "/") == name))).ok_or("The zip does not contain lienzo-ui.exe.")?;
    let mut f = a.by_index(idx).map_err(|_| "The zip could not be read.".to_string())?;
    if f.size() > MAX_EXE {
        return Err("The executable in the zip is too large.".into());
    }
    let mut out = Vec::with_capacity(f.size() as usize);
    f.by_ref().take(MAX_EXE + 1).read_to_end(&mut out).map_err(|_| "The zip could not be read.".to_string())?;
    if out.len() as u64 > MAX_EXE || out.len() < 4096 {
        return Err("The executable in the zip does not look valid.".into());
    }
    // The version resource is required: an executable that does not say which version it is cannot be checked against the release.
    let Some(v) = pe_version(&out)? else {
        return Err("The executable has no version information; not installing.".into());
    };
    if parse_version(version).is_none_or(|w| (v[0] as u32, v[1] as u32, v[2] as u32) != w) {
        return Err("The version inside the executable does not match the release; not installing.".into());
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
    install_zip(&exe, &zip, n)?;
    Ok((n.version.clone(), exe))
}

/// Verifies the downloaded zip (SHA-256, single valid executable of the right version) and only then swaps `exe`
/// (the previous one stays as `.old` and is put back if the swap fails). Nothing is touched when a check fails.
pub fn install_zip(exe: &std::path::Path, zip: &[u8], n: &Release) -> Result<(), String> {
    verify_sha256(zip, &n.sha256)?;
    let bytes = extract_exe(zip, &n.version)?;
    replace_exe(exe, &bytes)
}

/// First 600 characters of the release notes (cut at a character boundary, with an ellipsis when longer).
pub fn notes_preview(notes: &str) -> String {
    let mut it = notes.trim().chars();
    let head: String = it.by_ref().take(600).collect();
    if it.next().is_some() { format!("{}…", head.trim_end()) } else { head }
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

pub(crate) fn restart(ctx: &egui::Context, exe: &std::path::Path) {
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
                        ui.label(RichText::new(t("Release notes")).small().strong());
                        egui::ScrollArea::vertical().max_height(180.0).show(ui, |ui| {
                            ui.label(RichText::new(notes_preview(&n.notes)).small().color(MUTED()));
                        });
                        if n.notes.trim().chars().count() > 600 && ui.link(t("Full release notes")).clicked() {
                            let _ = webbrowser::open(RELEASES_PAGE);
                        }
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
            if self.request_restart(&exe) {
                restart(ctx, &exe);
            }
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

    /// A minimal but structurally valid PE32+ file; with `version` it carries a version resource.
    fn fake_exe_with(version: Option<[u16; 4]>) -> Vec<u8> {
        let mut v = vec![0u8; 0x800];
        v[..2].copy_from_slice(b"MZ");
        v[0x3c..0x40].copy_from_slice(&0x40u32.to_le_bytes());
        let pe = 0x40;
        v[pe..pe + 4].copy_from_slice(b"PE\0\0");
        v[pe + 4..pe + 6].copy_from_slice(&0x8664u16.to_le_bytes()); // machine
        v[pe + 6..pe + 8].copy_from_slice(&1u16.to_le_bytes()); // one section
        v[pe + 20..pe + 22].copy_from_slice(&240u16.to_le_bytes()); // optional header size
        let opt = pe + 24;
        v[opt..opt + 2].copy_from_slice(&0x20bu16.to_le_bytes());
        v[opt + 108..opt + 112].copy_from_slice(&16u32.to_le_bytes()); // data directories
        let sec = opt + 240;
        v[sec..sec + 6].copy_from_slice(b".rsrc\0");
        v[sec + 8..sec + 12].copy_from_slice(&0x200u32.to_le_bytes()); // virtual size
        v[sec + 12..sec + 16].copy_from_slice(&0x1000u32.to_le_bytes()); // rva
        v[sec + 16..sec + 20].copy_from_slice(&0x400u32.to_le_bytes()); // raw size
        v[sec + 20..sec + 24].copy_from_slice(&0x400u32.to_le_bytes()); // raw pointer
        if let Some(ver) = version {
            let dd = opt + 112 + 16;
            v[dd..dd + 4].copy_from_slice(&0x1000u32.to_le_bytes());
            v[dd + 4..dd + 8].copy_from_slice(&0x200u32.to_le_bytes());
            let r = 0x400;
            let mut put = |at: usize, x: u32| v[r + at..r + at + 4].copy_from_slice(&x.to_le_bytes());
            // root: one id entry (type 16) -> dir at 0x18 -> one entry (id 1) -> dir at 0x30 -> one entry (lang) -> data entry at 0x48
            put(12, 1 << 16);
            put(16, 16);
            put(20, 0x8000_0000 | 0x18);
            put(0x18 + 12, 1 << 16);
            put(0x18 + 16, 1);
            put(0x18 + 20, 0x8000_0000 | 0x30);
            put(0x30 + 12, 1 << 16);
            put(0x30 + 16, 0x409);
            put(0x30 + 20, 0x48);
            put(0x48, 0x1000 + 0x58); // data rva
            put(0x48 + 4, 0x60); // size
            // VS_VERSION_INFO: header, key, padding, then VS_FIXEDFILEINFO
            let d = 0x58;
            v[r + d + 6..r + d + 6 + 32].copy_from_slice(&"VS_VERSION_INFO\0".encode_utf16().flat_map(|c| c.to_le_bytes()).collect::<Vec<u8>>());
            let mut put = |at: usize, x: u32| v[r + d + at..r + d + at + 4].copy_from_slice(&x.to_le_bytes());
            put(40, 0xFEEF_04BD);
            put(40 + 8, ((ver[0] as u32) << 16) | ver[1] as u32);
            put(40 + 12, ((ver[2] as u32) << 16) | ver[3] as u32);
        }
        v.resize(8192, 7);
        v
    }

    fn fake_exe() -> Vec<u8> {
        fake_exe_with(None)
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
        let ok = zip_with("LienzoUI-1.2.0/lienzo-ui.exe", &fake_exe_with(Some([1, 2, 0, 0])));
        assert_eq!(extract_exe(&ok, "1.2.0").unwrap().len(), 8192);
        assert!(extract_exe(&zip_with("LienzoUI-1.2.0/otro.exe", &fake_exe()), "1.2.0").is_err());
        assert_eq!(extract_exe(&zip_with("LienzoUI-1.2.0\\lienzo-ui.exe", &fake_exe_with(Some([1, 2, 0, 0]))), "1.2.0").unwrap().len(), 8192);
        assert!(extract_exe(&zip_with("../lienzo-ui.exe", &fake_exe_with(Some([1, 2, 0, 0]))), "1.2.0").is_err());
        assert!(extract_exe(&zip_with("LienzoUI-1.2.0/lienzo-ui.exe", b"MZ corto"), "1.2.0").is_err());
        assert!(extract_exe(&zip_with("LienzoUI-1.2.0/lienzo-ui.exe", &vec![1u8; 8192]), "1.2.0").is_err());
        assert!(extract_exe(b"no es un zip", "1.2.0").is_err());
    }

    fn zip_with_files(files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            for (n, d) in files {
                z.start_file(*n, zip::write::SimpleFileOptions::default()).unwrap();
                z.write_all(d).unwrap();
            }
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    #[test]
    fn the_pe_header_and_the_version_resource_are_checked() {
        assert_eq!(pe_version(&fake_exe()).unwrap(), None, "valid PE without a version resource");
        assert_eq!(pe_version(&fake_exe_with(Some([1, 2, 0, 0]))).unwrap(), Some([1, 2, 0, 0]));
        // broken headers
        let mut bad = fake_exe();
        bad[0x40] = b'X'; // no PE signature
        assert!(pe_version(&bad).is_err());
        let mut bad = fake_exe();
        bad[0x44] = 0; // unknown machine
        bad[0x45] = 0;
        assert!(pe_version(&bad).is_err());
        let mut bad = fake_exe();
        bad[0x3c..0x40].copy_from_slice(&0xFFFF_FFF0u32.to_le_bytes()); // e_lfanew out of range
        assert!(pe_version(&bad).is_err());
        assert!(pe_version(b"MZ").is_err() && pe_version(&[]).is_err());
        // truncated resource tables do not panic
        for cut in [0x120, 0x200, 0x410, 0x440, 0x470, 0x500] {
            let mut t = fake_exe_with(Some([1, 2, 0, 0]));
            t.truncate(cut);
            let _ = pe_version(&t);
        }
        // through the zip: the right version passes, another one (or a bad PE) is refused
        let name = "LienzoUI-1.2.0/lienzo-ui.exe";
        assert!(extract_exe(&zip_with(name, &fake_exe_with(Some([1, 2, 0, 0]))), "1.2.0").is_ok());
        let e = extract_exe(&zip_with(name, &fake_exe_with(Some([1, 1, 9, 0]))), "1.2.0").unwrap_err();
        assert!(e.contains("does not match"), "{}", e);
        assert!(extract_exe(&zip_with(name, &fake_exe_with(Some([1, 2, 0, 0]))), "1.2.1").is_err());
        let mut nope = fake_exe();
        nope[0x41] = 0; // MZ intact is not enough
        nope[0x42] = 0;
        nope[0x43] = 0;
        assert!(extract_exe(&zip_with(name, &nope), "1.2.0").is_err());
    }

    #[test]
    fn an_exe_without_a_version_resource_is_refused() {
        let name = "LienzoUI-1.2.0/lienzo-ui.exe";
        let e = extract_exe(&zip_with(name, &fake_exe()), "1.2.0").unwrap_err();
        assert!(e.contains("no version"), "{}", e);
        assert!(extract_exe(&zip_with(name, &fake_exe_with(Some([1, 2, 0, 0]))), "1.2.0").is_ok());
    }

    #[test]
    fn the_zip_must_hold_a_single_exe() {
        let good = fake_exe_with(Some([1, 2, 0, 0]));
        let name = "LienzoUI-1.2.0/lienzo-ui.exe";
        assert!(extract_exe(&zip_with_files(&[(name, &good), ("LienzoUI-1.2.0/README.txt", b"hi")]), "1.2.0").is_ok(), "other non-exe files are ignored");
        let e = extract_exe(&zip_with_files(&[(name, &good), ("LienzoUI-1.2.0/helper.exe", &good)]), "1.2.0").unwrap_err();
        assert!(e.contains("single executable"), "{}", e);
        assert!(extract_exe(&zip_with_files(&[(name, &good), ("other/LIENZO-UI.EXE", &good)]), "1.2.0").is_err());
        assert!(extract_exe(&zip_with_files(&[("README.txt", b"x")]), "1.2.0").is_err());
    }

    #[test]
    fn install_checks_everything_before_touching_the_exe() {
        let dir = std::env::temp_dir().join(format!("lienzo-inst-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let exe = dir.join("lienzo-ui.exe");
        std::fs::write(&exe, b"current").unwrap();
        let name = "LienzoUI-1.2.0/lienzo-ui.exe";
        let zip = zip_with(name, &fake_exe_with(Some([1, 2, 0, 0])));
        let hex = |b: &[u8]| -> String { Sha256::digest(b).iter().map(|x| format!("{:02x}", x)).collect() };
        let rel = |sha: String| Release { version: "1.2.0".into(), notes: String::new(), url_zip: "x".into(), sha256: sha };
        // wrong hash, wrong version in the PE, two exes: the current exe stays
        assert!(install_zip(&exe, &zip, &rel("0".repeat(64))).is_err());
        let wrong = zip_with(name, &fake_exe_with(Some([9, 9, 9, 0])));
        assert!(install_zip(&exe, &wrong, &rel(hex(&wrong))).is_err());
        let two = zip_with_files(&[(name, &fake_exe()), ("LienzoUI-1.2.0/b.exe", &fake_exe())]);
        assert!(install_zip(&exe, &two, &rel(hex(&two))).is_err());
        assert_eq!(std::fs::read(&exe).unwrap(), b"current");
        assert!(!path_with_extension(&exe, ".old").exists() && !path_with_extension(&exe, ".new").exists());
        // everything right: replaced, the old one is kept for the rollback
        install_zip(&exe, &zip, &rel(hex(&zip))).unwrap();
        assert_eq!(std::fs::read(&exe).unwrap().len(), 8192);
        assert_eq!(std::fs::read(path_with_extension(&exe, ".old")).unwrap(), b"current");
        // rollback: the destination cannot be renamed into place -> the old exe is back
        let blocked = dir.join("sub").join("lienzo-ui.exe");
        assert!(replace_exe(&blocked, &fake_exe()).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The real executable built by this project (when a release build exists) has a valid PE header and the crate's version.
    #[test]
    fn the_real_release_exe_passes_the_pe_check() {
        let exe = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/release/lienzo-ui.exe");
        let Ok(bytes) = std::fs::read(exe) else { return };
        let v = pe_version(&bytes).expect("valid PE").expect("the release exe must carry a version resource");
        {
            assert_eq!(Some((v[0] as u32, v[1] as u32, v[2] as u32)), parse_version(CURRENT_VERSION));
        }
    }

    #[test]
    fn notes_preview_is_600_characters() {
        assert_eq!(notes_preview("short"), "short");
        let long = "é".repeat(700);
        let p = notes_preview(&long);
        assert_eq!(p.chars().count(), 601);
        assert!(p.ends_with('…'));
        assert_eq!(notes_preview(&"a".repeat(600)).chars().count(), 600, "exactly 600: no ellipsis");
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
