//! Lienzo UI community: browse, install, upload and report plugins, images, fonts, templates and elements
//! without leaving the app. The server is Supabase (see supabase/ and docs/COMUNIDAD.md).
//!
//! Privacy and security on the app side:
//!   * No connection is made until you open the Community tab.
//!   * Only the project's PUBLIC key is used; with RLS it opens nothing that isn't allowed. The `service_role` key never goes in.
//!   * Sign-in is through GitHub or Google (OAuth with PKCE and a temporary local port); the app never sees passwords.
//!   * The session (refresh token) is stored in the Windows Credential Manager, not in a file.
//!   * Everything downloaded is verified (size + SHA-256 + the same rules as the server) before being installed.
//!   * +18 content is hidden by default and only shown if you enable it with your account.

use crate::app::EditorApp;
use crate::comunidad_validar::{self as val, Paquete};
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
pub const CLAVE_PUBLICA: &str = "sb_publishable_UNvWtzs_M8N_N6fLrZ9VCQ_E6hkxXrL";
const PUERTO_LOCAL: u16 = 53682;
const SERVICIO_CREDENCIALES: &str = "LienzoUI";

// ───────────────────────────── data ─────────────────────────────

#[derive(Clone, Debug, Default)]
pub struct Sesion {
    pub acceso: String,
    pub renovacion: String,
    pub uid: String,
    pub expira: u64,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Perfil {
    pub nombre_publico: String,
    pub mostrar_18: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Item {
    pub id: String,
    pub tipo: String,
    pub titulo: String,
    pub descripcion: String,
    pub etiquetas: Vec<String>,
    pub es_18: bool,
    pub ruta_publica: Option<String>,
    pub sha256: Option<String>,
    pub bytes: Option<i64>,
    pub descargas: i64,
    pub version: String,
    pub autor: String,
    pub autor_nombre: String,
    pub icono_publico: Option<String>,
    pub vista_publica: Option<String>,
    pub corazones: i64,
}

/// Public profile of a creator (what the person chooses to show; no email or preferences).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct PerfilPublico {
    pub id: String,
    pub nombre_publico: String,
    pub bio: String,
    pub github: Option<String>,
    pub enlaces: std::collections::BTreeMap<String, String>,
    pub avatar_publico: Option<String>,
    pub banner_publico: Option<String>,
    pub avatar_pendiente: bool,
    pub banner_pendiente: bool,
    pub seguidores: i64,
    pub corazones_recibidos: i64,
}

/// Networks supported in the profile: (key, display name, allowed domains). Same as on the server.
const REDES: &[(&str, &str, &[&str])] = &[
    ("x", "X", &["x.com", "twitter.com"]),
    ("youtube", "YouTube", &["youtube.com", "www.youtube.com"]),
    ("itch", "itch.io", &["itch.io"]),
    ("twitch", "Twitch", &["twitch.tv", "www.twitch.tv"]),
    ("bluesky", "Bluesky", &["bsky.app"]),
    ("instagram", "Instagram", &["instagram.com", "www.instagram.com"]),
];

/// https only, no user or port, and from an allowed domain (prevents phishing or malware links).
fn enlace_valido(red: &str, url: &str) -> bool {
    let Some((_, _, dominios)) = REDES.iter().find(|(k, _, _)| *k == red) else { return false };
    if url.len() > 200 || !url.chars().all(|c| c.is_ascii_graphic()) {
        return false;
    }
    let Some(resto) = url.strip_prefix("https://") else { return false };
    let host = resto.split(['/', '?', '#']).next().unwrap_or("");
    if host.contains('@') || host.contains(':') {
        return false;
    }
    let h = host.to_lowercase();
    dominios.contains(&h.as_str()) || (red == "itch" && h.strip_suffix(".itch.io").is_some_and(|u| !u.is_empty() && u.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')))
}

fn github_valido(u: &str) -> bool {
    !u.is_empty() && u.len() <= 39 && !u.starts_with('-') && u.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

fn ruta_publica_valida(ruta: &str) -> bool {
    ruta.split('/').all(|p| !p.is_empty() && p != ".." && p.chars().all(|c| c.is_ascii_alphanumeric() || "-._".contains(c)))
}

#[derive(Default)]
pub struct FormPerfil {
    nombre: String,
    bio: String,
    github: String,
    enlaces: Vec<String>,
    cargado: bool,
}

enum Mensaje {
    Sesion(Result<Sesion, String>),
    Login(Result<Sesion, String>),
    SesionRenovada(Sesion),
    Perfil(Option<Perfil>),
    Items(Result<Vec<Item>, String>),
    Descargado { item: Item, bytes: Result<Vec<u8>, String> },
    Subido(Result<String, String>),
    Reportado(Result<(), String>),
    CuentaBorrada(Result<(), String>),
    Estado(String),
    Imagen { ruta: String, bytes: Option<Vec<u8>> },
    PerfilVer(Result<(PerfilPublico, Vec<Item>), String>),
    MiPerfil(Option<PerfilPublico>),
    /// My hearts (content ids) and the creators I follow (account ids).
    Mios { corazones: Vec<String>, seguidos: Vec<String> },
}

#[derive(Default)]
pub struct SubirForm {
    pub tipo: usize,
    pub titulo: String,
    pub descripcion: String,
    pub etiquetas: String,
    pub es_18: bool,
    pub archivo: Option<PathBuf>,
    pub vista: Option<PathBuf>,
}

pub struct Comunidad {
    tx: Sender<Mensaje>,
    rx: Receiver<Mensaje>,
    pub sesion: Option<Sesion>,
    pub perfil: Option<Perfil>,
    pub perfil_cargado: bool,
    items: Vec<Item>,
    filtro: Option<&'static str>,
    busqueda: String,
    ocupado: u32,
    iniciado: bool,
    pub estado: String,
    nombre_nuevo: String,
    acepto: bool,
    mayor_edad: bool,
    subir: SubirForm,
    borrar_confirmado: bool,
    reportar: Option<String>,
    /// A sign-in is waiting on the browser; the value is used to cancel it.
    login: Option<Arc<AtomicBool>>,
    /// Community images already loaded (None = loading or unavailable).
    texturas: std::collections::HashMap<String, Option<egui::TextureHandle>>,
    /// Open creator profile and its content.
    viendo: Option<(PerfilPublico, Vec<Item>)>,
    mi_perfil: Option<PerfilPublico>,
    form_perfil: FormPerfil,
    mis_corazones: std::collections::HashSet<String>,
    mis_seguidos: std::collections::HashSet<String>,
    solo_siguiendo: bool,
    /// Preview open at full size (public path of the image).
    ampliada: Option<String>,
    perfil_abierto: bool,
    /// Open the "Share" form the next time it is drawn (when arriving from the Themes window).
    abrir_subida: bool,
    /// Newly chosen photo and banner (shown immediately, before the server publishes them).
    previas: std::collections::HashMap<&'static str, egui::TextureHandle>,
}

impl Comunidad {
    /// Public name of the signed-in account, if any.
    pub fn perfil_nombre(&self) -> Option<String> {
        self.sesion.as_ref()?;
        self.perfil.as_ref().map(|p| p.nombre_publico.clone())
    }
}

impl Default for Comunidad {
    fn default() -> Self {
        Self::new()
    }
}

impl Comunidad {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Self {
            tx,
            rx,
            sesion: None,
            perfil: None,
            perfil_cargado: false,
            items: Vec::new(),
            filtro: None,
            busqueda: String::new(),
            ocupado: 0,
            iniciado: false,
            estado: String::new(),
            nombre_nuevo: String::new(),
            acepto: false,
            mayor_edad: false,
            subir: SubirForm::default(),
            borrar_confirmado: false,
            reportar: None,
            login: None,
            texturas: Default::default(),
            viendo: None,
            mi_perfil: None,
            form_perfil: FormPerfil::default(),
            mis_corazones: Default::default(),
            mis_seguidos: Default::default(),
            solo_siguiendo: false,
            ampliada: None,
            perfil_abierto: false,
            abrir_subida: false,
            previas: Default::default(),
        }
    }
}

// ───────────────────────────── utilities ─────────────────────────────

fn ahora() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// UTC date and time in ISO 8601 format (for `acepto_terminos_en`).
fn iso(t: u64) -> String {
    let (dias, resto) = ((t / 86400) as i64, t % 86400);
    let z = dias + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", y, m, d, resto / 3600, (resto % 3600) / 60, resto % 60)
}

fn aleatorio(n: usize) -> Vec<u8> {
    let mut b = vec![0u8; n];
    getrandom::fill(&mut b).expect("el sistema no dio números aleatorios");
    b
}

fn b64url(b: &[u8]) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b)
}

fn uuid_v4() -> String {
    let mut b = aleatorio(16);
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{:02x}", x)).collect();
    format!("{}-{}-{}-{}-{}", &h[0..8], &h[8..12], &h[12..16], &h[16..20], &h[20..32])
}

fn codificar(s: &str) -> String {
    s.bytes().map(|c| if c.is_ascii_alphanumeric() || b"-._~".contains(&c) { (c as char).to_string() } else { format!("%{:02X}", c) }).collect()
}

/// Safe file name for uploading (only letters, digits, dot, hyphen and underscore).
fn nombre_seguro(n: &str) -> String {
    let s: String = n.chars().map(|c| if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' }).collect();
    let s = s.trim_start_matches('.').to_string();
    if s.is_empty() { "archivo".into() } else { s.chars().take(60).collect() }
}

fn es_uuid(s: &str) -> bool {
    s.len() == 36 && s.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

// ───────────────────────────── network ─────────────────────────────

fn agente() -> ureq::Agent {
    ureq::Agent::config_builder().timeout_global(Some(Duration::from_secs(170))).http_status_as_error(false).user_agent("LienzoUI-Comunidad/1.0").build().into()
}

struct Resp {
    estado: u16,
    cuerpo: Vec<u8>,
}

impl Resp {
    fn texto(&self) -> String {
        String::from_utf8_lossy(&self.cuerpo).to_string()
    }
    fn ok(self) -> Result<Resp, String> {
        if (200..300).contains(&self.estado) {
            Ok(self)
        } else {
            Err(mensaje_error(self.estado, &self.texto()))
        }
    }
    fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, String> {
        serde_json::from_slice(&self.cuerpo).map_err(|_| "Respuesta inesperada del servidor.".to_string())
    }
}

/// Understandable error messages; the raw server body is never returned.
fn mensaje_error(estado: u16, cuerpo: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(cuerpo).unwrap_or_default();
    let msg = ["message", "msg", "error_description", "error"].iter().find_map(|k| v.get(k).and_then(|m| m.as_str())).unwrap_or("");
    match estado {
        401 | 403 => "No tienes permiso o tu sesión caducó. Vuelve a iniciar sesión.".into(),
        404 => "No encontrado.".into(),
        409 => "Ya existe o ya fue procesado.".into(),
        429 => "Demasiadas peticiones: espera un momento.".into(),
        _ if msg.contains("Demasiadas subidas") || msg.contains("Límite diario") => msg.to_string(),
        _ if msg.contains("duplicate key") => "Ese nombre ya está en uso.".into(),
        s if s >= 500 => "El servidor no está disponible ahora mismo.".into(),
        s => format!("Error {}", s),
    }
}

fn pedir(metodo: &str, ruta: &str, sesion: Option<&Sesion>, tipo: Option<&str>, cuerpo: Option<&[u8]>, extra: &[(&str, &str)], limite: usize) -> Result<Resp, String> {
    let url = format!("{}{}", URL, ruta);
    let a = agente();
    macro_rules! cabeceras {
        ($r:expr) => {{
            let mut r = $r.header("apikey", CLAVE_PUBLICA);
            if let Some(s) = sesion {
                r = r.header("Authorization", &format!("Bearer {}", s.acceso));
            }
            for (k, v) in extra {
                r = r.header(*k, *v);
            }
            r
        }};
    }
    let res = match (metodo, cuerpo) {
        ("GET", _) => cabeceras!(a.get(&url)).call(),
        ("DELETE", _) => cabeceras!(a.delete(&url)).call(),
        ("PATCH", Some(c)) => cabeceras!(a.patch(&url)).content_type(tipo.unwrap_or("application/json")).send(c),
        (_, Some(c)) => cabeceras!(a.post(&url)).content_type(tipo.unwrap_or("application/json")).send(c),
        (_, None) => cabeceras!(a.post(&url)).send_empty(),
    };
    let mut res = res.map_err(|_| "Sin conexión con el servidor.".to_string())?;
    let estado = res.status().as_u16();
    let cuerpo = res.body_mut().with_config().limit(limite as u64).read_to_vec().map_err(|_| "La respuesta del servidor es demasiado grande o se cortó.".to_string())?;
    Ok(Resp { estado, cuerpo })
}

fn json_post(ruta: &str, sesion: Option<&Sesion>, v: &serde_json::Value, extra: &[(&str, &str)]) -> Result<Resp, String> {
    pedir("POST", ruta, sesion, None, Some(v.to_string().as_bytes()), extra, 1 << 20)
}

// ───────────────────────────── session ─────────────────────────────

fn sesion_desde(v: &serde_json::Value) -> Result<Sesion, String> {
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).map(|x| x.to_string());
    let acceso = s("access_token").ok_or("El servidor no devolvió una sesión.")?;
    let renovacion = s("refresh_token").ok_or("El servidor no devolvió una sesión.")?;
    let uid = v.get("user").and_then(|u| u.get("id")).and_then(|i| i.as_str()).unwrap_or("").to_string();
    if !es_uuid(&uid) {
        return Err("Identificador de cuenta no válido.".into());
    }
    let dur = v.get("expires_in").and_then(|e| e.as_u64()).unwrap_or(3600);
    Ok(Sesion { acceso, renovacion, uid, expira: ahora() + dur })
}

fn guardar_credencial(s: Option<&Sesion>) {
    #[cfg(windows)]
    {
        if let Ok(e) = keyring::Entry::new(SERVICIO_CREDENCIALES, "sesion") {
            match s {
                Some(s) => {
                    let _ = e.set_password(&serde_json::json!({ "r": s.renovacion, "u": s.uid }).to_string());
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

fn leer_credencial() -> Option<(String, String)> {
    #[cfg(windows)]
    {
        let p = keyring::Entry::new(SERVICIO_CREDENCIALES, "sesion").ok()?.get_password().ok()?;
        let v: serde_json::Value = serde_json::from_str(&p).ok()?;
        return Some((v.get("r")?.as_str()?.to_string(), v.get("u")?.as_str()?.to_string()));
    }
    #[cfg(not(windows))]
    None
}

fn renovar(s: &Sesion) -> Result<Sesion, String> {
    let r = json_post("/auth/v1/token?grant_type=refresh_token", None, &serde_json::json!({ "refresh_token": s.renovacion }), &[])?.ok()?;
    let v: serde_json::Value = r.json()?;
    sesion_desde(&v)
}

/// Returns a valid session; if the token was renewed it notifies the UI.
fn vigente(s: &Sesion, tx: &Sender<Mensaje>) -> Result<Sesion, String> {
    if s.expira > ahora() + 60 {
        return Ok(s.clone());
    }
    let n = renovar(s)?;
    guardar_credencial(Some(&n));
    let _ = tx.send(Mensaje::SesionRenovada(n.clone()));
    Ok(n)
}

fn pagina_retorno(ok: bool) -> String {
    let (titulo, texto) = if ok { ("Listo", "Ya puedes volver a Lienzo UI. Puedes cerrar esta pestaña.") } else { ("No se pudo iniciar sesión", "Vuelve a Lienzo UI e inténtalo de nuevo.") };
    let cuerpo = format!("<!doctype html><meta charset=utf-8><title>Lienzo UI</title><body style=\"font-family:sans-serif;background:#14161f;color:#eee;text-align:center;padding-top:20vh\"><h1>{}</h1><p>{}</p>", titulo, texto);
    format!("HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n{}", cuerpo.len(), cuerpo)
}

/// Sign-in with GitHub or Google: OAuth with PKCE. The browser returns to a temporary local port (127.0.0.1)
/// and the app exchanges the code; no intermediary server sees the session.
fn flujo_oauth(proveedor: &str, cancelar: &AtomicBool) -> Result<Sesion, String> {
    if !["github", "google"].contains(&proveedor) {
        return Err("Proveedor no admitido.".into());
    }
    let escucha = std::net::TcpListener::bind(("127.0.0.1", PUERTO_LOCAL)).map_err(|_| format!("El puerto local {} está ocupado. Cierra otra sesión de Lienzo UI e inténtalo de nuevo.", PUERTO_LOCAL))?;
    escucha.set_nonblocking(true).map_err(|e| e.to_string())?;
    let verificador = b64url(&aleatorio(48));
    let desafio = b64url(&Sha256::digest(verificador.as_bytes()));
    let retorno = format!("http://127.0.0.1:{}/callback", PUERTO_LOCAL);
    let url = format!("{}/auth/v1/authorize?provider={}&redirect_to={}&code_challenge={}&code_challenge_method=s256", URL, proveedor, codificar(&retorno), desafio);
    webbrowser::open(&url).map_err(|_| "No se pudo abrir el navegador.".to_string())?;

    let limite = std::time::Instant::now() + Duration::from_secs(180);
    let codigo = loop {
        if cancelar.load(Ordering::Relaxed) {
            return Err("Inicio de sesión cancelado.".into());
        }
        if std::time::Instant::now() > limite {
            return Err("Se agotó el tiempo para iniciar sesión.".into());
        }
        match escucha.accept() {
            Ok((mut c, _)) => {
                let _ = c.set_nonblocking(false);
                let _ = c.set_read_timeout(Some(Duration::from_secs(5)));
                let mut buf = [0u8; 4096];
                let n = c.read(&mut buf).unwrap_or(0);
                let peticion = String::from_utf8_lossy(&buf[..n]).to_string();
                let linea = peticion.lines().next().unwrap_or("");
                let ruta = linea.strip_prefix("GET ").and_then(|r| r.split(' ').next()).unwrap_or("");
                let Some(consulta) = ruta.strip_prefix("/callback?") else {
                    let _ = c.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                    continue; // favicon or other requests: ignored
                };
                let param = |k: &str| consulta.split('&').find_map(|p| p.strip_prefix(&format!("{}=", k))).map(|v| v.to_string());
                if param("error").is_some() || param("error_description").is_some() {
                    let _ = c.write_all(pagina_retorno(false).as_bytes());
                    return Err("Se canceló el inicio de sesión.".into());
                }
                match param("code").filter(|c| !c.is_empty() && c.len() < 300 && c.chars().all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')) {
                    Some(code) => {
                        let _ = c.write_all(pagina_retorno(true).as_bytes());
                        break code;
                    }
                    None => {
                        let _ = c.write_all(pagina_retorno(false).as_bytes());
                        return Err("Respuesta de inicio de sesión no válida.".into());
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => std::thread::sleep(Duration::from_millis(150)),
            Err(e) => return Err(e.to_string()),
        }
    };
    let r = json_post("/auth/v1/token?grant_type=pkce", None, &serde_json::json!({ "auth_code": codigo, "code_verifier": verificador }), &[])?.ok()?;
    sesion_desde(&r.json::<serde_json::Value>()?)
}

// ───────────────────────────── queries ─────────────────────────────

fn limpiar_busqueda(q: &str) -> String {
    q.chars().filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_').take(40).collect::<String>().trim().to_string()
}

fn listar(sesion: Option<&Sesion>, filtro: Option<&str>, busqueda: &str, con_18: bool, solo_de: Option<&[String]>) -> Result<Vec<Item>, String> {
    if let Some(ids) = solo_de {
        if ids.is_empty() {
            return Ok(Vec::new()); // you follow no one: nothing to request
        }
    }
    let mut q = String::from("/rest/v1/items?select=id,tipo,titulo,descripcion,etiquetas,es_18,ruta_publica,sha256,bytes,descargas,version,autor,autor_nombre,icono_publico,vista_publica,corazones&estado=eq.aprobado&order=creado_en.desc&limit=60");
    if let Some(f) = filtro.filter(|f| val::TIPOS.contains(f)) {
        q.push_str(&format!("&tipo=eq.{}", f));
    }
    if !con_18 {
        q.push_str("&es_18=eq.false");
    }
    if let Some(ids) = solo_de {
        let validos: Vec<&str> = ids.iter().map(|i| i.as_str()).filter(|i| es_uuid(i)).collect();
        q.push_str(&format!("&autor=in.({})", validos.join(",")));
    }
    let b = limpiar_busqueda(busqueda);
    if !b.is_empty() {
        q.push_str(&format!("&or=({})", codificar(&format!("titulo.ilike.*{0}*,descripcion.ilike.*{0}*", b))));
    }
    let mut items: Vec<Item> = pedir("GET", &q, sesion, None, None, &[], 2 << 20)?.ok()?.json()?;
    items.retain(|i| es_uuid(&i.id) && es_uuid(&i.autor) && val::TIPOS.contains(&i.tipo.as_str()));
    Ok(items)
}

const COLUMNAS_ITEM: &str = "id,tipo,titulo,descripcion,etiquetas,es_18,ruta_publica,sha256,bytes,descargas,version,autor,autor_nombre,icono_publico,vista_publica,corazones";

fn cargar_perfil_publico(sesion: Option<&Sesion>, uid: &str) -> Result<(PerfilPublico, Vec<Item>), String> {
    if !es_uuid(uid) {
        return Err("Creador no válido.".into());
    }
    let p: Vec<PerfilPublico> = pedir("GET", &format!("/rest/v1/perfiles_publicos?select=id,nombre_publico,bio,github,enlaces,avatar_publico,banner_publico,seguidores,corazones_recibidos&id=eq.{}", uid), sesion, None, None, &[], 1 << 18)?.ok()?.json()?;
    let p = p.into_iter().next().ok_or("Este creador aún no tiene un perfil público.")?;
    let mut items: Vec<Item> = pedir("GET", &format!("/rest/v1/items?select={}&estado=eq.aprobado&autor=eq.{}&order=creado_en.desc&limit=60", COLUMNAS_ITEM, uid), sesion, None, None, &[], 2 << 20)?.ok()?.json()?;
    items.retain(|i| es_uuid(&i.id) && val::TIPOS.contains(&i.tipo.as_str()));
    Ok((p, items))
}

fn cargar_mi_perfil(s: &Sesion) -> Option<PerfilPublico> {
    let r = pedir("GET", &format!("/rest/v1/perfiles_publicos?select=*&id=eq.{}", s.uid), Some(s), None, None, &[], 1 << 18).ok()?.ok().ok()?;
    r.json::<Vec<PerfilPublico>>().ok()?.into_iter().next()
}

/// Crops a photo to the profile format (avatar 1:1, banner 3:1), shrinks it and converts it to PNG within the limits.
fn preparar_imagen_perfil(clase: &str, bytes: &[u8]) -> Result<Vec<u8>, String> {
    if bytes.len() > 25 * 1024 * 1024 {
        return Err("La imagen pesa demasiado (máximo 25 MB).".into());
    }
    if let Some(r) = val::detectar(bytes) {
        if r.starts_with("ejecutable") || r == "zip" || r.starts_with("script") {
            return Err(format!("Contenido peligroso detectado ({}).", r));
        }
    }
    let mut lector = image::ImageReader::new(std::io::Cursor::new(bytes)).with_guessed_format().map_err(|_| "No se pudo leer la imagen.".to_string())?;
    let mut limites = image::Limits::default();
    limites.max_image_width = Some(12_000);
    limites.max_image_height = Some(12_000);
    limites.max_alloc = Some(512 * 1024 * 1024);
    lector.limits(limites);
    let img = lector.decode().map_err(|_| "No se pudo leer la imagen (usa PNG, JPEG, WebP, GIF o BMP).".to_string())?;
    let (w, h) = (img.width().max(1), img.height().max(1));
    let proporcion = if clase == "avatar" { 1.0 } else { 3.0 };
    let (cw, ch) = if (w as f32 / h as f32) > proporcion { (((h as f32) * proporcion) as u32, h) } else { (w, ((w as f32) / proporcion) as u32) };
    let recorte = img.crop_imm((w - cw.min(w)) / 2, (h - ch.min(h)) / 2, cw.min(w).max(1), ch.min(h).max(1));
    let tamanos: &[(u32, u32)] = if clase == "avatar" { &[(400, 400), (320, 320), (256, 256)] } else { &[(1500, 500), (1200, 400), (900, 300)] };
    for &(tw, th) in tamanos {
        let r = recorte.resize_exact(tw, th, image::imageops::FilterType::Lanczos3);
        let mut out = std::io::Cursor::new(Vec::new());
        r.write_to(&mut out, image::ImageFormat::Png).map_err(|e| e.to_string())?;
        let b = out.into_inner();
        if val::validar_png(clase, &b).is_ok() {
            return Ok(b);
        }
    }
    Err("No se pudo reducir la imagen al tamaño permitido.".into())
}

/// Uploads a profile PNG image (avatar or banner) to quarantine and asks the server to moderate it.
fn subir_imagen_perfil(s: &Sesion, clase: &str, bytes: &[u8]) -> Result<String, String> {
    val::validar_png(clase, bytes)?;
    let ruta = format!("{}/perfil/{}.png", s.uid, clase);
    let _ = pedir("DELETE", &format!("/storage/v1/object/cuarentena/{}", ruta), Some(s), None, None, &[], 1 << 16); // replaces the previous one
    pedir("POST", &format!("/storage/v1/object/cuarentena/{}", ruta), Some(s), Some("image/png"), Some(bytes), &[("x-upsert", "false")], 1 << 16)?.ok()?;
    let r = json_post("/functions/v1/perfil", Some(s), &serde_json::json!({ "accion": "imagen", "clase": clase }), &[])?;
    let v: serde_json::Value = r.json().unwrap_or_default();
    if !(200..300).contains(&r.estado) {
        return Err(v["error"].as_str().unwrap_or("No se pudo subir la imagen.").to_string());
    }
    Ok(match v["estado"].as_str().unwrap_or("") {
        "publicado" => "Imagen publicada en tu perfil.".to_string(),
        _ => v["motivo"].as_str().unwrap_or("Imagen enviada a revisión.").to_string(),
    })
}

/// My hearts and my follows (only what is mine: under RLS nobody else sees it).
fn cargar_mios(s: &Sesion) -> Option<(Vec<String>, Vec<String>)> {
    let col = |ruta: &str, campo: &str| -> Option<Vec<String>> {
        let v: Vec<serde_json::Value> = pedir("GET", ruta, Some(s), None, None, &[], 1 << 20).ok()?.ok().ok()?.json().ok()?;
        Some(v.iter().filter_map(|f| f[campo].as_str().filter(|i| es_uuid(i)).map(|i| i.to_string())).collect())
    };
    Some((col(&format!("/rest/v1/corazones?select=item&persona=eq.{}&limit=1000", s.uid), "item")?, col(&format!("/rest/v1/seguidores?select=seguido&seguidor=eq.{}&limit=1000", s.uid), "seguido")?))
}

fn cargar_perfil(s: &Sesion) -> Option<Perfil> {
    let r = pedir("GET", &format!("/rest/v1/perfiles?select=nombre_publico,mostrar_18&id=eq.{}", s.uid), Some(s), None, None, &[], 1 << 16).ok()?.ok().ok()?;
    r.json::<Vec<Perfil>>().ok()?.into_iter().next()
}

fn descargar(item: &Item, sesion: Option<&Sesion>) -> Result<Vec<u8>, String> {
    let ruta = item.ruta_publica.as_deref().ok_or("Este contenido no tiene archivo.")?;
    if !ruta_publica_valida(ruta) {
        return Err("Ruta de archivo no válida.".into());
    }
    let limite = val::limite_bytes(&item.tipo);
    let r = pedir("GET", &format!("/storage/v1/object/public/publico/{}", ruta), None, None, None, &[], limite + 1)?.ok()?;
    let bytes = r.cuerpo;
    if bytes.len() > limite {
        return Err("El archivo supera el tamaño permitido.".into());
    }
    let esperado = item.sha256.as_deref().ok_or("El contenido no trae huella de verificación.")?;
    if val::sha256_hex(&bytes) != esperado {
        return Err("La huella SHA-256 no coincide: el archivo no es el que se aprobó. No se instaló.".into());
    }
    let nombre = ruta.rsplit('/').next().unwrap_or("");
    val::validar(&item.tipo, nombre, &bytes).map_err(|e| format!("No pasó la validación local: {}", e))?;
    if let Some(s) = sesion {
        let _ = json_post("/rest/v1/rpc/registrar_descarga", Some(s), &serde_json::json!({ "p_item": item.id }), &[]); // only counts when signed in
    }
    Ok(bytes)
}

fn subir_contenido(s: &Sesion, tipo: &str, titulo: &str, descripcion: &str, etiquetas: Vec<String>, es_18: bool, nombre: &str, bytes: &[u8], extras: &[(&str, Vec<u8>)], tx: &Sender<Mensaje>) -> Result<String, String> {
    val::validar(tipo, nombre, bytes)?;
    for (clase, b) in extras {
        val::validar_png(clase, b).map_err(|e| format!("{} ({})", e, clase))?;
    }
    let s = vigente(s, tx)?;
    let id = uuid_v4();
    let archivo = nombre_seguro(nombre);
    let ruta = format!("{}/{}/{}", s.uid, id, archivo);
    let fila = serde_json::json!({
        "id": id, "autor": s.uid, "tipo": tipo, "titulo": titulo, "descripcion": descripcion,
        "etiquetas": etiquetas, "es_18": es_18, "version": "1.0.0", "ruta_cuarentena": ruta,
        "icono_cuarentena": extras.iter().any(|(c, _)| *c == "icono").then(|| format!("{}/{}/icono.png", s.uid, id)),
        "vista_cuarentena": extras.iter().any(|(c, _)| *c == "vista").then(|| format!("{}/{}/vista.png", s.uid, id)),
    });
    json_post("/rest/v1/items", Some(&s), &fila, &[("Prefer", "return=minimal")])?.ok()?;
    let mime = match archivo.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ttf" => "font/ttf",
        "otf" => "font/otf",
        _ => "application/json",
    };
    let mut subida = pedir("POST", &format!("/storage/v1/object/cuarentena/{}", ruta), Some(&s), Some(mime), Some(bytes), &[("x-upsert", "false")], 1 << 20).and_then(|r| r.ok());
    for (clase, b) in extras {
        if subida.is_ok() {
            subida = pedir("POST", &format!("/storage/v1/object/cuarentena/{}/{}/{}.png", s.uid, id, clase), Some(&s), Some("image/png"), Some(b), &[("x-upsert", "false")], 1 << 20).and_then(|r| r.ok());
        }
    }
    if let Err(e) = subida {
        let _ = pedir("DELETE", &format!("/rest/v1/items?id=eq.{}", id), Some(&s), None, None, &[], 1 << 16); // don't leave a listing without a file
        return Err(e);
    }
    let r = json_post("/functions/v1/moderar", Some(&s), &serde_json::json!({ "item": id }), &[])?.ok()?;
    let v: serde_json::Value = r.json()?;
    let motivo = v["motivo"].as_str().unwrap_or("");
    Ok(match v["estado"].as_str().unwrap_or("") {
        "aprobado" => "Publicado en la comunidad.".to_string(),
        "rechazado" => format!("Rechazado: {}", motivo),
        _ => format!("Enviado a revisión: {}", motivo),
    })
}

// ───────────────────────────── installation (UI thread) ─────────────────────────────

fn escribir_temporal(nombre: &str, bytes: &[u8]) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join("lienzo-ui-comunidad");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let f = dir.join(nombre_seguro(nombre));
    std::fs::write(&f, bytes).map_err(|e| e.to_string())?;
    Ok(f)
}

fn instalar_plugin(bytes: &[u8]) -> Result<String, String> {
    let p: Paquete = val::leer_paquete(bytes)?;
    let nombre = p.manifiesto["nombre"].as_str().unwrap_or("plugin");
    let carpeta = crate::plugins::carpeta_usuario().join(format!("comunidad-{}", nombre_seguro(nombre).to_lowercase()));
    std::fs::create_dir_all(&carpeta).map_err(|e| e.to_string())?;
    for (ruta, contenido) in &p.archivos {
        let destino = carpeta.join(ruta); // validated: no "..", no absolute paths, only .rhai/.json
        if let Some(padre) = destino.parent() {
            std::fs::create_dir_all(padre).map_err(|e| e.to_string())?;
        }
        std::fs::write(&destino, contenido).map_err(|e| e.to_string())?;
    }
    if !p.archivos.contains_key("plugin.json") {
        std::fs::write(carpeta.join("plugin.json"), serde_json::to_string_pretty(&p.manifiesto).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    }
    Ok(nombre.to_string())
}

/// Turns a plugin folder into the package that gets uploaded.
fn empaquetar_plugin(carpeta: &std::path::Path) -> Result<Vec<u8>, String> {
    let manifiesto: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(carpeta.join("plugin.json")).map_err(|_| "La carpeta no tiene plugin.json.")?).map_err(|_| "plugin.json no es válido.")?;
    let mut archivos = std::collections::BTreeMap::new();
    archivos.insert("plugin.json".to_string(), serde_json::to_string(&manifiesto).map_err(|e| e.to_string())?);
    for clave in ["comandos", "exportadores"] {
        for x in manifiesto.get(clave).and_then(|v| v.as_array()).into_iter().flatten() {
            if let Some(s) = x.get("script").and_then(|s| s.as_str()) {
                let f = carpeta.join(s);
                if !f.starts_with(carpeta) || s.contains("..") {
                    return Err(format!("Ruta no permitida: {}", s));
                }
                archivos.insert(s.to_string(), std::fs::read_to_string(&f).map_err(|_| format!("No se pudo leer {}.", s))?);
            }
        }
    }
    for pl in manifiesto.get("plantillas").and_then(|v| v.as_array()).into_iter().flatten().filter_map(|p| p.as_str()) {
        if pl.contains("..") {
            return Err(format!("Ruta no permitida: {}", pl));
        }
        archivos.insert(pl.to_string(), std::fs::read_to_string(carpeta.join(pl)).map_err(|_| format!("No se pudo leer {}.", pl))?);
    }
    serde_json::to_vec(&Paquete { manifiesto, archivos }).map_err(|e| e.to_string())
}

// ───────────────────────────── UI ─────────────────────────────

const NOMBRES_TIPO: &[(&str, &str)] = &[("plugin", "Plugins"), ("imagen", "Imágenes"), ("fuente", "Fuentes"), ("plantilla", "Plantillas"), ("elemento", "Elementos"), ("tema", "Temas")];
const MOTIVOS: &[(&str, &str)] = &[("malware", "Malware o virus"), ("contenido_18", "Contenido +18 sin marcar"), ("odio_o_acoso", "Odio o acoso"), ("derechos_de_autor", "Derechos de autor"), ("otro", "Otro")];

fn nombre_tipo(t: &str) -> &'static str {
    NOMBRES_TIPO.iter().find(|(k, _)| *k == t).map(|(_, n)| *n).unwrap_or("")
}

impl EditorApp {
    fn com_lanzar(&mut self, ctx: &egui::Context, f: impl FnOnce(&Sender<Mensaje>) + Send + 'static) {
        self.comunidad.ocupado += 1;
        let tx = self.comunidad.tx.clone();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            f(&tx);
            ctx.request_repaint();
        });
    }

    fn com_cargar_lista(&mut self, ctx: &egui::Context) {
        let (s, f, b) = (self.comunidad.sesion.clone(), self.comunidad.filtro, self.comunidad.busqueda.clone());
        let con_18 = self.comunidad.perfil.as_ref().map(|p| p.mostrar_18).unwrap_or(false) && s.is_some();
        let solo_de: Option<Vec<String>> = (self.comunidad.solo_siguiendo && s.is_some()).then(|| self.comunidad.mis_seguidos.iter().cloned().collect());
        self.com_lanzar(ctx, move |tx| {
            let s = s.and_then(|s| vigente(&s, tx).ok());
            let _ = tx.send(Mensaje::Items(listar(s.as_ref(), f, &b, con_18, solo_de.as_deref())));
        });
    }

    fn com_tras_sesion(&mut self, ctx: &egui::Context, s: Sesion) {
        self.comunidad.sesion = Some(s.clone());
        guardar_credencial(Some(&s));
        self.com_lanzar(ctx, move |tx| {
            let _ = tx.send(Mensaje::Perfil(cargar_perfil(&s)));
        });
    }

    /// Processes what the threads have finished. Called every frame.
    pub fn comunidad_poll(&mut self, ctx: &egui::Context) {
        while let Ok(m) = self.comunidad.rx.try_recv() {
            let is_sesion = matches!(m, Mensaje::Sesion(_));
            match m {
                Mensaje::Sesion(r) | Mensaje::Login(r) => {
                    if is_sesion {
                        self.comunidad.ocupado = self.comunidad.ocupado.saturating_sub(1);
                    } else {
                        self.comunidad.login = None;
                    }
                    match r {
                        Ok(s) => {
                            self.com_tras_sesion(ctx, s);
                            self.comunidad.estado = t("Sesión iniciada.");
                        }
                        Err(e) => {
                            self.comunidad.estado = e;
                            self.comunidad.sesion = None;
                        }
                    }
                }
                Mensaje::SesionRenovada(s) => self.comunidad.sesion = Some(s),
                Mensaje::Perfil(p) => {
                    self.comunidad.ocupado = self.comunidad.ocupado.saturating_sub(1);
                    self.comunidad.perfil_cargado = true;
                    if p.is_some() {
                        if let Some(s) = self.comunidad.sesion.clone() {
                            let (tx, c2) = (self.comunidad.tx.clone(), ctx.clone());
                            std::thread::spawn(move || {
                                let _ = tx.send(Mensaje::MiPerfil(cargar_mi_perfil(&s)));
                                if let Some((corazones, seguidos)) = cargar_mios(&s) {
                                    let _ = tx.send(Mensaje::Mios { corazones, seguidos });
                                }
                                c2.request_repaint();
                            });
                        }
                    }
                    self.comunidad.perfil = p;
                    self.com_cargar_lista(ctx);
                }
                Mensaje::Items(r) => {
                    self.comunidad.ocupado = self.comunidad.ocupado.saturating_sub(1);
                    match r {
                        Ok(v) => self.comunidad.items = v,
                        Err(e) => self.comunidad.estado = e,
                    }
                }
                Mensaje::Descargado { item, bytes } => {
                    self.comunidad.ocupado = self.comunidad.ocupado.saturating_sub(1);
                    self.comunidad.estado = match bytes.and_then(|b| self.com_instalar(ctx, &item, &b)) {
                        Ok(m) => m,
                        Err(e) => e,
                    };
                }
                Mensaje::Subido(r) => {
                    self.comunidad.ocupado = self.comunidad.ocupado.saturating_sub(1);
                    self.comunidad.estado = r.unwrap_or_else(|e| e);
                    self.com_cargar_lista(ctx);
                }
                Mensaje::Reportado(r) => {
                    self.comunidad.ocupado = self.comunidad.ocupado.saturating_sub(1);
                    self.comunidad.estado = match r {
                        Ok(()) => t("Reporte enviado. Gracias."),
                        Err(e) => e,
                    };
                }
                Mensaje::CuentaBorrada(r) => {
                    self.comunidad.ocupado = self.comunidad.ocupado.saturating_sub(1);
                    match r {
                        Ok(()) => {
                            self.com_cerrar_local();
                            self.comunidad.estado = t("Tu cuenta y tu contenido se borraron.");
                        }
                        Err(e) => self.comunidad.estado = e,
                    }
                }
                Mensaje::Imagen { ruta, bytes } => {
                    let tex = bytes.and_then(|b| {
                        val::validar_png("vista", &b).or_else(|_| val::validar_png("banner", &b)).ok()?;
                        let img = image::load_from_memory_with_format(&b, image::ImageFormat::Png).ok()?.to_rgba8();
                        let (w, h) = img.dimensions();
                        let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], img.as_raw());
                        // small images (pixel art) are shown sharp
                        let opciones = if w <= 128 && h <= 128 { egui::TextureOptions::NEAREST } else { egui::TextureOptions::LINEAR };
                        Some(ctx.load_texture(format!("comunidad/{}", ruta), ci, opciones))
                    });
                    self.comunidad.texturas.insert(ruta, tex);
                }
                Mensaje::PerfilVer(r) => {
                    self.comunidad.ocupado = self.comunidad.ocupado.saturating_sub(1);
                    match r {
                        Ok(v) => self.comunidad.viendo = Some(v),
                        Err(e) => self.comunidad.estado = e,
                    }
                }
                Mensaje::Mios { corazones, seguidos } => {
                    self.comunidad.mis_corazones = corazones.into_iter().collect();
                    self.comunidad.mis_seguidos = seguidos.into_iter().collect();
                    if self.comunidad.solo_siguiendo {
                        self.com_cargar_lista(ctx);
                    }
                }
                Mensaje::MiPerfil(p) => {
                    if let Some(p) = &p {
                        if !self.comunidad.form_perfil.cargado {
                            let f = &mut self.comunidad.form_perfil;
                            f.nombre = p.nombre_publico.clone();
                            f.bio = p.bio.clone();
                            f.github = p.github.clone().unwrap_or_default();
                            f.enlaces = REDES.iter().map(|(k, _, _)| p.enlaces.get(*k).cloned().unwrap_or_default()).collect();
                            f.cargado = true;
                        }
                    }
                    if let (Some(n), Some(pr)) = (p.as_ref().map(|x| x.nombre_publico.clone()), self.comunidad.perfil.as_mut()) {
                        pr.nombre_publico = n;
                    }
                    // the image under review may have changed: request it again
                    self.comunidad.texturas.retain(|k, _| !k.starts_with("cuarentena:"));
                    self.comunidad.mi_perfil = p;
                }
                Mensaje::Estado(e) => {
                    self.comunidad.ocupado = self.comunidad.ocupado.saturating_sub(1);
                    self.comunidad.estado = e;
                }
            }
        }
    }

    fn com_cerrar_local(&mut self) {
        self.comunidad.sesion = None;
        self.comunidad.perfil = None;
        self.comunidad.perfil_cargado = false;
        self.comunidad.borrar_confirmado = false;
        self.comunidad.mi_perfil = None;
        self.comunidad.form_perfil = FormPerfil::default();
        self.comunidad.previas.clear();
        self.comunidad.mis_corazones.clear();
        self.comunidad.mis_seguidos.clear();
        self.comunidad.solo_siguiendo = false;
        self.comunidad.perfil_abierto = false;
        guardar_credencial(None);
    }

    fn com_instalar(&mut self, ctx: &egui::Context, item: &Item, bytes: &[u8]) -> Result<String, String> {
        let nombre = item.ruta_publica.as_deref().and_then(|r| r.rsplit('/').next()).unwrap_or("archivo");
        val::validar(&item.tipo, nombre, bytes)?; // second check right before touching the disk
        let msg = match item.tipo.as_str() {
            "plugin" => {
                let n = instalar_plugin(bytes)?;
                self.plugins = crate::plugins::cargar_todos();
                format!("{} «{}» (pestaña Plugins)", t("Plugin instalado:"), n)
            }
            "imagen" => {
                let f = escribir_temporal(nombre, bytes)?;
                let r = self.library.add_image(&f);
                let _ = std::fs::remove_file(&f);
                format!("{} «{}» (pestaña Imágenes)", t("Imagen instalada:"), r?)
            }
            "fuente" => {
                let f = escribir_temporal(nombre, bytes)?;
                let r = crate::library::add_font_file(&f);
                let _ = std::fs::remove_file(&f);
                let (n, datos, ruta) = r?;
                crate::render::add_font(ctx, &n, datos, ruta);
                format!("{} «{}» (pestaña Texto)", t("Fuente instalada:"), n)
            }
            "tema" => {
                let tema: crate::theme::Tema = serde_json::from_slice(bytes).map_err(|_| "El tema no es válido.".to_string())?;
                tema.validar()?;
                crate::temas::guardar(&tema)?;
                let n = tema.nombre.clone();
                self.usar_tema(ctx, tema);
                format!("{} «{}» (botón Temas para cambiarlo)", t("Tema instalado y en uso:"), n)
            }
            "plantilla" => {
                let f = escribir_temporal(&format!("{}.json", nombre_seguro(&item.titulo)), bytes)?;
                let r = self.library.add_template_file(&f);
                let _ = std::fs::remove_file(&f);
                format!("{} «{}» (Plantillas › Mis plantillas)", t("Plantilla instalada:"), r?)
            }
            _ => {
                let f = escribir_temporal(&format!("{}.json", nombre_seguro(&item.titulo)), bytes)?;
                let r = self.library.add_element_file(&f);
                let _ = std::fs::remove_file(&f);
                format!("{} «{}» (pestaña Elementos)", t("Elemento instalado:"), r?)
            }
        };
        Ok(msg)
    }

    pub fn tab_comunidad(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.comunidad_poll(&ctx);
        if !self.comunidad.iniciado {
            // First time the tab is opened: any connection starts here (and not before).
            self.comunidad.iniciado = true;
            if let Some((r, u)) = leer_credencial() {
                self.com_lanzar(&ctx, move |tx| {
                    let _ = tx.send(Mensaje::Sesion(renovar(&Sesion { renovacion: r, uid: u, ..Default::default() })));
                });
            } else {
                self.com_cargar_lista(&ctx);
            }
        }

        ui.horizontal(|ui| {
            ui.label(RichText::new(t("Comunidad")).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.comunidad.ocupado > 0 {
                    ui.spinner();
                } else if ui.small_button(t("Actualizar")).clicked() {
                    self.com_cargar_lista(&ctx);
                }
            });
        });
        ui.label(RichText::new(t("Plugins, imágenes, fuentes, plantillas y elementos compartidos. Todo se revisa antes de publicarse.")).small().color(MUTED()));
        if !self.comunidad.estado.is_empty() {
            ui.add_space(4.0);
            ui.label(RichText::new(&self.comunidad.estado).small().color(ACCENT()));
        }
        ui.add_space(6.0);
        self.com_cuenta(ui, &ctx);
        ui.add_space(6.0);
        self.com_explorar(ui, &ctx);
        if self.comunidad.sesion.is_some() && self.comunidad.perfil.is_some() {
            ui.add_space(8.0);
            self.com_subir(ui, &ctx);
        }
    }

    fn com_cuenta(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::Frame::new().fill(crate::theme::PANEL_DARK()).corner_radius(6.0).inner_margin(8.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            if self.comunidad.sesion.is_none() {
                ui.label(RichText::new(t("Inicia sesión para subir, reportar y activar el contenido +18. Explorar no necesita cuenta.")).small());
                ui.label(RichText::new(t("Sin contraseñas: entras con tu cuenta de GitHub o Google. La app solo guarda un nombre público.")).small().color(MUTED()));
                ui.horizontal(|ui| {
                    for (clave, nombre) in [("github", "GitHub"), ("google", "Google")] {
                        if ui.add_enabled(self.comunidad.login.is_none(), egui::Button::new(format!("{} {}", t("Entrar con"), nombre))).clicked() {
                            self.comunidad.estado = t("Se abrió tu navegador: termina el inicio de sesión allí.");
                            let cancelar = Arc::new(AtomicBool::new(false));
                            self.comunidad.login = Some(cancelar.clone());
                            let (tx, ctx2) = (self.comunidad.tx.clone(), ctx.clone());
                            std::thread::spawn(move || {
                                let _ = tx.send(Mensaje::Login(flujo_oauth(clave, &cancelar)));
                                ctx2.request_repaint();
                            });
                        }
                    }
                });
                if let Some(c) = &self.comunidad.login {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label(RichText::new(t("Esperando el inicio de sesión en tu navegador…")).small().color(MUTED()));
                        if ui.small_button(t("Cancelar")).clicked() {
                            c.store(true, Ordering::Relaxed);
                        }
                    });
                }
                return;
            }
            let Some(perfil) = self.comunidad.perfil.clone() else {
                if !self.comunidad.perfil_cargado {
                    ui.label(RichText::new(t("Cargando tu perfil…")).small().color(MUTED()));
                    return;
                }
                // First time: choose a public name and accept the rules.
                ui.label(RichText::new(t("Elige tu nombre público")).strong());
                ui.label(RichText::new(t("Es lo único que verá la comunidad. 3-32 caracteres: letras, números, punto, guion y guion bajo.")).small().color(MUTED()));
                ui.add(egui::TextEdit::singleline(&mut self.comunidad.nombre_nuevo).hint_text("nombre_publico").desired_width(f32::INFINITY));
                ui.checkbox(&mut self.comunidad.acepto, t("Acepto las reglas: nada de malware, odio, plagio ni contenido +18 sin marcar"));
                let n = self.comunidad.nombre_nuevo.trim().to_string();
                let valido = (3..=32).contains(&n.chars().count()) && n.chars().all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c));
                if ui.add_enabled(valido && self.comunidad.acepto && self.comunidad.ocupado == 0, egui::Button::new(t("Crear perfil"))).clicked() {
                    let s = self.comunidad.sesion.clone().unwrap();
                    self.com_lanzar(ctx, move |tx| {
                        let r = vigente(&s, tx).and_then(|s| json_post("/rest/v1/perfiles", Some(&s), &serde_json::json!({ "id": s.uid, "nombre_publico": n, "acepto_terminos_en": iso(ahora()) }), &[("Prefer", "return=minimal")])?.ok().map(|_| s));
                        match r {
                            Ok(s) => {
                                let _ = tx.send(Mensaje::Perfil(cargar_perfil(&s)));
                            }
                            Err(e) => {
                                let _ = tx.send(Mensaje::Estado(e));
                            }
                        }
                    });
                }
                if ui.small_button(t("Cerrar sesión")).clicked() {
                    self.com_cerrar_local();
                }
                return;
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{} {}", t("Sesión:"), perfil.nombre_publico)).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(t("Cerrar sesión")).clicked() {
                        let s = self.comunidad.sesion.clone();
                        self.com_cerrar_local();
                        if let Some(s) = s {
                            std::thread::spawn(move || {
                                let _ = pedir("POST", "/auth/v1/logout?scope=local", Some(&s), None, None, &[], 1 << 16);
                            });
                        }
                        self.comunidad.estado = t("Sesión cerrada.");
                    }
                });
            });
            let mut m18 = perfil.mostrar_18;
            if !perfil.mostrar_18 {
                ui.checkbox(&mut self.comunidad.mayor_edad, t("Confirmo que tengo 18 años o más"));
                if self.comunidad.mayor_edad && ui.small_button(t("Mostrar contenido +18")).clicked() {
                    m18 = true;
                }
            } else if ui.small_button(t("Ocultar contenido +18")).clicked() {
                m18 = false;
            }
            if m18 != perfil.mostrar_18 {
                let s = self.comunidad.sesion.clone().unwrap();
                if let Some(p) = self.comunidad.perfil.as_mut() {
                    p.mostrar_18 = m18;
                }
                self.com_lanzar(ctx, move |tx| {
                    let r = vigente(&s, tx).and_then(|s| pedir("PATCH", &format!("/rest/v1/perfiles?id=eq.{}", s.uid), Some(&s), None, Some(if m18 { serde_json::json!({ "mostrar_18": true, "declara_mayor_18_en": iso(ahora()) }) } else { serde_json::json!({ "mostrar_18": false }) }.to_string().as_bytes()), &[], 1 << 16)?.ok().map(|_| ()));
                    let _ = tx.send(Mensaje::Estado(match r {
                        Ok(()) => String::new(),
                        Err(e) => e,
                    }));
                });
                self.com_cargar_lista(ctx);
            }
            if ui.button(t("Editar mi perfil de creador")).clicked() {
                self.comunidad.perfil_abierto = true;
            }
            ui.collapsing(t("Mi cuenta y mis datos"), |ui| {
                ui.label(RichText::new(t("Guardamos solo tu nombre público, tu preferencia +18 y lo que subes. Tu correo lo gestiona el proveedor de inicio de sesión y no se muestra a nadie.")).small().color(MUTED()));
                ui.checkbox(&mut self.comunidad.borrar_confirmado, t("Quiero borrar mi cuenta y todo lo que subí (no se puede deshacer)"));
                if ui.add_enabled(self.comunidad.borrar_confirmado && self.comunidad.ocupado == 0, egui::Button::new(RichText::new(t("Borrar mi cuenta")).color(WARN()))).clicked() {
                    let s = self.comunidad.sesion.clone().unwrap();
                    self.com_lanzar(ctx, move |tx| {
                        let r = vigente(&s, tx).and_then(|s| json_post("/functions/v1/borrar-cuenta", Some(&s), &serde_json::json!({}), &[])?.ok().map(|_| ()));
                        let _ = tx.send(Mensaje::CuentaBorrada(r));
                    });
                }
            });
        });
    }

    fn com_explorar(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        if self.comunidad.viendo.is_some() {
            self.com_ver_perfil(ui, ctx);
            return;
        }
        let mut recargar = false;
        ui.horizontal_wrapped(|ui| {
            if ui.selectable_label(self.comunidad.filtro.is_none(), t("Todo")).clicked() {
                self.comunidad.filtro = None;
                recargar = true;
            }
            for (k, n) in NOMBRES_TIPO {
                if ui.selectable_label(self.comunidad.filtro == Some(*k), t(n)).clicked() {
                    self.comunidad.filtro = Some(k);
                    recargar = true;
                }
            }
            if self.comunidad.sesion.is_some() && ui.add(crate::icons::btn("heart", t("Siguiendo")).selected(self.comunidad.solo_siguiendo)).on_hover_text(t("Solo contenido de los creadores que sigues")).clicked() {
                self.comunidad.solo_siguiendo = !self.comunidad.solo_siguiendo;
                recargar = true;
            }
        });
        let r = ui.add(egui::TextEdit::singleline(&mut self.comunidad.busqueda).hint_text(t("Buscar en la comunidad…")).desired_width(f32::INFINITY));
        if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            recargar = true;
        }
        if recargar {
            self.com_cargar_lista(ctx);
        }
        ui.add_space(4.0);
        if self.comunidad.items.is_empty() {
            ui.label(RichText::new(if self.comunidad.ocupado > 0 { t("Cargando…") } else { t("No hay contenido que mostrar todavía.") }).color(MUTED()));
            return;
        }
        let items = self.comunidad.items.clone();
        let logueado = self.comunidad.sesion.is_some() && self.comunidad.perfil.is_some();
        for it in items {
            self.com_tarjeta(ui, ctx, &it, logueado);
            ui.add_space(4.0);
        }
    }

    fn com_subir(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let abrir = std::mem::take(&mut self.comunidad.abrir_subida);
        egui::CollapsingHeader::new(RichText::new(t("Compartir con la comunidad")).strong()).id_salt("com_subir").open(abrir.then_some(true)).show(ui, |ui| {
            ui.label(RichText::new(t("Se revisa antes de publicarse (formato, antivirus y moderación). Si es +18, márcalo.")).small().color(MUTED()));
            egui::ComboBox::from_id_salt("com_tipo").selected_text(t(NOMBRES_TIPO[self.comunidad.subir.tipo].1)).show_ui(ui, |ui| {
                for (i, (_, n)) in NOMBRES_TIPO.iter().enumerate() {
                    ui.selectable_value(&mut self.comunidad.subir.tipo, i, t(n));
                }
            });
            let tipo = NOMBRES_TIPO[self.comunidad.subir.tipo].0;
            ui.add(egui::TextEdit::singleline(&mut self.comunidad.subir.titulo).hint_text(t("Título (3-80 caracteres)")).desired_width(f32::INFINITY));
            ui.add(egui::TextEdit::multiline(&mut self.comunidad.subir.descripcion).hint_text(t("Descripción (máx. 1000)")).desired_rows(3).desired_width(f32::INFINITY));
            ui.add(egui::TextEdit::singleline(&mut self.comunidad.subir.etiquetas).hint_text(t("Etiquetas separadas por coma (máx. 8)")).desired_width(f32::INFINITY));
            ui.checkbox(&mut self.comunidad.subir.es_18, t("Contiene contenido +18"));
            ui.horizontal(|ui| {
                let boton = if tipo == "plugin" { t("Elegir carpeta del plugin…") } else { t("Elegir archivo…") };
                if ui.button(boton).clicked() {
                    self.comunidad.subir.archivo = match tipo {
                        "plugin" => rfd::FileDialog::new().set_title("Carpeta del plugin (con plugin.json)").pick_folder(),
                        "imagen" => rfd::FileDialog::new().add_filter("Imágenes", &["png", "jpg", "jpeg", "gif", "webp"]).pick_file(),
                        "fuente" => rfd::FileDialog::new().add_filter("Fuentes", &["ttf", "otf"]).pick_file(),
                        _ => rfd::FileDialog::new().add_filter("JSON", &["json"]).pick_file(),
                    };
                }
                if tipo == "tema" && ui.button(t("Usar el tema actual")).clicked() {
                    let tema = self.settings.tema.clone().unwrap_or_default();
                    let f = std::env::temp_dir().join("lienzo-ui-comunidad");
                    let _ = std::fs::create_dir_all(&f);
                    let f = f.join("tema-actual.json");
                    if std::fs::write(&f, serde_json::to_vec_pretty(&tema).unwrap_or_default()).is_ok() {
                        self.comunidad.subir.archivo = Some(f);
                        if self.comunidad.subir.titulo.trim().is_empty() {
                            self.comunidad.subir.titulo = tema.nombre.clone();
                        }
                    }
                }
                if tipo == "plantilla" && ui.button(t("Usar el diseño abierto")).clicked() {
                    let f = std::env::temp_dir().join("lienzo-ui-comunidad");
                    let _ = std::fs::create_dir_all(&f);
                    let f = f.join("diseno-actual.json");
                    if std::fs::write(&f, serde_json::to_vec(&self.doc).unwrap_or_default()).is_ok() {
                        self.comunidad.subir.archivo = Some(f);
                    }
                }
            });
            if let Some(a) = &self.comunidad.subir.archivo {
                ui.label(RichText::new(a.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()).small());
            }
            ui.label(RichText::new(t("Opcional: vista previa (PNG hasta 1920×1080). El icono del contenido lo pone la app según su tipo.")).small().color(MUTED()));
            ui.horizontal(|ui| {
                let (icono, texto) = match &self.comunidad.subir.vista {
                    Some(p) => ("check", format!("{} {}", t("Vista previa…"), p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default())),
                    None => ("image", t("Vista previa…")),
                };
                if ui.add(crate::icons::btn(icono, texto)).clicked() {
                    self.comunidad.subir.vista = rfd::FileDialog::new().add_filter("PNG", &["png"]).pick_file();
                }
            });
            let f = &self.comunidad.subir;
            let etiquetas: Vec<String> = f.etiquetas.split(',').map(|e| e.trim().to_lowercase()).filter(|e| !e.is_empty()).take(8).collect();
            let listo = (3..=80).contains(&f.titulo.trim().chars().count()) && f.archivo.is_some() && self.comunidad.ocupado == 0;
            if ui.add_enabled(listo, egui::Button::new(t("Enviar a revisión"))).clicked() {
                let f = &self.comunidad.subir;
                let (titulo, desc, es_18, ruta) = (f.titulo.trim().to_string(), f.descripcion.chars().take(1000).collect::<String>(), f.es_18, f.archivo.clone().unwrap());
                let lectura = if ruta.is_dir() { empaquetar_plugin(&ruta).map(|b| ("plugin.json".to_string(), b)) } else { std::fs::read(&ruta).map(|b| (ruta.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(), b)).map_err(|e| e.to_string()) };
                let mut extras: Vec<(&'static str, Vec<u8>)> = Vec::new();
                let mut fallo_extra = None;
                for (clase, ruta) in [("vista", self.comunidad.subir.vista.clone())] {
                    if let Some(r) = ruta {
                        match std::fs::read(&r) {
                            Ok(b) => extras.push((clase, b)),
                            Err(e) => fallo_extra = Some(e.to_string()),
                        }
                    }
                }
                let lectura = match fallo_extra {
                    Some(e) => Err(e),
                    None => lectura,
                };
                match lectura {
                    Err(e) => self.comunidad.estado = e,
                    Ok((nombre, bytes)) => {
                        let s = self.comunidad.sesion.clone().unwrap();
                        self.comunidad.estado = t("Enviando y analizando… (puede tardar hasta un minuto)");
                        self.com_lanzar(ctx, move |tx| {
                            let extras: Vec<(&str, Vec<u8>)> = extras.into_iter().collect();
                            let r = subir_contenido(&s, tipo, &titulo, &desc, etiquetas, es_18, &nombre, &bytes, &extras, tx);
                            let _ = tx.send(Mensaje::Subido(r));
                        });
                        self.comunidad.subir = SubirForm { tipo: self.comunidad.subir.tipo, ..Default::default() };
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
    fn fecha_iso() {
        assert_eq!(iso(0), "1970-01-01T00:00:00Z");
        assert_eq!(iso(1_794_865_809), "2026-11-16T21:50:09Z");
    }

    #[test]
    fn uuid_y_nombres() {
        let u = uuid_v4();
        assert!(es_uuid(&u) && u.as_bytes()[14] == b'4');
        assert_eq!(nombre_seguro("../mi plugin (1).json"), "_mi_plugin__1_.json");
    }

    #[test]
    fn codificar_y_busqueda() {
        assert_eq!(codificar("a b/c"), "a%20b%2Fc");
        assert_eq!(limpiar_busqueda("hola,(x)*%"), "holax");
    }

    #[test]
    fn mensajes_sin_filtrar_el_cuerpo() {
        let e = mensaje_error(500, r#"{"message":"clave=SECRETA"}"#);
        assert!(!e.contains("SECRETA"));
        assert!(mensaje_error(401, "").contains("sesión"));
    }

    #[test]
    fn desafio_pkce_conocido() {
        // RFC 7636, appendix B
        let d = b64url(&Sha256::digest(b"dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"));
        assert_eq!(d, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    }

    #[test]
    fn sesion_se_lee_de_la_respuesta() {
        let v = serde_json::json!({ "access_token": "a", "refresh_token": "r", "expires_in": 3600, "user": { "id": "123e4567-e89b-42d3-a456-426614174000" } });
        let s = sesion_desde(&v).unwrap();
        assert_eq!(s.renovacion, "r");
        assert!(sesion_desde(&serde_json::json!({ "access_token": "a", "refresh_token": "r", "user": { "id": "no-es-uuid" } })).is_err());
    }

    #[test]
    fn empaqueta_el_plugin_de_ejemplo() {
        let c = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("plugins/ajustar-cuadricula");
        let b = empaquetar_plugin(&c).unwrap();
        assert!(val::validar("plugin", "p.json", &b).is_ok());
    }
}

/// Upload from the command line, using the session saved by the app (Credential Manager).
///   lienzo-ui --comunidad-subir <tipo> <archivo|carpeta-del-plugin> --titulo "…" [--descripcion "…"] [--etiquetas a,b] [--18]
/// Uses exactly the same code as the "Submit for review" button. Close the app before using it: the session
/// token is renewed and the app, left open with the old token, would lose the session.
pub fn subir_cli(args: &[String]) -> i32 {
    let tipo = args.first().map(|s| s.as_str()).unwrap_or("");
    let ruta = PathBuf::from(args.get(1).map(|s| s.as_str()).unwrap_or(""));
    let opcion = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned().unwrap_or_default();
    let titulo = opcion("--titulo");
    let etiquetas: Vec<String> = opcion("--etiquetas").split(',').map(|e| e.trim().to_lowercase()).filter(|e| !e.is_empty()).take(8).collect();
    let es_18 = args.iter().any(|a| a == "--18");
    if !val::TIPOS.contains(&tipo) || !ruta.exists() || titulo.trim().chars().count() < 3 {
        eprintln!("Uso: lienzo-ui --comunidad-subir <plugin|imagen|fuente|plantilla|elemento> <archivo|carpeta> --titulo \"…\" [--descripcion \"…\"] [--etiquetas a,b] [--18] [--icono icono.png] [--vista vista.png]");
        return 2;
    }
    let Some((renovacion, uid)) = leer_credencial() else {
        eprintln!("No hay sesión guardada: abre la app, pestaña Comunidad, e inicia sesión.");
        return 1;
    };
    let s = match renovar(&Sesion { renovacion, uid, ..Default::default() }) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("No se pudo renovar la sesión: {}", e);
            return 1;
        }
    };
    guardar_credencial(Some(&s)); // the token rotates: the new one is saved for next time
    if cargar_perfil(&s).is_none() {
        eprintln!("Esta cuenta aún no tiene perfil: créalo en la app (pestaña Comunidad).");
        return 1;
    }
    let lectura = if ruta.is_dir() { empaquetar_plugin(&ruta).map(|b| ("plugin.json".to_string(), b)) } else { std::fs::read(&ruta).map(|b| (ruta.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default(), b)).map_err(|e| e.to_string()) };
    let (nombre, bytes) = match lectura {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let mut extras: Vec<(&str, Vec<u8>)> = Vec::new();
    for (clase, k) in [("icono", "--icono"), ("vista", "--vista")] {
        let r = opcion(k);
        if !r.is_empty() {
            match std::fs::read(&r) {
                Ok(b) => extras.push((clase, b)),
                Err(e) => {
                    eprintln!("{}: {}", r, e);
                    return 1;
                }
            }
        }
    }
    let (tx, _rx) = channel();
    match subir_contenido(&s, tipo, titulo.trim(), &opcion("--descripcion"), etiquetas, es_18, &nombre, &bytes, &extras, &tx) {
        Ok(m) => {
            println!("{}: {}", tipo, m);
            0
        }
        Err(e) => {
            eprintln!("{}: {}", tipo, e);
            1
        }
    }
}

/// Retries moderation of your listings left "pending" with no decision (for example, if the server failed).
///   lienzo-ui --comunidad-reintentar
pub fn reintentar_cli() -> i32 {
    let Some((renovacion, uid)) = leer_credencial() else {
        eprintln!("No hay sesión guardada: abre la app, pestaña Comunidad, e inicia sesión.");
        return 1;
    };
    let s = match renovar(&Sesion { renovacion, uid, ..Default::default() }) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("No se pudo renovar la sesión: {}", e);
            return 1;
        }
    };
    guardar_credencial(Some(&s));
    let lista = pedir("GET", &format!("/rest/v1/items?select=id,titulo&autor=eq.{}&estado=eq.pendiente&revisado_en=is.null&ruta_cuarentena=not.is.null", s.uid), Some(&s), None, None, &[], 1 << 20).and_then(|r| r.ok());
    let filas: Vec<serde_json::Value> = match lista.and_then(|r| r.json()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let mut fallos = 0;
    for f in &filas {
        let (id, titulo) = (f["id"].as_str().unwrap_or(""), f["titulo"].as_str().unwrap_or(""));
        if !es_uuid(id) {
            continue;
        }
        match json_post("/functions/v1/moderar", Some(&s), &serde_json::json!({ "item": id }), &[]).and_then(|r| r.ok()).and_then(|r| r.json::<serde_json::Value>()) {
            Ok(v) => println!("{}: {} ({})", titulo, v["estado"].as_str().unwrap_or("?"), v["motivo"].as_str().unwrap_or("")),
            Err(e) => {
                eprintln!("{}: {}", titulo, e);
                fallos += 1;
            }
        }
    }
    println!("{} ficha(s) procesada(s).", filas.len());
    if fallos > 0 { 1 } else { 0 }
}

fn sesion_guardada() -> Result<Sesion, String> {
    let (renovacion, uid) = leer_credencial().ok_or("No hay sesión guardada: abre la app, pestaña Comunidad, e inicia sesión.")?;
    let s = renovar(&Sesion { renovacion, uid, ..Default::default() }).map_err(|e| format!("No se pudo renovar la sesión: {}", e))?;
    guardar_credencial(Some(&s)); // the token rotates: the new one is saved
    Ok(s)
}

/// Your account identifier (to configure it as a moderator on the server).
///   lienzo-ui --comunidad-yo
pub fn yo_cli() -> i32 {
    match sesion_guardada() {
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

/// Human review (only accounts listed in the server's MODERADORES secret):
///   lienzo-ui --comunidad-revisar listar
///   lienzo-ui --comunidad-revisar aprobar <id> [--18]      · rechazar <id> "motivo"
///   lienzo-ui --comunidad-revisar aprobar-perfil <id-de-cuenta> avatar|banner  · rechazar-perfil <id-de-cuenta> avatar|banner
///   lienzo-ui --comunidad-revisar token-crear "nombre" leer,rechazar [dias]    (credential for scripts; shown once)
///   lienzo-ui --comunidad-revisar tokens                                       · token-revocar <id>
/// The same can be done in the moderation web page (web/moderacion).
pub fn revisar_cli(args: &[String]) -> i32 {
    let accion = args.first().map(|s| s.as_str()).unwrap_or("");
    let a1 = args.get(1).map(|s| s.as_str()).unwrap_or("");
    let a2 = args.get(2).map(|s| s.as_str()).unwrap_or("");
    let uso = "Uso: lienzo-ui --comunidad-revisar listar | aprobar <id> [--18] | rechazar <id> \"motivo\" | aprobar-perfil <cuenta> avatar|banner | rechazar-perfil <cuenta> avatar|banner | token-crear \"nombre\" leer,rechazar [dias] | tokens | token-revocar <id> | reanalizar [id]";
    let cuerpo = match accion {
        "listar" | "tokens" => serde_json::json!({ "accion": if accion == "tokens" { "listar_tokens" } else { "listar" } }),
        "aprobar" | "rechazar" if es_uuid(a1) => serde_json::json!({ "accion": accion, "item": a1, "motivo": a2, "es_18": args.iter().any(|x| x == "--18") }),
        "aprobar-perfil" | "rechazar-perfil" if es_uuid(a1) && ["avatar", "banner"].contains(&a2) => serde_json::json!({ "accion": accion.replace('-', "_"), "usuario": a1, "clase": a2 }),
        "token-crear" if !a1.is_empty() => {
            let alcances: Vec<&str> = a2.split(',').filter(|x| !x.is_empty()).collect();
            let dias: u32 = args.get(3).and_then(|d| d.parse().ok()).unwrap_or(30);
            serde_json::json!({ "accion": "crear_token", "nombre": a1, "alcances": alcances, "dias": dias })
        }
        "token-revocar" if es_uuid(a1) => serde_json::json!({ "accion": "revocar_token", "id": a1 }),
        "reanalizar" if a1.is_empty() || es_uuid(a1) => serde_json::json!({ "accion": "reanalizar", "item": if a1.is_empty() { serde_json::Value::Null } else { serde_json::json!(a1) } }),
        _ => {
            eprintln!("{}", uso);
            return 2;
        }
    };
    let s = match sesion_guardada() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let r = match json_post("/functions/v1/revisar", Some(&s), &cuerpo, &[]) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let v: serde_json::Value = r.json().unwrap_or_default();
    if !(200..300).contains(&r.estado) {
        eprintln!("{}", v["error"].as_str().unwrap_or("Error del servidor."));
        return 1;
    }
    match accion {
        "listar" => {
            let lista = v["pendientes"].as_array().cloned().unwrap_or_default();
            for f in &lista {
                println!("{}  [{}] {}  (por {}, {} bytes{})", f["id"].as_str().unwrap_or(""), f["tipo"].as_str().unwrap_or(""), f["titulo"].as_str().unwrap_or(""), f["autor_nombre"].as_str().unwrap_or(""), f["bytes"].as_i64().unwrap_or(0), if f["es_18"].as_bool().unwrap_or(false) { ", +18" } else { "" });
                println!("    motivo: {}", f["motivo_rechazo"].as_str().unwrap_or("-"));
            }
            let perfiles = v["perfiles"].as_array().cloned().unwrap_or_default();
            for p in &perfiles {
                println!("{}  [perfil {}] de {}", p["usuario"].as_str().unwrap_or(""), p["clase"].as_str().unwrap_or(""), p["nombre_publico"].as_str().unwrap_or(""));
            }
            println!("{} contenido(s) y {} imagen(es) de perfil pendientes. Para ver los archivos usa la web de moderación.", lista.len(), perfiles.len());
        }
        "tokens" => {
            for t in v["tokens"].as_array().cloned().unwrap_or_default() {
                println!("{}  {}  [{}]  caduca {}{}", t["id"].as_str().unwrap_or(""), t["nombre"].as_str().unwrap_or(""), t["alcances"].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(",")).unwrap_or_default(), t["expira_en"].as_str().unwrap_or("").get(..10).unwrap_or(""), if t["revocado"].as_bool().unwrap_or(false) { "  (revocado)" } else { "" });
            }
        }
        "token-crear" => {
            println!("Token creado (caduca {}). Cópialo ahora, no se puede volver a ver:", v["expira_en"].as_str().unwrap_or(""));
            println!("{}", v["token"].as_str().unwrap_or(""));
        }
        "reanalizar" => println!("reanalizar: {} {}{}", v["estado"].as_str().unwrap_or("?"), v["titulo"].as_str().unwrap_or(""), v["motivo"].as_str().map(|m| format!(" · {}", m)).unwrap_or_default()),
        _ => println!("{}: {}", accion, v["estado"].as_str().unwrap_or("?")),
    }
    0
}

// ───────────────────────────── cards, images and creator profiles ─────────────────────────────

/// X-style labeled field: a box with the small label on top and the text below.
fn campo_x(ui: &mut egui::Ui, etiqueta: &str, texto: &mut String, filas: usize, pista: &str) {
    egui::Frame::new().stroke(egui::Stroke::new(1.0, crate::theme::LINE())).corner_radius(5.0).inner_margin(egui::Margin::symmetric(10, 6)).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new(etiqueta).small().color(MUTED()));
        let editor = if filas > 1 { egui::TextEdit::multiline(texto).desired_rows(filas) } else { egui::TextEdit::singleline(texto) };
        ui.add(editor.frame(egui::Frame::NONE).desired_width(f32::INFINITY).hint_text(pista));
    });
    ui.add_space(6.0);
}

fn abrir_enlace(url: &str) {
    // Last check before opening the browser: https only.
    if url.starts_with("https://") && url.chars().all(|c| c.is_ascii_graphic()) {
        let _ = webbrowser::open(url);
    }
}

impl EditorApp {
    /// Public community image (icon, preview, avatar or banner). Downloaded in the background the first time.
    fn com_textura(&mut self, ctx: &egui::Context, ruta: &str) -> Option<egui::TextureHandle> {
        if let Some(t) = self.comunidad.texturas.get(ruta) {
            return t.clone();
        }
        self.comunidad.texturas.insert(ruta.to_string(), None);
        if !ruta_publica_valida(ruta) {
            return None;
        }
        let (tx, c2, r) = (self.comunidad.tx.clone(), ctx.clone(), ruta.to_string());
        std::thread::spawn(move || {
            let bytes = pedir("GET", &format!("/storage/v1/object/public/publico/{}", r), None, None, None, &[], (2 << 20) + 1)
                .and_then(|x| x.ok())
                .ok()
                .map(|x| x.cuerpo)
                .filter(|b| val::detectar(b) == Some("png"));
            let _ = tx.send(Mensaje::Imagen { ruta: r, bytes });
            c2.request_repaint();
        });
        None
    }

    /// Adjusts a content item's heart total in what is being displayed (without waiting for the server).
    fn ajustar_corazones(&mut self, id: &str, delta: i64) {
        let aplicar = |v: &mut Vec<Item>| {
            for i in v.iter_mut().filter(|i| i.id == id) {
                i.corazones = (i.corazones + delta).max(0);
            }
        };
        aplicar(&mut self.comunidad.items);
        if let Some((p, v)) = self.comunidad.viendo.as_mut() {
            let antes: i64 = v.iter().find(|i| i.id == id).map(|i| i.corazones).unwrap_or(0);
            aplicar(v);
            let despues: i64 = v.iter().find(|i| i.id == id).map(|i| i.corazones).unwrap_or(0);
            p.corazones_recibidos = (p.corazones_recibidos + (despues - antes)).max(0);
        }
    }

    fn com_corazon(&mut self, ctx: &egui::Context, item: String) {
        let Some(s) = self.comunidad.sesion.clone() else { return };
        if !es_uuid(&item) {
            return;
        }
        let dado = self.comunidad.mis_corazones.contains(&item);
        if dado {
            self.comunidad.mis_corazones.remove(&item);
        } else {
            self.comunidad.mis_corazones.insert(item.clone());
        }
        self.ajustar_corazones(&item, if dado { -1 } else { 1 });
        let (tx, c2) = (self.comunidad.tx.clone(), ctx.clone());
        std::thread::spawn(move || {
            let r = vigente(&s, &tx).and_then(|s| {
                if dado {
                    pedir("DELETE", &format!("/rest/v1/corazones?persona=eq.{}&item=eq.{}", s.uid, item), Some(&s), None, None, &[], 1 << 16)?.ok().map(|_| s)
                } else {
                    let r = json_post("/rest/v1/corazones", Some(&s), &serde_json::json!({ "persona": s.uid, "item": item }), &[("Prefer", "return=minimal")])?;
                    if r.estado == 409 { Ok(s) } else { r.ok().map(|_| s) } // already had it: fine
                }
            });
            match r {
                Ok(_) => {}
                Err(e) => {
                    let _ = tx.send(Mensaje::Estado(e));
                    // the server decides: re-read my data to undo the optimistic change
                    if let Ok(s) = vigente(&s, &tx) {
                        if let Some((corazones, seguidos)) = cargar_mios(&s) {
                            let _ = tx.send(Mensaje::Mios { corazones, seguidos });
                        }
                    }
                }
            }
            c2.request_repaint();
        });
    }

    fn com_seguir(&mut self, ctx: &egui::Context, creador: String) {
        let Some(s) = self.comunidad.sesion.clone() else { return };
        if !es_uuid(&creador) || creador == s.uid {
            return;
        }
        let sigo = self.comunidad.mis_seguidos.contains(&creador);
        if sigo {
            self.comunidad.mis_seguidos.remove(&creador);
        } else {
            self.comunidad.mis_seguidos.insert(creador.clone());
        }
        if let Some((p, _)) = self.comunidad.viendo.as_mut() {
            if p.id == creador {
                p.seguidores = (p.seguidores + if sigo { -1 } else { 1 }).max(0);
            }
        }
        let (tx, c2) = (self.comunidad.tx.clone(), ctx.clone());
        std::thread::spawn(move || {
            let r = vigente(&s, &tx).and_then(|s| {
                if sigo {
                    pedir("DELETE", &format!("/rest/v1/seguidores?seguidor=eq.{}&seguido=eq.{}", s.uid, creador), Some(&s), None, None, &[], 1 << 16)?.ok().map(|_| s)
                } else {
                    let r = json_post("/rest/v1/seguidores", Some(&s), &serde_json::json!({ "seguidor": s.uid, "seguido": creador }), &[("Prefer", "return=minimal")])?;
                    if r.estado == 409 { Ok(s) } else { r.ok().map(|_| s) }
                }
            });
            if let Err(e) = r {
                let _ = tx.send(Mensaje::Estado(e));
                if let Ok(s) = vigente(&s, &tx) {
                    if let Some((corazones, seguidos)) = cargar_mios(&s) {
                        let _ = tx.send(Mensaje::Mios { corazones, seguidos });
                    }
                }
            }
            c2.request_repaint();
        });
    }

    /// Floating community windows: the enlarged preview and the creator profile. Called every frame.
    pub fn comunidad_ventanas(&mut self, ctx: &egui::Context) {
        // Full-size preview: clicking outside, Esc or the button closes it.
        if let Some(ruta) = self.comunidad.ampliada.clone() {
            let tex = self.com_textura(ctx, &ruta);
            let pantalla = ctx.content_rect().size();
            #[allow(deprecated)]
            let id_modal = egui::Id::new("comunidad_vista_ampliada");
            let modal = egui::Modal::new(id_modal).show(ctx, |ui| {
                match &tex {
                    Some(t) => {
                        let s = t.size_vec2();
                        let escala = ((pantalla.x * 0.86) / s.x).min((pantalla.y * 0.78) / s.y).min(4.0).max(0.1);
                        ui.add(egui::Image::new(t).fit_to_exact_size(s * escala).corner_radius(6.0));
                    }
                    None => {
                        ui.spinner();
                    }
                }
                ui.add_space(6.0);
                ui.vertical_centered(|ui| {
                    if ui.button(t("Cerrar")).clicked() {
                        self.comunidad.ampliada = None;
                    }
                });
            });
            if modal.should_close() {
                self.comunidad.ampliada = None;
            }
        }
        // My creator profile (editable).
        if self.comunidad.perfil_abierto {
            egui::Window::new("mi_perfil_creador").title_bar(false).collapsible(false).resizable(false).default_width(520.0).anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO).show(ctx, |ui| {
                ui.set_width(520.0);
                self.com_editar_perfil(ui, ctx);
            });
            self.comunidad.perfil_abierto &= self.comunidad.sesion.is_some();
        }
    }

    /// Header button: your profile photo (or the default image). Opens your profile; when signed out, opens the Community tab.
    pub fn comunidad_boton_perfil(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        let propia = match self.comunidad.mi_perfil.clone() {
            Some(mi) => self.com_imagen_propia(&ctx, "avatar", &mi),
            None => None,
        };
        let lado = egui::Vec2::splat(28.0);
        let r = match &propia {
            Some(t) => ui.add(egui::Image::new(t).fit_to_exact_size(lado).corner_radius(14.0).sense(egui::Sense::click())),
            None => ui.add(egui::Image::new(egui::include_image!("../assets/marca/avatar.svg")).fit_to_exact_size(lado).sense(egui::Sense::click())),
        };
        let borde = if r.hovered() { ACCENT() } else { MUTED() };
        ui.painter().circle_stroke(r.rect.center(), 14.0, egui::Stroke::new(1.5, borde));
        let tip = match (&self.comunidad.sesion, &self.comunidad.perfil) {
            (Some(_), Some(p)) => format!("{}: {}", t("Mi perfil de creador"), p.nombre_publico),
            _ => t("Perfil de creador: inicia sesión en Comunidad"),
        };
        if r.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(tip).clicked() {
            if self.comunidad.sesion.is_some() && self.comunidad.perfil.is_some() {
                self.comunidad.perfil_abierto = true;
            } else {
                self.tab = crate::app::Tab::Comunidad;
                self.drawer_open = true;
            }
        }
    }

    /// ONLY for documentation screenshots: fakes a signed-in account (no network or real credentials) and loads the
    /// real public catalog. No action in this fake session reaches any server with anyone's data.
    pub fn comunidad_demo(&mut self, ctx: &egui::Context, abrir_perfil: bool, abrir_subida: bool) {
        let uid = "00000000-0000-4000-8000-000000000000".to_string();
        self.comunidad.iniciado = true;
        self.tab = crate::app::Tab::Comunidad;
        self.drawer_open = true;
        if !abrir_subida {
            self.com_cargar_lista(ctx); // no session yet: real anonymous catalog
        }
        self.comunidad.sesion = Some(Sesion { acceso: "demo".into(), renovacion: "demo".into(), uid: uid.clone(), expira: u64::MAX / 2 });
        self.comunidad.perfil = Some(Perfil { nombre_publico: "TuNombre".into(), mostrar_18: false });
        self.comunidad.perfil_cargado = true;
        let mut enlaces = std::collections::BTreeMap::new();
        enlaces.insert("itch".to_string(), "https://tunombre.itch.io/".to_string());
        self.comunidad.mi_perfil = Some(PerfilPublico { id: uid, nombre_publico: "TuNombre".into(), bio: "Hago interfaces para juegos con Unity.".into(), github: Some("tu-usuario".into()), enlaces, ..Default::default() });
        let f = &mut self.comunidad.form_perfil;
        f.nombre = "TuNombre".into();
        f.bio = "Hago interfaces para juegos con Unity.".into();
        f.github = "tu-usuario".into();
        f.enlaces = REDES.iter().map(|(k, _, _)| if *k == "itch" { "https://tunombre.itch.io/".to_string() } else { String::new() }).collect();
        f.cargado = true;
        self.comunidad.perfil_abierto = abrir_perfil;
        if abrir_subida {
            // "Share" form already open with an example, no catalog underneath
            self.comunidad.abrir_subida = true;
            self.comunidad.subir.tipo = 4;
            self.comunidad.subir.titulo = "Mi menú de botones".into();
            self.comunidad.subir.descripcion = "Cuatro botones listos para un menú principal.".into();
            self.comunidad.subir.etiquetas = "menu, botones".into();
            self.comunidad.subir.archivo = Some(PathBuf::from("menu_de_botones.json"));
        }
    }

    /// Opens the Community tab filtered by a type (e.g. "tema" from the Themes window).
    pub fn comunidad_ir_a(&mut self, ctx: &egui::Context, tipo: &'static str) {
        self.tab = crate::app::Tab::Comunidad;
        self.drawer_open = true;
        self.comunidad.viendo = None;
        self.comunidad.solo_siguiendo = false;
        self.comunidad.filtro = NOMBRES_TIPO.iter().find(|(k, _)| *k == tipo).map(|(k, _)| *k);
        if self.comunidad.iniciado {
            self.com_cargar_lista(ctx);
        }
    }

    /// Prepares the "Share" form with a file (e.g. the current theme from the Themes window).
    pub fn comunidad_preparar_subida(&mut self, ctx: &egui::Context, tipo: &'static str, archivo: PathBuf, titulo: &str) {
        self.comunidad_ir_a(ctx, tipo);
        if let Some(i) = NOMBRES_TIPO.iter().position(|(k, _)| *k == tipo) {
            self.comunidad.subir.tipo = i;
        }
        self.comunidad.subir.archivo = Some(archivo);
        self.comunidad.subir.titulo = titulo.chars().take(80).collect();
        self.comunidad.abrir_subida = true;
        self.comunidad.estado = if self.comunidad.sesion.is_some() { t("Revisa los datos y pulsa «Enviar a revisión».") } else { t("Inicia sesión para compartir.") };
    }

    fn com_abrir_perfil(&mut self, ctx: &egui::Context, uid: String) {
        let s = self.comunidad.sesion.clone();
        self.com_lanzar(ctx, move |tx| {
            let s = s.and_then(|s| vigente(&s, tx).ok());
            let _ = tx.send(Mensaje::PerfilVer(cargar_perfil_publico(s.as_ref(), &uid)));
        });
    }

    fn com_tarjeta(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, it: &Item, logueado: bool) {
        egui::Frame::new().fill(crate::theme::PANEL_DARK()).corner_radius(6.0).inner_margin(8.0).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                // Content type icon, drawn as SVG (uploaded images are only used as previews).
                let nombre_icono = match it.tipo.as_str() {
                    "plugin" => "tab_plugins",
                    "imagen" => "tab_images",
                    "fuente" => "tipo_fuente",
                    "plantilla" => "tab_templates",
                    "elemento" => "tab_elements",
                    _ => "tipo_tema",
                };
                let (r, _) = ui.allocate_exact_size(egui::Vec2::splat(40.0), egui::Sense::hover());
                ui.painter().rect_filled(r, 8.0, crate::theme::ACCENT_SOFT());
                crate::icons::paint_or(ui, nombre_icono, r.shrink(9.0), ACCENT(), "");
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&it.titulo).strong());
                        if it.es_18 {
                            ui.label(RichText::new("+18").small().color(WARN()));
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 3.0;
                        ui.label(RichText::new(format!("{} · {}", t(nombre_tipo(&it.tipo)), t("por"))).small().color(MUTED()));
                        let nombre = if it.autor_nombre.is_empty() { "—" } else { it.autor_nombre.as_str() };
                        if ui.link(RichText::new(nombre).small()).on_hover_text(t("Ver el perfil del creador")).clicked() {
                            self.com_abrir_perfil(ctx, it.autor.clone());
                        }
                        crate::icons::con_texto(ui, "download", RichText::new(it.descargas.to_string()).small().color(MUTED()), MUTED());
                    });
                });
            });
            if !it.descripcion.is_empty() {
                ui.label(RichText::new(&it.descripcion).small());
            }
            if !it.etiquetas.is_empty() {
                ui.label(RichText::new(it.etiquetas.iter().map(|e| format!("#{}", e)).collect::<Vec<_>>().join(" ")).small().color(MUTED()));
            }
            if let Some(r) = it.vista_publica.clone() {
                egui::CollapsingHeader::new(RichText::new(t("Vista previa")).small()).id_salt(("vista", &it.id)).show(ui, |ui| match self.com_textura(ctx, &r) {
                    Some(tex) => {
                        let w = ui.available_width();
                        let s = tex.size_vec2();
                        let img = ui.add(egui::Image::new(&tex).fit_to_exact_size(egui::Vec2::new(w, w * s.y / s.x.max(1.0))).corner_radius(4.0).sense(egui::Sense::click()));
                        if img.on_hover_cursor(egui::CursorIcon::PointingHand).on_hover_text(t("Clic para ampliar")).clicked() {
                            self.comunidad.ampliada = Some(r.clone());
                        }
                    }
                    None => {
                        ui.spinner();
                    }
                });
            }
            ui.horizontal(|ui| {
                if ui.add_enabled(self.comunidad.ocupado == 0, egui::Button::new(RichText::new(t("Instalar")).color(ACCENT()))).clicked() {
                    self.comunidad.estado = t("Descargando y verificando…");
                    let item = it.clone();
                    let sesion = self.comunidad.sesion.clone();
                    self.com_lanzar(ctx, move |tx| {
                        let sesion = sesion.and_then(|s| vigente(&s, tx).ok());
                        let bytes = descargar(&item, sesion.as_ref());
                        let _ = tx.send(Mensaje::Descargado { item, bytes });
                    });
                }
                let dado = self.comunidad.mis_corazones.contains(&it.id);
                let color = if dado { WARN() } else { MUTED() };
                let corazon = RichText::new(it.corazones.max(0).to_string()).color(color);
                let r = ui.add_enabled(logueado, crate::icons::btn_tint(if dado { "heart_fill" } else { "heart" }, corazon, color).small());
                if r.on_hover_text(if logueado { t("Dar o quitar corazón") } else { t("Inicia sesión para dar corazones y seguir a creadores.") }).clicked() {
                    self.com_corazon(ctx, it.id.clone());
                }
                if logueado && ui.small_button(t("Reportar")).clicked() {
                    self.comunidad.reportar = Some(it.id.clone());
                }
            });
            if self.comunidad.reportar.as_deref() == Some(it.id.as_str()) {
                ui.label(RichText::new(t("¿Qué problema tiene?")).small());
                for (k, n) in MOTIVOS {
                    if ui.small_button(t(n)).clicked() {
                        let (s, id, motivo) = (self.comunidad.sesion.clone().unwrap(), it.id.clone(), k.to_string());
                        self.comunidad.reportar = None;
                        self.com_lanzar(ctx, move |tx| {
                            let r = vigente(&s, tx).and_then(|s| json_post("/rest/v1/reportes", Some(&s), &serde_json::json!({ "item": id, "reportante": s.uid, "motivo": motivo }), &[("Prefer", "return=minimal")])?.ok().map(|_| ()));
                            let _ = tx.send(Mensaje::Reportado(r));
                        });
                    }
                }
                if ui.small_button(t("Cancelar")).clicked() {
                    self.comunidad.reportar = None;
                }
            }
        });
    }

    fn com_ver_perfil(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let Some((p, items)) = self.comunidad.viendo.clone() else { return };
        if ui.add(crate::icons::btn("arrow_left", t("Volver a la comunidad"))).clicked() {
            self.comunidad.viendo = None;
            return;
        }
        ui.add_space(4.0);
        let es_mio = self.comunidad.sesion.as_ref().map(|s| s.uid == p.id).unwrap_or(false);
        let mi = self.comunidad.mi_perfil.clone().filter(|_| es_mio);
        let banner = match &mi {
            Some(m) => self.com_imagen_propia(ctx, "banner", m),
            None => p.banner_publico.as_deref().and_then(|r| self.com_textura(ctx, r)),
        };
        let avatar = match &mi {
            Some(m) => self.com_imagen_propia(ctx, "avatar", m),
            None => p.avatar_publico.as_deref().and_then(|r| self.com_textura(ctx, r)),
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
                    None => ui.add(egui::Image::new(egui::include_image!("../assets/marca/avatar.svg")).fit_to_exact_size(egui::Vec2::splat(56.0))),
                };
                ui.vertical(|ui| {
                    ui.label(RichText::new(&p.nombre_publico).strong().size(16.0));
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(format!("{} {} · {} {} ·", items.len(), t("contenidos publicados"), p.seguidores.max(0), t("seguidores"))).small().color(MUTED()));
                        crate::icons::con_texto(ui, "heart_fill", RichText::new(p.corazones_recibidos.max(0).to_string()).small().color(MUTED()), WARN());
                    });
                    let propio = self.comunidad.sesion.as_ref().map(|s| s.uid == p.id).unwrap_or(false);
                    if !propio {
                        let sigo = self.comunidad.mis_seguidos.contains(&p.id);
                        let etiqueta = if sigo { t("Siguiendo") } else { t("Seguir") };
                        let r = ui.add_enabled(self.comunidad.sesion.is_some() && self.comunidad.perfil.is_some(), crate::icons::btn(if sigo { "check" } else { "plus" }, etiqueta).small());
                        if r.on_hover_text(if sigo { t("Dejar de seguir") } else { t("Inicia sesión para dar corazones y seguir a creadores.") }).clicked() {
                            self.com_seguir(ctx, p.id.clone());
                        }
                    }
                });
            });
            if !p.bio.is_empty() {
                ui.label(RichText::new(&p.bio).small());
            }
            ui.horizontal_wrapped(|ui| {
                if let Some(g) = p.github.as_deref().filter(|g| github_valido(g)) {
                    if ui.small_button(format!("GitHub · {}", g)).clicked() {
                        abrir_enlace(&format!("https://github.com/{}", g));
                    }
                }
                for (k, nombre, _) in REDES {
                    if let Some(url) = p.enlaces.get(*k).filter(|u| enlace_valido(k, u)) {
                        if ui.small_button(*nombre).on_hover_text(url.as_str()).clicked() {
                            abrir_enlace(url);
                        }
                    }
                }
            });
        });
        ui.add_space(6.0);
        let logueado = self.comunidad.sesion.is_some() && self.comunidad.perfil.is_some();
        for it in &items {
            self.com_tarjeta(ui, ctx, it, logueado);
            ui.add_space(4.0);
        }
    }

    /// "My creator profile" editor, X-style: banner with overlaid photo, labeled fields and "Save" at the top.
    fn com_editar_perfil(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let Some(s) = self.comunidad.sesion.clone() else { return };
        let mi = self.comunidad.mi_perfil.clone().unwrap_or_default();
        if !self.comunidad.form_perfil.cargado {
            if let Some(p) = &self.comunidad.perfil {
                self.comunidad.form_perfil.nombre = p.nombre_publico.clone();
            }
        }
        // ── Top bar: close · title · Save ──
        let mut guardar = false;
        ui.horizontal(|ui| {
            if ui.add(crate::icons::btn("close", "").frame(false)).on_hover_text(t("Cerrar")).clicked() {
                self.comunidad.perfil_abierto = false;
            }
            ui.add_space(10.0);
            ui.label(RichText::new(t("Editar perfil")).strong().size(19.0));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let b = egui::Button::new(RichText::new(t("Guardar")).strong().color(crate::theme::PANEL())).fill(crate::theme::TEXT()).corner_radius(16.0).min_size(egui::vec2(86.0, 32.0));
                if ui.add_enabled(self.comunidad.ocupado == 0, b).clicked() {
                    guardar = true;
                }
            });
        });
        if !self.comunidad.estado.is_empty() {
            ui.label(RichText::new(&self.comunidad.estado).small().color(ACCENT()));
        }
        ui.add_space(4.0);

        egui::ScrollArea::vertical().max_height(ctx.content_rect().height() * 0.72).show(ui, |ui| {
            // ── Banner with its buttons ──
            let ancho = ui.available_width();
            let (rb, _) = ui.allocate_exact_size(egui::vec2(ancho, (ancho / 3.0).round()), egui::Sense::hover());
            let banner = self.com_imagen_propia(ctx, "banner", &mi);
            match &banner {
                Some(tex) => {
                    egui::Image::new(tex).corner_radius(6.0).paint_at(ui, rb);
                }
                None => {
                    ui.painter().rect_filled(rb, 6.0, crate::theme::ACCENT_SOFT());
                }
            }
            let hay_banner = banner.is_some();
            let mut accion: Option<(&'static str, bool)> = None; // (class, true = choose / false = remove)
            let redondo = |icono: &str| crate::icons::redondo(icono, 44.0, egui::Color32::WHITE);
            let c = rb.center();
            let r1 = egui::Rect::from_center_size(c - egui::vec2(if hay_banner { 28.0 } else { 0.0 }, 0.0), egui::vec2(44.0, 44.0));
            if ui.put(r1, redondo("camera")).on_hover_text(t("Cambiar el banner")).clicked() {
                accion = Some(("banner", true));
            }
            if hay_banner {
                let r2 = egui::Rect::from_center_size(c + egui::vec2(28.0, 0.0), egui::vec2(44.0, 44.0));
                if ui.put(r2, redondo("close")).on_hover_text(t("Quitar el banner")).clicked() {
                    accion = Some(("banner", false));
                }
            }
            // ── Overlaid profile photo ──
            let ra = egui::Rect::from_center_size(egui::pos2(rb.left() + 70.0, rb.bottom()), egui::Vec2::splat(108.0));
            ui.painter().circle_filled(ra.center(), 58.0, crate::theme::PANEL());
            match self.com_imagen_propia(ctx, "avatar", &mi) {
                Some(tex) => egui::Image::new(&tex).corner_radius(54.0).paint_at(ui, ra),
                None => egui::Image::new(egui::include_image!("../assets/marca/avatar.svg")).paint_at(ui, ra),
            }
            if ui.put(egui::Rect::from_center_size(ra.center(), egui::vec2(44.0, 44.0)), redondo("camera")).on_hover_text(t("Cambiar la foto de perfil")).clicked() {
                accion = Some(("avatar", true));
            }
            ui.add_space(62.0);
            let hay_avatar = mi.avatar_publico.is_some() || mi.avatar_pendiente || self.comunidad.previas.contains_key("avatar");
            if hay_avatar && ui.small_button(t("Quitar la foto de perfil")).clicked() {
                accion = Some(("avatar", false));
            }
            // ── Review notice (like X's) ──
            let pendientes: Vec<String> = [("avatar", mi.avatar_pendiente, t("la foto de perfil")), ("banner", mi.banner_pendiente, t("el banner"))]
                .into_iter()
                .filter(|(_, p, _)| *p)
                .map(|(_, _, n)| n)
                .collect();
            if !pendientes.is_empty() {
                ui.add_space(6.0);
                egui::Frame::new().fill(crate::theme::ACCENT_SOFT()).corner_radius(6.0).inner_margin(10.0).show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(format!("{} {}. {}", t("Se está revisando"), pendientes.join(&format!(" {} ", t("y"))), t("Tú ya la ves aquí; los demás la verán cuando la apruebe una persona."))).color(crate::theme::TEXT()));
                });
            }
            ui.add_space(10.0);
            // ── Fields ──
            let f = &mut self.comunidad.form_perfil;
            if f.enlaces.len() != REDES.len() {
                f.enlaces.resize(REDES.len(), String::new());
            }
            let mut errores: Vec<String> = Vec::new();
            campo_x(ui, &t("Nombre"), &mut f.nombre, 1, "nombre_publico");
            let nombre = f.nombre.trim().to_string();
            if !((3..=32).contains(&nombre.chars().count()) && nombre.chars().all(|c| c.is_ascii_alphanumeric() || "_.-".contains(c))) {
                errores.push(t("Nombre: 3-32 caracteres (letras, números, punto, guion y guion bajo)."));
            }
            campo_x(ui, &format!("{} · {}/300", t("Biografía"), f.bio.chars().count()), &mut f.bio, 3, &t("Cuéntale a la comunidad quién eres (sin enlaces)"));
            if f.bio.chars().count() > 300 {
                errores.push(t("La descripción admite hasta 300 caracteres."));
            }
            campo_x(ui, "GitHub", &mut f.github, 1, &t("Usuario de GitHub"));
            let gh = f.github.trim().trim_start_matches('@').to_string();
            if !gh.is_empty() && !github_valido(&gh) {
                errores.push(t("Usuario de GitHub no válido."));
            }
            for (i, (k, nombre_red, dominios)) in REDES.iter().enumerate() {
                campo_x(ui, nombre_red, &mut f.enlaces[i], 1, &format!("https://{}/…", dominios[0]));
                let u = f.enlaces[i].trim();
                if !u.is_empty() && !enlace_valido(k, u) {
                    errores.push(format!("{}: {} https://{}/…", nombre_red, t("debe empezar por"), dominios[0]));
                }
            }
            for e in &errores {
                ui.label(RichText::new(e).small().color(WARN()));
            }
            ui.add_space(6.0);
            // ── "View my public profile ›" row ──
            let fila = ui.add(crate::icons::btn("user", RichText::new(t("Ver mi perfil público")).size(15.0)).frame(false));
            if fila.clicked() {
                self.comunidad.perfil_abierto = false;
                self.tab = crate::app::Tab::Comunidad;
                self.drawer_open = true;
                self.com_abrir_perfil(ctx, s.uid.clone());
            }
            ui.label(RichText::new(t("Tu perfil es público cuando tienes al menos un contenido aprobado. Las imágenes y la descripción se moderan.")).small().color(MUTED()));

            // ── Actions ──
            match accion {
                Some((clase, true)) => self.com_elegir_imagen_perfil(ctx, clase),
                Some((clase, false)) => {
                    self.comunidad.previas.remove(clase);
                    let s = s.clone();
                    self.com_lanzar(ctx, move |tx| {
                        let r = vigente(&s, tx).and_then(|s| json_post("/functions/v1/perfil", Some(&s), &serde_json::json!({ "accion": "quitar", "clase": clase }), &[])?.ok().map(|_| s));
                        if let Ok(s) = &r {
                            let _ = tx.send(Mensaje::MiPerfil(cargar_mi_perfil(s)));
                        }
                        let _ = tx.send(Mensaje::Estado(r.map(|_| t("Imagen quitada.")).unwrap_or_else(|e| e)));
                    });
                }
                None => {}
            }
            if guardar {
                if !errores.is_empty() {
                    self.comunidad.estado = errores[0].clone();
                } else {
                    let f = &self.comunidad.form_perfil;
                    let nombre_nuevo = (Some(nombre.clone()) != self.comunidad.perfil.as_ref().map(|p| p.nombre_publico.clone())).then_some(nombre.clone());
                    let enlaces: serde_json::Map<String, serde_json::Value> = REDES.iter().zip(&f.enlaces).filter(|(_, u)| !u.trim().is_empty()).map(|((k, _, _), u)| (k.to_string(), serde_json::Value::String(u.trim().to_string()))).collect();
                    let cuerpo = serde_json::json!({ "accion": "guardar", "bio": f.bio.trim(), "github": gh, "enlaces": enlaces });
                    let s = s.clone();
                    self.comunidad.estado = t("Guardando…");
                    self.com_lanzar(ctx, move |tx| {
                        let r = vigente(&s, tx).and_then(|s| {
                            if let Some(n) = &nombre_nuevo {
                                pedir("PATCH", &format!("/rest/v1/perfiles?id=eq.{}", s.uid), Some(&s), None, Some(serde_json::json!({ "nombre_publico": n }).to_string().as_bytes()), &[], 1 << 16)?.ok()?;
                            }
                            let r = json_post("/functions/v1/perfil", Some(&s), &cuerpo, &[])?;
                            if (200..300).contains(&r.estado) {
                                Ok(s)
                            } else {
                                Err(r.json::<serde_json::Value>().ok().and_then(|v| v["error"].as_str().map(|e| e.to_string())).unwrap_or_else(|| mensaje_error(r.estado, "")))
                            }
                        });
                        if let Ok(s) = &r {
                            let _ = tx.send(Mensaje::MiPerfil(cargar_mi_perfil(s)));
                        }
                        let _ = tx.send(Mensaje::Estado(r.map(|_| t("Perfil guardado.")).unwrap_or_else(|e| e)));
                    });
                }
            }
        });
    }

    /// My profile image as I see it: the one I just chose, the one under review (my quarantine) or the published one.
    fn com_imagen_propia(&mut self, ctx: &egui::Context, clase: &'static str, mi: &PerfilPublico) -> Option<egui::TextureHandle> {
        if let Some(t) = self.comunidad.previas.get(clase) {
            return Some(t.clone());
        }
        let pendiente = if clase == "avatar" { mi.avatar_pendiente } else { mi.banner_pendiente };
        let publica = if clase == "avatar" { mi.avatar_publico.clone() } else { mi.banner_publico.clone() };
        if pendiente {
            return self.com_textura_cuarentena(ctx, clase);
        }
        publica.and_then(|r| self.com_textura(ctx, &r))
    }

    /// My profile image under review: only its owner can read it (quarantine is private).
    fn com_textura_cuarentena(&mut self, ctx: &egui::Context, clase: &'static str) -> Option<egui::TextureHandle> {
        let clave = format!("cuarentena:{}", clase);
        if let Some(t) = self.comunidad.texturas.get(&clave) {
            return t.clone();
        }
        let s = self.comunidad.sesion.clone()?;
        self.comunidad.texturas.insert(clave.clone(), None);
        let (tx, c2) = (self.comunidad.tx.clone(), ctx.clone());
        std::thread::spawn(move || {
            let bytes = vigente(&s, &tx)
                .and_then(|s| pedir("GET", &format!("/storage/v1/object/authenticated/cuarentena/{}/perfil/{}.png", s.uid, clase), Some(&s), None, None, &[], (2 << 20) + 1)?.ok())
                .ok()
                .map(|r| r.cuerpo)
                .filter(|b| val::detectar(b) == Some("png"));
            let _ = tx.send(Mensaje::Imagen { ruta: clave, bytes });
            c2.request_repaint();
        });
        None
    }

    /// Picks a photo or banner (PNG, JPEG, WebP, GIF or BMP of any size): the app crops and shrinks it,
    /// shows it immediately and uploads it for review.
    fn com_elegir_imagen_perfil(&mut self, ctx: &egui::Context, clase: &'static str) {
        let Some(f) = rfd::FileDialog::new().add_filter(t("Imágenes"), &["png", "jpg", "jpeg", "webp", "gif", "bmp"]).pick_file() else { return };
        let preparada = std::fs::read(&f).map_err(|e| e.to_string()).and_then(|b| preparar_imagen_perfil(clase, &b));
        let bytes = match preparada {
            Ok(b) => b,
            Err(e) => {
                self.comunidad.estado = e;
                return;
            }
        };
        if let Ok(img) = image::load_from_memory_with_format(&bytes, image::ImageFormat::Png) {
            let img = img.to_rgba8();
            let (w, h) = img.dimensions();
            let ci = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], img.as_raw());
            self.comunidad.previas.insert(clase, ctx.load_texture(format!("previa/{}", clase), ci, egui::TextureOptions::LINEAR));
        }
        let Some(s) = self.comunidad.sesion.clone() else { return };
        self.comunidad.estado = t("Subiendo y analizando la imagen… (puede tardar hasta un minuto)");
        self.com_lanzar(ctx, move |tx| {
            let r = vigente(&s, tx).and_then(|s| subir_imagen_perfil(&s, clase, &bytes).map(|m| (s, m)));
            match r {
                Ok((s, m)) => {
                    let _ = tx.send(Mensaje::MiPerfil(cargar_mi_perfil(&s)));
                    let _ = tx.send(Mensaje::Estado(m));
                }
                Err(e) => {
                    let _ = tx.send(Mensaje::Estado(e));
                }
            }
        });
    }
}

