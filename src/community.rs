//! Lienzo UI community: browse, install, upload and report plugins, images, fonts, templates and elements
//! without leaving the app. The server is Supabase (see supabase/ and docs/COMMUNITY.md).
//!
//! Privacy and security on the app side:
//!   * No connection is made until you open the Community tab.
//!   * Only the project's PUBLIC key is used; with RLS it opens nothing that isn't allowed. The `service_role` key never goes in.
//!   * Sign-in is through GitHub or Google (OAuth with PKCE and a temporary local port); the app never sees passwords.
//!   * The session (refresh token) is stored in the Windows Credential Manager, not in a file.
//!   * Everything downloaded is verified (size + SHA-256 + the same rules as the server) before being installed.
//!   * +18 content is hidden by default and only shown if you enable it with your account.

use crate::app::EditorApp;
use crate::community_net::{self as net, NetCfg, Outcome};
use crate::community_validate::{self as val, Package};
use crate::i18n::{t, tf};
use crate::theme::{ACCENT, MUTED, WARN};
use base64::Engine;
use eframe::egui::{self, RichText};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Supabase project address and its PUBLIC key (designed to ship in the app).
pub const URL: &str = "https://btotvbdhoyfjpdixlcrw.supabase.co";
pub const PUBLIC_KEY: &str = "sb_publishable_UNvWtzs_M8N_N6fLrZ9VCQ_E6hkxXrL";
const LOCAL_PORT: u16 = 53682;
const CREDENTIALS_SERVICE: &str = "LienzoUI";

// ───────────────────────────── data ─────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct Session {
    pub access: String,
    pub renewal: String,
    pub uid: String,
    pub expires: u64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub public_name: String,
    pub show_18: bool,
}

#[derive(Clone, Debug, Default, Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Item {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub is_18: bool,
    pub public_path: Option<String>,
    pub sha256: Option<String>,
    pub bytes: Option<i64>,
    pub downloads: i64,
    pub version: String,
    pub author: String,
    pub author_name: String,
    pub public_icon: Option<String>,
    pub public_preview: Option<String>,
    pub hearts: i64,
    /// Licence code and source link (migration 0015; null on older rows and servers).
    pub license: Option<String>,
    pub source_url: Option<String>,
}

/// Public profile of a creator (what the person chooses to show; no email or preferences).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ProfilePublic {
    pub id: String,
    pub public_name: String,
    pub bio: String,
    pub github: Option<String>,
    pub links: std::collections::BTreeMap<String, String>,
    pub public_avatar: Option<String>,
    pub public_banner: Option<String>,
    pub avatar_pending: bool,
    pub banner_pending: bool,
    pub followers: i64,
    pub hearts_received: i64,
}

/// Networks supported in the profile: (key, display name, allowed domains). Same as on the server.
const NETWORKS: &[(&str, &str, &[&str])] = &[
    ("x", "X", &["x.com", "twitter.com"]),
    ("youtube", "YouTube", &["youtube.com", "www.youtube.com"]),
    ("itch", "itch.io", &["itch.io"]),
    ("twitch", "Twitch", &["twitch.tv", "www.twitch.tv"]),
    ("bluesky", "Bluesky", &["bsky.app"]),
    ("instagram", "Instagram", &["instagram.com", "www.instagram.com"]),
];

/// https only, no user or port, and from an allowed domain (prevents phishing or malware links).
fn valid_link(red: &str, url: &str) -> bool {
    let Some((_, _, domains)) = NETWORKS.iter().find(|(k, _, _)| *k == red) else { return false };
    if url.len() > 200 || !url.chars().all(|c| c.is_ascii_graphic()) {
        return false;
    }
    let Some(rest) = url.strip_prefix("https://") else { return false };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    if host.contains('@') || host.contains(':') {
        return false;
    }
    let h = host.to_lowercase();
    domains.contains(&h.as_str()) || (red == "itch" && h.strip_suffix(".itch.io").is_some_and(|u| !u.is_empty() && u.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')))
}

fn github_valid(u: &str) -> bool {
    !u.is_empty() && u.len() <= 39 && !u.starts_with('-') && u.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn valid_public_path(path: &str) -> bool {
    path.split('/').all(|p| !p.is_empty() && p != ".." && p.chars().all(|c| c.is_ascii_alphanumeric() || "-._".contains(c)))
}

#[derive(Default)]
pub struct FormProfile {
    name: String,
    bio: String,
    github: String,
    links: Vec<String>,
    loaded: bool,
}

enum Message {
    Session(Result<Session, String>),
    Login(Result<Session, String>),
    SessionRenewed(Session),
    Profile(Option<Profile>),
    Items(Result<ListOut, String>),
    Downloaded { item: Item, bytes: Result<Vec<u8>, String> },
    Uploaded(Result<String, String>),
    Reported(Result<(), String>),
    Claimed(Result<(), String>),
    /// Terms state of the account (None inside = the server could not tell).
    Terms(TermsInfo),
    TermsAccepted(Result<(), String>),
    AccountDeleted(Result<(), String>),
    State(String),
    /// A background task panicked (releases `busy`).
    TaskFailed,
    Image { path: String, bytes: Option<Vec<u8>> },
    ProfileView(Result<(ProfilePublic, Vec<Item>), String>),
    MyProfile(Option<ProfilePublic>),
    /// My hearts (content ids) and the creators I follow (account ids).
    Mine { hearts: Vec<String>, following: Vec<String> },
}

#[derive(Default, Clone)]
pub struct UploadForm {
    pub kind: usize,
    pub title: String,
    pub description: String,
    pub labels: String,
    pub is_18: bool,
    pub file: Option<PathBuf>,
    pub view: Option<PathBuf>,
    /// Index into `net::LICENSES`; None = the default of the kind.
    pub license: Option<usize>,
    pub source: String,
    /// "I own this or have the right to share it under this licence".
    pub rights: bool,
    /// Field the server refused ("license", "source", "rights"): shown highlighted.
    pub field_error: Option<&'static str>,
}

/// Licence, source link and rights declaration that travel with an upload (migration 0015). Default = nothing is sent.
#[derive(Clone, Debug, Default)]
pub struct UploadMeta {
    pub license: Option<String>,
    pub source_url: String,
    pub rights: bool,
}

/// What the server and the profile say about the terms and the own moderation state.
#[derive(Clone, Debug, Default)]
pub struct TermsInfo {
    pub server_current: Option<String>,
    pub own: Option<net::OwnState>,
    /// The terms version could not be read (not the same as a server without terms): uploads wait until it is.
    pub server_unavailable: bool,
}

impl TermsInfo {
    pub fn decision(&self) -> net::TermsDecision {
        if self.server_unavailable {
            return net::TermsDecision::Unknown;
        }
        match &self.own {
            None => net::TermsDecision::NotApplicable,
            Some(o) => net::terms_decision(self.server_current.as_deref(), o.terms_version.as_deref(), &net::embedded_terms_version()),
        }
    }
}

/// Why the terms screen is open.
#[derive(Clone, Copy, Debug, PartialEq)]
enum TermsMode {
    /// Creating the profile: accepting here ticks the box of the form.
    Create,
    /// The account must accept the current version.
    Accept,
    /// Only reading (guidelines button).
    Read,
}

#[derive(Default)]
struct ClaimForm {
    item: String,
    description: String,
    claimant: String,
    contact: String,
}

pub struct Community {
    tx: Sender<Message>,
    rx: Receiver<Message>,
    pub session: Option<Session>,
    pub profile: Option<Profile>,
    pub profile_loaded: bool,
    items: Vec<Item>,
    filter: Option<&'static str>,
    search: String,
    busy: u32,
    started: bool,
    pub state: String,
    new_name: String,
    accepted: bool,
    adult: bool,
    upload: UploadForm,
    delete_confirmed: bool,
    report: Option<String>,
    claim: Option<ClaimForm>,
    terms: Option<TermsInfo>,
    terms_open: Option<TermsMode>,
    terms_tab: usize,
    terms_check: bool,
    /// The "accept the new version" screen was already shown to this session (it comes back on a `terms_required`).
    terms_prompted: bool,
    /// The form of the last upload, restored when the server refuses a field or asks for the terms.
    upload_backup: Option<UploadForm>,
    /// A sign-in is waiting on the browser; the value is used to cancel it.
    login: Option<Arc<AtomicBool>>,
    /// Community images already loaded (None = loading or unavailable).
    textures: std::collections::HashMap<String, Option<egui::TextureHandle>>,
    /// Which images are downloading and which failed recently (a failure is retried later, never remembered forever).
    tex_gate: TexGate,
    /// Open creator profile and its content.
    viewing: Option<(ProfilePublic, Vec<Item>)>,
    my_profile: Option<ProfilePublic>,
    profile_form: FormProfile,
    my_hearts: std::collections::HashSet<String>,
    my_following: std::collections::HashSet<String>,
    only_following: bool,
    /// Preview open at full size (public path of the image).
    enlarged: Option<String>,
    profile_open: bool,
    /// Open the "Share" form the next time it is drawn (when arriving from the Themes window).
    open_upload: bool,
    /// Newly chosen photo and banner (shown immediately, before the server publishes them).
    previous: std::collections::HashMap<&'static str, egui::TextureHandle>,
    /// The list on screen comes from the local cache (age in seconds): the server could not be reached.
    offline_age: Option<u64>,
}

impl Community {
    /// Public name of the signed-in account, if any.
    pub fn profile_name(&self) -> Option<String> {
        self.session.as_ref()?;
        self.profile.as_ref().map(|p| p.public_name.clone())
    }
}

impl Default for Community {
    fn default() -> Self {
        Self::new()
    }
}

impl Community {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            tx,
            rx,
            session: None,
            profile: None,
            profile_loaded: false,
            items: Vec::new(),
            filter: None,
            search: String::new(),
            busy: 0,
            started: false,
            state: String::new(),
            new_name: String::new(),
            accepted: false,
            adult: false,
            upload: UploadForm::default(),
            delete_confirmed: false,
            report: None,
            claim: None,
            terms: None,
            terms_open: None,
            terms_tab: 0,
            terms_check: false,
            terms_prompted: false,
            upload_backup: None,
            login: None,
            textures: Default::default(),
            tex_gate: TexGate::default(),
            viewing: None,
            my_profile: None,
            profile_form: FormProfile::default(),
            my_hearts: Default::default(),
            my_following: Default::default(),
            only_following: false,
            enlarged: None,
            profile_open: false,
            open_upload: false,
            previous: Default::default(),
            offline_age: None,
        }
    }
}

// ───────────────────────────── utilities ─────────────────────────────

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// UTC date and time in ISO 8601 format (for `acepto_terminos_en`).
fn iso(t: u64) -> String {
    let (days, rest) = ((t / 86400) as i64, t % 86400);
    let z = days + 719468;
    let was = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + was * 400 + if m <= 2 { 1 } else { 0 };
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, rest / 3600, (rest % 3600) / 60, rest % 60)
}

fn random(n: usize) -> Vec<u8> {
    let mut b = vec![0u8; n];
    getrandom::fill(&mut b).expect("the system did not provide random numbers");
    b
}

fn b64url(b: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b)
}

fn uuid_v4() -> String {
    let mut b = random(16);
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{:02x}", x)).collect();
    format!("{}-{}-{}-{}-{}", &h[0..8], &h[8..12], &h[12..16], &h[16..20], &h[20..32])
}

fn encode(s: &str) -> String {
    s.bytes().map(|c| if c.is_ascii_alphanumeric() || b"-._~".contains(&c) { (c as char).to_string() } else { format!("%{:02X}", c) }).collect()
}

/// Safe file name for uploading (only letters, digits, dot, hyphen and underscore).
fn safe_name(n: &str) -> String {
    let s: String = n.chars().map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' }).collect();
    let s = s.trim_start_matches('.').to_string();
    if s.is_empty() { "archivo".into() } else { s.chars().take(60).collect() }
}

fn is_uuid(s: &str) -> bool {
    s.len() == 36 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

// ───────────────────────────── network ─────────────────────────────

/// Base address of the server (a local fake one in the tests).
fn base_url() -> String {
    #[cfg(test)]
    if let Some(b) = TEST_BASE.with(|b| b.borrow().clone()) {
        return b;
    }
    URL.to_string()
}

#[cfg(test)]
thread_local! {
    static TEST_BASE: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// One agent per request, with its own timeouts: connect, per-request limit (`global`) and a limit on every phase.
fn agent(cfg: &NetCfg, global: Duration) -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(cfg.connect))
        .timeout_global(Some(global))
        .timeout_recv_response(Some(global.min(Duration::from_secs(60))))
        .http_status_as_error(false)
        .user_agent("LienzoUI-Community/1.0")
        .build()
        .into()
}

struct Resp {
    state: u16,
    body: Vec<u8>,
    /// Seconds from the `Retry-After` header, when it came.
    retry_after: Option<u64>,
}

impl Resp {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
    fn ok(self) -> Result<Resp, String> {
        if (200..300).contains(&self.state) {
            Ok(self)
        } else {
            Err(error_message_with(self.state, &self.text(), self.retry_after))
        }
    }
    fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, String> {
        serde_json::from_slice(&self.body).map_err(|_| t("Unexpected response from the server."))
    }
}

/// Understandable error messages; the raw server body is never returned.
fn error_message(state: u16, body: &str) -> String {
    error_message_with(state, body, None)
}

fn error_message_with(state: u16, body: &str, retry_after: Option<u64>) -> String {
    let v: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    let msg = ["message", "msg", "error_description", "error"].iter().find_map(|k| v.get(k).and_then(|m| m.as_str())).unwrap_or("");
    // Machine codes of migration 0015 (CLIENT_CONTRACT.md): matched on `hint`, never on the HTTP status.
    if let Some(code) = net::error_code(body) {
        if let Some(m) = net::code_message(&code) {
            return m;
        }
        if net::is_detail_code(&code) {
            if let Some(d) = net::error_detail(body) {
                return val::terminal_safe(&d);
            }
        }
    }
    match state {
        401 => net::expired_message(),
        403 => t("You do not have permission to do this."),
        404 => t("Not found."),
        409 => t("It already exists or was already processed."),
        429 if msg.contains("Demasiadas subidas") || msg.contains("Daily limit") => val::terminal_safe(msg),
        429 => net::rate_limited_message(retry_after),
        _ if msg.contains("Demasiadas subidas") || msg.contains("Daily limit") => val::terminal_safe(msg),
        _ if msg.contains("duplicate key") => t("That name is already in use."),
        s if s >= 500 => net::unavailable_message(),
        s => tf("Error {}", &[&s.to_string()]),
    }
}

fn request(method: &str, path: &str, session: Option<&Session>, kind: Option<&str>, body: Option<&[u8]>, extra: &[(&str, &str)], limit: usize) -> Result<Resp, String> {
    try_request(&NetCfg::default(), &base_url(), method, path, session, kind, body, extra, limit).map_err(|f| f.msg)
}

/// A request with retries: only GET (idempotent) is retried, with backoff, on a transient failure (offline, timeout, cut
/// connection, 502/503/504) and on 429 when `Retry-After` is short. Writes (uploads, PATCH, POST, DELETE) are tried once.
#[allow(clippy::too_many_arguments)]
fn try_request(cfg: &NetCfg, base: &str, method: &str, path: &str, session: Option<&Session>, kind: Option<&str>, body: Option<&[u8]>, extra: &[(&str, &str)], limit: usize) -> Result<Resp, net::Failure> {
    let mut attempt = 0;
    loop {
        let r = send_once(cfg, base, method, path, session, kind, body, extra, limit);
        let outcome = match &r {
            Ok(resp) => Some(Outcome::Status(resp.state, resp.retry_after)),
            Err(f) if f.transient => Some(Outcome::Transport),
            Err(_) => None,
        };
        match outcome.filter(|_| method == "GET").and_then(|o| net::retry_delay(cfg, attempt, o)) {
            Some(wait) => {
                std::thread::sleep(wait);
                attempt += 1;
            }
            None => return r,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn send_once(cfg: &NetCfg, base: &str, method: &str, path: &str, session: Option<&Session>, kind: Option<&str>, body: Option<&[u8]>, extra: &[(&str, &str)], limit: usize) -> Result<Resp, net::Failure> {
    let url = format!("{}{}", base, path);
    let global = if method != "GET" {
        cfg.write
    } else if path.starts_with("/storage/") {
        cfg.download
    } else {
        cfg.read
    };
    let a = agent(cfg, global);
    macro_rules! headers {
        ($r:expr) => {{
            let mut r = $r.header("apikey", PUBLIC_KEY);
            if let Some(s) = session {
                r = r.header("Authorization", &format!("Bearer {}", s.access));
            }
            for (k, v) in extra {
                r = r.header(*k, *v);
            }
            r
        }};
    }
    let res = match (method, body) {
        ("GET", _) => headers!(a.get(&url)).call(),
        ("DELETE", _) => headers!(a.delete(&url)).call(),
        ("PATCH", Some(c)) => headers!(a.patch(&url)).content_type(kind.unwrap_or("application/json")).send(c),
        (_, Some(c)) => headers!(a.post(&url)).content_type(kind.unwrap_or("application/json")).send(c),
        (_, None) => headers!(a.post(&url)).send_empty(),
    };
    let mut res = res.map_err(|e| net::classify(&e))?;
    let state = res.status().as_u16();
    let retry_after = res.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(net::parse_retry_after);
    let body = res.body_mut().with_config().limit(limit as u64).read_to_vec().map_err(|e| match e {
        ureq::Error::BodyExceedsLimit(_) => net::Failure { msg: t("The server response is too large or was cut off."), transient: false },
        other => {
            let mut f = net::classify(&other);
            f.transient = true;
            f
        }
    })?;
    Ok(Resp { state, body, retry_after })
}

fn json_post(path: &str, session: Option<&Session>, v: &serde_json::Value, extra: &[(&str, &str)]) -> Result<Resp, String> {
    request("POST", path, session, None, Some(v.to_string().as_bytes()), extra, 1 << 20)
}

// ───────────────────────────── session ─────────────────────────────

fn session_from(v: &serde_json::Value) -> Result<Session, String> {
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).map(|x| x.to_string());
    let access = s("access_token").ok_or("The server did not return a session.")?;
    let renewal = s("refresh_token").ok_or("The server did not return a session.")?;
    let uid = v.get("user").and_then(|u| u.get("id")).and_then(|i| i.as_str()).unwrap_or("").to_string();
    if !is_uuid(&uid) {
        return Err("Invalid account identifier.".into());
    }
    let dur = v.get("expires_in").and_then(|e| e.as_u64()).unwrap_or(3600);
    Ok(Session { access, renewal, uid, expires: now() + dur })
}

fn save_credential(s: Option<&Session>) {
    #[cfg(windows)]
    {
        if let Ok(e) = keyring::Entry::new(CREDENTIALS_SERVICE, "sesion") {
            match s {
                Some(s) => {
                    let _ = e.set_password(&serde_json::json!({ "r": s.renewal, "u": s.uid }).to_string());
                }
                None => {
                    let _ = e.delete_credential();
                }
            }
        }
    }
    #[cfg(not(windows))]
    let _ = s;
}

fn read_credential() -> Option<(String, String)> {
    #[cfg(windows)]
    {
        let p = keyring::Entry::new(CREDENTIALS_SERVICE, "sesion").ok()?.get_password().ok()?;
        let v: serde_json::Value = serde_json::from_str(&p).ok()?;
        return Some((v.get("r")?.as_str()?.to_string(), v.get("u")?.as_str()?.to_string()));
    }
    #[cfg(not(windows))]
    None
}

/// GoTrue answers a refresh token it no longer accepts (revoked, already used, unknown) with a 400 whose body says
/// `refresh_token_not_found`, `refresh_token_already_used`, `invalid_grant` or "Invalid Refresh Token": the session is over, not a glitch.
fn refresh_token_rejected(state: u16, body: &str) -> bool {
    if !matches!(state, 400 | 401 | 403 | 422) {
        return false;
    }
    let v: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    let field = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_lowercase();
    let (code, error, text) = (field("error_code"), field("error"), format!("{} {}", field("msg"), format!("{} {}", field("message"), field("error_description"))));
    matches!(code.as_str(), "refresh_token_not_found" | "refresh_token_already_used" | "session_not_found" | "session_expired" | "invalid_grant")
        || error == "invalid_grant"
        || text.contains("invalid refresh token")
        || text.contains("refresh token not found")
        || text.contains("already used")
}

fn renew(s: &Session) -> Result<Session, String> {
    let r = json_post("/auth/v1/token?grant_type=refresh_token", None, &serde_json::json!({ "refresh_token": s.renewal }), &[])?;
    if refresh_token_rejected(r.state, &r.text()) {
        save_credential(None); // the stored token is dead: next start must not try it again
        return Err(net::expired_message());
    }
    let r = r.ok()?;
    let v: serde_json::Value = r.json()?;
    session_from(&v)
}

/// Returns a valid session; if the token was renewed it notifies the UI.
fn current(s: &Session, tx: &Sender<Message>) -> Result<Session, String> {
    if s.expires > now() + 60 {
        return Ok(s.clone());
    }
    let n = renew(s)?;
    save_credential(Some(&n));
    let _ = tx.send(Message::SessionRenewed(n.clone()));
    Ok(n)
}

fn return_page(ok: bool) -> String {
    let (title, text) = if ok { ("Done", "You can go back to Lienzo UI now. You can close this tab.") } else { ("Could not sign in", "Go back to Lienzo UI and try again.") };
    let body = format!("<!doctype html><meta charset=utf-8><title>Lienzo UI</title><body style=\"font-family:sans-serif;background:#14161f;color:#eee;text-align:center;padding-top:20vh\"><h1>{}</h1><p>{}</p>", title, text);
    format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n{}", body.len(), body)
}

/// What a request to the temporary local port means for the sign-in.
#[derive(Debug, PartialEq)]
enum Callback {
    /// Not the sign-in answer (favicon, scanners, a `state` that is not ours...): ignored, the login goes on.
    Ignore,
    /// The provider reported an error (the user canceled).
    Failed,
    /// An authorization code to exchange.
    Code(String),
}

/// Reads the first line of an HTTP request (`GET /callback?code=... HTTP/1.1`) and decides what it means.
/// `state` is the random value of this login: an answer that carries a `state` must carry ours, and an error answer is
/// only believed with our `state`. (Supabase does not echo a `state` on the success redirect today; PKCE then still binds
/// the code to this login because the exchange needs our verifier.)
fn parse_callback(request_line: &str, state: &str) -> Callback {
    let path = request_line.strip_prefix("GET ").and_then(|r| r.split(' ').next()).unwrap_or("");
    let Some(query) = path.strip_prefix("/callback?") else { return Callback::Ignore };
    let param = |k: &str| query.split('&').find_map(|p| p.strip_prefix(&format!("{}=", k)));
    if param("state").is_some_and(|s| s != state) {
        return Callback::Ignore;
    }
    // A cancel/error answer only counts when it carries OUR state: anything else (a page, a scanner, another site) could cancel the login.
    if param("error").is_some() || param("error_description").is_some() {
        return if param("state") == Some(state) { Callback::Failed } else { Callback::Ignore };
    }
    match param("code").filter(|c| !c.is_empty() && c.len() < 300 && c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')) {
        Some(code) => Callback::Code(code.to_string()),
        None => Callback::Ignore,
    }
}

/// Reads the request line of one connection (own thread: an idle or slow connection never blocks the accept loop).
fn read_request_line(c: &mut std::net::TcpStream) -> Option<String> {
    let _ = c.set_nonblocking(false);
    let _ = c.set_read_timeout(Some(Duration::from_millis(1500)));
    let end = std::time::Instant::now() + Duration::from_secs(4);
    let mut buf = Vec::with_capacity(512);
    let mut chunk = [0u8; 512];
    while buf.len() < 4096 && !buf.contains(&b'\n') && std::time::Instant::now() < end {
        match c.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => buf.extend_from_slice(&chunk[..n]),
            Err(_) => return None, // idle / timed out
        }
    }
    let text = String::from_utf8_lossy(&buf).to_string();
    text.lines().next().map(|l| l.trim_end().to_string())
}

/// Sign-in with GitHub or Google: OAuth with PKCE and a `state` check. The browser returns to a temporary local port
/// (127.0.0.1) and the app exchanges the code; no intermediary server sees the session. Requests that are not the
/// answer to THIS login are ignored; idle connections do not block anything.
fn oauth_flow(provider: &str, cancel: &AtomicBool) -> Result<Session, String> {
    if !["github", "google"].contains(&provider) {
        return Err("Provider not supported.".into());
    }
    let listen = std::net::TcpListener::bind(("127.0.0.1", LOCAL_PORT)).map_err(|_| format!("Local port {} is busy. Close another Lienzo UI session and try again.", LOCAL_PORT))?;
    listen.set_nonblocking(true).map_err(|e| e.to_string())?;
    let verifier = b64url(&random(48));
    let challenge = b64url(&Sha256::digest(verifier.as_bytes()));
    let state = b64url(&random(24));
    let return_url = format!("http://127.0.0.1:{}/callback", LOCAL_PORT);
    let url = format!("{}/auth/v1/authorize?provider={}&redirect_to={}&code_challenge={}&code_challenge_method=s256&state={}", URL, provider, encode(&return_url), challenge, state);
    webbrowser::open(&url).map_err(|_| "Could not open the browser.".to_string())?;

    let limit = std::time::Instant::now() + Duration::from_secs(180);
    let (tx, rx) = channel::<(Callback, std::net::TcpStream)>();
    let in_flight = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut last_error: Option<String> = None;
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("Sign-in canceled.".into());
        }
        if std::time::Instant::now() > limit {
            return Err(last_error.unwrap_or_else(|| "Timed out waiting for sign-in.".into()));
        }
        match listen.accept() {
            Ok((mut c, _)) => {
                if in_flight.load(Ordering::Relaxed) >= 16 {
                    continue; // too many open connections: this one is dropped
                }
                in_flight.fetch_add(1, Ordering::Relaxed);
                let (tx, counter, st) = (tx.clone(), in_flight.clone(), state.clone());
                std::thread::spawn(move || {
                    if let Some(line) = read_request_line(&mut c) {
                        let what = parse_callback(&line, &st);
                        let _ = tx.send((what, c));
                    }
                    counter.fetch_sub(1, Ordering::Relaxed);
                });
                continue;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => return Err(e.to_string()),
        }
        match rx.recv_timeout(Duration::from_millis(150)) {
            Ok((Callback::Ignore, mut c)) => {
                let _ = c.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            }
            Ok((Callback::Failed, mut c)) => {
                let _ = c.write_all(return_page(false).as_bytes());
                return Err("Sign-in was canceled.".into());
            }
            Ok((Callback::Code(code), mut c)) => {
                // A wrong code (not made for this login) fails the exchange: the login keeps waiting for the real one.
                let attempt = json_post("/auth/v1/token?grant_type=pkce", None, &serde_json::json!({ "auth_code": code, "code_verifier": verifier }), &[]).and_then(|r| r.ok()).and_then(|r| session_from(&r.json::<serde_json::Value>()?));
                match attempt {
                    Ok(session) => {
                        let _ = c.write_all(return_page(true).as_bytes());
                        return Ok(session);
                    }
                    Err(e) => {
                        let _ = c.write_all(return_page(false).as_bytes());
                        last_error = Some(e);
                    }
                }
            }
            Err(_) => {}
        }
    }
}

