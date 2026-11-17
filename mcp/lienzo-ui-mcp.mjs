#!/usr/bin/env node
// Lienzo UI — servidor MCP (Model Context Protocol) sin dependencias. Requiere Node 18+.
// Transporte stdio (JSON-RPC 2.0, un mensaje por línea).
//
// Permite que cualquier IA compatible con MCP (Claude, Cursor, VS Code, Windsurf, etc.) cree y
// edite diseños de Lienzo UI, los exporte a un proyecto de Unity y saque capturas.
// Configuración: docs/MCP.md
//
// Seguridad: los diseños solo se leen y escriben dentro de la carpeta de diseños
// (LIENZO_DISENOS, por defecto <repo>/disenos); la exportación solo escribe en
// <proyecto Unity>/Assets/EditorGUI (lo hace el propio Lienzo UI).

import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import readline from "node:readline";
import { fileURLToPath } from "node:url";

const VERSION = "1.0.0";
const RAIZ = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const DISENOS = path.resolve(process.env.LIENZO_DISENOS || path.join(RAIZ, "disenos"));
const EXT = ".editorgui.json";
const TIPOS = ["Panel", "Ellipse", "Image", "Text", "Button", "Toggle", "Slider", "InputField", "Dropdown", "ScrollView", "Empty"];

const CAMPOS_EDITABLES = {
  name: "string", text: "string", x: "number", y: "number", w: "number", h: "number", rot: "number",
  fill: "color", stroke: "color", text_color: "color", stroke_w: "number", radius: "number", opacity: "number",
  font_size: "number", bold: "boolean", italic: "boolean", underline: "boolean", visible: "boolean",
  value: "number", on_click: "string", options: "strings", interactable: "boolean", image: "image",
};

// ---------- utilidades ----------

function binario() {
  const candidatos = [
    process.env.LIENZO_UI,
    path.join(RAIZ, "lienzo-ui.exe"),
    path.join(RAIZ, "target", "release", process.platform === "win32" ? "lienzo-ui.exe" : "lienzo-ui"),
    path.join(RAIZ, "target", "debug", process.platform === "win32" ? "lienzo-ui.exe" : "lienzo-ui"),
  ].filter(Boolean);
  const b = candidatos.find((c) => fs.existsSync(c));
  if (!b) throw new Error("No encontré lienzo-ui. Compílalo con «cargo build --release» o define LIENZO_UI con la ruta al ejecutable.");
  return b;
}

function ejecutar(args, timeoutMs = 120_000) {
  return new Promise((resolve) => {
    const p = spawn(binario(), args, { cwd: RAIZ, windowsHide: true });
    let out = "", err = "";
    p.stdout.on("data", (d) => (out += d));
    p.stderr.on("data", (d) => (err += d));
    const t = setTimeout(() => p.kill(), timeoutMs);
    p.on("error", (e) => { clearTimeout(t); resolve({ codigo: -1, out, err: String(e) }); });
    p.on("close", (codigo) => { clearTimeout(t); resolve({ codigo, out: out.trim(), err: err.trim() }); });
  });
}

/** Ruta segura de un diseño: solo un nombre de archivo dentro de la carpeta de diseños. */
function rutaDiseno(nombre) {
  if (typeof nombre !== "string" || !/^[\p{L}\p{N} _.-]{1,80}$/u.test(nombre) || nombre.includes(".."))
    throw new Error("'archivo' debe ser un nombre simple (letras, números, espacios, _ . -), sin carpetas.");
  const archivo = nombre.endsWith(EXT) ? nombre : nombre + EXT;
  if (/^(con|prn|aux|nul|com\d|lpt\d)(\.|$)/i.test(archivo) || /[. ]$/.test(nombre)) throw new Error("'archivo' usa un nombre reservado de Windows o termina en punto o espacio.");
  const abs = path.resolve(DISENOS, archivo);
  if (path.dirname(abs) !== DISENOS) throw new Error("Ruta fuera de la carpeta de diseños.");
  return abs;
}

function leer(nombre) {
  const abs = rutaDiseno(nombre);
  if (!fs.existsSync(abs)) throw new Error(`No existe el diseño ${path.basename(abs)}. Usa listar_disenos.`);
  let doc;
  try { doc = JSON.parse(fs.readFileSync(abs, "utf8")); } catch (err) { throw new Error(`${path.basename(abs)} no es un JSON válido (${err.message}).`); }
  if (!doc || typeof doc !== "object" || !Array.isArray(doc.elements)) throw new Error(`${path.basename(abs)} no parece un diseño de Lienzo UI (falta la lista 'elements').`);
  return { abs, doc };
}

