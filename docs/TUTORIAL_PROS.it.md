# Tutorial per professionisti

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROS.es.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

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

- **Nell'editor**: in modalità Prova, ogni elemento può eseguire uno script Rhai al clic (`show("Name")`, `hide`, `toggle`, `set_text`, `set_color`, `move_by`…). Utile per prototipare flussi. Le modifiche fatte durante la prova vengono scartate quando esci dalla modalità prova e non vengono mai salvate né esportate.
- **In Unity**: `<Design>View` eredita da `<Design>ViewBase`, con un campo per ogni pulsante, slider, toggle o campo di testo, e metodi `On<Element>Click` / `On<Element>Changed` da sovrascrivere.

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

Dai nomi univoci agli elementi interattivi: gli script e `GameObject.Find` trovano solo il primo risultato. **Controlla** segnala i nomi ripetuti.

## Controlla design

Il pulsante **Controlla** (e `--review` da riga di comando) verifica, alla risoluzione di riferimento:

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
lienzo-ui --templates                                   # elenca i modelli (0-12)
lienzo-ui --template 4 --save inventory.editorgui.json
lienzo-ui --open inventory.editorgui.json --review    # codice di uscita 1 se ci sono errori
lienzo-ui --open inventory.editorgui.json --export "C:/MyGame"   # senza finestra
lienzo-ui --open inventory.editorgui.json --screenshot capture.png
lienzo-ui --test 1080x2400                             # apre in modalità Prova con quello schermo
lienzo-ui --tutorial [--pro]                     # forza il tutorial
lienzo-ui --no-tutorial                                 # non lo apre
```

`--save`, `--export` e `--review` non aprono una finestra: usali per script, CI e il [server MCP](MCP.md) (strumento `review_design`).

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

Per estenderlo: [EXTENDING.md](EXTENDING.md).

## Layout verticale nel dettaglio

Il layout verticale è una seconda vista dello stesso design (docs/FORMAT.md › PortraitLayout):

- Ha una propria risoluzione di riferimento, «Dimensione verticale» (1080 × 1920 per impostazione predefinita).
- Gli elementi senza eccezione mantengono il RectTransform orizzontale e vengono rivalutati rispetto alla pagina verticale: uno sfondo a tutta pagina si allunga, quelli ancorati a destra seguono il bordo destro.
- Viene salvato solo ciò che modifichi nella vista verticale, come eccezioni (posizione, dimensione, rotazione, ancoraggi, visibilità, dimensione del carattere, layout). Clic destro › «Torna a orizzontale» ne rimuove una.
- Esportazione: il JSON contiene le eccezioni e lo script generato LienzoOrientationLayout le applica quando Screen.width < Screen.height, e di nuovo quando la dimensione cambia.
- «Rimuovi layout verticale» elimina tutte le eccezioni.

## 9-slice e pacchetto incluso

Per cornici, pulsanti e pannelli che si allungano:

- Seleziona un'Immagine e imposta «9-slice» in «Impostazioni immagine (Unity)»: il bordo in pixel dell'immagine originale (sinistra, basso, destra, alto), come nello Sprite Editor di Unity. L'esportazione genera uno Sprite 9-slice.
- «Pixel per unità»: 100 mantiene i bordi alla loro dimensione in pixel, 200 li dimezza.
- Il pacchetto incluso (assets/pack/manifest.json, docs/CONTENT.md) imposta già il taglio e i pixel per unità: cornici 512×512 con taglio 96, pulsanti 512×192.
- Usare un elemento lo copia nella tua libreria come pack_<id>.<ext> (nascosto in «Le tue immagini»); l'esportatore lo copia negli Sprite di Unity.
- I modelli HUD RPG pixel art e Menu principale pixel art usano il pacchetto pixel. Attiva «Pixel art» per il filtro Point.

## Controllo e correzioni rapide

La finestra Controlla elenca errori e avvisi; fai clic su uno per selezionarne l'elemento. Sei problemi hanno un pulsante «Correggi» (un passaggio di annullamento):

- Testo sotto la dimensione minima: portato al minimo.
- Controllo troppo piccolo da toccare: ingrandito attorno al centro.
- Elemento in parte fuori dalla pagina: spostato dentro, se ci sta.
- Nome ripetuto: si aggiunge _2, _3…
- Immagine mancante: il riferimento viene rimosso.
- Basso contrasto: il colore del testo viene scurito o schiarito.

«Correggi tutti i problemi sicuri» li applica in alcuni passaggi. Problemi come un font senza certi caratteri non hanno correzione automatica. Le correzioni funzionano anche nel layout verticale. Da riga di comando: lienzo-ui --open file --review (codice 1 se ci sono errori).

## Importare da Unity: prefab e CLI

File › Importa da Unity… legge un .prefab uGUI (il YAML di testo di Unity) o un Designs/*.json esportato da Lienzo UI e crea un design normale.

- Ciò che non si è potuto convertire compare come avvisi.
- Se rileva la cartella del progetto Unity, le immagini vengono risolte rispetto ad essa durante la sessione.
- Senza finestra: lienzo-ui --import-unity Menu.prefab --save Menu.editorgui.json (anche con --review e --export).
- Progetto intero: lienzo-ui --import-unity-project MioProgetto --out Importati converte tutti i prefab di interfaccia. Non sovrascrive mai (aggiunge _2) e salta i prefab senza interfaccia.
- Dettagli in docs/UNITY_IMPORT.md.

## Community: plugin, font e regole

Nella scheda Community, Installa aggiunge contenuti condivisi con un clic: plugin, font, immagini, modelli, elementi e temi.

- Un plugin della community va in una cartella chiamata community-<nome>-<hash>. Viene scritto prima in una cartella temporanea, quindi un'installazione interrotta non lascia mai un plugin a metà. Compare nella scheda Plugin.
- I suoi script sono gli stessi Rhai: isolati, senza accesso a file né rete e con limiti di operazioni, tempo e memoria (docs/PLUGINS.md).
- Un font installato viene aggiunto alla scheda Testo ed esportato in Unity con il design.
- I download vengono verificati con l'impronta SHA-256 pubblicata; se non corrisponde, non si installa nulla.
- Per pubblicare accedi (GitHub o Google), scegli un nome pubblico e accetti le regole. Tutto viene controllato prima e i contenuti +18 vanno segnalati. Vedi docs/TERMS.md, docs/COMMUNITY_GUIDELINES.md e docs/PRIVACY.md.

## Salvataggio automatico, recupero e annulla

- Circa ogni minuto, mentre modifichi, il design completo (layout verticale incluso) viene scritto nella cartella di recupero dell'app. Mai in modalità Prova. Si conservano al massimo 10 recuperi.
- All'avvio viene offerto un recupero più recente del file di progetto: Ripristina o Scarta. Dopo un normale Salva non serve più.
- Si può disattivare in Accessibilità › Salvataggio automatico.
- La cronologia di annullamento è limitata a 200 passaggi e a circa 64 MB. Quando i passaggi più vecchi vengono scartati, la barra di stato mostra «Cronologia ridotta».
- Ogni comando di plugin e ogni Correggi della finestra Controlla è un solo passaggio di annullamento.
