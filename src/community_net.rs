//! Network resilience for the community: timeouts, the retry policy, distinct error messages and the two small
//! disk caches (last good catalog listing, thumbnails). Pure helpers: nothing here talks to the network, so the
//! rules are testable; `community.rs` applies them around its requests (always on worker threads, never on the UI thread).

use crate::i18n::{t, tf};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A cached catalog listing older than this is not shown.
pub const CATALOG_TTL_SECS: u64 = 24 * 3600;
/// A listing bigger than this is not cached.
pub const CATALOG_MAX_BYTES: usize = 1 << 20;
/// Total size cap of the thumbnail cache (oldest files are deleted first).
pub const THUMBS_MAX_BYTES: u64 = 24 << 20;
/// One cached thumbnail is never bigger than this.
pub const THUMB_MAX_FILE: usize = 2 << 20;

#[derive(Clone, Debug)]
pub struct NetCfg {
    /// Time allowed to open the connection (all requests).
    pub connect: Duration,
    /// Whole-request limit of a read (GET) of data.
    pub read: Duration,
    /// Whole-request limit of a write (POST / PATCH / DELETE): moderation of an upload can take about a minute.
    pub write: Duration,
    /// Whole-request limit of a file download.
    pub download: Duration,
    /// Extra attempts of an idempotent GET after a transient failure.
    pub retries: u32,
    /// Wait before the first retry (it triples each time).
    pub backoff: Duration,
    /// A `Retry-After` longer than this is not waited for: the error is reported instead.
    pub max_retry_wait: Duration,
}

impl Default for NetCfg {
    fn default() -> Self {
        NetCfg {
            connect: Duration::from_secs(8),
            read: Duration::from_secs(30),
            write: Duration::from_secs(170),
            download: Duration::from_secs(120),
            retries: 2,
            backoff: Duration::from_millis(400),
            max_retry_wait: Duration::from_secs(5),
        }
    }
}

/// A request that did not produce an HTTP answer.
#[derive(Clone, Debug)]
pub struct Failure {
    /// Message for the user (already in the current language).
    pub msg: String,
    /// Worth retrying (and a reason to show the cached listing): offline, timeout, connection cut.
    pub transient: bool,
}

pub fn offline_message() -> String {
    t("You appear to be offline. Check your internet connection.")
}

pub fn timeout_message() -> String {
    t("The server took too long to answer. Try again in a moment.")
}

pub fn unavailable_message() -> String {
    t("The server is not available right now.")
}

pub fn expired_message() -> String {
    t("Your session expired. Sign in again.")
}

/// 429 message; with the server's `Retry-After` it says how long to wait.
pub fn rate_limited_message(retry_after: Option<u64>) -> String {
    match retry_after {
        Some(s) => tf("Too many requests: wait {} s and try again.", &[&s.to_string()]),
        None => t("Too many requests: wait a moment."),
    }
}

/// True when `msg` is the "session expired" message (the UI then asks to sign in again).
pub fn is_expired(msg: &str) -> bool {
    msg == expired_message() || msg == "Your session expired. Sign in again."
}

/// `Retry-After` in seconds (only the numeric form; an HTTP date is ignored). Capped at one hour.
pub fn parse_retry_after(v: &str) -> Option<u64> {
    v.trim().parse::<u64>().ok().map(|s| s.min(3600))
}

pub fn classify(e: &ureq::Error) -> Failure {
    match e {
        ureq::Error::Timeout(_) => Failure { msg: timeout_message(), transient: true },
        ureq::Error::HostNotFound | ureq::Error::ConnectionFailed => Failure { msg: offline_message(), transient: true },
        ureq::Error::Protocol(_) => Failure { msg: unavailable_message(), transient: true },
        ureq::Error::Io(io) => match io.kind() {
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => Failure { msg: timeout_message(), transient: true },
            std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted | std::io::ErrorKind::UnexpectedEof | std::io::ErrorKind::BrokenPipe => Failure { msg: unavailable_message(), transient: true },
            _ => Failure { msg: offline_message(), transient: true },
        },
        _ => Failure { msg: unavailable_message(), transient: false },
    }
}

/// What happened to an attempt, for the retry decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    /// No HTTP answer (offline, timeout, cut connection).
    Transport,
    /// HTTP status and the `Retry-After` seconds.
    Status(u16, Option<u64>),
}

/// How long to wait before attempt `attempt + 1` (`attempt` counts the failures so far, from 0), or None to give up.
/// Only for idempotent requests: the caller never asks for writes.
pub fn retry_delay(cfg: &NetCfg, attempt: u32, outcome: Outcome) -> Option<Duration> {
    if attempt >= cfg.retries {
        return None;
    }
    let backoff = cfg.backoff.saturating_mul(3u32.saturating_pow(attempt));
    match outcome {
        Outcome::Transport | Outcome::Status(502..=504, _) => Some(backoff),
        Outcome::Status(429, Some(s)) => {
            let wait = Duration::from_secs(s);
            (wait <= cfg.max_retry_wait).then_some(wait.max(backoff))
        }
        Outcome::Status(429, None) => Some(backoff),
        _ => None,
    }
}

// ----------------------------------------------------------------- disk caches

/// Folder of the caches, next to `ajustes.json`.
pub fn cache_dir() -> PathBuf {
    crate::library::root_dir().parent().map(|p| p.join("cache")).unwrap_or_else(|| PathBuf::from("cache"))
}

fn short_hash(s: &str) -> String {
    Sha256::digest(s.as_bytes())[..8].iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn catalog_key(query: &str) -> String {
    short_hash(query)
}

#[derive(serde::Serialize)]
struct CatalogRef<'a, T> {
    saved_at: u64,
    items: &'a [T],
}

