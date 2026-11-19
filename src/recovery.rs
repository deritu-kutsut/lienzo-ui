//! Autosave and crash recovery.
//!
//! Every minute (when the design changed and nothing is being dragged or typed) the full design is written atomically to
//! `<app data>/recovery/<key>.editorgui.json` with a small `<key>.meta.json` next to it (original path, title, time).
//! `<key>` is a hash of the project path (or `untitled`). A clean exit or a successful Save removes the files; on
//! startup a recovery newer than the project file is offered to the user. At most [`MAX_FILES`] recoveries are kept.

use crate::library::write_atomic;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub const MAX_FILES: usize = 10;
pub const INTERVAL: Duration = Duration::from_secs(60);
/// Recoveries older than this are deleted at startup.
pub const STALE_MS: u64 = 30 * 24 * 3600 * 1000;
pub const UNTITLED: &str = "untitled";
/// Largest design / meta file read from the recovery folder (anything bigger is ignored or deleted).
pub const MAX_BYTES: u64 = 64 * 1024 * 1024;
/// Orphan files younger than this are left alone by [`rotate`] (another running instance may be mid-write).
pub const ORPHAN_AGE: Duration = Duration::from_secs(300);

/// True for the key of a design that was never saved (`untitled` or `untitled-<instance>`).
pub fn is_untitled(key: &str) -> bool {
    key == UNTITLED || key.starts_with("untitled-")
}

/// Reads a text file only when it is not bigger than [`MAX_BYTES`].
pub fn read_capped(p: &Path) -> Option<String> {
    if std::fs::metadata(p).ok()?.len() > MAX_BYTES {
        return None;
    }
    std::fs::read_to_string(p).ok()
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Meta {
    /// Original project file ("" = never saved).
    pub path: String,
    pub title: String,
    /// Unix time in milliseconds.
    pub saved_at_ms: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub key: String,
    pub meta: Meta,
    pub design: PathBuf,
}

pub fn recovery_dir() -> PathBuf {
    crate::library::root_dir().parent().map(|p| p.join("recovery")).unwrap_or_else(|| PathBuf::from("recovery"))
}

pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// File key of a project: a short hash of its path, or `untitled`.
pub fn key_for(project: Option<&str>) -> String {
    match project {
        Some(p) if !p.trim().is_empty() => {
            let h = Sha256::digest(p.trim().to_lowercase().replace('\\', "/").as_bytes());
            h[..8].iter().map(|b| format!("{:02x}", b)).collect()
        }
        _ => UNTITLED.to_string(),
    }
}

fn design_file(dir: &Path, key: &str) -> PathBuf {
    dir.join(format!("{}.editorgui.json", key))
}

fn meta_file(dir: &Path, key: &str) -> PathBuf {
    dir.join(format!("{}.meta.json", key))
}

/// Writes (atomically) the design and its meta, then rotates old recoveries.
pub fn write(dir: &Path, key: &str, json: &str, meta: &Meta) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    write_atomic(&design_file(dir, key), json.as_bytes())?;
    let m = serde_json::to_string(meta).map_err(std::io::Error::other)?;
    write_atomic(&meta_file(dir, key), m.as_bytes())?;
    rotate(dir, MAX_FILES);
    Ok(())
}

pub fn read(dir: &Path, key: &str) -> Option<(String, Meta)> {
    let json = read_capped(&design_file(dir, key))?;
    let meta = read_meta(dir, key)?;
    Some((json, meta))
}

fn read_meta(dir: &Path, key: &str) -> Option<Meta> {
    let m = read_capped(&meta_file(dir, key))?;
    serde_json::from_str(m.trim_start_matches('\u{feff}')).ok()
}

pub fn remove(dir: &Path, key: &str) {
    let _ = std::fs::remove_file(design_file(dir, key));
    let _ = std::fs::remove_file(meta_file(dir, key));
}

/// Every recovery in the folder, newest first. Files without a readable meta (or without the design) are ignored.
pub fn list(dir: &Path) -> Vec<Entry> {
    let mut v: Vec<Entry> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter_map(|e| {
                    let name = e.file_name().to_string_lossy().to_string();
                    let key = name.strip_suffix(".meta.json")?.to_string();
                    let design = design_file(dir, &key);
                    if !design.is_file() {
                        return None;
                    }
                    Some(Entry { meta: read_meta(dir, &key)?, key, design })
                })
                .collect()
        })
        .unwrap_or_default();
    v.sort_by(|a, b| b.meta.saved_at_ms.cmp(&a.meta.saved_at_ms).then(a.key.cmp(&b.key)));
    v
}

