# Tutoriel débutant

🌐 [English](TUTORIAL_BEGINNERS.md) · [Español](TUTORIAL_BEGINNERS.es.md) · [Português](TUTORIAL_BEGINNERS.pt.md) · [Français](TUTORIAL_BEGINNERS.fr.md) · [Deutsch](TUTORIAL_BEGINNERS.de.md) · [Italiano](TUTORIAL_BEGINNERS.it.md) · [Русский](TUTORIAL_BEGINNERS.ru.md) · [简体中文](TUTORIAL_BEGINNERS.zh.md) · [日本語](TUTORIAL_BEGINNERS.ja.md) · [한국어](TUTORIAL_BEGINNERS.ko.md)

Votre première interface avec Lienzo UI et votre premier export vers Unity. Aucune programmation nécessaire.

## 1. Installer et ouvrir

1. Téléchargez `LienzoUI-windows.zip` depuis les releases GitHub ou depuis itch.io.
2. Décompressez-le et ouvrez `lienzo-ui.exe`.
3. La première fois, le **tutoriel** s'ouvre (ce même contenu, dans l'application). Quand vous le terminez, ou avec «Passer et aller à l'éditeur», l'éditeur s'ouvre. Pour le revoir : le bouton **Tutoriel** de la barre supérieure ou **F1**.

Pour utiliser l'application en français : bouton **Accessibilité** › **Langue** › Français.