#[derive(serde::Deserialize)]
struct CatalogFile<T> {
    saved_at: u64,
    items: Vec<T>,
}

fn catalog_file(dir: &Path, key: &str) -> PathBuf {
    dir.join(format!("catalog-{}.json", key))
}

/// Saves the listing (atomically). False when it is too big (over 1 MB) or could not be written.
pub fn catalog_store<T: Serialize>(dir: &Path, key: &str, items: &[T], now: u64) -> bool {
    let Ok(json) = serde_json::to_vec(&CatalogRef { saved_at: now, items }) else { return false };
    if json.len() > CATALOG_MAX_BYTES {
        return false;
    }
    let _ = std::fs::create_dir_all(dir);
    crate::library::write_atomic(&catalog_file(dir, key), &json).is_ok()
}

/// The cached listing and its age in seconds; None when missing, corrupt, too big, from the future or older than 24 hours.
pub fn catalog_load<T: DeserializeOwned>(dir: &Path, key: &str, now: u64) -> Option<(Vec<T>, u64)> {
    let path = catalog_file(dir, key);
    if std::fs::metadata(&path).ok()?.len() > CATALOG_MAX_BYTES as u64 {
        return None;
    }
    let f: CatalogFile<T> = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    let age = now.checked_sub(f.saved_at)?;
    (age <= CATALOG_TTL_SECS).then_some((f.items, age))
}

/// Deletes catalog files older than the TTL (called now and then; harmless if the folder does not exist).
pub fn prune_catalogs(dir: &Path, now: u64) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.filter_map(|e| e.ok()) {
        let name = e.file_name().to_string_lossy().to_string();
        if !(name.starts_with("catalog-") && name.ends_with(".json")) {
            continue;
        }
        let ok = std::fs::read(e.path()).ok().and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok()).and_then(|v| v["saved_at"].as_u64()).is_some_and(|t| now.saturating_sub(t) <= CATALOG_TTL_SECS);
        if !ok {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

fn thumb_file(dir: &Path, public_path: &str) -> PathBuf {
    dir.join(format!("{}.png", short_hash(public_path)))
}

pub fn thumb_get(dir: &Path, public_path: &str) -> Option<Vec<u8>> {
    let p = thumb_file(dir, public_path);
    if std::fs::metadata(&p).ok()?.len() > THUMB_MAX_FILE as u64 {
        return None;
    }
    std::fs::read(p).ok()
}

/// Stores a thumbnail and then deletes the oldest ones while the folder is over `cap` bytes.
pub fn thumb_put(dir: &Path, public_path: &str, bytes: &[u8], cap: u64) -> std::io::Result<()> {
    if bytes.len() > THUMB_MAX_FILE {
        return Err(std::io::Error::other("thumbnail too large"));
    }
    std::fs::create_dir_all(dir)?;
    crate::library::write_atomic(&thumb_file(dir, public_path), bytes)?;
    let mut files: Vec<(std::time::SystemTime, u64, PathBuf)> = std::fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "png"))
        .filter_map(|e| e.metadata().ok().map(|m| (m.modified().unwrap_or(std::time::UNIX_EPOCH), m.len(), e.path())))
        .collect();
    let mut total: u64 = files.iter().map(|f| f.1).sum();
    if total > cap {
        files.sort_by_key(|f| f.0); // oldest first
        let keep = thumb_file(dir, public_path);
        for (_, len, p) in files {
            if total <= cap {
                break;
            }
            if p != keep && std::fs::remove_file(&p).is_ok() {
                total = total.saturating_sub(len);
            }
        }
    }
    Ok(())
}

// ----------------------------------------------------------------- terms, licences, claims (migration 0015)
// Pure rules of `supabase/CLIENT_CONTRACT.md`: nothing here talks to the network.

/// Short legal texts shipped with the app. Each starts with its version ("... version 1.0)").
pub const LEGAL_TERMS: &str = include_str!("../assets/legal/terms_summary.txt");
pub const LEGAL_GUIDELINES: &str = include_str!("../assets/legal/guidelines_summary.txt");
pub const LEGAL_PRIVACY: &str = include_str!("../assets/legal/privacy_summary.txt");
pub const LEGAL_COPYRIGHT: &str = include_str!("../assets/legal/copyright_summary.txt");

/// Full documents (opened in the browser on click).
pub const DOC_TERMS: &str = "https://github.com/deritu-kutsut/lienzo-ui/blob/main/docs/TERMS.md";
pub const DOC_GUIDELINES: &str = "https://github.com/deritu-kutsut/lienzo-ui/blob/main/docs/COMMUNITY_GUIDELINES.md";
pub const DOC_PRIVACY: &str = "https://github.com/deritu-kutsut/lienzo-ui/blob/main/docs/PRIVACY.md";
pub const DOC_COPYRIGHT: &str = "https://github.com/deritu-kutsut/lienzo-ui/blob/main/docs/COPYRIGHT.md";

/// Version written in the first line of an embedded text, e.g. "Lienzo UI Community - Terms of Use (summary, version 1.0)" gives "1.0".
pub fn text_version(text: &str) -> Option<String> {
    let line = text.lines().next()?;
    let rest = &line[line.find("version ")? + 8..];
    let v: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-').collect();
    (!v.is_empty()).then_some(v)
}

/// Version of the terms embedded in this build.
pub fn embedded_terms_version() -> String {
    text_version(LEGAL_TERMS).unwrap_or_default()
}

