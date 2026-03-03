# La Scène

Une **scène** est le conteneur principal de tout ce sur quoi vous travaillez
dans Sova. Elle contient tout le matériau musical — le code, le timing, la
structure — dans une hiérarchie conçue pour la performance live.

## Hiérarchie de la scène

```
Scène
 └─ Ligne        (pistes parallèles — colonnes de la grille)
     └─ Case     (étapes séquentielles — rangées de la grille)
         └─ Script  (code + identifiant de langage)
```

- Une **scène** contient une ou plusieurs **lignes**.
- Les lignes s'exécutent **en parallèle** — ce sont des pistes indépendantes,
  chacune produisant son propre flux d'événements simultanément.
- Chaque ligne contient une ou plusieurs **cases**.
- Les cases s'exécutent **en séquence** — quand une case se termine, la
  suivante commence.
- Chaque case contient un **script** : un morceau de code écrit dans l'un des
  langages de Sova.

Imaginez un tableau : les lignes sont les colonnes, les cases sont les rangées.
La scène joue toutes les colonnes en même temps, et au sein de chaque colonne,
les rangées se jouent l'une après l'autre.

## Propriétés des cases

Chaque case possède ces propriétés :

- **Durée** (battements) — combien de temps la case joue avant de passer à la
  suivante. Par défaut : 1 battement. Vous pouvez utiliser des valeurs
  fractionnaires (0.25, 0.5, 2.5, etc.).
- **Répétitions** — combien de fois le script de la case s'exécute durant sa
  durée. Par défaut : 1. Une case avec une durée de 4 et 4 répétitions exécute
  son script une fois par battement pendant quatre battements.
- **Activée** — si la case est jouée ou non. Les cases désactivées sont
  ignorées pendant la lecture. Utile pour couper des parties sans les supprimer.
- **Nom** — un label optionnel pour la case, affiché dans la cellule de la
  grille.
- **Script** — le code et son langage (Bob, Boinx, Cagire ou BaLi).

La **durée effective** d'une case est `durée × répétitions`. Une case avec une
durée de 0.5 et 8 répétitions occupe 4 battements au total.

## Propriétés des lignes

Chaque ligne dispose de contrôles qui définissent comment ses cases sont jouées :

- **Boucle** — quand elle est activée, la ligne reprend depuis le début après
  la fin de sa dernière case. Sinon, la ligne se joue une fois et s'arrête.
- **Trailing** — quand il est activé, les événements des cases précédentes
  continuent de résonner pendant que la case suivante commence. Sinon, les
  événements précédents sont coupés.
- **Vitesse** — un multiplicateur sur le tempo de la ligne. Une vitesse de 2.0
  signifie que la ligne joue deux fois plus vite ; 0.5 signifie à mi-vitesse.
  N'affecte que cette ligne.
- **Case de début / Case de fin** — restreint optionnellement la lecture à une
  plage de cases au sein de la ligne. Utile pour se concentrer sur une section
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

Les scènes sont sérialisées en données MessagePack. Vous pouvez sauvegarder et
charger des scènes via le menu de scène. La scène capture tout : toutes les
lignes, cases, scripts, stockages de variables et configuration. Quand vous vous
connectez à un serveur, vous recevez sa scène actuelle automatiquement.
