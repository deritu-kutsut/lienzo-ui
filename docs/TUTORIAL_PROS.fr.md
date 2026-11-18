# Tutoriel pro

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROFESIONALES.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

## Ce que génère l'export

Tout est écrit dans `Assets/EditorGUI` du projet Unity :

| Fichier | Rôle |
|---|---|
| `Designs/<Design>.json` | Le design. L'importateur reconstruit le prefab chaque fois qu'il change. |
| `Editor/EditorGuiImporter.cs` | Construit le prefab avec uGUI + TextMeshPro. Menu **Tools › Lienzo UI**. |
| `Prefabs/<Design>.prefab` | Prêt à l'emploi. |
| `UI Toolkit/<Design>.uxml` + `.uss` | Version UI Toolkit (UIDocument, PanelSettings 1920×1080 Scale With Screen Size). |
| `Scripts/<Design>OnGUI.cs` | Version IMGUI : ajoutez-la à un GameObject. |
| `Scripts/<Design>ViewBase.cs` | Références typées et événements (régénéré). |
| `Scripts/<Design>View.cs` | **Votre** logique : créé une seule fois et jamais écrasé. |
| `Sprites/`, `Fonts/` | Images et polices utilisées par le design. |

`<Design>` est le titre en PascalCase sans accents (`Menú principal` → `MenuPrincipal`).

Si une image manque, rien n'est écrit : l'export s'arrête avant de toucher au projet et vous indique le fichier qu'il n'a pas pu lire.

## Ancrages et mise à l'échelle

- Sans sélection, l'Inspecteur affiche le **Canvas Scaler** : résolution de référence, *match* et arrière-plan.
- Chaque élément a des ancrages, un pivot et des décalages comme un `RectTransform`. Quand la résolution de référence change, les éléments suivent leurs ancrages.
- Les modèles intégrés sont conçus en 1920×1080 ; sur des pages plus petites ou en portrait, ils sont réduits uniformément pour tenir.
- Vérifiez avec **Tester** (F5) sur plusieurs écrans avant d'exporter.

## Dispositions et listes

Un conteneur peut disposer ses enfants horizontalement, verticalement ou en grille (comme les *Layout Groups* d'Unity), avec espacement et marges. Les ScrollViews calculent leur contenu. L'ordre de la Hiérarchie est l'ordre de dessin.

## Logique

- **Dans l'éditeur** : en mode Test, chaque élément peut exécuter un script Rhai au clic (`mostrar("Nombre")`, `ocultar`, `alternar`, `texto`, `color`, `mover`…). Utile pour prototyper des parcours. Les changements faits pendant le test sont perdus quand vous quittez le mode test et ne sont jamais enregistrés ni exportés.
- **Dans Unity** : `<Design>View` hérite de `<Design>ViewBase`, avec un champ par bouton, curseur, case à cocher ou champ de saisie, et des méthodes `On<Element>Click` / `On<Element>Changed` à redéfinir.

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

Donnez des noms uniques aux éléments interactifs : les scripts et `GameObject.Find` ne trouvent que la première correspondance. **Vérifier** signale les noms répétés.

## Vérifier le design

Le bouton **Vérifier** (et `--revisar` en ligne de commande) contrôle, à la résolution de référence :

| Contrôle | Règle |
|---|---|
| Taille du texte | Au moins 18 px en 1080 p |
| Contraste | WCAG : 4.5:1, ou 3:1 pour le grand texte (24 px, ou 18.7 px en gras) ; les contrôles désactivés sont exemptés |
| Taille tactile | Boutons, cases à cocher, champs de saisie et listes déroulantes d'au moins 64 px sur leur petit côté en 1080 p |
| Page | Éléments entièrement ou partiellement hors de la page |
| Noms | Noms répétés sur les éléments interactifs |
| Images | Fichiers d'images qui n'existent plus |

Un contraste inférieur à 2:1, des éléments entièrement hors de la page et des images manquantes sont des erreurs ; le reste sont des avertissements.

## Ligne de commande

```bash
lienzo-ui --plantillas                                   # liste les modèles (0-12)
lienzo-ui --plantilla 4 --guardar inventario.editorgui.json
lienzo-ui --abrir inventario.editorgui.json --revisar    # code de sortie 1 s'il y a des erreurs
lienzo-ui --abrir inventario.editorgui.json --exportar "C:/MyGame"   # sans fenêtre
lienzo-ui --abrir inventario.editorgui.json --screenshot capture.png
lienzo-ui --probar 1080x2400                             # ouvre en mode Test avec cet écran
lienzo-ui --tutorial [--profesional]                     # force le tutoriel
lienzo-ui --sin-tutorial                                 # ne l'ouvre pas
```

`--guardar`, `--exportar` et `--revisar` n'ouvrent pas de fenêtre : utilisez-les pour des scripts, la CI et le [serveur MCP](MCP.md) (outil `revisar_diseno`).

## Code source

Rust + [egui](https://github.com/emilk/egui) :

| Fichier | Contenu |
|---|---|
| `src/model.rs` | Données : `Doc`, `Element`, `Kind`, ancrages, dispositions, modèles |
| `src/canvas.rs` | Toile : sélection, poignées, repères, zoom |
| `src/panels.rs` | Barre supérieure, barre latérale, Inspecteur, fenêtres |
| `src/app.rs` | État, annulation, commandes, raccourcis |
| `src/render.rs` | Dessin des éléments et des polices |
| `src/revision.rs` | Contrôles de Vérifier le design |
| `src/export/` | Exportateurs : `ugui.rs`, `uitk.rs`, `imgui.rs`, `view.rs` et l'importateur Unity |
| `src/tutorial.rs` | Tutoriel intégré (pages en espagnol et en anglais) |
| `src/script.rs` | Scripts Rhai pour le mode Test |

Pour l'étendre : [EXTENDER.md](EXTENDER.md).