/// What the client must do about the terms (contract sections 1 and 2).
#[derive(Clone, Debug, PartialEq)]
pub enum TermsDecision {
    /// The server does not know terms (migration 0015 not applied) or we have no own state: do not ask for anything.
    NotApplicable,
    /// The terms version could not be read right now (network, server error): not the same as no terms. Uploads wait for a retry.
    Unknown,
    /// The profile already accepted the current version.
    UpToDate,
    /// Show the terms; send `accept_terms` with this version.
    NeedAccept(String),
    /// The server has a version this build has no text for: the user cannot be shown what they would accept, so they must update the app.
    AppOutdated(String),
}

/// `server_current`: version from `current_terms()` (None when unavailable). `profile_version`: `profiles.terms_version`
/// (None = never accepted a version, or a legacy "v0" acceptance: both ask again). `embedded`: version of the texts in this build.
pub fn terms_decision(server_current: Option<&str>, profile_version: Option<&str>, embedded: &str) -> TermsDecision {
    let Some(cur) = server_current.map(str::trim).filter(|v| !v.is_empty()) else { return TermsDecision::NotApplicable };
    if profile_version == Some(cur) {
        return TermsDecision::UpToDate;
    }
    if cur == embedded {
        TermsDecision::NeedAccept(cur.to_string())
    } else {
        TermsDecision::AppOutdated(cur.to_string())
    }
}

/// Version to send with `accept_terms` right after the profile is created: only when the text the user read IS the current one.
pub fn version_to_accept(server_current: Option<&str>, embedded: &str) -> Option<String> {
    server_current.map(str::trim).filter(|v| !v.is_empty() && *v == embedded).map(str::to_string)
}

/// Own moderation state read from the profile (contract section 7).
#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize)]
#[serde(default)]
pub struct OwnState {
    pub terms_version: Option<String>,
    pub terms_accepted_at: Option<String>,
    pub strikes: i64,
    pub upload_banned_until: Option<String>,
    pub suspended: bool,
}

/// What `OwnState` means for the Upload button.
#[derive(Clone, Debug, PartialEq)]
pub enum UploadBlock {
    None,
    Suspended,
    /// Banned until this timestamp (as the server sent it).
    Until(String),
    Permanent,
}

