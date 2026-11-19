# Tutorial für Profis

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROS.es.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

## Was der Export erzeugt

Alles wird in `Assets/EditorGUI` im Unity-Projekt geschrieben:

| Datei | Zweck |
|---|---|
| `Designs/<Design>.json` | Der Entwurf. Der Importer baut das Prefab neu, sobald er sich ändert. |
| `Editor/EditorGuiImporter.cs` | Baut das Prefab mit uGUI + TextMeshPro. Menü **Tools › Lienzo UI**. |
| `Prefabs/<Design>.prefab` | Sofort einsatzbereit. |
| `UI Toolkit/<Design>.uxml` + `.uss` | UI-Toolkit-Version (UIDocument, PanelSettings 1920×1080 Scale With Screen Size). |
| `Scripts/<Design>OnGUI.cs` | IMGUI-Version: an ein GameObject anhängen. |
| `Scripts/<Design>ViewBase.cs` | Typisierte Referenzen und Ereignisse (wird neu erzeugt). |
| `Scripts/<Design>View.cs` | **Deine** Logik: wird einmal erstellt und nie überschrieben. |
| `Sprites/`, `Fonts/` | Bilder und Schriften des Entwurfs. |

`<Design>` ist der Titel in PascalCase ohne Akzente (`Menú principal` → `MenuPrincipal`).

Wenn ein Bild fehlt, wird nichts geschrieben: Der Export stoppt, bevor er das Projekt berührt, und sagt dir, welche Datei er nicht lesen konnte.

## Anker und Skalierung

- Ohne Auswahl zeigt der Inspektor den **Canvas Scaler**: Referenzauflösung, *match* und Hintergrund.
- Jedes Element hat Anker, Pivot und Versätze wie ein `RectTransform`. Wenn sich die Referenzauflösung ändert, folgen die Elemente ihren Ankern.
- Eingebaute Vorlagen sind für 1920×1080 entworfen; auf kleineren oder Hochformat-Seiten werden sie gleichmäßig verkleinert, damit sie passen.
- Prüfe mit **Testen** (F5) auf mehreren Bildschirmen, bevor du exportierst.

## Layouts und Listen

Ein Container kann seine untergeordneten Elemente horizontal, vertikal oder im Raster anordnen (wie Unitys *Layout Groups*), mit Abstand und Innenabstand. ScrollViews berechnen ihren Inhalt. Die Reihenfolge in der Hierarchie ist die Zeichenreihenfolge.

## Logik

- **Im Editor**: Im Testmodus kann jedes Element beim Klick ein Rhai-Skript ausführen (`show("Name")`, `hide`, `toggle`, `set_text`, `set_color`, `move_by`…). Das ist nützlich, um Abläufe zu prototypen. Änderungen während des Tests werden verworfen, sobald du den Testmodus verlässt, und nie gespeichert oder exportiert.
- **In Unity**: `<Design>View` erbt von `<Design>ViewBase`, mit einem Feld pro Button, Schieberegler, Schalter oder Eingabefeld sowie den Methoden `On<Element>Click` / `On<Element>Changed`, die du überschreiben kannst.

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

Gib interaktiven Elementen eindeutige Namen: Skripte und `GameObject.Find` finden nur den ersten Treffer. **Prüfen** warnt vor doppelten Namen.

## Entwurf prüfen

Der Button **Prüfen** (und `--review` in der Kommandozeile) prüft bei der Referenzauflösung:

| Prüfung | Regel |
|---|---|
| Textgröße | Mindestens 18 px bei 1080p |
| Kontrast | WCAG: 4,5:1, oder 3:1 für große Schrift (24 px, bzw. 18,7 px fett); deaktivierte Steuerelemente sind ausgenommen |
| Berührungsgröße | Buttons, Schalter, Eingabefelder und Dropdowns mindestens 64 px an der kurzen Seite bei 1080p |
| Seite | Elemente ganz oder teilweise außerhalb der Seite |
| Namen | Doppelte Namen bei interaktiven Elementen |
| Bilder | Bilddateien, die nicht mehr existieren |

Kontrast unter 2:1, Elemente, die ganz außerhalb der Seite liegen, und fehlende Bilder sind Fehler; der Rest sind Warnungen.

