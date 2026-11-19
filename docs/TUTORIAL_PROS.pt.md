# Tutorial para profissionais

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROS.es.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

## O que a exportação gera

Tudo é gravado em `Assets/EditorGUI` no projeto do Unity:

| Arquivo | Finalidade |
|---|---|
| `Designs/<Design>.json` | O design. O importador reconstrói o prefab sempre que ele muda. |
| `Editor/EditorGuiImporter.cs` | Constrói o prefab com uGUI + TextMeshPro. Menu **Tools › Lienzo UI**. |
| `Prefabs/<Design>.prefab` | Pronto para usar. |
| `UI Toolkit/<Design>.uxml` + `.uss` | Versão UI Toolkit (UIDocument, PanelSettings 1920×1080 Scale With Screen Size). |
| `Scripts/<Design>OnGUI.cs` | Versão IMGUI: adicione-o a um GameObject. |
| `Scripts/<Design>ViewBase.cs` | Referências tipadas e eventos (regenerado). |
| `Scripts/<Design>View.cs` | **Sua** lógica: criado uma vez e nunca sobrescrito. |
| `Sprites/`, `Fonts/` | Imagens e fontes usadas pelo design. |

`<Design>` é o título em PascalCase sem acentos (`Menú principal` → `MenuPrincipal`).

Se faltar uma imagem, nada é gravado: a exportação para antes de tocar no projeto e informa qual arquivo não pôde ser lido.

## Âncoras e escala

- Sem nada selecionado, o Inspetor mostra o **Canvas Scaler**: resolução de referência, *match* e fundo.
- Cada elemento tem âncoras, pivô e deslocamentos como um `RectTransform`. Quando a resolução de referência muda, os elementos seguem suas âncoras.
- Os modelos incluídos são projetados em 1920×1080; em páginas menores ou no modo retrato, eles são reduzidos de forma uniforme para caber.
- Verifique com **Testar** (F5) em várias telas antes de exportar.

## Layouts e listas

Um contêiner pode organizar seus filhos na horizontal, na vertical ou em grade (como os *Layout Groups* do Unity), com espaçamento e preenchimento. As ScrollViews calculam seu conteúdo. A ordem da Hierarquia é a ordem de desenho.

## Lógica

- **No editor**: no modo Testar, cada elemento pode executar um script Rhai ao clicar (`show("Name")`, `hide`, `toggle`, `set_text`, `set_color`, `move_by`…). Útil para prototipar fluxos. As alterações feitas durante o teste são descartadas ao sair do modo de teste e nunca são salvas nem exportadas.
- **No Unity**: `<Design>View` herda de `<Design>ViewBase`, com um campo por botão, slider, toggle ou campo de texto, e métodos `On<Element>Click` / `On<Element>Changed` para sobrescrever.

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

Dê nomes únicos aos elementos interativos: os scripts e `GameObject.Find` encontram apenas o primeiro. **Revisar** avisa sobre nomes repetidos.

## Revisar design

O botão **Revisar** (e `--review` na linha de comando) verifica, na resolução de referência:

| Verificação | Regra |
|---|---|
| Tamanho da fonte | Pelo menos 18 px em 1080 p |
| Contraste | WCAG: 4,5:1, ou 3:1 para texto grande (24 px, ou 18,7 px em negrito); controles desativados ficam isentos |
| Tamanho de toque | Botões, caixas de seleção, campos de texto e menus suspensos com pelo menos 64 px no lado menor em 1080 p |
| Página | Elementos total ou parcialmente fora da página |
| Nomes | Nomes repetidos em elementos interativos |
| Imagens | Arquivos de imagem que não existem mais |

Contraste abaixo de 2:1, elementos totalmente fora da página e imagens ausentes são erros; o restante são avisos.

## Linha de comando

```bash
lienzo-ui --templates                                   # lista os modelos (0-12)
lienzo-ui --template 4 --save inventory.editorgui.json
lienzo-ui --open inventory.editorgui.json --review    # código de saída 1 se houver erros
lienzo-ui --open inventory.editorgui.json --export "C:/MyGame"   # sem janela
lienzo-ui --open inventory.editorgui.json --screenshot capture.png
lienzo-ui --test 1080x2400                             # abre no modo Testar com essa tela
lienzo-ui --tutorial [--pro]                     # força o tutorial
lienzo-ui --no-tutorial                                 # não o abre
```

`--save`, `--export` e `--review` não abrem janela: use-os para scripts, CI e para o [servidor MCP](MCP.md) (ferramenta `review_design`).

## Código-fonte

Rust + [egui](https://github.com/emilk/egui):

| Arquivo | Conteúdo |
|---|---|
| `src/model.rs` | Dados: `Doc`, `Element`, `Kind`, âncoras, layouts, modelos |
| `src/canvas.rs` | Tela: seleção, alças, guias, zoom |
| `src/panels.rs` | Barra superior, barra lateral, Inspetor, janelas |
| `src/app.rs` | Estado, desfazer, comandos, atalhos |
| `src/render.rs` | Desenho de elementos e fontes |
| `src/revision.rs` | Verificações de Revisar design |
| `src/export/` | Exportadores: `ugui.rs`, `uitk.rs`, `imgui.rs`, `view.rs` e o importador do Unity |
| `src/tutorial.rs` | Tutorial integrado (páginas em espanhol e inglês) |
| `src/script.rs` | Scripts Rhai do modo Testar |

Para estendê-lo: [EXTENDING.md](EXTENDING.md).
