# Tutorial para iniciantes

🌐 [English](TUTORIAL_BEGINNERS.md) · [Español](TUTORIAL_BEGINNERS.es.md) · [Português](TUTORIAL_BEGINNERS.pt.md) · [Français](TUTORIAL_BEGINNERS.fr.md) · [Deutsch](TUTORIAL_BEGINNERS.de.md) · [Italiano](TUTORIAL_BEGINNERS.it.md) · [Русский](TUTORIAL_BEGINNERS.ru.md) · [简体中文](TUTORIAL_BEGINNERS.zh.md) · [日本語](TUTORIAL_BEGINNERS.ja.md) · [한국어](TUTORIAL_BEGINNERS.ko.md)

Sua primeira interface com o Lienzo UI e sua primeira exportação para o Unity. Não precisa saber programar.

## 1. Instalar e abrir

1. Baixe `LienzoUI-windows.zip` nas releases do GitHub ou no itch.io.
2. Descompacte o arquivo e abra `lienzo-ui.exe`.
3. Na primeira vez, o **tutorial** se abre (este mesmo conteúdo, dentro do app). Ao terminá-lo, ou com «Pular e ir ao editor», o editor se abre. Para vê-lo novamente: botão **Tutorial** na barra superior ou **F1**.

Para usar o app em português: botão **Acessibilidade** › **Idioma** › Português.

![Tutorial dentro do app](images/tutorial.png)

## 2. A tela

| Área | O que há |
|---|---|
| Barra superior | Arquivo, Editar, Ver, Tela, **Testar**, nome do design, **Acessibilidade**, **Revisar**, **Temas**, **Tutorial**, **Salvar**, **Código…**, **Exportar para Unity** |
| Barra esquerda | **Hierarquia** (camadas), **Elementos**, **Modelos**, **Texto**, **Imagens**, **Plugins**, **Comunidade** |
| Centro | A tela: a tela do seu jogo (1920 × 1080 por padrão) |
| Direita | O **Inspetor**: propriedades da seleção |

![Editor](images/editor.png)

## 3. Comece com um modelo

Em **Modelos**, escolha um dos 15 layouts: Menu principal, HUD de jogo, Janela de diálogo, Configurações, Inventário, Menu de pausa, Fim de jogo, Seleção de fase, Tela de carregamento, Diálogo de personagem, Classificação ou Seleção de personagem. Tudo o que ele traz pode ser alterado, e Ctrl+Z desfaz a alteração. Para começar do zero: «Em branco». Duas delas têm estilo pixel art: HUD de RPG pixel art e Menu principal pixel art.

## 4. Edite

- Arraste elementos de **Elementos** para a tela. Em **Componentes prontos** há botões, barras de vida e mana, cartões, insígnias e slots de inventário.
- **Texto** traz estilos de título e peças de HUD: placar, temporizador, moedas, números de dano, combo, nível e legendas.
- Clique seleciona; arrastar move; os cantos mudam o tamanho; a alça superior gira.
- Dê um duplo clique em um texto para digitar.
- No **Inspetor**, altere cores, bordas, sombras, fontes e âncoras.

| Atalho | Ação |
|---|---|
| Ctrl+Z / Ctrl+Y | Desfazer / refazer |
| Ctrl+D / Del | Duplicar / excluir |
| Ctrl+G / Ctrl+Shift+G | Agrupar / desagrupar |
| Setas (Shift) | Mover 1 px (10 px) |
| Ctrl+S | Salvar |
| F5 | Testar |
| Ctrl+E | Exportar para Unity |
| F1 | Tutorial |

## 5. Teste

Clique em **Testar** (F5): a interface fica interativa e você pode escolher outras telas (celular, 4:3, ultrawide) para ver como ela se adapta. Esc sai do modo de teste e o design volta a ficar como estava antes do teste.

## 6. Revise

Clique em **Revisar** na barra superior. Ele avisa sobre fonte pequena demais para ler, baixo contraste, botões pequenos demais para tocar, elementos fora da página, nomes repetidos e imagens ausentes. Clique em um aviso para selecionar esse elemento e corrigi-lo.

## 7. Exporte para o Unity

1. Clique em **Exportar para Unity** (Ctrl+E). Na primeira vez, escolha a pasta do seu projeto (a que contém `Assets`).
2. Volte ao Unity: o design aparece como um prefab em `Assets/EditorGUI/Prefabs`.
3. Arraste-o para a cena ou use o menu **Tools › Lienzo UI › Import design into scene…**.

