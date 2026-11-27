#!/usr/bin/env node
// Tests for the Lienzo UI MCP server: protocol, full flow, invalid inputs and the legacy Spanish aliases.
// Usage: node mcp/probar-mcp.mjs   (needs lienzo-ui built; exits with 1 if anything fails)
// Creates and deletes a temporary design in a temporary designs folder.
import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import readline from "node:readline";
import { fileURLToPath } from "node:url";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const DESIGNS = fs.mkdtempSync(path.join(os.tmpdir(), "lienzo-mcp-"));
const SERVER = path.join(ROOT, "mcp", "lienzo-ui-mcp.mjs");

/** Starts a server with the given extra environment and returns helpers to talk to it. */
function start(env) {
  const srv = spawn(process.execPath, [SERVER], { stdio: ["pipe", "pipe", "inherit"], env: { ...process.env, LIENZO_DESIGNS: "", LIENZO_DISENOS: "", ...env } });
  const pending = new Map();
  readline.createInterface({ input: srv.stdout }).on("line", (l) => { const m = JSON.parse(l); pending.get(m.id)?.(m); pending.delete(m.id); });
  let n = 1;
  const request = (method, params) => new Promise((done) => { const id = n++; pending.set(id, done); srv.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n"); });
  return { srv, request, call: (name, args) => request("tools/call", { name, arguments: args }) };
}

/** Raw access to a server: sends lines as they are and collects every message it prints (for protocol tests). */
function startRaw(env) {
  const srv = spawn(process.execPath, [SERVER], { stdio: ["pipe", "pipe", "inherit"], env: { ...process.env, LIENZO_DESIGNS: "", LIENZO_DISENOS: "", ...env } });
  const all = [];
  const waiters = [];
  readline.createInterface({ input: srv.stdout }).on("line", (l) => {
    let m;
    try { m = JSON.parse(l); } catch { m = { unparsable: l }; }
    all.push(m);
    for (const w of [...waiters]) if (w.test(m)) { waiters.splice(waiters.indexOf(w), 1); w.done(); }
  });
  return {
    srv,
    send: (line) => srv.stdin.write(line + "\n"),
    /** Resolves with every message received since the last call, once one matches `test` (or after 5 s, with what there is). */
    until: (test) => new Promise((resolve) => {
      const finish = () => { clearTimeout(t); resolve(all.splice(0, all.length)); };
      const w = { test, done: finish };
      const t = setTimeout(() => { waiters.splice(waiters.indexOf(w), 1); finish(); }, 5000);
      if (all.some(test)) finish(); else waiters.push(w);
    }),
  };
}

const { srv, request, call } = start({ LIENZO_DESIGNS: DESIGNS });
const text = (r) => r.result?.content?.[0]?.text ?? "";
const isError = (r) => r.result?.isError === true || !!r.error;
let failures = 0, total = 0;
const ok = (name, cond, detail = "") => { total++; if (cond) console.log(`  ok  ${name}`); else { failures++; console.log(`  ✗   ${name} ${detail}`); } };

const init = await request("initialize", { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "p", version: "0" } });
ok("initialize", init.result?.serverInfo?.name === "lienzo-ui");
ok("initialize instructions warn that export_unity writes directly and say to dry-run first", /writes straight/.test(init.result?.instructions ?? "") && /dry_run: true/.test(init.result?.instructions ?? ""));
const listed = (await request("tools/list", {})).result?.tools ?? [];
ok("tools/list: 13 tools", listed.length === 13);
ok("tools/list: English names only", listed.every((t) => /^[a-z_]+$/.test(t.name)) && ["lienzo_info", "list_templates", "create_design", "adjust_design", "review_design", "list_designs", "read_design", "modify_element", "add_element", "duplicate_element", "delete_element", "screenshot", "export_unity"].every((n) => listed.some((t) => t.name === n)));
ok("export_unity description warns about the direct write and the dry run", /no confirmation in the editor/.test(listed.find((t) => t.name === "export_unity")?.description ?? "") && /dry_run: true first/.test(listed.find((t) => t.name === "export_unity")?.description ?? ""));
ok("lienzo_info repeats the export safety rule", /dry_run: true first/.test(text(await call("lienzo_info", {}))));
ok("tools/list: legacy names are hidden", !listed.some((t) => /diseno|plantillas|captura|elemento|lienzo$/.test(t.name) && t.name !== "lienzo_info"));
ok("tools/list: descriptions are English", listed.every((t) => !/[áéíóú¿¡]|\[AVISO\]/.test(t.description)));
ok("list_templates includes Inventory", text(await call("list_templates", {})).includes("Inventory"));

console.log("Full flow:");
ok("list_templates includes the newer ones", text(await call("list_templates", {})).includes("Character select"));
ok("create_design from Pause menu", !isError(await call("create_design", { file: "Pause", template: 6 })));
ok("review_design: no errors in the template", !text(await call("review_design", { file: "Pause" })).includes("[ERROR]"));
ok("create_design from Inventory", !isError(await call("create_design", { file: "Shop", template: 4, title: "Blacksmith shop" })));
const summary = text(await call("read_design", { file: "Shop" }));
ok("read_design shows the title", summary.includes("Blacksmith shop"), summary.slice(0, 80));
ok("read_design header is English", summary.includes("id\tname\ttype\tposition\tsize\ttext\tparent"));
ok("modify_element changes text and color", !isError(await call("modify_element", { file: "Shop", element: "Title", changes: { text: "Smithy", text_color: "#FFD27A" } })));
ok("duplicate_element creates a new button", !isError(await call("duplicate_element", { file: "Shop", element: "Close", new_name: "Buy", dx: -80, dy: 0 })));
ok("the design has «Buy»", text(await call("read_design", { file: "Shop" })).includes("Buy"));
ok("delete_element removes «Slot 12»", !isError(await call("delete_element", { file: "Shop", element: "Slot 12" })));
ok("list_designs shows it", text(await call("list_designs", {})).includes("Shop.editorgui.json"));
ok("adjust_design turns on pixel art", !isError(await call("adjust_design", { file: "Shop", pixel_art: true })));
ok("pixel art rounds to whole pixels", JSON.parse(fs.readFileSync(path.join(DESIGNS, "Shop.editorgui.json"), "utf8")).elements.every((e) => Number.isInteger(e.x) && Number.isInteger(e.w)));
ok("adjust_design rejects non-boolean pixel_art", isError(await call("adjust_design", { file: "Shop", pixel_art: "yes" })));
ok("image rejects files that are not PNG/JPG", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: { image: process.execPath } })));
ok("image rejects relative paths", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: { image: "a.png" } })));
{
  // 5 simultaneous changes on the same design: with the queue none is lost
  const names = ["A1", "A2", "A3", "A4", "A5"];
  await Promise.all(names.map((n, i) => call("duplicate_element", { file: "Shop", element: "Close", new_name: n, dx: -90 * (i + 1) })));
  const t = text(await call("read_design", { file: "Shop" }));
  ok("simultaneous commands do not overwrite each other", names.every((n) => t.includes(n)));
}

