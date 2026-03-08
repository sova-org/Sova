# La Scène

Une scène, c'est votre session live. Elle contient tout ce qui joue en ce
moment : les pistes parallèles, le code dans chaque slot, le timing. Quand vous
performez avec Sova, vous éditez une scène en temps réel.

## Structure

Une scène contient des **lignes** et des **frames**. Les lignes sont les
colonnes de la grille. Elles tournent en parallèle, chacune produisant son
propre flux d'événements. Au sein d'une ligne, les **frames** se jouent en
séquence. Quand la durée d'une frame s'écoule, la suivante démarre.

Une ligne fait tourner un pattern de kick. Une autre joue la basse. Une
troisième envoie de l'OSC vers un synthé visuel. Chacune avance à son propre
rythme, indépendamment.

## Frames

Chaque frame contient un script (votre code) et quelques propriétés :

- Durée (beats) : combien de temps la frame dure. Par défaut, 1 beat. Les
  valeurs fractionnaires fonctionnent : 0.25 pour un feeling de double-croche,
  4 pour une mesure entière en 4/4.
- Répétitions : combien de fois le script s'exécute dans cette durée. Une frame
  de durée 4 avec 4 répétitions exécute son script une fois par beat. Avec 8
  répétitions, le script se déclenche sur chaque croche.
- Activée : active ou désactive la frame. Les frames désactivées sont sautées.
  Pratique pour couper une section en pleine performance sans perdre le code.
- Nom : label optionnel affiché sur la cellule de la grille. Utilisez-le.
- Script : le code, plus le langage utilisé (Bob, Boinx, Cagire ou BaLi).

Le temps total occupé par une frame est `durée * répétitions`. Une frame de
durée 0.5 avec 8 répétitions prend 4 beats.

## Lignes

Les lignes ont leurs propres contrôles :

- Boucle : quand elle est activée, la ligne reprend du début après sa dernière
  frame. Sinon, elle se joue une fois et s'arrête.
- Trailing : quand il est activé, les événements des frames précédentes
  continuent de sonner pendant que la frame suivante commence. Sinon, ils sont
  coupés.
- Vitesse : multiplicateur sur le tempo de la ligne. 2.0 pour jouer en double,
  0.5 pour la moitié. Une ligne à vitesse normale, une autre à mi-vitesse --
  les structures polymétriques viennent naturellement.
- Frame de début / Frame de fin : restreint la lecture à une plage au sein de la
  ligne. En performance, réduisez la plage pour boucler une section spécifique
  pendant que vous éditez ce qui suit.

## Modes d'exécution

Le mode d'exécution contrôle comment les lignes se synchronisent quand la scène
démarre ou redémarre. On le change depuis la barre de transport.

**Free** est le mode par défaut. Les lignes démarrent immédiatement et bouclent
à leur propre rythme. Chaque ligne est indépendante. Adapté au jam, à
l'empilement de patterns qui dérivent les uns par rapport aux autres, à la
construction de textures.

**AtQuantum** fait attendre les lignes jusqu'à la prochaine limite de quantum
(début de mesure) avant de démarrer. Tout se cale sur la phrase globale. Pour
les arrangements serrés où chaque partie doit tomber sur le temps fort.

**LongestLine** attend que la ligne la plus longue termine son cycle complet
avant que quoi que ce soit ne redémarre. Toutes les lignes se réinitialisent
ensemble. La scène devient une grille de boucles synchronisées -- utile quand
vous voulez que toutes vos parties cyclent comme un seul bloc.

## Sauvegarde et chargement

Sauvegardez et chargez des scènes via le menu de scène. Le fichier capture
tout : lignes, frames, scripts, variables et configuration. Quand vous vous
connectez à un serveur, vous recevez sa scène courante automatiquement.

Voir **La Grille** pour la navigation et les raccourcis d'édition.
