# Tutorial für Einsteiger

🌐 [English](TUTORIAL_BEGINNERS.md) · [Español](TUTORIAL_BEGINNERS.es.md) · [Português](TUTORIAL_BEGINNERS.pt.md) · [Français](TUTORIAL_BEGINNERS.fr.md) · [Deutsch](TUTORIAL_BEGINNERS.de.md) · [Italiano](TUTORIAL_BEGINNERS.it.md) · [Русский](TUTORIAL_BEGINNERS.ru.md) · [简体中文](TUTORIAL_BEGINNERS.zh.md) · [日本語](TUTORIAL_BEGINNERS.ja.md) · [한국어](TUTORIAL_BEGINNERS.ko.md)

Deine erste Oberfläche mit Lienzo UI und dein erster Export nach Unity. Du brauchst keine Programmierkenntnisse.

## 1. Installieren und öffnen

1. Lade `LienzoUI-windows.zip` von den GitHub-Releases oder von itch.io herunter.
2. Entpacke es und öffne `lienzo-ui.exe`.
3. Beim ersten Start öffnet sich das **Tutorial** (derselbe Inhalt, direkt in der App). Wenn du es beendest oder «Überspringen und zum Editor gehen» wählst, öffnet sich der Editor. Um es erneut zu sehen: die Schaltfläche **Tutorial** in der oberen Leiste oder **F1**.

Um die App auf Englisch zu verwenden: Schaltfläche **Barrierefreiheit** › **Sprache** › English.

![Tutorial in der App](images/tutorial.png)

## 2. Der Bildschirm

| Bereich | Was dort ist |
|---|---|
| Obere Leiste | Datei, Bearbeiten, Ansicht, Arbeitsfläche, **Testen**, Name des Entwurfs, **Barrierefreiheit**, **Prüfen**, **Themes**, **Tutorial**, **Speichern**, **Code…**, **In Unity exportieren** |
| Linke Leiste | **Hierarchie** (Ebenen), **Elemente**, **Vorlagen**, **Text**, **Bilder**, **Plugins**, **Community** |
| Mitte | Die Arbeitsfläche: der Bildschirm deines Spiels (standardmäßig 1920 × 1080) |
| Rechts | Der **Inspektor**: Eigenschaften der Auswahl |

![Editor](images/editor.png)

## 3. Mit einer Vorlage beginnen

Unter **Vorlagen** wählst du eines von 15 Layouts: Hauptmenü, Spiel-HUD, Dialogfenster, Einstellungen, Inventar, Pausenmenü, Spielende, Levelauswahl, Ladebildschirm, Charakterdialog, Rangliste oder Charakterauswahl. Alles darin kannst du ändern, und Strg+Z macht die Änderung rückgängig. Um von vorne zu beginnen: «Leer». Zwei davon sind im Pixel-Art-Stil: Pixel-RPG-HUD und Pixel-Hauptmenü.

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
3. Zieh es in die Szene, oder nutze das Menü **Tools › Lienzo UI › Import design into scene…**.

Bei jedem erneuten Export aktualisiert Unity das Prefab von selbst.

## Mitgelieferte Bilder

Der Tab Bilder in der linken Leiste enthält fertige Grafiken, ganz ohne eigene Dateien:

- Die Chips oben filtern nach Deine Bilder, Hintergründe, Rahmen, Buttons, Symbole, Pixel-Art und Farbverläufe. Die Suche findet nach Name oder Tag.
- Klicke auf einen Eintrag, um ihn hinzuzufügen, oder ziehe ihn auf die Arbeitsfläche. Rahmen und Buttons haben schon ihren 9-Slice-Rand und lassen sich dehnen, ohne dass die Ecken verzerrt werden. Hintergründe landen hinter allem anderen.
- Das Bild wird beim ersten Benutzen in deine Bibliothek kopiert und zusammen mit dem Entwurf nach Unity exportiert.
- Bei Pixel-Art-Sprites aktivierst du „Pixel-Art“ im Inspektor der Arbeitsfläche (nichts ausgewählt), damit sie scharf bleiben.

