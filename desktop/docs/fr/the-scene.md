# La scène

La scène représente votre session de performance. Elle contient l'ensemble des
éléments en cours de lecture : pistes parallèles, code de chaque slot, timing.
Lorsque vous jouez avec Sova, vous éditez une scène en temps réel.

Sova peut être vu comme un séquenceur pas-à-pas dont le comportement de chaque
pas est défini par du code. Contrairement aux séquenceurs conventionnels, la
durée de chaque pas est libre : un pas peut durer une fraction de temps ou une
mesure entière. Les scripts se modifient en temps réel, permettant des
performances dynamiques et spontanées.

## Structure

Une scène contient des **lignes** et des **frames**. Les lignes sont les
colonnes de la grille. Elles tournent en parallèle, chacune produisant son
propre flux d'événements. Au sein d'une ligne, les **frames** se jouent en
séquence. Quand la durée d'une frame s'écoule, la suivante démarre.

Une ligne fait tourner un motif de kick. Une autre joue la basse. Une troisième
envoie de l'OSC vers un synthétiseur visuel. Chacune avance à son propre rythme,
indépendamment.

## Frames

Chaque frame contient un script (votre code) et quelques propriétés :

- Durée (beats) : la durée de la frame. Par défaut, 1 beat. Les valeurs
  fractionnaires sont acceptées : 0.25 pour une subdivision en doubles-croches,
  4 pour une mesure entière en 4/4.
- Répétitions : le nombre de fois que le script s'exécute dans cette durée. Une
  frame de durée 4 avec 4 répétitions exécute son script une fois par beat. Avec
  8 répétitions, le script se déclenche sur chaque croche.
- Activée : active ou désactive la frame. Les frames désactivées sont ignorées.
  Utile pour couper une section en pleine performance sans perdre le code.
- Nom : label optionnel affiché sur la cellule de la grille.
- Script : le code, associé au langage utilisé (Bob, Boinx, Cagire ou BaLi).

Le temps total occupé par une frame est `durée × répétitions`. Une frame de
durée 0.5 avec 8 répétitions occupe 4 beats.

## Lignes

Les lignes disposent de leurs propres contrôles :

- Boucle : lorsqu'elle est activée, la ligne reprend du début après sa dernière
  frame. Sinon, elle se joue une fois et s'arrête.
- Trailing : lorsqu'il est activé, les événements des frames précédentes
  continuent de sonner pendant que la frame suivante commence. Sinon, ils sont
  coupés.
- Vitesse : multiplicateur sur le tempo de la ligne. 2.0 pour le double, 0.5
  pour la moitié. Une ligne à vitesse normale, une autre à mi-vitesse — les
  structures polymétriques naissent naturellement.
- Frame de début / Frame de fin : restreint la lecture à une plage au sein de la
  ligne. En performance, on réduit la plage pour boucler une section spécifique
  pendant que l'on édite la suite.

## Modes d'exécution

Le mode d'exécution contrôle la synchronisation des lignes au démarrage ou au
redémarrage de la scène. On le modifie depuis la barre de transport.

**Free** est le mode par défaut. Les lignes démarrent immédiatement et bouclent
à leur propre rythme. Chaque ligne est indépendante. Adapté au jam, à
l'empilement de motifs qui dérivent les uns par rapport aux autres, à la
construction de textures.

**AtQuantum** fait attendre les lignes jusqu'à la prochaine limite de quantum
(début de mesure) avant de démarrer. Tout se cale sur la phrase globale. Pour
les arrangements serrés où chaque partie doit tomber sur le temps fort.

**LongestLine** attend que la ligne la plus longue termine son cycle complet
avant que quoi que ce soit ne redémarre. Toutes les lignes se réinitialisent
ensemble. La scène devient une grille de boucles synchronisées — utile lorsque
toutes les parties doivent cycler comme un seul bloc.

## Sauvegarde et chargement

On sauvegarde et charge les scènes via le menu de scène. Le fichier capture
l'intégralité : lignes, frames, scripts, variables et configuration. Lorsque
vous vous connectez à un serveur, vous recevez sa scène courante
automatiquement.

Voir **La grille** pour la navigation et les raccourcis d'édition.