console.log("Invalid inputs:");
for (const file of ["../outside", "..\\outside", "C:/Windows/win", "/etc/passwd", "a/b", ""])
  ok(`file rejected «${file}»`, isError(await call("read_design", { file })));
ok("field not editable (id)", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: { id: 1 } })));
ok("__proto__ field", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: JSON.parse('{"__proto__": {"x": 1}}') })));
ok("wrong type (x as text)", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: { x: "10" } })));
ok("invalid color", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: { fill: "red" } })));
ok("w ≤ 0 rejected", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: { w: -50 } })));
ok("opacity > 1 rejected", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: { opacity: 5 } })));
ok("font_size ≤ 0 rejected", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: { font_size: 0 } })));
ok("empty name rejected", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: { name: "  " } })));
ok("duplicate name rejected", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: { name: "Close" } })));
ok("reserved file name «CON»", isError(await call("create_design", { file: "CON", template: 0 })));
fs.writeFileSync(path.join(DESIGNS, "Empty.editorgui.json"), "{}");
{ const r = await call("read_design", { file: "Empty" }); ok("design {} gives a clear error", isError(r) && !text(r).includes("undefined"), text(r)); }
ok("huge text rejected", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: { text: "t".repeat(6000) } })));
ok("huge name rejected", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: { name: "n".repeat(500) } })));
ok("huge list rejected", isError(await call("modify_element", { file: "Shop", element: "Buy", changes: { options: Array(1000).fill("a") } })));
{
  // an element named like another's id must be found by name
  await call("add_element", { file: "Shop", type: "Button", name: "3", x: 0, y: 0, w: 50, h: 20 });
  await call("modify_element", { file: "Shop", element: "3", changes: { text: "BY-NAME" } });
  const doc = JSON.parse(fs.readFileSync(path.join(DESIGNS, "Shop.editorgui.json"), "utf8"));
  ok("the name wins over the numeric id", doc.elements.find((e) => e.name === "3")?.text === "BY-NAME" && doc.elements.find((e) => e.id === 3)?.text !== "BY-NAME");
}
ok("element does not exist", isError(await call("modify_element", { file: "Shop", element: "Nothing", changes: { text: "x" } })));
ok("duplicate name", isError(await call("duplicate_element", { file: "Shop", element: "Close", new_name: "Buy" })));
ok("create over an existing one", isError(await call("create_design", { file: "Shop", template: 0 })));
ok("template does not exist", isError(await call("create_design", { file: "Other", template: 999 })));
ok("export to a folder without Assets", isError(await call("export_unity", { file: "Shop", unity_project: os.tmpdir() })));
ok("unknown tool", !!(await call("no_such_tool", {})).error);

