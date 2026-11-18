# Tutorial für Einsteiger

🌐 [English](TUTORIAL_BEGINNERS.md) · [Español](TUTORIAL_NOVATOS.md) · [Português](TUTORIAL_BEGINNERS.pt.md) · [Français](TUTORIAL_BEGINNERS.fr.md) · [Deutsch](TUTORIAL_BEGINNERS.de.md) · [Italiano](TUTORIAL_BEGINNERS.it.md) · [Русский](TUTORIAL_BEGINNERS.ru.md)

Deine erste Oberfläche mit Lienzo UI und dein erster Export nach Unity. Du brauchst keine Programmierkenntnisse.

## 1. Installieren und öffnen

1. Lade `LienzoUI-windows.zip` von den GitHub-Releases oder von itch.io herunter.
2. Entpacke es und öffne `lienzo-ui.exe`.
3. Beim ersten Start öffnet sich das **Tutorial** (derselbe Inhalt, direkt in der App). Wenn du es beendest oder «Überspringen und zum Editor gehen» wählst, öffnet sich der Editor. Um es erneut zu sehen: die Schaltfläche **Tutorial** in der oberen Leiste oder **F1**.

Um die App auf Englisch zu verwenden: Schaltfläche **Barrierefreiheit** › **Sprache** › English.

![Tutorial in der App](imagenes/tutorial.png)

## 2. Der Bildschirm

| Bereich | Was dort ist |
|---|---|
| Obere Leiste | Datei, Bearbeiten, Ansicht, Arbeitsfläche, **Testen**, Name des Entwurfs, **Barrierefreiheit**, **Prüfen**, **Themes**, **Tutorial**, **Speichern**, **Code…**, **In Unity exportieren** |
| Linke Leiste | **Hierarchie** (Ebenen), **Elemente**, **Vorlagen**, **Text**, **Bilder**, **Plugins**, **Community** |
| Mitte | Die Arbeitsfläche: der Bildschirm deines Spiels (standardmäßig 1920 × 1080) |
| Rechts | Der **Inspektor**: Eigenschaften der Auswahl |

![Editor](imagenes/editor.png)

## 3. Mit einer Vorlage beginnen

Unter **Vorlagen** wählst du eines von 12 Layouts: Hauptmenü, Spiel-HUD, Dialogfenster, Einstellungen, Inventar, Pausenmenü, Spielende, Levelauswahl, Ladebildschirm, Charakterdialog, Rangliste oder Charakterauswahl. Alles darin kannst du ändern, und Strg+Z macht die Änderung rückgängig. Um von vorne zu beginnen: «Leer».

## 4. Bearbeiten

- Ziehe Elemente aus **Elemente** auf die Arbeitsfläche. **Fertige Komponenten** enthält Buttons, Lebens- und Manabalken, Karten, Abzeichen und Inventarfelder.
- **Text** enthält Überschriftenstile und HUD-Teile: Punkte, Timer, Münzen, Schadenszahlen, Combo, Levelmarkierung und Untertitel.
- Klick wählt aus; Ziehen verschiebt; die Ecken ändern die Größe; der obere Griff dreht.
- Doppelklick auf einen Text, um ihn zu schreiben.
- Im **Inspektor** änderst du Farben, Rahmen, Schatten, Schriften und Anker.

| Tastenkürzel | Aktion |
|---|---|
| Strg+Z / Strg+Y | Rückgängig / Wiederholen |
| Strg+D / Entf | Duplizieren / Löschen |
| Strg+G / Strg+Umschalt+G | Gruppieren / Gruppierung aufheben |
| Pfeile (Umschalt) | 1 px verschieben (10 px) |
| Strg+S | Speichern |
| F5 | Testen |
| Strg+E | In Unity exportieren |
| F1 | Tutorial |

## 5. Testen

Drücke **Testen** (F5): Die Oberfläche wird interaktiv, und du kannst andere Bildschirme wählen (Handy, 4:3, Ultrawide 21:9), um zu sehen, wie sie sich anpasst. Esc verlässt den Testmodus, und der Entwurf kehrt in den Zustand vor dem Test zurück.

## 6. Prüfen

Drücke **Prüfen** in der oberen Leiste. Es warnt vor zu kleiner Schrift, schwachem Kontrast, Buttons, die zum Antippen zu klein sind, Elementen außerhalb der Seite, doppelten Namen und fehlenden Bildern. Klicke auf eine Warnung, um dieses Element auszuwählen und zu korrigieren.

## 7. In Unity exportieren

1. Drücke **In Unity exportieren** (Strg+E). Beim ersten Mal wähle deinen Projektordner (den, der `Assets` enthält).
2. Geh zurück zu Unity: Der Entwurf erscheint als Prefab in `Assets/EditorGUI/Prefabs`.
3. Zieh es in die Szene, oder nutze das Menü **Tools › Lienzo UI › Importar diseño en la escena** (Entwurf in die Szene importieren; das Menü in Unity ist auf Spanisch).

Bei jedem erneuten Export aktualisiert Unity das Prefab von selbst.

## Nächste Schritte

- [Tutorial für Profis](TUTORIAL_PROS.de.md): was der Export erzeugt und wo du deine Logik ablegst.
- [MCP-Server](MCP.md): bitte eine KI in natürlicher Sprache um Änderungen.