#[cfg(test)]
mod tests_perfil {
    use super::*;

    #[test]
    fn enlaces_de_redes() {
        assert!(enlace_valido("x", "https://x.com/deritu"));
        assert!(enlace_valido("itch", "https://deritu.itch.io"));
        assert!(enlace_valido("youtube", "https://www.youtube.com/@deritu"));
        assert!(!enlace_valido("x", "http://x.com/deritu"));
        assert!(!enlace_valido("x", "https://x.com.malo.example/deritu"));
        assert!(!enlace_valido("x", "https://usuario:clave@x.com/"));
        assert!(!enlace_valido("x", "https://x.com:8080/"));
        assert!(!enlace_valido("youtube", "javascript:alert(1)"));
        assert!(!enlace_valido("discord", "https://discord.gg/x"));
        assert!(!enlace_valido("itch", "https://.itch.io"));
    }

    #[test]
    fn usuarios_de_github_y_rutas() {
        assert!(github_valido("deritu-kutsut"));
        assert!(!github_valido("-malo"));
        assert!(!github_valido("con espacio"));
        assert!(ruta_publica_valida("plugin/abc-123/icono.png"));
        assert!(!ruta_publica_valida("../secreto.png"));
        assert!(!ruta_publica_valida("plugin//x.png"));
    }
}

