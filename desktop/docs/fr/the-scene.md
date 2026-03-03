# La Scène

La **scène** est la structure centrale de Sova : c'est elle que vous manipulez
en live. Elle organise votre code, votre timing et votre structure musicale
en une hiérarchie simple pensée pour l'improvisation.

## Hiérarchie de la scène

Une scène contient des **lignes** qui tournent en parallèle — chacune est une
piste indépendante qui produit son propre flux d'événements. Chaque ligne
contient des **frames** qui se jouent en séquence : quand une frame se termine,
la suivante démarre. Chaque frame contient un **script** écrit dans l'un des
langages de Sova.

Sur la grille, les lignes sont les colonnes et les frames sont les rangées.
Toutes les colonnes jouent en même temps ; au sein de chaque colonne, les
rangées se succèdent.

## Propriétés des frames

Chaque frame possède ces propriétés :

- **Durée** (beats) — combien de temps la frame joue avant de passer à la
  suivante. Par défaut : 1 beat. Vous pouvez utiliser des valeurs
  fractionnaires (0.25, 0.5, 2.5, etc.).
- **Répétitions** — combien de fois le script de la frame s'exécute durant sa
  durée. Par défaut : 1. Une frame avec une durée de 4 et 4 répétitions exécute
  son script une fois par beat pendant quatre beats.
- **Activée** — si la frame est jouée ou non. Les frames désactivées sont
  ignorées pendant la lecture. Utile pour couper des parties sans les supprimer.
- **Nom** — un label optionnel pour la frame, affiché dans la cellule de la
  grille.
- **Script** — le code et son langage (Bob, Boinx, Cagire ou BaLi).

La **durée effective** d'une frame est `durée × répétitions`. Une frame avec une
durée de 0.5 et 8 répétitions occupe 4 beats au total.

## Propriétés des lignes

Chaque ligne dispose de contrôles qui définissent comment ses frames sont jouées :

- **Boucle** — quand elle est activée, la ligne reprend depuis le début après
  la fin de sa dernière frame. Sinon, la ligne se joue une fois et s'arrête.
- **Trailing** — quand il est activé, les événements des frames précédentes
  continuent de jouer pendant que la frame suivante commence. Sinon, les
  événements précédents sont coupés.
- **Vitesse** — un multiplicateur sur le tempo de la ligne. Une vitesse de 2.0
  signifie que la ligne joue deux fois plus vite ; 0.5 signifie à mi-vitesse.
  N'affecte que cette ligne.
- **Frame de début / Frame de fin** — restreint optionnellement la lecture à une
  plage de frames au sein de la ligne. Utile pour se concentrer sur une section
  pendant la performance.

## Modes d'exécution

Le **mode d'exécution** de la scène contrôle comment les lignes se synchronisent
au démarrage :

- **Free** — les lignes démarrent immédiatement, quelle que soit la position
  des autres lignes. Chaque ligne boucle à son propre rythme. C'est le mode par
  défaut.
- **AtQuantum** — les lignes attendent la prochaine limite de quantum (début de
  mesure) avant de démarrer. Cela garde tout aligné sur la structure globale de
  la phrase.
- **LongestLine** — toutes les lignes attendent que la plus longue ligne en
  cours termine son cycle avant de redémarrer. Cela crée une grille de boucles
  naturelle où tout se réinitialise ensemble.

Vous pouvez changer le mode d'exécution depuis la barre de transport.

## Sauvegarde et chargement

Vous pouvez sauvegarder et charger des scènes via le menu de scène. La scène
capture tout : lignes, frames, scripts, variables et configuration. Quand vous
vous connectez à un serveur, vous recevez sa scène actuelle automatiquement.
