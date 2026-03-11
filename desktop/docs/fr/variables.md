# Variables

Les variables stockent des valeurs qui persistent entre les événements ou entre
les frames. On les utilise pour coordonner les scripts, accumuler de l'état et
construire des motifs évolutifs — la base de toute musique générative.

## Portées

Quatre portées. La portée détermine qui voit la variable et combien de temps
elle vit.

**Instance** — mémoire locale. Se réinitialise à chaque exécution du script.
On l'utilise pour des calculs intermédiaires qui n'ont pas besoin de survivre
au-delà d'une seule exécution.

**Frame** — survit aux répétitions au sein de la même frame. Se réinitialise
quand la ligne passe à la frame suivante. Bien adaptée aux compteurs qui
s'accumulent d'une répétition à l'autre : chaque exécution lit la valeur
précédente, la modifie et la restocke.

**Ligne** — partagée entre toutes les frames d'une ligne. Une frame écrit une
valeur, une autre la lit. Utile pour faire circuler du contexte le long d'une
séquence : définir une note fondamentale dans la frame A, transposer à partir
de celle-ci dans les frames B et C.

**Globale** — visible par tous les scripts de la session, toutes lignes et
frames confondues. À utiliser avec parcimonie. Réservée de préférence à un
état partagé au niveau de la session : tonalité commune, compteur lu par
plusieurs lignes.

## Stocker et récupérer

Chaque langage a sa propre syntaxe pour lire et écrire des variables. Le
mécanisme sous-jacent est le même : stocker une valeur sous un nom, la
récupérer plus tard. Les variables inconnues renvoient zéro. Voir les onglets
de langage pour la syntaxe.

La portée fait partie du nom de la variable. Un préfixe indique si l'on
s'adresse à une variable de frame, de ligne ou globale. Sans préfixe, la
variable est de portée instance.

## Accumulateurs

Le motif le plus courant : récupérer une valeur, la modifier, la restocker.
Cela transforme une variable en compteur, en accumulateur de phase ou en toute
quantité évolutive. Combiné avec des variables de portée frame et des
répétitions, une seule ligne de logique peut engendrer une séquence entière qui
se décale à chaque répétition.

## Nommer les sons

Stocker un nom de son dans une variable de portée ligne et le référencer depuis
plusieurs frames. Modifier la valeur à un seul endroit et toutes les frames de
la ligne adoptent le nouveau son. Cela évite de dupliquer les noms de sons
entre les frames et accélère les ajustements en live.

## Valeurs d'environnement

Valeurs en lecture seule fournies par le runtime. Les plus utiles : position en
beats, tempo, nombre aléatoire, index de frame, index de ligne et compteur
d'itération. Elles permettent à vos scripts de réagir à leur position dans la
timeline sans comptabilité manuelle. Chaque langage les expose différemment —
voir les onglets de langage.

## Visibilité temporelle

Dans une même frame, on relit ce que l'on vient d'écrire. Les modifications ne
deviennent visibles pour les autres frames qu'après la fin d'exécution de la
frame courante. Si deux frames s'exécutent dans la même passe, chacune voit les
valeurs précédentes de l'autre, pas les valeurs actuelles. Cela prévient les
surprises d'ordonnancement : le résultat ne dépend pas de l'ordre dans lequel
l'ordonnanceur exécute les frames.