console.log("Legacy Spanish aliases:");
ok("alias info_lienzo", text(await call("info_lienzo", {})).includes("Designs folder"));
ok("alias listar_plantillas", text(await call("listar_plantillas", {})).includes("Inventory"));
ok("alias crear_diseno with Spanish parameters", !isError(await call("crear_diseno", { archivo: "Viejo", plantilla: 4, titulo: "Tienda vieja" })));
ok("alias listar_disenos", text(await call("listar_disenos", {})).includes("Viejo.editorgui.json"));
ok("alias leer_diseno with 'archivo'", text(await call("leer_diseno", { archivo: "Viejo" })).includes("Tienda vieja"));
ok("alias modificar_elemento (archivo, elemento, cambios)", !isError(await call("modificar_elemento", { archivo: "Viejo", elemento: "Title", cambios: { text: "Hola" } })));
ok("alias agregar_elemento (tipo, nombre, padre, texto)", !isError(await call("agregar_elemento", { archivo: "Viejo", tipo: "Text", nombre: "Nota", padre: "Inventory", x: 10, y: 10, w: 100, h: 30, texto: "Nota" })));
ok("alias duplicar_elemento (nuevo_nombre)", !isError(await call("duplicar_elemento", { archivo: "Viejo", elemento: "Close", nuevo_nombre: "Cerrar 2", dx: -40 })));
ok("alias ajustar_diseno (fondo, titulo)", !isError(await call("ajustar_diseno", { archivo: "Viejo", fondo: "#102030", titulo: "Vieja" })));
ok("alias revisar_diseno", !isError(await call("revisar_diseno", { archivo: "Viejo" })));
ok("alias eliminar_elemento", !isError(await call("eliminar_elemento", { archivo: "Viejo", elemento: "Cerrar 2" })));
{
  const doc = JSON.parse(fs.readFileSync(path.join(DESIGNS, "Viejo.editorgui.json"), "utf8"));
  ok("aliased edits were applied", doc.title === "Vieja" && doc.elements.find((e) => e.name === "Title")?.text === "Hola" && doc.elements.some((e) => e.name === "Nota") && !doc.elements.some((e) => e.name === "Cerrar 2"));
}
ok("alias exportar_unity (proyecto_unity) still validates", isError(await call("exportar_unity", { archivo: "Viejo", proyecto_unity: os.tmpdir() })));
ok("English parameter wins over the Spanish one", text(await call("read_design", { file: "Viejo", archivo: "Nope" })).includes("Vieja"));
{
  // LIENZO_DISENOS (old variable) and an old «disenos» folder are still honored
  const old = fs.mkdtempSync(path.join(os.tmpdir(), "lienzo-mcp-old-"));
  fs.copyFileSync(path.join(DESIGNS, "Viejo.editorgui.json"), path.join(old, "FromOld.editorgui.json"));
  const legacy = start({ LIENZO_DISENOS: old });
  ok("LIENZO_DISENOS is used as a fallback", text(await legacy.call("list_designs", {})).includes("FromOld.editorgui.json"));
  legacy.srv.kill();
  const both = start({ LIENZO_DESIGNS: DESIGNS, LIENZO_DISENOS: old });
  ok("LIENZO_DESIGNS wins over LIENZO_DISENOS", !text(await both.call("list_designs", {})).includes("FromOld"));
  both.srv.kill();
  fs.rmSync(old, { recursive: true, force: true });
}
{
  // without any variable the default folder is <repo>/designs (or an old <repo>/disenos if only that exists)
  const none = start({});
  const info = text(await none.call("lienzo_info", {}));
  const expected = fs.existsSync(path.join(ROOT, "designs")) || !fs.existsSync(path.join(ROOT, "disenos")) ? "designs" : "disenos";
  ok(`default designs folder is <repo>/${expected}`, info.includes(path.join(ROOT, expected)), info.split("\n")[1]);
  none.srv.kill();
}
console.log("Protocol robustness (never crash, proper JSON-RPC errors):");
{
  const raw = startRaw({ LIENZO_DESIGNS: DESIGNS });
  let seq = 1000;
  /** Sends a raw line, then a ping; returns every message received up to the ping's answer (so we know the server is alive). */
  const exchange = async (payload) => {
    const pingId = seq++;
    raw.send(payload);
    raw.send(JSON.stringify({ jsonrpc: "2.0", id: pingId, method: "ping" }));
    const got = await raw.until((m) => m.id === pingId);
    return got;
  };
  const errors = (msgs) => msgs.filter((m) => m.error);
  const wellFormed = (m) => m.jsonrpc === "2.0" && "id" in m && typeof m.error?.code === "number" && typeof m.error?.message === "string";
  for (const [label, payload] of [
    ["null", "null"], ["[null]", "[null]"], ["a number", "42"], ["a string", '"hi"'], ["true", "true"], ["an empty batch", "[]"],
    ["method that is not text", '{"jsonrpc":"2.0","id":1,"method":5}'], ["an object as id", '{"jsonrpc":"2.0","id":{},"method":"ping"}'],
    ["params that is a string", '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":"x"}'],
  ]) {
    const got = await exchange(payload);
    ok(`${label}: JSON-RPC error and the server stays alive`, got.length >= 2 && errors(got).length >= 1 && errors(got).every(wellFormed), JSON.stringify(got).slice(0, 200));
  }
  {
    const got = await exchange('[null,{"jsonrpc":"2.0","id":7,"method":"ping"},[1],42]');
    ok("a batch with null and non-objects: errors for the bad ones, the good one is answered", errors(got).length >= 3 && got.some((m) => m.id === 7 && m.result) && errors(got).every(wellFormed), JSON.stringify(got).slice(0, 300));
  }
  {
    const got = await exchange('{"jsonrpc":"2.0","method":"tools/call","params":{"name":"list_designs"}}');
    ok("tools/call without an id is refused (an error, never a result without id)", errors(got).length === 1 && !got.some((m) => m.result && m.id === undefined), JSON.stringify(got).slice(0, 200));
  }
  for (const [label, params] of [["a name that is not text", { name: 5 }], ["no name", {}], ["arguments as text", { name: "list_designs", arguments: "abc" }], ["arguments as a list", { name: "read_design", arguments: [1] }]]) {
    raw.send(JSON.stringify({ jsonrpc: "2.0", id: 50, method: "tools/call", params }));
    const got = await raw.until((m) => m.id === 50);
    const r = got.find((m) => m.id === 50);
    ok(`tools/call with ${label}: error, no crash`, !!r && (!!r.error || r.result?.isError === true), JSON.stringify(r));
  }
  const init = (v) => exchange(JSON.stringify({ jsonrpc: "2.0", id: 60, method: "initialize", params: { protocolVersion: v } })).then((g) => g.find((m) => m.id === 60)?.result);
  const cargo = fs.readFileSync(path.join(ROOT, "Cargo.toml"), "utf8").match(/^version\s*=\s*"([^"]+)"/m)[1];
  const r1 = await init("2024-11-05");
  ok("protocolVersion: a supported client version is echoed", r1?.protocolVersion === "2024-11-05");
  ok("protocolVersion: an unknown one gets the newest supported", (await init("1999-01-01"))?.protocolVersion === "2025-06-18");
  ok("protocolVersion: a non-text one gets the newest supported", (await init(5))?.protocolVersion === "2025-06-18");
  ok(`serverInfo.version comes from Cargo.toml (${cargo})`, r1?.serverInfo?.version === cargo, r1?.serverInfo?.version);
  raw.srv.kill();
}