/// Seconds since the Unix epoch of an ISO 8601 timestamp WITH an offset (`2026-12-01T10:00:00Z`, `...+02:00`, `...-0530`,
/// optional fraction). None when it is malformed or has no offset (the zone is then unknown: it is not guessed).
pub fn iso_epoch(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 20 || !s.is_ascii() || b[4] != b'-' || b[7] != b'-' || !matches!(b[10], b'T' | b't' | b' ') || b[13] != b':' || b[16] != b':' {
        return None;
    }
    let num = |a: usize, z: usize| s.get(a..z).filter(|x| x.bytes().all(|c| c.is_ascii_digit())).and_then(|x| x.parse::<i64>().ok());
    let (y, mo, d, h, mi, se) = (num(0, 4)?, num(5, 7)?, num(8, 10)?, num(11, 13)?, num(14, 16)?, num(17, 19)?);
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) || h > 23 || mi > 59 || se > 60 {
        return None;
    }
    let mut i = 19;
    if b.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while b.get(i).is_some_and(|c| c.is_ascii_digit()) {
            i += 1;
        }
        if i == start {
            return None;
        }
    }
    let offset = match b.get(i)? {
        b'Z' | b'z' if i + 1 == b.len() => 0,
        sign @ (b'+' | b'-') => {
            let rest = &s[i + 1..];
            let (oh, om) = match rest.len() {
                5 if rest.as_bytes()[2] == b':' => (rest[..2].parse::<i64>().ok()?, rest[3..].parse::<i64>().ok()?),
                4 => (rest[..2].parse::<i64>().ok()?, rest[2..].parse::<i64>().ok()?),
                2 => (rest.parse::<i64>().ok()?, 0),
                _ => return None,
            };
            if oh > 23 || om > 59 || !rest.bytes().all(|c| c.is_ascii_digit() || c == b':') {
                return None;
            }
            let o = oh * 3600 + om * 60;
            if *sign == b'-' { -o } else { o }
        }
        _ => return None,
    };
    // days from civil (Howard Hinnant)
    let yy = if mo <= 2 { y - 1 } else { y };
    let era = yy.div_euclid(400);
    let yoe = yy - era * 400;
    let doy = (153 * (if mo > 2 { mo - 3 } else { mo + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some(days * 86400 + h * 3600 + mi * 60 + se - offset)
}

/// A ban date starting with `9999-` is permanent; a date in the past is over. Both dates are parsed with their own UTC
/// offset (a PostgREST timestamp may come as `+02:00`); a ban date that cannot be read counts as still active (not
/// hiding a ban the server reported), shown as a plain sanitised text.
pub fn upload_block(s: &OwnState, now_iso: &str) -> UploadBlock {
    if s.suspended {
        return UploadBlock::Suspended;
    }
    match s.upload_banned_until.as_deref() {
        Some(u) if u.starts_with("9999-") => UploadBlock::Permanent,
        Some(u) => match (iso_epoch(u), iso_epoch(now_iso)) {
            (Some(until), Some(now)) if until > now => UploadBlock::Until(crate::recovery::format_time(until as u64 * 1000)),
            (Some(_), Some(_)) => UploadBlock::None,
            // unreadable ban date (or clock): keep the ban visible
            _ => UploadBlock::Until(u.chars().filter(|c| c.is_ascii_graphic() || *c == ' ').take(32).collect()),
        },
        None => UploadBlock::None,
    }
}

pub fn accept_terms_body(version: &str) -> serde_json::Value {
    serde_json::json!({ "p_version": version })
}

/// Allowed licences: (server code, label). "All rights reserved" does not exist on purpose.
pub const LICENSES: &[(&str, &str)] = &[
    ("cc0", "CC0"),
    ("public-domain", "Public domain"),
    ("cc-by", "CC BY"),
    ("cc-by-sa", "CC BY-SA"),
    ("cc-by-nc", "CC BY-NC"),
    ("mit", "MIT"),
    ("apache-2", "Apache 2.0"),
    ("ofl", "SIL OFL"),
    ("proprietary-free-use", "Free use (proprietary)"),
];

/// Index into `LICENSES` suggested for a kind: fonts `ofl`, plugins `mit`, the rest `cc-by`.
pub fn default_license(kind: &str) -> usize {
    let code = match kind {
        "font" => "ofl",
        "plugin" => "mit",
        _ => "cc-by",
    };
    LICENSES.iter().position(|(c, _)| *c == code).unwrap_or(0)
}

pub fn license_label(code: &str) -> Option<&'static str> {
    LICENSES.iter().find(|(c, _)| *c == code).map(|(_, l)| *l)
}

/// True for addresses a public source link must never point at (loopback, private, link-local, CGNAT, reserved...).
fn non_public_ipv4(ip: std::net::Ipv4Addr) -> bool {
    let o = ip.octets();
    ip.is_loopback() || ip.is_private() || ip.is_link_local() || ip.is_unspecified() || ip.is_broadcast() || ip.is_documentation() || o[0] == 0 || (o[0] == 100 && (64..128).contains(&o[1])) || o[0] >= 224
}

/// Strict split of `http(s)://host[:port]/...` into (host, rest after the authority). None for anything odd: a
/// backslash, user info, `%00`, a port over 65535, an empty / non-ASCII-DNS host, `localhost` or a local name, an IP
/// literal (v6, or v4 that is private / loopback / written in a non-standard form) or a name without a dot.
fn split_source_url(u: &str) -> Option<(String, &str)> {
    if u.is_empty() || u.chars().count() > 300 || !u.chars().all(|c| c.is_ascii_graphic()) || u.contains('\\') || u.to_ascii_lowercase().contains("%00") {
        return None;
    }
    let rest = u.strip_prefix("https://").or_else(|| u.strip_prefix("http://"))?;
    let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let (authority, tail) = rest.split_at(end);
    if authority.is_empty() || authority.contains('@') || authority.starts_with('[') {
        return None;
    }
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (h, Some(p)),
        None => (authority, None),
    };
    if let Some(p) = port {
        if p.is_empty() || p.len() > 5 || !p.bytes().all(|b| b.is_ascii_digit()) || p.parse::<u16>().ok().filter(|n| *n > 0).is_none() {
            return None;
        }
    }
    let host = host.trim_end_matches('.').to_ascii_lowercase();
    if host.is_empty() || host.len() > 253 || !host.contains('.') {
        return None;
    }
    let labels: Vec<&str> = host.split('.').collect();
    if labels.iter().any(|l| l.is_empty() || l.len() > 63 || l.starts_with('-') || l.ends_with('-') || !l.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')) {
        return None;
    }
    let last = labels[labels.len() - 1];
    if last.bytes().all(|b| b.is_ascii_digit()) || last.starts_with("0x") {
        // an IPv4 literal: only the plain dotted form (the browser would read the others as other addresses)
        let ip: std::net::Ipv4Addr = host.parse().ok()?;
        if non_public_ipv4(ip) {
            return None;
        }
    }
    if host == "localhost" || ["localhost", "local", "internal", "localdomain", "lan", "home", "corp"].iter().any(|s| last == *s) {
        return None;
    }
    Some((host, tail))
}

/// `http(s)://host[:port]/...`: strict check, see [`split_source_url`]. At most 300 characters, no spaces or control characters.
pub fn valid_source_url(u: &str) -> bool {
    split_source_url(u).is_some()
}

/// How a (valid) source link is shown: the complete host first, then as much of the rest as fits (never a cut host).
pub fn source_display(u: &str) -> String {
    let Some((host, tail)) = split_source_url(u) else { return String::new() };
    let port = u.strip_prefix("https://").or_else(|| u.strip_prefix("http://")).and_then(|r| r.split(['/', '?', '#']).next()).and_then(|a| a.rsplit_once(':')).map(|(_, p)| format!(":{}", p)).unwrap_or_default();
    let mut s = format!("{}{}", host, port);
    let t: String = tail.chars().take(32).collect();
    s.push_str(&t);
    if tail.chars().count() > 32 {
        s.push('…');
    }
    s
}

/// Fields of the upload row for migration 0015. Empty when no licence was chosen (server without 0015, or old behaviour).
/// `license` is only sent together with `rights_confirmed_at` (contract section 3); an empty `source_url` is not sent.
pub fn upload_extras(license: Option<&str>, source_url: &str, rights: bool, now_iso: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut m = serde_json::Map::new();
    let Some(code) = license.filter(|c| license_label(c).is_some()) else { return m };
    if !rights {
        return m;
    }
    m.insert("license".into(), code.into());
    m.insert("rights_confirmed_at".into(), now_iso.into());
    let src = source_url.trim();
    if !src.is_empty() {
        m.insert("source_url".into(), src.into());
    }
    m
}

/// The three keys that an older database does not have: a 400 about one of them means "migration 0015 is not applied".
pub const EXTRA_COLUMNS: [&str; 3] = ["license", "source_url", "rights_confirmed_at"];

