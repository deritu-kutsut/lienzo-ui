# Tutoriel pro

🌐 [English](TUTORIAL_PROS.md) · [Español](TUTORIAL_PROS.es.md) · [Português](TUTORIAL_PROS.pt.md) · [Français](TUTORIAL_PROS.fr.md) · [Deutsch](TUTORIAL_PROS.de.md) · [Italiano](TUTORIAL_PROS.it.md) · [Русский](TUTORIAL_PROS.ru.md) · [简体中文](TUTORIAL_PROS.zh.md) · [日本語](TUTORIAL_PROS.ja.md) · [한국어](TUTORIAL_PROS.ko.md)

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

- **Dans l'éditeur** : en mode Test, chaque élément peut exécuter un script Rhai au clic (`show("Name")`, `hide`, `toggle`, `set_text`, `set_color`, `move_by`…). Utile pour prototyper des parcours. Les changements faits pendant le test sont perdus quand vous quittez le mode test et ne sont jamais enregistrés ni exportés.
- **Dans Unity** : `<Design>View` hérite de `<Design>ViewBase`, avec un champ par bouton, curseur, case à cocher ou champ de saisie, et des méthodes `On<Element>Click` / `On<Element>Changed` à redéfinir.

```csharp
public class AjustesView : AjustesViewBase
{
    protected override void OnBotonVolverClick() => gameObject.SetActive(false);
}
```

Donnez des noms uniques aux éléments interactifs : les scripts et `GameObject.Find` ne trouvent que la première correspondance. **Vérifier** signale les noms répétés.

## Vérifier le design