#[cfg(test)]
mod oauth_tests {
    use super::*;

    #[test]
    fn callback_requires_our_state_when_present_and_ignores_noise() {
        let s = "abc_DEF-123";
        assert_eq!(parse_callback("GET /callback?code=xyz123 HTTP/1.1", s), Callback::Code("xyz123".into()));
        assert_eq!(parse_callback("GET /callback?code=xyz123&state=abc_DEF-123 HTTP/1.1", s), Callback::Code("xyz123".into()));
        assert_eq!(parse_callback("GET /callback?code=xyz123&state=other HTTP/1.1", s), Callback::Ignore);
        assert_eq!(parse_callback("GET /callback?error=access_denied&state=other HTTP/1.1", s), Callback::Ignore);
        assert_eq!(parse_callback("GET /callback?error=access_denied&state=abc_DEF-123 HTTP/1.1", s), Callback::Failed);
        assert_eq!(parse_callback("GET /callback?error=access_denied HTTP/1.1", s), Callback::Ignore, "an error without our state cannot cancel the login");
        assert_eq!(parse_callback("GET /callback?error_description=x&state= HTTP/1.1", s), Callback::Ignore);
        for noise in ["GET /favicon.ico HTTP/1.1", "POST /callback?code=x HTTP/1.1", "", "garbage", "GET /callback?code= HTTP/1.1", "GET /callback?code=a%20b HTTP/1.1", "GET /callback HTTP/1.1"] {
            assert_eq!(parse_callback(noise, s), Callback::Ignore, "{}", noise);
        }
    }

    #[test]
    fn idle_connection_is_dropped_by_the_reader() {
        let l = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let addr = l.local_addr().unwrap();
        let idle = std::net::TcpStream::connect(addr).unwrap(); // never writes
        let (mut srv, _) = l.accept().unwrap();
        let t = std::time::Instant::now();
        assert!(read_request_line(&mut srv).is_none());
        assert!(t.elapsed() < Duration::from_secs(5));
        drop(idle);
        let mut ok = std::net::TcpStream::connect(addr).unwrap();
        ok.write_all(b"GET /callback?code=abc HTTP/1.1\r\nHost: x\r\n\r\n").unwrap();
        let (mut srv, _) = l.accept().unwrap();
        assert_eq!(read_request_line(&mut srv).as_deref(), Some("GET /callback?code=abc HTTP/1.1"));
    }
}

// ───────────────────────────── queries ─────────────────────────────

fn clear_search(q: &str) -> String {
    q.chars().filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_').take(40).collect::<String>().trim().to_string()
}

/// The catalog listing, and whether it came from the local cache because the server could not be reached.
struct ListOut {
    items: Vec<Item>,
    /// Some(age in seconds) when the list is the cached one.
    stale: Option<u64>,
}

fn list(session: Option<&Session>, filter: Option<&str>, search: &str, with_18: bool, only_from: Option<&[String]>) -> Result<ListOut, String> {
    list_with(&NetCfg::default(), &base_url(), &net::cache_dir(), session, filter, search, with_18, only_from)
}

#[allow(clippy::too_many_arguments)]
fn list_with(cfg: &NetCfg, base: &str, cache: &std::path::Path, session: Option<&Session>, filter: Option<&str>, search: &str, with_18: bool, only_from: Option<&[String]>) -> Result<ListOut, String> {
    let extended = !NO_LICENSE_COLUMNS.load(Ordering::Relaxed);
    match list_cols(cfg, base, cache, session, filter, search, with_18, only_from, extended) {
        Err(e) if e == LEGACY_COLUMNS => {
            NO_LICENSE_COLUMNS.store(true, Ordering::Relaxed);
            list_cols(cfg, base, cache, session, filter, search, with_18, only_from, false)
        }
        other => other,
    }
}

#[allow(clippy::too_many_arguments)]
fn list_cols(cfg: &NetCfg, base: &str, cache: &std::path::Path, session: Option<&Session>, filter: Option<&str>, search: &str, with_18: bool, only_from: Option<&[String]>, extended: bool) -> Result<ListOut, String> {
    if let Some(ids) = only_from {
        if ids.is_empty() {
            return Ok(ListOut { items: Vec::new(), stale: None }); // you follow no one: nothing to request
        }
    }
    let mut q = format!("/rest/v1/community_items?select={}&status=eq.approved&order=created_at.desc&limit=60", if extended { ITEM_COLUMNS_V15 } else { ITEM_COLUMNS });
    if let Some(f) = filter.filter(|f| val::KINDS.contains(f)) {
        q.push_str(&format!("&kind=eq.{}", f));
    }
    if !with_18 {
        q.push_str("&is_18=eq.false");
    }
    if let Some(ids) = only_from {
        let valid: Vec<&str> = ids.iter().map(|i| i.as_str()).filter(|i| is_uuid(i)).collect();
        q.push_str(&format!("&author=in.({})", valid.join(",")));
    }
    let b = clear_search(search);
    if !b.is_empty() {
        q.push_str(&format!("&or=({})", encode(&format!("title.ilike.*{0}*,description.ilike.*{0}*", b))));
    }
    // only the plain browsing lists are cached (not searches or the "following" list)
    // (the key includes the base URL: a list cached from one server is never shown for another)
    let key = (b.is_empty() && only_from.is_none()).then(|| net::catalog_key(&format!("{}|{}", base, q)));
    let from_cache = |msg: String| -> Result<ListOut, String> {
        match key.as_ref().and_then(|k| net::catalog_load::<Item>(cache, k, now())) {
            // the cache file is untrusted: same checks as a live answer, and an implausible size is dropped
            Some((items, age)) if items.len() <= CATALOG_MAX_ITEMS => Ok(ListOut { items: sanitize_items(items, with_18), stale: Some(age) }),
            _ => Err(msg),
        }
    };
    match try_request(cfg, base, "GET", &q, session, None, None, &[], 2 << 20) {
        Ok(r) if (200..300).contains(&r.state) => {
            let mut items: Vec<Item> = r.json()?;
            let items = sanitize_items(items, with_18);
            if let Some(k) = &key {
                net::catalog_store(cache, k, &items, now());
            }
            Ok(ListOut { items, stale: None })
        }
        Ok(r) if r.state == 400 && extended => Err(LEGACY_COLUMNS.to_string()),
        Ok(r) => {
            let transient = r.state >= 500 || r.state == 429;
            let msg = r.ok().err().unwrap_or_default();
            if transient { from_cache(msg) } else { Err(msg) }
        }
        Err(f) if f.transient => from_cache(f.msg),
        Err(f) => Err(f.msg),
    }
}

/// Tracks community image downloads: in flight, and failed (retried after [`TEXTURE_RETRY`]).
#[derive(Default)]
struct TexGate {
    loading: std::collections::HashSet<String>,
    failed: std::collections::HashMap<String, std::time::Instant>,
}

const TEXTURE_RETRY: Duration = Duration::from_secs(30);

impl TexGate {
    /// True when a download of `key` should start now (and marks it as in flight).
    fn may_start(&mut self, key: &str, now: std::time::Instant) -> bool {
        if self.loading.contains(key) || self.failed.get(key).is_some_and(|t| now.saturating_duration_since(*t) < TEXTURE_RETRY) {
            return false;
        }
        self.loading.insert(key.to_string());
        true
    }

    fn finish(&mut self, key: &str, ok: bool, now: std::time::Instant) {
        self.loading.remove(key);
        if ok {
            self.failed.remove(key);
        } else {
            if self.failed.len() > 2000 {
                self.failed.clear();
            }
            self.failed.insert(key.to_string(), now);
        }
    }

    fn forget_prefix(&mut self, prefix: &str) {
        self.loading.retain(|k| !k.starts_with(prefix));
        self.failed.retain(|k, _| !k.starts_with(prefix));
    }
}

/// Text that came from the server, made safe to paint: no control characters (except line breaks when `multiline`),
/// no bidi / zero-width spoofing characters, no line / paragraph separators.
fn display_clean(s: &str, multiline: bool) -> String {
    s.chars().filter(|&c| (!c.is_control() || (multiline && c == '\n')) && !val::spoofing_char(c) && c != '\u{2028}' && c != '\u{2029}').collect()
}

impl Item {
    fn clean_texts(&mut self) {
        self.title = display_clean(&self.title, false);
        self.description = display_clean(&self.description, true);
        self.author_name = display_clean(&self.author_name, false);
        self.version = display_clean(&self.version, false);
        for t in self.tags.iter_mut() {
            *t = display_clean(t, false);
        }
    }
}

impl ProfilePublic {
    fn clean_texts(&mut self) {
        self.public_name = display_clean(&self.public_name, false);
        self.bio = display_clean(&self.bio, true);
        self.github = self.github.take().map(|g| display_clean(&g, false));
        self.links = std::mem::take(&mut self.links).into_iter().map(|(k, v)| (display_clean(&k, false), display_clean(&v, false))).collect();
    }
}

/// A listing never has more rows than the request asks for (`limit=60`); a bigger cache file is not ours.
const CATALOG_MAX_ITEMS: usize = 200;

/// Keeps only well-formed rows: UUID id and author, a known kind and, when 18+ content is off, nothing marked 18+.
/// Applied to live answers and to the offline cache alike.
fn sanitize_items(mut items: Vec<Item>, with_18: bool) -> Vec<Item> {
    items.retain(|i| is_uuid(&i.id) && is_uuid(&i.author) && val::KINDS.contains(&i.kind.as_str()) && (with_18 || !i.is_18));
    items.iter_mut().for_each(Item::clean_texts);
    items
}

/// Columns of `public_profiles` that anybody may read (never `select=*`: migration 0014 revokes the pending flags and `updated_at` from everyone).
const PROFILE_PUBLIC_COLUMNS: &str = "id,public_name,bio,github,links,public_avatar,public_banner,followers,hearts_received";
/// The same plus the caller's own pending flags: only valid on `my_public_profile` or, before migration 0014, on `public_profiles`.
const PROFILE_OWN_COLUMNS: &str = "id,public_name,bio,github,links,public_avatar,public_banner,avatar_pending,banner_pending,followers,hearts_received";

const ITEM_COLUMNS: &str = "id,kind,title,description,tags,is_18,public_path,sha256,bytes,downloads,version,author,author_name,public_icon,public_preview,hearts";
/// The same plus the licence columns of migration 0015.
const ITEM_COLUMNS_V15: &str = "id,kind,title,description,tags,is_18,public_path,sha256,bytes,downloads,version,author,author_name,public_icon,public_preview,hearts,license,source_url";
/// Set once a read with the licence columns was refused (400) and the plain one worked: the server has no migration 0015 yet.
static NO_LICENSE_COLUMNS: AtomicBool = AtomicBool::new(false);
/// Marker returned by a listing that the server refused only because of the new columns.
const LEGACY_COLUMNS: &str = "legacy-columns-marker";

fn item_columns() -> &'static str {
    if NO_LICENSE_COLUMNS.load(Ordering::Relaxed) {
        ITEM_COLUMNS
    } else {
        ITEM_COLUMNS_V15
    }
}

fn load_public_profile(session: Option<&Session>, uid: &str) -> Result<(ProfilePublic, Vec<Item>), String> {
    if !is_uuid(uid) {
        return Err("Invalid creator.".into());
    }
    let p: Vec<ProfilePublic> = request("GET", &format!("/rest/v1/public_profiles?select={}&id=eq.{}", PROFILE_PUBLIC_COLUMNS, uid), session, None, None, &[], 1 << 18)?.ok()?.json()?;
    let p = p.into_iter().next().ok_or("This creator does not have a public profile yet.")?;
    let fetch = |cols: &str| -> Result<Vec<Item>, String> { request("GET", &format!("/rest/v1/community_items?select={}&status=eq.approved&author=eq.{}&order=created_at.desc&limit=60", cols, uid), session, None, None, &[], 2 << 20)?.ok()?.json() };
    let mut items = fetch(item_columns()).or_else(|e| {
        if item_columns() == ITEM_COLUMNS {
            return Err(e);
        }
        let r = fetch(ITEM_COLUMNS); // a server without migration 0015 has no licence columns
        if r.is_ok() {
            NO_LICENSE_COLUMNS.store(true, Ordering::Relaxed);
        }
        r
    })?;
    items.retain(|i| is_uuid(&i.id) && val::KINDS.contains(&i.kind.as_str()));
    items.iter_mut().for_each(Item::clean_texts);
    let mut p = p;
    p.clean_texts();
    Ok((p, items))
}

/// Your own profile, with the pending-review flags. Read from the view `my_public_profile` (migration 0014: only the caller's row);
/// while that migration is not applied yet the view does not exist and the same explicit columns are read from the table (never `select=*`).
fn load_my_profile(s: &Session) -> Option<ProfilePublic> {
    let from_view = request("GET", &format!("/rest/v1/my_public_profile?select={}&id=eq.{}", PROFILE_OWN_COLUMNS, s.uid), Some(s), None, None, &[], 1 << 18).and_then(|r| r.ok());
    let r = match from_view {
        Ok(r) => r,
        Err(_) => request("GET", &format!("/rest/v1/public_profiles?select={}&id=eq.{}", PROFILE_OWN_COLUMNS, s.uid), Some(s), None, None, &[], 1 << 18).ok()?.ok().ok()?,
    };
    let mut p = r.json::<Vec<ProfilePublic>>().ok()?.into_iter().next().filter(|p| p.id == s.uid)?;
    p.clean_texts();
    Some(p)
}

