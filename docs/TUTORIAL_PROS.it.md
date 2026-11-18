# Tutorial per professionisti

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROFESIONALES.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

## Cosa genera l'esportazione

Tutto viene scritto in `Assets/EditorGUI` nel progetto Unity:

| File | Scopo |
|---|---|
| `Designs/<Design>.json` | Il design. L'importatore ricostruisce il prefab ogni volta che cambia. |
| `Editor/EditorGuiImporter.cs` | Costruisce il prefab con uGUI + TextMeshPro. Menu **Tools › Lienzo UI**. |
| `Prefabs/<Design>.prefab` | Pronto all'uso. |
| `UI Toolkit/<Design>.uxml` + `.uss` | Versione UI Toolkit (UIDocument, PanelSettings 1920×1080 Scale With Screen Size). |
| `Scripts/<Design>OnGUI.cs` | Versione IMGUI: aggiungilo a un GameObject. |
| `Scripts/<Design>ViewBase.cs` | Riferimenti tipizzati ed eventi (rigenerato). |
| `Scripts/<Design>View.cs` | La **tua** logica: creato una volta sola e mai sovrascritto. |
| `Sprites/`, `Fonts/` | Immagini e font usati dal design. |

`<Design>` è il titolo in PascalCase senza accenti (`Menú principal` → `MenuPrincipal`).

Se manca un'immagine, non viene scritto nulla: l'esportazione si ferma prima di toccare il progetto e ti indica quale file non è stato possibile leggere.

## Ancore e ridimensionamento

- Senza nulla selezionato, l'Inspector mostra il **Canvas Scaler**: risoluzione di riferimento, *match* e sfondo.
- Ogni elemento ha ancore, pivot e offset come un `RectTransform`. Quando cambia la risoluzione di riferimento, gli elementi seguono le proprie ancore.
- I modelli integrati sono progettati a 1920×1080; su pagine più piccole o verticali vengono ridotti in modo uniforme per adattarsi.
- Verifica con **Prova** (F5) su diversi schermi prima di esportare.

## Layout ed elenchi

Un contenitore può disporre i suoi figli in orizzontale, in verticale o in una griglia (come i *Layout Group* di Unity), con spaziatura e padding. Le ScrollView calcolano il proprio contenuto. L'ordine nella Gerarchia è l'ordine di disegno.

## Logica

- **Nell'editor**: in modalità Prova, ogni elemento può eseguire uno script Rhai al clic (`mostrar("Name")`, `ocultar`, `alternar`, `texto`, `color`, `mover`…). Utile per prototipare flussi. Le modifiche fatte durante la prova vengono scartate quando esci dalla modalità prova e non vengono mai salvate né esportate.
- **In Unity**: `<Design>View` eredita da `<Design>ViewBase`, con un campo per ogni pulsante, slider, toggle o campo di testo, e metodi `On<Element>Click` / `On<Element>Changed` da sovrascrivere.

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

Dai nomi univoci agli elementi interattivi: gli script e `GameObject.Find` trovano solo il primo risultato. **Controlla** segnala i nomi ripetuti.

## Controlla design

Il pulsante **Controlla** (e `--revisar` da riga di comando) verifica, alla risoluzione di riferimento:

| Controllo | Regola |
|---|---|
| Dimensione del testo | Almeno 18 px a 1080 p |
| Contrasto | WCAG: 4.5:1, oppure 3:1 per testo grande (24 px, o 18.7 px in grassetto); i controlli disattivati sono esenti |
| Dimensione touch | Pulsanti, toggle, campi di testo e menu a tendina di almeno 64 px sul lato corto a 1080 p |
| Pagina | Elementi completamente o in parte fuori dalla pagina |
| Nomi | Nomi ripetuti su elementi interattivi |
| Immagini | File immagine che non esistono più |

Il contrasto inferiore a 2:1, gli elementi completamente fuori dalla pagina e le immagini mancanti sono errori; il resto sono avvisi.

## Riga di comando

```bash
lienzo-ui --plantillas                                   # elenca i modelli (0-12)
lienzo-ui --plantilla 4 --guardar inventario.editorgui.json
lienzo-ui --abrir inventario.editorgui.json --revisar    # codice di uscita 1 se ci sono errori
lienzo-ui --abrir inventario.editorgui.json --exportar "C:/MyGame"   # senza finestra
lienzo-ui --abrir inventario.editorgui.json --screenshot capture.png
lienzo-ui --probar 1080x2400                             # apre in modalità Prova con quello schermo
lienzo-ui --tutorial [--profesional]                     # forza il tutorial
lienzo-ui --sin-tutorial                                 # non lo apre
```

`--guardar`, `--exportar` e `--revisar` non aprono una finestra: usali per script, CI e il [server MCP](MCP.md) (strumento `revisar_diseno`).

## Codice sorgente

Rust + [egui](https://github.com/emilk/egui):

| File | Contenuto |
|---|---|
| `src/model.rs` | Dati: `Doc`, `Element`, `Kind`, ancore, layout, modelli |
| `src/canvas.rs` | Area di disegno: selezione, maniglie, guide, zoom |
| `src/panels.rs` | Barra superiore, barra laterale, Inspector, finestre |
| `src/app.rs` | Stato, annulla, comandi, scorciatoie |
| `src/render.rs` | Disegno di elementi e font |
| `src/revision.rs` | Controlli di Controlla design |
| `src/export/` | Esportatori: `ugui.rs`, `uitk.rs`, `imgui.rs`, `view.rs` e l'importatore di Unity |
| `src/tutorial.rs` | Tutorial integrato (pagine in spagnolo e inglese) |
| `src/script.rs` | Script Rhai per la modalità Prova |

Per estenderlo: [EXTENDER.md](EXTENDER.md).