/// Keeps the `max` newest recoveries (and deletes orphan files and leftover temporary files).
pub fn rotate(dir: &Path, max: usize) {
    rotate_aged(dir, max, ORPHAN_AGE)
}

/// [`rotate`] with an explicit minimum age for deleting orphan files (files younger than `orphan_age` may belong to
/// another running instance that is mid-write).
pub fn rotate_aged(dir: &Path, max: usize, orphan_age: Duration) {
    let keep = list(dir);
    for e in keep.iter().skip(max) {
        remove(dir, &e.key);
    }
    let valid: std::collections::HashSet<String> = keep.iter().take(max).flat_map(|e| [format!("{}.editorgui.json", e.key), format!("{}.meta.json", e.key)]).collect();
    if let Ok(rd) = std::fs::read_dir(dir) {
        for f in rd.filter_map(|e| e.ok()) {
            let name = f.file_name().to_string_lossy().to_string();
            let ours = name.ends_with(".editorgui.json") || name.ends_with(".meta.json");
            let old_tmp = name.starts_with('.') && name.ends_with(".tmp") && f.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|d| d > Duration::from_secs(3600));
            let old_enough = f.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok()).is_some_and(|d| d >= orphan_age);
            if (ours && !valid.contains(&name) && old_enough) || old_tmp {
                let _ = std::fs::remove_file(f.path());
            }
        }
    }
}

/// Deletes recoveries older than [`STALE_MS`] or whose design is not valid JSON. Returns how many were removed.
pub fn cleanup_stale(dir: &Path, now: u64) -> usize {
    let mut n = 0;
    for e in list(dir) {
        let old = now.saturating_sub(e.meta.saved_at_ms) > STALE_MS;
        let broken = read_capped(&e.design).and_then(|s| serde_json::from_str::<serde_json::Value>(s.trim_start_matches('\u{feff}')).ok()).is_none();
        if old || broken {
            remove(dir, &e.key);
            n += 1;
        }
    }
    n
}

/// Pure decision: is a recovery worth offering? Yes when there is no project file to compare with, or the recovery
/// is newer than that file.
pub fn should_offer(recovery_saved_ms: u64, project_mtime_ms: Option<u64>) -> bool {
    match project_mtime_ms {
        None => true,
        Some(p) => recovery_saved_ms > p,
    }
}

fn mtime_ms(p: &str) -> Option<u64> {
    if p.trim().is_empty() {
        return None;
    }
    std::fs::metadata(p).and_then(|m| m.modified()).ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis() as u64)
}

/// The recovery to offer at startup: the newest one that passes [`should_offer`] (only `only_key` when a project was
/// given on the command line).
pub fn pick_offer(dir: &Path, only_key: Option<&str>) -> Option<Entry> {
    list(dir).into_iter().filter(|e| only_key.is_none_or(|k| k == e.key || is_untitled(&e.key))).find(|e| should_offer(e.meta.saved_at_ms, mtime_ms(&e.meta.path)))
}

/// «2026-11-19 14:03 UTC» (no time-zone database is bundled).
pub fn format_time(ms: u64) -> String {
    let secs = ms / 1000;
    let (days, rem) = ((secs / 86400) as i64, secs % 86400);
    // civil-from-days (Howard Hinnant)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02} {:02}:{:02} UTC", y, m, d, rem / 3600, (rem % 3600) / 60)
}

/// Autosave bookkeeping of the running editor.
pub struct Session {
    pub dir: PathBuf,
    /// Project file the open design is bound to (None = never saved / restored as unsaved).
    pub bound: Option<String>,
    last: Instant,
    /// Counter bumped by every committed edit.
    edits: u64,
    autosaved_edits: u64,
    saved_edits: u64,
    pub interval: Duration,
    /// Recovery waiting for the user's answer.
    pub offer: Option<Entry>,
    /// Unique per running editor: the key of a never-saved design, so two windows do not share one `untitled` file.
    instance: String,
    /// The user chose "Discard changes" on exit: the recovery may be deleted although the design is dirty.
    pub discard_ok: bool,
    /// The "unsaved changes" dialog is open (close or updater restart was requested).
    pub close_prompt: bool,
    /// The updater asked to restart into this exe once the dialog is resolved.
    pub pending_restart: Option<PathBuf>,
}

impl Default for Session {
    fn default() -> Self {
        let instance = format!("{}-{:x}", std::process::id(), now_ms());
        Session { dir: recovery_dir(), bound: None, last: Instant::now(), edits: 0, autosaved_edits: 0, saved_edits: 0, interval: INTERVAL, offer: None, instance, discard_ok: false, close_prompt: false, pending_restart: None }
    }
}

