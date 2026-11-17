# Servidor MCP para cualquier IA

`mcp/lienzo-ui-mcp.mjs` es un servidor [MCP](https://modelcontextprotocol.io) que permite a cualquier asistente compatible crear y editar diseños de Lienzo UI, exportarlos a Unity y sacar capturas.

- Sin dependencias: **Node.js 18+**.
- Transporte **stdio**.
- Necesita el ejecutable de Lienzo UI: lo busca en la carpeta del repositorio (`lienzo-ui.exe`, `target/release`, `target/debug`) o en la variable `LIENZO_UI`.
- Los diseños viven en `disenos/` (o en `LIENZO_DISENOS`). Solo se leen y escriben archivos `.editorgui.json` de esa carpeta, por nombre simple.

## Herramientas

| Herramienta | Qué hace |
|---|---|
| `info_lienzo` | Resumen, carpeta de diseños y flujo recomendado. Llamarla primero. |
| `listar_plantillas` | Plantillas incluidas. |
| `crear_diseno` | Diseño nuevo desde una plantilla (5 = en blanco), con título opcional. |
| `listar_disenos` | Diseños guardados. |
| `leer_diseno` | Elementos: id, nombre, tipo, posición, tamaño, texto, padre. |
| `modificar_elemento` | Cambia texto, posición, tamaño, colores, fuente, visibilidad, valor, opciones… |
| `ajustar_diseno` | Título, modo pixel art, fondo y *match* del lienzo. |
| `agregar_elemento` | Añade un elemento (Image, Text, Button…), opcionalmente dentro de otro; las imágenes, por ruta absoluta a PNG/JPG. |
| `duplicar_elemento` | Copia un elemento con sus hijos (para añadir botones o casillas del mismo estilo). |
| `eliminar_elemento` | Borra un elemento y sus hijos. |
| `captura` | PNG del diseño abierto en el editor. |
| `exportar_unity` | Exporta a un proyecto de Unity (escribe en `Assets/EditorGUI`). |

## Configuración

Sustituye `RUTA/AL/REPO` por la carpeta del repositorio (o la del zip descomprimido).

**Claude Code:** el repositorio trae `.mcp.json`. A mano: `claude mcp add lienzo-ui -- node mcp/lienzo-ui-mcp.mjs`

**Claude Desktop, Cursor, Windsurf** (`claude_desktop_config.json`, `.cursor/mcp.json`…):

```json
{ "mcpServers": { "lienzo-ui": { "command": "node", "args": ["RUTA/AL/REPO/mcp/lienzo-ui-mcp.mjs"] } } }
```

**VS Code** (`.vscode/mcp.json`):

```json
{ "servers": { "lienzo-ui": { "type": "stdio", "command": "node", "args": ["${workspaceFolder}/mcp/lienzo-ui-mcp.mjs"] } } }
```

## Conectarlo a otras IAs

El servidor habla el estándar MCP por **stdio**, así que sirve para cualquier cliente que lo soporte. La app **no guarda claves de ninguna IA**: la clave (o el modelo local) la pones en el cliente que elijas, nunca en Lienzo UI ni en el repositorio.

Antes de empezar: Node.js 18+ y la carpeta del repositorio (o del zip descomprimido). En todos los casos el comando es el mismo: `node RUTA/AL/REPO/mcp/lienzo-ui-mcp.mjs`.

| Cliente | Cómo se añade |
|---|---|
| **Claude Code** | `claude mcp add lienzo-ui -- node mcp/lienzo-ui-mcp.mjs` (o usa el `.mcp.json` del repositorio) |
| **Claude Desktop, Cursor, Windsurf** | Bloque `mcpServers` de su archivo de configuración (ver arriba) |
| **VS Code (Copilot)** | `.vscode/mcp.json` (ver arriba) |
| **Codex CLI** | `codex mcp add lienzo-ui -- node RUTA/AL/REPO/mcp/lienzo-ui-mcp.mjs` |
| **Gemini CLI** | En `~/.gemini/settings.json`: `{ "mcpServers": { "lienzo-ui": { "command": "node", "args": ["RUTA/AL/REPO/mcp/lienzo-ui-mcp.mjs"] } } }` |
| **Cline, Continue, Zed, LM Studio, Open WebUI…** | Añade un servidor MCP «stdio» con `command: node` y el argumento de arriba (cada uno lo llama algo distinto en sus ajustes) |

Los nombres de los comandos y archivos de configuración cambian con las versiones de cada programa: si alguno no coincide, busca en su documentación «add MCP server (stdio)». Para comprobar que Lienzo UI responde, pídele a la IA: «llama a `info_lienzo`».

**Con un modelo local (sin enviar nada a internet):** usa un cliente MCP que funcione con modelos locales (LM Studio, Open WebUI, Cline con Ollama…) y añade el mismo servidor. Elige un modelo que soporte *tool calling*.

## Usar la API de una IA para generar cosas dentro de la app

Lienzo UI no llama a ninguna IA por su cuenta: el flujo es que **una IA con tu clave maneja el editor a través de MCP** y tú ves el resultado en la ventana (el diseño abierto se actualiza y la herramienta `captura` le deja comprobar cómo quedó).

**Opción A: un cliente MCP con tu clave (la más fácil).** Configura Claude, Cursor, VS Code, Codex, Gemini… con **su propia clave**, como indica cada programa, y añade el servidor de arriba. Luego pide en lenguaje natural: «crea un HUD con vida y maná en modo pixel art y expórtalo a mi proyecto».

**Opción B: tu propio script con la API de una IA.** Para automatizar (por ejemplo, generar 20 pantallas desde una lista), arranca el servidor como proceso hijo, pasa sus herramientas al modelo y devuelve las llamadas. Este ejemplo usa el SDK oficial de MCP y cualquier API compatible con el formato de OpenAI (OpenAI, DeepSeek, OpenRouter, Ollama… cambiando `baseURL` y `model`). No está probado con todos los proveedores:

```js
// npm i @modelcontextprotocol/sdk openai
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import OpenAI from "openai";

const mcp = new Client({ name: "mi-script", version: "1.0.0" });
await mcp.connect(new StdioClientTransport({ command: "node", args: ["RUTA/AL/REPO/mcp/lienzo-ui-mcp.mjs"] }));
const { tools } = await mcp.listTools();

// La clave sale SOLO de una variable de entorno (nunca del código ni del repositorio)
const ia = new OpenAI({ apiKey: process.env.IA_API_KEY, baseURL: process.env.IA_BASE_URL });
const mensajes = [{ role: "user", content: "Crea un menú principal llamado Ecos y expórtalo a C:/MiJuego" }];
const herramientas = tools.map(t => ({ type: "function", function: { name: t.name, description: t.description, parameters: t.inputSchema } }));

for (let i = 0; i < 20; i++) {                       // límite de pasos
  const r = (await ia.chat.completions.create({ model: process.env.IA_MODELO, messages: mensajes, tools: herramientas })).choices[0].message;
  mensajes.push(r);
  if (!r.tool_calls?.length) { console.log(r.content); break; }
  for (const c of r.tool_calls) {
    const out = await mcp.callTool({ name: c.function.name, arguments: JSON.parse(c.function.arguments) });
    mensajes.push({ role: "tool", tool_call_id: c.id, content: JSON.stringify(out.content) });
  }
}
await mcp.close();
```

Ejecútalo con las variables en tu consola, sin escribirlas en ningún archivo del proyecto:

```powershell
$env:IA_API_KEY = "…"; $env:IA_BASE_URL = "https://api.openai.com/v1"; $env:IA_MODELO = "nombre-del-modelo"; node mi-script.mjs
```

**Seguridad:**
- No pegues claves en el chat, en el código ni en `.mcp.json`; usa variables de entorno o el gestor de secretos de tu cliente.
- El servidor solo lee y escribe `.editorgui.json` de la carpeta de diseños y exporta donde le indiques; revisa lo que la IA va a exportar.
- Un modelo puede equivocarse: trabaja sobre copias de tus diseños y usa `captura` para verificar el resultado.

## Ejemplos de peticiones

- «Crea una tienda a partir de la plantilla de inventario, titúlala Herrería y añade un botón verde Comprar.»
- «En el menú principal cambia el título a Ecos del Abismo y pon los botones en naranja.»
- «Exporta todos los diseños a C:/MiJuego y haz una captura de cada uno.»

## Pruebas

```bash
node mcp/probar-mcp.mjs                 # 32 comprobaciones (protocolo, flujo y entradas maliciosas)
node mcp/probar-mcp.mjs "C:/MiJuego"    # además exporta un diseño de prueba a ese proyecto
```
