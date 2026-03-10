Sova est un séquenceur musical conçu pour le live coding et l'improvisation musicale collective. Il est conçu comme un instrument de musique et comme un outil d'expérimentation. Sova peut être catégorisé comme un logiciel expérimental. Il s'agit d'un artefact aussi bien technique que poétique. Le logiciel est fait pour évoluer avec ses utilisateurs, pour se transformer, pour accompagner un/des musiciens dans leur pratique musicale et leur exploration du live coding.

Dans l'environnement proposé par Sova, on écrit du code que le logiciel transforme en instructions MIDI, en messages OSC, en voix de synthétiseurs ou d'échantillonneurs. Le code est exécuté en rythme, et tout les instruments présents sur le même réseau peuvent se synchroniser à Sova ou inversement (Ableton Link). Plusieurs langages de programmation sont disponibles et proposent chacun une approche différente de l'expression musicale. Plusieurs musiciens peuvent se connecter et jouer ensemble, partageant les mêmes scripts, les mêmes données, la même temporalité. Aucun prérequis n'est nécessaire pour débuter, sinon l'envie de pratiquer cet instrument.

## Connexion

Lors de l'ouverture du logiciel, vous vous retrouvez immédiatement confronté à la fenêtre de connexion. Vous pouvez cliquer sur le bouton `'Démarrer le serveur'` pour lancer un serveur local avec sa configuration par défaut. À côté de ce premier bouton, vous en trouverez un autre : '`Connecter'`. Cliquez sur ce dernier pour... vous connecter à une nouvelle session de jeu sur le serveur que vous venez de lancer. Sova est fondé sur une architecture client / serveur.

Il existera toujours un musicien dont le rôle sera d'héberger un espace de jeu. Ce musicien lancera un serveur. Le serveur peut être lancé par vous-même si vous êtes seul, ou par un camarade / ami. Il est tout à fait possible de jouer en local, sur le même réseau, ou de vous connecter à un serveur distant à l'autre bout du monde. L'option `'Audio Local'` vous permettra de lancer également un serveur audio pour entendre ce qu'il se passe si vous n'êtes pas dans la même pièce que les autres musiciens.

Cliquez sur l'icône en forme d'engrenage pour ouvrir la configuration du serveur local si le besoin s'en fait ressentir. Les options sont assez simples et vous permettent de décider de l'adresse du serveur et du tempo / de la métrique de base.

## Votre premier son

Après votre connexion, appuyez sur `Play` dans la barre supérieure, en haut à gauche.  Double cliquez ensuite sur la `Frame 0` de la `Ligne 0`, votre premier script ! Un éditeur devrait immédiatement s'ouvrir. C'est ici que vous taperez votre code. Dans la barre supérieure, choisissez le langage `Cagire` et tapez :

```forth
kick snd .
```

Appuyez sur `Cmd+Entrée` (`Ctrl+Entrée` sous Linux/Windows) pour évaluer le code. L'éditeur devrait émettre un flash pour confirmer l'action et un kick devrait retentir à chaque temps. Bravo ! Vous venez de live coder votre tout premier script. Notez que chaque **frame** peut utiliser un langage différent, et que vous avez la possibilité de changer le code en temps réel, au cours de l'exécution.

Vous pouvez substituer le code à un autre extrait plus mélodique :

```forth
0 0.25 0.5 0.75 at
c4 e4 g4 c5 arp note 
sine snd 
.5 decay
.4 verb 
.
```

Remplacez maintenant `sine` par `tri` et réévaluez. Le changement est immédiat. Vous comprenez ainsi l'esprit du live coding : on explore et on cherche les idées musicales en temps réel, en improvisant au travers du code source.

## L'espace de jeu

L'espace de jeu central sur l'écran s'appelle 'la scène'. Cette scène est d'une importance primordiale, et il est utile de comprendre la terminologie de base employée pour en parler. Chaque colonne est une **ligne** (`line`) d'exécution. Toutes les lignes s'exécutent en parallèle. Chaque tuile dans une colonne est désignée sous le nom de **frame**. Toutes les **frames** d'une **ligne** s'exécutent les unes après les autres, de manière séquentielle. Une ligne boucle sur ses **frames** de haut en bas, puis recommence.

C'est à vous de déterminer cette scène va être employée et quel sera son rôle au cours d'une improvisation. Il peut y avoir une ligne par musicien, ou une ligne par type d'instrument, etc. Personne ne vous dira exactement quoi faire ou comment vous répartir l'espace de jeu. Consultez l'article consacré à la scène pour en apprendre plus sur le modèle d'exécution :)

