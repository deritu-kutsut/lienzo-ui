# MCP server for any AI

`mcp/lienzo-ui-mcp.mjs` is an [MCP](https://modelcontextprotocol.io) server that lets any compatible assistant create and edit Lienzo UI designs, export them to Unity and take screenshots.

- No dependencies: **Node.js 18+**.
- **stdio** transport.
- Tool names, parameters, messages and the designs folder are English (see [Legacy names](#legacy-names) for the old Spanish ones).
- It needs the Lienzo UI executable: it looks for it in the repository folder (`lienzo-ui.exe`, `target/release`, `target/debug`) or in the `LIENZO_UI` variable.
- Designs live in `designs/` (or in `LIENZO_DESIGNS`). Only `.editorgui.json` files in that folder are read and written, by plain file name.

## Tools

| Tool | What it does |
|---|---|
| `lienzo_info` | Summary, designs folder and recommended workflow. Call it first. |
| `list_templates` | Included templates. |
| `create_design` | New design from a template (0-12; 5 = blank), with an optional title. |
| `review_design` | Checks legibility, contrast, touch target size, elements outside the page, repeated names and missing images. |
| `list_designs` | Saved designs. |
| `read_design` | Elements: id, name, type, position, size, text, parent. |
| `modify_element` | Changes text, position, size, colors, font, visibility, value, options… |
| `adjust_design` | Title, pixel art mode, background and canvas *match*. |
| `add_element` | Adds an element (Image, Text, Button…), optionally inside another one; images by absolute path to a PNG/JPG. |
| `duplicate_element` | Copies an element with its children (to add buttons or checkboxes with the same style). |
| `delete_element` | Deletes an element and its children. |
| `screenshot` | PNG of the design open in the editor. |
| `export_unity` | Exports to a Unity project (writes to `Assets/EditorGUI`). |

Parameters (all tools take `file`, the design name without folders; `.editorgui.json` is optional):

| Tool | Parameters |
|---|---|
| `create_design` | `file`, `template` (number), `title` (optional) |
| `adjust_design` | `file`, `title`, `pixel_art`, `background` (`"#RRGGBB"`), `match` (0-1) |
| `modify_element` | `file`, `element` (name or id), `changes` (object: `text`, `x`, `y`, `w`, `h`, `fill`, `text_color`, `font_size`, `visible`, `image`…) |
| `add_element` | `file`, `type`, `name`, `x`, `y`, `w`, `h`, optional `parent`, `text`, `image`, `color` |
| `duplicate_element` | `file`, `element`, `new_name`, optional `dx`, `dy`, `text` |
| `delete_element` | `file`, `element` |
| `export_unity` | `file`, `unity_project` (the folder that contains `Assets`) |

## Setup

Replace `PATH/TO/REPO` with the repository folder (or the unzipped folder).

**Claude Code:** the repository ships with `.mcp.json`. By hand: `claude mcp add lienzo-ui -- node mcp/lienzo-ui-mcp.mjs`

**Claude Desktop, Cursor, Windsurf** (`claude_desktop_config.json`, `.cursor/mcp.json`…):

```json
{ "mcpServers": { "lienzo-ui": { "command": "node", "args": ["PATH/TO/REPO/mcp/lienzo-ui-mcp.mjs"] } } }
```

**VS Code** (`.vscode/mcp.json`):

```json
{ "servers": { "lienzo-ui": { "type": "stdio", "command": "node", "args": ["${workspaceFolder}/mcp/lienzo-ui-mcp.mjs"] } } }
```

## Connecting it to other AIs

The server speaks the MCP standard over **stdio**, so it works with any client that supports it. The app **does not store keys for any AI**: you put the key (or the local model) in the client you choose, never in Lienzo UI or in the repository.

Before you start: Node.js 18+ and the repository folder (or the unzipped folder). In every case the command is the same: `node PATH/TO/REPO/mcp/lienzo-ui-mcp.mjs`.

| Client | How to add it |
|---|---|
| **Claude Code** | `claude mcp add lienzo-ui -- node mcp/lienzo-ui-mcp.mjs` (or use the repository's `.mcp.json`) |
| **Claude Desktop, Cursor, Windsurf** | `mcpServers` block in their configuration file (see above) |
| **VS Code (Copilot)** | `.vscode/mcp.json` (see above) |
| **Codex CLI** | `codex mcp add lienzo-ui -- node PATH/TO/REPO/mcp/lienzo-ui-mcp.mjs` |
| **Gemini CLI** | In `~/.gemini/settings.json`: `{ "mcpServers": { "lienzo-ui": { "command": "node", "args": ["PATH/TO/REPO/mcp/lienzo-ui-mcp.mjs"] } } }` |
| **Cline, Continue, Zed, LM Studio, Open WebUI…** | Add a "stdio" MCP server with `command: node` and the argument above (each one calls it something different in its settings) |

Command names and configuration files change between versions of each program: if one doesn't match, look in its documentation for "add MCP server (stdio)". To check that Lienzo UI responds, ask the AI: "call `lienzo_info`".

**With a local model (nothing sent to the internet):** use an MCP client that works with local models (LM Studio, Open WebUI, Cline with Ollama…) and add the same server. Pick a model that supports *tool calling*.

## Using an AI's API to generate things inside the app

Lienzo UI does not call any AI on its own: the workflow is that **an AI with your key drives the editor through MCP** and you see the result in the window (the open design updates, and the `screenshot` tool lets the AI check how it turned out).

**Option A: an MCP client with your key (the easiest).** Configure Claude, Cursor, VS Code, Codex, Gemini… with **its own key**, as each program indicates, and add the server above. Then ask in natural language: "create a HUD with health and mana in pixel art mode and export it to my project".

**Option B: your own script with an AI's API.** To automate (for example, generating 20 screens from a list), start the server as a child process, pass its tools to the model and return the calls. This example uses the official MCP SDK and any API compatible with the OpenAI format (OpenAI, DeepSeek, OpenRouter, Ollama… by changing `baseURL` and `model`). It has not been tested with every provider:

```js
// npm i @modelcontextprotocol/sdk openai
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { StdioClientTransport } from "@modelcontextprotocol/sdk/client/stdio.js";
import OpenAI from "openai";

const mcp = new Client({ name: "my-script", version: "1.0.0" });
await mcp.connect(new StdioClientTransport({ command: "node", args: ["PATH/TO/REPO/mcp/lienzo-ui-mcp.mjs"] }));
const { tools } = await mcp.listTools();

// The key comes ONLY from an environment variable (never from the code or the repository)
const ai = new OpenAI({ apiKey: process.env.AI_API_KEY, baseURL: process.env.AI_BASE_URL });
const messages = [{ role: "user", content: "Create a main menu called Echoes and export it to C:/MyGame" }];
const toolList = tools.map(t => ({ type: "function", function: { name: t.name, description: t.description, parameters: t.inputSchema } }));

for (let i = 0; i < 20; i++) {                       // step limit
  const r = (await ai.chat.completions.create({ model: process.env.AI_MODEL, messages, tools: toolList })).choices[0].message;
  messages.push(r);
  if (!r.tool_calls?.length) { console.log(r.content); break; }
  for (const c of r.tool_calls) {
    const out = await mcp.callTool({ name: c.function.name, arguments: JSON.parse(c.function.arguments) });
    messages.push({ role: "tool", tool_call_id: c.id, content: JSON.stringify(out.content) });
  }
}
await mcp.close();
```

Run it with the variables set in your console, without writing them to any file in the project:

```powershell
$env:AI_API_KEY = "…"; $env:AI_BASE_URL = "https://api.openai.com/v1"; $env:AI_MODEL = "model-name"; node my-script.mjs
```

**Security:**
- Don't paste keys into the chat, the code or `.mcp.json`; use environment variables or your client's secrets manager.
- The server only reads and writes `.editorgui.json` files in the designs folder and exports where you tell it to; review what the AI is about to export.
- A model can make mistakes: work on copies of your designs and use `screenshot` to verify the result.

## Example requests

- "Create a shop from the inventory template, title it Blacksmith and add a green Buy button."
- "In the main menu change the title to Echoes of the Abyss and make the buttons orange."
- "Export all designs to C:/MyGame and take a screenshot of each one."

## Tests

```bash
node mcp/probar-mcp.mjs                 # protocol, workflow, malicious inputs and legacy aliases
node mcp/probar-mcp.mjs "C:/MyGame"    # also exports a test design to that project
```

## Legacy names

Everything used to be Spanish, and **the old names still work** (nothing to change in existing configurations or prompts):

- **Tools** (hidden aliases, not listed by `tools/list`): `info_lienzo`, `listar_plantillas`, `crear_diseno`, `ajustar_diseno`, `revisar_diseno`, `listar_disenos`, `leer_diseno`, `modificar_elemento`, `agregar_elemento`, `duplicar_elemento`, `eliminar_elemento`, `captura`, `exportar_unity`.
- **Parameters**: `archivo` (`file`), `plantilla` (`template`), `titulo` (`title`), `fondo` (`background`), `elemento` (`element`), `cambios` (`changes`), `tipo` (`type`), `nombre` (`name`), `padre` (`parent`), `texto` (`text`), `nuevo_nombre` (`new_name`), `proyecto_unity` (`unity_project`). If both the old and the new name are sent, the new one wins.
- **Environment**: `LIENZO_DISENOS` is read when `LIENZO_DESIGNS` is not set, and an existing `disenos/` folder next to `mcp/` is used when there is no `designs/` folder.
- **Command line** of the app: the old flags (`--abrir`, `--exportar`, `--guardar`, `--plantilla`, `--revisar`…) are rewritten to `--open`, `--export`, `--save`, `--template`, `--review`…
