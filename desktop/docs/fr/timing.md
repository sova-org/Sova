# Timing

Sova mesure le temps en beats. Un beat a 120 BPM dure 500ms. A 60 BPM, une
seconde entiere. Durees de frames, attentes, longueurs de notes : tout est
en beats. Changez le tempo, vos patterns suivent -- rien a recalculer.

## Tempo et synchronisation

L'horloge tourne sur Ableton Link. Toutes les applications Link du reseau
partagent le meme tempo et la meme position. Changez le BPM dans Sova,
Ableton Live le voit. Changez-le dans Live, Sova suit. Si rien d'autre
n'est sur le reseau, Sova tourne sur sa propre horloge. Aucune configuration
necessaire.

Link partage aussi l'etat lecture/arret. Lancez la lecture dans Sova, les
autres pairs Link peuvent demarrer avec vous.

## Mesures et phrases

Le **quantum** definit combien de beats forment une mesure. Par defaut, 4 --
une mesure 4/4 standard. La **phase** indique ou vous etes dans cette
mesure : beat 0, 1, 2 ou 3.

C'est important pour le lancement des lignes. En mode **AtQuantum**, les
lignes attendent le premier temps (phase 0) avant de demarrer. Vous editez
du code en pleine mesure, le changement tombe sur le prochain "un". En mode
**Free**, les lignes demarrent immediatement -- utile pour l'independance
polyrythmique.

## La barre de transport

En haut de l'ecran : lecture/arret, BPM, quantum, position actuelle en
beats. Cliquez sur le BPM pour taper une nouvelle valeur. Minimum 20 BPM.

## Espacer les evenements dans le code

Sans timing explicite, tous les evenements d'un script se declenchent en
meme temps -- au beat zero de la frame. On les espace avec des attentes.

En Cagire, `at` definit des offsets de timing en fractions de la duree de
frame :

```forth
0 0.5 at kick snd .       ;; kick au debut et a la moitie
0 0.25 0.5 0.75 at hat snd .  ;; quatre hats, espaces regulierement
```

En Bob, `WAIT` avance le temps en beats :

```
>> [note: 60 vel: 100]
WAIT 0.5
>> [note: 64 vel: 80]
WAIT 0.5
>> [note: 67 vel: 100]
```

## Frames, duree et repetitions

Chaque frame a une duree en beats. Une frame de 2 beats donne a votre script
2 beats a remplir d'evenements.

Les repetitions subdivisent cette duree. Une frame de 4 beats avec 4
repetitions execute le script 4 fois, une fois par beat. Ca cree des boucles
rythmiques sans code de boucle explicite :

```
-- Bob : un kick par beat pendant 4 beats (duree frame=4, reps=4)
>> [note: 36 vel: 100]
```

```forth
;; Cagire : meme idee
36 note 100 vel .
```

Une ligne de code, quatre kicks. Le sequenceur gere la repetition.

## Vitesse de ligne

Le facteur de vitesse d'une ligne multiplie le tempo par rapport au BPM
global. A 2.0, double-temps. A 0.5, mi-temps. Combinez avec des valeurs
de quantum differentes entre les lignes pour des structures polymetriques.

## Modes d'execution

Trois modes controlent comment les lignes demarrent apres une modification
ou un changement de scene :

- **Free** -- les lignes demarrent immediatement. Timing independant.
- **AtQuantum** -- les lignes attendent le prochain premier temps de mesure.
  Tout reste aligne sur la phrase.
- **LongestLine** -- attend que la ligne la plus longue en cours finisse son
  cycle avant de redemarrer.

Choisissez **AtQuantum** pour des arrangements serres. Choisissez **Free**
quand vous voulez que les choses derivent et se superposent.
