# Langages

Sova dispose de quatre langages integres. Chaque frame en choisit un. Vous
pouvez les melanger librement au sein d'une meme ligne -- une melodie Bob suivie
d'un drone Cagire suivie d'un break Boinx.

## Cagire

A pile, inspire de Forth. Vous empilez des valeurs et appliquez des mots qui
consomment et produisent des valeurs sur la pile. `.` emet la commande sonore
courante.

Un kick :

```forth
kick snd .
```

Un accord avec reverb :

```forth
c4 min7 note 0.4 verb sine snd .
```

Un pattern rythmique avec distribution euclidienne :

```forth
3 8 euclid at hat snd .
```

Cagire integre de la theorie musicale -- notes, intervalles, accords, gammes --
plus de l'aleatoire, du cycling, des variables et des definitions utilisateur.
Voir l'onglet **Cagire**.

## Bob

Imperatif, notation polonaise. Les operateurs precedent les operandes :
`ADD 2 3` au lieu de `2 + 3`. Les evenements sont des maps cle-valeur emis avec
`>>`. Le temps avance avec `WAIT`.

Une sequence de quatre notes :

```
RANGE 0 3 :
  >> [note: ADD 60 MUL I 4 vel: 100]
  WAIT 0.25
END
```

Rythme euclidien avec notes fantomes :

```
EU 3 8 0.125 :
  >> [note: 36 vel: 100]
ELSE :
  >> [note: 36 vel: 20]
END
```

Selection aleatoire dans une liste :

```
SET G.NOTES '[60 64 67 72]
>> [note: PICK G.NOTES vel: RRAND 60 127]
```

Bob a des variables (globales, frame, ligne), des conditionnelles, des boucles,
des fonctions et des generateurs de rythme euclidien/binaire. Voir l'onglet
**Bob**.

## BaLi

Style Lisp, a base d'expressions. Tout est une S-expression entre parentheses.
Boucles, notes et effets se composent par imbrication. Les fractions comme
`1//4` expriment les durees directement.

Une sequence de notes en boucle :

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
et les patterns generatifs. Voir l'onglet **bali**.

## Boinx

Notation de patterns declarative. Vous decrivez *quoi* joue *ou* dans le temps
avec des crochets et des operateurs. Les sequences `[...]` repartissent les
elements regulierement dans la frame. Les evenements simultanes utilisent
`(...)`. Les donnees d'evenement cle-valeur vont dans `<...>`.

Un pattern kick-hat :

```
<s: 'kick'> | [. _ . _]
```

Batterie en couches avec kick et hat simultanes :

```
(<s: 'kick'> <s: 'hat'>) | [. _ . _]
```

Notes cycliques sur une grille rythmique :

```
(C4 E4 G4) ° [. . . .]
```

Les operateurs Boinx (`|`, `°`, `~`, `!`, `#`) controlent comment les donnees
d'evenement circulent dans les slots du pattern. La disposition visuelle du code
reflete la structure rythmique. Voir l'onglet **Boinx**.

## Melanger les langages

Une seule ligne peut contenir des frames dans differents langages. La frame 1
peut etre un drone Cagire, la frame 2 une melodie Bob, la frame 3 un fill Boinx.
Le sequenceur les joue dans l'ordre quel que soit le langage. Pour changer le
langage d'une frame, ouvrez l'editeur et choisissez dans le menu deroulant en
haut.