Le bouton **Vérifier** (et `--review` en ligne de commande) contrôle, à la résolution de référence :

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
lienzo-ui --templates                                   # liste les modèles (0-12)
lienzo-ui --template 4 --save inventory.editorgui.json
lienzo-ui --open inventory.editorgui.json --review    # code de sortie 1 s'il y a des erreurs
lienzo-ui --open inventory.editorgui.json --export "C:/MyGame"   # sans fenêtre
lienzo-ui --open inventory.editorgui.json --screenshot capture.png
lienzo-ui --test 1080x2400                             # ouvre en mode Test avec cet écran
lienzo-ui --tutorial [--pro]                     # force le tutoriel
lienzo-ui --no-tutorial                                 # ne l'ouvre pas
```

`--save`, `--export` et `--review` n'ouvrent pas de fenêtre : utilisez-les pour des scripts, la CI et le [serveur MCP](MCP.md) (outil `review_design`).

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

Pour l'étendre : [EXTENDING.md](EXTENDING.md).

## Mise en page portrait en détail

La mise en page portrait est une seconde vue du même design (docs/FORMAT.md › PortraitLayout) :

- Elle a sa propre résolution de référence, « Taille portrait » (1080 × 1920 par défaut).
- Les éléments sans exception gardent leur RectTransform paysage et sont réévalués par rapport à la page portrait : un fond plein écran s'étire, ceux ancrés à droite suivent le bord droit.
- Seul ce que tu modifies en vue portrait est enregistré, sous forme d'exceptions (position, taille, rotation, ancres, visibilité, taille de police, layout). Clic droit › « Revenir au paysage » en supprime une.
- Export : le JSON contient les exceptions et le script généré LienzoOrientationLayout les applique quand Screen.width < Screen.height, puis de nouveau si la taille change.
- « Supprimer la mise en page portrait » efface toutes les exceptions.

## 9-slice et pack intégré

Pour les cadres, boutons et panneaux qui s'étirent :

- Sélectionne une Image et définis « 9-slice » dans « Réglages de l'image (Unity) » : la bordure en pixels de l'image source (gauche, bas, droite, haut), comme dans le Sprite Editor d'Unity. L'export génère un Sprite 9-slice.
- « Pixels par unité » : 100 garde les bordures à leur taille en pixels, 200 les réduit de moitié.
- Le pack intégré (assets/pack/manifest.json, docs/CONTENT.md) définit déjà le découpage et les pixels par unité : cadres de 512×512 avec découpage 96, boutons de 512×192.
- Utiliser un élément le copie dans ta bibliothèque sous le nom pack_<id>.<ext> (masqué dans « Tes images ») ; l'exporteur le copie dans les Sprites d'Unity.
- Les modèles ATH de RPG pixel art et Menu principal pixel art utilisent le pack pixel. Active « Pixel art » pour le filtre Point.

## Vérification et corrections rapides

La fenêtre Vérifier liste les erreurs et les avertissements ; clique sur l'un d'eux pour sélectionner son élément. Six problèmes ont un bouton « Corriger » (une étape d'annulation) :

- Texte sous la taille minimale : porté au minimum.
- Contrôle trop petit pour être touché : agrandi autour de son centre.
- Élément en partie hors de la page : déplacé à l'intérieur, s'il y tient.
- Nom répété : _2, _3… est ajouté.
- Image manquante : la référence est supprimée.
- Contraste faible : la couleur du texte est assombrie ou éclaircie.

« Corriger tous les problèmes sûrs » les applique en quelques passes. Des problèmes comme une police sans certains caractères n'ont pas de correction automatique. Les corrections fonctionnent aussi dans la mise en page portrait. En ligne de commande : lienzo-ui --open fichier --review (code 1 en cas d'erreurs).

## Importer depuis Unity : prefabs et CLI

Fichier › Importer depuis Unity… lit un .prefab uGUI (le YAML texte d'Unity) ou un Designs/*.json exporté par Lienzo UI et crée un design ordinaire.

- Ce qui n'a pas pu être converti apparaît sous forme d'avertissements.
- Si le dossier du projet Unity est détecté, les images sont résolues par rapport à lui pendant la session.
- Sans fenêtre : lienzo-ui --import-unity Menu.prefab --save Menu.editorgui.json (aussi avec --review et --export).
- Projet entier : lienzo-ui --import-unity-project MonProjet --out Importes convertit tous les prefabs d'interface. Il n'écrase jamais (il ajoute _2) et ignore les prefabs sans interface.
- Détails dans docs/UNITY_IMPORT.md.

## Communauté : plugins, polices et règles

Dans l'onglet Communauté, Installer ajoute du contenu partagé en un clic : plugins, polices, images, modèles, éléments et thèmes.

- Un plugin de la communauté va dans un dossier nommé community-<nom>-<hash>. Il est d'abord écrit dans un dossier temporaire : une installation interrompue ne laisse jamais un plugin à moitié installé. Il apparaît dans l'onglet Plugins.
- Ses scripts sont les mêmes scripts Rhai : isolés, sans accès aux fichiers ni au réseau, avec des limites d'opérations, de temps et de mémoire (docs/PLUGINS.md).
- Une police installée est ajoutée à l'onglet Texte et exportée vers Unity avec le design.
- Les téléchargements sont vérifiés avec l'empreinte SHA-256 publiée ; si elle ne correspond pas, rien n'est installé.
- Pour publier, tu te connectes (GitHub ou Google), tu choisis un nom public et tu acceptes les règles. Tout est vérifié avant, et le contenu +18 doit être signalé. Voir docs/TERMS.md, docs/COMMUNITY_GUIDELINES.md et docs/PRIVACY.md.

## Enregistrement automatique, récupération et annulation

- Environ chaque minute, pendant que tu modifies, le design complet (mise en page portrait comprise) est écrit dans le dossier de récupération de l'app. Jamais en mode Test. 10 récupérations au maximum sont conservées.
- Au démarrage, une récupération plus récente que le fichier du projet est proposée : Restaurer ou Ignorer. Après un Enregistrer normal, elle n'est plus utile.
- On peut la désactiver dans Accessibilité › Enregistrement automatique.
- L'historique d'annulation est limité à 200 étapes et environ 64 Mo. Quand les plus anciennes sont supprimées, la barre d'état affiche « Historique réduit ».
- Chaque commande de plugin et chaque Corriger de la fenêtre Vérifier compte pour une seule étape d'annulation.