/// True when the PostgREST answer says a column is unknown (PGRST204 / 42703 / "Could not find the ... column").
pub fn is_missing_column(state: u16, body: &str) -> bool {
    state == 400 && (body.contains("PGRST204") || body.contains("42703") || body.contains("Could not find")) && EXTRA_COLUMNS.iter().any(|c| body.contains(c))
}

/// The eight report reasons of the new client: (server code, label). Copyright goes through `file_copyright_claim`.
pub const REPORT_REASONS: &[(&str, &str)] = &[
    ("copyright", "Copyright"),
    ("adult", "Unmarked 18+ content"),
    ("malware", "Malware or viruses"),
    ("hate", "Hate or harassment"),
    ("spam", "Spam"),
    ("impersonation", "Impersonation"),
    ("personal_data", "Personal data"),
    ("other", "Other"),
];

/// Spelling a server without migration 0015 understands (its check constraint only knows the old reasons).
pub fn legacy_reason(r: &str) -> &'static str {
    match r {
        "copyright" => "copyright",
        "adult" => "adult_content",
        "malware" => "malware",
        "hate" => "hate_or_harassment",
        _ => "other",
    }
}

pub fn report_body(item: &str, reporter: &str, reason: &str, legacy: bool) -> serde_json::Value {
    serde_json::json!({ "item": item, "reporter": reporter, "reason": if legacy { legacy_reason(reason) } else { reason } })
}

/// A check-constraint violation (SQLSTATE 23514) on `reports.reason`: the server does not know the new spellings yet.
pub fn is_reason_rejected(state: u16, body: &str) -> bool {
    state == 400 && body.contains("23514")
}

/// Why a copyright claim form cannot be sent yet (None = it can). Same limits as the server.
pub fn claim_problem(description: &str, claimant: &str, contact: &str) -> Option<&'static str> {
    let n = description.trim().chars().count();
    if !(10..=2000).contains(&n) {
        return Some("description");
    }
    if claimant.trim().chars().count() > 200 {
        return Some("claimant");
    }
    if contact.trim().chars().count() > 200 {
        return Some("contact");
    }
    None
}

/// Body of `file_copyright_claim`; the claimant and the contact are omitted when empty (the public name is used).
pub fn claim_body(item: &str, description: &str, claimant: &str, contact: &str) -> serde_json::Value {
    let mut m = serde_json::Map::new();
    m.insert("p_item".into(), item.into());
    m.insert("p_description".into(), description.trim().into());
    if !claimant.trim().is_empty() {
        m.insert("p_claimant".into(), claimant.trim().into());
    }
    if !contact.trim().is_empty() {
        m.insert("p_contact".into(), contact.trim().into());
    }
    serde_json::Value::Object(m)
}

/// Machine code of an error answer: `hint` of PostgREST, else the `message` prefix before the first ": ", else the `code` of an edge function.
pub fn error_code(body: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let known = |c: &str| !c.is_empty() && c.len() <= 40 && c.chars().all(|ch| ch.is_ascii_lowercase() || ch == '_');
    if let Some(h) = v["hint"].as_str().filter(|h| known(h)) {
        return Some(h.to_string());
    }
    if let Some(p) = v["message"].as_str().and_then(|m| m.split_once(": ")).map(|(p, _)| p).filter(|p| known(p)) {
        return Some(p.to_string());
    }
    v["code"].as_str().filter(|c| known(c)).map(str::to_string)
}

/// Longest server message shown to the user.
pub const ERROR_DETAIL_MAX: usize = 200;

/// The readable part of a PostgREST message ("upload_banned: Uploads are disabled until ...") without the machine prefix.
pub fn error_detail(body: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(body).ok()?;
    let m = v["message"].as_str()?;
    // a server text shown to the user: capped, the caller removes control / spoofing characters
    Some(m.split_once(": ").map(|(_, d)| d).unwrap_or(m).chars().take(ERROR_DETAIL_MAX).collect())
}

/// Fixed user messages for the codes of the contract. `upload_banned`, `claim_invalid` and the like carry the server's own text instead.
pub fn code_message(code: &str) -> Option<String> {
    Some(match code {
        "terms_required" => t("Accept the Terms of Use to share."),
        "license_invalid" => t("Pick a license from the list."),
        "source_url_invalid" => t("Use an http or https link, up to 300 characters, no spaces."),
        "rights_required" => t("Confirm that you own this or may share it."),
        "terms_unknown_version" | "terms_outdated" => t("The Terms of Use changed. Close this window and open it again to read the new version."),
        "profile_required" => t("Create your profile first."),
        "sign_in_required" => expired_message(),
        "claim_rate_limited" => t("Too many claims today. Use the Copyright notice template on GitHub."),
        "claims_blocked" => t("Claims are blocked for now. Use the Copyright notice template on GitHub."),
        "item_not_found" => t("That item is already under review."),
        "account_suspended" => t("Your account is suspended."),
        _ => return None,
    })
}

/// Codes whose message is the server's own readable text (it says until when, or which field is wrong).
pub fn is_detail_code(code: &str) -> bool {
    matches!(code, "upload_banned" | "claim_invalid")
}

/// The field to highlight in the upload form for a message produced by `code_message`.
pub fn field_of_message(msg: &str) -> Option<&'static str> {
    [("license_invalid", "license"), ("source_url_invalid", "source"), ("rights_required", "rights")].iter().find(|(c, _)| code_message(c).as_deref() == Some(msg)).map(|(_, f)| *f)
}