console.log("add_element / duplicate_element validate like modify_element:");
{
  const base = { file: "Shop", type: "Button", name: "V1", x: 10, y: 10, w: 50, h: 20 };
  const before = text(await call("read_design", { file: "Shop" }));
  for (const [label, patch] of [
    ["x as text", { x: "10" }], ["y as null", { y: null }], ["w = 0", { w: 0 }], ["w negative", { w: -5 }], ["h too large", { h: 1e9 }], ["x out of range", { x: 1e9 }],
    ["text as number", { text: 5 }], ["text as object", { text: { a: 1 } }], ["parent as object", { parent: {} }], ["parent as boolean", { parent: true }],
    ["type as number", { type: 5 }], ["type unknown", { type: "Nope" }], ["name as number", { name: 5 }], ["color invalid", { color: "red" }], ["image on a Button", { image: process.execPath }],
    ["huge text", { text: "t".repeat(6000) }], ["UNC image", { type: "Image", image: "\\\\server\\share\\a.png" }], ["UNC image with slashes", { type: "Image", image: "//server/share/a.png" }],
  ]) ok(`add_element rejects ${label}`, isError(await call("add_element", { ...base, ...patch })));
  ok("add_element rejects a design that does not exist after validating (no crash)", isError(await call("add_element", { ...base, file: "NoSuchDesign" })));
  ok("add_element Spanish alias validates too", isError(await call("agregar_elemento", { archivo: "Shop", tipo: "Button", nombre: "V2", x: "1", y: 1, w: 10, h: 10 })));
  ok("add_element valid call still works", !isError(await call("add_element", { ...base, name: "V-ok" })));
  const dup = { file: "Shop", element: "Close", new_name: "D1" };
  for (const [label, patch] of [
    ["dx as text", { dx: "5" }], ["dy as null", { dy: null }], ["dx out of range", { dx: 1e12 }], ["dy as object", { dy: {} }], ["text as number", { text: 7 }],
    ["element as object", { element: {} }], ["element as list", { element: ["Close"] }], ["new_name as number", { new_name: 5 }], ["huge text", { text: "t".repeat(6000) }],
  ]) ok(`duplicate_element rejects ${label}`, isError(await call("duplicate_element", { ...dup, ...patch })));
  ok("duplicate_element Spanish alias validates too", isError(await call("duplicar_elemento", { archivo: "Shop", elemento: "Close", nuevo_nombre: "D2", dx: "3" })));
  ok("duplicate_element valid call still works", !isError(await call("duplicate_element", { ...dup, new_name: "D-ok", dx: 5, dy: -5, text: "Copy" })));
  const after = text(await call("read_design", { file: "Shop" }));
  ok("rejected calls did not change the design", !/\bV1\b|\bV2\b|\bD1\b|\bD2\b/.test(after) && after.includes("V-ok") && after.includes("D-ok") && before.split("\n").length < after.split("\n").length);
  ok("modify_element rejects UNC image paths", isError(await call("modify_element", { file: "Shop", element: "V-ok", changes: { image: "\\\\server\\share\\a.png" } })));
}