/// Crops a photo to the profile format (avatar 1:1, banner 3:1), shrinks it and converts it to PNG within the limits.
fn prepare_profile_image(class: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.len() > 25 * 1024 * 1024 {
        return Err("The image is too heavy (maximum 25 MB).".into());
    }
    if let Some(r) = val::detect(bytes) {
        if r.starts_with("executable") || r == "zip" || r.starts_with("script") {
            return Err(format!("Dangerous content detected ({}).", r));
        }
    }
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().map_err(|_| "Could not read the image.".to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(12_000);
    limits.max_image_height = Some(12_000);
    limits.max_alloc = Some(512 * 1024 * 1024);
    reader.limits(limits);
    let img = reader.decode().map_err(|_| "Could not read the image (use PNG, JPEG, WebP, GIF or BMP).".to_string())?;
    let (w, h) = (img.width().max(1), img.height().max(1));
    let ratio = if class == "avatar" { 1.0 } else { 3.0 };
    let (cw, ch) = if (w as f32 / h as f32) > ratio { (((h as f32) * ratio) as u32, h) } else { (w, ((w as f32) / ratio) as u32) };
    let crop = img.crop_imm((w - cw.min(w)) / 2, (h - ch.min(h)) / 2, cw.min(w).max(1), ch.min(h).max(1));
    let sizes: &[(u32, u32)] = if class == "avatar" { &[(400, 400), (320, 320), (256, 256)] } else { &[(1500, 500), (1200, 400), (900, 300)] };
    for &(tw, th) in sizes {
        let r = crop.resize_exact(tw, th, image::imageops::FilterType::Lanczos3);
        let mut out = std::io::Cursor::new(Vec::new());
        r.write_to(&mut out, image::ImageFormat::Png).map_err(|e| e.to_string())?;
        let b = out.into_inner();
        if val::validate_png(class, &b).is_ok() {
            return Ok(b);
        }
    }
    Err("Could not reduce the image to the allowed size.".into())
}

/// Uploads a profile PNG image (avatar or banner) to quarantine and asks the server to moderate it.
fn upload_profile_image(s: &Session, class: &str, bytes: &[u8]) -> Result<String, String> {
    val::validate_png(class, bytes)?;
    let path = format!("{}/perfil/{}.png", s.uid, class);
    let _ = request("DELETE", &format!("/storage/v1/object/cuarentena/{}", path), Some(s), None, None, &[], 1 << 16); // replaces the previous one
    request("POST", &format!("/storage/v1/object/cuarentena/{}", path), Some(s), Some("image/png"), Some(bytes), &[("x-upsert", "false")], 1 << 16)?.ok()?;
    let r = json_post("/functions/v1/profile", Some(s), &serde_json::json!({ "v": 2, "action": "image", "class": class }), &[])?;
    let v: serde_json::Value = r.json().unwrap_or_default();
    if !(200..300).contains(&r.state) {
        return Err(v["error"].as_str().unwrap_or("Could not upload the image.").to_string());
    }
    Ok(match v["status"].as_str().unwrap_or("") {
        "published" => "Image published on your profile.".to_string(),
        _ => v["reason"].as_str().unwrap_or("Image sent for review.").to_string(),
    })
}

/// My hearts and my follows (only what is mine: under RLS nobody else sees it).
fn load_mine(s: &Session) -> Option<(Vec<String>, Vec<String>)> {
    let col = |path: &str, field: &str| -> Option<Vec<String>> {
        let v: Vec<serde_json::Value> = request("GET", path, Some(s), None, None, &[], 1 << 20).ok()?.ok().ok()?.json().ok()?;
        Some(v.iter().filter_map(|f| f[field].as_str().filter(|i| is_uuid(i)).map(|i| i.to_string())).collect())
    };
    Some((col(&format!("/rest/v1/hearts?select=item&person=eq.{}&limit=1000", s.uid), "item")?, col(&format!("/rest/v1/follows?select=followed&follower=eq.{}&limit=1000", s.uid), "followed")?))
}

fn load_profile(s: &Session) -> Option<Profile> {
    let r = request("GET", &format!("/rest/v1/profiles?select=public_name,show_18&id=eq.{}", s.uid), Some(s), None, None, &[], 1 << 16).ok()?.ok().ok()?;
    r.json::<Vec<Profile>>().ok()?.into_iter().next()
}

/// What `current_terms()` answered.
#[derive(Debug, PartialEq)]
enum TermsFetch {
    /// The published version.
    Version(String),
    /// The server really has no terms: the function does not exist (404 / PGRST202 / 42883) or it published nothing (migration 0015 not applied).
    Missing,
    /// The answer could not be obtained (network, 5xx, 429, a refused session, garbage): NOT the same as "no terms".
    Failed,
}

/// Asks for the current terms version, retrying (up to `attempts` times) on a network failure, 5xx or 429.
fn fetch_current_terms_with(cfg: &NetCfg, base: &str, s: &Session, attempts: usize) -> TermsFetch {
    for i in 0..attempts.max(1) {
        let last = i + 1 >= attempts.max(1);
        match try_request(cfg, base, "POST", "/rest/v1/rpc/current_terms", Some(s), None, Some(b"{}"), &[], 1 << 20) {
            Ok(r) if (200..300).contains(&r.state) => {
                return match r.json::<serde_json::Value>() {
                    Ok(v) => match v["version"].as_str().map(|x| x.chars().take(40).collect::<String>()).filter(|x| !x.trim().is_empty()) {
                        Some(x) => TermsFetch::Version(x),
                        None => TermsFetch::Missing,
                    },
                    Err(_) => TermsFetch::Failed,
                };
            }
            Ok(r) if r.state == 404 => return TermsFetch::Missing,
            Ok(r) if r.state == 400 && { let b = r.text(); b.contains("PGRST202") || b.contains("42883") || b.contains("Could not find the function") } => return TermsFetch::Missing,
            Ok(r) if r.state >= 500 || r.state == 429 => {}
            Ok(_) => return TermsFetch::Failed,
            Err(f) if f.transient => {}
            Err(_) => return TermsFetch::Failed,
        }
        if !last {
            std::thread::sleep(cfg.backoff * (i as u32 + 1));
        }
    }
    TermsFetch::Failed
}

/// Version published by `current_terms()`; None when the server does not have migration 0015 (or could not be reached).
fn fetch_current_terms(s: &Session) -> Option<String> {
    match fetch_current_terms_with(&NetCfg::default(), &base_url(), s, 3) {
        TermsFetch::Version(v) => Some(v),
        _ => None,
    }
}

/// Terms version of the server and own terms/moderation state (contract sections 1 and 7). Anything that fails reads as "unknown":
/// a server without migration 0015 has neither the function nor the columns, and then nothing is asked of the user.
fn load_terms(s: &Session) -> TermsInfo {
    let own = request("GET", &format!("/rest/v1/profiles?select=terms_version,terms_accepted_at,strikes,upload_banned_until,suspended&id=eq.{}", s.uid), Some(s), None, None, &[], 1 << 16)
        .ok()
        .and_then(|r| r.ok().ok())
        .and_then(|r| r.json::<Vec<net::OwnState>>().ok())
        .and_then(|v| v.into_iter().next());
    let (server_current, server_unavailable) = match fetch_current_terms_with(&NetCfg::default(), &base_url(), s, 3) {
        TermsFetch::Version(v) => (Some(v), false),
        TermsFetch::Missing => (None, false),
        TermsFetch::Failed => (None, true),
    };
    TermsInfo { server_current, own, server_unavailable }
}

/// `export_my_data()` saved as pretty JSON at `path` (own rows only; items are capped at 2000 by the server).
fn export_my_data(s: &Session, path: &std::path::Path) -> Result<(), String> {
    let r = request("POST", "/rest/v1/rpc/export_my_data", Some(s), None, Some(b"{}"), &[], 32 << 20)?.ok()?;
    let v: serde_json::Value = r.json()?;
    let pretty = serde_json::to_vec_pretty(&v).map_err(|e| e.to_string())?;
    std::fs::write(path, pretty).map_err(|e| e.to_string())
}

fn download(item: &Item, session: Option<&Session>) -> Result<Vec<u8>, String> {
    let path = item.public_path.as_deref().ok_or("This content has no file.")?;
    if !valid_public_path(path) {
        return Err("Invalid file path.".into());
    }
    let limit = val::byte_limit(&item.kind);
    let r = request("GET", &format!("/storage/v1/object/public/publico/{}", path), None, None, None, &[], limit + 1)?.ok()?;
    let bytes = r.body;
    if bytes.len() > limit {
        return Err("The file exceeds the allowed size.".into());
    }
    let expected = item.sha256.as_deref().ok_or("The content has no verification fingerprint.")?;
    if val::sha256_hex(&bytes) != expected {
        return Err("The SHA-256 fingerprint does not match: the file is not the one that was approved. It was not installed.".into());
    }
    let name = path.rsplit('/').next().unwrap_or("");
    val::validate(&item.kind, name, &bytes).map_err(|e| format!("Did not pass local validation: {}", e))?;
    if let Some(s) = session {
        let _ = json_post("/rest/v1/rpc/record_download", Some(s), &serde_json::json!({ "p_item": item.id }), &[]); // only counts when signed in
    }
    Ok(bytes)
}

#[allow(clippy::too_many_arguments)]
fn upload_content(s: &Session, kind: &str, title: &str, description: &str, labels: Vec<String>, is_18: bool, meta: &UploadMeta, name: &str, bytes: &[u8], extras: &[(&str, Vec<u8>)], tx: &Sender<Message>) -> Result<String, String> {
    if !val::valid_display_text(title) {
        return Err("The title has control or invisible characters (for example direction marks or zero-width spaces).".into());
    }
    val::validate(kind, name, bytes)?;
    for (class, b) in extras {
        val::validate_png(class, b).map_err(|e| format!("{} ({})", e, class))?;
    }
    let s = current(s, tx)?;
    let id = uuid_v4();
    let file = safe_name(name);
    let path = format!("{}/{}/{}", s.uid, id, file);
    let mut row = serde_json::json!({
        "id": id, "author": s.uid, "kind": kind, "title": title, "description": description,
        "tags": labels, "is_18": is_18, "version": "1.0.0", "quarantine_path": path,
        "quarantine_icon": extras.iter().any(|(c, _)| *c == "icon").then(|| format!("{}/{}/icon.png", s.uid, id)),
        "quarantine_preview": extras.iter().any(|(c, _)| *c == "preview").then(|| format!("{}/{}/preview.png", s.uid, id)),
    });
    // Licence, source and rights declaration (migration 0015). A server that does not have the columns yet refuses them (400): the row is sent again without.
    let fields = net::upload_extras(meta.license.as_deref(), &meta.source_url, meta.rights, &iso(now()));
    let plain = row.clone();
    if let Some(o) = row.as_object_mut() {
        o.extend(fields.clone());
    }
    let mut resp = json_post("/rest/v1/community_items", Some(&s), &row, &[("Prefer", "return=minimal")])?;
    if !fields.is_empty() && net::is_missing_column(resp.state, &resp.text()) {
        resp = json_post("/rest/v1/community_items", Some(&s), &plain, &[("Prefer", "return=minimal")])?;
    }
    resp.ok()?;
    let mime = match file.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        _ => "application/json",
    };
    let mut upload = request("POST", &format!("/storage/v1/object/cuarentena/{}", path), Some(&s), Some(mime), Some(bytes), &[("x-upsert", "false")], 1 << 20).and_then(|r| r.ok());
    for (class, b) in extras {
        if upload.is_ok() {
            upload = request("POST", &format!("/storage/v1/object/cuarentena/{}/{}/{}.png", s.uid, id, class), Some(&s), Some("image/png"), Some(b), &[("x-upsert", "false")], 1 << 20).and_then(|r| r.ok());
        }
    }
    if let Err(e) = upload {
        let _ = request("DELETE", &format!("/rest/v1/community_items?id=eq.{}", id), Some(&s), None, None, &[], 1 << 16); // don't leave a listing without a file
        return Err(e);
    }
    let r = json_post("/functions/v1/moderate", Some(&s), &serde_json::json!({ "v": 2, "item": id }), &[])?.ok()?;
    let v: serde_json::Value = r.json()?;
    let reason = v["reason"].as_str().unwrap_or("");
    Ok(match v["status"].as_str().unwrap_or("") {
        "approved" => "Published in the community.".to_string(),
        "rejected" => format!("Rejected: {}", reason),
        _ => format!("Sent for review: {}", reason),
    })
}

// ───────────────────────────── installation (UI thread) ─────────────────────────────

fn write_temp(name: &str, bytes: &[u8]) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("lienzo-ui-community");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let f = dir.join(safe_name(name));
    std::fs::write(&f, bytes).map_err(|e| e.to_string())?;
    Ok(f)
}

/// Folder name of an installed community plugin. Injective on the exact name: the readable part only keeps
/// `[a-z0-9]` (so "A b", "A_b" and "abc." never differ only by characters Windows folds or strips) and a hash
/// of the exact name makes every distinct name a distinct folder.
fn plugin_folder_name(name: &str) -> String {
    let slug: String = name.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' }).collect::<String>().split('-').filter(|p| !p.is_empty()).collect::<Vec<_>>().join("-");
    let slug: String = slug.chars().take(30).collect();
    format!("community-{}-{}", if slug.is_empty() { "plugin".to_string() } else { slug }, &val::sha256_hex(name.as_bytes())[..8])
}

/// Installs a downloaded plugin. All files are written to a temporary folder next to the final one and swapped in
/// with a rename, so a failure half way never leaves a half-written plugin (or deletes the previous version).
fn install_plugin(bytes: &[u8]) -> Result<String, String> {
    let p: Package = val::read_package(bytes)?;
    let name = crate::plugins::manifest_key(&p.manifest, "name", "nombre").and_then(|n| n.as_str()).unwrap_or("plugin").to_string();
    let base = crate::plugins::user_folder();
    let folder = base.join(plugin_folder_name(&name));
    install_plugin_into(&p, &base, &folder)?;
    // Older versions installed to "comunidad-<name>": the same plugin is not shown twice.
    let legacy = base.join(format!("comunidad-{}", safe_name(&name).to_lowercase()));
    if legacy != folder && legacy.join("plugin.json").exists() {
        let same = std::fs::read_to_string(legacy.join("plugin.json")).ok().and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()).is_some_and(|m| crate::plugins::manifest_key(&m, "name", "nombre").and_then(|n| n.as_str()) == Some(name.as_str()));
        if same {
            let _ = std::fs::remove_dir_all(&legacy);
        }
    }
    Ok(name)
}

fn install_plugin_into(p: &Package, base: &std::path::Path, folder: &std::path::Path) -> Result<(), String> {
    std::fs::create_dir_all(base).map_err(|e| e.to_string())?;
    let token = b64url(&random(6));
    let temp = base.join(format!(".installing-{}", token));
    let result = (|| -> Result<(), String> {
        std::fs::create_dir_all(&temp).map_err(|e| e.to_string())?;
        for (path, content) in &p.files {
            if path.eq_ignore_ascii_case("plugin.json") {
                continue; // never taken from the package: the validated manifest is serialized below
            }
            let destination = temp.join(path); // validated: no "..", no absolute paths, only .rhai/.json
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            std::fs::write(&destination, content).map_err(|e| e.to_string())?;
        }
        std::fs::write(temp.join("plugin.json"), serde_json::to_string_pretty(&p.manifest).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        // swap: the old version (if any) is moved aside and only deleted once the new one is in place
        let old = base.join(format!(".replaced-{}", token));
        let had_old = folder.exists();
        if had_old {
            std::fs::rename(folder, &old).map_err(|e| e.to_string())?;
        }
        if let Err(e) = std::fs::rename(&temp, folder) {
            if had_old {
                let _ = std::fs::rename(&old, folder);
            }
            return Err(e.to_string());
        }
        if had_old {
            let _ = std::fs::remove_dir_all(&old);
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&temp);
    }
    result
}

/// Turns a plugin folder into the package that gets uploaded.
fn pack_plugin(folder: &std::path::Path) -> Result<Vec<u8>, String> {
    let manifest: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(folder.join("plugin.json")).map_err(|_| "The folder has no plugin.json.")?).map_err(|_| "plugin.json is not valid.")?;
    let mut files = std::collections::BTreeMap::new();
    for (new, old) in [("commands", "comandos"), ("exporters", "exportadores")] {
        for x in crate::plugins::manifest_key(&manifest, new, old).and_then(|v| v.as_array()).into_iter().flatten() {
            if let Some(s) = x.get("script").and_then(|s| s.as_str()) {
                let f = folder.join(s);
                if !f.starts_with(folder) || s.contains("..") {
                    return Err(format!("Path not allowed: {}", s));
                }
                files.insert(s.to_string(), std::fs::read_to_string(&f).map_err(|_| format!("Could not read {}.", s))?);
            }
        }
    }
    for pl in crate::plugins::manifest_key(&manifest, "templates", "plantillas").and_then(|v| v.as_array()).into_iter().flatten().filter_map(|p| p.as_str()) {
        if pl.contains("..") {
            return Err(format!("Path not allowed: {}", pl));
        }
        files.insert(pl.to_string(), std::fs::read_to_string(folder.join(pl)).map_err(|_| format!("Could not read {}.", pl))?);
    }
    serde_json::to_vec(&Package { manifest, files }).map_err(|e| e.to_string())
}

// ───────────────────────────── UI ─────────────────────────────

const KIND_NAMES: &[(&str, &str)] = &[("plugin", "Plugins"), ("image", "Images"), ("font", "Fonts"), ("template", "Templates"), ("element", "Elements"), ("theme", "Themes")];

fn kind_name(t: &str) -> &'static str {
    KIND_NAMES.iter().find(|(k, _)| *k == t).map(|(_, n)| *n).unwrap_or("")
}

