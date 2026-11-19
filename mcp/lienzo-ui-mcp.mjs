#!/usr/bin/env node
// Lienzo UI — MCP (Model Context Protocol) server with no dependencies. Requires Node 18+.
// stdio transport (JSON-RPC 2.0, one message per line).
//
// Lets any MCP-compatible AI (Claude, Cursor, VS Code, Windsurf, etc.) create and
// edit Lienzo UI designs, export them to a Unity project and take screenshots.
// Setup: docs/MCP.md
//
// Security: designs are only read and written inside the designs folder
// (LIENZO_DESIGNS, default <repo>/designs); export only writes to
// <Unity project>/Assets/EditorGUI (done by Lienzo UI itself).
//
// Legacy names: the old Spanish tool names (info_lienzo, crear_diseno, captura…) and parameter names
// (archivo, elemento, cambios…) still work as hidden aliases; they are not listed in tools/list.
// The old LIENZO_DISENOS variable and an existing <repo>/disenos folder are still honored.

import { spawn } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import readline from "node:readline";
import { fileURLToPath } from "node:url";

const VERSION = "1.0.0";
const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
/** Designs folder: LIENZO_DESIGNS, else the legacy LIENZO_DISENOS, else <repo>/designs (or an old <repo>/disenos if that is what exists). */
function designsFolder() {
  const env = process.env.LIENZO_DESIGNS || process.env.LIENZO_DISENOS;
  if (env) return path.resolve(env);
  const fresh = path.join(ROOT, "designs"), legacy = path.join(ROOT, "disenos");
  return fs.existsSync(fresh) || !fs.existsSync(legacy) ? fresh : legacy;
}
const DESIGNS = designsFolder();
const EXT = ".editorgui.json";
const TYPES = ["Panel", "Ellipse", "Image", "Text", "Button", "Toggle", "Slider", "InputField", "Dropdown", "ScrollView", "Empty"];

const EDITABLE_FIELDS = {
  name: "string", text: "string", x: "number", y: "number", w: "number", h: "number", rot: "number",
  fill: "color", stroke: "color", text_color: "color", stroke_w: "number", radius: "number", opacity: "number",
  font_size: "number", bold: "boolean", italic: "boolean", underline: "boolean", visible: "boolean",
  value: "number", on_click: "string", options: "strings", interactable: "boolean", image: "image",
};

// Old Spanish names, kept working as hidden aliases (not listed in tools/list).
const LEGACY_TOOLS = {
  info_lienzo: "lienzo_info", listar_plantillas: "list_templates", crear_diseno: "create_design", ajustar_diseno: "adjust_design",
  revisar_diseno: "review_design", listar_disenos: "list_designs", leer_diseno: "read_design", modificar_elemento: "modify_element",
  agregar_elemento: "add_element", duplicar_elemento: "duplicate_element", eliminar_elemento: "delete_element",
  captura: "screenshot", exportar_unity: "export_unity",
};
const LEGACY_PARAMS = {
  archivo: "file", plantilla: "template", titulo: "title", fondo: "background", elemento: "element", cambios: "changes",
  tipo: "type", nombre: "name", padre: "parent", texto: "text", nuevo_nombre: "new_name", proyecto_unity: "unity_project",
};
/** Renames old Spanish parameters to the English ones (the English name wins if both are given). */
function normalizeArgs(args) {
  const out = Object.create(null);
  for (const [k, v] of Object.entries(args || {})) {
    const key = Object.hasOwn(LEGACY_PARAMS, k) ? LEGACY_PARAMS[k] : k;
    if (key !== k && Object.hasOwn(args, key)) continue;
    out[key] = v;
  }
  return out;
}


// ---------- utilities ----------