/// Gives or removes a heart on a content item, or follows / unfollows a creator, using the saved session.
///   lienzo-ui --comunidad-corazon <id-del-contenido>
///   lienzo-ui --comunidad-seguir <id-del-creador>
pub fn social_cli(que: &str, id: &str) -> i32 {
    if !es_uuid(id) {
        eprintln!("Indica un identificador válido.");
        return 2;
    }
    let s = match sesion_guardada() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let Some((corazones, seguidos)) = cargar_mios(&s) else {
        eprintln!("No se pudo leer lo tuyo (¿tienes perfil?).");
        return 1;
    };
    let (ruta_get, ruta_post, campo, fila, ya) = if que == "corazon" {
        (format!("/rest/v1/corazones?persona=eq.{}&item=eq.{}", s.uid, id), "/rest/v1/corazones", "item", serde_json::json!({ "persona": s.uid, "item": id }), corazones.iter().any(|i| i == id))
    } else {
        (format!("/rest/v1/seguidores?seguidor=eq.{}&seguido=eq.{}", s.uid, id), "/rest/v1/seguidores", "seguido", serde_json::json!({ "seguidor": s.uid, "seguido": id }), seguidos.iter().any(|i| i == id))
    };
    let r = if ya { pedir("DELETE", &ruta_get, Some(&s), None, None, &[], 1 << 16) } else { json_post(ruta_post, Some(&s), &fila, &[("Prefer", "return=minimal")]) };
    match r.and_then(|r| r.ok()) {
        Ok(_) => println!("{} {}: {}", que, campo, if ya { "quitado" } else { "añadido" }),
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    }
    let total = if que == "corazon" { pedir("GET", &format!("/rest/v1/items?select=corazones&id=eq.{}", id), None, None, None, &[], 1 << 16) } else { pedir("GET", &format!("/rest/v1/perfiles_publicos?select=seguidores&id=eq.{}", id), None, None, None, &[], 1 << 16) };
    if let Ok(t) = total.and_then(|r| r.ok()) {
        println!("total ahora: {}", t.texto());
    }
    0
}

#[cfg(test)]
mod tests_imagenes {
    use super::*;

    fn jpeg(w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbImage::from_fn(w, h, |x, y| image::Rgb([(x % 256) as u8, (y % 256) as u8, 128]));
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgb8(img).write_to(&mut out, image::ImageFormat::Jpeg).unwrap();
        out.into_inner()
    }

    #[test]
    fn una_foto_grande_se_recorta_y_reduce() {
        let a = preparar_imagen_perfil("avatar", &jpeg(1600, 900)).unwrap();
        assert!(val::validar_png("avatar", &a).is_ok());
        let b = preparar_imagen_perfil("banner", &jpeg(1000, 1000)).unwrap();
        let (w, h) = val::validar_png("banner", &b).unwrap();
        assert_eq!(w, h * 3, "el banner queda en 3:1");
    }

    #[test]
    fn un_ejecutable_no_pasa_por_imagen() {
        let mut exe = vec![0u8; 64];
        exe[..2].copy_from_slice(b"MZ");
        assert!(preparar_imagen_perfil("avatar", &exe).is_err());
    }
}
