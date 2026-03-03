# Langages

Sova est polyglotte — chaque frame peut utiliser un langage de programmation
différent. Les quatre langages intégrés offrent des approches distinctes de
l'expression musicale. Choisissez celui qui correspond à votre façon de penser
la musique, ou mélangez-les librement.

## Compilé vs interprété

Les langages de Sova se divisent en deux catégories :

- **Langages compilés** (Bob, BaLi) sont traduits en bytecode pour la machine
  virtuelle de Sova. La VM exécute le bytecode à chaque lecture de la frame. La
  compilation a lieu une seule fois à l'évaluation ; l'exécution est rapide et
  reproductible.
- **Langages interprétés** (Boinx, Cagire) produisent directement une liste
  d'événements à partir du code source à chaque lecture de la frame. Il n'y a
  pas d'étape intermédiaire de bytecode.

Du point de vue utilisateur, les deux fonctionnent de la même manière : écrire
du code, évaluer, écouter le résultat. La différence compte quand vous voulez
comprendre comment votre code interagit avec les variables, le timing et les
répétitions.

## Vue d'ensemble

- **Bob** — Compilé. Impératif, event maps. Idéal pour séquences mélodiques, contrôle précis.
- **BaLi** — Compilé. À base d'expressions, fonctionnel. Idéal pour patterns algorithmiques, approche mathématique.
- **Boinx** — Interprété. Notation de patterns. Idéal pour patterns rythmiques rapides.
- **Cagire** — Interprété. À pile (style Forth). Idéal pour synthèse audio, DSP, expérimentation.

## Bob

Bob est un langage impératif avec une syntaxe concise pour générer des
événements MIDI et OSC. Il utilise des **event maps** — des structures
clé-valeur qui décrivent les notes, les changements de contrôle et d'autres
messages. Bob dispose de variables, de conditionnelles, de boucles et de
fonctions.

```
>> [note: 60 vel: 100 dur: 0.5]
WAIT 0.5
>> [note: 64 vel: 80 dur: 0.5]
```

Consultez l'onglet **Bob** pour la référence complète.

## BaLi

BaLi est un langage compilé à base d'expressions avec une saveur fonctionnelle.
Il met l'accent sur la composition de transformations et convient bien aux
patterns algorithmiques et génératifs.

Consultez l'onglet **BaLi** pour la référence complète.

## Boinx

Boinx est un langage de notation de patterns — sa syntaxe est conçue pour
écrire rapidement des séquences rythmiques. Les patterns décrivent quand les
événements se déclenchent au sein d'un beat ou d'une mesure, ce qui le rend
naturel pour les patterns de batterie et les séquences percussives.

Consultez l'onglet **Boinx** pour la référence complète.

## Cagire

Cagire est un langage à pile inspiré de Forth. Vous empilez des valeurs sur une
pile et appliquez des mots (opérations) dessus. Cagire est étroitement intégré
au moteur audio Doux pour la synthèse sonore en temps réel et le DSP, mais il
fonctionne aussi pour la sortie MIDI et OSC.

Consultez l'onglet **Cagire** pour la référence complète.

## Changer de langage

Chaque frame a son propre réglage de langage. Pour changer le langage d'une frame :

1. Ouvrez l'éditeur de code (double-clic sur une cellule de frame).
2. Sélectionnez le langage dans le menu déroulant en haut de l'éditeur.
3. Écrivez ou réécrivez votre code dans le nouveau langage.
4. Évaluez.

Différentes frames dans la même ligne peuvent utiliser des langages différents —
Sova s'en accommode parfaitement. Une ligne peut avoir une frame Bob générant des
mélodies suivie d'une frame Boinx pour un break de batterie. Mélangez et
combinez comme bon vous semble.
