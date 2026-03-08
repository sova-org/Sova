# Variables

Les variables stockent des valeurs qui persistent entre les événements ou
entre les frames. On les utilise pour coordonner les scripts, accumuler de
l'état, et construire des patterns évolutifs.

## Portées par l'exemple

Quatre portées. La portée détermine qui voit la variable et combien de temps
elle vit.

**Instance** -- mémoire locale. Se réinitialise à chaque exécution du script.

```forth
10 !x @x      ;; stocker 10, le récupérer
```

**Frame** -- survit aux répétitions. Se réinitialise quand la ligne avance.
Idéal pour les compteurs.

```forth
@F.n 1 + !F.n
@F.n 12 mod note sine snd .   ;; parcourt 12 notes en boucle
```

```
SET F.count ADD F.count 1
>> [note: MOD F.count 12]
```

**Ligne** -- partagée entre toutes les frames d'une ligne. Une frame définit,
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

**Globale** -- visible par tous les scripts de la session. À utiliser avec
parcimonie.

```forth
c4 !G.key
@G.key note sine snd .
```

```
SET G.key 60
>> [note: G.key]
```

## Stocker et récupérer (Cagire)

`!nom` stocke le sommet de la pile. `@nom` le récupère. Les variables
inconnues renvoient 0. `,nom` stocke et garde la valeur sur la pile :

```forth
440 ,freq sine snd .   ;; stocke 440 ET passe la valeur
```

Les préfixes de portée se placent entre l'opérateur et le nom : `!G.x`,
`@L.root`, `,F.count`.

## Accumulateurs

Récupérer, modifier, restocker. Pattern classique pour des séquences
évolutives :

```forth
@F.n 1 + !F.n
( 0 !F.n ) @F.n 16 > ?    ;; repart à zéro après 16
```

Bob :

```
SET F.n ADD F.n 1
IF GT F.n 16 : SET F.n 0 END
>> [note: ADD 48 MOD F.n 12]
```

## Nommer les sons

Stocker un nom de son, le réutiliser entre les frames :

```forth
;; frame A
"sine" !L.synth
;; frame B, C, D...
c4 note @L.synth snd .
```

Changez une frame, toutes les frames de la ligne suivent.

## Valeurs d'environnement

Valeurs en lecture seule depuis le runtime. Les plus utiles :

- Position en beats, tempo, nombre aléatoire
- Index de frame, index de ligne, compteur d'itération

Cagire : `iter` empile le compteur d'itération, `rand` empile une valeur
aléatoire. Bob : `R` est un aléatoire 0-127, `I` l'index de boucle, `T`
le tempo.

## Visibilité temporelle

Dans une même frame, on relit ce qu'on vient d'écrire. Les modifications ne
deviennent visibles pour les autres frames qu'après la fin d'exécution de la
frame courante. Si la frame A écrit `10 !G.x` et la frame B lit `@G.x` dans
la même passe, B voit l'ancienne valeur.
