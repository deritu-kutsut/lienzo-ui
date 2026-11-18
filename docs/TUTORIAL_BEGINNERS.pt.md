# Tutorial para iniciantes

🌐 [English](TUTORIAL_BEGINNERS.md) · [Español](TUTORIAL_NOVATOS.md) · [Português](TUTORIAL_BEGINNERS.pt.md) · [Français](TUTORIAL_BEGINNERS.fr.md) · [Deutsch](TUTORIAL_BEGINNERS.de.md) · [Italiano](TUTORIAL_BEGINNERS.it.md) · [Русский](TUTORIAL_BEGINNERS.ru.md) · [简体中文](TUTORIAL_BEGINNERS.zh.md) · [日本語](TUTORIAL_BEGINNERS.ja.md) · [한국어](TUTORIAL_BEGINNERS.ko.md)

Sua primeira interface com o Lienzo UI e sua primeira exportação para o Unity. Não precisa saber programar.

## 1. Instalar e abrir

1. Baixe `LienzoUI-windows.zip` nas releases do GitHub ou no itch.io.
2. Descompacte o arquivo e abra `lienzo-ui.exe`.
3. Na primeira vez, o **tutorial** se abre (este mesmo conteúdo, dentro do app). Ao terminá-lo, ou com «Pular e ir ao editor», o editor se abre. Para vê-lo novamente: botão **Tutorial** na barra superior ou **F1**.

Para usar o app em português: botão **Acessibilidade** › **Idioma** › Português.

![Tutorial dentro do app](imagenes/tutorial.png)

## 2. A tela

| Área | O que há |
|---|---|
| Barra superior | Arquivo, Editar, Ver, Tela, **Testar**, nome do design, **Acessibilidade**, **Revisar**, **Temas**, **Tutorial**, **Salvar**, **Código…**, **Exportar para Unity** |
| Barra esquerda | **Hierarquia** (camadas), **Elementos**, **Modelos**, **Texto**, **Imagens**, **Plugins**, **Comunidade** |
| Centro | A tela: a tela do seu jogo (1920 × 1080 por padrão) |
| Direita | O **Inspetor**: propriedades da seleção |

![Editor](imagenes/editor.png)

## 3. Comece com um modelo

Em **Modelos**, escolha um dos 12 layouts: Menu principal, HUD de jogo, Janela de diálogo, Configurações, Inventário, Menu de pausa, Fim de jogo, Seleção de fase, Tela de carregamento, Diálogo de personagem, Classificação ou Seleção de personagem. Tudo o que ele traz pode ser alterado, e Ctrl+Z desfaz a alteração. Para começar do zero: «Em branco».

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
3. Arraste-o para a cena ou use o menu **Tools › Lienzo UI › Importar diseño en la escena** (o menu no Unity está em espanhol: «Importar design na cena»).

Sempre que você exportar novamente, o Unity atualiza o prefab sozinho.

## Próximos passos

- [Tutorial para profissionais](TUTORIAL_PROS.pt.md): o que a exportação gera e onde colocar sua lógica.
- [Servidor MCP](MCP.md): peça alterações a uma IA em linguagem natural.