console.log("create_design validates before creating anything:");
for (const [label, title] of [["a title that is a number", 123], ["a title of 100 characters", "x".repeat(100)], ["a title with control characters", "a\u0007b"]]) {
  const r = await call("create_design", { file: "BadTitle", template: 0, title });
  ok(`create_design rejects ${label}, no file is left`, isError(r) && !fs.existsSync(path.join(DESIGNS, "BadTitle.editorgui.json")), text(r));
}

console.log("App failures (fake app):");
{
  const fakeDir = fs.mkdtempSync(path.join(os.tmpdir(), "lienzo-mcp-fake-"));
  const fake = path.join(fakeDir, "fake-app.mjs");
  const argsOut = path.join(fakeDir, "args.json");
  fs.writeFileSync(fake, 'import fs from "node:fs";\nfs.writeFileSync(process.env.FAKE_OUT, JSON.stringify(process.argv.slice(2)));\nif (process.env.FAKE_MODE === "hang") setInterval(() => {}, 1000); else console.log("fake ok");\n');
  const hang = start({ LIENZO_DESIGNS: DESIGNS, LIENZO_UI: fake, LIENZO_TIMEOUT_MS: "800", FAKE_MODE: "hang", FAKE_OUT: argsOut });
  const r = await hang.call("review_design", { file: "Shop" });
  ok("review_design errors (not an empty string) when the app times out", isError(r) && /did not answer/.test(text(r)), JSON.stringify(r.result ?? r.error));
  const s2 = await hang.call("screenshot", { file: "Shop" });
  ok("screenshot errors when the app times out", isError(s2));
  hang.srv.kill();
  const fine = start({ LIENZO_DESIGNS: DESIGNS, LIENZO_UI: fake, FAKE_OUT: argsOut });
  const project = fs.mkdtempSync(path.join(os.tmpdir(), "lienzo-mcp-unity-"));
  fs.mkdirSync(path.join(project, "Assets"));
  const rel = path.relative(process.cwd(), project);
  ok("export_unity accepts a relative project path", !isError(await fine.call("export_unity", { file: "Shop", unity_project: rel })), rel);
  const sent = JSON.parse(fs.readFileSync(argsOut, "utf8"));
  ok("export_unity passes the ABSOLUTE project path to the app", sent.includes(path.resolve(rel)) && path.isAbsolute(sent[sent.indexOf("--export") + 1]), JSON.stringify(sent));
  ok("export_unity dry_run answer asks the person to confirm and says how to export for real", /confirmation/.test(text(await fine.call("export_unity", { file: "Shop", unity_project: rel, dry_run: true }))) && /without dry_run/.test(text(await fine.call("export_unity", { file: "Shop", unity_project: rel, dry_run: true }))));
  ok("export_unity real export reminds to use dry_run first", /dry_run: true first/.test(text(await fine.call("export_unity", { file: "Shop", unity_project: rel }))));
  ok("export_unity rejects a non-boolean dry_run", isError(await fine.call("export_unity", { file: "Shop", unity_project: rel, dry_run: "yes" })));
  const dry = await fine.call("export_unity", { file: "Shop", unity_project: rel, dry_run: true });
  const sentDry = JSON.parse(fs.readFileSync(argsOut, "utf8"));
  ok("export_unity dry_run passes --dry-run to the app and says nothing was written", !isError(dry) && sentDry.includes("--dry-run") && /nothing was written/i.test(text(dry)), JSON.stringify(sentDry));
  await fine.call("export_unity", { file: "Shop", unity_project: rel });
  ok("export_unity without dry_run does not pass --dry-run", !JSON.parse(fs.readFileSync(argsOut, "utf8")).includes("--dry-run"));
  ok("export_unity rejects UNC project paths", isError(await fine.call("export_unity", { file: "Shop", unity_project: "\\\\server\\share\\proj" })));
  ok("export_unity rejects a folder without Assets", isError(await fine.call("export_unity", { file: "Shop", unity_project: fakeDir })));
  fine.srv.kill();
  fs.rmSync(fakeDir, { recursive: true, force: true });
  fs.rmSync(project, { recursive: true, force: true });
}