## Quatre langages

Chaque frame possède son propre langage. Choisissez celui qui correspond à votre
intention musicale.

**Cagire** — à pile, style Forth. On empile des valeurs, on applique des mots,
on émet avec `.`. Adapté au sound design et à l'expérimentation rapide.

```forth
c4 min7 arp note 0.5 decay 0.4 verb sine snd .
```

**Bob** — impératif, notation polonaise. Event maps, boucles, timing explicite
avec WAIT. Adapté aux séquences mélodiques précises.

```
RANGE 0 3 :
  >> [note: ADD 60 MUL I 4 vel: 100]
  WAIT 0.25
END
```

**BaLi** — style Lisp, à base d'expressions. S-expressions imbriquées, les
boucles et transformations se composent naturellement. Adapté à la composition
algorithmique et générative.

```
(loop 4
  (note (+ 60 (* $i 3)) 90)
  1//4)
```

**Boinx** — notation de motifs déclarative. Séquences et simultanéité exprimées
visuellement par des crochets. Adapté aux motifs rythmiques lisibles d'un coup
d'œil.

```
<s: 'kick'> | [. _ . _]
```

Pour changer le langage d'une frame, ouvrez l'éditeur et sélectionnez dans le
menu déroulant en haut, ou appuyez sur Cmd+L (Ctrl+L). Voir l'article
**Langages** pour plus de détails, et cliquez sur les onglets de chaque langage
(Bob, BaLi, Boinx, Cagire) pour les références complètes.

## Timing

Chaque frame possède une durée en temps (beats). Le séquenceur exécute le script
de la frame une fois par durée, puis passe à la suivante. Une frame d'un temps à
120 BPM s'exécute toutes les demi-secondes.

À l'intérieur d'une frame, on peut subdiviser le temps. En Cagire, `at` place
des sons à des positions fractionnaires dans la durée. En Bob, `WAIT` fait
avancer l'horloge explicitement.

Réglez le tempo et le quantum dans la barre de transport en haut. Voir l'article
**Timing**.

## Vos instruments

Sova envoie les événements vers des slots numérotés de 1 à 16. Ouvrez le
panneau Périphériques pour connecter des ports MIDI, créer des points d'accès OSC
ou activer le moteur audio intégré (Doux). Le slot 1 est celui par défaut. Dans
votre code, utilisez `dev` pour cibler un slot :

```forth
2 dev c4 note 100 vel .
```

Voir l'article **Périphériques**.

## Jouer ensemble

Lancez un serveur. Les autres musiciens se connectent à votre adresse IP et
votre port. Tout le monde voit la même scène, édite en temps réel et reste
synchronisé via Ableton Link. Utilisez des lignes différentes pour éviter les
conflits d'édition. Le chat est intégré. Voir l'article **Multijoueur**.

## Visuels

Sova intègre une version Rust d'un outil de live coding visuel très populaire : [Hydra](https://hydra.ojack.xyz). Il s'agit originalement d'un outil de live coding GLSL écrit en Javascript, conçu par Olivia Jack. La plupart des live coders sont assez familiers avec ce langage. Un éditeur est disponible dans le menu `Affichage > Visuels`. Les visuels générés sont affichés en arrière-plan de l'interface de jeu, en temps réel. Ces visuels ne sont pas partagés avec les autres utilisateurs.

```
osc(60, 0.1).rotate(0, 0.1).kaleid(4).out()
```

Voir l'article **Visuels (Hydra)**.

## Raccourcis

| Action | macOS | Linux/Windows |
|--------|-------|---------------|
| Évaluer le code | Cmd+Entrée | Ctrl+Entrée |
| Palette de commandes | Cmd+K | Ctrl+K |
| Lecture / Arrêt | Cmd+Shift+Espace | Ctrl+Shift+Espace |
| Sauvegarder la scène | Cmd+S | Ctrl+S |
| Charger une scène | Cmd+O | Ctrl+O |
| Panneau Serveur | Cmd+Shift+S | Ctrl+Shift+S |
| Panneau Périphériques | Cmd+Shift+I | Ctrl+Shift+I |
| Documentation | Cmd+Shift+H | Ctrl+Shift+H |
| Visuels | Cmd+Shift+V | Ctrl+Shift+V |
| Changer de langage | Cmd+L | Ctrl+L |

Appuyez sur F1 pour afficher tous les raccourcis. La palette de commandes
(Cmd+K) liste toutes les actions avec leur raccourci.
