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
use crate::community_validate::{self as val, Package};
use crate::i18n::t;
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

#[derive(Clone, Debug, Default, Deserialize)]
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
    Items(Result<Vec<Item>, String>),
    Downloaded { item: Item, bytes: Result<Vec<u8>, String> },
    Uploaded(Result<String, String>),
    Reported(Result<(), String>),
    AccountDeleted(Result<(), String>),
    State(String),
    Image { path: String, bytes: Option<Vec<u8>> },
    ProfileView(Result<(ProfilePublic, Vec<Item>), String>),
    MyProfile(Option<ProfilePublic>),
    /// My hearts (content ids) and the creators I follow (account ids).
    Mine { hearts: Vec<String>, following: Vec<String> },
}

#[derive(Default)]
pub struct UploadForm {
    pub kind: usize,
    pub title: String,
    pub description: String,
    pub labels: String,
    pub is_18: bool,
    pub file: Option<PathBuf>,
    pub view: Option<PathBuf>,
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
    /// A sign-in is waiting on the browser; the value is used to cancel it.
    login: Option<Arc<AtomicBool>>,
    /// Community images already loaded (None = loading or unavailable).
    textures: std::collections::HashMap<String, Option<egui::TextureHandle>>,
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
            login: None,
            textures: Default::default(),
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

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(170))).http_status_as_error(false).user_agent("LienzoUI-Community/1.0").build().into()
}

struct Resp {
    state: u16,
    body: Vec<u8>,
}

impl Resp {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
    fn ok(self) -> Result<Resp, String> {
        if (200..300).contains(&self.state) {
            Ok(self)
        } else {
            Err(error_message(self.state, &self.text()))
        }
    }
    fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, String> {
        serde_json::from_slice(&self.body).map_err(|_| "Unexpected response from the server.".to_string())
    }
}

/// Understandable error messages; the raw server body is never returned.
fn error_message(state: u16, body: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
    let msg = ["message", "msg", "error_description", "error"].iter().find_map(|k| v.get(k).and_then(|m| m.as_str())).unwrap_or("");
    match state {
        401 | 403 => "You do not have permission or your session expired. Sign in again.".into(),
        404 => "No encontrado.".into(),
        409 => "It already exists or was already processed.".into(),
        429 => "Demasiadas peticiones: espera un momento.".into(),
        _ if msg.contains("Demasiadas subidas") || msg.contains("Daily limit") => msg.to_string(),
        _ if msg.contains("duplicate key") => "That name is already in use.".into(),
        s if s >= 500 => "The server is not available right now.".into(),
        s => format!("Error {}", s),
    }
}