function appBinary() {
  const exe = process.platform === "win32" ? "lienzo-ui.exe" : "lienzo-ui";
  const candidates = [
    process.env.LIENZO_UI,
    path.join(ROOT, "lienzo-ui.exe"),
    path.join(ROOT, "target", "release", exe),
    path.join(ROOT, "target", "debug", exe),
  ].filter(Boolean);
  const found = candidates.find((c) => fs.existsSync(c));
  if (!found) throw new Error("lienzo-ui not found. Build it with \"cargo build --release\" or set LIENZO_UI to the path of the executable.");
  return found;
}

function runApp(args, timeoutMs = 120_000) {
  return new Promise((resolve) => {
    const p = spawn(appBinary(), args, { cwd: ROOT, windowsHide: true });
    let out = "", err = "";
    p.stdout.on("data", (d) => (out += d));
    p.stderr.on("data", (d) => (err += d));
    const t = setTimeout(() => p.kill(), timeoutMs);
    p.on("error", (e) => { clearTimeout(t); resolve({ code: -1, out, err: String(e) }); });
    p.on("close", (code) => { clearTimeout(t); resolve({ code, out: out.trim(), err: err.trim() }); });
  });
}

/** Safe path of a design: only a file name inside the designs folder. */
function designPath(name) {
  if (typeof name !== "string" || !/^[\p{L}\p{N} _.-]{1,80}$/u.test(name) || name.includes(".."))
    throw new Error("'file' must be a plain name (letters, numbers, spaces, _ . -), without folders.");
  const file = name.endsWith(EXT) ? name : name + EXT;
  if (/^(con|prn|aux|nul|com\d|lpt\d)(\.|$)/i.test(file) || /[. ]$/.test(name)) throw new Error("'file' uses a reserved Windows name or ends with a dot or space.");
  const abs = path.resolve(DESIGNS, file);
  if (path.dirname(abs) !== DESIGNS) throw new Error("Path outside the designs folder.");
  return abs;
}

function load(name) {
  const abs = designPath(name);
  if (!fs.existsSync(abs)) throw new Error(`Design ${path.basename(abs)} does not exist. Use list_designs.`);
  let doc;
  try { doc = JSON.parse(fs.readFileSync(abs, "utf8")); } catch (err) { throw new Error(`${path.basename(abs)} is not valid JSON (${err.message}).`); }
  if (!doc || typeof doc !== "object" || !Array.isArray(doc.elements)) throw new Error(`${path.basename(abs)} does not look like a Lienzo UI design (the 'elements' list is missing).`);
  return { abs, doc };
}

/** Valid ranges for numeric fields: [min, max, exclusiveMin]. */
const RANGES = { w: [0, 100000, true], h: [0, 100000, true], font_size: [0, 1000, true], opacity: [0, 1], stroke_w: [0, 1000], radius: [0, 100000], x: [-1e6, 1e6], y: [-1e6, 1e6] };
function validateRange(k, v) {
  const r = RANGES[k];
  if (r && (v < r[0] || v > r[1] || (r[2] && v === r[0]))) throw new Error(`${k} out of range (${r[2] ? ">" : "≥"} ${r[0]} and ≤ ${r[1]}).`);
}

function save(abs, doc) {
  fs.writeFileSync(abs, JSON.stringify(doc, null, 2), "utf8");
}

function color(v) {
  if (Array.isArray(v) && v.length >= 3 && v.every((n) => Number.isInteger(n) && n >= 0 && n <= 255)) return v.length === 3 ? [...v, 255] : v.slice(0, 4);
  const m = typeof v === "string" && v.match(/^#?([0-9a-f]{6})([0-9a-f]{2})?$/i);
  if (!m) throw new Error(`Invalid color: ${JSON.stringify(v)} (use "#RRGGBB", "#RRGGBBAA" or [r,g,b,a])`);
  const n = parseInt(m[1], 16);
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255, m[2] ? parseInt(m[2], 16) : 255];
}

