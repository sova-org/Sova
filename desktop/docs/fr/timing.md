# Timing

Sova mesure le temps en beats. Un beat à 120 BPM dure 500ms. À 60 BPM, une
seconde entière. Durées de frames, attentes, longueurs de notes : tout est
en beats. Changez le tempo, vos patterns suivent -- rien à recalculer.

## Tempo et synchronisation

L'horloge tourne sur Ableton Link. Toutes les applications Link du réseau
partagent le même tempo et la même position. Changez le BPM dans Sova,
Ableton Live le voit. Changez-le dans Live, Sova suit. Si rien d'autre
n'est sur le réseau, Sova tourne sur sa propre horloge. Aucune configuration
nécessaire.

Link partage aussi l'état lecture/arrêt. Lancez la lecture dans Sova, les
autres pairs Link peuvent démarrer avec vous.

## Mesures et phrases

Le **quantum** définit combien de beats forment une mesure. Par défaut, 4 --
une mesure 4/4 standard. La **phase** indique où vous êtes dans cette
mesure : beat 0, 1, 2 ou 3.

C'est important pour le lancement des lignes. En mode **AtQuantum**, les
lignes attendent le premier temps (phase 0) avant de démarrer. Vous éditez
du code en pleine mesure, le changement tombe sur le prochain "un". En mode
**Free**, les lignes démarrent immédiatement -- idéal pour l'indépendance
polyrythmique.

## La barre de transport

En haut de l'écran : lecture/arrêt, BPM, quantum, position actuelle en
beats. Cliquez sur le BPM pour taper une nouvelle valeur. Minimum 20 BPM.

## Espacer les événements dans le code

Sans timing explicite, tous les événements d'un script se déclenchent en
même temps -- au beat zéro de la frame. On les espace avec des attentes.

En Cagire, `at` définit des offsets de timing en fractions de la durée de
frame :

```forth
0 0.5 at kick snd .       ;; kick au début et à la moitié
0 0.25 0.5 0.75 at hat snd .  ;; quatre hats, espacés régulièrement
```

En Bob, `WAIT` avance le temps en beats :

```
>> [note: 60 vel: 100]
WAIT 0.5
>> [note: 64 vel: 80]
WAIT 0.5
>> [note: 67 vel: 100]
```

## Frames, durée et répétitions

Chaque frame a une durée en beats. Une frame de 2 beats donne à votre script
2 beats à remplir d'événements.

Les répétitions subdivisent cette durée. Une frame de 4 beats avec 4
répétitions exécute le script 4 fois, une fois par beat. Ça crée des boucles
rythmiques sans code de boucle explicite :

```
-- Bob : un kick par beat pendant 4 beats (durée frame=4, reps=4)
>> [note: 36 vel: 100]
```

```forth
;; Cagire : même idée
36 note 100 vel .
```

Une ligne de code, quatre kicks. Le séquenceur gère la répétition.

## Vitesse de ligne

Le facteur de vitesse d'une ligne multiplie le tempo par rapport au BPM
global. À 2.0, double-temps. À 0.5, mi-temps. Combinez avec des valeurs
de quantum différentes entre les lignes pour des structures polymétriques.

## Modes d'exécution

Trois modes contrôlent comment les lignes démarrent après une modification
ou un changement de scène :

- **Free** -- les lignes démarrent immédiatement. Timing indépendant.
- **AtQuantum** -- les lignes attendent le prochain premier temps de mesure.
  Tout reste aligné sur la phrase.
- **LongestLine** -- attend que la ligne la plus longue en cours finisse son
  cycle avant de redémarrer.

Choisissez **AtQuantum** pour des arrangements serrés. Choisissez **Free**
quand vous voulez que les choses dérivent et se superposent.
