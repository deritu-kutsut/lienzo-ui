# Tutoriel débutant

🌐 [English](TUTORIAL_BEGINNERS.md) · [Español](TUTORIAL_NOVATOS.md) · [Português](TUTORIAL_BEGINNERS.pt.md) · [Français](TUTORIAL_BEGINNERS.fr.md) · [Deutsch](TUTORIAL_BEGINNERS.de.md) · [Italiano](TUTORIAL_BEGINNERS.it.md) · [Русский](TUTORIAL_BEGINNERS.ru.md) · [简体中文](TUTORIAL_BEGINNERS.zh.md) · [日本語](TUTORIAL_BEGINNERS.ja.md) · [한국어](TUTORIAL_BEGINNERS.ko.md)

Votre première interface avec Lienzo UI et votre premier export vers Unity. Aucune programmation nécessaire.

## 1. Installer et ouvrir

1. Téléchargez `LienzoUI-windows.zip` depuis les releases GitHub ou depuis itch.io.
2. Décompressez-le et ouvrez `lienzo-ui.exe`.
3. La première fois, le **tutoriel** s'ouvre (ce même contenu, dans l'application). Quand vous le terminez, ou avec «Passer et aller à l'éditeur», l'éditeur s'ouvre. Pour le revoir : le bouton **Tutoriel** de la barre supérieure ou **F1**.

Pour utiliser l'application en français : bouton **Accessibilité** › **Langue** › Français.

![Tutoriel dans l'application](imagenes/tutorial.png)

## 2. L'écran

| Zone | Ce qu'on y trouve |
|---|---|
| Barre supérieure | Fichier, Modifier, Affichage, Toile, **Tester**, nom du design, **Accessibilité**, **Vérifier**, **Thèmes**, **Tutoriel**, **Enregistrer**, **Code…**, **Exporter vers Unity** |
| Barre de gauche | **Hiérarchie** (calques), **Éléments**, **Modèles**, **Texte**, **Images**, **Plugins**, **Communauté** |
| Centre | La toile : l'écran de votre jeu (1920 × 1080 par défaut) |
| Droite | L'**Inspecteur** : les propriétés de la sélection |

![Éditeur](imagenes/editor.png)

## 3. Partir d'un modèle

Dans **Modèles**, choisissez l'une des 12 dispositions : Menu principal, HUD de jeu, Fenêtre de dialogue, Réglages, Inventaire, Menu pause, Fin de partie, Sélection de niveau, Écran de chargement, Dialogue de personnage, Classement ou Sélection de personnage. Tout peut y être modifié, et Ctrl+Z annule le changement. Pour partir de zéro : «Vierge».

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
3. Faites-le glisser dans la scène, ou utilisez le menu **Tools › Lienzo UI › Importar diseño en la escena** (le menu d'Unity est en espagnol : «Importer le design dans la scène»).

Chaque fois que vous exportez à nouveau, Unity met à jour le prefab tout seul.

## Étapes suivantes

- [Tutoriel pro](TUTORIAL_PROS.fr.md) : ce que l'export génère et où mettre votre logique.
- [Serveur MCP](MCP.md) : demandez des changements à une IA en langage naturel.
