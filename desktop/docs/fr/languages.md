# Langages

Sova est un environnement *polyglotte*. Plusieurs langages coexistent au sein
de la même machine virtuelle, partageant le même ordonnanceur et les mêmes
entrées-sorties. Chaque langage suit son propre paradigme et expose des
abstractions différentes. Cette diversité encourage l'expérimentation : on
choisit le langage qui correspond le mieux à l'idée musicale du moment.

Quatre langages sont intégrés. Chaque frame en choisit un. On peut les mélanger
librement au sein d'une même ligne — une mélodie Bob suivie d'un drone Cagire,
puis d'une transition Boinx.

## Cagire

À pile, inspiré de Forth. On empile des valeurs et on applique des mots qui
consomment et produisent des valeurs sur la pile. `.` émet la commande sonore
courante.

Un kick :

```forth
kick snd .
```

Un accord avec reverb :

```forth
c4 min7 note 0.4 verb sine snd .
```

Un motif rythmique avec distribution euclidienne :

```forth
3 8 euclid at hat snd .
```

Cagire intègre de la théorie musicale — notes, intervalles, accords, gammes —
ainsi que de l'aléatoire, du cycling, des variables et des définitions
utilisateur. Voir l'onglet **Cagire**.

## Bob

Impératif, notation polonaise. Les opérateurs précèdent les opérandes :
`ADD 2 3` au lieu de `2 + 3`. Les événements sont des maps clé-valeur émis avec
`>>`. Le temps avance avec `WAIT`.

Une séquence de quatre notes :

```
RANGE 0 3 :
  >> [note: ADD 60 MUL I 4 vel: 100]
  WAIT 0.25
END
```

Rythme euclidien avec notes fantômes :

```
EU 3 8 0.125 :
  >> [note: 36 vel: 100]
ELSE :
  >> [note: 36 vel: 20]
END
```

Sélection aléatoire dans une liste :

```
SET G.NOTES '[60 64 67 72]
>> [note: PICK G.NOTES vel: RRAND 60 127]
```

Bob dispose de variables (globales, frame, ligne), de conditionnelles, de
boucles, de fonctions et de générateurs de rythme euclidien/binaire. Voir
l'onglet **Bob**.

## BaLi

Style Lisp, à base d'expressions. Tout est une S-expression entre parenthèses.
Boucles, notes et effets se composent par imbrication. Les fractions comme
`1//4` expriment les durées directement.

Une séquence de notes en boucle :

```
(loop 4
  (note (+ 60 (* $i 3)) 90)
  1//4)
```

Un accord sur le temps :

```
(note 60 100 dev:1 ch:1)
(note 64 100 dev:1 ch:1)
(note 67 100 dev:1 ch:1)
```

Rythme euclidien :

```
(eucloop 3 8
  (note 36 100)
  1//8)
```

Le style fonctionnel de BaLi le rend naturel pour la composition algorithmique
et les motifs génératifs. Voir l'onglet **BaLi**.

## Boinx

Notation de motifs déclarative. On décrit *quoi* joue *où* dans le temps avec
des crochets et des opérateurs. Les séquences `[...]` répartissent les éléments
régulièrement dans la frame. Les événements simultanés utilisent `(...)`. Les
données d'événement clé-valeur vont dans `<...>`.

Un motif kick-hat :

```
<s: 'kick'> | [. _ . _]
```

Batterie en couches avec kick et hat simultanés :

```
(<s: 'kick'> <s: 'hat'>) | [. _ . _]
```

Notes cycliques sur une grille rythmique :

```
(C4 E4 G4) ° [. . . .]
```

Les opérateurs Boinx (`|`, `°`, `~`, `!`, `#`) contrôlent la circulation des
données d'événement dans les emplacements du motif. La disposition visuelle du
code reflète la structure rythmique. Voir l'onglet **Boinx**.

## Mélanger les langages

Une seule ligne peut contenir des frames dans différents langages. La frame 1
peut être un drone Cagire, la frame 2 une mélodie Bob, la frame 3 un roulement
Boinx. Le séquenceur les joue dans l'ordre quel que soit le langage. Pour
changer le langage d'une frame, ouvrez l'éditeur et choisissez dans le menu
déroulant en haut.
