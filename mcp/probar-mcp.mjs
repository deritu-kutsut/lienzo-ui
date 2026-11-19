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

const { srv, request, call } = start({ LIENZO_DESIGNS: DESIGNS });
const text = (r) => r.result?.content?.[0]?.text ?? "";
const isError = (r) => r.result?.isError === true || !!r.error;
let failures = 0, total = 0;
const ok = (name, cond, detail = "") => { total++; if (cond) console.log(`  ok  ${name}`); else { failures++; console.log(`  ✗   ${name} ${detail}`); } };

ok("initialize", (await request("initialize", { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "p", version: "0" } })).result?.serverInfo?.name === "lienzo-ui");
const listed = (await request("tools/list", {})).result?.tools ?? [];
ok("tools/list: 13 tools", listed.length === 13);
ok("tools/list: English names only", listed.every((t) => /^[a-z_]+$/.test(t.name)) && ["lienzo_info", "list_templates", "create_design", "adjust_design", "review_design", "list_designs", "read_design", "modify_element", "add_element", "duplicate_element", "delete_element", "screenshot", "export_unity"].every((n) => listed.some((t) => t.name === n)));
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
ok("template does not exist", isError(await call("create_design", { file: "Other", template: 99 })));
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
srv.stdin.write("{not json}\n");
ok("invalid JSON does not bring the server down", (await request("ping", {})).result !== undefined);

if (process.argv[2]) {
  // node mcp/probar-mcp.mjs <Unity project>  → also exports the shop and leaves it in designs/
  const r = await call("export_unity", { file: "Shop", unity_project: process.argv[2] });
  ok("export_unity to the given project", !isError(r), text(r).slice(0, 200));
  fs.mkdirSync(path.join(ROOT, "designs"), { recursive: true });
  fs.copyFileSync(path.join(DESIGNS, "Shop.editorgui.json"), path.join(ROOT, "designs", "shop.editorgui.json"));
}

srv.kill();
fs.rmSync(DESIGNS, { recursive: true, force: true });
console.log(`\n${total - failures}/${total} checks passed`);
process.exit(failures ? 1 : 0);
