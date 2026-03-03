# Timing

Tout dans Sova fonctionne sur une horloge partagée. Le tempo, les beats et
la synchronisation sont gérés par Ableton Link, qui maintient toutes les
applications et tous les appareils connectés alignés sur la même timeline.

## Beats et tempo

Sova mesure le temps en **beats**. Un beat est une pulsation musicale
dont la durée réelle dépend du tempo (BPM). À 120 BPM, un beat dure
500 millisecondes. À 60 BPM, un beat dure une seconde entière.

Les durées des frames, les commandes d'attente et les longueurs de notes sont
toutes exprimées en beats. Cela signifie que vos patterns accélèrent ou
ralentissent automatiquement quand le tempo change — vous n'avez rien à
recalculer.

## Ableton Link

Ableton Link est un protocole de synchronisation du tempo, des beats et de
la phase entre applications et appareils sur le même réseau. Sova utilise Link
comme horloge maîtresse.

Quand vous changez le tempo dans Sova, toute application compatible Link
(Ableton Live, d'autres instances de Sova, des applications mobiles, etc.) voit
le changement instantanément. Inversement, si une autre application change le
tempo, Sova suit.

Link synchronise aussi l'état **lecture/arrêt**. Quand vous lancez la lecture
dans Sova, les autres pairs Link peuvent démarrer aussi (s'ils activent la
synchronisation lecture/arrêt).

Vous n'avez pas besoin de configurer Link — il fonctionne automatiquement sur
votre réseau local. Si aucun autre pair Link n'est présent, Sova utilise
simplement sa propre horloge.

## Quantum et phase

Le **quantum** définit combien de beats composent une phrase ou une mesure.
Avec un quantum de 4 (la valeur par défaut), la timeline est divisée en groupes
de 4 beats. La **phase** indique où vous vous trouvez dans le quantum
courant — beat 0, 1, 2 ou 3.

Le quantum est important pour la synchronisation :

- En mode d'exécution **AtQuantum**, les lignes attendent la prochaine limite de
  quantum (le prochain « beat 0 » d'une mesure) avant de démarrer.
- Vous pouvez planifier des événements pour qu'ils se déclenchent à la prochaine
  réinitialisation de phase grâce aux contrôles de timing dans votre code.

Changer le quantum ne change pas le tempo — cela change la façon dont la grille
de beats est regroupée.

## La barre de transport

La barre de transport en haut de l'écran affiche :

- **Lecture / Arrêt** — démarrer ou arrêter la lecture. Synchronisé via Link.
- **Tempo** (BPM) — cliquez pour modifier. Minimum 20 BPM. Partagé entre tous
  les pairs Link.
- **Quantum** — la valeur de beats par phrase.
- **Compteur de beats** — la position actuelle en beats et en phase.

## Timing dans le code

Vos scripts peuvent contrôler quand les événements se produisent au sein d'une
frame :

- **Attente** — mettre l'exécution en pause pendant un nombre de beats
  avant de continuer. C'est ainsi que vous espacez les événements dans le temps.
- **Durée de frame** — le temps total de lecture d'une frame. Une frame avec une
  durée de 2 donne à votre script 2 beats à remplir d'événements.
- **Répétitions** — combien de fois le script s'exécute durant la durée de la
  frame. Une durée de 4 avec 4 répétitions signifie que le script s'exécute
  4 fois, une fois par beat.

La syntaxe exacte pour les attentes et le timing varie selon le langage —
consultez la référence de chaque langage pour les détails.

## Garanties de timing

L'architecture à deux threads de Sova est conçue pour un timing précis :

- Le **planificateur** s'exécute ~30 ms en avance sur le temps réel, compilant
  et préparant les événements à l'avance.
- Le **thread monde** s'exécute en priorité temps réel, envoyant les événements
  vers MIDI (2 ms d'anticipation) et OSC (20 ms d'anticipation) avec une
  précision inférieure à la milliseconde.

Cela signifie que vos événements arrivent à temps même sous charge CPU, tant que
le planificateur peut suivre.

## Astuces

- Durée de frame × répétitions = durée totale de la frame. Utilisez les
  répétitions pour créer des subdivisions rythmiques sans écrire de boucles
  explicites.
- La vitesse d'une ligne multiplie son tempo par rapport au BPM global. Une
  ligne à vitesse 2.0 joue en double-temps ; 0.5 joue à mi-vitesse.
- Utilisez le mode d'exécution **AtQuantum** quand vous voulez que toutes les
  lignes restent alignées sur la phrase après des modifications. Utilisez
  **Free** quand vous voulez une indépendance polyrythmique.
