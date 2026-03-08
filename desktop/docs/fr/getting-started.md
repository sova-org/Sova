Sova est un sequenceur de live coding. Vous ecrivez du code, Sova le transforme
en notes MIDI, messages OSC et audio -- le tout synchronise sur une horloge
partagee via Ableton Link. Quatre langages integres, chacun avec sa propre facon
de penser la musique. Plusieurs musiciens peuvent se connecter et jouer ensemble
sur la meme scene.

## Connexion

Ouvrez le panneau Serveur et cliquez sur Demarrer. L'application lance un
serveur local et s'y connecte. C'est tout.

## Premier son

Double-cliquez sur une cellule de frame dans la grille pour ouvrir l'editeur. Le
langage par defaut est Cagire. Tapez :

```forth
kick snd .
```

Appuyez sur Cmd+Entree (Ctrl+Entree sous Linux/Windows) pour evaluer. Un kick
retentit a chaque temps.

Une melodie :

```forth
0 0.25 0.5 0.75 at
c4 e4 g4 c5 arp note sine snd .
```

Quatre notes, reparties dans la frame. Remplacez `sine` par `saw`, re-evaluez.
Le changement est immediat.

## La grille

La grille de scene est votre espace de travail. Chaque colonne est une **ligne**
(lecture en parallele). Chaque rangee dans une colonne est une **frame** (lecture
en sequence). Une ligne boucle sur ses frames de haut en bas, puis recommence.

Vous pouvez avoir un pattern de batterie en ligne 1, une basse en ligne 2 et des
accords en ligne 3, le tout en meme temps. Voir l'article **La Grille** pour la
navigation et l'edition.

## Quatre langages

Chaque frame a son propre langage. Choisissez celui qui correspond a ce que vous
faites.

**Cagire** -- a pile, style Forth. Empilez des valeurs, appliquez des mots,
emettez avec `.`. Ideal pour le sound design et l'experimentation rapide.

```forth
c4 min7 arp note 0.5 decay 0.4 verb sine snd .
```

**Bob** -- imperatif, notation polonaise. Event maps, boucles, timing explicite
avec WAIT. Ideal pour les sequences melodiques precises.

```
RANGE 0 3 :
  >> [note: ADD 60 MUL I 4 vel: 100]
  WAIT 0.25
END
```

**BaLi** -- style Lisp, a base d'expressions. S-expressions imbriquees, les
boucles et transformations se composent naturellement. Ideal pour la composition
algorithmique et generative.

```
(loop 4
  (note (+ 60 (* $i 3)) 90)
  1//4)
```

**Boinx** -- notation de patterns declarative. Sequences et simultaneite
exprimees visuellement par des crochets. Ideal pour les patterns rythmiques
lisibles d'un coup d'oeil.

```
<s: 'kick'> | [. _ . _]
```

Pour changer le langage d'une frame, ouvrez l'editeur et selectionnez dans le
menu deroulant en haut, ou appuyez sur Cmd+L (Ctrl+L). Voir l'article
**Langages** pour plus de details, et cliquez sur les onglets de chaque langage
(Bob, bali, Boinx, Cagire) pour les references completes.

## Timing

Chaque frame a une duree en temps (beats). Le sequenceur joue le script de la
frame une fois par duree, puis passe a la frame suivante. Une frame d'un temps a
120 BPM s'execute toutes les demi-secondes.

A l'interieur d'une frame, vous pouvez subdiviser le temps. En Cagire, `at`
place des sons a des positions fractionnaires dans le temps. En Bob, `WAIT` fait
avancer l'horloge explicitement.

Reglez le tempo et le quantum dans la barre de transport en haut. Voir l'article
**Timing**.

## Vos instruments

Sova envoie les evenements vers des slots numerotes de 1 a 16. Ouvrez le panneau
Peripheriques pour connecter des ports MIDI, creer des endpoints OSC ou activer
le moteur audio integre (Doux). Le slot 1 est celui par defaut. Dans votre code,
utilisez `dev` pour cibler un slot :

```forth
2 dev c4 note 100 vel .
```

Voir l'article **Peripheriques**.

## Jouer ensemble

Lancez un serveur. Les autres musiciens se connectent a votre adresse IP et
port. Tout le monde voit la meme scene, edite en temps reel et reste synchronise
via Ableton Link. Utilisez des lignes differentes pour eviter de vous marcher
dessus. Le chat est integre. Voir l'article **Multijoueur**.

## Visuels

Sova integre un moteur de shaders inspire de Hydra. Ecrivez des pipelines
visuels -- oscillateurs, bruit, kaleidoscopes, boucles de feedback -- et ils
s'affichent derriere l'interface en temps reel.

```
osc(60, 0.1).rotate(0, 0.1).kaleid(4).out()
```

Voir l'article **Visuels (Hydra)**.

## Raccourcis

| Action | macOS | Linux/Windows |
|--------|-------|---------------|
| Evaluer le code | Cmd+Entree | Ctrl+Entree |
| Palette de commandes | Cmd+K | Ctrl+K |
| Lecture / Arret | Cmd+Shift+Espace | Ctrl+Shift+Espace |
| Sauvegarder la scene | Cmd+S | Ctrl+S |
| Charger une scene | Cmd+O | Ctrl+O |
| Panneau Serveur | Cmd+Shift+S | Ctrl+Shift+S |
| Panneau Peripheriques | Cmd+Shift+I | Ctrl+Shift+I |
| Documentation | Cmd+Shift+H | Ctrl+Shift+H |
| Visuels | Cmd+Shift+V | Ctrl+Shift+V |
| Changer de langage | Cmd+L | Ctrl+L |

Appuyez sur F1 pour voir tous les raccourcis. La palette de commandes (Cmd+K)
liste toutes les actions avec leur raccourci.
