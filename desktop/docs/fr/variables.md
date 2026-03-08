# Variables

Les variables stockent des valeurs qui persistent entre les evenements ou
entre les frames. On les utilise pour coordonner les scripts, accumuler de
l'etat, et construire des patterns evolutifs.

## Portees par l'exemple

Quatre portees. La portee determine qui voit la variable et combien de temps
elle vit.

**Instance** -- memoire locale. Se reinitialise a chaque execution du script.

```forth
10 !x @x      ;; stocker 10, le recuperer
```

**Frame** -- survit aux repetitions. Se reinitialise quand la ligne avance.
Ideal pour les compteurs.

```forth
@F.n 1 + !F.n
@F.n 12 mod note sine snd .   ;; parcourt 12 notes en boucle
```

```
SET F.count ADD F.count 1
>> [note: MOD F.count 12]
```

**Ligne** -- partagee entre toutes les frames d'une ligne. Une frame definit,
une autre lit.

```forth
;; frame A
c4 !L.root
;; frame B
@L.root 7 + note sine snd .
```

```
-- frame A
SET L.root 60
-- frame B
>> [note: ADD L.root 7]
```

**Globale** -- visible par tous les scripts de la session. A utiliser avec
parcimonie.

```forth
c4 !G.key
@G.key note sine snd .
```

```
SET G.key 60
>> [note: G.key]
```

## Stocker et recuperer (Cagire)

`!nom` stocke le sommet de la pile. `@nom` le recupere. Les variables
inconnues renvoient 0. `,nom` stocke et garde la valeur sur la pile :

```forth
440 ,freq sine snd .   ;; stocke 440 ET passe la valeur
```

Les prefixes de portee se placent entre l'operateur et le nom : `!G.x`,
`@L.root`, `,F.count`.

## Accumulateurs

Recuperer, modifier, restocker. Pattern classique pour des sequences
evolutives :

```forth
@F.n 1 + !F.n
( 0 !F.n ) @F.n 16 > ?    ;; repart a zero apres 16
```

Bob :

```
SET F.n ADD F.n 1
IF GT F.n 16 : SET F.n 0 END
>> [note: ADD 48 MOD F.n 12]
```

## Nommer les sons

Stocker un nom de son, le reutiliser entre les frames :

```forth
;; frame A
"sine" !L.synth
;; frame B, C, D...
c4 note @L.synth snd .
```

Changez une frame, toutes les frames de la ligne suivent.

## Valeurs d'environnement

Valeurs en lecture seule depuis le runtime. Les plus utiles :

- Position en beats, tempo, nombre aleatoire
- Index de frame, index de ligne, compteur d'iteration

Cagire : `iter` empile le compteur d'iteration, `rand` empile une valeur
aleatoire. Bob : `R` est un aleatoire 0-127, `I` l'index de boucle, `T`
le tempo.

## Visibilite temporelle

Dans une meme frame, vous relisez ce que vous venez d'ecrire. Les
modifications ne deviennent visibles pour les autres frames qu'apres la fin
d'execution de la frame courante. Si la frame A ecrit `10 !G.x` et la
frame B lit `@G.x` dans la meme passe, B voit l'ancienne valeur.
