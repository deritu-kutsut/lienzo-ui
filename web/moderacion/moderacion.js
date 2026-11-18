// Lienzo UI moderation web. No libraries or build step.
// Security: everything coming from users is rendered with textContent (never as HTML); the session lives in sessionStorage
// (cleared when the tab closes); the CSP policy in index.html prevents loading anything outside this site and Supabase.
"use strict";

const URL_SUPABASE = "https://btotvbdhoyfjpdixlcrw.supabase.co";
const CLAVE_PUBLICA = "sb_publishable_UNvWtzs_M8N_N6fLrZ9VCQ_E6hkxXrL"; // public by design
const RETORNO = location.origin + location.pathname;
const $ = (id) => document.getElementById(id);

// ───────── utilities ─────────
function el(tag, props = {}, ...hijos) {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(props)) {
    if (k === "class") e.className = v;
    else if (k === "texto") e.textContent = v;
    else if (k.startsWith("on")) e.addEventListener(k.slice(2), v);
    else if (v !== false && v !== undefined) e.setAttribute(k, v === true ? "" : v);
  }
  for (const h of hijos) if (h) e.append(h);
  return e;
}
function estado(texto, error = false) {
  const e = $("estado");
  e.textContent = texto || "";
  e.className = error ? "error" : "";
}
function b64url(bytes) {
  let s = "";
  for (const b of bytes) s += String.fromCharCode(b);
  return btoa(s).replace(/\+/g, "-").replace(/\//g, "_").replace(/=+$/, "");
}
function aleatorio(n) {
  return crypto.getRandomValues(new Uint8Array(n));
}
const guardar = (s) => sessionStorage.setItem("lm_sesion", JSON.stringify(s));
const leer = () => {
  try { return JSON.parse(sessionStorage.getItem("lm_sesion") || "null"); } catch { return null; }
};
// Only signed links from our Supabase are followed: nothing a user wrote.
const urlSegura = (u) => typeof u === "string" && u.startsWith(URL_SUPABASE + "/storage/v1/object/sign/") ? u : null;

// ───────── session ─────────
let sesion = leer();

async function entrarOAuth(proveedor) {
  const verificador = b64url(aleatorio(48));
  const desafio = b64url(new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(verificador))));
  sessionStorage.setItem("lm_pkce", verificador);
  const q = new URLSearchParams({ provider: proveedor, redirect_to: RETORNO, code_challenge: desafio, code_challenge_method: "s256" });
  location.href = `${URL_SUPABASE}/auth/v1/authorize?${q}`;
}

async function canjearCodigo(codigo) {
  const verificador = sessionStorage.getItem("lm_pkce");
  sessionStorage.removeItem("lm_pkce");
  history.replaceState(null, "", RETORNO); // the code does not stay in the address bar
  if (!verificador || !/^[A-Za-z0-9_-]{1,300}$/.test(codigo)) throw new Error("Inicio de sesión no válido.");
  const r = await fetch(`${URL_SUPABASE}/auth/v1/token?grant_type=pkce`, {
    method: "POST",
    headers: { "Content-Type": "application/json", apikey: CLAVE_PUBLICA },
    body: JSON.stringify({ auth_code: codigo, code_verifier: verificador }),
  });
  const j = await r.json().catch(() => ({}));
  if (!r.ok || !j.access_token) throw new Error("No se pudo iniciar sesión.");
  sesion = { tipo: "persona", acceso: j.access_token };
  guardar(sesion);
}

function salir() {
  sessionStorage.clear();
  sesion = null;
  pintar();
  estado("");
}

// ───────── API ─────────
async function api(accion, cuerpo = {}) {
  if (!sesion) throw new Error("Sin sesión.");
  const credencial = sesion.tipo === "token" ? sesion.token : sesion.acceso;
  const r = await fetch(`${URL_SUPABASE}/functions/v1/revisar`, {
    method: "POST",
    headers: { "Content-Type": "application/json", Authorization: `Bearer ${credencial}`, apikey: CLAVE_PUBLICA },
    body: JSON.stringify({ accion, ...cuerpo }),
  });
  const j = await r.json().catch(() => ({}));
  if (r.status === 401) {
    salir();
    throw new Error("La sesión caducó o la credencial ya no es válida. Vuelve a entrar.");
  }
  if (!r.ok) throw new Error(j.error || `Error ${r.status}`);
  return j;
}

// ───────── screen ─────────
let pestana = "pendientes";

function pintar() {
  const dentro = !!sesion;
  $("entrada").hidden = dentro;
  $("nav").hidden = !dentro;
  $("t-tokens").hidden = !(dentro && sesion.tipo === "persona");
  $("pendientes").hidden = !(dentro && pestana === "pendientes");
  $("tokens").hidden = !(dentro && pestana === "tokens");
  $("t-pendientes").className = pestana === "pendientes" ? "activa" : "";
  $("t-tokens").className = pestana === "tokens" ? "activa" : "";
  if (dentro) (pestana === "pendientes" ? cargarPendientes() : cargarTokens());
}

