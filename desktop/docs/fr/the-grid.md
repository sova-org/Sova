# La Grille

La grille est votre espace de travail. Les lignes sont les colonnes, les frames
sont les rangées. On y écrit du code, on réorganise les parties, on pilote la
musique pendant la performance.

## Disposition

Chaque colonne est une ligne. L'en-tête affiche le numéro de ligne et les
contrôles de boucle, trailing et vitesse. Sous l'en-tête, chaque cellule est
une frame qui montre son nom, sa durée, ses répétitions et un aperçu du code.

La frame en cours de lecture est mise en surbrillance. En multijoueur, les
curseurs des autres musiciens apparaissent sur les cellules qu'ils éditent.

## Navigation

On se déplace avec les flèches ou la souris.

- Flèche Haut / Bas -- se déplacer entre les frames de la ligne courante
- Flèche Gauche / Droite -- se déplacer entre les lignes
- Clic -- sélectionner une cellule
- Shift + Clic -- étendre la sélection du point d'ancrage à la cellule cliquée
- Shift + Flèche Haut/Bas -- étendre la sélection verticalement
- Double-clic -- ouvrir l'éditeur de code pour une frame
- Echap -- effacer la sélection

## Édition des propriétés de frame

Sélectionnez une cellule, puis appuyez sur une touche pour éditer une propriété
directement :

- Entrée ou D -- durée
- R -- répétitions
- N -- nom

Dans le champ d'édition :

- Entrée -- valider
- Tab -- valider et passer au champ suivant
- Shift+Tab -- valider et revenir au champ précédent
- Echap -- annuler

Pour éditer le code, double-cliquez sur la cellule. L'éditeur s'ouvre avec la
coloration syntaxique du langage de la frame.

## Contrôles de ligne

- S -- modifier la vitesse de la ligne
- L -- activer/désactiver la boucle
- T -- activer/désactiver le trailing

Tab permet de passer du champ Frame de début au champ Frame de fin dans
l'en-tête de ligne.

## Opérations sur les frames

- Suppr / Retour arrière -- supprimer la/les frame(s) sélectionnée(s)
- Cmd+D -- dupliquer
- Cmd+C -- copier
- Cmd+X -- couper
- Cmd+V -- coller après le curseur
- Alt+Haut -- déplacer vers le haut
- Alt+Bas -- déplacer vers le bas

## Opérations sur les lignes

- Cmd+Shift+D -- dupliquer la ligne courante
- Cmd+Suppr -- supprimer la ligne courante
- Alt+Gauche -- déplacer la ligne vers la gauche
- Alt+Droite -- déplacer la ligne vers la droite

## Sélection

- Cmd+A -- sélectionner toutes les frames de la ligne courante
- Echap -- effacer la sélection

La multi-sélection fonctionne avec toutes les opérations : supprimer,
dupliquer, copier, couper, déplacer.

## Menu contextuel

Clic droit sur une cellule pour les options supplémentaires : ajout de frames,
insertion de lignes, visibilité des panneaux, activation/désactivation de
frames.

## Conseils pratiques

Nommez vos frames (N). Une grille pleine de cellules sans nom devient vite
illisible. Étiquetez vos sections : "intro", "drop", "breakdown". En
performance, il faut retrouver les choses d'un coup d'oeil.

Dupliquez avant de modifier (Cmd+D). Copiez une frame qui marche, changez un
seul truc. L'original reste intact et vous avez un filet de sécurité si l'edit
ne passe pas.

Réordonnez à la volée (Alt+Haut/Bas). En pleine performance, on peut changer
l'ordre des frames dans une ligne sans couper la lecture. Déplacez un fill
avant le drop, avancez une transition.

Désactivez les frames au lieu de les supprimer. Clic droit sur une cellule pour
la désactiver. Le code reste visible mais la frame est sautée pendant la
lecture. Réactivez-la quand vous en avez besoin.

Utilisez les plages de frames (début/fin) pour isoler une section. Mettez une
ligne en boucle sur les frames 2-4 pendant que vous construisez la frame 5.
Quand c'est prêt, élargissez la plage.

Gardez les lignes ciblées. Une ligne par rôle musical : drums, basse, mélodie,
effets. C'est plus simple de couper, isoler ou réarranger une ligne quand elle
ne fait qu'une seule chose.

Voir **La Scène** pour le fonctionnement des lignes, des frames et des modes
d'exécution.
