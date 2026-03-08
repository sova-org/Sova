Sova est un séquenceur de live coding. Vous écrivez du code, Sova le transforme
en notes MIDI, messages OSC et audio -- le tout synchronisé sur une horloge
partagée via Ableton Link. Quatre langages intégrés, chacun avec sa propre façon
de penser la musique. Plusieurs musiciens peuvent se connecter et jouer ensemble
sur la même scène.

## Connexion

Ouvrez le panneau Serveur et cliquez sur Démarrer. L'application lance un
serveur local et s'y connecte. C'est tout.

## Premier son

Double-cliquez sur une cellule de frame dans la grille pour ouvrir l'éditeur. Le
langage par défaut est Cagire. Tapez :

```forth
kick snd .
```

Appuyez sur Cmd+Entrée (Ctrl+Entrée sous Linux/Windows) pour évaluer. Un kick
retentit à chaque temps.

Une mélodie :

```forth
0 0.25 0.5 0.75 at
c4 e4 g4 c5 arp note sine snd .
```

Quatre notes, réparties dans la frame. Remplacez `sine` par `saw`, réévaluez.
Le changement est immédiat.

## La grille

La grille de scène est votre espace de travail. Chaque colonne est une **ligne**
(lecture en parallèle). Chaque rangée dans une colonne est une **frame** (lecture
en séquence). Une ligne boucle sur ses frames de haut en bas, puis recommence.

Vous pouvez avoir un pattern de batterie en ligne 1, une basse en ligne 2 et des
accords en ligne 3, le tout en même temps. Voir l'article **La Grille** pour la
navigation et l'édition.

## Quatre langages

Chaque frame a son propre langage. Choisissez celui qui correspond à ce que vous
faites.

**Cagire** -- à pile, style Forth. Empilez des valeurs, appliquez des mots,
émettez avec `.`. Idéal pour le sound design et l'expérimentation rapide.

```forth
c4 min7 arp note 0.5 decay 0.4 verb sine snd .
```

**Bob** -- impératif, notation polonaise. Event maps, boucles, timing explicite
avec WAIT. Idéal pour les séquences mélodiques précises.

```
RANGE 0 3 :
  >> [note: ADD 60 MUL I 4 vel: 100]
  WAIT 0.25
END
```

**BaLi** -- style Lisp, à base d'expressions. S-expressions imbriquées, les
boucles et transformations se composent naturellement. Idéal pour la composition
algorithmique et générative.

```
(loop 4
  (note (+ 60 (* $i 3)) 90)
  1//4)
```

**Boinx** -- notation de patterns déclarative. Séquences et simultanéité
exprimées visuellement par des crochets. Idéal pour les patterns rythmiques
lisibles d'un coup d'oeil.

```
<s: 'kick'> | [. _ . _]
```

Pour changer le langage d'une frame, ouvrez l'éditeur et sélectionnez dans le
menu déroulant en haut, ou appuyez sur Cmd+L (Ctrl+L). Voir l'article
**Langages** pour plus de détails, et cliquez sur les onglets de chaque langage
(Bob, bali, Boinx, Cagire) pour les références complètes.

## Timing

Chaque frame a une durée en temps (beats). Le séquenceur joue le script de la
frame une fois par durée, puis passe à la suivante. Une frame d'un temps à
120 BPM s'exécute toutes les demi-secondes.

À l'intérieur d'une frame, on peut subdiviser le temps. En Cagire, `at`
place des sons à des positions fractionnaires dans le temps. En Bob, `WAIT` fait
avancer l'horloge explicitement.

Réglez le tempo et le quantum dans la barre de transport en haut. Voir l'article
**Timing**.

## Vos instruments

Sova envoie les événements vers des slots numérotés de 1 à 16. Ouvrez le panneau
Périphériques pour connecter des ports MIDI, créer des endpoints OSC ou activer
le moteur audio intégré (Doux). Le slot 1 est celui par défaut. Dans votre code,
utilisez `dev` pour cibler un slot :

```forth
2 dev c4 note 100 vel .
```

Voir l'article **Périphériques**.

## Jouer ensemble

Lancez un serveur. Les autres musiciens se connectent à votre adresse IP et
port. Tout le monde voit la même scène, édite en temps réel et reste synchronisé
via Ableton Link. Utilisez des lignes différentes pour éviter de vous marcher
dessus. Le chat est intégré. Voir l'article **Multijoueur**.

## Visuels

Sova intègre un moteur de shaders inspiré de Hydra. Écrivez des pipelines
visuels -- oscillateurs, bruit, kaléidoscopes, boucles de feedback -- et ils
s'affichent derrière l'interface en temps réel.

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

Appuyez sur F1 pour voir tous les raccourcis. La palette de commandes (Cmd+K)
liste toutes les actions avec leur raccourci.
