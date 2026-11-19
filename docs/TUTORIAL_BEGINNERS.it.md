# Tutorial per principianti

🌐 [English](TUTORIAL_BEGINNERS.md) · [Español](TUTORIAL_NOVATOS.md) · [Português](TUTORIAL_BEGINNERS.pt.md) · [Français](TUTORIAL_BEGINNERS.fr.md) · [Deutsch](TUTORIAL_BEGINNERS.de.md) · [Italiano](TUTORIAL_BEGINNERS.it.md) · [Русский](TUTORIAL_BEGINNERS.ru.md) · [简体中文](TUTORIAL_BEGINNERS.zh.md) · [日本語](TUTORIAL_BEGINNERS.ja.md) · [한국어](TUTORIAL_BEGINNERS.ko.md)

La tua prima interfaccia con Lienzo UI e la tua prima esportazione in Unity. Non serve saper programmare.

## 1. Installa e apri

1. Scarica `LienzoUI-windows.zip` dalle release di GitHub o da itch.io.
2. Estrailo e apri `lienzo-ui.exe`.
3. La prima volta si apre il **tutorial** (questo stesso contenuto, dentro l'app). Al termine, o con «Salta e vai all'editor», si apre l'editor. Per rivederlo: pulsante **Tutorial** nella barra superiore o **F1**.

Per usare l'app in italiano: pulsante **Accessibilità** › **Lingua** › Italiano.

![Tutorial dentro l'app](imagenes/tutorial.png)

## 2. La schermata

| Zona | Cosa c'è |
|---|---|
| Barra superiore | File, Modifica, Visualizza, Area di disegno, **Prova**, nome del design, **Accessibilità**, **Controlla**, **Temi**, **Tutorial**, **Salva**, **Codice…**, **Esporta in Unity** |
| Barra sinistra | **Gerarchia** (livelli), **Elementi**, **Modelli**, **Testo**, **Immagini**, **Plugin**, **Community** |
| Centro | L'area di disegno: lo schermo del tuo gioco (1920 × 1080 per impostazione predefinita) |
| Destra | L'**Inspector**: proprietà della selezione |

![Editor](imagenes/editor.png)

## 3. Inizia da un modello

In **Modelli** scegli uno dei 12 layout: Menu principale, HUD di gioco, Finestra di dialogo, Impostazioni, Inventario, Menu di pausa, Fine partita, Selezione livello, Schermata di caricamento, Dialogo personaggio, Classifica o Selezione personaggio. Tutto ciò che contiene può essere modificato e Ctrl+Z annulla la modifica. Per iniziare da zero: «Vuoto».

## 4. Modifica

- Trascina gli elementi da **Elementi** sull'area di disegno. **Componenti pronti** contiene pulsanti, barre della salute e del mana, schede, badge e slot dell'inventario.
- **Testo** contiene stili di titolo ed elementi HUD: punteggio, timer, monete, danno, combo, livello e sottotitoli.
- Un clic seleziona; trascinare sposta; gli angoli ridimensionano; la maniglia superiore ruota.
- Doppio clic su un testo per scrivere.
- Nell'**Inspector** cambia colori, bordi, ombre, font e ancore.

| Scorciatoia | Azione |
|---|---|
| Ctrl+Z / Ctrl+Y | Annulla / ripeti |
| Ctrl+D / Canc | Duplica / elimina |
| Ctrl+G / Ctrl+Shift+G | Raggruppa / separa |
| Frecce (Shift) | Sposta di 1 px (10 px) |
| Ctrl+S | Salva |
| F5 | Prova |
| Ctrl+E | Esporta in Unity |
| F1 | Tutorial |

## 5. Prova

Premi **Prova** (F5): l'interfaccia diventa interattiva e puoi scegliere altri schermi (telefono, 4:3, ultrawide) per vedere come si adatta. Esc esce dalla modalità prova e il design torna com'era prima della prova.

## 6. Controlla

Premi **Controlla** nella barra superiore. Segnala testo troppo piccolo da leggere, contrasto basso, pulsanti troppo piccoli da toccare, elementi fuori dalla pagina, nomi ripetuti e immagini mancanti. Clicca su un avviso per selezionare quell'elemento e correggerlo.

## 7. Esporta in Unity

1. Premi **Esporta in Unity** (Ctrl+E). La prima volta scegli la cartella del tuo progetto (quella che contiene `Assets`).
2. Torna in Unity: il design appare come prefab in `Assets/EditorGUI/Prefabs`.
3. Trascinalo nella scena, oppure usa il menu **Tools › Lienzo UI › Import design into scene…**.

Ogni volta che esporti di nuovo, Unity aggiorna il prefab da solo.

## Prossimi passi

- [Tutorial per professionisti](TUTORIAL_PROS.it.md): cosa genera l'esportazione e dove mettere la tua logica.
- [Server MCP](MCP.md): chiedi modifiche a un'IA in linguaggio naturale.
