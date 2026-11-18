#!/usr/bin/env node
// Tests for the Lienzo UI MCP server: protocol, full flow and invalid inputs.
// Usage: node mcp/probar-mcp.mjs   (needs lienzo-ui built; exits with 1 if anything fails)
// Creates and deletes a temporary design in the designs folder.
import { spawn } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import readline from "node:readline";
import { fileURLToPath } from "node:url";

const RAIZ = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const DISENOS = fs.mkdtempSync(path.join(os.tmpdir(), "lienzo-mcp-"));
const srv = spawn(process.execPath, [path.join(RAIZ, "mcp", "lienzo-ui-mcp.mjs")], {
  stdio: ["pipe", "pipe", "inherit"], env: { ...process.env, LIENZO_DISENOS: DISENOS },
});
const pend = new Map();
readline.createInterface({ input: srv.stdout }).on("line", (l) => { const m = JSON.parse(l); pend.get(m.id)?.(m); pend.delete(m.id); });
let n = 1;
const pedir = (method, params) => new Promise((ok) => { const id = n++; pend.set(id, ok); srv.stdin.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n"); });
const llamar = (name, args) => pedir("tools/call", { name, arguments: args });
const texto = (r) => r.result?.content?.[0]?.text ?? "";
const esError = (r) => r.result?.isError === true || !!r.error;
let fallos = 0, total = 0;
const ok = (nombre, cond, det = "") => { total++; if (cond) console.log(`  ok  ${nombre}`); else { fallos++; console.log(`  ✗   ${nombre} ${det}`); } };

ok("initialize", (await pedir("initialize", { protocolVersion: "2025-06-18", capabilities: {}, clientInfo: { name: "p", version: "0" } })).result?.serverInfo?.name === "lienzo-ui");
ok("tools/list: 13 herramientas", (await pedir("tools/list", {})).result?.tools?.length === 13);
ok("listar_plantillas incluye Inventario", texto(await llamar("listar_plantillas", {})).includes("Inventario"));

console.log("Flujo completo:");
ok("listar_plantillas incluye las nuevas", texto(await llamar("listar_plantillas", {})).includes("Selección de personaje"));
ok("crear_diseno desde Menú de pausa", !esError(await llamar("crear_diseno", { archivo: "Pausa", plantilla: 6 })));
ok("revisar_diseno sin errores en la plantilla", !texto(await llamar("revisar_diseno", { archivo: "Pausa" })).includes("[ERROR]"));
ok("crear_diseno desde Inventario", !esError(await llamar("crear_diseno", { archivo: "Tienda", plantilla: 4, titulo: "Tienda del herrero" })));
const resumen = texto(await llamar("leer_diseno", { archivo: "Tienda" }));
ok("leer_diseno muestra el título", resumen.includes("Tienda del herrero"), resumen.slice(0, 80));
ok("modificar_elemento cambia texto y color", !esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Título", cambios: { text: "Herrería", text_color: "#FFD27A" } })));
ok("duplicar_elemento crea un botón nuevo", !esError(await llamar("duplicar_elemento", { archivo: "Tienda", elemento: "Cerrar", nuevo_nombre: "Comprar", dx: -80, dy: 0 })));
ok("el diseño tiene «Comprar»", texto(await llamar("leer_diseno", { archivo: "Tienda" })).includes("Comprar"));
ok("eliminar_elemento borra «Casilla 12»", !esError(await llamar("eliminar_elemento", { archivo: "Tienda", elemento: "Casilla 12" })));
ok("listar_disenos lo muestra", texto(await llamar("listar_disenos", {})).includes("Tienda.editorgui.json"));
ok("ajustar_diseno activa pixel art", !esError(await llamar("ajustar_diseno", { archivo: "Tienda", pixel_art: true })));
ok("pixel art redondea a píxeles enteros", JSON.parse(fs.readFileSync(path.join(DISENOS, "Tienda.editorgui.json"), "utf8")).elements.every((e) => Number.isInteger(e.x) && Number.isInteger(e.w)));
ok("ajustar_diseno rechaza pixel_art no booleano", esError(await llamar("ajustar_diseno", { archivo: "Tienda", pixel_art: "si" })));
ok("image rechaza archivos que no son PNG/JPG", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: { image: process.execPath } })));
ok("image rechaza rutas relativas", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: { image: "a.png" } })));
{
  // 5 simultaneous changes on the same design: with the queue none is lost
  const nombres = ["A1", "A2", "A3", "A4", "A5"];
  await Promise.all(nombres.map((n, i) => llamar("duplicar_elemento", { archivo: "Tienda", elemento: "Cerrar", nuevo_nombre: n, dx: -90 * (i + 1) })));
  const t = texto(await llamar("leer_diseno", { archivo: "Tienda" }));
  ok("órdenes simultáneas no se pisan", nombres.every((n) => t.includes(n)));
}

