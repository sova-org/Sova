# Visuels (Hydra)

Sova embarque un moteur visuel inspire de Hydra (https://hydra.ojack.xyz), le
synthetiseur video live-codable d'Olivia Jack. On ecrit de courts scripts
decrivant des pipelines visuels et Sova les rend en shader derriere l'interface.

La syntaxe suit les conventions Hydra. Deux differences importantes d'emblee :
pas de variable `time` (la vitesse d'animation se controle par source via les
arguments), et le langage de script est Rhai, pas JavaScript.

## Ouvrir l'editeur

Activez l'editeur de visuels depuis le menu ou la liste des panneaux. Appuyez
sur Cmd+Entree (macOS) ou Ctrl+Entree pour evaluer.

Pour activer le fond visuel, activez Visuels dans les options d'apparence.

## Pipeline de base

Un pipeline commence par une source et se termine par `.out()` :

```
osc(60, 0.1).out()
```

Un motif oscillateur, envoye a l'ecran. Les sources generent une couleur par
pixel en fonction des coordonnees et du temps.

On enchaine des transformations entre la source et `.out()` :

```
osc(60, 0.1).rotate(0, 0.1).kaleid(4).out()
```

Si le script retourne un noeud sans appeler `.out()`, il est envoye au buffer 0
automatiquement. Ces deux ecritures sont equivalentes :

```
osc(60).rotate(0, 0.1)
```

```
osc(60).rotate(0, 0.1).out(o0)
```

## Sources

Chaque pipeline commence par une source. Tous les arguments ont des valeurs par
defaut -- on peut appeler n'importe quelle source sans arguments.

`osc(freq, sync, offset)` -- oscillateur sinusoidal. `freq` controle la densite,
`sync` la vitesse d'animation, `offset` decale la phase.

```
osc(60, 0.1, 0.5).out()
```

`noise(scale, offset)` -- bruit simplex. `scale` zoom, `offset` decale dans le
temps.

```
noise(10, 0.1).out()
```

`voronoi(scale, speed, blending)` -- cellules de Voronoi.

```
voronoi(5, 0.3, 0.3).out()
```

`shape(sides, radius, smoothing)` -- polygone regulier.

```
shape(3, 0.5, 0.01).out()
```

`gradient(speed)` -- degrade UV.

`solid(r, g, b, a)` -- couleur unie.

## Transformations geometriques

Modifient l'espace de coordonnees avant l'echantillonnage.

- `rotate(angle, speed)` -- rotation
- `scale(amount, xMult, yMult, offsetX, offsetY)` -- mise a l'echelle
- `kaleid(sides)` -- kaleidoscope
- `pixelate(x, y)` -- pixellisation
- `repeat(x, y, offsetX, offsetY)` -- pavage
- `scroll(scrollX, scrollY, speedX, speedY)` -- defilement
- `scrollX(amount, speed)` -- defilement horizontal
- `scrollY(amount, speed)` -- defilement vertical
- `repeatX(reps, offset)` -- repetition horizontale
- `repeatY(reps, offset)` -- repetition verticale

Un oscillateur pave et en rotation :

```
osc(40).rotate(0, 0.05).repeat(3, 3).out()
```

Un kaleidoscope avec rotation lente :

```
noise(10).kaleid(6).rotate(0, 0.02).out()
```

## Transformations de couleur

Modifient la sortie colorimetrique d'une source.

- `color(r, g, b, a)` -- multiplication de couleur
- `invert(amount)` -- inversion
- `contrast(amount)` -- contraste
- `brightness(amount)` -- luminosite
- `saturate(amount)` -- saturation
- `hue(shift)` -- decalage de teinte
- `posterize(bins, gamma)` -- posterisation
- `luma(threshold, tolerance)` -- masque de luminance
- `colorama(amount)` -- cycle HSV
- `shift(r, g, b, a)` -- decalage de canaux
- `thresh(threshold, tolerance)` -- seuillage
- `r(scale, offset)` -- canal rouge
- `g(scale, offset)` -- canal vert
- `b(scale, offset)` -- canal bleu

Bruit desature, fort contraste :

```
noise(10).saturate(0.2).contrast(2.0).out()
```

Cycle de couleurs sur un oscillateur :

```
osc(30, 0.1).colorama(0.05).out()
```

## Melange (blending)

Le melange combine deux pipelines. Le second pipeline est le premier argument :

```
osc(60).add(noise(10), 0.5).out()
```

- `add(source, amount)` -- additif
- `mult(source, amount)` -- multiplicatif
- `blend(source, amount)` -- interpolation lineaire
- `diff(source)` -- difference absolue
- `layer(source)` -- superposition par alpha
- `mask(source)` -- masque par luminance
- `sub(source, amount)` -- soustractif

Une forme melee au bruit :

```
noise(5).mult(shape(4, 0.8), 0.7).out()
```

## Modulation

La modulation utilise la sortie d'un pipeline pour deformer les coordonnees d'un
autre. Meme syntaxe que le melange -- le modulateur est le premier argument :

```
osc(60).modulate(noise(10), 0.1).out()
```

- `modulate(source, amount)` -- decalage XY
- `modulateScale(source, multiple, offset)` -- modulation d'echelle
- `modulateRotate(source, multiple, offset)` -- modulation de rotation
- `modulateRepeat(source, rX, rY, oX, oY)` -- modulation de repetition
- `modulateRepeatX(source, reps, offset)` -- modulation de rep. horizontale
- `modulateRepeatY(source, reps, offset)` -- modulation de rep. verticale
- `modulateKaleid(source, sides)` -- modulation kaleidoscope
- `modulateScrollX(source, amount, speed)` -- modulation de defilement H
- `modulateScrollY(source, amount, speed)` -- modulation de defilement V
- `modulatePixelate(source, multiple, offset)` -- modulation de pixellisation
- `modulateHue(source, amount)` -- decalage par teinte

Cellules de Voronoi deformees par un oscillateur :

```
voronoi(10, 0.3).modulate(osc(40), 0.1).out()
```

Distorsion type feedback via auto-modulation par un buffer :

```
osc(60).modulate(src(o0), 0.05).out(o0)
```

## Buffers de sortie

Quatre buffers de sortie : `o0`, `o1`, `o2`, `o3`.

Par defaut `.out()` envoie au buffer `o0`. On peut specifier une cible :

```
osc(60).out(o0)
noise(10).out(o1)
```

Controle de l'affichage avec `render()` :

- `render()` -- les 4 buffers en grille 2x2
- `render(o1)` -- uniquement le buffer 1

```
osc(60).out(o0)
noise(10).out(o1)
shape(3).out(o2)
voronoi(5).out(o3)
render()
```

## References croisees avec src()

`src()` lit depuis un buffer, ce qui permet d'injecter la sortie d'un buffer
dans un autre pipeline :

```
osc(60).out(o0)
src(o0).kaleid(4).out(o1)
render(o1)
```

Le buffer 1 prend l'oscillateur du buffer 0 et lui applique un kaleidoscope.

## Feedback

Router un buffer vers lui-meme. Chaque image lit la sortie de l'image
precedente :

```
src(o0).colorama(0.01).scale(1.01).out(o0)
```

Les decalages de teinte et les changements d'echelle s'accumulent, creant des
trainees. Melanger avec une source pour enrichir le feedback :

```
osc(60).blend(src(o0), 0.9).out(o0)
```

L'oscillateur se melange avec sa propre image precedente -- un echo doux.

## Differences avec Hydra navigateur

Le moteur de Sova est inspire de Hydra, pas un portage. Differences principales :

Pas de variable `time`. Dans Hydra navigateur on ecrit
`osc(60, 0.1, () => time * 0.1)`. Dans Sova, l'animation est integree aux
arguments des sources : `osc(freq, sync)` ou `sync` controle la vitesse
directement. Pas de callbacks, pas de fonctions flechees.

Pas de reactivite `mouse`. La position de la souris n'est pas connectee au
moteur visuel.

Pas d'entrees externes. Camera, video et images ne sont pas disponibles.

Pas de `speed` global. La vitesse d'animation est par source, controlee via les
arguments.

Scripts Rhai, pas JavaScript. Le langage de script est Rhai. Il supporte `let`,
`if`/`else`, `while`, `for`, `fn` et l'arithmetique de base. Pas de closures,
pas de fonctions flechees. Voir la documentation Rhai pour la syntaxe.

GLSL 330. Les shaders ciblent le profil core OpenGL 3.3.