/** Valid image path: absolute, PNG or JPG and existing (or "" to remove it). */
function imagePath(v) {
  if (typeof v !== "string") throw new Error("image must be the absolute path of a PNG or JPG, or \"\" to remove it");
  if (v && (!path.isAbsolute(v) || !/\.(png|jpe?g)$/i.test(v) || !fs.existsSync(v))) throw new Error(`image: does not exist or is not a PNG/JPG: ${v}`);
  return v;
}

/** Size limits: a design with huge texts or lists would leave the app unresponsive for minutes. */
const LIMITS = { name: 120, text: 5000, list: 200, item: 200 };
function limit(field, v, max) {
  if (typeof v === "string" && v.length > max) throw new Error(`${field} allows at most ${max} characters.`);
  return v;
}

function find(doc, ref) {
  // Exact name first; only if there is none is it interpreted as an id (an element named «3» must not lose to id 3).
  const e = doc.elements.find((x) => x.name === ref) ?? doc.elements.find((x) => String(x.id) === String(ref));
  if (!e) throw new Error(`There is no element «${ref}». Use read_design to see the names.`);
  return e;
}

function descendants(doc, id) {
  const ids = new Set([id]);
  let changed = true;
  while (changed) {
    changed = false;
    for (const e of doc.elements) if (e.parent != null && ids.has(e.parent) && !ids.has(e.id)) { ids.add(e.id); changed = true; }
  }
  return ids;
}

// ---------- tools ----------

