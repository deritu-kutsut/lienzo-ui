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

Adicione --dry-run a --export para listar cada arquivo como new, update, same ou remove sem gravar nada.

export_unity grava direto em Assets/EditorGUI do projeto indicado pela IA, sem nenhum aviso no editor: a aprovação é a das ferramentas do seu cliente MCP. Peça à IA para chamá-lo antes com dry_run: true, leia a lista e depois exporte de verdade.

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

## Layout vertical em detalhe

O layout vertical é uma segunda visualização do mesmo design (docs/FORMAT.md › PortraitLayout):

- Tem a sua própria resolução de referência, “Tamanho vertical” (1080 × 1920 por padrão).
- Elementos sem exceção mantêm o RectTransform horizontal e são reavaliados em relação à página vertical: um fundo de tela cheia se estica, e os ancorados à direita acompanham a borda direita.
- Só é guardado o que você altera na visualização vertical, como exceções (posição, tamanho, rotação, âncoras, visibilidade, tamanho da fonte, layout). Clique direito › “Voltar ao horizontal” remove uma.
- Exportação: o JSON leva as exceções e o script gerado LienzoOrientationLayout as aplica quando Screen.width < Screen.height, e de novo quando o tamanho muda.
- “Remover layout vertical” apaga todas as exceções.

Os modelos já trazem um layout vertical embutido, guardado como exceções do horizontal, então Retrato mostra desde o início um layout pensado para celulares.

## 9-slice e o pacote incluído

Para molduras, botões e painéis que esticam:

- Selecione uma Imagem e defina “9-slice” em “Ajustes de imagem (Unity)”: a borda em pixels da imagem original (esquerda, abaixo, direita, acima), como no Sprite Editor do Unity. A exportação gera um Sprite 9-slice.
- “Pixels por unidade”: 100 mantém as bordas no tamanho em pixels, 200 as reduz à metade.
- O pacote incluído (assets/pack/manifest.json, docs/CONTENT.md) já define o corte e os pixels por unidade: molduras de 512×512 com corte 96, botões de 512×192.
- Usar um item o copia para a sua biblioteca como pack_<id>.<ext> (oculto em “Suas imagens”); o exportador o copia para os Sprites do Unity.
- Os modelos Pixel RPG HUD e Pixel main menu usam o pacote pixel. Ative “Pixel art” para o filtro Point.

## Revisão e correções rápidas

A janela Revisar lista erros e avisos; clique em um para selecionar o elemento. Seis problemas têm um botão “Corrigir” (um passo de desfazer):

- Texto abaixo do tamanho mínimo: aumentado até o mínimo.
- Controle pequeno demais para tocar: ampliado em torno do centro.
- Elemento parcialmente fora da página: movido para dentro, se couber.
- Nome repetido: acrescenta-se _2, _3…
- Imagem ausente: a referência é removida.
- Pouco contraste: a cor do texto é escurecida ou clareada.

“Corrigir todos os problemas seguros” aplica tudo em algumas passadas. Problemas como uma fonte sem certos caracteres não têm correção automática. As correções também funcionam no layout vertical. Pela linha de comando: lienzo-ui --open arquivo --review (código 1 se houver erros).

## Importar do Unity: prefabs e CLI

Arquivo › Importar do Unity… lê um .prefab de uGUI (o YAML de texto do Unity) ou um Designs/*.json exportado pelo Lienzo UI e cria um design comum.

- O que não pôde ser convertido aparece como avisos.
- Se detectar a pasta do projeto Unity, as imagens são resolvidas a partir dela durante a sessão.
- Sem janela: lienzo-ui --import-unity Menu.prefab --save Menu.editorgui.json (também com --review e --export).
- Projeto inteiro: lienzo-ui --import-unity-project MeuProjeto --out Importados converte todos os prefabs de interface. Nunca sobrescreve (acrescenta _2) e ignora prefabs sem interface.
- Detalhes em docs/UNITY_IMPORT.md.

## Comunidade: plugins, fontes e regras

Na aba Comunidade, Instalar adiciona conteúdo compartilhado com um clique: plugins, fontes, imagens, modelos, elementos e temas.

- Um plugin da comunidade vai para uma pasta chamada community-<nome>-<hash>. Ele é gravado primeiro em uma pasta temporária, então uma instalação interrompida nunca deixa um plugin pela metade. Aparece na aba Plugins.
- Os scripts são os mesmos Rhai: isolados, sem acesso a arquivos nem à rede e com limites de operações, tempo e memória (docs/PLUGINS.md).
- Uma fonte instalada é adicionada à aba Texto e exportada para o Unity junto com o design.
- Os downloads são conferidos com a impressão digital SHA-256 publicada; se não coincidir, nada é instalado.
- Para publicar, você entra (GitHub ou Google), escolhe um nome público e aceita as regras. Tudo é revisado antes e o conteúdo +18 precisa ser marcado. Veja docs/TERMS.md, docs/COMMUNITY_GUIDELINES.md e docs/PRIVACY.md.

Prévia mostra uma fonte com texto de exemplo, ou os comandos, exportadores, arquivos e o código-fonte de um plugin (somente leitura), antes de instalar; nada é instalado e nenhum download é contado. Traduzir envia só essa descrição ao serviço gratuito MyMemory quando você pressiona (docs/PRIVACY.md).

## Salvamento automático, recuperação e desfazer

- Mais ou menos a cada minuto, enquanto você edita, o design completo (com o layout vertical) é gravado na pasta de recuperação do app. Nunca no modo Testar. No máximo 10 recuperações são mantidas.
- Ao iniciar, se houver uma recuperação mais recente que o arquivo do projeto, ela é oferecida: Restaurar ou Descartar. Depois de um Salvar normal ela deixa de ser necessária.
- Pode ser desativado em Acessibilidade › Salvamento automático.
- O histórico de desfazer é limitado a 200 passos e cerca de 64 MB. Quando os passos mais antigos são descartados, a barra de status mostra “Histórico reduzido”.
- Cada comando de plugin e cada Corrigir da janela Revisar é um único passo de desfazer.