{
  // next_id is optional in FORMAT.md; unity_name & co. belong to one element and are not inherited by new ones
  const f = path.join(DESIGNS, "Shop.editorgui.json");
  const d = JSON.parse(fs.readFileSync(f, "utf8"));
  delete d.next_id;
  const base = d.elements.find((e) => e.kind === "Button");
  Object.assign(base, { unity_name: "BtnBase", sprite: "Assets/x.png", slice: [1, 1, 1, 1], ppu: 50, image_mode: "Sliced" });
  fs.writeFileSync(f, JSON.stringify(d));
  const top = Math.max(...d.elements.map((e) => e.id));
  ok("add_element works without next_id", !isError(await call("add_element", { file: "Shop", type: "Button", name: "NoNextId", x: 10, y: 10, w: 100, h: 40 })));
  const d2 = JSON.parse(fs.readFileSync(f, "utf8"));
  const added = d2.elements.find((e) => e.name === "NoNextId");
  ok("add_element gets a free id", added && added.id > top && new Set(d2.elements.map((e) => e.id)).size === d2.elements.length);
  ok("add_element does not copy unity_name, sprite, slice, ppu, image_mode", added && ["unity_name", "sprite", "slice", "ppu", "image_mode"].every((k) => !(k in added)));
  ok("duplicate_element does not copy them either", !isError(await call("duplicate_element", { file: "Shop", element: base.name, new_name: "DupNoMeta" })));
  const dup = JSON.parse(fs.readFileSync(f, "utf8")).elements.find((e) => e.name === "DupNoMeta");
  ok("the duplicate has no unity_name, sprite, slice, ppu, image_mode", dup && ["unity_name", "sprite", "slice", "ppu", "image_mode"].every((k) => !(k in dup)));
}
{
  // a huge line without a newline must not be buffered forever: one error, and the server keeps working
  const big = start({ LIENZO_DESIGNS: DESIGNS });
  const errP = new Promise((res) => big.srv.stdout.once("data", (b) => res(b.toString())));
  big.srv.stdin.write("x".repeat(9 * 1024 * 1024));
  const out = await errP;
  ok("an oversized stdin line is rejected with an error", /too large/.test(out), out.slice(0, 100));
  big.srv.stdin.write("\n");
  ok("the server still answers after an oversized line", (await big.request("ping", {})).result !== undefined);
  big.srv.kill();
}

