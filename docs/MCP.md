# MCP server for any AI

`mcp/lienzo-ui-mcp.mjs` is an [MCP](https://modelcontextprotocol.io) server that lets any compatible assistant create and edit Lienzo UI designs, export them to Unity and take screenshots.

- No dependencies: **Node.js 18+**.
- **stdio** transport.
- It needs the Lienzo UI executable: it looks for it in the repository folder (`lienzo-ui.exe`, `target/release`, `target/debug`) or in the `LIENZO_UI` variable.
- Designs live in `disenos/` (or in `LIENZO_DISENOS`). Only `.editorgui.json` files in that folder are read and written, by plain file name.

## Tools

| Tool | What it does |
|---|---|
| `info_lienzo` | Summary, designs folder and recommended workflow. Call it first. |
| `listar_plantillas` | Included templates. |
| `crear_diseno` | New design from a template (0-12; 5 = blank), with an optional title. |
| `revisar_diseno` | Checks legibility, contrast, touch target size, elements outside the page, repeated names and missing images. |
| `listar_disenos` | Saved designs. |
| `leer_diseno` | Elements: id, name, type, position, size, text, parent. |
| `modificar_elemento` | Changes text, position, size, colors, font, visibility, value, options… |
| `ajustar_diseno` | Title, pixel art mode, background and canvas *match*. |
| `agregar_elemento` | Adds an element (Image, Text, Button…), optionally inside another one; images by absolute path to a PNG/JPG. |
| `duplicar_elemento` | Copies an element with its children (to add buttons or checkboxes with the same style). |
| `eliminar_elemento` | Deletes an element and its children. |
| `captura` | PNG of the design open in the editor. |
| `exportar_unity` | Exports to a Unity project (writes to `Assets/EditorGUI`). |

## Setup

Replace `RUTA/AL/REPO` with the repository folder (or the unzipped folder).

**Claude Code:** the repository ships with `.mcp.json`. By hand: `claude mcp add lienzo-ui -- node mcp/lienzo-ui-mcp.mjs`

**Claude Desktop, Cursor, Windsurf** (`claude_desktop_config.json`, `.cursor/mcp.json`…):

```json
{ "mcpServers": { "lienzo-ui": { "command": "node", "args": ["RUTA/AL/REPO/mcp/lienzo-ui-mcp.mjs"] } } }
```

**VS Code** (`.vscode/mcp.json`):

```json
{ "servers": { "lienzo-ui": { "type": "stdio", "command": "node", "args": ["${workspaceFolder}/mcp/lienzo-ui-mcp.mjs"] } } }
```

## Connecting it to other AIs

The server speaks the MCP standard over **stdio**, so it works with any client that supports it. The app **does not store keys for any AI**: you put the key (or the local model) in the client you choose, never in Lienzo UI or in the repository.

Before you start: Node.js 18+ and the repository folder (or the unzipped folder). In every case the command is the same: `node RUTA/AL/REPO/mcp/lienzo-ui-mcp.mjs`.

| Client | How to add it |
|---|---|
| **Claude Code** | `claude mcp add lienzo-ui -- node mcp/lienzo-ui-mcp.mjs` (or use the repository's `.mcp.json`) |
| **Claude Desktop, Cursor, Windsurf** | `mcpServers` block in their configuration file (see above) |
| **VS Code (Copilot)** | `.vscode/mcp.json` (see above) |
| **Codex CLI** | `codex mcp add lienzo-ui -- node RUTA/AL/REPO/mcp/lienzo-ui-mcp.mjs` |
| **Gemini CLI** | In `~/.gemini/settings.json`: `{ "mcpServers": { "lienzo-ui": { "command": "node", "args": ["RUTA/AL/REPO/mcp/lienzo-ui-mcp.mjs"] } } }` |
| **Cline, Continue, Zed, LM Studio, Open WebUI…** | Add a "stdio" MCP server with `command: node` and the argument above (each one calls it something different in its settings) |

Command names and configuration files change between versions of each program: if one doesn't match, look in its documentation for "add MCP server (stdio)". To check that Lienzo UI responds, ask the AI: "call `info_lienzo`".

**With a local model (nothing sent to the internet):** use an MCP client that works with local models (LM Studio, Open WebUI, Cline with Ollama…) and add the same server. Pick a model that supports *tool calling*.

## Using an AI's API to generate things inside the app

Lienzo UI does not call any AI on its own: the workflow is that **an AI with your key drives the editor through MCP** and you see the result in the window (the open design updates, and the `captura` tool lets the AI check how it turned out).

**Option A: an MCP client with your key (the easiest).** Configure Claude, Cursor, VS Code, Codex, Gemini… with **its own key**, as each program indicates, and add the server above. Then ask in natural language: "create a HUD with health and mana in pixel art mode and export it to my project".

**Option B: your own script with an AI's API.** To automate (for example, generating 20 screens from a list), start the server as a child process, pass its tools to the model and return the calls. This example uses the official MCP SDK and any API compatible with the OpenAI format (OpenAI, DeepSeek, OpenRouter, Ollama… by changing `baseURL` and `model`). It has not been tested with every provider:

```js
// npm i @modelcontextprotocol/sdk openai
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import OpenAI from "openai";

const mcp = new Client({ name: "mi-script", version: "1.0.0" });
await mcp.connect(new StdioClientTransport({ command: "node", args: ["RUTA/AL/REPO/mcp/lienzo-ui-mcp.mjs"] }));
const { tools } = await mcp.listTools();

// The key comes ONLY from an environment variable (never from the code or the repository)
const ia = new OpenAI({ apiKey: process.env.IA_API_KEY, baseURL: process.env.IA_BASE_URL });
const mensajes = [{ role: "user", content: "Crea un menú principal llamado Ecos y expórtalo a C:/MiJuego" }];
const herramientas = tools.map(t => ({ type: "function", function: { name: t.name, description: t.description, parameters: t.inputSchema } }));

for (let i = 0; i < 20; i++) {                       // step limit
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

Run it with the variables set in your console, without writing them to any file in the project:

```powershell
$env:IA_API_KEY = "…"; $env:IA_BASE_URL = "https://api.openai.com/v1"; $env:IA_MODELO = "nombre-del-modelo"; node mi-script.mjs
```

**Security:**
- Don't paste keys into the chat, the code or `.mcp.json`; use environment variables or your client's secrets manager.
- The server only reads and writes `.editorgui.json` files in the designs folder and exports where you tell it to; review what the AI is about to export.
- A model can make mistakes: work on copies of your designs and use `captura` to verify the result.

## Example requests

- "Create a shop from the inventory template, title it Blacksmith and add a green Buy button."
- "In the main menu change the title to Echoes of the Abyss and make the buttons orange."
- "Export all designs to C:/MiJuego and take a screenshot of each one."

## Tests

```bash
node mcp/probar-mcp.mjs                 # 32 checks (protocol, workflow and malicious inputs)
node mcp/probar-mcp.mjs "C:/MiJuego"    # also exports a test design to that project
```