impl EditorApp {
    fn community_launch(&mut self, ctx: &egui::Context, f: impl FnOnce(&Sender<Message>) + Send + 'static) {
        self.community.busy += 1;
        let tx = self.community.tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            // A panic in the task must not leave the spinner on forever: the UI is told and releases `busy`.
            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&tx))).is_err() {
                let _ = tx.send(Message::TaskFailed);
            }
            ctx.request_repaint();
        });
    }

    fn community_load_list(&mut self, ctx: &egui::Context) {
        let (s, f, b) = (self.community.session.clone(), self.community.filter, self.community.search.clone());
        let with_18 = self.community.profile.as_ref().map(|p| p.show_18).unwrap_or(false) && s.is_some();
        let only_from: Option<Vec<String>> = (self.community.only_following && s.is_some()).then(|| self.community.my_following.iter().cloned().collect());
        self.community_launch(ctx, move |tx| {
            // A failure to get a valid session (expired refresh token, offline...) is reported, not swallowed into an anonymous list.
            let s = match s.map(|s| current(&s, tx)).transpose() {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx.send(Message::Items(Err(e)));
                    return;
                }
            };
            let _ = tx.send(Message::Items(list(s.as_ref(), f, &b, with_18, only_from.as_deref())));
        });
    }

    /// Reads the terms version and the own moderation state again (no spinner: it only updates what the screen shows).
    fn community_refresh_terms(&mut self, ctx: &egui::Context) {
        let Some(s) = self.community.session.clone() else { return };
        let (tx, ctx2) = (self.community.tx.clone(), ctx.clone());
        std::thread::spawn(move || {
            if let Ok(s) = current(&s, &tx) {
                let _ = tx.send(Message::Terms(load_terms(&s)));
                ctx2.request_repaint();
            }
        });
    }

    fn community_after_session(&mut self, ctx: &egui::Context, s: Session) {
        self.community.session = Some(s.clone());
        save_credential(Some(&s));
        self.community_launch(ctx, move |tx| {
            let _ = tx.send(Message::Profile(load_profile(&s)));
        });
    }

    /// Processes what the threads have finished. Called every frame.
    pub fn community_poll(&mut self, ctx: &egui::Context) {
        while let Ok(m) = self.community.rx.try_recv() {
            let is_session = matches!(m, Message::Session(_));
            match m {
                Message::Session(r) | Message::Login(r) => {
                    if is_session {
                        self.community.busy = self.community.busy.saturating_sub(1);
                    } else {
                        self.community.login = None;
                    }
                    match r {
                        Ok(s) => {
                            self.community_after_session(ctx, s);
                            self.community.state = t("Signed in.");
                        }
                        Err(e) => {
                            self.community.state = e;
                            self.community.session = None;
                        }
                    }
                }
                Message::SessionRenewed(s) => self.community.session = Some(s),
                Message::Profile(p) => {
                    self.community.busy = self.community.busy.saturating_sub(1);
                    self.community.profile_loaded = true;
                    if p.is_some() {
                        if let Some(s) = self.community.session.clone() {
                            let (tx, c2) = (self.community.tx.clone(), ctx.clone());
                            std::thread::spawn(move || {
                                let _ = tx.send(Message::MyProfile(load_my_profile(&s)));
                                let _ = tx.send(Message::Terms(load_terms(&s)));
                                if let Some((hearts, following)) = load_mine(&s) {
                                    let _ = tx.send(Message::Mine { hearts, following });
                                }
                                c2.request_repaint();
                            });
                        }
                    }
                    self.community.profile = p;
                    self.community_load_list(ctx);
                }
                Message::Items(r) => {
                    self.community.busy = self.community.busy.saturating_sub(1);
                    match r {
                        Ok(out) => {
                            self.community.items = out.items;
                            self.community.offline_age = out.stale;
                        }
                        Err(e) => self.community.state = e,
                    }
                }
                Message::Downloaded { item, bytes } => {
                    self.community.busy = self.community.busy.saturating_sub(1);
                    self.community.state = match bytes.and_then(|b| self.community_install(ctx, &item, &b)) {
                        Ok(m) => m,
                        Err(e) => e,
                    };
                }
                Message::Uploaded(r) => {
                    self.community.busy = self.community.busy.saturating_sub(1);
                    match r {
                        Ok(m) => {
                            self.community.state = m;
                            self.community.upload_backup = None;
                        }
                        Err(e) => {
                            // A refused field or the terms: give the form back so the user can fix it and send again.
                            let field = net::field_of_message(&e);
                            let terms = net::is_terms_required(&e);
                            if field.is_some() || terms {
                                if let Some(mut f) = self.community.upload_backup.take() {
                                    f.field_error = field;
                                    self.community.upload = f;
                                    self.community.open_upload = true;
                                }
                            }
                            if terms {
                                self.community.terms_open = Some(TermsMode::Accept);
                                self.community.terms_check = false;
                                self.community_refresh_terms(ctx);
                            }
                            self.community.state = e;
                        }
                    }
                    self.community_load_list(ctx);
                }
                Message::Claimed(r) => {
                    self.community.busy = self.community.busy.saturating_sub(1);
                    match r {
                        Ok(()) => {
                            self.community.state = t("Thanks: the item is hidden while a moderator reviews it. Details: docs/COPYRIGHT.md");
                            self.community_load_list(ctx); // the item is gone from the catalog
                        }
                        Err(e) => self.community.state = e,
                    }
                }
                Message::Terms(info) => {
                    let ask = matches!(info.decision(), net::TermsDecision::NeedAccept(_));
                    self.community.terms = Some(info);
                    if ask && !self.community.terms_prompted && self.community.profile.is_some() && self.community.terms_open.is_none() {
                        self.community.terms_prompted = true;
                        self.community.terms_check = false;
                        self.community.terms_open = Some(TermsMode::Accept);
                    }
                }
                Message::TermsAccepted(r) => {
                    self.community.busy = self.community.busy.saturating_sub(1);
                    match r {
                        Ok(()) => {
                            self.community.state = t("Terms accepted.");
                            self.community.terms_open = None;
                        }
                        Err(e) => self.community.state = e,
                    }
                    self.community_refresh_terms(ctx);
                }
                Message::Reported(r) => {
                    self.community.busy = self.community.busy.saturating_sub(1);
                    self.community.state = match r {
                        Ok(()) => t("Report sent. Thank you."),
                        Err(e) => e,
                    };
                }
                Message::AccountDeleted(r) => {
                    self.community.busy = self.community.busy.saturating_sub(1);
                    match r {
                        Ok(()) => {
                            self.community_close_local();
                            self.community.state = t("Your account and content were deleted.");
                        }
                        Err(e) => self.community.state = e,
                    }
                }
                Message::Image { path, bytes } => {
                    let tex = bytes.and_then(|b| {
                        val::validate_png("preview", &b).or_else(|_| val::validate_png("banner", &b)).ok()?;
                        let img = image::load_from_memory_with_format(&b, image::ImageFormat::Png).ok()?.to_rgba8();
                        let (w, h) = img.dimensions();
                        let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], img.as_raw());
                        // small images (pixel art) are shown sharp
                        let options = if w <= 128 && h <= 128 { egui::TextureOptions::NEAREST } else { egui::TextureOptions::LINEAR };
                        Some(ctx.load_texture(format!("community/{}", path), ci, options))
                    });
                    self.community.tex_gate.finish(&path, tex.is_some(), std::time::Instant::now());
                    if tex.is_some() {
                        self.community.textures.insert(path, tex);
                    }
                }
                Message::ProfileView(r) => {
                    self.community.busy = self.community.busy.saturating_sub(1);
                    match r {
                        Ok(v) => self.community.viewing = Some(v),
                        Err(e) => self.community.state = e,
                    }
                }
                Message::Mine { hearts, following } => {
                    self.community.my_hearts = hearts.into_iter().collect();
                    self.community.my_following = following.into_iter().collect();
                    if self.community.only_following {
                        self.community_load_list(ctx);
                    }
                }
                Message::MyProfile(p) => {
                    if let Some(p) = &p {
                        if !self.community.profile_form.loaded {
                            let f = &mut self.community.profile_form;
                            f.name = p.public_name.clone();
                            f.bio = p.bio.clone();
                            f.github = p.github.clone().unwrap_or_default();
                            f.links = NETWORKS.iter().map(|(k, _, _)| p.links.get(*k).cloned().unwrap_or_default()).collect();
                            f.loaded = true;
                        }
                    }
                    if let (Some(n), Some(pr)) = (p.as_ref().map(|x| x.public_name.clone()), self.community.profile.as_mut()) {
                        pr.public_name = n;
                    }
                    // the image under review may have changed: request it again
                    self.community.textures.retain(|k, _| !k.starts_with("quarantine:"));
                    self.community.tex_gate.forget_prefix("quarantine:");
                    self.community.my_profile = p;
                }
                Message::State(e) => {
                    self.community.busy = self.community.busy.saturating_sub(1);
                    self.community.state = e;
                }
                Message::TaskFailed => {
                    self.community.busy = self.community.busy.saturating_sub(1);
                    self.community.state = t("Something went wrong in a background task. Try again.");
                }
            }
        }
        // The server refused the session (expired / revoked): forget it so the sign-in buttons come back.
        if self.community.session.is_some() && net::is_expired(&self.community.state) {
            self.community_close_local();
        }
    }

    fn community_close_local(&mut self) {
        self.community.session = None;
        self.community.profile = None;
        self.community.profile_loaded = false;
        self.community.delete_confirmed = false;
        self.community.my_profile = None;
        self.community.terms = None;
        self.community.terms_open = None;
        self.community.terms_prompted = false;
        self.community.accepted = false;
        self.community.claim = None;
        self.community.upload_backup = None;
        self.community.profile_form = FormProfile::default();
        self.community.previous.clear();
        self.community.my_hearts.clear();
        self.community.my_following.clear();
        self.community.only_following = false;
        self.community.profile_open = false;
        save_credential(None);
    }

    fn community_install(&mut self, ctx: &egui::Context, item: &Item, bytes: &[u8]) -> Result<String, String> {
        let name = item.public_path.as_deref().and_then(|r| r.rsplit('/').next()).unwrap_or("archivo");
        val::validate(&item.kind, name, bytes)?; // second check right before touching the disk
        let msg = match item.kind.as_str() {
            "plugin" => {
                let n = install_plugin(bytes)?;
                self.plugins = crate::plugins::load_all();
                format!("{} “{}” (Plugins tab)", t("Plugin installed:"), n)
            }
            "image" => {
                let f = write_temp(name, bytes)?;
                let r = self.library.add_image(&f);
                let _ = std::fs::remove_file(&f);
                format!("{} “{}” (Images tab)", t("Image installed:"), r?)
            }
            "font" => {
                let f = write_temp(name, bytes)?;
                let r = crate::library::add_font_file(&f);
                let _ = std::fs::remove_file(&f);
                let (n, data, path) = r?;
                crate::render::add_font(ctx, &n, data, path);
                format!("{} “{}” (Text tab)", t("Font installed:"), n)
            }
            "theme" => {
                let theme: crate::theme::Theme = serde_json::from_slice(bytes).map_err(|_| "The theme is not valid.".to_string())?;
                theme.validate()?;
                crate::themes::save(&theme)?;
                let n = theme.name.clone();
                self.apply_theme(ctx, theme);
                format!("{} “{}” (Themes button to change it)", t("Theme installed and in use:"), n)
            }
            "template" => {
                let _untrusted = crate::plugins::UntrustedSource::begin(); // downloaded: absolute image paths are refused
                let f = write_temp(&format!("{}.json", safe_name(&item.title)), bytes)?;
                let r = self.library.add_template_file(&f);
                let _ = std::fs::remove_file(&f);
                format!("{} “{}” (Templates › My templates)", t("Template installed:"), r?)
            }
            _ => {
                let _untrusted = crate::plugins::UntrustedSource::begin();
                let f = write_temp(&format!("{}.json", safe_name(&item.title)), bytes)?;
                let r = self.library.add_element_file(&f);
                let _ = std::fs::remove_file(&f);
                format!("{} “{}” (Elements tab)", t("Element installed:"), r?)
            }
        };
        Ok(msg)
    }

    pub fn tab_community(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.community_poll(&ctx);
        if !self.community.started {
            // First time the tab is opened: any connection starts here (and not before).
            self.community.started = true;
            std::thread::spawn(|| net::prune_catalogs(&net::cache_dir(), now()));
            if let Some((r, u)) = read_credential() {
                self.community_launch(&ctx, move |tx| {
                    let _ = tx.send(Message::Session(renew(&Session { renewal: r, uid: u, ..Default::default() })));
                });
            } else {
                self.community_load_list(&ctx);
            }
        }

        ui.horizontal(|ui| {
            ui.label(RichText::new(t("Community")).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.community.busy > 0 {
                    ui.spinner();
                } else if ui.small_button(t("Refresh")).clicked() {
                    self.community_load_list(&ctx);
                    self.community_refresh_terms(&ctx);
                }
            });
        });
        ui.label(RichText::new(t("Shared plugins, images, fonts, templates and elements. Everything is reviewed before it is published.")).small().color(MUTED()));
        if ui.small_button(t("Community guidelines")).clicked() {
            let ask = self.community.session.is_some() && self.community.terms.as_ref().is_some_and(|i| matches!(i.decision(), net::TermsDecision::NeedAccept(_)));
            self.community.terms_tab = 1;
            self.community.terms_check = false;
            self.community.terms_open = Some(if ask { TermsMode::Accept } else { TermsMode::Read });
        }
        let block = match self.community.terms.as_ref().and_then(|i| i.own.as_ref()) {
            Some(o) => net::upload_block(o, &iso(now())),
            None => net::UploadBlock::None,
        };
        match &block {
            net::UploadBlock::None => {}
            net::UploadBlock::Suspended => {
                ui.label(RichText::new(t("Your account is suspended.")).small().color(WARN()));
            }
            net::UploadBlock::Permanent => {
                ui.label(RichText::new(t("Uploads are disabled for this account.")).small().color(WARN()));
            }
            net::UploadBlock::Until(d) => {
                ui.label(RichText::new(tf("Uploads are disabled until {}.", &[d])).small().color(WARN()));
            }
        }
        if !self.community.state.is_empty() {
            ui.add_space(4.0);
            ui.label(RichText::new(t(&self.community.state)).small().color(ACCENT()));
        }
        if self.community.offline_age.is_some() {
            ui.add_space(2.0);
            ui.label(RichText::new(t("Offline, showing the last known list.")).small().color(WARN()));
        }
        ui.add_space(6.0);
        self.community_account(ui, &ctx);
        ui.add_space(6.0);
        self.community_explore(ui, &ctx);
        let terms_unknown = self.community.terms.as_ref().is_some_and(|i| i.decision() == net::TermsDecision::Unknown);
        if terms_unknown && self.community.session.is_some() {
            ui.horizontal(|ui| {
                ui.label(RichText::new(t("Could not check the Terms of Use right now.")).small().color(WARN()));
                if ui.small_button(t("Retry")).clicked() {
                    self.community_refresh_terms(&ctx);
                }
            });
        }
        if self.community.session.is_some() && self.community.profile.is_some() && block == net::UploadBlock::None && !terms_unknown {
            ui.add_space(8.0);
            self.community_upload(ui, &ctx);
        }
    }

    fn community_account(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::Frame::new().fill(crate::theme::PANEL_DARK()).corner_radius(6.0).inner_margin(8.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            if self.community.session.is_none() {
                ui.label(RichText::new(t("Sign in to upload, report and enable 18+ content. Browsing needs no account.")).small());
                ui.label(RichText::new(t("No passwords: you sign in with your GitHub or Google account. The app only keeps a public name.")).small().color(MUTED()));
                ui.horizontal(|ui| {
                    for (key, name) in [("github", "GitHub"), ("google", "Google")] {
                        if ui.add_enabled(self.community.login.is_none(), egui::Button::new(format!("{} {}", t("Sign in with"), name))).clicked() {
                            self.community.state = t("Your browser opened: finish signing in there.");
                            let cancel = Arc::new(AtomicBool::new(false));
                            self.community.login = Some(cancel.clone());
                            let (tx, ctx2) = (self.community.tx.clone(), ctx.clone());
                            std::thread::spawn(move || {
                                let _ = tx.send(Message::Login(oauth_flow(key, &cancel)));
                                ctx2.request_repaint();
                            });
                        }
                    }
                });
                if let Some(c) = &self.community.login {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(RichText::new(t("Waiting for you to sign in in your browser…")).small().color(MUTED()));
                        if ui.small_button(t("Cancel")).clicked() {
                            c.store(true, Ordering::Relaxed);
                        }
                    });
                }
                return;
            }
            let Some(profile) = self.community.profile.clone() else {
                if !self.community.profile_loaded {
                    ui.label(RichText::new(t("Loading your profile…")).small().color(MUTED()));
                    return;
                }
                // First time: choose a public name and accept the rules.
                ui.label(RichText::new(t("Choose your public name")).strong());
                ui.label(RichText::new(t("It is all the community will see. 3-32 characters: letters, numbers, dot, hyphen and underscore.")).small().color(MUTED()));
                ui.add(egui::TextEdit::singleline(&mut self.community.new_name).hint_text("public_name").desired_width(f32::INFINITY));
                ui.horizontal(|ui| {
                    if self.community.accepted {
                        crate::icons::with_text(ui, "check", RichText::new(t("Terms accepted.")).small().color(ACCENT()), ACCENT());
                    }
                    if ui.small_button(if self.community.accepted { t("Terms of Use") } else { t("Read and accept the Terms and guidelines…") }).clicked() {
                        self.community.terms_tab = 0;
                        self.community.terms_check = self.community.accepted;
                        self.community.terms_open = Some(TermsMode::Create);
                    }
                });
                let n = self.community.new_name.trim().to_string();
                let valid = (3..=32).contains(&n.chars().count()) && n.chars().all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c));
                if ui.add_enabled(valid && self.community.accepted && self.community.busy == 0, egui::Button::new(t("Create profile"))).clicked() {
                    let s = self.community.session.clone().unwrap();
                    self.community_launch(ctx, move |tx| {
                        let r = current(&s, tx).and_then(|s| json_post("/rest/v1/profiles", Some(&s), &serde_json::json!({ "id": s.uid, "public_name": n, "terms_accepted_at": iso(now()) }), &[("Prefer", "return=minimal")])?.ok().map(|_| s));
                        match r {
                            Ok(s) => {
                                // The server reviews a new public name when the profile function runs (an empty save does that): until it is
                                // reviewed the name is not shown publicly. Best effort: the first upload or any later save does it as well.
                                let _ = json_post("/functions/v1/profile", Some(&s), &serde_json::json!({ "v": 2, "action": "save", "bio": "", "github": "", "links": {} }), &[]);
                                // Record the version the user just read (the profile must exist). Best effort: if it fails, the account asks again on the next load.
                                if let Some(v) = net::version_to_accept(fetch_current_terms(&s).as_deref(), &net::embedded_terms_version()) {
                                    let _ = json_post("/rest/v1/rpc/accept_terms", Some(&s), &net::accept_terms_body(&v), &[]);
                                }
                                let _ = tx.send(Message::Profile(load_profile(&s)));
                            }
                            Err(e) => {
                                let _ = tx.send(Message::State(e));
                            }
                        }
                    });
                }
                if ui.small_button(t("Sign out")).clicked() {
                    self.community_close_local();
                }
                return;
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{} {}", t("Session:"), profile.public_name)).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(t("Sign out")).clicked() {
                        let s = self.community.session.clone();
                        self.community_close_local();
                        if let Some(s) = s {
                            std::thread::spawn(move || {
                                let _ = request("POST", "/auth/v1/logout?scope=local", Some(&s), None, None, &[], 1 << 16);
                            });
                        }
                        self.community.state = t("Signed out.");
                    }
                });
            });
            let mut m18 = profile.show_18;
            if !profile.show_18 {
                ui.checkbox(&mut self.community.adult, t("I confirm I am 18 or older"));
                if self.community.adult && ui.small_button(t("Show 18+ content")).clicked() {
                    m18 = true;
                }
            } else if ui.small_button(t("Hide 18+ content")).clicked() {
                m18 = false;
            }
            if m18 != profile.show_18 {
                let s = self.community.session.clone().unwrap();
                if let Some(p) = self.community.profile.as_mut() {
                    p.show_18 = m18;
                }
                self.community_launch(ctx, move |tx| {
                    let r = current(&s, tx).and_then(|s| request("PATCH", &format!("/rest/v1/profiles?id=eq.{}", s.uid), Some(&s), None, Some(if m18 { serde_json::json!({ "show_18": true, "declared_adult_at": iso(now()) }) } else { serde_json::json!({ "show_18": false }) }.to_string().as_bytes()), &[], 1 << 16)?.ok().map(|_| ()));
                    let _ = tx.send(Message::State(match r {
                        Ok(()) => String::new(),
                        Err(e) => e,
                    }));
                });
                self.community_load_list(ctx);
            }
            if ui.button(t("Edit my creator profile")).clicked() {
                self.community.profile_open = true;
            }
            ui.collapsing(t("My account and data"), |ui| {
                if ui.add_enabled(self.community.busy == 0, egui::Button::new(t("Export my data"))).clicked() {
                    if let Some(path) = rfd::FileDialog::new().set_file_name("lienzo-my-data.json").add_filter("JSON", &["json"]).save_file() {
                        let s = self.community.session.clone().unwrap();
                        self.community_launch(ctx, move |tx| {
                            let r = current(&s, tx).and_then(|s| export_my_data(&s, &path));
                            let _ = tx.send(Message::State(match r {
                                Ok(()) => tf("Your data was saved to {}", &[&path.display().to_string()]),
                                Err(e) => e,
                            }));
                        });
                    }
                }
                ui.label(RichText::new(t("We only keep your public name, your 18+ preference and what you upload. Your email is handled by the sign-in provider and is never shown to anyone.")).small().color(MUTED()));
                ui.checkbox(&mut self.community.delete_confirmed, t("I want to delete my account and everything I uploaded (cannot be undone)"));
                if ui.add_enabled(self.community.delete_confirmed && self.community.busy == 0, egui::Button::new(RichText::new(t("Delete my account")).color(WARN()))).clicked() {
                    let s = self.community.session.clone().unwrap();
                    self.community_launch(ctx, move |tx| {
                        let r = current(&s, tx).and_then(|s| json_post("/functions/v1/delete-account", Some(&s), &serde_json::json!({ "v": 2 }), &[])?.ok().map(|_| ()));
                        let _ = tx.send(Message::AccountDeleted(r));
                    });
                }
            });
        });
    }

    fn community_explore(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.community.viewing.is_some() {
            self.community_view_profile(ui, ctx);
            return;
        }
        let mut reload = false;
        ui.horizontal_wrapped(|ui| {
            if ui.selectable_label(self.community.filter.is_none(), t("All")).clicked() {
                self.community.filter = None;
                reload = true;
            }
            for (k, n) in KIND_NAMES {
                if ui.selectable_label(self.community.filter == Some(*k), t(n)).clicked() {
                    self.community.filter = Some(k);
                    reload = true;
                }
            }
            if self.community.session.is_some() && ui.add(crate::icons::btn("heart", t("Following")).selected(self.community.only_following)).on_hover_text(t("Only content from creators you follow")).clicked() {
                self.community.only_following = !self.community.only_following;
                reload = true;
            }
        });
        let r = ui.add(egui::TextEdit::singleline(&mut self.community.search).hint_text(t("Search the community…")).desired_width(f32::INFINITY));
        if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            reload = true;
        }
        if reload {
            self.community_load_list(ctx);
        }
        ui.add_space(4.0);
        if self.community.items.is_empty() {
            ui.label(RichText::new(if self.community.busy > 0 { t("Loading…") } else { t("There is no content to show yet.") }).color(MUTED()));
            return;
        }
        let items = self.community.items.clone();
        let signed_in = self.community.session.is_some() && self.community.profile.is_some();
        for it in items {
            self.community_card(ui, ctx, &it, signed_in);
            ui.add_space(4.0);
        }
    }

    fn community_upload(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let open = std::mem::take(&mut self.community.open_upload);
        egui::CollapsingHeader::new(RichText::new(t("Share with the community")).strong()).id_salt("community_upload").open(open.then_some(true)).show(ui, |ui| {
            ui.label(RichText::new(t("It is reviewed before being published (format, antivirus and moderation). If it is 18+, mark it.")).small().color(MUTED()));
            egui::ComboBox::from_id_salt("community_kind").selected_text(t(KIND_NAMES[self.community.upload.kind].1)).show_ui(ui, |ui| {
                for (i, (_, n)) in KIND_NAMES.iter().enumerate() {
                    ui.selectable_value(&mut self.community.upload.kind, i, t(n));
                }
            });
            let kind = KIND_NAMES[self.community.upload.kind].0;
            ui.add(egui::TextEdit::singleline(&mut self.community.upload.title).hint_text(t("Title (3-80 characters)")).desired_width(f32::INFINITY));
            ui.add(egui::TextEdit::multiline(&mut self.community.upload.description).hint_text(t("Description (max. 1000)")).desired_rows(3).desired_width(f32::INFINITY));
            ui.add(egui::TextEdit::singleline(&mut self.community.upload.labels).hint_text(t("Tags separated by commas (max. 8)")).desired_width(f32::INFINITY));
            ui.checkbox(&mut self.community.upload.is_18, t("Contains 18+ content"));
            // Licence, source link and rights declaration (migration 0015).
            let eff = self.community.upload.license.unwrap_or_else(|| net::default_license(kind));
            let bad = self.community.upload.field_error;
            ui.horizontal(|ui| {
                let l = RichText::new(t("License"));
                ui.label(if bad == Some("license") { l.color(WARN()) } else { l });
                egui::ComboBox::from_id_salt("community_license").selected_text(t(net::LICENSES[eff].1)).show_ui(ui, |ui| {
                    for (i, (_, n)) in net::LICENSES.iter().enumerate() {
                        if ui.selectable_label(eff == i, t(n)).clicked() {
                            self.community.upload.license = Some(i);
                        }
                    }
                });
            });
            let src_ok = self.community.upload.source.trim().is_empty() || net::valid_source_url(self.community.upload.source.trim());
            let mut source = egui::TextEdit::singleline(&mut self.community.upload.source).hint_text(t("Source link (optional)")).desired_width(f32::INFINITY);
            if !src_ok || bad == Some("source") {
                source = source.text_color(WARN());
            }
            ui.add(source);
            if !src_ok {
                ui.label(RichText::new(t("Use an http or https link, up to 300 characters, no spaces.")).small().color(WARN()));
            }
            let rights = RichText::new(t("I own this or have the right to share it under this license"));
            ui.checkbox(&mut self.community.upload.rights, if bad == Some("rights") { rights.color(WARN()) } else { rights });
            ui.horizontal(|ui| {
                let button = if kind == "plugin" { t("Choose plugin folder…") } else { t("Choose file…") };
                if ui.button(button).clicked() {
                    self.community.upload.file = match kind {
                        "plugin" => rfd::FileDialog::new().set_title("Plugin folder (with plugin.json)").pick_folder(),
                        "image" => rfd::FileDialog::new().add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp"]).pick_file(),
                        "font" => rfd::FileDialog::new().add_filter("Fonts", &["ttf", "otf"]).pick_file(),
                        _ => rfd::FileDialog::new().add_filter("JSON", &["json"]).pick_file(),
                    };
                }
                if kind == "theme" && ui.button(t("Use the current theme")).clicked() {
                    let theme = self.settings.theme.clone().unwrap_or_default();
                    let f = std::env::temp_dir().join("lienzo-ui-community");
                    let _ = std::fs::create_dir_all(&f);
                    let f = f.join("tema-actual.json");
                    if std::fs::write(&f, serde_json::to_vec_pretty(&theme).unwrap_or_default()).is_ok() {
                        self.community.upload.file = Some(f);
                        if self.community.upload.title.trim().is_empty() {
                            self.community.upload.title = theme.name.clone();
                        }
                    }
                }
                if kind == "template" && ui.button(t("Use the open design")).clicked() {
                    let f = std::env::temp_dir().join("lienzo-ui-community");
                    let _ = std::fs::create_dir_all(&f);
                    let f = f.join("current-design.json");
                    if std::fs::write(&f, serde_json::to_vec(&self.doc).unwrap_or_default()).is_ok() {
                        self.community.upload.file = Some(f);
                    }
                }
            });
            if let Some(a) = &self.community.upload.file {
                ui.label(RichText::new(a.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()).small());
            }
            ui.label(RichText::new(t("Optional: preview (PNG up to 1920×1080). The app sets the content icon from its type.")).small().color(MUTED()));
            ui.horizontal(|ui| {
                let (icon, text) = match &self.community.upload.view {
                    Some(p) => ("check", format!("{} {}", t("Preview…"), p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default())),
                    None => ("image", t("Preview…")),
                };
                if ui.add(crate::icons::btn(icon, text)).clicked() {
                    self.community.upload.view = rfd::FileDialog::new().add_filter("PNG", &["png"]).pick_file();
                }
            });
            let f = &self.community.upload;
            let labels: Vec<String> = f.labels.split(',').map(|e| e.trim().to_lowercase()).filter(|e| !e.is_empty()).take(8).collect();
            let ready = (3..=80).contains(&f.title.trim().chars().count()) && val::valid_display_text(&f.title) && f.file.is_some() && self.community.busy == 0 && f.rights && src_ok;
            if ui.add_enabled(ready, egui::Button::new(t("Send for review"))).clicked() {
                let f = &self.community.upload;
                let (title, desc, is_18, path) = (f.title.trim().to_string(), f.description.chars().take(1000).collect::<String>(), f.is_18, f.file.clone().unwrap());
                let meta = UploadMeta { license: Some(net::LICENSES[f.license.unwrap_or_else(|| net::default_license(kind))].0.to_string()), source_url: f.source.trim().to_string(), rights: f.rights };
                let read = if path.is_dir() { pack_plugin(&path).map(|b| ("plugin.json".to_string(), b)) } else { std::fs::read(&path).map(|b| (path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(), b)).map_err(|e| e.to_string()) };
                let mut extras: Vec<(&'static str, Vec<u8>)> = Vec::new();
                let mut extra_failure = None;
                for (class, path) in [("preview", self.community.upload.view.clone())] {
                    if let Some(r) = path {
                        match std::fs::read(&r) {
                            Ok(b) => extras.push((class, b)),
                            Err(e) => extra_failure = Some(e.to_string()),
                        }
                    }
                }
                let read = match extra_failure {
                    Some(e) => Err(e),
                    None => read,
                };
                match read {
                    Err(e) => self.community.state = e,
                    Ok((name, bytes)) => {
                        let s = self.community.session.clone().unwrap();
                        self.community.state = t("Sending and analysing… (can take up to a minute)");
                        self.community_launch(ctx, move |tx| {
                            let extras: Vec<(&str, Vec<u8>)> = extras.into_iter().collect();
                            let r = upload_content(&s, kind, &title, &desc, labels, is_18, &meta, &name, &bytes, &extras, tx);
                            let _ = tx.send(Message::Uploaded(r));
                        });
                        self.community.upload_backup = Some(self.community.upload.clone());
                        self.community.upload = UploadForm { kind: self.community.upload.kind, ..Default::default() };
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_reads_never_use_select_star() {
        // Migration 0014 hides the pending flags / updated_at of the table from everybody: a `select=*` on public_profiles would fail.
        let src = include_str!("community.rs");
        let needle = ["public_profiles?select", "=*"].concat(); // built at run time so this test does not match itself
        for (n, line) in src.lines().enumerate() {
            assert!(!line.contains(&needle), "line {}: select=* on public_profiles", n + 1);
        }
        assert!(!PROFILE_PUBLIC_COLUMNS.contains("pending") && !PROFILE_PUBLIC_COLUMNS.contains("updated_at"));
        assert!(PROFILE_OWN_COLUMNS.contains("avatar_pending") && PROFILE_OWN_COLUMNS.contains("banner_pending") && !PROFILE_OWN_COLUMNS.contains("updated_at"));
    }

    #[test]
    fn iso_date() {
        assert_eq!(iso(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso(1_794_865_809), "2026-11-16T21:50:09Z");
    }

    #[test]
    fn uuid_and_names() {
        let u = uuid_v4();
        assert!(is_uuid(&u) && u.as_bytes()[14] == b'4');
        assert_eq!(safe_name("../mi plugin (1).json"), "_mi_plugin__1_.json");
    }

    #[test]
    fn encode_and_search() {
        assert_eq!(encode("a b/c"), "a%20b%2Fc");
        assert_eq!(clear_search("hola,(x)*%"), "holax");
    }

    #[test]
    fn messages_do_not_leak_the_body() {
        let e = error_message(500, r#"{"message":"key=SECRET"}"#);
        assert!(!e.contains("SECRET"));
        assert!(error_message(401, "").contains("session"));
    }

    #[test]
    fn contract_error_codes_become_readable_messages() {
        let body = r#"{"code":"P0001","details":null,"hint":"terms_required","message":"terms_required: Accept the current Terms of Use (version 1.0) to upload."}"#;
        assert!(net::is_terms_required(&error_message(400, body)));
        let banned = r#"{"code":"P0001","hint":"upload_banned","message":"upload_banned: Uploads are disabled until 2026-12-01 10:00 UTC."}"#;
        assert!(error_message(400, banned).contains("2026-12-01"));
        let lic = error_message(400, r#"{"hint":"license_invalid","message":"license_invalid: x"}"#);
        assert_eq!(net::field_of_message(&lic), Some("license"));
        assert!(!error_message(500, r#"{"hint":"x","message":"key=SECRET"}"#).contains("SECRET"));
        // the new item columns are optional: an old row has none
        let it: Item = serde_json::from_str(r#"{"id":"x"}"#).unwrap();
        assert!(it.license.is_none() && it.source_url.is_none());
        let it: Item = serde_json::from_str(r#"{"id":"x","license":"mit","source_url":"https://a.b/c"}"#).unwrap();
        assert_eq!(it.license.as_deref(), Some("mit"));
    }

    #[test]
    fn known_pkce_challenge() {
        // RFC 7636, appendix B
        let d = b64url(&Sha256::digest(b"dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"));
        assert_eq!(d, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn session_is_read_from_the_response() {
        let v = serde_json::json!({ "access_token": "a", "refresh_token": "r", "expires_in": 3600, "user": { "id": "123e4567-e89b-42d3-a456-426614174000" } });
        let s = session_from(&v).unwrap();
        assert_eq!(s.renewal, "r");
        assert!(session_from(&serde_json::json!({ "access_token": "a", "refresh_token": "r", "user": { "id": "no-es-uuid" } })).is_err());
    }

    #[test]
    fn packs_the_example_plugin() {
        let c = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("plugins/snap-to-grid");
        let b = pack_plugin(&c).unwrap();
        assert!(val::validate("plugin", "p.json", &b).is_ok());
    }

    #[test]
    fn cli_accepts_english_and_old_spanish_names() {
        assert_eq!((cli_kind("image"), cli_kind("imagen"), cli_kind("font"), cli_kind("fuente")), ("image", "image", "font", "font"));
        assert_eq!((cli_kind("template"), cli_kind("plantilla"), cli_kind("element"), cli_kind("theme"), cli_kind("plugin")), ("template", "template", "element", "theme", "plugin"));
        assert_eq!((cli_kind("elemento"), cli_kind("tema")), ("element", "theme"));
        assert_eq!((review_action("approve"), review_action("aprobar"), review_action("reject-profile"), review_action("rechazar-perfil")), ("approve", "approve", "reject-profile", "reject-profile"));
        assert_eq!((review_action("token-create"), review_action("token-crear"), review_action("reanalizar"), review_action("rescan"), review_action("list"), review_action("tokens")), ("token-create", "token-create", "rescan", "rescan", "list", "tokens"));
        assert_eq!((review_scope("read"), review_scope("leer"), review_scope("reject"), review_scope("aprobar")), ("read", "read", "reject", "approve"));
        assert!(cli_kind("image").len() > 0 && val::KINDS.contains(&cli_kind("image")));
    }

    #[test]
    fn english_wire_names_of_the_server_are_read() {
        let row = r#"{"id":"x","kind":"template","title":"T","description":"D","tags":["a"],"is_18":true,"public_path":"p/q.json","downloads":3,
            "author":"u","author_name":"N","public_icon":"i.png","public_preview":"v.png","hearts":2}"#;
        let it: Item = serde_json::from_str(row).unwrap();
        assert_eq!((it.kind.as_str(), it.title.as_str(), it.is_18, it.downloads, it.author_name.as_str(), it.hearts), ("template", "T", true, 3, "N", 2));
        assert_eq!((it.tags.as_slice(), it.public_preview.as_deref()), (&["a".to_string()][..], Some("v.png")));
        let p: ProfilePublic = serde_json::from_str(r#"{"id":"u","public_name":"N","links":{"x":"https://x.com/a"},"avatar_pending":true,"followers":4,"hearts_received":5}"#).unwrap();
        assert_eq!((p.public_name.as_str(), p.avatar_pending, p.followers, p.hearts_received, p.links.len()), ("N", true, 4, 5, 1));
    }
}

/// Upload from the command line, using the session saved by the app (Credential Manager).
///   lienzo-ui --community-upload <kind> <file|folder-of-plugin> --title "…" [--description "…"] [--labels a,b] [--18]
/// Uses exactly the same code as the "Submit for review" button. Close the app before using it: the session
/// token is renewed and the app, left open with the old token, would lose the session.
/// CLI names for the kinds of content. English names are primary; the old Spanish ones are still accepted.
/// The value returned is the canonical English kind the server stores.
fn cli_kind(k: &str) -> &str {
    match k {
        "image" | "imagen" => "image",
        "font" | "fuente" => "font",
        "template" | "plantilla" => "template",
        "element" | "elemento" => "element",
        "theme" | "tema" => "theme",
        other => other,
    }
}

/// Review actions and token scopes of `--community-review`: English names are primary, old Spanish ones still work.
/// Returns the canonical English name (the server API v2 uses it, with `_` instead of `-`).
fn review_action(a: &str) -> &str {
    match a {
        "list" | "listar" => "list",
        "approve" | "aprobar" => "approve",
        "reject" | "rechazar" => "reject",
        "approve-profile" | "aprobar-perfil" => "approve-profile",
        "reject-profile" | "rechazar-perfil" => "reject-profile",
        "token-create" | "token-crear" => "token-create",
        "token-revoke" | "token-revocar" => "token-revoke",
        "rescan" | "reanalizar" => "rescan",
        "cleanup" | "limpiar" => "cleanup",
        other => other,
    }
}

fn review_scope(a: &str) -> &str {
    match a {
        "read" | "leer" => "read",
        "approve" | "aprobar" => "approve",
        "reject" | "rechazar" => "reject",
        other => other,
    }
}

pub fn upload_cli(args: &[String]) -> i32 {
    let kind = cli_kind(args.first().map(|s| s.as_str()).unwrap_or(""));
    let path = PathBuf::from(args.get(1).map(|s| s.as_str()).unwrap_or(""));
    let option = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned().unwrap_or_default();
    let title = option("--title");
    let labels: Vec<String> = option("--labels").split(',').map(|e| e.trim().to_lowercase()).filter(|e| !e.is_empty()).take(8).collect();
    let is_18 = args.iter().any(|a| a == "--18");
    if !val::KINDS.contains(&kind) || !path.exists() || title.trim().chars().count() < 3 || !val::valid_display_text(&title) {
        eprintln!("Usage: lienzo-ui --community-upload <plugin|image|font|template|element|theme> <file|folder> --title \"…\" [--description \"…\"] [--labels a,b] [--18] [--icon icon.png] [--preview preview.png] [--license cc-by --rights [--source https://…]]");
        return 2;
    }
    let Some((renewal, uid)) = read_credential() else {
        eprintln!("No saved session: open the app, Community tab, and sign in.");
        return 1;
    };
    let s = match renew(&Session { renewal, uid, ..Default::default() }) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Could not renew the session: {}", e);
            return 1;
        }
    };
    save_credential(Some(&s)); // the token rotates: the new one is saved for next time
    if load_profile(&s).is_none() {
        eprintln!("This account does not have a profile yet: create it in the app (Community tab).");
        return 1;
    }
    let read = if path.is_dir() { pack_plugin(&path).map(|b| ("plugin.json".to_string(), b)) } else { std::fs::read(&path).map(|b| (path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(), b)).map_err(|e| e.to_string()) };
    let (name, bytes) = match read {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let mut extras: Vec<(&str, Vec<u8>)> = Vec::new();
    for (class, k) in [("icon", "--icon"), ("preview", "--preview")] {
        let r = option(k);
        if !r.is_empty() {
            match std::fs::read(&r) {
                Ok(b) => extras.push((class, b)),
                Err(e) => {
                    eprintln!("{}: {}", r, e);
                    return 1;
                }
            }
        }
    }
    let (tx, _rx) = channel();
    let meta = UploadMeta { license: Some(option("--license")).filter(|l| !l.is_empty()), source_url: option("--source"), rights: args.iter().any(|a| a == "--rights") };
    match upload_content(&s, kind, title.trim(), &option("--description"), labels, is_18, &meta, &name, &bytes, &extras, &tx) {
        Ok(m) => {
            println!("{}: {}", kind, m);
            0
        }
        Err(e) => {
            eprintln!("{}: {}", kind, val::terminal_safe(&e));
            1
        }
    }
}

/// Retries moderation of your listings left "pending" with no decision (for example, if the server failed).
///   lienzo-ui --community-retry
pub fn retry_cli() -> i32 {
    let Some((renewal, uid)) = read_credential() else {
        eprintln!("No saved session: open the app, Community tab, and sign in.");
        return 1;
    };
    let s = match renew(&Session { renewal, uid, ..Default::default() }) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Could not renew the session: {}", e);
            return 1;
        }
    };
    save_credential(Some(&s));
    let list = request("GET", &format!("/rest/v1/community_items?select=id,title&author=eq.{}&status=eq.pending&reviewed_at=is.null&quarantine_path=not.is.null", s.uid), Some(&s), None, None, &[], 1 << 20).and_then(|r| r.ok());
    let rows: Vec<serde_json::Value> = match list.and_then(|r| r.json()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let mut failures = 0;
    for f in &rows {
        let (id, title) = (f["id"].as_str().unwrap_or(""), f["title"].as_str().unwrap_or(""));
        if !is_uuid(id) {
            continue;
        }
        match json_post("/functions/v1/moderate", Some(&s), &serde_json::json!({ "v": 2, "item": id }), &[]).and_then(|r| r.ok()).and_then(|r| r.json::<serde_json::Value>()) {
            Ok(mut v) => {
                val::terminal_safe_json(&mut v);
                println!("{}: {} ({})", val::terminal_safe(title), v["status"].as_str().unwrap_or("?"), v["reason"].as_str().unwrap_or(""))
            }
            Err(e) => {
                eprintln!("{}: {}", val::terminal_safe(title), val::terminal_safe(&e));
                failures += 1;
            }
        }
    }
    println!("{} listing(s) processed.", rows.len());
    if failures > 0 { 1 } else { 0 }
}

fn saved_session() -> Result<Session, String> {
    let (renewal, uid) = read_credential().ok_or("No saved session: open the app, Community tab, and sign in.")?;
    let s = renew(&Session { renewal, uid, ..Default::default() }).map_err(|e| format!("Could not renew the session: {}", e))?;
    save_credential(Some(&s)); // the token rotates: the new one is saved
    Ok(s)
}

/// Your account identifier (to configure it as a moderator on the server).
///   lienzo-ui --community-me
pub fn me_cli() -> i32 {
    match saved_session() {
        Ok(s) => {
            println!("{}", s.uid);
            0
        }
        Err(e) => {
            eprintln!("{}", e);
            1
        }
    }
}

/// Maximum number of `cleanup` calls per run of `--community-review cleanup` (each call removes up to 25 files).
const CLEANUP_MAX_CALLS: usize = 20;

/// Repeats the server's `cleanup` action until it reports `done: 0` (queue drained or nothing more to do right now)
/// or the call bound is reached. Prints how many files were removed. Exit code 1 on a request/server error.
fn cleanup_loop(s: &Session, body: &serde_json::Value) -> i32 {
    let (mut removed, mut failed) = (0i64, 0i64);
    for call in 1..=CLEANUP_MAX_CALLS {
        let r = match json_post("/functions/v1/review", Some(s), body, &[]) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}", val::terminal_safe(&e));
                println!("{}", crate::i18n::tf("cleanup: {} file(s) removed before the error.", &[&removed.to_string()]));
                return 1;
            }
        };
        let mut v: serde_json::Value = r.json().unwrap_or_default();
        val::terminal_safe_json(&mut v);
        if !(200..300).contains(&r.state) {
            eprintln!("{}", v["error"].as_str().unwrap_or("Server error."));
            println!("{}", crate::i18n::tf("cleanup: {} file(s) removed before the error.", &[&removed.to_string()]));
            return 1;
        }
        let done = v["done"].as_i64().unwrap_or(0).max(0);
        removed += done;
        failed += v["failed"].as_i64().unwrap_or(0).max(0);
        if done == 0 {
            break;
        }
        if call == CLEANUP_MAX_CALLS {
            println!("{}", crate::i18n::tf("cleanup: call limit reached ({}); run it again if more files are queued.", &[&CLEANUP_MAX_CALLS.to_string()]));
        }
    }
    if failed > 0 {
        println!("{}", crate::i18n::tf("cleanup: {} file(s) removed, {} failed (they will be retried).", &[&removed.to_string(), &failed.to_string()]));
    } else {
        println!("{}", crate::i18n::tf("cleanup: {} file(s) removed.", &[&removed.to_string()]));
    }
    0
}

/// Human review (only accounts listed in the server's MODERATORS secret):
///   lienzo-ui --community-review list
///   lienzo-ui --community-review approve <id> [--18]      · reject <id> "reason"
///   lienzo-ui --community-review approve-profile <account-id> avatar|banner  · reject-profile <account-id> avatar|banner
///   lienzo-ui --community-review token-create "name" read,reject [days]    (credential for scripts; shown once)
///   lienzo-ui --community-review tokens                                       · token-revoke <id>
///   lienzo-ui --community-review cleanup                                      (drains the takedown queue: removes the files of rejected / deleted listings; needs the reject scope)
/// (The old Spanish action names — listar, aprobar, rechazar, … — still work.)
/// The same can be done in the moderation web page (web/moderacion).
pub fn review_cli(args: &[String]) -> i32 {
    let action = review_action(args.first().map(|s| s.as_str()).unwrap_or(""));
    let a1 = args.get(1).map(|s| s.as_str()).unwrap_or("");
    let a2 = args.get(2).map(|s| s.as_str()).unwrap_or("");
    let usage = "Usage: lienzo-ui --community-review list | approve <id> [--18] | reject <id> \"reason\" | approve-profile <account> avatar|banner | reject-profile <account> avatar|banner | token-create \"name\" read,reject [days] | tokens | token-revoke <id> | rescan [id] | cleanup";
    let body = match action {
        "list" | "tokens" => serde_json::json!({ "v": 2, "action": if action == "tokens" { "list_tokens" } else { "list" } }),
        "approve" | "reject" if is_uuid(a1) => serde_json::json!({ "v": 2, "action": action, "item": a1, "reason": a2, "is_18": args.iter().any(|x| x == "--18") }),
        "approve-profile" | "reject-profile" if is_uuid(a1) && ["avatar", "banner"].contains(&a2) => serde_json::json!({ "v": 2, "action": action.replace('-', "_"), "user": a1, "class": a2 }),
        "token-create" if !a1.is_empty() => {
            let scopes: Vec<&str> = a2.split(',').filter(|x| !x.is_empty()).map(review_scope).collect();
            let days: u32 = args.get(3).and_then(|d| d.parse().ok()).unwrap_or(30);
            serde_json::json!({ "v": 2, "action": "create_token", "name": a1, "scopes": scopes, "days": days })
        }
        "token-revoke" if is_uuid(a1) => serde_json::json!({ "v": 2, "action": "revoke_token", "id": a1 }),
        "rescan" if a1.is_empty() || is_uuid(a1) => serde_json::json!({ "v": 2, "action": "rescan", "item": if a1.is_empty() { serde_json::Value::Null } else { serde_json::json!(a1) } }),
        "cleanup" => serde_json::json!({ "v": 2, "action": "cleanup" }),
        _ => {
            eprintln!("{}", usage);
            return 2;
        }
    };
    let s = match saved_session() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    if action == "cleanup" {
        return cleanup_loop(&s, &body);
    }
    let r = match json_post("/functions/v1/review", Some(&s), &body, &[]) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let mut v: serde_json::Value = r.json().unwrap_or_default();
    val::terminal_safe_json(&mut v); // titles, author names and reasons come from other people: no ESC/BEL/control characters in the terminal
    if !(200..300).contains(&r.state) {
        eprintln!("{}", v["error"].as_str().unwrap_or("Server error."));
        return 1;
    }
    match action {
        "list" => {
            let list = v["pending"].as_array().cloned().unwrap_or_default();
            for f in &list {
                println!("{}  [{}] {}  (by {}, {} bytes{})", f["id"].as_str().unwrap_or(""), f["kind"].as_str().unwrap_or(""), f["title"].as_str().unwrap_or(""), f["author_name"].as_str().unwrap_or(""), f["bytes"].as_i64().unwrap_or(0), if f["is_18"].as_bool().unwrap_or(false) { ", +18" } else { "" });
                println!("    reason: {}", f["rejection_reason"].as_str().unwrap_or("-"));
            }
            let profiles = v["profiles"].as_array().cloned().unwrap_or_default();
            for p in &profiles {
                println!("{}  [profile {}] by {}", p["user"].as_str().unwrap_or(""), p["class"].as_str().unwrap_or(""), p["public_name"].as_str().unwrap_or(""));
            }
            println!("{} content item(s) and {} profile image(s) pending. To see the files use the moderation website.", list.len(), profiles.len());
        }
        "tokens" => {
            for t in v["tokens"].as_array().cloned().unwrap_or_default() {
                println!("{}  {}  [{}]  expires {}{}", t["id"].as_str().unwrap_or(""), t["name"].as_str().unwrap_or(""), t["scopes"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(",")).unwrap_or_default(), t["expires_at"].as_str().unwrap_or("").get(..10).unwrap_or(""), if t["revoked"].as_bool().unwrap_or(false) { "  (revoked)" } else { "" });
            }
        }
        "token-create" => {
            println!("Token created (expires {}). Copy it now, it cannot be viewed again:", v["expires_at"].as_str().unwrap_or(""));
            println!("{}", v["token"].as_str().unwrap_or(""));
        }
        "rescan" => println!("rescan: {} {}{}", v["status"].as_str().unwrap_or("?"), v["title"].as_str().unwrap_or(""), v["reason"].as_str().map(|m| format!(" · {}", m)).unwrap_or_default()),
        _ => println!("{}: {}", action, v["status"].as_str().unwrap_or("?")),
    }
    0
}

// ───────────────────────────── cards, images and creator profiles ─────────────────────────────

/// X-style labeled field: a box with the small label on top and the text below.
fn field_x(ui: &mut egui::Ui, label: &str, text: &mut String, rows: usize, track: &str) {
    egui::Frame::new().stroke(egui::Stroke::new(1.0, crate::theme::LINE())).corner_radius(5.0).inner_margin(egui::Margin::symmetric(10, 6)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(label).small().color(MUTED()));
        let editor = if rows > 1 { egui::TextEdit::multiline(text).desired_rows(rows) } else { egui::TextEdit::singleline(text) };
        ui.add(editor.frame(egui::Frame::NONE).desired_width(f32::INFINITY).hint_text(track));
    });
    ui.add_space(6.0);
}

/// The source link of an item: plain text on screen, opened only on click and only when it is a clean http(s) address.
fn open_source(url: &str) {
    if net::valid_source_url(url) {
        let _ = webbrowser::open(url);
    }
}

fn open_link(url: &str) {
    // Last check before opening the browser: https only.
    if url.starts_with("https://") && url.chars().all(|c| c.is_ascii_graphic()) {
        let _ = webbrowser::open(url);
    }
}

impl EditorApp {
    /// Public community image (icon, preview, avatar or banner). Downloaded in the background the first time.
    fn community_texture(&mut self, ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
        if let Some(Some(t)) = self.community.textures.get(path) {
            return Some(t.clone());
        }
        // not marked as unavailable before the download: a failure only pauses new attempts for a while
        if !self.community.tex_gate.may_start(path, std::time::Instant::now()) {
            return None;
        }
        if !valid_public_path(path) {
            self.community.tex_gate.finish(path, false, std::time::Instant::now());
            return None;
        }
        let (tx, c2, r) = (self.community.tx.clone(), ctx.clone(), path.to_string());
        std::thread::spawn(move || {
            let dir = net::cache_dir().join("thumbs");
            let cached = net::thumb_get(&dir, &r).filter(|b| val::detect(b) == Some("png"));
            let bytes = cached.or_else(|| {
                let b = request("GET", &format!("/storage/v1/object/public/publico/{}", r), None, None, None, &[], (2 << 20) + 1)
                    .and_then(|x| x.ok())
                    .ok()
                    .map(|x| x.body)
                    .filter(|b| val::detect(b) == Some("png"))?;
                let _ = net::thumb_put(&dir, &r, &b, net::THUMBS_MAX_BYTES);
                Some(b)
            });
            let _ = tx.send(Message::Image { path: r, bytes });
            c2.request_repaint();
        });
        None
    }

    /// Adjusts a content item's heart total in what is being displayed (without waiting for the server).
    fn adjust_hearts(&mut self, id: &str, delta: i64) {
        let apply = |v: &mut Vec<Item>| {
            for i in v.iter_mut().filter(|i| i.id == id) {
                i.hearts = (i.hearts + delta).max(0);
            }
        };
        apply(&mut self.community.items);
        if let Some((p, v)) = self.community.viewing.as_mut() {
            let before: i64 = v.iter().find(|i| i.id == id).map(|i| i.hearts).unwrap_or(0);
            apply(v);
            let after: i64 = v.iter().find(|i| i.id == id).map(|i| i.hearts).unwrap_or(0);
            p.hearts_received = (p.hearts_received + (after - before)).max(0);
        }
    }

    fn community_heart(&mut self, ctx: &egui::Context, item: String) {
        let Some(s) = self.community.session.clone() else { return };
        if !is_uuid(&item) {
            return;
        }
        let hearted = self.community.my_hearts.contains(&item);
        if hearted {
            self.community.my_hearts.remove(&item);
        } else {
            self.community.my_hearts.insert(item.clone());
        }
        self.adjust_hearts(&item, if hearted { -1 } else { 1 });
        let (tx, c2) = (self.community.tx.clone(), ctx.clone());
        std::thread::spawn(move || {
            let r = current(&s, &tx).and_then(|s| {
                if hearted {
                    request("DELETE", &format!("/rest/v1/hearts?person=eq.{}&item=eq.{}", s.uid, item), Some(&s), None, None, &[], 1 << 16)?.ok().map(|_| s)
                } else {
                    let r = json_post("/rest/v1/hearts", Some(&s), &serde_json::json!({ "person": s.uid, "item": item }), &[("Prefer", "return=minimal")])?;
                    if r.state == 409 { Ok(s) } else { r.ok().map(|_| s) } // already had it: fine
                }
            });
            match r {
                Ok(_) => {}
                Err(e) => {
                    let _ = tx.send(Message::State(e));
                    // the server decides: re-read my data to undo the optimistic change
                    if let Ok(s) = current(&s, &tx) {
                        if let Some((hearts, following)) = load_mine(&s) {
                            let _ = tx.send(Message::Mine { hearts, following });
                        }
                    }
                }
            }
            c2.request_repaint();
        });
    }

    fn community_follow(&mut self, ctx: &egui::Context, creator: String) {
        let Some(s) = self.community.session.clone() else { return };
        if !is_uuid(&creator) || creator == s.uid {
            return;
        }
        let following_creator = self.community.my_following.contains(&creator);
        if following_creator {
            self.community.my_following.remove(&creator);
        } else {
            self.community.my_following.insert(creator.clone());
        }
        if let Some((p, _)) = self.community.viewing.as_mut() {
            if p.id == creator {
                p.followers = (p.followers + if following_creator { -1 } else { 1 }).max(0);
            }
        }
        let (tx, c2) = (self.community.tx.clone(), ctx.clone());
        std::thread::spawn(move || {
            let r = current(&s, &tx).and_then(|s| {
                if following_creator {
                    request("DELETE", &format!("/rest/v1/follows?follower=eq.{}&followed=eq.{}", s.uid, creator), Some(&s), None, None, &[], 1 << 16)?.ok().map(|_| s)
                } else {
                    let r = json_post("/rest/v1/follows", Some(&s), &serde_json::json!({ "follower": s.uid, "followed": creator }), &[("Prefer", "return=minimal")])?;
                    if r.state == 409 { Ok(s) } else { r.ok().map(|_| s) }
                }
            });
            if let Err(e) = r {
                let _ = tx.send(Message::State(e));
                if let Ok(s) = current(&s, &tx) {
                    if let Some((hearts, following)) = load_mine(&s) {
                        let _ = tx.send(Message::Mine { hearts, following });
                    }
                }
            }
            c2.request_repaint();
        });
    }

    /// Floating community windows: the enlarged preview and the creator profile. Called every frame.
    pub fn community_windows(&mut self, ctx: &egui::Context) {
        // Full-size preview: clicking outside, Esc or the button closes it.
        if let Some(path) = self.community.enlarged.clone() {
            let tex = self.community_texture(ctx, &path);
            let screen = ctx.content_rect().size();
            #[allow(deprecated)]
            let id_modal = egui::Id::new("comunidad_vista_ampliada");
            let modal = egui::Modal::new(id_modal).show(ctx, |ui| {
                match &tex {
                    Some(t) => {
                        let s = t.size_vec2();
                        let scale = ((screen.x * 0.86) / s.x).min((screen.y * 0.78) / s.y).min(4.0).max(0.1);
                        ui.add(egui::Image::new(t).fit_to_exact_size(s * scale).corner_radius(6.0));
                    }
                    None => {
                        ui.spinner();
                    }
                }
                ui.add_space(6.0);
                ui.vertical_centered(|ui| {
                    if ui.button(t("Close")).clicked() {
                        self.community.enlarged = None;
                    }
                });
            });
            if modal.should_close() {
                self.community.enlarged = None;
            }
        }
        // Terms of Use and community guidelines (embedded short texts; the full documents open in the browser).
        if let Some(mode) = self.community.terms_open {
            let decision = self.community.terms.as_ref().map(|i| i.decision()).unwrap_or(net::TermsDecision::NotApplicable);
            let outdated = matches!(decision, net::TermsDecision::AppOutdated(_));
            let version = match &decision {
                net::TermsDecision::NeedAccept(v) => Some(v.clone()),
                net::TermsDecision::AppOutdated(_) => None,
                // the server asked for the terms (terms_required) and this build has the text: accept the one it shows
                _ if mode == TermsMode::Accept => Some(net::embedded_terms_version()),
                _ => None,
            };
            let mut close = false;
            let mut accept = false;
            let tabs = [("Terms of Use", net::LEGAL_TERMS, net::DOC_TERMS), ("Community guidelines", net::LEGAL_GUIDELINES, net::DOC_GUIDELINES), ("Privacy", net::LEGAL_PRIVACY, net::DOC_PRIVACY), ("Copyright", net::LEGAL_COPYRIGHT, net::DOC_COPYRIGHT)];
            egui::Window::new("community_terms").title_bar(false).collapsible(false).resizable(false).anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).show(ctx, |ui| {
                ui.set_width(560.0);
                ui.label(RichText::new(t("Terms and community guidelines")).strong());
                ui.horizontal(|ui| {
                    for (i, (n, _, _)) in tabs.iter().enumerate() {
                        if ui.selectable_label(self.community.terms_tab == i, t(n)).clicked() {
                            self.community.terms_tab = i;
                        }
                    }
                });
                let (_, text, doc) = tabs[self.community.terms_tab.min(tabs.len() - 1)];
                egui::ScrollArea::vertical().max_height(280.0).auto_shrink([false, true]).show(ui, |ui| {
                    ui.label(RichText::new(text).small());
                });
                if ui.link(t("Full text on GitHub")).clicked() {
                    open_link(doc);
                }
                ui.separator();
                if outdated {
                    ui.label(RichText::new(t("This app has older legal texts than the server. Update the app to accept the new terms.")).small().color(WARN()));
                } else if mode != TermsMode::Read {
                    ui.checkbox(&mut self.community.terms_check, t("I have read and accept the Terms of Use and the Community guidelines"));
                }
                ui.horizontal(|ui| {
                    if mode != TermsMode::Read && !outdated {
                        let can = self.community.terms_check && (mode == TermsMode::Create || (self.community.busy == 0 && version.is_some()));
                        accept = ui.add_enabled(can, egui::Button::new(t("Accept"))).clicked();
                    }
                    close = ui.button(t("Close")).clicked();
                });
            });
            if accept {
                if mode == TermsMode::Create {
                    self.community.accepted = true;
                    self.community.terms_open = None;
                } else if let (Some(v), Some(s)) = (version, self.community.session.clone()) {
                    self.community_launch(ctx, move |tx| {
                        let r = current(&s, tx).and_then(|s| json_post("/rest/v1/rpc/accept_terms", Some(&s), &net::accept_terms_body(&v), &[])?.ok().map(|_| ()));
                        let _ = tx.send(Message::TermsAccepted(r));
                    });
                }
            } else if close {
                self.community.terms_open = None;
            }
        }
        // My creator profile (editable).
        if self.community.profile_open {
            egui::Window::new("mi_perfil_creador").title_bar(false).collapsible(false).resizable(false).default_width(520.0).anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).show(ctx, |ui| {
                ui.set_width(520.0);
                self.community_edit_profile(ui, ctx);
            });
            self.community.profile_open &= self.community.session.is_some();
        }
    }

    /// Header button: your profile photo (or the default image). Opens your profile; when signed out, opens the Community tab.
    pub fn community_button_profile(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let own = match self.community.my_profile.clone() {
            Some(mi) => self.community_own_image(&ctx, "avatar", &mi),
            None => None,
        };
        let side = egui::Vec2::splat(28.0);
        let r = match &own {
            Some(t) => ui.add(egui::Image::new(t).fit_to_exact_size(side).corner_radius(14.0).sense(egui::Sense::click())),
            None => ui.add(egui::Image::new(egui::include_image!("../assets/brand/avatar.svg")).fit_to_exact_size(side).sense(egui::Sense::click())),
        };
        let border = if r.hovered() { ACCENT() } else { MUTED() };
        ui.painter().circle_stroke(r.rect.center(), 14.0, egui::Stroke::new(1.5, border));
        let tip = match (&self.community.session, &self.community.profile) {
            (Some(_), Some(p)) => format!("{}: {}", t("My creator profile"), p.public_name),
            _ => t("Creator profile: sign in at Community"),
        };
        if r.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(tip).clicked() {
            if self.community.session.is_some() && self.community.profile.is_some() {
                self.community.profile_open = true;
            } else {
                self.tab = crate::app::Tab::Community;
                self.drawer_open = true;
            }
        }
    }

    /// ONLY for documentation screenshots: fakes a signed-in account (no network or real credentials) and loads the
    /// real public catalog. No action in this fake session reaches any server with anyone's data.
    pub fn community_demo(&mut self, ctx: &egui::Context, open_profile: bool, open_upload: bool) {
        let uid = "00000000-0000-4000-8000-000000000000".to_string();
        self.community.started = true;
        self.tab = crate::app::Tab::Community;
        self.drawer_open = true;
        if !open_upload {
            self.community_load_list(ctx); // no session yet: real anonymous catalog
        }
        self.community.session = Some(Session { access: "demo".into(), renewal: "demo".into(), uid: uid.clone(), expires: u64::MAX / 2 });
        self.community.profile = Some(Profile { public_name: "TuNombre".into(), show_18: false });
        self.community.profile_loaded = true;
        let mut links = std::collections::BTreeMap::new();
        links.insert("itch".to_string(), "https://tunombre.itch.io/".to_string());
        self.community.my_profile = Some(ProfilePublic { id: uid, public_name: "TuNombre".into(), bio: "I make game interfaces with Unity.".into(), github: Some("tu-usuario".into()), links, ..Default::default() });
        let f = &mut self.community.profile_form;
        f.name = "TuNombre".into();
        f.bio = "I make game interfaces with Unity.".into();
        f.github = "tu-usuario".into();
        f.links = NETWORKS.iter().map(|(k, _, _)| if *k == "itch" { "https://tunombre.itch.io/".to_string() } else { String::new() }).collect();
        f.loaded = true;
        self.community.profile_open = open_profile;
        if open_upload {
            // "Share" form already open with an example, no catalog underneath
            self.community.open_upload = true;
            self.community.upload.kind = 4;
            self.community.upload.title = "My button menu".into();
            self.community.upload.description = "Four buttons ready for a main menu.".into();
            self.community.upload.labels = "menu, botones".into();
            self.community.upload.file = Some(PathBuf::from("menu_de_botones.json"));
        }
    }

    /// Opens the Community tab filtered by a type (e.g. "theme" from the Themes window).
    pub fn community_goto(&mut self, ctx: &egui::Context, kind: &'static str) {
        self.tab = crate::app::Tab::Community;
        self.drawer_open = true;
        self.community.viewing = None;
        self.community.only_following = false;
        self.community.filter = KIND_NAMES.iter().find(|(k, _)| *k == kind).map(|(k, _)| *k);
        if self.community.started {
            self.community_load_list(ctx);
        }
    }

    /// Prepares the "Share" form with a file (e.g. the current theme from the Themes window).
    pub fn community_prepare_upload(&mut self, ctx: &egui::Context, kind: &'static str, file: PathBuf, title: &str) {
        self.community_goto(ctx, kind);
        if let Some(i) = KIND_NAMES.iter().position(|(k, _)| *k == kind) {
            self.community.upload.kind = i;
        }
        self.community.upload.file = Some(file);
        self.community.upload.title = title.chars().take(80).collect();
        self.community.open_upload = true;
        self.community.state = if self.community.session.is_some() { t("Check the details and press \"Send for review\".") } else { t("Sign in to share.") };
    }

    fn community_open_profile(&mut self, ctx: &egui::Context, uid: String) {
        let s = self.community.session.clone();
        self.community_launch(ctx, move |tx| {
            let s = s.and_then(|s| current(&s, tx).ok());
            let _ = tx.send(Message::ProfileView(load_public_profile(s.as_ref(), &uid)));
        });
    }

    fn community_card(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, it: &Item, signed_in: bool) {
        egui::Frame::new().fill(crate::theme::PANEL_DARK()).corner_radius(6.0).inner_margin(8.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                // Content type icon, drawn as SVG (uploaded images are only used as previews).
                let icon_name = match it.kind.as_str() {
                    "plugin" => "tab_plugins",
                    "image" => "tab_images",
                    "font" => "kind_font",
                    "template" => "tab_templates",
                    "element" => "tab_elements",
                    _ => "kind_theme",
                };
                let (r, _) = ui.allocate_exact_size(egui::Vec2::splat(40.0), egui::Sense::hover());
                ui.painter().rect_filled(r, 8.0, crate::theme::ACCENT_SOFT());
                crate::icons::paint_or(ui, icon_name, r.shrink(9.0), ACCENT(), "");
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&it.title).strong());
                        if it.is_18 {
                            ui.label(RichText::new("+18").small().color(WARN()));
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 3.0;
                        ui.label(RichText::new(format!("{} · {}", t(kind_name(&it.kind)), t("by"))).small().color(MUTED()));
                        let name = if it.author_name.is_empty() { "—" } else { it.author_name.as_str() };
                        if ui.link(RichText::new(name).small()).on_hover_text(t("View the creator's profile")).clicked() {
                            self.community_open_profile(ctx, it.author.clone());
                        }
                        crate::icons::with_text(ui, "download", RichText::new(it.downloads.to_string()).small().color(MUTED()), MUTED());
                    });
                });
            });
            if !it.description.is_empty() {
                ui.label(RichText::new(&it.description).small());
            }
            let license = it.license.as_deref().and_then(net::license_label);
            let source = it.source_url.as_deref().filter(|u| net::valid_source_url(u));
            if license.is_some() || source.is_some() {
                ui.horizontal_wrapped(|ui| {
                    if let Some(l) = license {
                        ui.label(RichText::new(format!("{}: {}", t("License"), t(l))).small().color(MUTED()));
                    }
                    if let Some(u) = source {
                        // plain text; it opens only on click, and only http/https
                        let shown = net::source_display(u); // the complete host, only the path is shortened
                        ui.label(RichText::new(format!("{}:", t("Source"))).small().color(MUTED()));
                        if ui.link(RichText::new(shown).small()).on_hover_text(u).clicked() {
                            open_source(u);
                        }
                    }
                });
            }
            if !it.tags.is_empty() {
                ui.label(RichText::new(it.tags.iter().map(|e| format!("#{}", e)).collect::<Vec<_>>().join(" ")).small().color(MUTED()));
            }
            if let Some(r) = it.public_preview.clone() {
                egui::CollapsingHeader::new(RichText::new(t("Preview")).small()).id_salt(("preview", &it.id)).show(ui, |ui| match self.community_texture(ctx, &r) {
                    Some(tex) => {
                        let w = ui.available_width();
                        let s = tex.size_vec2();
                        let img = ui.add(egui::Image::new(&tex).fit_to_exact_size(egui::Vec2::new(w, w * s.y / s.x.max(1.0))).corner_radius(4.0).sense(egui::Sense::click()));
                        if img.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(t("Click to enlarge")).clicked() {
                            self.community.enlarged = Some(r.clone());
                        }
                    }
                    None => {
                        ui.spinner();
                    }
                });
            }
            ui.horizontal(|ui| {
                if ui.add_enabled(self.community.busy == 0, egui::Button::new(RichText::new(t("Install")).color(ACCENT()))).clicked() {
                    self.community.state = t("Downloading and verifying…");
                    let item = it.clone();
                    let session = self.community.session.clone();
                    self.community_launch(ctx, move |tx| {
                        let session = session.and_then(|s| current(&s, tx).ok());
                        let bytes = download(&item, session.as_ref());
                        let _ = tx.send(Message::Downloaded { item, bytes });
                    });
                }
                let hearted = self.community.my_hearts.contains(&it.id);
                let color = if hearted { WARN() } else { MUTED() };
                let heart = RichText::new(it.hearts.max(0).to_string()).color(color);
                let r = ui.add_enabled(signed_in, crate::icons::btn_tint(if hearted { "heart_fill" } else { "heart" }, heart, color).small());
                if r.on_hover_text(if signed_in { t("Give or remove a heart") } else { t("Sign in to give hearts and follow creators.") }).clicked() {
                    self.community_heart(ctx, it.id.clone());
                }
                if signed_in && ui.small_button(t("Report")).clicked() {
                    self.community.report = Some(it.id.clone());
                }
            });
            if self.community.report.as_deref() == Some(it.id.as_str()) {
                ui.label(RichText::new(t("What is wrong with it?")).small());
                ui.horizontal_wrapped(|ui| {
                    for (k, n) in net::REPORT_REASONS {
                        if !ui.small_button(t(n)).clicked() {
                            continue;
                        }
                        self.community.report = None;
                        if *k == "copyright" {
                            // Report > Copyright: the claim form (it hides the item at once)
                            self.community.claim = Some(ClaimForm { item: it.id.clone(), ..Default::default() });
                            continue;
                        }
                        let (s, id, reason) = (self.community.session.clone().unwrap(), it.id.clone(), k.to_string());
                        self.community_launch(ctx, move |tx| {
                            let r = current(&s, tx).and_then(|s| {
                                let prefer = [("Prefer", "return=minimal")];
                                let mut resp = json_post("/rest/v1/reports", Some(&s), &net::report_body(&id, &s.uid, &reason, false), &prefer)?;
                                if net::is_reason_rejected(resp.state, &resp.text()) {
                                    // a server without migration 0015 only knows the old spellings
                                    resp = json_post("/rest/v1/reports", Some(&s), &net::report_body(&id, &s.uid, &reason, true), &prefer)?;
                                }
                                resp.ok().map(|_| ())
                            });
                            let _ = tx.send(Message::Reported(r));
                        });
                    }
                });
                if ui.small_button(t("Cancel")).clicked() {
                    self.community.report = None;
                }
            }
            if self.community.claim.as_ref().is_some_and(|c| c.item == it.id) {
                let (busy, mut send, mut cancel) = (self.community.busy, false, false);
                ui.label(RichText::new(t("Report copyright infringement")).small().strong());
                if let Some(c) = self.community.claim.as_mut() {
                    ui.add(egui::TextEdit::multiline(&mut c.description).hint_text(t("The work and why it is infringed (10-2000 characters)")).desired_rows(3).desired_width(f32::INFINITY));
                    ui.add(egui::TextEdit::singleline(&mut c.claimant).hint_text(t("Rights holder (optional)")).desired_width(f32::INFINITY));
                    ui.add(egui::TextEdit::singleline(&mut c.contact).hint_text(t("Contact (optional)")).desired_width(f32::INFINITY));
                    let ok = net::claim_problem(&c.description, &c.claimant, &c.contact).is_none();
                    ui.horizontal(|ui| {
                        send = ui.add_enabled(ok && busy == 0, egui::Button::new(t("Send claim"))).clicked();
                        cancel = ui.small_button(t("Cancel")).clicked();
                        if ui.link(RichText::new(t("Full text on GitHub")).small()).clicked() {
                            open_link(net::DOC_COPYRIGHT);
                        }
                    });
                }
                if cancel {
                    self.community.claim = None;
                } else if send {
                    if let (Some(c), Some(s)) = (self.community.claim.take(), self.community.session.clone()) {
                        self.community_launch(ctx, move |tx| {
                            let r = current(&s, tx).and_then(|s| json_post("/rest/v1/rpc/file_copyright_claim", Some(&s), &net::claim_body(&c.item, &c.description, &c.claimant, &c.contact), &[])?.ok().map(|_| ()));
                            let _ = tx.send(Message::Claimed(r));
                        });
                    }
                }
            }
        });
    }

    fn community_view_profile(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let Some((p, items)) = self.community.viewing.clone() else { return };
        if ui.add(crate::icons::btn("arrow_left", t("Back to the community"))).clicked() {
            self.community.viewing = None;
            return;
        }
        ui.add_space(4.0);
        let is_mine = self.community.session.as_ref().map(|s| s.uid == p.id).unwrap_or(false);
        let mi = self.community.my_profile.clone().filter(|_| is_mine);
        let banner = match &mi {
            Some(m) => self.community_own_image(ctx, "banner", m),
            None => p.public_banner.as_deref().and_then(|r| self.community_texture(ctx, r)),
        };
        let avatar = match &mi {
            Some(m) => self.community_own_image(ctx, "avatar", m),
            None => p.public_avatar.as_deref().and_then(|r| self.community_texture(ctx, r)),
        };
        egui::Frame::new().fill(crate::theme::PANEL_DARK()).corner_radius(6.0).inner_margin(8.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            if let Some(b) = &banner {
                let w = ui.available_width();
                let s = b.size_vec2();
                ui.add(egui::Image::new(b).fit_to_exact_size(egui::Vec2::new(w, w * s.y / s.x.max(1.0))).corner_radius(4.0));
            }
            ui.horizontal(|ui| {
                match &avatar {
                    Some(a) => ui.add(egui::Image::new(a).fit_to_exact_size(egui::Vec2::splat(56.0)).corner_radius(28.0)),
                    None => ui.add(egui::Image::new(egui::include_image!("../assets/brand/avatar.svg")).fit_to_exact_size(egui::Vec2::splat(56.0))),
                };
                ui.vertical(|ui| {
                    ui.label(RichText::new(&p.public_name).strong().size(16.0));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{} {} · {} {} ·", items.len(), t("published items"), p.followers.max(0), t("followers"))).small().color(MUTED()));
                        crate::icons::with_text(ui, "heart_fill", RichText::new(p.hearts_received.max(0).to_string()).small().color(MUTED()), WARN());
                    });
                    let own = self.community.session.as_ref().map(|s| s.uid == p.id).unwrap_or(false);
                    if !own {
                        let following_creator = self.community.my_following.contains(&p.id);
                        let label = if following_creator { t("Following") } else { t("Follow") };
                        let r = ui.add_enabled(self.community.session.is_some() && self.community.profile.is_some(), crate::icons::btn(if following_creator { "check" } else { "plus" }, label).small());
                        if r.on_hover_text(if following_creator { t("Unfollow") } else { t("Sign in to give hearts and follow creators.") }).clicked() {
                            self.community_follow(ctx, p.id.clone());
                        }
                    }
                });
            });
            if !p.bio.is_empty() {
                ui.label(RichText::new(&p.bio).small());
            }
            ui.horizontal_wrapped(|ui| {
                if let Some(g) = p.github.as_deref().filter(|g| github_valid(g)) {
                    if ui.small_button(format!("GitHub · {}", g)).clicked() {
                        open_link(&format!("https://github.com/{}", g));
                    }
                }
                for (k, name, _) in NETWORKS {
                    if let Some(url) = p.links.get(*k).filter(|u| valid_link(k, u)) {
                        if ui.small_button(*name).on_hover_text(url.as_str()).clicked() {
                            open_link(url);
                        }
                    }
                }
            });
        });
        ui.add_space(6.0);
        let signed_in = self.community.session.is_some() && self.community.profile.is_some();
        for it in &items {
            self.community_card(ui, ctx, it, signed_in);
            ui.add_space(4.0);
        }
    }

    /// "My creator profile" editor, X-style: banner with overlaid photo, labeled fields and "Save" at the top.
    fn community_edit_profile(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let Some(s) = self.community.session.clone() else { return };
        let mi = self.community.my_profile.clone().unwrap_or_default();
        if !self.community.profile_form.loaded {
            if let Some(p) = &self.community.profile {
                self.community.profile_form.name = p.public_name.clone();
            }
        }
        // ── Top bar: close · title · Save ──
        let mut save = false;
        ui.horizontal(|ui| {
            if ui.add(crate::icons::btn("close", "").frame(false)).on_hover_text(t("Close")).clicked() {
                self.community.profile_open = false;
            }
            ui.add_space(10.0);
            ui.label(RichText::new(t("Edit profile")).strong().size(19.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let b = egui::Button::new(RichText::new(t("Save")).strong().color(crate::theme::PANEL())).fill(crate::theme::TEXT()).corner_radius(16.0).min_size(egui::vec2(86.0, 32.0));
                if ui.add_enabled(self.community.busy == 0, b).clicked() {
                    save = true;
                }
            });
        });
        if !self.community.state.is_empty() {
            ui.label(RichText::new(t(&self.community.state)).small().color(ACCENT()));
        }
        ui.add_space(4.0);

        egui::ScrollArea::vertical().max_height(ctx.content_rect().height() * 0.72).show(ui, |ui| {
            // ── Banner with its buttons ──
            let width = ui.available_width();
            let (rb, _) = ui.allocate_exact_size(egui::vec2(width, (width / 3.0).round()), egui::Sense::hover());
            let banner = self.community_own_image(ctx, "banner", &mi);
            match &banner {
                Some(tex) => {
                    egui::Image::new(tex).corner_radius(6.0).paint_at(ui, rb);
                }
                None => {
                    ui.painter().rect_filled(rb, 6.0, crate::theme::ACCENT_SOFT());
                }
            }
            let has_banner = banner.is_some();
            let mut action: Option<(&'static str, bool)> = None; // (class, true = choose / false = remove)
            let round = |icon: &str| crate::icons::round(icon, 44.0, egui::Color32::WHITE);
            let c = rb.center();
            let r1 = egui::Rect::from_center_size(c - egui::vec2(if has_banner { 28.0 } else { 0.0 }, 0.0), egui::vec2(44.0, 44.0));
            if ui.put(r1, round("camera")).on_hover_text(t("Change the banner")).clicked() {
                action = Some(("banner", true));
            }
            if has_banner {
                let r2 = egui::Rect::from_center_size(c + egui::vec2(28.0, 0.0), egui::vec2(44.0, 44.0));
                if ui.put(r2, round("close")).on_hover_text(t("Remove the banner")).clicked() {
                    action = Some(("banner", false));
                }
            }
            // ── Overlaid profile photo ──
            let ra = egui::Rect::from_center_size(egui::pos2(rb.left() + 70.0, rb.bottom()), egui::Vec2::splat(108.0));
            ui.painter().circle_filled(ra.center(), 58.0, crate::theme::PANEL());
            match self.community_own_image(ctx, "avatar", &mi) {
                Some(tex) => egui::Image::new(&tex).corner_radius(54.0).paint_at(ui, ra),
                None => egui::Image::new(egui::include_image!("../assets/brand/avatar.svg")).paint_at(ui, ra),
            }
            if ui.put(egui::Rect::from_center_size(ra.center(), egui::vec2(44.0, 44.0)), round("camera")).on_hover_text(t("Change the profile picture")).clicked() {
                action = Some(("avatar", true));
            }
            ui.add_space(62.0);
            let has_avatar = mi.public_avatar.is_some() || mi.avatar_pending || self.community.previous.contains_key("avatar");
            if has_avatar && ui.small_button(t("Remove the profile picture")).clicked() {
                action = Some(("avatar", false));
            }
            // ── Review notice (like X's) ──
            let pending: Vec<String> = [("avatar", mi.avatar_pending, t("the profile picture")), ("banner", mi.banner_pending, t("the banner"))]
                .into_iter()
                .filter(|(_, p, _)| *p)
                .map(|(_, _, n)| n)
                .collect();
            if !pending.is_empty() {
                ui.add_space(6.0);
                egui::Frame::new().fill(crate::theme::ACCENT_SOFT()).corner_radius(6.0).inner_margin(10.0).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(format!("{} {}. {}", t("Under review:"), pending.join(&format!(" {} ", t("and"))), t("You already see it here; others will once a person approves it."))).color(crate::theme::TEXT()));
                });
            }
            ui.add_space(10.0);
            // ── Fields ──
            let f = &mut self.community.profile_form;
            if f.links.len() != NETWORKS.len() {
                f.links.resize(NETWORKS.len(), String::new());
            }
            let mut errors: Vec<String> = Vec::new();
            field_x(ui, &t("Name"), &mut f.name, 1, "public_name");
            let name = f.name.trim().to_string();
            if !((3..=32).contains(&name.chars().count()) && name.chars().all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c))) {
                errors.push(t("Name: 3-32 characters (letters, numbers, dot, hyphen and underscore)."));
            }
            field_x(ui, &format!("{} · {}/300", t("Bio"), f.bio.chars().count()), &mut f.bio, 3, &t("Tell the community who you are (no links)"));
            if f.bio.chars().count() > 300 {
                errors.push(t("The description allows up to 300 characters."));
            }
            field_x(ui, "GitHub", &mut f.github, 1, &t("GitHub username"));
            let gh = f.github.trim().trim_start_matches('@').to_string();
            if !gh.is_empty() && !github_valid(&gh) {
                errors.push(t("Invalid GitHub username."));
            }
            for (i, (k, network_name, domains)) in NETWORKS.iter().enumerate() {
                field_x(ui, network_name, &mut f.links[i], 1, &format!("https://{}/…", domains[0]));
                let u = f.links[i].trim();
                if !u.is_empty() && !valid_link(k, u) {
                    errors.push(format!("{}: {} https://{}/…", network_name, t("must start with"), domains[0]));
                }
            }
            for e in &errors {
                ui.label(RichText::new(e).small().color(WARN()));
            }
            ui.add_space(6.0);
            // ── "View my public profile ›" row ──
            let row = ui.add(crate::icons::btn("user", RichText::new(t("View my public profile")).size(15.0)).frame(false));
            if row.clicked() {
                self.community.profile_open = false;
                self.tab = crate::app::Tab::Community;
                self.drawer_open = true;
                self.community_open_profile(ctx, s.uid.clone());
            }
            ui.label(RichText::new(t("Your profile is public once you have at least one approved item. Images and description are moderated.")).small().color(MUTED()));

            // ── Actions ──
            match action {
                Some((class, true)) => self.community_choose_profile_image(ctx, class),
                Some((class, false)) => {
                    self.community.previous.remove(class);
                    let s = s.clone();
                    self.community_launch(ctx, move |tx| {
                        let r = current(&s, tx).and_then(|s| json_post("/functions/v1/profile", Some(&s), &serde_json::json!({ "v": 2, "action": "remove", "class": class }), &[])?.ok().map(|_| s));
                        if let Ok(s) = &r {
                            let _ = tx.send(Message::MyProfile(load_my_profile(s)));
                        }
                        let _ = tx.send(Message::State(r.map(|_| t("Image removed.")).unwrap_or_else(|e| e)));
                    });
                }
                None => {}
            }
            if save {
                if !errors.is_empty() {
                    self.community.state = errors[0].clone();
                } else {
                    let f = &self.community.profile_form;
                    let new_name = (Some(name.clone()) != self.community.profile.as_ref().map(|p| p.public_name.clone())).then_some(name.clone());
                    let links: serde_json::Map<String, serde_json::Value> = NETWORKS.iter().zip(&f.links).filter(|(_, u)| !u.trim().is_empty()).map(|((k, _, _), u)| (k.to_string(), serde_json::Value::String(u.trim().to_string()))).collect();
                    let body = serde_json::json!({ "v": 2, "action": "save", "bio": f.bio.trim(), "github": gh, "links": links });
                    let s = s.clone();
                    self.community.state = t("Saving…");
                    self.community_launch(ctx, move |tx| {
                        let r = current(&s, tx).and_then(|s| {
                            if let Some(n) = &new_name {
                                request("PATCH", &format!("/rest/v1/profiles?id=eq.{}", s.uid), Some(&s), None, Some(serde_json::json!({ "public_name": n }).to_string().as_bytes()), &[], 1 << 16)?.ok()?;
                            }
                            let r = json_post("/functions/v1/profile", Some(&s), &body, &[])?;
                            if (200..300).contains(&r.state) {
                                Ok(s)
                            } else {
                                Err(r.json::<serde_json::Value>().ok().and_then(|v| v["error"].as_str().map(|e| e.to_string())).unwrap_or_else(|| error_message(r.state, "")))
                            }
                        });
                        if let Ok(s) = &r {
                            let _ = tx.send(Message::MyProfile(load_my_profile(s)));
                        }
                        let _ = tx.send(Message::State(r.map(|_| t("Profile saved.")).unwrap_or_else(|e| e)));
                    });
                }
            }
        });
    }

    /// My profile image as I see it: the one I just chose, the one under review (my quarantine) or the published one.
    fn community_own_image(&mut self, ctx: &egui::Context, class: &'static str, mi: &ProfilePublic) -> Option<egui::TextureHandle> {
        if let Some(t) = self.community.previous.get(class) {
            return Some(t.clone());
        }
        let pending = if class == "avatar" { mi.avatar_pending } else { mi.banner_pending };
        let public = if class == "avatar" { mi.public_avatar.clone() } else { mi.public_banner.clone() };
        if pending {
            return self.community_quarantine_texture(ctx, class);
        }
        public.and_then(|r| self.community_texture(ctx, &r))
    }

    /// My profile image under review: only its owner can read it (quarantine is private).
    fn community_quarantine_texture(&mut self, ctx: &egui::Context, class: &'static str) -> Option<egui::TextureHandle> {
        let key = format!("quarantine:{}", class);
        if let Some(Some(t)) = self.community.textures.get(&key) {
            return Some(t.clone());
        }
        let s = self.community.session.clone()?;
        if !self.community.tex_gate.may_start(&key, std::time::Instant::now()) {
            return None;
        }
        let (tx, c2) = (self.community.tx.clone(), ctx.clone());
        std::thread::spawn(move || {
            let bytes = current(&s, &tx)
                .and_then(|s| request("GET", &format!("/storage/v1/object/authenticated/cuarentena/{}/perfil/{}.png", s.uid, class), Some(&s), None, None, &[], (2 << 20) + 1)?.ok())
                .ok()
                .map(|r| r.body)
                .filter(|b| val::detect(b) == Some("png"));
            let _ = tx.send(Message::Image { path: key, bytes });
            c2.request_repaint();
        });
        None
    }

    /// Picks a photo or banner (PNG, JPEG, WebP, GIF or BMP of any size): the app crops and shrinks it,
    /// shows it immediately and uploads it for review.
    fn community_choose_profile_image(&mut self, ctx: &egui::Context, class: &'static str) {
        let Some(f) = rfd::FileDialog::new().add_filter(t("Images"), &["png", "jpg", "jpeg", "webp", "gif", "bmp"]).pick_file() else { return };
        let ready = std::fs::read(&f).map_err(|e| e.to_string()).and_then(|b| prepare_profile_image(class, &b));
        let bytes = match ready {
            Ok(b) => b,
            Err(e) => {
                self.community.state = e;
                return;
            }
        };
        if let Ok(img) = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png) {
            let img = img.to_rgba8();
            let (w, h) = img.dimensions();
            let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], img.as_raw());
            self.community.previous.insert(class, ctx.load_texture(format!("previa/{}", class), ci, egui::TextureOptions::LINEAR));
        }
        let Some(s) = self.community.session.clone() else { return };
        self.community.state = t("Uploading and analysing the image… (can take up to a minute)");
        self.community_launch(ctx, move |tx| {
            let r = current(&s, tx).and_then(|s| upload_profile_image(&s, class, &bytes).map(|m| (s, m)));
            match r {
                Ok((s, m)) => {
                    let _ = tx.send(Message::MyProfile(load_my_profile(&s)));
                    let _ = tx.send(Message::State(m));
                }
                Err(e) => {
                    let _ = tx.send(Message::State(e));
                }
            }
        });
    }
}