const TOOLS = [
  {
    name: "lienzo_info",
    description: "What Lienzo UI is, where the designs are and the recommended workflow. Call it first.",
    inputSchema: { type: "object", properties: {} },
    run: () => [
      `Lienzo UI v${VERSION} (MCP server). Visual editor for game interfaces that exports to Unity.`,
      `Designs folder: ${DESIGNS}`,
      "Workflow: list_templates → create_design → read_design → modify/duplicate/delete elements → review_design → screenshot → export_unity.",
      "Coordinates are in pixels of the reference resolution (normally 1920×1080), origin at the top left.",
      "Element types: Panel, Ellipse, Image, Text, Button, Toggle, Slider, InputField, Dropdown, ScrollView, Empty.",
      "In Unity: the design appears as a prefab in Assets/EditorGUI/Prefabs (uGUI + TextMeshPro), plus UXML/USS (UI Toolkit) and an OnGUI script (IMGUI).",
      "Editable fields: " + Object.keys(EDITABLE_FIELDS).join(", "),
    ].join("\n"),
  },
  {
    name: "list_templates",
    description: "Lists the included templates (number and name).",
    inputSchema: { type: "object", properties: {} },
    run: async () => (await runApp(["--templates"])).out,
  },
  {
    name: "create_design",
    description: "Creates a new design from a template (use 5 for \"Blank\") and saves it in the designs folder.",
    inputSchema: {
      type: "object",
      properties: { file: { type: "string" }, template: { type: "integer", minimum: 0 }, title: { type: "string" } },
      required: ["file", "template"],
    },
    run: async ({ file, template, title }) => {
      const abs = designPath(file);
      if (fs.existsSync(abs)) throw new Error(`${path.basename(abs)} already exists.`);
      if (!Number.isInteger(template) || template < 0) throw new Error("'template' must be an integer ≥ 0.");
      fs.mkdirSync(DESIGNS, { recursive: true });
      const r = await runApp(["--template", String(template), "--save", abs]);
      if (r.code !== 0) throw new Error(r.err || "Could not create the design.");
      if (title) {
        if (typeof title !== "string" || title.length > 80) throw new Error("'title' must be text of up to 80 characters.");
        const doc = JSON.parse(fs.readFileSync(abs, "utf8"));
        doc.title = title;
        save(abs, doc);
      }
      return `Created ${path.basename(abs)}.`;
    },
  },
  {
    name: "adjust_design",
    description: "Canvas settings: title, pixel_art (images without smoothing, whole pixels; in Unity Canvas Pixel Perfect and Point sprites), background (\"#RRGGBB\") and match (0 = width, 1 = height).",
    inputSchema: {
      type: "object",
      properties: { file: { type: "string" }, title: { type: "string" }, pixel_art: { type: "boolean" }, background: { type: "string" }, match: { type: "number" } },
      required: ["file"],
    },
    run: ({ file, title, pixel_art, background, match }) => {
      const { abs, doc } = load(file);
      const done = [];
      if (title !== undefined) { if (typeof title !== "string" || !title.trim() || title.length > 80) throw new Error("title: text of 1 to 80 characters"); doc.title = title; done.push("title"); }
      if (pixel_art !== undefined) { if (typeof pixel_art !== "boolean") throw new Error("pixel_art must be true or false"); doc.pixel_art = pixel_art; done.push("pixel_art"); }
      if (background !== undefined) { doc.background = color(background); done.push("background"); }
      if (match !== undefined) { if (typeof match !== "number" || match < 0 || match > 1) throw new Error("match between 0 and 1"); doc.canvas_match = match; done.push("match"); }
      if (pixel_art) for (const e of doc.elements) { e.x = Math.round(e.x); e.y = Math.round(e.y); e.w = Math.max(1, Math.round(e.w)); e.h = Math.max(1, Math.round(e.h)); }
      save(abs, doc);
      return done.length ? `Adjusted: ${done.join(", ")}.` : "No changes.";
    },
  },
  {
    name: "review_design",
    description: "Reviews a design before exporting it: text too small, low contrast (WCAG), controls too small to tap, elements outside the page, repeated names and missing images. Returns one line per problem ([ERROR] or [WARNING]).",
    inputSchema: { type: "object", properties: { file: { type: "string" } }, required: ["file"] },
    run: async ({ file }) => {
      const { abs } = load(file);
      const r = await runApp(["--open", abs, "--review"]);
      if (r.code === 2 || (!r.out && r.err)) throw new Error(r.err || "Could not review the design.");
      return r.out.trim();
    },
  },
  {
    name: "list_designs",
    description: "Lists the designs saved in the designs folder.",
    inputSchema: { type: "object", properties: {} },
    run: () => (fs.existsSync(DESIGNS) ? fs.readdirSync(DESIGNS).filter((f) => f.endsWith(EXT)).join("\n") : "") || "(no designs)",
  },
  {
    name: "read_design",
    description: "Summary of a design: title, resolution and the list of elements (id, name, type, position, size, text, parent).",
    inputSchema: { type: "object", properties: { file: { type: "string" } }, required: ["file"] },
    run: ({ file }) => {
      const { doc } = load(file);
      const rows = doc.elements.map((e) =>
        `${e.id}\t${e.name}\t${e.kind}\t(${Math.round(e.x)}, ${Math.round(e.y)})\t${Math.round(e.w)}×${Math.round(e.h)}\t${e.text ? JSON.stringify(e.text) : ""}\t${e.parent ?? ""}${e.visible === false ? "\thidden" : ""}`);
      return [`«${doc.title}» · ${doc.width}×${doc.height} · ${doc.elements.length} elements`, "id\tname\ttype\tposition\tsize\ttext\tparent", ...rows].join("\n");
    },
  },
  {
    name: "modify_element",
    description: "Changes properties of an element (by name or id). 'changes' accepts: " + Object.keys(EDITABLE_FIELDS).join(", ") + ". Colors as \"#RRGGBB\".",
    inputSchema: {
      type: "object",
      properties: { file: { type: "string" }, element: { type: "string" }, changes: { type: "object" } },
      required: ["file", "element", "changes"],
    },
    run: ({ file, element, changes }) => {
      const { abs, doc } = load(file);
      const e = find(doc, element);
      if (!changes || typeof changes !== "object" || Array.isArray(changes)) throw new Error("'changes' must be an object.");
      const done = [];
      for (const [k, v] of Object.entries(changes)) {
        const kind = Object.hasOwn(EDITABLE_FIELDS, k) ? EDITABLE_FIELDS[k] : null;
        if (!kind) throw new Error(`Field not editable: ${k}`);
        if (kind === "color") e[k] = color(v);
        else if (kind === "image") {
          if (e.kind !== "Image") throw new Error(`«${e.name}» is ${e.kind}: only Image elements show an image. Use add_element with type Image (for example inside «${e.name}»).`);
          e[k] = imagePath(v);
        }
        else if (kind === "strings") { if (!Array.isArray(v) || !v.every((s) => typeof s === "string")) throw new Error(`${k} must be a list of texts`);
          if (v.length > LIMITS.list || v.some((s) => s.length > LIMITS.item)) throw new Error(`${k} allows at most ${LIMITS.list} items of ${LIMITS.item} characters.`); e[k] = v; }
        else if (typeof v !== kind || (kind === "number" && !Number.isFinite(v))) throw new Error(`${k} must be ${kind}`);
        else {
          if (kind === "string") limit(k, v, k === "name" ? LIMITS.name : LIMITS.text);
          if (kind === "number") validateRange(k, v);
          if (k === "name") {
            if (!v.trim()) throw new Error("name cannot be empty.");
            if (doc.elements.some((x) => x !== e && x.name === v)) throw new Error(`An element «${v}» already exists.`);
          }
          e[k] = v;
        }
        done.push(k);
      }
      save(abs, doc);
      return `«${e.name}» updated: ${done.join(", ")}. (Anchors do not change; if you move a child, its coordinates are still absolute.)`;
    },
  },
  {
    name: "add_element",
    description: "Adds a new element, optionally inside another one (parent). Types: " + TYPES.join(", ") + ". For icons or pixel art use type Image with 'image' (absolute path to a PNG/JPG). Position and size in canvas pixels.",
    inputSchema: {
      type: "object",
      properties: {
        file: { type: "string" }, type: { type: "string", enum: TYPES }, name: { type: "string" }, parent: { type: "string" },
        x: { type: "number" }, y: { type: "number" }, w: { type: "number" }, h: { type: "number" },
        text: { type: "string" }, image: { type: "string" }, color: { type: "string" },
      },
      required: ["file", "type", "name", "x", "y", "w", "h"],
    },
    run: ({ file, type, name, parent, x, y, w, h, text, image, color: c }) => {
      const { abs, doc } = load(file);
      if (!TYPES.includes(type)) throw new Error(`invalid type. Use: ${TYPES.join(", ")}`);
      if (typeof name !== "string" || !name.trim()) throw new Error("'name' is missing.");
      limit("name", name, LIMITS.name);
      limit("text", text, LIMITS.text);
      if (doc.elements.some((e) => e.name === name)) throw new Error(`An element «${name}» already exists.`);
      for (const [k, v] of Object.entries({ x, y, w, h })) if (typeof v !== "number" || !Number.isFinite(v)) throw new Error(`${k} must be a number`);
      if (w <= 0 || h <= 0) throw new Error("w and h must be greater than 0");
      if (image !== undefined && type !== "Image") throw new Error("'image' is only used with type Image.");
      const p = parent !== undefined ? find(doc, parent) : null;
      // Base: an element of the same type (same defaults) or, if none, any existing one.
      const base = doc.elements.find((e) => e.kind === type) || doc.elements[0];
      if (!base) throw new Error("The design is empty: create the design from a template that has elements.");
      const e = structuredClone(base);
      Object.assign(e, {
        id: doc.next_id++, name, kind: type, x, y, w, h, rot: 0, parent: p ? p.id : null, group: null,
        visible: true, locked: false, collapsed: false, text: text ?? (type === "Text" || type === "Button" ? name : ""),
        image: image !== undefined ? imagePath(image) : "", anchor_min: [0.5, 0.5], anchor_max: [0.5, 0.5], pivot: [0.5, 0.5],
        on_click: "", opacity: 1,
      });
      e.layout = { ...e.layout, mode: "None" };
      if (e.shadow) e.shadow.on = false;
      if (type === "Image") { e.fill = [255, 255, 255, 255]; e.fill2 = [255, 255, 255, 255]; e.stroke_w = 0; e.radius = 0; e.raycast = false; }
      if (c !== undefined) e.fill = color(c);
      if (doc.pixel_art) { e.x = Math.round(e.x); e.y = Math.round(e.y); e.w = Math.round(e.w); e.h = Math.round(e.h); }
      // Drawn on top of the parent and its current children.
      const ids = p ? descendants(doc, p.id) : null;
      const last = ids ? Math.max(...doc.elements.map((x, i) => (ids.has(x.id) ? i : -1))) : doc.elements.length - 1;
      doc.elements.splice(last + 1, 0, e);
      save(abs, doc);
      return `Added «${name}» (${type})${p ? ` inside «${p.name}»` : ""}.`;
    },
  },
  {
    name: "duplicate_element",
    description: "Duplicates an element (with its children) under another name and moves it dx, dy pixels. Useful to add buttons, checkboxes, etc. of the same style.",
    inputSchema: {
      type: "object",
      properties: { file: { type: "string" }, element: { type: "string" }, new_name: { type: "string" }, dx: { type: "number" }, dy: { type: "number" }, text: { type: "string" } },
      required: ["file", "element", "new_name"],
    },
    run: ({ file, element, new_name, dx = 0, dy = 0, text }) => {
      const { abs, doc } = load(file);
      const base = find(doc, element);
      if (typeof new_name !== "string" || !new_name.trim()) throw new Error("'new_name' is missing.");
      limit("new_name", new_name, LIMITS.name);
      limit("text", text, LIMITS.text);
      if (doc.elements.some((x) => x.name === new_name)) throw new Error(`An element «${new_name}» already exists.`);
      const ids = descendants(doc, base.id);
      const idMap = new Map();
      for (const id of ids) idMap.set(id, doc.next_id++);
      const copies = doc.elements.filter((x) => ids.has(x.id)).map((x) => {
        const c = structuredClone(x);
        c.id = idMap.get(x.id);
        if (c.parent != null && idMap.has(c.parent)) c.parent = idMap.get(c.parent);
        if (c.group != null && idMap.has(c.group)) c.group = idMap.get(c.group);
        c.x += dx; c.y += dy;
        return c;
      });
      copies[0].name = new_name;
      if (text !== undefined) copies[0].text = String(text);
      const pos = doc.elements.findIndex((x) => x.id === base.id);
      const last = Math.max(...doc.elements.map((x, i) => (ids.has(x.id) ? i : -1)));
      doc.elements.splice(Math.max(pos, last) + 1, 0, ...copies);
      save(abs, doc);
      return `Created «${new_name}» (${copies.length} element(s)) from «${base.name}».`;
    },
  },
  {
    name: "delete_element",
    description: "Deletes an element and its children.",
    inputSchema: { type: "object", properties: { file: { type: "string" }, element: { type: "string" } }, required: ["file", "element"] },
    run: ({ file, element }) => {
      const { abs, doc } = load(file);
      const e = find(doc, element);
      const ids = descendants(doc, e.id);
      doc.elements = doc.elements.filter((x) => !ids.has(x.id));
      save(abs, doc);
      return `Deleted «${e.name}» (${ids.size} element(s)).`;
    },
  },
  {
    name: "screenshot",
    description: "Opens the design in Lienzo UI and saves a PNG screenshot of the editor in the designs folder (screenshots/<file>.png).",
    inputSchema: { type: "object", properties: { file: { type: "string" } }, required: ["file"] },
    run: async ({ file }) => {
      const { abs } = load(file);
      const out = path.join(DESIGNS, "screenshots", path.basename(abs, EXT) + ".png");
      fs.mkdirSync(path.dirname(out), { recursive: true });
      const r = await runApp(["--open", abs, "--no-tutorial", "--screenshot", out], 90_000);
      if (r.code !== 0 || !fs.existsSync(out)) throw new Error("Could not take the screenshot. " + r.err);
      return `Screenshot saved to ${out}`;
    },
  },
  {
    name: "export_unity",
    description: "Exports the design to a Unity project (the folder that contains Assets). Writes to Assets/EditorGUI: design, importer, UXML/USS, IMGUI script and views. Unity creates the prefab when you return to its window.",
    inputSchema: { type: "object", properties: { file: { type: "string" }, unity_project: { type: "string" } }, required: ["file", "unity_project"] },
    run: async ({ file, unity_project }) => {
      const { abs } = load(file);
      if (typeof unity_project !== "string" || !fs.existsSync(path.join(unity_project, "Assets")))
        throw new Error("'unity_project' must be the folder of a Unity project (with an Assets folder).");
      const r = await runApp(["--open", abs, "--export", unity_project]);
      if (r.code !== 0) throw new Error(r.err || "Could not export.");
      return "Files written:\n" + r.out;
    },
  },
];