impl Session {
    /// File key of the open design: the hash of its path, or `untitled-<instance>` when it was never saved.
    pub fn key(&self) -> String {
        match self.bound.as_deref() {
            Some(p) if !p.trim().is_empty() => key_for(Some(p)),
            _ => format!("{}-{}", UNTITLED, self.instance),
        }
    }

    pub fn note_edit(&mut self) {
        self.edits += 1;
    }

    /// The document was replaced (open / new): nothing is pending.
    pub fn reset(&mut self) {
        self.saved_edits = self.edits;
        self.autosaved_edits = self.edits;
        self.last = Instant::now();
    }

    pub fn mark_saved(&mut self) {
        self.saved_edits = self.edits;
        self.autosaved_edits = self.edits;
    }

    pub fn dirty(&self) -> bool {
        self.edits != self.saved_edits
    }

    /// Time to autosave: enabled, the interval elapsed, unsaved edits not yet autosaved, and the user is not mid-gesture.
    pub fn due(&self, now: Instant, busy: bool, enabled: bool) -> bool {
        enabled && !busy && now.duration_since(self.last) >= self.interval && self.edits != self.saved_edits && self.edits != self.autosaved_edits
    }

    pub fn wrote(&mut self, now: Instant) {
        self.last = now;
        self.autosaved_edits = self.edits;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("lienzo_rec_{}_{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn meta(path: &str, ms: u64) -> Meta {
        Meta { path: path.into(), title: "T".into(), saved_at_ms: ms }
    }

    #[test]
    fn write_read_remove() {
        let d = tmp("wr");
        let k = key_for(Some("C:\\a\\b.editorgui.json"));
        assert_eq!(k.len(), 16);
        assert_eq!(k, key_for(Some("c:/a/b.editorgui.json")), "case and separators do not change the key");
        assert_ne!(k, key_for(Some("C:/a/c.json")));
        assert_eq!(key_for(None), UNTITLED);
        write(&d, &k, "{\"a\":1}", &meta("C:\\a\\b.editorgui.json", 5)).unwrap();
        let (j, m) = read(&d, &k).unwrap();
        assert_eq!(j, "{\"a\":1}");
        assert_eq!(m.saved_at_ms, 5);
        remove(&d, &k);
        assert!(read(&d, &k).is_none() && list(&d).is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn rotation_keeps_the_newest_ten() {
        let d = tmp("rot");
        for i in 0..15u64 {
            write(&d, &format!("k{:02}", i), "{}", &meta("", 1000 + i)).unwrap();
        }
        let l = list(&d);
        assert_eq!(l.len(), MAX_FILES);
        assert_eq!(l[0].key, "k14");
        assert_eq!(l.last().unwrap().key, "k05");
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), MAX_FILES * 2, "no orphan files");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn writes_are_atomic_and_leave_no_temporary_files() {
        let d = tmp("atomic");
        write(&d, "k", "{\"v\":1}", &meta("", 1)).unwrap();
        write(&d, "k", "{\"v\":2}", &meta("", 2)).unwrap();
        assert_eq!(read(&d, "k").unwrap().0, "{\"v\":2}");
        assert!(std::fs::read_dir(&d).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().ends_with(".tmp")));
        // a failed write (the destination folder is a file) does not touch the existing recovery
        let blocker = d.join("blocker");
        std::fs::write(&blocker, "x").unwrap();
        assert!(write(&blocker, "k", "{}", &meta("", 3)).is_err());
        assert_eq!(read(&d, "k").unwrap().0, "{\"v\":2}");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn stale_and_broken_recoveries_are_cleaned() {
        let d = tmp("stale");
        let now = 100 * 24 * 3600 * 1000u64;
        write(&d, "old", "{}", &meta("", now - STALE_MS - 1)).unwrap();
        write(&d, "fresh", "{}", &meta("", now - 1000)).unwrap();
        write(&d, "broken", "{not json", &meta("", now - 1000)).unwrap();
        std::fs::write(d.join("orphan.editorgui.json"), "{}").unwrap(); // no meta: ignored by list
        assert_eq!(cleanup_stale(&d, now), 2);
        let keys: Vec<String> = list(&d).into_iter().map(|e| e.key).collect();
        assert_eq!(keys, vec!["fresh".to_string()]);
        rotate(&d, MAX_FILES);
        assert!(d.join("orphan.editorgui.json").exists(), "a fresh orphan may belong to another instance mid-write (B3)");
        rotate_aged(&d, MAX_FILES, Duration::ZERO);
        assert!(!d.join("orphan.editorgui.json").exists(), "old orphans are removed");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn restore_decision_logic() {
        assert!(should_offer(10, None), "no project file: offer");
        assert!(should_offer(10, Some(5)), "recovery newer than the file");
        assert!(!should_offer(5, Some(5)), "same time: the file is as good");
        assert!(!should_offer(4, Some(5)), "file newer: do not offer");
        // pick_offer: a recovery whose project file does not exist is offered; one older than its file is not
        let d = tmp("pick");
        std::fs::create_dir_all(d.join("proj")).unwrap();
        let proj = d.join("proj").join("p.editorgui.json");
        std::fs::write(&proj, "{}").unwrap();
        let ps = proj.to_string_lossy().to_string();
        write(&d, "stale", "{}", &meta(&ps, 1)).unwrap();
        assert!(pick_offer(&d, None).is_none());
        write(&d, "gone", "{}", &meta(&d.join("none.json").to_string_lossy(), 2)).unwrap();
        assert_eq!(pick_offer(&d, None).unwrap().key, "gone");
        assert!(pick_offer(&d, Some("other")).is_none(), "another project given on the command line");
        write(&d, UNTITLED, "{}", &meta("", 3)).unwrap();
        assert_eq!(pick_offer(&d, Some("other")).unwrap().key, UNTITLED);
        write(&d, "untitled-77-abc", "{}", &meta("", 9)).unwrap();
        assert_eq!(pick_offer(&d, Some("other")).unwrap().key, "untitled-77-abc");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn two_instances_never_share_the_untitled_key() {
        let mut a = Session::default();
        let mut b = Session::default();
        a.instance = "1-a".into();
        b.instance = "2-b".into();
        assert_ne!(a.key(), b.key());
        a.bound = Some("C:/x.editorgui.json".into());
        b.bound = Some("C:/x.editorgui.json".into());
        assert_eq!(a.key(), b.key(), "saved designs are keyed by path");
    }

    #[test]
    fn oversized_recovery_files_are_not_read_and_are_cleaned() {
        let d = tmp("big");
        let now = 1000u64;
        write(&d, "big", "{}", &meta("", now)).unwrap();
        let f = std::fs::OpenOptions::new().write(true).open(d.join("big.editorgui.json")).unwrap();
        f.set_len(MAX_BYTES + 1).unwrap();
        drop(f);
        assert!(read(&d, "big").is_none());
        assert_eq!(cleanup_stale(&d, now), 1);
        assert!(list(&d).is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }

    fn app_in(dir: &Path) -> crate::app::EditorApp {
        let mut a = crate::app::EditorApp::new(None);
        a.settings.read_only = true;
        a.recovery.dir = dir.to_path_buf();
        for _ in 0..4 {
            a.commit_state(false);
        }
        a
    }

    #[test]
    fn editor_autosaves_the_real_design_restores_it_and_cleans_up() {
        let d = tmp("app");
        let mut a = app_in(&d);
        a.recovery.interval = Duration::ZERO;
        // nothing changed: nothing is written
        a.autosave_tick(false);
        assert!(list(&d).is_empty());
        a.doc.title = "Edited".into();
        a.commit_state(false);
        a.autosave_tick(true);
        assert!(list(&d).is_empty(), "never while a drag or text edit is in progress");
        a.autosave_tick(false);
        let l = list(&d);
        assert_eq!(l.len(), 1);
        let ukey = a.recovery.key();
        assert!(is_untitled(&ukey) && ukey != UNTITLED, "never-saved designs get a per-instance key");
        assert_eq!(l[0].key, ukey);
        let (json, rmeta) = read(&d, &ukey).unwrap();
        assert_eq!(rmeta.title, "Edited");
        assert_eq!(crate::model::Doc::from_json_str(&json).unwrap().title, "Edited");
        // disabled in settings
        a.settings.no_autosave = true;
        a.doc.title = "Edited again".into();
        a.commit_state(false);
        a.autosave_tick(false);
        assert_eq!(read(&d, &ukey).unwrap().1.title, "Edited");
        // test mode clones are never autosaved
        a.settings.no_autosave = false;
        a.set_preview(true);
        a.doc.title = "Preview clone".into();
        a.autosave_tick(false);
        assert!(!a.autosave_now());
        assert_eq!(read(&d, &ukey).unwrap().1.title, "Edited");
        a.set_preview(false);
        // restore in a fresh session
        let mut b = app_in(&d);
        b.recovery_startup(None);
        assert!(b.recovery.offer.is_some());
        b.recovery_restore();
        assert_eq!(b.doc.title, "Edited");
        assert!(b.recovery.offer.is_none() && b.recovery.dirty(), "restored as an unsaved design");
        // a second crash must not lose it: still on disk until Save / clean exit
        let bkey = b.recovery.key();
        assert!(read(&d, &bkey).is_some());
        b.recovery_clean_exit();
        assert!(read(&d, &bkey).is_some(), "dirty design: the exit keeps the recovery (M1)");
        b.recovery.discard_ok = true;
        b.recovery_clean_exit();
        assert!(read(&d, &bkey).is_none(), "discarded: clean exit deletes the recovery");
        // Discard deletes it too, and a pending offer survives an exit that never answered it
        write(&d, "k", "{}", &meta("", 1)).unwrap();
        let mut c = app_in(&d);
        c.recovery.offer = pick_offer(&d, None);
        c.recovery.bound = Some("k".into());
        c.recovery_clean_exit();
        assert!(read(&d, "k").is_some() && read(&d, &key_for(Some("k"))).is_none());
        c.recovery_discard();
        assert!(read(&d, "k").is_none());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn restore_ignores_the_path_stored_in_the_meta_and_never_binds_it() {
        let d = tmp("m2");
        let json = serde_json::to_string(&crate::model::Doc::default()).unwrap();
        write(&d, "abc", &json, &meta(r"\\evil\share\x.editorgui.json", now_ms())).unwrap();
        let mut a = app_in(&d);
        let before = a.project_path.clone();
        a.recovery_startup(None);
        assert!(a.recovery.offer.is_some());
        a.recovery_restore();
        assert_eq!(a.project_path, before, "project path untouched");
        assert!(a.recovery.bound.is_none(), "restored as unsaved");
        assert!(a.recovery.dirty());
        assert!(is_untitled(&a.recovery.key()));
        assert!(read(&d, &a.recovery.key()).is_some());
        assert!(read(&d, "abc").is_none(), "the old entry is replaced by the instance copy");
        assert!(read(&d, &a.recovery.key()).unwrap().1.path.is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn dirty_design_is_never_dropped_by_exit_and_prompt_logic() {
        let d = tmp("m1");
        let mut a = app_in(&d);
        a.recovery.interval = Duration::ZERO;
        assert!(!a.has_unsaved_changes());
        a.doc.title = "Dirty".into();
        a.commit_state(false);
        assert!(a.has_unsaved_changes());
        a.recovery_clean_exit();
        assert_eq!(read(&d, &a.recovery.key()).unwrap().1.title, "Dirty", "final autosave on exit with changes");
        let exe = std::path::Path::new("x.exe");
        assert!(!a.request_restart(exe) && a.recovery.close_prompt && a.recovery.pending_restart.is_some());
        a.recovery.close_prompt = false;
        a.recovery.discard_ok = true;
        assert!(a.request_restart(exe));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn save_deletes_the_recovery() {
        let d = tmp("save");
        let proj = d.join("p.editorgui.json");
        let mut a = app_in(&d.join("rec"));
        a.recovery.interval = Duration::ZERO;
        a.doc.title = "X".into();
        a.commit_state(false);
        a.autosave_tick(false);
        assert_eq!(list(&d.join("rec")).len(), 1);
        a.project_path = proj.to_string_lossy().to_string();
        a.save_project();
        assert!(proj.is_file());
        assert!(list(&d.join("rec")).is_empty());
        assert!(!a.recovery.dirty());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn time_format_and_session_schedule() {
        assert_eq!(format_time(0), "1970-01-01 00:00 UTC");
        assert_eq!(format_time(1_700_000_000_000), "2023-11-14 22:13 UTC");
        assert_eq!(format_time(1_709_164_800_000 + 86_399_000), "2024-02-29 23:59 UTC");
        let mut s = Session { interval: Duration::from_secs(60), ..Session::default() };
        let t0 = Instant::now();
        let later = t0 + Duration::from_secs(61);
        assert!(!s.due(later, false, true), "nothing changed");
        s.note_edit();
        assert!(!s.due(t0, false, true), "interval not elapsed");
        assert!(!s.due(later, true, true), "never mid-drag or while typing");
        assert!(!s.due(later, false, false), "disabled in settings");
        assert!(s.due(later, false, true));
        s.wrote(later);
        assert!(!s.due(later + Duration::from_secs(120), false, true), "already autosaved");
        s.note_edit();
        assert!(s.due(later + Duration::from_secs(120), false, true));
        s.mark_saved();
        assert!(!s.dirty() && !s.due(later + Duration::from_secs(300), false, true));
    }
}