![Tutoriel dans l'application](images/tutorial.png)

## 2. L'écran

| Zone | Ce qu'on y trouve |
|---|---|
| Barre supérieure | Fichier, Modifier, Affichage, Toile, **Tester**, nom du design, **Accessibilité**, **Vérifier**, **Thèmes**, **Tutoriel**, **Enregistrer**, **Code…**, **Exporter vers Unity** |
| Barre de gauche | **Hiérarchie** (calques), **Éléments**, **Modèles**, **Texte**, **Images**, **Plugins**, **Communauté** |
| Centre | La toile : l'écran de votre jeu (1920 × 1080 par défaut) |
| Droite | L'**Inspecteur** : les propriétés de la sélection |

![Éditeur](images/editor.png)

## 3. Partir d'un modèle

Dans **Modèles**, choisissez l'une des 15 dispositions : Menu principal, HUD de jeu, Fenêtre de dialogue, Réglages, Inventaire, Menu pause, Fin de partie, Sélection de niveau, Écran de chargement, Dialogue de personnage, Classement ou Sélection de personnage. Tout peut y être modifié, et Ctrl+Z annule le changement. Pour partir de zéro : «Vierge». Deux d'entre eux sont en pixel art : ATH de RPG pixel art et Menu principal pixel art.

## 4. Modifier

- Glissez des éléments depuis **Éléments** sur la toile. **Composants prêts à l'emploi** contient des boutons, des barres de vie et de mana, des cartes, des badges et des emplacements d'inventaire.
- **Texte** contient des styles de titre et des pièces de HUD : score, minuteur, pièces, nombres de dégâts, combo, étiquette de niveau et sous-titres.
- Cliquer sélectionne ; glisser déplace ; les coins redimensionnent ; la poignée du haut fait tourner.
- Double-cliquez sur un texte pour le saisir.
- Dans l'**Inspecteur**, changez les couleurs, les bordures, les ombres, les polices et les ancrages.

| Raccourci | Action |
|---|---|
| Ctrl+Z / Ctrl+Y | Annuler / rétablir |
| Ctrl+D / Suppr | Dupliquer / supprimer |
| Ctrl+G / Ctrl+Maj+G | Grouper / dégrouper |
| Flèches (Maj) | Déplacer de 1 px (10 px) |
| Ctrl+S | Enregistrer |
| F5 | Tester |
| Ctrl+E | Exporter vers Unity |
| F1 | Tutoriel |

## 5. Tester

Appuyez sur **Tester** (F5) : l'interface devient interactive et vous pouvez choisir d'autres écrans (téléphone, 4:3, ultra-large) pour voir comment elle s'adapte. Échap quitte le mode test, et le design revient à l'état où il était avant le test.

## 6. Vérifier

Appuyez sur **Vérifier** dans la barre supérieure. L'application signale les textes trop petits pour être lus, le contraste faible, les boutons trop petits pour être touchés, les éléments hors de la page, les noms répétés et les images manquantes. Cliquez sur un avertissement pour sélectionner cet élément et le corriger.

## 7. Exporter vers Unity

1. Appuyez sur **Exporter vers Unity** (Ctrl+E). La première fois, choisissez le dossier de votre projet (celui qui contient `Assets`).
2. Retournez dans Unity : le design apparaît comme un prefab dans `Assets/EditorGUI/Prefabs`.
3. Faites-le glisser dans la scène, ou utilisez le menu **Tools › Lienzo UI › Import design into scene…**.

Chaque fois que vous exportez à nouveau, Unity met à jour le prefab tout seul.

## Images incluses

L'onglet Images de la barre de gauche contient des visuels prêts à l'emploi, sans fichiers à fournir :

- Les pastilles du haut filtrent par Tes images, Arrière-plans, Cadres, Boutons, Icônes, Pixel art et Dégradés. La recherche trouve par nom ou par étiquette.
- Clique sur un élément pour l'ajouter, ou glisse-le sur le canevas. Les cadres et les boutons arrivent avec leur bordure 9-slice : ils s'étirent sans déformer les coins. Les arrière-plans se placent derrière tout le reste.
- L'image est copiée dans ta bibliothèque la première fois que tu l'utilises, et elle est exportée vers Unity avec le design.
- Avec les sprites en pixel art, coche « Pixel art » dans l'Inspector du canevas (rien de sélectionné) pour qu'ils restent nets.

## Écrans verticaux (portrait)

Les téléphones sont souvent tenus à la verticale. Lienzo UI garde une seconde mise en page pour cela, sans toucher à la première :

- Change de vue avec les boutons Paysage / Portrait de la barre du haut.
- Sans rien sélectionner, ouvre « Mise en page portrait » dans l'Inspector et clique sur « Créer la mise en page portrait ». Les éléments démarrent là où ils sont en paysage.
- En vue portrait, ce que tu déplaces ou redimensionnes ne modifie que la mise en page portrait. Clic droit sur un élément › « Revenir au paysage » annule une modification.
- L'export contient les deux mises en page : dans Unity, le prefab passe tout seul à la version verticale quand l'écran est plus haut que large.

## Importer depuis Unity

Tu as déjà une interface dans Unity ? Amène-la dans Lienzo UI :

- Fichier › Importer depuis Unity…
- Choisis un prefab uGUI (.prefab) ou un design que Lienzo UI a déjà exporté (Assets/EditorGUI/Designs/….json).
- Il s'ouvre comme un nouveau design non enregistré : enregistre-le comme projet pour le conserver.
- Si quelque chose n'a pas pu être converti exactement, une fenêtre liste les avertissements : rien ne se perd en silence.
- Si le dossier de ton projet Unity est trouvé, les images du design sont retrouvées automatiquement.

## Ne perds pas ton travail

- Enregistrement automatique : environ chaque minute, Lienzo UI enregistre une copie de récupération. Tu peux le désactiver dans Accessibilité › Enregistrement automatique.
- Si l'application se ferme de façon inattendue, elle te demande la fois suivante « Récupérer le travail non enregistré… ? ». Clique sur Restaurer pour le retrouver ou sur Ignorer pour l'abandonner. Le travail récupéré n'est pas encore un fichier enregistré : appuie sur Ctrl+S pour le conserver.
- Ctrl+S enregistre toujours ton projet comme d'habitude.
- Annuler (Ctrl+Z) garde jusqu'à 200 étapes (avec une limite de mémoire). Si les plus anciennes sont supprimées, la barre d'état affiche « Historique réduit ».

## Communauté

L'onglet Communauté de la barre de gauche est un catalogue partagé de plugins, d'images, de polices, de modèles, d'éléments et de thèmes :

- Parcourir et installer ne demande aucun compte : cherche puis clique sur Installer.
- Pour publier, signaler ou donner des cœurs, tu te connectes avec GitHub ou Google et tu choisis un nom public. Pas de mots de passe : l'app ne garde que ce nom.
- En créant ton profil, tu acceptes les règles : pas de malware, de haine, de plagiat ni de contenu +18 non signalé. Le texte complet se trouve dans docs/TERMS.md et docs/COMMUNITY_GUIDELINES.md.
- Le contenu +18 est masqué, sauf si tu te connectes et actives « Afficher le contenu +18 ».
- Tout ce qui est partagé est vérifié avant publication, et tu peux signaler ce qui enfreint les règles.

## Étapes suivantes

- [Tutoriel pro](TUTORIAL_PROS.fr.md) : ce que l'export génère et où mettre votre logique.
- [Serveur MCP](MCP.md) : demandez des changements à une IA en langage naturel.