/// True when `msg` is the "terms required" message (the UI then opens the terms screen).
pub fn is_terms_required(msg: &str) -> bool {
    code_message("terms_required").as_deref() == Some(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(n: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("lienzo_net_{}_{}", n, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn retry_after_parsing() {
        assert_eq!(parse_retry_after("3"), Some(3));
        assert_eq!(parse_retry_after(" 120 "), Some(120));
        assert_eq!(parse_retry_after("999999"), Some(3600));
        assert_eq!(parse_retry_after("Wed, 21 Oct 2026 07:28:00 GMT"), None);
        assert_eq!(parse_retry_after(""), None);
    }

    #[test]
    fn retry_policy() {
        let c = NetCfg { backoff: Duration::from_millis(100), ..NetCfg::default() };
        assert_eq!(retry_delay(&c, 0, Outcome::Transport), Some(Duration::from_millis(100)));
        assert_eq!(retry_delay(&c, 1, Outcome::Transport), Some(Duration::from_millis(300)), "backoff grows");
        assert_eq!(retry_delay(&c, 2, Outcome::Transport), None, "bounded");
        assert!(retry_delay(&c, 0, Outcome::Status(503, None)).is_some());
        assert!(retry_delay(&c, 0, Outcome::Status(500, None)).is_none(), "a plain 500 is not retried");
        assert!(retry_delay(&c, 0, Outcome::Status(404, None)).is_none());
        assert!(retry_delay(&c, 0, Outcome::Status(401, None)).is_none());
        assert_eq!(retry_delay(&c, 0, Outcome::Status(429, Some(2))), Some(Duration::from_secs(2)));
        assert_eq!(retry_delay(&c, 0, Outcome::Status(429, Some(600))), None, "too long to wait: report it");
    }

    #[test]
    fn messages_are_distinct() {
        let all = [offline_message(), timeout_message(), unavailable_message(), expired_message(), rate_limited_message(None), rate_limited_message(Some(7))];
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a, b);
            }
        }
        assert!(rate_limited_message(Some(7)).contains('7'));
        assert!(is_expired(&expired_message()));
        assert!(!is_expired(&offline_message()));
    }

    #[test]
    fn catalog_cache_roundtrip_ttl_and_size() {
        let d = tmp("cat");
        let items = vec!["a".to_string(), "b".to_string()];
        assert!(catalog_store(&d, "k", &items, 1000));
        let (got, age): (Vec<String>, u64) = catalog_load(&d, "k", 1060).unwrap();
        assert_eq!((got, age), (items.clone(), 60));
        assert!(catalog_load::<String>(&d, "k", 1000 + CATALOG_TTL_SECS).is_some(), "exactly 24 h is still valid");
        assert!(catalog_load::<String>(&d, "k", 1001 + CATALOG_TTL_SECS).is_none(), "older than 24 h");
        assert!(catalog_load::<String>(&d, "k", 500).is_none(), "from the future");
        assert!(catalog_load::<String>(&d, "missing", 1000).is_none());
        std::fs::write(d.join("catalog-bad.json"), "{not json").unwrap();
        assert!(catalog_load::<String>(&d, "bad", 1000).is_none());
        // over 1 MB: not written, and the previous good copy stays
        let huge: Vec<String> = (0..20000).map(|_| "x".repeat(60)).collect();
        assert!(!catalog_store(&d, "k", &huge, 2000));
        assert_eq!(catalog_load::<String>(&d, "k", 1060).unwrap().0.len(), 2);
        prune_catalogs(&d, 1000 + CATALOG_TTL_SECS + 10);
        assert!(!catalog_file(&d, "k").exists() && !catalog_file(&d, "bad").exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn thumbnail_cache_is_size_capped_and_keeps_the_newest() {
        let d = tmp("thumbs");
        let png = vec![7u8; 1000];
        for i in 0..10 {
            thumb_put(&d, &format!("p/{}.png", i), &png, 4500).unwrap();
            std::thread::sleep(Duration::from_millis(15));
        }
        let total: u64 = std::fs::read_dir(&d).unwrap().map(|e| e.unwrap().metadata().unwrap().len()).sum();
        assert!(total <= 4500, "{}", total);
        assert!(thumb_get(&d, "p/9.png").is_some(), "the newest survives");
        assert!(thumb_get(&d, "p/0.png").is_none(), "the oldest was evicted");
        assert!(thumb_put(&d, "big", &vec![0u8; THUMB_MAX_FILE + 1], 1 << 30).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn embedded_texts_carry_one_version() {
        let v = embedded_terms_version();
        assert!(!v.is_empty());
        for txt in [LEGAL_TERMS, LEGAL_GUIDELINES, LEGAL_PRIVACY, LEGAL_COPYRIGHT] {
            assert_eq!(text_version(txt), Some(v.clone()), "all four texts share the version");
        }
        assert_eq!(text_version("Title (summary, version 2.1-beta)"), Some("2.1-beta".into()));
        assert_eq!(text_version("no number here"), None);
    }

    #[test]
    fn terms_decision_rules() {
        use TermsDecision::*;
        assert_eq!(terms_decision(None, None, "1.0"), NotApplicable, "server without 0015");
        assert_eq!(terms_decision(Some(" "), None, "1.0"), NotApplicable);
        assert_eq!(terms_decision(Some("1.0"), Some("1.0"), "1.0"), UpToDate);
        assert_eq!(terms_decision(Some("1.0"), None, "1.0"), NeedAccept("1.0".into()), "never accepted, or a legacy v0 acceptance");
        assert_eq!(terms_decision(Some("1.1"), Some("1.0"), "1.1"), NeedAccept("1.1".into()), "re-accept when the version changes");
        assert_eq!(terms_decision(Some("1.1"), Some("1.1"), "1.0"), UpToDate, "already accepted the current one");
        assert_eq!(terms_decision(Some("2.0"), Some("1.0"), "1.0"), AppOutdated("2.0".into()), "no text to show for it");
        assert_eq!(version_to_accept(Some("1.0"), "1.0"), Some("1.0".into()));
        assert_eq!(version_to_accept(Some("1.1"), "1.0"), None);
        assert_eq!(version_to_accept(None, "1.0"), None);
    }

    #[test]
    fn upload_block_honours_the_utc_offset_of_both_dates() {
        assert_eq!(iso_epoch("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(iso_epoch("1970-01-01T02:00:00+02:00"), Some(0));
        assert_eq!(iso_epoch("1969-12-31T18:30:00-0530"), Some(0));
        assert_eq!(iso_epoch("1970-01-01T00:00:01.250Z"), Some(1));
        assert_eq!(iso_epoch("2026-11-19T10:00:00Z"), Some(1_794_996_000 - 1_794_996_000 + iso_epoch("2026-11-19T10:00:00+00:00").unwrap()));
        for bad in ["", "2026-11-19T10:00:00", "2026-13-19T10:00:00Z", "2026-11-19T25:00:00Z", "2026-11-19 10:00", "nope", "2026-11-19T10:00:00+99:00", "2026-11-19T10:00:00ZZ"] {
            assert_eq!(iso_epoch(bad), None, "{}", bad);
        }
        let now = "2026-11-19T10:00:00Z";
        let mut s = OwnState::default();
        // 11:30+02:00 is 09:30 UTC: already over (the old text comparison said it was still active)
        s.upload_banned_until = Some("2026-11-19T11:30:00+02:00".into());
        assert_eq!(upload_block(&s, now), UploadBlock::None);
        // 09:30-02:00 is 11:30 UTC: still active, and shown in UTC
        s.upload_banned_until = Some("2026-11-19T09:30:00-02:00".into());
        assert_eq!(upload_block(&s, now), UploadBlock::Until("2026-11-19 11:30 UTC".into()));
        // the clock may also carry an offset
        assert_eq!(upload_block(&s, "2026-11-19T13:00:00+02:00"), UploadBlock::Until("2026-11-19 11:30 UTC".into()), "11:00 UTC now");
        assert_eq!(upload_block(&s, "2026-11-19T14:00:00+02:00"), UploadBlock::None, "12:00 UTC now");
        // a ban date with no zone is not guessed: it stays visible as text
        s.upload_banned_until = Some("2020-01-01T00:00:00".into());
        assert!(matches!(upload_block(&s, now), UploadBlock::Until(_)));
    }

    #[test]
    fn own_state_blocks_uploads() {
        let now = "2026-11-19T10:00:00Z";
        let mut s = OwnState::default();
        assert_eq!(upload_block(&s, now), UploadBlock::None);
        s.upload_banned_until = Some("2026-12-01T10:00:00+00:00".into());
        assert_eq!(upload_block(&s, now), UploadBlock::Until("2026-12-01 10:00 UTC".into()));
        s.upload_banned_until = Some("2026-11-01T10:00:00+00:00".into());
        assert_eq!(upload_block(&s, now), UploadBlock::None, "an expired ban");
        s.upload_banned_until = Some("9999-12-31T00:00:00+00:00".into());
        assert_eq!(upload_block(&s, now), UploadBlock::Permanent);
        s.suspended = true;
        assert_eq!(upload_block(&s, now), UploadBlock::Suspended);
        let p: OwnState = serde_json::from_str(r#"{"terms_version":null,"terms_accepted_at":"2026-05-01T10:00:00+00:00","strikes":2,"upload_banned_until":null,"suspended":false}"#).unwrap();
        assert_eq!((p.terms_version, p.strikes, p.suspended), (None, 2, false));
    }

    #[test]
    fn upload_fields_follow_the_contract() {
        let now = "2026-11-19T10:00:00Z";
        let m = upload_extras(Some("cc-by"), " https://github.com/me/neon-hud ", true, now);
        assert_eq!(m["license"], "cc-by");
        assert_eq!(m["source_url"], "https://github.com/me/neon-hud");
        assert_eq!(m["rights_confirmed_at"], now);
        assert!(upload_extras(Some("cc-by"), "", false, now).is_empty(), "a licence is never sent without the rights confirmation");
        assert!(upload_extras(Some("all-rights-reserved"), "", true, now).is_empty(), "unknown licence");
        assert!(upload_extras(None, "https://x.org", true, now).is_empty());
        assert!(!upload_extras(Some("mit"), "  ", true, now).contains_key("source_url"));
        assert_eq!(LICENSES[default_license("font")].0, "ofl");
        assert_eq!(LICENSES[default_license("plugin")].0, "mit");
        assert_eq!(LICENSES[default_license("theme")].0, "cc-by");
        assert_eq!(LICENSES.len(), 9);
    }

    #[test]
    fn source_url_rules() {
        assert!(valid_source_url("https://github.com/me/neon-hud"));
        assert!(valid_source_url("http://example.org"));
        assert!(!valid_source_url(""));
        assert!(!valid_source_url("ftp://example.org/a"));
        assert!(!valid_source_url("https://user:pw@example.org/a"));
        assert!(!valid_source_url("https://example.org/a b"));
        assert!(!valid_source_url("https://"));
        assert!(!valid_source_url(&format!("https://e.org/{}", "a".repeat(300))));
        assert!(valid_source_url(&format!("https://e.org/{}", "a".repeat(280))));
        // strict parsing (B1)
        for bad in [
            r"https://example.org\@evil.com/", r"https://example.org/a\b", "https://user@example.org/", "https://example.org@evil.com/",
            "http://localhost/a", "http://localhost:8080/", "http://app.localhost/", "http://127.0.0.1/", "http://127.1/", "http://2130706433/",
            "http://0x7f.0.0.1/", "http://10.0.0.5/x", "http://192.168.1.1/", "http://172.16.0.1/", "http://169.254.169.254/latest", "http://0.0.0.0/",
            "http://100.64.0.1/", "http://[::1]/", "http://[::ffff:127.0.0.1]/", "https://example.org:99999/", "https://example.org:0/", "https://example.org:/",
            "https://example.org:80a/", "https://example.org/a%00b", "https://example.org/a%00", "https://intranet/", "https://printer.local/", "https://a..b.com/",
            "https://-a.com/", "https://exa_mple.com/", "https://ex ample.com/", "https:///a", "https://.com/", "https://ex\u{e1}mple.com/",
        ] {
            assert!(!valid_source_url(bad), "{}", bad);
        }
        for good in ["https://example.org:8443/a?b=c#d", "http://8.8.8.8/x", "https://sub.example.co.uk", "https://EXAMPLE.org/Path"] {
            assert!(valid_source_url(good), "{}", good);
        }
        // the host is never cut when displayed
        let long = format!("https://a-very-long-host-name-that-goes-on-and-on.example.org/{}", "p".repeat(100));
        let shown = source_display(&long);
        assert!(shown.starts_with("a-very-long-host-name-that-goes-on-and-on.example.org/") && shown.ends_with('…'), "{}", shown);
        assert_eq!(source_display("https://example.org:8443/x"), "example.org:8443/x");
        assert_eq!(source_display("http://localhost/"), "");
    }

    #[test]
    fn old_servers_are_detected_and_tolerated() {
        assert!(is_missing_column(400, r#"{"code":"PGRST204","message":"Could not find the 'license' column of 'community_items' in the schema cache"}"#));
        assert!(!is_missing_column(400, r#"{"hint":"license_invalid","message":"license_invalid: nope"}"#));
        assert!(!is_missing_column(500, "PGRST204 license"));
        assert!(is_reason_rejected(400, r#"{"code":"23514","message":"new row for relation \"reports\" violates check constraint"}"#));
        assert!(!is_reason_rejected(409, r#"{"code":"23505"}"#));
        assert_eq!(legacy_reason("adult"), "adult_content");
        assert_eq!(legacy_reason("hate"), "hate_or_harassment");
        assert_eq!(legacy_reason("spam"), "other");
        assert_eq!(legacy_reason("copyright"), "copyright");
        assert_eq!(report_body("i", "u", "hate", true)["reason"], "hate_or_harassment");
        assert_eq!(report_body("i", "u", "hate", false)["reason"], "hate");
        assert_eq!(REPORT_REASONS.len(), 8);
        assert!(REPORT_REASONS.iter().all(|(c, _)| *c != "adult_content" && *c != "hate_or_harassment"), "the old spellings are not sent by the new client");
    }

    #[test]
    fn claim_form_and_payload() {
        assert_eq!(claim_problem("short", "", ""), Some("description"));
        assert_eq!(claim_problem(&"x".repeat(2001), "", ""), Some("description"));
        assert_eq!(claim_problem("this is my artwork, copied", "", ""), None);
        assert_eq!(claim_problem("this is my artwork, copied", &"n".repeat(201), ""), Some("claimant"));
        assert_eq!(claim_problem("this is my artwork, copied", "", &"c".repeat(201)), Some("contact"));
        let b = claim_body("uuid", "  this is my artwork, copied  ", " ", "me@x.org");
        assert_eq!(b["p_item"], "uuid");
        assert_eq!(b["p_description"], "this is my artwork, copied");
        assert!(b.get("p_claimant").is_none(), "empty claimant is omitted");
        assert_eq!(b["p_contact"], "me@x.org");
        assert_eq!(accept_terms_body("1.0"), serde_json::json!({ "p_version": "1.0" }));
    }

    #[test]
    fn error_codes_are_read_from_hint_then_message() {
        assert_eq!(error_code(r#"{"code":"P0001","details":null,"hint":"terms_required","message":"terms_required: Accept the current Terms"}"#), Some("terms_required".into()));
        assert_eq!(error_code(r#"{"code":"P0001","message":"upload_banned: Uploads are disabled until 2026-12-01 10:00 UTC."}"#), Some("upload_banned".into()));
        assert_eq!(error_code(r#"{"v":2,"error":"x","code":"claim_rate_limited"}"#), Some("claim_rate_limited".into()));
        assert_eq!(error_code(r#"{"message":"Daily upload limit reached (20)."}"#), None, "plain errors have no code");
        assert_eq!(error_code("not json"), None);
        assert_eq!(error_detail(r#"{"message":"upload_banned: Uploads are disabled for this account."}"#), Some("Uploads are disabled for this account.".into()));
        let long = format!(r#"{{"message":"upload_banned: {}"}}"#, "x".repeat(5000));
        assert_eq!(error_detail(&long).unwrap().chars().count(), ERROR_DETAIL_MAX);
        for c in ["terms_required", "license_invalid", "source_url_invalid", "rights_required", "terms_unknown_version", "terms_outdated", "profile_required", "sign_in_required", "claim_rate_limited", "claims_blocked", "item_not_found", "account_suspended"] {
            assert!(code_message(c).is_some(), "{}", c);
        }
        assert!(code_message("upload_banned").is_none() && is_detail_code("upload_banned"));
        assert_eq!(field_of_message(&code_message("license_invalid").unwrap()), Some("license"));
        assert_eq!(field_of_message(&code_message("rights_required").unwrap()), Some("rights"));
        assert!(is_terms_required(&code_message("terms_required").unwrap()));
        assert!(!is_terms_required("something else"));
    }
}