console.log("Entradas inválidas:");
for (const archivo of ["../fuera", "..\\fuera", "C:/Windows/win", "/etc/passwd", "a/b", ""])
  ok(`archivo rechazado «${archivo}»`, esError(await llamar("leer_diseno", { archivo })));
ok("campo no editable (id)", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: { id: 1 } })));
ok("campo __proto__", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: JSON.parse('{"__proto__": {"x": 1}}') })));
ok("tipo incorrecto (x texto)", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: { x: "10" } })));
ok("color no válido", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: { fill: "rojo" } })));
ok("w ≤ 0 rechazado", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: { w: -50 } })));
ok("opacity > 1 rechazada", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: { opacity: 5 } })));
ok("font_size ≤ 0 rechazado", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: { font_size: 0 } })));
ok("name vacío rechazado", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: { name: "  " } })));
ok("name duplicado rechazado", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: { name: "Cerrar" } })));
ok("archivo reservado «CON»", esError(await llamar("crear_diseno", { archivo: "CON", plantilla: 0 })));
fs.writeFileSync(path.join(DISENOS, "Vacio.editorgui.json"), "{}");
{ const r = await llamar("leer_diseno", { archivo: "Vacio" }); ok("diseño {} da error claro", esError(r) && !texto(r).includes("undefined"), texto(r)); }
ok("texto enorme rechazado", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: { text: "t".repeat(6000) } })));
ok("nombre enorme rechazado", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: { name: "n".repeat(500) } })));
ok("lista enorme rechazada", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Comprar", cambios: { options: Array(1000).fill("a") } })));
{
  // an element named like another's id must be found by name
  await llamar("agregar_elemento", { archivo: "Tienda", tipo: "Button", nombre: "3", x: 0, y: 0, w: 50, h: 20 });
  await llamar("modificar_elemento", { archivo: "Tienda", elemento: "3", cambios: { text: "POR-NOMBRE" } });
  const docT = JSON.parse(fs.readFileSync(path.join(DISENOS, "Tienda.editorgui.json"), "utf8"));
  ok("el nombre gana al id numérico", docT.elements.find((e) => e.name === "3")?.text === "POR-NOMBRE" && docT.elements.find((e) => e.id === 3)?.text !== "POR-NOMBRE");
}
ok("elemento inexistente", esError(await llamar("modificar_elemento", { archivo: "Tienda", elemento: "Nada", cambios: { text: "x" } })));
ok("nombre duplicado", esError(await llamar("duplicar_elemento", { archivo: "Tienda", elemento: "Cerrar", nuevo_nombre: "Comprar" })));
ok("crear sobre uno existente", esError(await llamar("crear_diseno", { archivo: "Tienda", plantilla: 0 })));
ok("plantilla inexistente", esError(await llamar("crear_diseno", { archivo: "Otro", plantilla: 99 })));
ok("exportar a carpeta sin Assets", esError(await llamar("exportar_unity", { archivo: "Tienda", proyecto_unity: os.tmpdir() })));
srv.stdin.write("{no es json}\n");
ok("JSON inválido no tumba el servidor", (await pedir("ping", {})).result !== undefined);

if (process.argv[2]) {
  // node mcp/probar-mcp.mjs <Unity project>  → also exports the store and leaves it in disenos/
  const r = await llamar("exportar_unity", { archivo: "Tienda", proyecto_unity: process.argv[2] });
  ok("exportar_unity al proyecto indicado", !esError(r), texto(r).slice(0, 200));
  fs.mkdirSync(path.join(RAIZ, "disenos"), { recursive: true });
  fs.copyFileSync(path.join(DISENOS, "Tienda.editorgui.json"), path.join(RAIZ, "disenos", "tienda.editorgui.json"));
}

srv.kill();
fs.rmSync(DISENOS, { recursive: true, force: true });
console.log(`\n${total - fallos}/${total} comprobaciones correctas`);
process.exit(fallos ? 1 : 0);