srv.stdin.write("{not json}\n");
ok("invalid JSON does not bring the server down", (await request("ping", {})).result !== undefined);

if (process.argv[2]) {
  // node mcp/probar-mcp.mjs <Unity project>  → also exports the shop and leaves it in designs/
  const r = await call("export_unity", { file: "Shop", unity_project: process.argv[2] });
  ok("export_unity to the given project", !isError(r), text(r).slice(0, 200));
  fs.mkdirSync(path.join(ROOT, "designs"), { recursive: true });
  fs.copyFileSync(path.join(DESIGNS, "Shop.editorgui.json"), path.join(ROOT, "designs", "shop.editorgui.json"));
}

// ---- npm package layout: the script sits at the package root (no mcp/ folder), inside node_modules ----
{
  const sandbox = fs.mkdtempSync(path.join(os.tmpdir(), "lienzo-npm-"));
  const pkg = path.join(sandbox, "node_modules", "lienzo-ui-mcp");
  fs.mkdirSync(pkg, { recursive: true });
  fs.copyFileSync(SERVER, path.join(pkg, "lienzo-ui-mcp.mjs"));
  fs.writeFileSync(path.join(pkg, "package.json"), JSON.stringify({ name: "lienzo-ui-mcp", version: "9.9.9", type: "module" }));
  // a planted executable next to node_modules must never be picked up
  fs.writeFileSync(path.join(sandbox, "node_modules", "lienzo-ui.exe"), "not the app");
  const home = path.join(sandbox, "home");
  fs.mkdirSync(home);
  const npmSrv = spawn(process.execPath, [path.join(pkg, "lienzo-ui-mcp.mjs")], { stdio: ["pipe", "pipe", "inherit"], cwd: sandbox, env: { ...process.env, LIENZO_DESIGNS: "", LIENZO_DISENOS: "", LIENZO_UI: "", HOME: home, USERPROFILE: home } });
  const waiting = new Map();
  readline.createInterface({ input: npmSrv.stdout }).on("line", (l) => { const m = JSON.parse(l); waiting.get(m.id)?.(m); waiting.delete(m.id); });
  let nid = 1;
  const ask = (method, params) => new Promise((done) => { const id = nid++; waiting.set(id, done); npmSrv.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n"); });
  const init = await ask("initialize", { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "t", version: "1" } });
  ok("npm layout: serverInfo.version comes from the package.json", init.result?.serverInfo?.version === "9.9.9");
  const info = text(await ask("tools/call", { name: "lienzo_info", arguments: {} }));
  ok("npm layout: designs go to ~/LienzoUI/designs, not into node_modules", info.includes(path.join(home, "LienzoUI", "designs")) && !info.includes("node_modules"), info.slice(0, 300));
  const shot = await ask("tools/call", { name: "list_templates", arguments: {} });
  const made = await ask("tools/call", { name: "create_design", arguments: { file: "X", template: 0 } });
  ok("npm layout: a planted lienzo-ui.exe in node_modules is ignored", isError(made) && /LIENZO_UI/.test(text(made)), text(made).slice(0, 200));
  ok("npm layout: nothing was written under node_modules", !fs.existsSync(path.join(sandbox, "node_modules", "designs")));
  const closed = new Promise((r) => npmSrv.once("exit", r));
  npmSrv.kill();
  await closed;
  try { fs.rmSync(sandbox, { recursive: true, force: true }); } catch { /* Windows may keep the folder busy for a moment */ }
}

srv.kill();
fs.rmSync(DESIGNS, { recursive: true, force: true });
console.log(`\n${total - failures}/${total} checks passed`);
process.exit(failures ? 1 : 0);
