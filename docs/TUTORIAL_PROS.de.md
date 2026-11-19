# Tutorial für Profis

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROFESIONALES.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

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

Um es zu erweitern: [EXTENDER.md](EXTENDER.md).