// ---------- MCP protocol (JSON-RPC 2.0 over stdio) ----------

let queue = Promise.resolve();
const send = (m) => { if (!process.stdout.destroyed) process.stdout.write(JSON.stringify(m) + "\n"); };
process.stdout.on("error", () => process.exit(0)); // the client closed the connection
const respond = (id, result) => send({ jsonrpc: "2.0", id, result });
const fail = (id, code, message) => send({ jsonrpc: "2.0", id, error: { code, message } });

/** Finds a tool by its English name or by its legacy Spanish alias (hidden: not in tools/list). */
function findTool(name) {
  const canonical = Object.hasOwn(LEGACY_TOOLS, name) ? LEGACY_TOOLS[name] : name;
  return TOOLS.find((x) => x.name === canonical);
}

async function handle(m) {
  const { id, method, params } = m;
  const isNotification = id === undefined || id === null;
  try {
    switch (method) {
      case "initialize":
        return respond(id, {
          protocolVersion: params?.protocolVersion || "2025-06-18",
          capabilities: { tools: { listChanged: false } },
          serverInfo: { name: "lienzo-ui", version: VERSION },
          instructions: "Lienzo UI server (interface editor for Unity). Start with lienzo_info.",
        });
      case "notifications/initialized":
      case "notifications/cancelled":
        return;
      case "ping":
        return respond(id, {});
      case "tools/list":
        return respond(id, { tools: TOOLS.map(({ name, description, inputSchema }) => ({ name, description, inputSchema })) });
      case "tools/call": {
        const tool = findTool(params?.name);
        if (!tool) return fail(id, -32602, `Unknown tool: ${params?.name}`);
        // Calls run one at a time: two simultaneous commands on the same design do not overwrite each other.
        const turn = queue.then(async () => {
          try {
            respond(id, { content: [{ type: "text", text: String(await tool.run(normalizeArgs(params?.arguments))) }] });
          } catch (e) {
            respond(id, { content: [{ type: "text", text: `Error: ${e.message}` }], isError: true });
          }
        });
        queue = turn;
        return turn;
      }
      default:
        if (!isNotification) fail(id, -32601, `Method not supported: ${method}`);
    }
  } catch (e) {
    if (!isNotification) fail(id, -32603, String(e?.message || e));
  }
}

readline.createInterface({ input: process.stdin }).on("line", (l) => {
  if (!l.trim()) return;
  let m;
  try { m = JSON.parse(l); } catch { return fail(null, -32700, "Invalid JSON"); }
  if (Array.isArray(m)) m.forEach(handle); else handle(m);
});