fn request(method: &str, path: &str, session: Option<&Session>, kind: Option<&str>, body: Option<&[u8]>, extra: &[(&str, &str)], limit: usize) -> Result<Resp, String> {
    let url = format!("{}{}", URL, path);
    let a = agent();
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
    let mut res = res.map_err(|_| "No connection to the server.".to_string())?;
    let state = res.status().as_u16();
    let body = res.body_mut().with_config().limit(limit as u64).read_to_vec().map_err(|_| "The server response is too large or was cut off.".to_string())?;
    Ok(Resp { state, body })
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

fn renew(s: &Session) -> Result<Session, String> {
    let r = json_post("/auth/v1/token?grant_type=refresh_token", None, &serde_json::json!({ "refresh_token": s.renewal }), &[])?.ok()?;
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

/// Sign-in with GitHub or Google: OAuth with PKCE. The browser returns to a temporary local port (127.0.0.1)
/// and the app exchanges the code; no intermediary server sees the session.
fn oauth_flow(provider: &str, cancel: &AtomicBool) -> Result<Session, String> {
    if !["github", "google"].contains(&provider) {
        return Err("Provider not supported.".into());
    }
    let listen = std::net::TcpListener::bind(("127.0.0.1", LOCAL_PORT)).map_err(|_| format!("Local port {} is busy. Close another Lienzo UI session and try again.", LOCAL_PORT))?;
    listen.set_nonblocking(true).map_err(|e| e.to_string())?;
    let verifier = b64url(&random(48));
    let challenge = b64url(&Sha256::digest(verifier.as_bytes()));
    let return_url = format!("http://127.0.0.1:{}/callback", LOCAL_PORT);
    let url = format!("{}/auth/v1/authorize?provider={}&redirect_to={}&code_challenge={}&code_challenge_method=s256", URL, provider, encode(&return_url), challenge);
    webbrowser::open(&url).map_err(|_| "Could not open the browser.".to_string())?;

    let limit = std::time::Instant::now() + Duration::from_secs(180);
    let code = loop {
        if cancel.load(Ordering::Relaxed) {
            return Err("Sign-in canceled.".into());
        }
        if std::time::Instant::now() > limit {
            return Err("Timed out waiting for sign-in.".into());
        }
        match listen.accept() {
            Ok((mut c, _)) => {
                let _ = c.set_nonblocking(false);
                let _ = c.set_read_timeout(Some(Duration::from_secs(5)));
                let mut buf = [0u8; 4096];
                let n = c.read(&mut buf).unwrap_or(0);
                let request = String::from_utf8_lossy(&buf[..n]).to_string();
                let line = request.lines().next().unwrap_or("");
                let path = line.strip_prefix("GET ").and_then(|r| r.split(' ').next()).unwrap_or("");
                let Some(query) = path.strip_prefix("/callback?") else {
                    let _ = c.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                    continue; // favicon or other requests: ignored
                };
                let param = |k: &str| query.split('&').find_map(|p| p.strip_prefix(&format!("{}=", k))).map(|v| v.to_string());
                if param("error").is_some() || param("error_description").is_some() {
                    let _ = c.write_all(return_page(false).as_bytes());
                    return Err("Sign-in was canceled.".into());
                }
                match param("code").filter(|c| !c.is_empty() && c.len() < 300 && c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')) {
                    Some(code) => {
                        let _ = c.write_all(return_page(true).as_bytes());
                        break code;
                    }
                    None => {
                        let _ = c.write_all(return_page(false).as_bytes());
                        return Err("Invalid sign-in response.".into());
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(150)),
            Err(e) => return Err(e.to_string()),
        }
    };
    let r = json_post("/auth/v1/token?grant_type=pkce", None, &serde_json::json!({ "auth_code": code, "code_verifier": verifier }), &[])?.ok()?;
    session_from(&r.json::<serde_json::Value>()?)
}

// ───────────────────────────── queries ─────────────────────────────

fn clear_search(q: &str) -> String {
    q.chars().filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_').take(40).collect::<String>().trim().to_string()
}

fn list(session: Option<&Session>, filter: Option<&str>, search: &str, with_18: bool, only_from: Option<&[String]>) -> Result<Vec<Item>, String> {
    if let Some(ids) = only_from {
        if ids.is_empty() {
            return Ok(Vec::new()); // you follow no one: nothing to request
        }
    }
    let mut q = String::from("/rest/v1/community_items?select=id,kind,title,description,tags,is_18,public_path,sha256,bytes,downloads,version,author,author_name,public_icon,public_preview,hearts&status=eq.approved&order=created_at.desc&limit=60");
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
    let mut items: Vec<Item> = request("GET", &q, session, None, None, &[], 2 << 20)?.ok()?.json()?;
    items.retain(|i| is_uuid(&i.id) && is_uuid(&i.author) && val::KINDS.contains(&i.kind.as_str()));
    Ok(items)
}

const ITEM_COLUMNS: &str = "id,kind,title,description,tags,is_18,public_path,sha256,bytes,downloads,version,author,author_name,public_icon,public_preview,hearts";

fn load_public_profile(session: Option<&Session>, uid: &str) -> Result<(ProfilePublic, Vec<Item>), String> {
    if !is_uuid(uid) {
        return Err("Invalid creator.".into());
    }
    let p: Vec<ProfilePublic> = request("GET", &format!("/rest/v1/public_profiles?select=id,public_name,bio,github,links,public_avatar,public_banner,followers,hearts_received&id=eq.{}", uid), session, None, None, &[], 1 << 18)?.ok()?.json()?;
    let p = p.into_iter().next().ok_or("This creator does not have a public profile yet.")?;
    let mut items: Vec<Item> = request("GET", &format!("/rest/v1/community_items?select={}&status=eq.approved&author=eq.{}&order=created_at.desc&limit=60", ITEM_COLUMNS, uid), session, None, None, &[], 2 << 20)?.ok()?.json()?;
    items.retain(|i| is_uuid(&i.id) && val::KINDS.contains(&i.kind.as_str()));
    Ok((p, items))
}

fn load_my_profile(s: &Session) -> Option<ProfilePublic> {
    let r = request("GET", &format!("/rest/v1/public_profiles?select=*&id=eq.{}", s.uid), Some(s), None, None, &[], 1 << 18).ok()?.ok().ok()?;
    r.json::<Vec<ProfilePublic>>().ok()?.into_iter().next()
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

fn upload_content(s: &Session, kind: &str, title: &str, description: &str, labels: Vec<String>, is_18: bool, name: &str, bytes: &[u8], extras: &[(&str, Vec<u8>)], tx: &Sender<Message>) -> Result<String, String> {
    val::validate(kind, name, bytes)?;
    for (class, b) in extras {
        val::validate_png(class, b).map_err(|e| format!("{} ({})", e, class))?;
    }
    let s = current(s, tx)?;
    let id = uuid_v4();
    let file = safe_name(name);
    let path = format!("{}/{}/{}", s.uid, id, file);
    let row = serde_json::json!({
        "id": id, "author": s.uid, "kind": kind, "title": title, "description": description,
        "tags": labels, "is_18": is_18, "version": "1.0.0", "quarantine_path": path,
        "quarantine_icon": extras.iter().any(|(c, _)| *c == "icon").then(|| format!("{}/{}/icon.png", s.uid, id)),
        "quarantine_preview": extras.iter().any(|(c, _)| *c == "preview").then(|| format!("{}/{}/preview.png", s.uid, id)),
    });
    json_post("/rest/v1/community_items", Some(&s), &row, &[("Prefer", "return=minimal")])?.ok()?;
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

fn install_plugin(bytes: &[u8]) -> Result<String, String> {
    let p: Package = val::read_package(bytes)?;
    let name = crate::plugins::manifest_key(&p.manifest, "name", "nombre").and_then(|n| n.as_str()).unwrap_or("plugin");
    let folder = crate::plugins::user_folder().join(format!("comunidad-{}", safe_name(name).to_lowercase()));
    std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    for (path, content) in &p.files {
        let destination = folder.join(path); // validated: no "..", no absolute paths, only .rhai/.json
        if let Some(parent) = destination.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(&destination, content).map_err(|e| e.to_string())?;
    }
    if !p.files.contains_key("plugin.json") {
        std::fs::write(folder.join("plugin.json"), serde_json::to_string_pretty(&p.manifest).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    }
    Ok(name.to_string())
}

/// Turns a plugin folder into the package that gets uploaded.
fn pack_plugin(folder: &std::path::Path) -> Result<Vec<u8>, String> {
    let manifest: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(folder.join("plugin.json")).map_err(|_| "The folder has no plugin.json.")?).map_err(|_| "plugin.json is not valid.")?;
    let mut files = std::collections::BTreeMap::new();
    files.insert("plugin.json".to_string(), serde_json::to_string(&manifest).map_err(|e| e.to_string())?);
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
const REASONS: &[(&str, &str)] = &[("malware", "Malware or viruses"), ("adult_content", "Unmarked 18+ content"), ("hate_or_harassment", "Hate or harassment"), ("copyright", "Copyright"), ("other", "Other")];

fn kind_name(t: &str) -> &'static str {
    KIND_NAMES.iter().find(|(k, _)| *k == t).map(|(_, n)| *n).unwrap_or("")
}

impl EditorApp {
    fn community_launch(&mut self, ctx: &egui::Context, f: impl FnOnce(&Sender<Message>) + Send + 'static) {
        self.community.busy += 1;
        let tx = self.community.tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            f(&tx);
            ctx.request_repaint();
        });
    }

    fn community_load_list(&mut self, ctx: &egui::Context) {
        let (s, f, b) = (self.community.session.clone(), self.community.filter, self.community.search.clone());
        let with_18 = self.community.profile.as_ref().map(|p| p.show_18).unwrap_or(false) && s.is_some();
        let only_from: Option<Vec<String>> = (self.community.only_following && s.is_some()).then(|| self.community.my_following.iter().cloned().collect());
        self.community_launch(ctx, move |tx| {
            let s = s.and_then(|s| current(&s, tx).ok());
            let _ = tx.send(Message::Items(list(s.as_ref(), f, &b, with_18, only_from.as_deref())));
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
                        Ok(v) => self.community.items = v,
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
                    self.community.state = r.unwrap_or_else(|e| e);
                    self.community_load_list(ctx);
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
                    self.community.textures.insert(path, tex);
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
                    self.community.my_profile = p;
                }
                Message::State(e) => {
                    self.community.busy = self.community.busy.saturating_sub(1);
                    self.community.state = e;
                }
            }
        }
    }

    fn community_close_local(&mut self) {
        self.community.session = None;
        self.community.profile = None;
        self.community.profile_loaded = false;
        self.community.delete_confirmed = false;
        self.community.my_profile = None;
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
                let f = write_temp(&format!("{}.json", safe_name(&item.title)), bytes)?;
                let r = self.library.add_template_file(&f);
                let _ = std::fs::remove_file(&f);
                format!("{} “{}” (Templates › My templates)", t("Template installed:"), r?)
            }
            _ => {
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
                }
            });
        });
        ui.label(RichText::new(t("Shared plugins, images, fonts, templates and elements. Everything is reviewed before it is published.")).small().color(MUTED()));
        if !self.community.state.is_empty() {
            ui.add_space(4.0);
            ui.label(RichText::new(&self.community.state).small().color(ACCENT()));
        }
        ui.add_space(6.0);
        self.community_account(ui, &ctx);
        ui.add_space(6.0);
        self.community_explore(ui, &ctx);
        if self.community.session.is_some() && self.community.profile.is_some() {
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
                ui.checkbox(&mut self.community.accepted, t("I accept the rules: no malware, hate, plagiarism or unmarked 18+ content"));
                let n = self.community.new_name.trim().to_string();
                let valid = (3..=32).contains(&n.chars().count()) && n.chars().all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c));
                if ui.add_enabled(valid && self.community.accepted && self.community.busy == 0, egui::Button::new(t("Create profile"))).clicked() {
                    let s = self.community.session.clone().unwrap();
                    self.community_launch(ctx, move |tx| {
                        let r = current(&s, tx).and_then(|s| json_post("/rest/v1/profiles", Some(&s), &serde_json::json!({ "id": s.uid, "public_name": n, "terms_accepted_at": iso(now()) }), &[("Prefer", "return=minimal")])?.ok().map(|_| s));
                        match r {
                            Ok(s) => {
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
            let ready = (3..=80).contains(&f.title.trim().chars().count()) && f.file.is_some() && self.community.busy == 0;
            if ui.add_enabled(ready, egui::Button::new(t("Send for review"))).clicked() {
                let f = &self.community.upload;
                let (title, desc, is_18, path) = (f.title.trim().to_string(), f.description.chars().take(1000).collect::<String>(), f.is_18, f.file.clone().unwrap());
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
                            let r = upload_content(&s, kind, &title, &desc, labels, is_18, &name, &bytes, &extras, tx);
                            let _ = tx.send(Message::Uploaded(r));
                        });
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
    if !val::KINDS.contains(&kind) || !path.exists() || title.trim().chars().count() < 3 {
        eprintln!("Usage: lienzo-ui --community-upload <plugin|image|font|template|element|theme> <file|folder> --title \"…\" [--description \"…\"] [--labels a,b] [--18] [--icon icon.png] [--preview preview.png]");
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
    match upload_content(&s, kind, title.trim(), &option("--description"), labels, is_18, &name, &bytes, &extras, &tx) {
        Ok(m) => {
            println!("{}: {}", kind, m);
            0
        }
        Err(e) => {
            eprintln!("{}: {}", kind, e);
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
            Ok(v) => println!("{}: {} ({})", title, v["status"].as_str().unwrap_or("?"), v["reason"].as_str().unwrap_or("")),
            Err(e) => {
                eprintln!("{}: {}", title, e);
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

/// Human review (only accounts listed in the server's MODERATORS secret):
///   lienzo-ui --community-review list
///   lienzo-ui --community-review approve <id> [--18]      · reject <id> "reason"
///   lienzo-ui --community-review approve-profile <account-id> avatar|banner  · reject-profile <account-id> avatar|banner
///   lienzo-ui --community-review token-create "name" read,reject [days]    (credential for scripts; shown once)
///   lienzo-ui --community-review tokens                                       · token-revoke <id>
/// (The old Spanish action names — listar, aprobar, rechazar, … — still work.)
/// The same can be done in the moderation web page (web/moderacion).
pub fn review_cli(args: &[String]) -> i32 {
    let action = review_action(args.first().map(|s| s.as_str()).unwrap_or(""));
    let a1 = args.get(1).map(|s| s.as_str()).unwrap_or("");
    let a2 = args.get(2).map(|s| s.as_str()).unwrap_or("");
    let usage = "Usage: lienzo-ui --community-review list | approve <id> [--18] | reject <id> \"reason\" | approve-profile <account> avatar|banner | reject-profile <account> avatar|banner | token-create \"name\" read,reject [days] | tokens | token-revoke <id> | rescan [id]";
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
    let r = match json_post("/functions/v1/review", Some(&s), &body, &[]) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let v: serde_json::Value = r.json().unwrap_or_default();
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

fn open_link(url: &str) {
    // Last check before opening the browser: https only.
    if url.starts_with("https://") && url.chars().all(|c| c.is_ascii_graphic()) {
        let _ = webbrowser::open(url);
    }
}

impl EditorApp {
    /// Public community image (icon, preview, avatar or banner). Downloaded in the background the first time.
    fn community_texture(&mut self, ctx: &egui::Context, path: &str) -> Option<egui::TextureHandle> {
        if let Some(t) = self.community.textures.get(path) {
            return t.clone();
        }
        self.community.textures.insert(path.to_string(), None);
        if !valid_public_path(path) {
            return None;
        }
        let (tx, c2, r) = (self.community.tx.clone(), ctx.clone(), path.to_string());
        std::thread::spawn(move || {
            let bytes = request("GET", &format!("/storage/v1/object/public/publico/{}", r), None, None, None, &[], (2 << 20) + 1)
                .and_then(|x| x.ok())
                .ok()
                .map(|x| x.body)
                .filter(|b| val::detect(b) == Some("png"));
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
                for (k, n) in REASONS {
                    if ui.small_button(t(n)).clicked() {
                        let (s, id, reason) = (self.community.session.clone().unwrap(), it.id.clone(), k.to_string());
                        self.community.report = None;
                        self.community_launch(ctx, move |tx| {
                            let r = current(&s, tx).and_then(|s| json_post("/rest/v1/reports", Some(&s), &serde_json::json!({ "item": id, "reporter": s.uid, "reason": reason }), &[("Prefer", "return=minimal")])?.ok().map(|_| ()));
                            let _ = tx.send(Message::Reported(r));
                        });
                    }
                }
                if ui.small_button(t("Cancel")).clicked() {
                    self.community.report = None;
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
            ui.label(RichText::new(&self.community.state).small().color(ACCENT()));
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
        if let Some(t) = self.community.textures.get(&key) {
            return t.clone();
        }
        let s = self.community.session.clone()?;
        self.community.textures.insert(key.clone(), None);
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
        println!("total now: {}", t.text());
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