/** Rangos válidos de campos numéricos: [mín, máx, exclusivoMín]. */
const RANGOS = { w: [0, 100000, true], h: [0, 100000, true], font_size: [0, 1000, true], opacity: [0, 1], stroke_w: [0, 1000], radius: [0, 100000], x: [-1e6, 1e6], y: [-1e6, 1e6] };
function validarRango(k, v) {
  const r = RANGOS[k];
  if (r && (v < r[0] || v > r[1] || (r[2] && v === r[0]))) throw new Error(`${k} fuera de rango (${r[2] ? ">" : "≥"} ${r[0]} y ≤ ${r[1]}).`);
}

function guardar(abs, doc) {
  fs.writeFileSync(abs, JSON.stringify(doc, null, 2), "utf8");
}

function color(v) {
  if (Array.isArray(v) && v.length >= 3 && v.every((n) => Number.isInteger(n) && n >= 0 && n <= 255)) return v.length === 3 ? [...v, 255] : v.slice(0, 4);
  const m = typeof v === "string" && v.match(/^#?([0-9a-f]{6})([0-9a-f]{2})?$/i);
  if (!m) throw new Error(`Color no válido: ${JSON.stringify(v)} (usa "#RRGGBB", "#RRGGBBAA" o [r,g,b,a])`);
  const n = parseInt(m[1], 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255, m[2] ? parseInt(m[2], 16) : 255];
}

/** Ruta de imagen válida: absoluta, PNG o JPG y existente (o "" para quitarla). */
function rutaImagen(v) {
  if (typeof v !== "string") throw new Error("image debe ser la ruta absoluta de un PNG o JPG, o \"\" para quitarla");
  if (v && (!path.isAbsolute(v) || !/\.(png|jpe?g)$/i.test(v) || !fs.existsSync(v))) throw new Error(`image: no existe o no es PNG/JPG: ${v}`);
  return v;
}

/** Límites de tamaño: un diseño con textos o listas enormes dejaría la app minutos sin responder. */
const LIM = { nombre: 120, texto: 5000, lista: 200, item: 200 };
function limitar(campo, v, max) {
  if (typeof v === "string" && v.length > max) throw new Error(`${campo} admite como máximo ${max} caracteres.`);
  return v;
}

function buscar(doc, ref) {
  // Primero el nombre exacto; solo si no hay ninguno se interpreta como id (un elemento llamado «3» no debe perder ante el id 3).
  const e = doc.elements.find((x) => x.name === ref) ?? doc.elements.find((x) => String(x.id) === String(ref));
  if (!e) throw new Error(`No hay ningún elemento «${ref}». Usa leer_diseno para ver los nombres.`);
  return e;
}

function descendientes(doc, id) {
  const ids = new Set([id]);
  let cambio = true;
  while (cambio) {
    cambio = false;
    for (const e of doc.elements) if (e.parent != null && ids.has(e.parent) && !ids.has(e.id)) { ids.add(e.id); cambio = true; }
  }
  return ids;
}

// ---------- herramientas ----------

const HERRAMIENTAS = [
  {
    name: "info_lienzo",
    description: "Qué es Lienzo UI, dónde están los diseños y el flujo recomendado. Llámala primero.",
    inputSchema: { type: "object", properties: {} },
    run: () => [
      `Lienzo UI v${VERSION} (servidor MCP). Editor visual de interfaces de juego que exporta a Unity.`,
      `Carpeta de diseños: ${DISENOS}`,
      "Flujo: listar_plantillas → crear_diseno → leer_diseno → modificar/duplicar/eliminar elementos → captura → exportar_unity.",
      "Coordenadas en píxeles de la resolución de referencia (normalmente 1920×1080), origen arriba a la izquierda.",
      "Tipos de elemento: Panel, Ellipse, Image, Text, Button, Toggle, Slider, InputField, Dropdown, ScrollView, Empty.",
      "En Unity: el diseño aparece como prefab en Assets/EditorGUI/Prefabs (uGUI + TextMeshPro), además de UXML/USS (UI Toolkit) y un script OnGUI (IMGUI).",
      "Campos editables: " + Object.keys(CAMPOS_EDITABLES).join(", "),
    ].join("\n"),
  },
  {
    name: "listar_plantillas",
    description: "Lista las plantillas incluidas (número y nombre).",
    inputSchema: { type: "object", properties: {} },
    run: async () => (await ejecutar(["--plantillas"])).out,
  },
  {
    name: "crear_diseno",
    description: "Crea un diseño nuevo a partir de una plantilla (usa 5 para «En blanco») y lo guarda en la carpeta de diseños.",
    inputSchema: {
      type: "object",
      properties: { archivo: { type: "string" }, plantilla: { type: "integer", minimum: 0 }, titulo: { type: "string" } },
      required: ["archivo", "plantilla"],
    },
    run: async ({ archivo, plantilla, titulo }) => {
      const abs = rutaDiseno(archivo);
      if (fs.existsSync(abs)) throw new Error(`Ya existe ${path.basename(abs)}.`);
      if (!Number.isInteger(plantilla) || plantilla < 0) throw new Error("'plantilla' debe ser un entero ≥ 0.");
      fs.mkdirSync(DISENOS, { recursive: true });
      const r = await ejecutar(["--plantilla", String(plantilla), "--guardar", abs]);
      if (r.codigo !== 0) throw new Error(r.err || "No se pudo crear el diseño.");
      if (titulo) {
        if (typeof titulo !== "string" || titulo.length > 80) throw new Error("'titulo' debe ser texto de hasta 80 caracteres.");
        const doc = JSON.parse(fs.readFileSync(abs, "utf8"));
        doc.title = titulo;
        guardar(abs, doc);
      }
      return `Creado ${path.basename(abs)}.`;
    },
  },
  {
    name: "ajustar_diseno",
    description: "Ajustes del lienzo: titulo, pixel_art (imágenes sin suavizado, píxeles enteros; en Unity Canvas Pixel Perfect y sprites Point), fondo (\"#RRGGBB\") y match (0 = ancho, 1 = alto).",
    inputSchema: {
      type: "object",
      properties: { archivo: { type: "string" }, titulo: { type: "string" }, pixel_art: { type: "boolean" }, fondo: { type: "string" }, match: { type: "number" } },
      required: ["archivo"],
    },
    run: ({ archivo, titulo, pixel_art, fondo, match }) => {
      const { abs, doc } = leer(archivo);
      const hechos = [];
      if (titulo !== undefined) { if (typeof titulo !== "string" || !titulo.trim() || titulo.length > 80) throw new Error("titulo: texto de 1 a 80 caracteres"); doc.title = titulo; hechos.push("titulo"); }
      if (pixel_art !== undefined) { if (typeof pixel_art !== "boolean") throw new Error("pixel_art debe ser true o false"); doc.pixel_art = pixel_art; hechos.push("pixel_art"); }
      if (fondo !== undefined) { doc.background = color(fondo); hechos.push("fondo"); }
      if (match !== undefined) { if (typeof match !== "number" || match < 0 || match > 1) throw new Error("match entre 0 y 1"); doc.canvas_match = match; hechos.push("match"); }
      if (pixel_art) for (const e of doc.elements) { e.x = Math.round(e.x); e.y = Math.round(e.y); e.w = Math.max(1, Math.round(e.w)); e.h = Math.max(1, Math.round(e.h)); }
      guardar(abs, doc);
      return hechos.length ? `Ajustado: ${hechos.join(", ")}.` : "Sin cambios.";
    },
  },
  {
    name: "listar_disenos",
    description: "Lista los diseños guardados en la carpeta de diseños.",
    inputSchema: { type: "object", properties: {} },
    run: () => (fs.existsSync(DISENOS) ? fs.readdirSync(DISENOS).filter((f) => f.endsWith(EXT)).join("\n") : "") || "(no hay diseños)",
  },
  {
    name: "leer_diseno",
    description: "Resumen de un diseño: título, resolución y la lista de elementos (id, nombre, tipo, posición, tamaño, texto, padre).",
    inputSchema: { type: "object", properties: { archivo: { type: "string" } }, required: ["archivo"] },
    run: ({ archivo }) => {
      const { doc } = leer(archivo);
      const filas = doc.elements.map((e) =>
        `${e.id}\t${e.name}\t${e.kind}\t(${Math.round(e.x)}, ${Math.round(e.y)})\t${Math.round(e.w)}×${Math.round(e.h)}\t${e.text ? JSON.stringify(e.text) : ""}\t${e.parent ?? ""}${e.visible === false ? "\toculto" : ""}`);
      return [`«${doc.title}» · ${doc.width}×${doc.height} · ${doc.elements.length} elementos`, "id\tnombre\ttipo\tposición\ttamaño\ttexto\tpadre", ...filas].join("\n");
    },
  },
  {
    name: "modificar_elemento",
    description: "Cambia propiedades de un elemento (por nombre o id). 'cambios' admite: " + Object.keys(CAMPOS_EDITABLES).join(", ") + ". Colores como \"#RRGGBB\".",
    inputSchema: {
      type: "object",
      properties: { archivo: { type: "string" }, elemento: { type: "string" }, cambios: { type: "object" } },
      required: ["archivo", "elemento", "cambios"],
    },
    run: ({ archivo, elemento, cambios }) => {
      const { abs, doc } = leer(archivo);
      const e = buscar(doc, elemento);
      if (!cambios || typeof cambios !== "object" || Array.isArray(cambios)) throw new Error("'cambios' debe ser un objeto.");
      const hechos = [];
      for (const [k, v] of Object.entries(cambios)) {
        const tipo = Object.hasOwn(CAMPOS_EDITABLES, k) ? CAMPOS_EDITABLES[k] : null;
        if (!tipo) throw new Error(`Campo no editable: ${k}`);
        if (tipo === "color") e[k] = color(v);
        else if (tipo === "image") {
          if (e.kind !== "Image") throw new Error(`«${e.name}» es ${e.kind}: solo los elementos Image muestran imagen. Usa agregar_elemento con tipo Image (por ejemplo dentro de «${e.name}»).`);
          e[k] = rutaImagen(v);
        }
        else if (tipo === "strings") { if (!Array.isArray(v) || !v.every((s) => typeof s === "string")) throw new Error(`${k} debe ser una lista de textos`);
          if (v.length > LIM.lista || v.some((s) => s.length > LIM.item)) throw new Error(`${k} admite como máximo ${LIM.lista} elementos de ${LIM.item} caracteres.`); e[k] = v; }
        else if (typeof v !== tipo || (tipo === "number" && !Number.isFinite(v))) throw new Error(`${k} debe ser ${tipo}`);
        else {
          if (tipo === "string") limitar(k, v, k === "name" ? LIM.nombre : LIM.texto);
          if (tipo === "number") validarRango(k, v);
          if (k === "name") {
            if (!v.trim()) throw new Error("name no puede estar vacío.");
            if (doc.elements.some((x) => x !== e && x.name === v)) throw new Error(`Ya existe un elemento «${v}».`);
          }
          e[k] = v;
        }
        hechos.push(k);
      }
      guardar(abs, doc);
      return `«${e.name}» actualizado: ${hechos.join(", ")}. (Las anclas no cambian; si mueves un hijo, sus coordenadas siguen siendo absolutas.)`;
    },
  },
  {
    name: "agregar_elemento",
    description: "Añade un elemento nuevo, opcionalmente dentro de otro (padre). Tipos: " + TIPOS.join(", ") + ". Para iconos o pixel art usa tipo Image con 'image' (ruta absoluta a un PNG/JPG). Posición y tamaño en píxeles del lienzo.",
    inputSchema: {
      type: "object",
      properties: {
        archivo: { type: "string" }, tipo: { type: "string", enum: TIPOS }, nombre: { type: "string" }, padre: { type: "string" },
        x: { type: "number" }, y: { type: "number" }, w: { type: "number" }, h: { type: "number" },
        texto: { type: "string" }, image: { type: "string" }, color: { type: "string" },
      },
      required: ["archivo", "tipo", "nombre", "x", "y", "w", "h"],
    },
    run: ({ archivo, tipo, nombre, padre, x, y, w, h, texto, image, color: c }) => {
      const { abs, doc } = leer(archivo);
      if (!TIPOS.includes(tipo)) throw new Error(`tipo no válido. Usa: ${TIPOS.join(", ")}`);
      if (typeof nombre !== "string" || !nombre.trim()) throw new Error("Falta 'nombre'.");
      limitar("nombre", nombre, LIM.nombre);
      limitar("texto", texto, LIM.texto);
      if (doc.elements.some((e) => e.name === nombre)) throw new Error(`Ya existe un elemento «${nombre}».`);
      for (const [k, v] of Object.entries({ x, y, w, h })) if (typeof v !== "number" || !Number.isFinite(v)) throw new Error(`${k} debe ser un número`);
      if (w <= 0 || h <= 0) throw new Error("w y h deben ser mayores que 0");
      if (image !== undefined && tipo !== "Image") throw new Error("'image' solo se usa con tipo Image.");
      const p = padre !== undefined ? buscar(doc, padre) : null;
      // Base: un elemento del mismo tipo (mismos valores por defecto) o, si no hay, cualquiera ya existente.
      const base = doc.elements.find((e) => e.kind === tipo) || doc.elements[0];
      if (!base) throw new Error("El diseño está vacío: crea el diseño desde una plantilla que tenga elementos.");
      const e = structuredClone(base);
      Object.assign(e, {
        id: doc.next_id++, name: nombre, kind: tipo, x, y, w, h, rot: 0, parent: p ? p.id : null, group: null,
        visible: true, locked: false, collapsed: false, text: texto ?? (tipo === "Text" || tipo === "Button" ? nombre : ""),
        image: image !== undefined ? rutaImagen(image) : "", anchor_min: [0.5, 0.5], anchor_max: [0.5, 0.5], pivot: [0.5, 0.5],
        on_click: "", opacity: 1,
      });
      e.layout = { ...e.layout, mode: "None" };
      if (e.shadow) e.shadow.on = false;
      if (tipo === "Image") { e.fill = [255, 255, 255, 255]; e.fill2 = [255, 255, 255, 255]; e.stroke_w = 0; e.radius = 0; e.raycast = false; }
      if (c !== undefined) e.fill = color(c);
      if (doc.pixel_art) { e.x = Math.round(e.x); e.y = Math.round(e.y); e.w = Math.round(e.w); e.h = Math.round(e.h); }
      // Se dibuja encima del padre y de sus hijos actuales.
      const ids = p ? descendientes(doc, p.id) : null;
      const ultimo = ids ? Math.max(...doc.elements.map((x, i) => (ids.has(x.id) ? i : -1))) : doc.elements.length - 1;
      doc.elements.splice(ultimo + 1, 0, e);
      guardar(abs, doc);
      return `Añadido «${nombre}» (${tipo})${p ? ` dentro de «${p.name}»` : ""}.`;
    },
  },
  {
    name: "duplicar_elemento",
    description: "Duplica un elemento (con sus hijos) con otro nombre y lo desplaza dx, dy píxeles. Sirve para añadir botones, casillas, etc. del mismo estilo.",
    inputSchema: {
      type: "object",
      properties: { archivo: { type: "string" }, elemento: { type: "string" }, nuevo_nombre: { type: "string" }, dx: { type: "number" }, dy: { type: "number" }, texto: { type: "string" } },
      required: ["archivo", "elemento", "nuevo_nombre"],
    },
    run: ({ archivo, elemento, nuevo_nombre, dx = 0, dy = 0, texto }) => {
      const { abs, doc } = leer(archivo);
      const base = buscar(doc, elemento);
      if (typeof nuevo_nombre !== "string" || !nuevo_nombre.trim()) throw new Error("Falta 'nuevo_nombre'.");
      limitar("nuevo_nombre", nuevo_nombre, LIM.nombre);
      limitar("texto", texto, LIM.texto);
      if (doc.elements.some((x) => x.name === nuevo_nombre)) throw new Error(`Ya existe un elemento «${nuevo_nombre}».`);
      const ids = descendientes(doc, base.id);
      const mapa = new Map();
      for (const id of ids) mapa.set(id, doc.next_id++);
      const copias = doc.elements.filter((x) => ids.has(x.id)).map((x) => {
        const c = structuredClone(x);
        c.id = mapa.get(x.id);
        if (c.parent != null && mapa.has(c.parent)) c.parent = mapa.get(c.parent);
        if (c.group != null && mapa.has(c.group)) c.group = mapa.get(c.group);
        c.x += dx; c.y += dy;
        return c;
      });
      copias[0].name = nuevo_nombre;
      if (texto !== undefined) copias[0].text = String(texto);
      const pos = doc.elements.findIndex((x) => x.id === base.id);
      const ultimo = Math.max(...doc.elements.map((x, i) => (ids.has(x.id) ? i : -1)));
      doc.elements.splice(Math.max(pos, ultimo) + 1, 0, ...copias);
      guardar(abs, doc);
      return `Creado «${nuevo_nombre}» (${copias.length} elemento(s)) a partir de «${base.name}».`;
    },
  },
  {
    name: "eliminar_elemento",
    description: "Elimina un elemento y sus hijos.",
    inputSchema: { type: "object", properties: { archivo: { type: "string" }, elemento: { type: "string" } }, required: ["archivo", "elemento"] },
    run: ({ archivo, elemento }) => {
      const { abs, doc } = leer(archivo);
      const e = buscar(doc, elemento);
      const ids = descendientes(doc, e.id);
      doc.elements = doc.elements.filter((x) => !ids.has(x.id));
      guardar(abs, doc);
      return `Eliminado «${e.name}» (${ids.size} elemento(s)).`;
    },
  },
  {
    name: "captura",
    description: "Abre el diseño en Lienzo UI y guarda una captura PNG del editor en la carpeta de diseños (capturas/<archivo>.png).",
    inputSchema: { type: "object", properties: { archivo: { type: "string" } }, required: ["archivo"] },
    run: async ({ archivo }) => {
      const { abs } = leer(archivo);
      const salida = path.join(DISENOS, "capturas", path.basename(abs, EXT) + ".png");
      fs.mkdirSync(path.dirname(salida), { recursive: true });
      const r = await ejecutar(["--abrir", abs, "--sin-tutorial", "--screenshot", salida], 90_000);
      if (r.codigo !== 0 || !fs.existsSync(salida)) throw new Error("No se pudo capturar. " + r.err);
      return `Captura guardada en ${salida}`;
    },
  },
  {
    name: "exportar_unity",
    description: "Exporta el diseño a un proyecto de Unity (la carpeta que contiene Assets). Escribe en Assets/EditorGUI: diseño, importador, UXML/USS, script IMGUI y vistas. Unity crea el prefab al volver a la ventana.",
    inputSchema: { type: "object", properties: { archivo: { type: "string" }, proyecto_unity: { type: "string" } }, required: ["archivo", "proyecto_unity"] },
    run: async ({ archivo, proyecto_unity }) => {
      const { abs } = leer(archivo);
      if (typeof proyecto_unity !== "string" || !fs.existsSync(path.join(proyecto_unity, "Assets")))
        throw new Error("'proyecto_unity' debe ser la carpeta de un proyecto de Unity (con carpeta Assets).");
      const r = await ejecutar(["--abrir", abs, "--exportar", proyecto_unity]);
      if (r.codigo !== 0) throw new Error(r.err || "No se pudo exportar.");
      return "Archivos escritos:\n" + r.out;
    },
  },
];

// ---------- protocolo MCP (JSON-RPC 2.0 por stdio) ----------

let cola = Promise.resolve();
const enviar = (m) => { if (!process.stdout.destroyed) process.stdout.write(JSON.stringify(m) + "\n"); };
process.stdout.on("error", () => process.exit(0)); // el cliente cerró la conexión
const responder = (id, result) => enviar({ jsonrpc: "2.0", id, result });
const fallo = (id, code, message) => enviar({ jsonrpc: "2.0", id, error: { code, message } });

async function manejar(m) {
  const { id, method, params } = m;
  const notificacion = id === undefined || id === null;
  try {
    switch (method) {
      case "initialize":
        return responder(id, {
          protocolVersion: params?.protocolVersion || "2025-06-18",
          capabilities: { tools: { listChanged: false } },
          serverInfo: { name: "lienzo-ui", version: VERSION },
          instructions: "Servidor de Lienzo UI (editor de interfaces para Unity). Empieza con info_lienzo.",
        });
      case "notifications/initialized":
      case "notifications/cancelled":
        return;
      case "ping":
        return responder(id, {});
      case "tools/list":
        return responder(id, { tools: HERRAMIENTAS.map(({ name, description, inputSchema }) => ({ name, description, inputSchema })) });
      case "tools/call": {
        const h = HERRAMIENTAS.find((x) => x.name === params?.name);
        if (!h) return fallo(id, -32602, `Herramienta desconocida: ${params?.name}`);
        // Las llamadas se ejecutan de una en una: dos órdenes simultáneas sobre el mismo diseño no se pisan.
        const turno = cola.then(async () => {
          try {
            responder(id, { content: [{ type: "text", text: String(await h.run(params?.arguments || {})) }] });
          } catch (e) {
            responder(id, { content: [{ type: "text", text: `Error: ${e.message}` }], isError: true });
          }
        });
        cola = turno;
        return turno;
      }
      default:
        if (!notificacion) fallo(id, -32601, `Método no soportado: ${method}`);
    }
  } catch (e) {
    if (!notificacion) fallo(id, -32603, String(e?.message || e));
  }
}

readline.createInterface({ input: process.stdin }).on("line", (l) => {
  if (!l.trim()) return;
  let m;
  try { m = JSON.parse(l); } catch { return fallo(null, -32700, "JSON inválido"); }
  if (Array.isArray(m)) m.forEach(manejar); else manejar(m);
});