async function accion(nombre, cuerpo, mensajeOk) {
  estado("Trabajando…");
  try {
    await api(nombre, cuerpo);
    estado(mensajeOk);
    await cargarPendientes(false);
  } catch (e) {
    estado(e.message, true);
  }
}

function imagen(url, alt) {
  const u = urlSegura(url);
  return u ? el("img", { src: u, alt, loading: "lazy", referrerpolicy: "no-referrer" }) : null;
}

async function verContenido(url, destino, boton) {
  const u = urlSegura(url);
  if (!u) return;
  boton.disabled = true;
  try {
    const r = await fetch(u);
    const texto = (await r.text()).slice(0, 200000);
    destino.textContent = texto; // as text: even if the file contains HTML or scripts, they do not run here
    destino.hidden = false;
  } catch {
    estado("No se pudo leer el archivo.", true);
  }
  boton.disabled = false;
}

function tarjetaContenido(p) {
  const textoArchivo = el("pre", { hidden: true });
  const motivo = el("input", { type: "text", placeholder: "Motivo del rechazo (lo verá la persona)", maxlength: "300" });
  const esTexto = ["plugin", "plantilla", "elemento", "tema"].includes(p.tipo);
  const bVer = el("button", { texto: "Ver contenido", onclick: (ev) => verContenido(p.url_temporal, textoArchivo, ev.target) });
  return el("div", { class: "tarjeta" },
    el("h2", { texto: p.titulo }, p.es_18 ? el("span", { class: "etiqueta18", texto: "+18" }) : null),
    el("div", { class: "meta", texto: `${p.tipo} · por ${p.autor_nombre || "—"} · ${p.bytes ?? "?"} bytes · ${p.creado_en}` }),
    p.motivo_rechazo ? el("div", { class: "motivo", texto: `Por qué está pendiente: ${p.motivo_rechazo}` }) : null,
    p.descripcion ? el("p", { texto: p.descripcion }) : null,
    p.etiquetas && p.etiquetas.length ? el("div", { class: "etiquetas", texto: p.etiquetas.map((e) => "#" + e).join(" ") }) : null,
    el("div", { class: "imagenes" }, p.tipo === "imagen" ? imagen(p.url_temporal, "Imagen enviada") : null, imagen(p.url_icono, "Icono"), imagen(p.url_vista, "Vista previa")),
    esTexto ? bVer : null,
    p.tipo === "fuente" && urlSegura(p.url_temporal) ? el("a", { href: urlSegura(p.url_temporal), rel: "noopener noreferrer", target: "_blank", texto: "Descargar la fuente (enlace temporal)" }) : null,
    textoArchivo,
    el("div", { class: "acciones" },
      el("button", { class: "ok", texto: "Aprobar", onclick: () => accion("aprobar", { item: p.id }, "Aprobado y publicado.") }),
      el("button", { class: "ok", texto: "Aprobar como +18", onclick: () => accion("aprobar", { item: p.id, es_18: true }, "Aprobado como +18.") }),
      motivo,
      el("button", { class: "mal", texto: "Rechazar", onclick: () => accion("rechazar", { item: p.id, motivo: motivo.value.trim() || "Rechazado en la revisión." }, "Rechazado.") })));
}

function tarjetaPerfil(p) {
  const motivo = el("input", { type: "text", placeholder: "Motivo (opcional)", maxlength: "300" });
  return el("div", { class: "tarjeta" },
    el("h2", { texto: `${p.clase === "avatar" ? "Foto de perfil" : "Banner"} de ${p.nombre_publico}` }),
    p.bio ? el("p", { class: "meta", texto: p.bio }) : null,
    el("div", { class: "imagenes" }, imagen(p.url_temporal, p.clase)),
    el("div", { class: "acciones" },
      el("button", { class: "ok", texto: "Aprobar", onclick: () => accion("aprobar_perfil", { usuario: p.usuario, clase: p.clase }, "Imagen de perfil aprobada.") }),
      motivo,
      el("button", { class: "mal", texto: "Rechazar", onclick: () => accion("rechazar_perfil", { usuario: p.usuario, clase: p.clase }, "Imagen de perfil rechazada.") })));
}

const CATEGORIAS = [
  ["plugin", "Plugins"], ["tema", "Temas"], ["plantilla", "Plantillas"], ["elemento", "Elementos"], ["fuente", "Fuentes"], ["imagen", "Imágenes"],
  ["perfil", "Fotos de perfil y banners"],
];
let categoria = "todo";
let ultimo = null; // last list received, to switch category without requesting it again