#[cfg(test)]
mod profile_tests {
    use super::*;

    #[test]
    fn network_links() {
        assert!(valid_link("x", "https://x.com/deritu"));
        assert!(valid_link("itch", "https://deritu.itch.io"));
        assert!(valid_link("youtube", "https://www.youtube.com/@deritu"));
        assert!(!valid_link("x", "http://x.com/deritu"));
        assert!(!valid_link("x", "https://x.com.malo.example/deritu"));
        assert!(!valid_link("x", "https://usuario:clave@x.com/"));
        assert!(!valid_link("x", "https://x.com:8080/"));
        assert!(!valid_link("youtube", "javascript:alert(1)"));
        assert!(!valid_link("discord", "https://discord.gg/x"));
        assert!(!valid_link("itch", "https://.itch.io"));
    }

    #[test]
    fn github_users_and_paths() {
        assert!(github_valid("deritu-kutsut"));
        assert!(!github_valid("-malo"));
        assert!(!github_valid("con espacio"));
        assert!(valid_public_path("plugin/abc-123/icono.png"));
        assert!(!valid_public_path("../secreto.png"));
        assert!(!valid_public_path("plugin//x.png"));
    }
}

/// Gives or removes a heart on a content item, or follows / unfollows a creator, using the saved session.
///   lienzo-ui --community-heart <id-of-content>
///   lienzo-ui --community-follow <id-of-creator>
pub fn social_cli(what: &str, id: &str) -> i32 {
    if !is_uuid(id) {
        eprintln!("Provide a valid identifier.");
        return 2;
    }
    let s = match saved_session() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let Some((hearts, following)) = load_mine(&s) else {
        eprintln!("Could not read your data (do you have a profile?).");
        return 1;
    };
    let (path_get, path_post, field, row, ya) = if what == "heart" {
        (format!("/rest/v1/hearts?person=eq.{}&item=eq.{}", s.uid, id), "/rest/v1/hearts", "item", serde_json::json!({ "person": s.uid, "item": id }), hearts.iter().any(|i| i == id))
    } else {
        (format!("/rest/v1/follows?follower=eq.{}&followed=eq.{}", s.uid, id), "/rest/v1/follows", "followed", serde_json::json!({ "follower": s.uid, "followed": id }), following.iter().any(|i| i == id))
    };
    let r = if ya { request("DELETE", &path_get, Some(&s), None, None, &[], 1 << 16) } else { json_post(path_post, Some(&s), &row, &[("Prefer", "return=minimal")]) };
    match r.and_then(|r| r.ok()) {
        Ok(_) => println!("{} {}: {}", what, field, if ya { "removed" } else { "added" }),
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    }
    let total = if what == "heart" { request("GET", &format!("/rest/v1/community_items?select=hearts&id=eq.{}", id), None, None, None, &[], 1 << 16) } else { request("GET", &format!("/rest/v1/public_profiles?select=followers&id=eq.{}", id), None, None, None, &[], 1 << 16) };
    if let Ok(t) = total.and_then(|r| r.ok()) {
        println!("total now: {}", val::terminal_safe(&t.text()));
    }
    0
}