Sempre que você exportar novamente, o Unity atualiza o prefab sozinho.

## Imagens incluídas

A aba Imagens na barra à esquerda traz arte pronta para usar, sem precisar de arquivos:

- Os chips no topo filtram por Suas imagens, Planos de fundo, Molduras, Botões, Ícones, Pixel art e Gradientes. A busca encontra por nome ou tag.
- Clique em um item para adicioná-lo ou arraste-o para o canvas. Molduras e botões vêm com a borda 9-slice, então esticam sem deformar os cantos. Os planos de fundo ficam atrás de tudo.
- A imagem é copiada para a sua biblioteca na primeira vez que você a usa e é exportada para o Unity junto com o design.
- Com os sprites de pixel art, marque “Pixel art” no Inspetor do canvas (sem nada selecionado) para que fiquem nítidos.

## Telas verticais

Celulares costumam ser usados na vertical. O Lienzo UI guarda um segundo layout para isso, sem mexer no primeiro:

- Troque a visualização com os botões Paisagem / Retrato da barra superior.
- Sem nada selecionado, abra “Layout vertical” no Inspetor e clique em “Criar layout vertical”. Os elementos começam onde estão na horizontal.
- Na visualização vertical, o que você mover ou redimensionar altera só o layout vertical. Clique direito em um elemento › “Voltar ao horizontal” desfaz uma alteração.
- A exportação leva os dois layouts: no Unity o prefab muda sozinho para o vertical quando a tela é mais alta do que larga.

Cada modelo já vem com o próprio layout vertical: aplique um modelo e pressione Retrato para vê-lo pensado para celulares (1080 × 1920).

## Importar do Unity

Já tem uma interface no Unity? Traga-a para o Lienzo UI:

- Arquivo › Importar do Unity…
- Escolha um prefab de uGUI (.prefab) ou um design que o Lienzo UI exportou antes (Assets/EditorGUI/Designs/….json).
- Ele abre como um design novo, sem salvar: salve-o como projeto para mantê-lo.
- Se algo não puder ser convertido com exatidão, uma janela lista os avisos, então nada se perde em silêncio.
- Se ele encontrar a pasta do seu projeto Unity, as imagens do design são localizadas automaticamente.

## Não perca seu trabalho

- Salvamento automático: mais ou menos a cada minuto o Lienzo UI salva uma cópia de recuperação. Você pode desativar em Acessibilidade › Salvamento automático.
- Se o aplicativo fechar de forma inesperada, na próxima vez ele pergunta “Recuperar o trabalho não salvo…?”. Clique em Restaurar para recuperá-lo ou em Descartar para ignorá-lo. O trabalho recuperado ainda não é um arquivo salvo: pressione Ctrl+S para mantê-lo.
- Ctrl+S continua salvando o seu projeto como sempre.
- Desfazer (Ctrl+Z) lembra até 200 passos (e tem um limite de memória). Se os mais antigos forem descartados, a barra de status mostra “Histórico reduzido”.

## Comunidade

A aba Comunidade na barra à esquerda é um catálogo compartilhado de plugins, imagens, fontes, modelos, elementos e temas:

- Explorar e instalar não exige conta: pesquise e clique em Instalar.
- Para enviar, denunciar ou dar corações, você entra com GitHub ou Google e escolhe um nome público. Sem senhas: o app guarda apenas esse nome.
- Ao criar seu perfil você aceita as regras: nada de malware, ódio, plágio nem conteúdo +18 sem marcação. O texto completo está em docs/TERMS.md e docs/COMMUNITY_GUIDELINES.md.
- O conteúdo +18 fica oculto, a menos que você entre na conta e ative “Mostrar conteúdo +18”.
- Tudo o que é compartilhado é revisado antes de ser publicado, e você pode denunciar o que violar as regras.

Em fontes e plugins, pressione Prévia para ver um texto de exemplo, ou para ler o que um plugin faz e o seu código, antes de instalar; nada é instalado. Pressione Traduzir para ler qualquer descrição no seu idioma. Ele usa um serviço online gratuito, só quando você pressiona, e «Ver original» volta ao texto original.

## Próximos passos

- [Tutorial para profissionais](TUTORIAL_PROS.pt.md): o que a exportação gera e onde colocar sua lógica.
- [Servidor MCP](MCP.md): peça alterações a uma IA em linguagem natural.
