# La Grille

La grille de scène est l'espace de travail principal dans Sova. Les lignes sont
affichées en colonnes et les cases en rangées au sein de chaque colonne. Vous
naviguez, éditez et organisez votre matériau musical ici.

## Disposition

Chaque colonne est une **ligne**. L'en-tête de colonne affiche le numéro de
ligne et les contrôles (boucle, trailing, vitesse). Sous l'en-tête, chaque
cellule est une **case** — elle affiche le nom de la case (le cas échéant), la
durée, les répétitions et un aperçu du code du script.

La case en cours de lecture est mise en surbrillance. Si d'autres musiciens
éditent une case, vous verrez leur indicateur de curseur sur la cellule.

## Navigation

| Touche | Action |
|--------|--------|
| Flèche Haut / Bas | Déplacer le curseur entre les cases de la ligne courante |
| Flèche Gauche / Droite | Déplacer le curseur entre les lignes (même index de case) |
| Clic | Sélectionner une cellule |
| Shift + Clic | Étendre la sélection du point d'ancrage jusqu'à la cellule cliquée |
| Shift + Flèche Haut/Bas | Étendre la sélection verticalement |
| Double-clic | Ouvrir l'éditeur de code pour une cellule |
| Échap | Effacer la sélection |

## Édition des propriétés de case

Avec une cellule sélectionnée, appuyez sur une touche pour éditer une propriété
en ligne :

| Touche | Édition |
|--------|---------|
| Entrée ou D | Durée |
| R | Répétitions |
| N | Nom |

Dans un champ d'édition :

| Touche | Action |
|--------|--------|
| Entrée | Valider l'édition |
| Tab | Valider et passer au champ suivant |
| Shift+Tab | Valider et revenir au champ précédent |
| Échap | Annuler l'édition |

Pour éditer le **code** d'une case, double-cliquez sur la cellule ou appuyez
sur Entrée si l'éditeur de code est configuré pour s'ouvrir ainsi. L'éditeur de
code est un éditeur complet avec coloration syntaxique pour le langage de la
case.

## Contrôles de ligne

| Touche | Action |
|--------|--------|
| S | Modifier la vitesse de la ligne |
| L | Activer/Désactiver la boucle |
| T | Activer/Désactiver le trailing |

Vous pouvez aussi ajuster la case de début et la case de fin depuis l'en-tête
de ligne. Tab permet de passer du champ Case de début au champ Case de fin.

## Opérations sur les cases

| Touche | Action |
|--------|--------|
| Suppr / Retour arrière | Supprimer la/les case(s) sélectionnée(s) |
| Cmd+D | Dupliquer la/les case(s) sélectionnée(s) |
| Cmd+C | Copier la/les case(s) sélectionnée(s) |
| Cmd+X | Couper la/les case(s) sélectionnée(s) |
| Cmd+V | Coller les cases après le curseur |
| Alt+Haut | Déplacer la/les case(s) sélectionnée(s) vers le haut |
| Alt+Bas | Déplacer la/les case(s) sélectionnée(s) vers le bas |

## Opérations sur les lignes

| Touche | Action |
|--------|--------|
| Cmd+Shift+D | Dupliquer la ligne courante |
| Cmd+Suppr | Supprimer la ligne courante |
| Alt+Gauche | Déplacer la ligne d'une position vers la gauche |
| Alt+Droite | Déplacer la ligne d'une position vers la droite |

## Sélection

| Touche | Action |
|--------|--------|
| Cmd+A | Sélectionner toutes les cases de la ligne courante |
| Échap | Effacer la sélection |

Vous pouvez sélectionner plusieurs cases et appliquer des opérations (supprimer,
dupliquer, copier, couper, déplacer) à toutes en une seule fois.

## Menu contextuel

Faites un clic droit sur une cellule de case pour accéder aux options
supplémentaires : ajout de cases, insertion de lignes, visibilité des panneaux,
et plus encore.

## Astuces

- Utilisez le **Nom** (N) pour étiqueter les sections de votre arrangement —
  cela rend la grille bien plus lisible d'un coup d'oeil.
- **Dupliquer** (Cmd+D) est le moyen le plus rapide de créer des variations :
  copiez une case, puis modifiez le code.
- **Alt+Haut/Bas** vous permet de réorganiser les cases à la volée pendant une
  performance.
- Les cases désactivées (basculer via le menu contextuel) restent visibles mais
  ne sont pas jouées — pratique pour garder des idées alternatives sous la main.