## Kommandozeile

```bash
lienzo-ui --templates                                   # listet die Vorlagen (0-12)
lienzo-ui --template 4 --save inventory.editorgui.json
lienzo-ui --open inventory.editorgui.json --review    # Exitcode 1, wenn es Fehler gibt
lienzo-ui --open inventory.editorgui.json --export "C:/MyGame"   # ohne Fenster
lienzo-ui --open inventory.editorgui.json --screenshot capture.png
lienzo-ui --test 1080x2400                             # öffnet im Testmodus mit diesem Bildschirm
lienzo-ui --tutorial [--pro]                     # erzwingt das Tutorial
lienzo-ui --no-tutorial                                 # öffnet es nicht
```

`--save`, `--export` und `--review` öffnen kein Fenster: Verwende sie für Skripte, CI und den [MCP-Server](MCP.md) (Werkzeug `review_design`).

## Quellcode

Rust + [egui](https://github.com/emilk/egui):

| Datei | Inhalt |
|---|---|
| `src/model.rs` | Daten: `Doc`, `Element`, `Kind`, Anker, Layouts, Vorlagen |
| `src/canvas.rs` | Arbeitsfläche: Auswahl, Griffe, Hilfslinien, Zoom |
| `src/panels.rs` | Obere Leiste, Seitenleiste, Inspektor, Fenster |
| `src/app.rs` | Zustand, Rückgängig, Befehle, Tastenkürzel |
| `src/render.rs` | Zeichnen von Elementen und Schriften |
| `src/revision.rs` | Prüfungen des Entwurfs |
| `src/export/` | Exporter: `ugui.rs`, `uitk.rs`, `imgui.rs`, `view.rs` und der Unity-Importer |
| `src/tutorial.rs` | Eingebautes Tutorial (Seiten auf Spanisch und Englisch) |
| `src/script.rs` | Rhai-Skripte für den Testmodus |

Um es zu erweitern: [EXTENDING.md](EXTENDING.md).

## Hochformat-Layout im Detail

Das Hochformat-Layout ist eine zweite Ansicht desselben Entwurfs (docs/FORMAT.md › PortraitLayout):

- Es hat eine eigene Referenzauflösung, „Hochformat-Größe“ (standardmäßig 1080 × 1920).
- Elemente ohne Override behalten ihr Quer-RectTransform und werden gegen die Hochformat-Seite neu ausgewertet: Ein vollflächiger Hintergrund dehnt sich, rechts verankerte Elemente folgen dem rechten Rand.
- Gespeichert wird nur, was du in der Hochformat-Ansicht änderst, als Overrides (Position, Größe, Rotation, Anker, Sichtbarkeit, Schriftgröße, Layout). Rechtsklick › „Auf Querformat zurücksetzen“ entfernt einen.
- Export: Das JSON enthält die Overrides, und das erzeugte Skript LienzoOrientationLayout wendet sie an, wenn Screen.width < Screen.height ist, und erneut bei Größenänderung.
- „Hochformat-Layout entfernen“ löscht alle Overrides.

## 9-Slice und das mitgelieferte Paket

Für Rahmen, Buttons und Panels, die sich dehnen:

- Wähle ein Bild und setze „9-Slice“ unter „Bildeinstellungen (Unity)“: den Rand in Pixeln des Originalbilds (links, unten, rechts, oben), wie im Sprite Editor von Unity. Der Export erzeugt ein 9-Slice-Sprite.
- „Pixel pro Einheit“: 100 hält die Ränder in ihrer Pixelgröße, 200 halbiert sie.
- Das mitgelieferte Paket (assets/pack/manifest.json, docs/CONTENT.md) legt Schnitt und Pixel pro Einheit schon fest: Rahmen 512×512 mit Schnitt 96, Buttons 512×192.
- Wer einen Eintrag benutzt, kopiert ihn als pack_<id>.<ext> in die Bibliothek (in „Deine Bilder“ ausgeblendet); der Exporter kopiert ihn in die Unity-Sprites.
- Die Vorlagen Pixel-RPG-HUD und Pixel-Hauptmenü nutzen das Pixel-Paket. Aktiviere „Pixel-Art“ für die Point-Filterung.

## Prüfen und schnelle Korrekturen

Das Fenster Prüfen listet Fehler und Warnungen; ein Klick wählt das zugehörige Element aus. Sechs Probleme haben eine Schaltfläche „Beheben“ (ein Rückgängig-Schritt):

- Text unter der Mindestgröße: wird auf das Minimum erhöht.
- Zu kleines Bedienelement zum Antippen: wird um seine Mitte vergrößert.
- Element teilweise außerhalb der Seite: wird hineinverschoben, falls es passt.
- Doppelter Name: _2, _3… wird angehängt.
- Fehlendes Bild: Der Verweis wird entfernt.
- Geringer Kontrast: Die Textfarbe wird abgedunkelt oder aufgehellt.

„Alle sicheren Probleme beheben“ wendet sie in mehreren Durchgängen an. Probleme wie eine Schrift mit fehlenden Zeichen haben keine automatische Korrektur. Die Korrekturen funktionieren auch im Hochformat-Layout. Per Kommandozeile: lienzo-ui --open Datei --review (Exitcode 1 bei Fehlern).

## Aus Unity importieren: Prefabs und CLI

Datei › Aus Unity importieren… liest ein uGUI-.prefab (Unitys Text-YAML) oder eine von Lienzo UI exportierte Designs/*.json und erzeugt einen normalen Entwurf.

- Was nicht abgebildet werden konnte, steht als Warnung in einer Liste.
- Wird der Unity-Projektordner erkannt, werden Bilder in dieser Sitzung relativ dazu aufgelöst.
- Ohne Fenster: lienzo-ui --import-unity Menue.prefab --save Menue.editorgui.json (auch mit --review und --export).
- Ganzes Projekt: lienzo-ui --import-unity-project MeinProjekt --out Importiert wandelt jedes UI-Prefab um. Es überschreibt nie (hängt _2 an) und überspringt Prefabs ohne UI.
- Details in docs/UNITY_IMPORT.md.

## Community: Plugins, Schriften und Regeln

Im Tab Community fügt Installieren geteilte Inhalte mit einem Klick hinzu: Plugins, Schriften, Bilder, Vorlagen, Elemente und Themes.

- Ein Community-Plugin landet in einem Ordner namens community-<Name>-<Hash>. Es wird zuerst in einen temporären Ordner geschrieben, sodass eine unterbrochene Installation nie ein halbes Plugin hinterlässt. Es erscheint im Tab Plugins.
- Seine Skripte sind dieselben Rhai-Skripte: abgeschottet, ohne Datei- oder Netzwerkzugriff und mit Grenzen für Operationen, Zeit und Speicher (docs/PLUGINS.md).
- Eine installierte Schrift erscheint im Tab Text und wird mit dem Entwurf nach Unity exportiert.
- Downloads werden mit dem veröffentlichten SHA-256-Fingerabdruck geprüft; stimmt er nicht, wird nichts installiert.
- Zum Veröffentlichen meldest du dich an (GitHub oder Google), wählst einen öffentlichen Namen und akzeptierst die Regeln. Alles wird vorher geprüft, und Inhalte ab 18 müssen gekennzeichnet sein. Siehe docs/TERMS.md, docs/COMMUNITY_GUIDELINES.md und docs/PRIVACY.md.

## Automatisch speichern, Wiederherstellung und Rückgängig

- Etwa jede Minute, während du bearbeitest, wird der vollständige Entwurf (samt Hochformat-Layout) in den Wiederherstellungsordner der App geschrieben. Nie im Testmodus. Höchstens 10 Wiederherstellungen bleiben erhalten.
- Beim Start wird eine Wiederherstellung angeboten, die neuer als die Projektdatei ist: Wiederherstellen oder Verwerfen. Nach einem normalen Speichern wird sie nicht mehr gebraucht.
- Sie lässt sich unter Barrierefreiheit › Automatisch speichern ausschalten.
- Der Rückgängig-Verlauf ist auf 200 Schritte und etwa 64 MB begrenzt. Werden die ältesten Schritte verworfen, zeigt die Statusleiste „Verlauf gekürzt“.
- Jeder Plugin-Befehl und jedes Beheben aus dem Fenster Prüfen ist ein einziger Rückgängig-Schritt.