function pintarPendientes() {
  const caja = $("pendientes");
  if (!ultimo) return;
  const porCategoria = {};
  for (const p of ultimo.pendientes) (porCategoria[p.tipo] ||= []).push(p);
  porCategoria.perfil = ultimo.perfiles;
  const total = ultimo.pendientes.length + ultimo.perfiles.length;
  const chips = [["todo", "Todo", total], ...CATEGORIAS.map(([k, n]) => [k, n, (porCategoria[k] || []).length])];
  const barra = el("div", { class: "fila" }, ...chips.map(([k, n, c]) =>
    el("button", { class: categoria === k ? "activa" : "", texto: `${n} (${c})`, onclick: () => { categoria = k; pintarPendientes(); } })));
  const secciones = [];
  for (const [k, nombre] of CATEGORIAS) {
    const lista = porCategoria[k] || [];
    if (!lista.length || (categoria !== "todo" && categoria !== k)) continue;
    secciones.push(el("h3", { class: "seccion", texto: `${nombre} (${lista.length})` }));
    secciones.push(...lista.map(k === "perfil" ? tarjetaPerfil : tarjetaContenido));
  }
  caja.replaceChildren(
    el("p", { class: "aviso", texto: `${total} pendiente(s) en total. ${ultimo.nota || ""}` }),
    el("div", { class: "fila" }, el("button", { texto: "Actualizar", onclick: () => cargarPendientes() })),
    barra,
    ...(secciones.length ? secciones : [el("p", { class: "meta", texto: "No hay nada pendiente en esta categoría." })]));
}

async function cargarPendientes(avisar = true) {
  try {
    ultimo = await api("listar");
    pintarPendientes();
    if (avisar) estado("");
  } catch (e) {
    estado(e.message, true);
  }
}

// ───────── tokens (people only) ─────────
async function cargarTokens() {
  const caja = $("tokens");
  try {
    const { tokens } = await api("listar_tokens");
    const nombre = el("input", { type: "text", placeholder: "Nombre (p. ej. «Claude revisor»)", maxlength: "60" });
    const dias = el("input", { type: "number", min: "1", max: "90", value: "30" });
    const marcas = ["leer", "aprobar", "rechazar"].map((a) => ({ a, c: el("input", { type: "checkbox", checked: a !== "aprobar" }) }));
    const nuevo = el("div");
    const crear = el("button", { texto: "Crear token", onclick: async () => {
      try {
        const alcances = marcas.filter((m) => m.c.checked).map((m) => m.a);
        const j = await api("crear_token", { nombre: nombre.value.trim(), alcances, dias: Number(dias.value) });
        const campo = el("input", { type: "text", readonly: true, value: j.token });
        nuevo.replaceChildren(el("div", { class: "token-nuevo" },
          el("strong", { texto: "Token creado. Cópialo ahora: no se puede volver a ver." }), campo,
          el("div", { class: "meta", texto: `Caduca: ${j.expira_en}` })));
        campo.select();
        estado("Token creado.");
        await cargarTokens(); // refreshes the list but keeps the new token notice
        $("tokens").prepend(nuevo);
      } catch (e) { estado(e.message, true); }
    } });
    caja.replaceChildren(
      el("p", { class: "aviso", texto: "Credenciales para automatizaciones y scripts. Dales solo lo necesario: «leer» para listar, «rechazar» para retirar contenido y «aprobar» solo si confías en el script. Un token no puede crear otros tokens." }),
      el("label", { texto: "Nombre" }), nombre,
      el("label", { texto: "Alcances" }), el("div", { class: "fila" }, ...marcas.map((m) => el("label", {}, m.c, ` ${m.a}`))),
      el("label", { texto: "Caduca en (días, máx. 90)" }), dias,
      el("div", { class: "fila" }, crear),
      nuevo,
      el("table", {},
        el("thead", {}, el("tr", {}, ...["Nombre", "Alcances", "Caduca", "Último uso", ""].map((t) => el("th", { texto: t })))),
        el("tbody", {}, ...tokens.map((t) => el("tr", {},
          el("td", { texto: t.nombre }), el("td", { texto: t.alcances.join(", ") }), el("td", { texto: t.expira_en.slice(0, 10) }),
          el("td", { texto: t.ultimo_uso ? t.ultimo_uso.slice(0, 16).replace("T", " ") : "nunca" }),
          el("td", {}, t.revocado ? el("span", { class: "meta", texto: "revocado" }) : el("button", { class: "mal", texto: "Revocar", onclick: async () => {
            try { await api("revocar_token", { id: t.id }); estado("Token revocado."); await cargarTokens(); } catch (e) { estado(e.message, true); }
          } })))))));
  } catch (e) {
    estado(e.message, true);
  }
}

// ───────── startup ─────────
$("github").addEventListener("click", () => entrarOAuth("github"));
$("google").addEventListener("click", () => entrarOAuth("google"));
$("salir").addEventListener("click", salir);
$("t-pendientes").addEventListener("click", () => { pestana = "pendientes"; pintar(); });
$("t-tokens").addEventListener("click", () => { pestana = "tokens"; pintar(); });
$("entrar-token").addEventListener("click", () => {
  const t = $("token").value.trim();
  $("token").value = "";
  if (!/^lm_[A-Za-z0-9_-]{43}$/.test(t)) return estado("Ese no parece un token de moderación (lm_…).", true);
  sesion = { tipo: "token", token: t };
  guardar(sesion);
  estado("");
  pintar();
});

(async () => {
  const q = new URLSearchParams(location.search);
  if (q.get("code")) {
    try { await canjearCodigo(q.get("code")); } catch (e) { estado(e.message, true); }
  } else if (q.get("error")) {
    history.replaceState(null, "", RETORNO);
    estado("Se canceló el inicio de sesión.", true);
  }
  pintar();
})();