#[cfg(test)]
mod image_tests {
    use super::*;

    fn jpeg(w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbImage::from_fn(w, h, |x, y| image::Rgb([(x % 256) as u8, (y % 256) as u8, 128]));
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(img).write_to(&mut out, image::ImageFormat::Jpeg).unwrap();
        out.into_inner()
    }

    #[test]
    fn large_photo_is_cropped_and_reduced() {
        let a = prepare_profile_image("avatar", &jpeg(1600, 900)).unwrap();
        assert!(val::validate_png("avatar", &a).is_ok());
        let b = prepare_profile_image("banner", &jpeg(1000, 1000)).unwrap();
        let (w, h) = val::validate_png("banner", &b).unwrap();
        assert_eq!(w, h * 3, "the banner ends up 3:1");
    }

    #[test]
    fn executable_does_not_pass_as_image() {
        let mut exe = vec![0u8; 64];
        exe[..2].copy_from_slice(b"MZ");
        assert!(prepare_profile_image("avatar", &exe).is_err());
    }
}

#[cfg(test)]
mod install_tests {
    use super::*;

    #[test]
    fn folder_names_are_unambiguous_and_windows_safe() {
        let names = ["A b", "A_b", "abc.", "abc", "ABC", "a b", "a  b", "日本語", "", "con", "x/y", "x\\y"];
        let folders: Vec<String> = names.iter().map(|n| plugin_folder_name(n)).collect();
        let unique: std::collections::HashSet<&String> = folders.iter().collect();
        assert_eq!(unique.len(), names.len(), "{:?}", folders);
        for f in &folders {
            assert!(f.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') && !f.ends_with('-') && f.starts_with("community-"), "{}", f);
        }
    }

    #[test]
    fn plugins_whose_names_only_differ_by_case_or_symbols_never_share_a_folder() {
        let base = std::env::temp_dir().join(format!("lienzo-collide-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let names = ["Tools", "tools", "TOOLS", "Tools.", "Tools ", "T ools", "T_ools", "Tools\u{200b}"];
        for (i, n) in names.iter().enumerate() {
            let manifest = serde_json::json!({"name": n, "commands":[{"name":"x","script":"a.rhai"}]});
            let pkg = Package { manifest, files: [("a.rhai".to_string(), format!("fn run(d){{ d }} // plugin {}", i))].into() };
            install_plugin_into(&pkg, &base, &base.join(plugin_folder_name(n))).unwrap();
        }
        for (i, n) in names.iter().enumerate() {
            let code = std::fs::read_to_string(base.join(plugin_folder_name(n)).join("a.rhai")).unwrap();
            assert!(code.ends_with(&format!("// plugin {}", i)), "{:?} was overwritten by another plugin: {}", n, code);
        }
        assert_eq!(std::fs::read_dir(&base).unwrap().count(), names.len());
        // a package can never overwrite the validated manifest through a differently cased plugin.json
        let manifest = serde_json::json!({"name":"Guard","commands":[{"name":"x","script":"a.rhai"}]});
        let sneaky = Package { manifest: manifest.clone(), files: [("a.rhai".to_string(), "fn run(d){ d }".to_string()), ("Plugin.JSON".to_string(), "{\"name\":\"Evil\"}".to_string())].into() };
        let folder = base.join(plugin_folder_name("Guard"));
        install_plugin_into(&sneaky, &base, &folder).unwrap();
        let written: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(folder.join("plugin.json")).unwrap()).unwrap();
        assert_eq!(written["name"], "Guard");
        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn install_swaps_atomically_and_failure_leaves_the_old_version() {
        let base = std::env::temp_dir().join(format!("lienzo-install-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let manifest = serde_json::json!({"name":"Inst","commands":[{"name":"x","script":"a.rhai"}]});
        let mk = |code: &str| Package { manifest: manifest.clone(), files: [("a.rhai".to_string(), code.to_string())].into() };
        let folder = base.join(plugin_folder_name("Inst"));
        install_plugin_into(&mk("fn run(d){ d } // v1"), &base, &folder).unwrap();
        install_plugin_into(&mk("fn run(d){ d } // v2"), &base, &folder).unwrap();
        assert!(std::fs::read_to_string(folder.join("a.rhai")).unwrap().contains("v2"));
        assert!(folder.join("plugin.json").exists());
        // a file that cannot be written (a file is in the way of a folder) fails without touching v2 and leaves no temp folder
        let bad = Package { manifest: manifest.clone(), files: [("a.rhai".to_string(), "x".to_string()), ("a.rhai/inner.rhai".to_string(), "y".to_string())].into() };
        assert!(install_plugin_into(&bad, &base, &folder).is_err());
        assert!(std::fs::read_to_string(folder.join("a.rhai")).unwrap().contains("v2"));
        let leftovers: Vec<_> = std::fs::read_dir(&base).unwrap().filter_map(|e| e.ok()).filter(|e| e.file_name().to_string_lossy().starts_with('.')).collect();
        assert!(leftovers.is_empty(), "{:?}", leftovers.iter().map(|e| e.file_name()).collect::<Vec<_>>());
        let _ = std::fs::remove_dir_all(&base);
    }
}

/// Tests of the network layer against a fake local HTTP server (std `TcpListener`): timeouts, retries, 429 / Retry-After and
/// the offline cache fallback. Nothing here touches the real server.
#[cfg(test)]
mod net_tests {
    use super::*;
    use std::net::TcpListener;
    use std::sync::atomic::AtomicUsize;
    use std::sync::Mutex;

    enum Act {
        Reply(u16, Vec<(&'static str, String)>, String),
        /// Reads the request and then does not answer for this many milliseconds.
        Hang(u64),
    }

    struct Fake {
        base: String,
        hits: Arc<AtomicUsize>,
        methods: Arc<Mutex<Vec<String>>>,
    }

    impl Fake {
        fn hits(&self) -> usize {
            self.hits.load(Ordering::SeqCst)
        }
    }

    fn read_request(c: &mut std::net::TcpStream) -> Option<String> {
        let _ = c.set_read_timeout(Some(Duration::from_secs(3)));
        let mut buf = Vec::new();
        let mut chunk = [0u8; 4096];
        loop {
            let n = c.read(&mut chunk).ok()?;
            if n == 0 {
                return None;
            }
            buf.extend_from_slice(&chunk[..n]);
            if let Some(end) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
                let head = String::from_utf8_lossy(&buf[..end]).to_string();
                let len = head.lines().find_map(|l| l.to_ascii_lowercase().strip_prefix("content-length:").and_then(|v| v.trim().parse::<usize>().ok())).unwrap_or(0);
                while buf.len() < end + 4 + len {
                    let n = c.read(&mut chunk).ok()?;
                    if n == 0 {
                        break;
                    }
                    buf.extend_from_slice(&chunk[..n]);
                }
                return Some(head);
            }
        }
    }

    /// Answers one scripted action per connection; once the script is over every connection is closed without an answer.
    fn fake(script: Vec<Act>) -> Fake {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://127.0.0.1:{}", l.local_addr().unwrap().port());
        let (hits, methods) = (Arc::new(AtomicUsize::new(0)), Arc::new(Mutex::new(Vec::new())));
        let (h2, m2) = (hits.clone(), methods.clone());
        std::thread::spawn(move || {
            let mut script = script.into_iter();
            for conn in l.incoming() {
                let Ok(mut c) = conn else { break };
                h2.fetch_add(1, Ordering::SeqCst);
                let act = script.next();
                let m3 = m2.clone();
                std::thread::spawn(move || {
                    let Some(head) = read_request(&mut c) else { return };
                    m3.lock().unwrap().push(head.split(' ').next().unwrap_or("").to_string());
                    match act {
                        Some(Act::Reply(status, headers, body)) => {
                            let extra: String = headers.iter().map(|(k, v)| format!("{}: {}\r\n", k, v)).collect();
                            let _ = write!(c, "HTTP/1.1 {} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}", status, body.len(), extra, body);
                        }
                        Some(Act::Hang(ms)) => std::thread::sleep(Duration::from_millis(ms)),
                        None => {}
                    }
                });
            }
        });
        Fake { base, hits, methods }
    }

    fn reply(status: u16, body: &str) -> Act {
        Act::Reply(status, vec![], body.to_string())
    }

    fn fast() -> NetCfg {
        NetCfg { connect: Duration::from_millis(500), read: Duration::from_millis(500), write: Duration::from_millis(500), download: Duration::from_millis(500), retries: 2, backoff: Duration::from_millis(10), max_retry_wait: Duration::from_secs(2) }
    }

    fn get(cfg: &NetCfg, base: &str) -> Result<Resp, net::Failure> {
        try_request(cfg, base, "GET", "/rest/v1/x", None, None, None, &[], 1 << 20)
    }

    const UID: &str = "0b9f3c3e-6f41-4f0e-9a52-1d8f6b2e7a10";

    fn items_json() -> String {
        format!(r#"[{{"id":"{0}","kind":"image","title":"Cat","author":"{0}"}}]"#, UID)
    }

    fn tmp(n: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("lienzo_cnet_{}_{}", n, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// An address nothing listens on (the port was just released).
    fn closed_port() -> String {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let p = l.local_addr().unwrap().port();
        drop(l);
        format!("http://127.0.0.1:{}", p)
    }

    #[test]
    fn a_silent_server_times_out_instead_of_hanging() {
        let f = fake(vec![Act::Hang(4000)]);
        let cfg = NetCfg { retries: 0, read: Duration::from_millis(400), ..fast() };
        let t0 = std::time::Instant::now();
        let e = get(&cfg, &f.base).err().expect("must fail");
        assert!(t0.elapsed() < Duration::from_secs(3), "{:?}", t0.elapsed());
        assert!(e.transient, "a timeout is transient");
        assert_eq!(f.hits(), 1);
    }

    #[test]
    fn get_is_retried_with_backoff_until_it_works() {
        let f = fake(vec![reply(503, ""), reply(502, ""), reply(200, "[]")]);
        let r = get(&fast(), &f.base).ok().expect("third attempt works");
        assert_eq!(r.state, 200);
        assert_eq!(f.hits(), 3);
        // a closed connection (no answer) is retried too
        let f = fake(vec![Act::Hang(0), reply(200, "[]")]);
        assert_eq!(get(&fast(), &f.base).ok().map(|r| r.state), Some(200));
        assert_eq!(f.hits(), 2);
    }

    #[test]
    fn retries_are_bounded_and_plain_errors_are_not_retried() {
        let f = fake(vec![reply(503, ""), reply(503, ""), reply(503, ""), reply(503, ""), reply(503, "")]);
        let r = get(&fast(), &f.base).ok().unwrap();
        assert_eq!(r.state, 503);
        assert_eq!(f.hits(), 3, "1 try + 2 retries");
        let f = fake(vec![reply(404, "{}"), reply(200, "[]")]);
        assert_eq!(get(&fast(), &f.base).ok().unwrap().state, 404);
        assert_eq!(f.hits(), 1);
        let f = fake(vec![reply(500, "{}"), reply(200, "[]")]);
        assert_eq!(get(&fast(), &f.base).ok().unwrap().state, 500);
        assert_eq!(f.hits(), 1, "a plain 500 is not retried");
    }

    #[test]
    fn writes_are_never_retried() {
        for method in ["POST", "PATCH", "DELETE"] {
            let f = fake(vec![reply(503, ""), reply(200, "{}")]);
            let r = try_request(&fast(), &f.base, method, "/rest/v1/x", None, None, Some(b"{}"), &[], 1 << 16).ok().unwrap();
            assert_eq!(r.state, 503, "{}", method);
            assert_eq!(f.hits(), 1, "{}", method);
            assert_eq!(f.methods.lock().unwrap().as_slice(), [method]);
        }
        // a timeout on an upload is not retried either (it could create the item twice)
        let f = fake(vec![Act::Hang(3000), reply(200, "{}")]);
        let cfg = NetCfg { write: Duration::from_millis(300), ..fast() };
        assert!(try_request(&cfg, &f.base, "POST", "/rest/v1/x", None, None, Some(b"{}"), &[], 1 << 16).is_err());
        assert_eq!(f.hits(), 1);
    }

    #[test]
    fn rate_limit_honours_retry_after_and_says_how_long() {
        // a short Retry-After is waited for, then the request works
        let f = fake(vec![Act::Reply(429, vec![("Retry-After", "0".into())], String::new()), reply(200, "[]")]);
        assert_eq!(get(&fast(), &f.base).ok().unwrap().state, 200);
        assert_eq!(f.hits(), 2);
        // a long one is not waited for: the message tells the number of seconds
        let f = fake(vec![Act::Reply(429, vec![("Retry-After", "600".into())], String::new()), reply(200, "[]")]);
        let r = get(&fast(), &f.base).ok().unwrap();
        assert_eq!((r.state, r.retry_after), (429, Some(600)));
        assert_eq!(f.hits(), 1);
        let e = r.ok().err().unwrap();
        assert!(e.contains("600"), "{}", e);
        // without the header there is a generic message (and the retries are used)
        let f = fake(vec![reply(429, ""), reply(429, ""), reply(429, "")]);
        let r = get(&fast(), &f.base).ok().unwrap();
        assert_eq!((r.state, f.hits()), (429, 3));
        assert!(!r.ok().err().unwrap().contains("600"));
    }

    #[test]
    fn offline_is_distinguished_and_session_expiry_is_a_clear_message() {
        let e = get(&NetCfg { retries: 1, ..fast() }, &closed_port()).err().unwrap();
        assert!(e.transient);
        assert!(e.msg == net::offline_message() || e.msg == net::timeout_message() || e.msg == net::unavailable_message());
        assert!(error_message(401, "").contains("session"));
        assert!(net::is_expired(&error_message(401, "")));
        assert!(!net::is_expired(&error_message(403, "")));
        assert!(!net::is_expired(&error_message(503, "")));
    }

    fn sess() -> Session {
        Session { access: "a".into(), renewal: "r".into(), uid: UID.into(), expires: now() + 3600 }
    }

    #[test]
    fn current_terms_retries_failures_and_only_reads_a_missing_function_as_no_terms() {
        let one = |script: Vec<Act>| {
            let f = fake(script);
            let r = fetch_current_terms_with(&fast(), &f.base, &sess(), 3);
            (r, f.hits())
        };
        // a good answer
        assert_eq!(one(vec![reply(200, r#"{"version":"1.1"}"#)]), (TermsFetch::Version("1.1".into()), 1));
        // a 503 or a cut connection is retried and the later answer is used
        assert_eq!(one(vec![reply(503, ""), reply(200, r#"{"version":"1.1"}"#)]).0, TermsFetch::Version("1.1".into()));
        assert_eq!(one(vec![Act::Hang(1500), reply(200, r#"{"version":"2.0"}"#)]).0, TermsFetch::Version("2.0".into()));
        assert_eq!(one(vec![reply(429, ""), reply(503, ""), reply(200, r#"{"version":"1.2"}"#)]).0, TermsFetch::Version("1.2".into()));
        // failing every time: Failed (NOT "no terms")
        assert_eq!(one(vec![reply(503, ""), reply(503, ""), reply(503, "")]).0, TermsFetch::Failed);
        assert_eq!(fetch_current_terms_with(&fast(), &closed_port(), &sess(), 2), TermsFetch::Failed);
        // a refused session or any other 4xx is not "no terms" either, and is not retried
        assert_eq!(one(vec![reply(401, "{}")]), (TermsFetch::Failed, 1));
        assert_eq!(one(vec![reply(403, "{}")]), (TermsFetch::Failed, 1));
        // garbage in a 200
        assert_eq!(one(vec![reply(200, "<html>")]).0, TermsFetch::Failed);
        // really missing: 404, the PostgREST "function not found", or nothing published
        assert_eq!(one(vec![reply(404, "{}")]), (TermsFetch::Missing, 1));
        assert_eq!(one(vec![reply(400, r#"{"code":"PGRST202","message":"Could not find the function public.current_terms"}"#)]).0, TermsFetch::Missing);
        assert_eq!(one(vec![reply(200, "null")]).0, TermsFetch::Missing);
        assert_eq!(one(vec![reply(200, r#"{"version":null}"#)]).0, TermsFetch::Missing);
        // the decision: an unreadable server is Unknown (uploads wait), a missing one is NotApplicable
        let own = Some(net::OwnState::default());
        let unknown = TermsInfo { server_current: None, own: own.clone(), server_unavailable: true };
        assert_eq!(unknown.decision(), net::TermsDecision::Unknown);
        let missing = TermsInfo { server_current: None, own, server_unavailable: false };
        assert_eq!(missing.decision(), net::TermsDecision::NotApplicable);
    }

    #[test]
    fn texture_downloads_are_retried_after_a_failure_and_never_stuck() {
        let t0 = std::time::Instant::now();
        let mut g = TexGate::default();
        assert!(g.may_start("a", t0));
        assert!(!g.may_start("a", t0), "already downloading");
        g.finish("a", false, t0);
        assert!(!g.may_start("a", t0 + Duration::from_secs(5)), "paused right after a failure");
        assert!(g.may_start("a", t0 + TEXTURE_RETRY + Duration::from_secs(1)), "retried later");
        g.finish("a", true, t0 + TEXTURE_RETRY);
        assert!(g.failed.is_empty() && g.loading.is_empty());
        assert!(g.may_start("quarantine:avatar", t0));
        g.forget_prefix("quarantine:");
        assert!(g.may_start("quarantine:avatar", t0), "forgotten");
    }

    #[test]
    fn server_texts_are_cleaned_before_they_are_painted() {
        let evil = "A\u{202E}b\u{200B}c\u{7}d\ne\u{2028}f";
        assert_eq!(display_clean(evil, false), "Abcdef");
        assert_eq!(display_clean(evil, true), "Abcd\nef");
        let mut it = Item { title: evil.into(), description: evil.into(), author_name: evil.into(), tags: vec![evil.into()], ..Default::default() };
        it.clean_texts();
        assert_eq!((it.title.as_str(), it.description.as_str(), it.author_name.as_str(), it.tags[0].as_str()), ("Abcdef", "Abcd\nef", "Abcdef", "Abcdef"));
        let mut p = ProfilePublic { public_name: evil.into(), bio: evil.into(), github: Some(evil.into()), ..Default::default() };
        p.links.insert("x".into(), evil.into());
        p.clean_texts();
        assert_eq!((p.public_name.as_str(), p.bio.as_str(), p.github.as_deref(), p.links["x"].as_str()), ("Abcdef", "Abcd\nef", Some("Abcdef"), "Abcdef"));
        // list rows go through the same cleaning
        let mut row: Item = serde_json::from_str(&items_json().trim_start_matches('[').trim_end_matches(']')).unwrap();
        row.title = evil.into();
        assert_eq!(sanitize_items(vec![row], true)[0].title, "Abcdef");
    }

    #[test]
    fn the_last_good_list_is_shown_when_the_server_cannot_be_reached() {
        let cache = tmp("list");
        let run = |base: &str, search: &str| list_with(&fast(), base, &cache, None, None, search, false, None);
        let query = format!("/rest/v1/community_items?select={}&status=eq.approved&order=created_at.desc&limit=60&is_18=eq.false", ITEM_COLUMNS_V15);
        let key_of = |base: &str| net::catalog_key(&format!("{}|{}", base, query));
        let item = || serde_json::from_str::<Item>(&items_json().trim_start_matches('[').trim_end_matches(']')).unwrap();
        // the cache belongs to one server: copy the fresh entry to the base under test
        let seed = |base: &str| assert!(net::catalog_store(&cache, &key_of(base), &[item()], now()));
        // nothing cached and the server is down: the error is reported
        let down = closed_port();
        assert!(run(&down, "").is_err());
        // a good answer is returned fresh and cached
        let f = fake(vec![reply(200, &items_json())]);
        let out = run(&f.base, "").ok().unwrap();
        assert_eq!((out.items.len(), out.stale), (1, None));
        assert!(net::catalog_load::<Item>(&cache, &key_of(&f.base), now()).is_some(), "the key used by the tests is the real one");
        assert!(net::catalog_load::<Item>(&cache, &key_of(&f.base), now() + net::CATALOG_TTL_SECS + 60).is_none(), "expired after 24 h");
        // offline (same server): the cached list comes back, marked as stale
        let out = run(&f.base, "").ok().unwrap();
        assert_eq!(out.items.len(), 1);
        assert!(out.stale.is_some());
        assert_eq!(out.items[0].title, "Cat");
        // another server never gets this server's list (M3)
        assert!(run(&down, "").is_err(), "different base URL, different cache");
        // the server answers 503 forever: also the cache
        let f = fake(vec![reply(503, ""), reply(503, ""), reply(503, "")]);
        seed(&f.base);
        assert!(run(&f.base, "").ok().unwrap().stale.is_some());
        // 429 too
        let f = fake(vec![reply(429, ""), reply(429, ""), reply(429, "")]);
        seed(&f.base);
        assert!(run(&f.base, "").ok().unwrap().stale.is_some());
        // but a refused session is NOT hidden behind the cache
        let f = fake(vec![reply(401, "{}")]);
        seed(&f.base);
        let e = run(&f.base, "").err().unwrap();
        assert!(net::is_expired(&e), "{}", e);
        // searches are never cached nor served from the cache
        let f = fake(vec![reply(200, &items_json())]);
        seed(&f.base);
        assert!(run(&f.base, "cat").ok().unwrap().stale.is_none());
        assert!(run(&f.base, "cat").is_err(), "no cached search");
        // a poisoned cache file: bad rows are dropped, 18+ rows too while 18+ is off, an invalid shape is ignored
        let f = fake(vec![]);
        let mut bad = item();
        bad.id = "x".into();
        bad.kind = "evil".into();
        let mut adult = item();
        adult.is_18 = true;
        assert!(net::catalog_store(&cache, &key_of(&f.base), &[bad, adult, item()], now()));
        assert_eq!(run(&f.base, "").ok().unwrap().items.len(), 1, "only the valid, non-18+ row survives");
        std::fs::write(cache.join(format!("catalog-{}.json", key_of(&f.base))), r#"{"saved_at":1,"items":"nope"}"#).unwrap();
        assert!(run(&f.base, "").is_err());
        let _ = std::fs::remove_dir_all(&cache);
    }
}