## Hochformat-Bildschirme

Smartphones hält man oft hochkant. Lienzo UI führt dafür ein zweites Layout, ohne das erste anzutasten:

- Wechsle die Ansicht mit den Schaltflächen Landschaft / Porträt in der oberen Leiste.
- Öffne bei nichts Ausgewähltem im Inspektor „Hochformat-Layout“ und klicke auf „Hochformat-Layout erstellen“. Die Elemente starten dort, wo sie im Querformat sind.
- In der Hochformat-Ansicht ändert alles, was du verschiebst oder skalierst, nur das Hochformat-Layout. Rechtsklick auf ein Element › „Auf Querformat zurücksetzen“ macht eine Änderung rückgängig.
- Der Export enthält beide Layouts: In Unity wechselt das Prefab von selbst zum Hochformat, wenn der Bildschirm höher als breit ist.

## Aus Unity importieren

Du hast schon eine Oberfläche in Unity? Hole sie in Lienzo UI:

- Datei › Aus Unity importieren…
- Wähle ein uGUI-Prefab (.prefab) oder einen Entwurf, den Lienzo UI früher exportiert hat (Assets/EditorGUI/Designs/….json).
- Er öffnet sich als neuer, ungespeicherter Entwurf: Speichere ihn als Projekt, um ihn zu behalten.
- Konnte etwas nicht exakt umgesetzt werden, listet ein Fenster die Warnungen auf, damit nichts stillschweigend verloren geht.
- Findet Lienzo UI deinen Unity-Projektordner, werden die Bilder des Entwurfs automatisch gefunden.

## Verliere deine Arbeit nicht

- Automatisch speichern: Etwa jede Minute legt Lienzo UI eine Wiederherstellungskopie an. Du kannst das unter Barrierefreiheit › Automatisch speichern ausschalten.
- Wird die App unerwartet beendet, fragt sie beim nächsten Start „Nicht gespeicherte Arbeit wiederherstellen…?“. Mit Wiederherstellen holst du sie zurück, mit Verwerfen löschst du sie. Wiederhergestellte Arbeit ist noch keine gespeicherte Datei: Drücke Strg+S, um sie zu behalten.
- Strg+S speichert dein Projekt weiterhin wie gewohnt.
- Rückgängig (Strg+Z) merkt sich bis zu 200 Schritte (und hat eine Speichergrenze). Werden die ältesten verworfen, zeigt die Statusleiste „Verlauf gekürzt“.

## Community

Der Tab Community in der linken Leiste ist ein gemeinsamer Katalog mit Plugins, Bildern, Schriften, Vorlagen, Elementen und Themes:

- Stöbern und Installieren geht ohne Konto: suchen und auf Installieren klicken.
- Zum Hochladen, Melden oder Herzen-Vergeben meldest du dich mit GitHub oder Google an und wählst einen öffentlichen Namen. Keine Passwörter: Die App speichert nur diesen Namen.
- Beim Anlegen deines Profils akzeptierst du die Regeln: keine Malware, kein Hass, kein Plagiat und keine nicht gekennzeichneten Inhalte ab 18. Den vollständigen Text findest du in docs/TERMS.md und docs/COMMUNITY_GUIDELINES.md.
- Inhalte ab 18 sind ausgeblendet, außer du meldest dich an und aktivierst „Inhalte ab 18 anzeigen“.
- Alles Geteilte wird vor der Veröffentlichung geprüft, und du kannst melden, was gegen die Regeln verstößt.

## Nächste Schritte

- [Tutorial für Profis](TUTORIAL_PROS.de.md): was der Export erzeugt und wo du deine Logik ablegst.
- [MCP-Server](MCP.md): bitte eine KI in natürlicher Sprache um Änderungen.
