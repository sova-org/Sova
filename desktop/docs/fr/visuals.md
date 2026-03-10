# Visuels (Hydra)

Sova embarque un moteur visuel inspiré de [Hydra](https://hydra.ojack.xyz), le
synthétiseur vidéo live-codable d'Olivia Jack. On écrit de courts scripts
décrivant des pipelines visuels et Sova les rend en shader derrière l'interface.
Code et image se mêlent dans un même geste de performance audiovisuelle.

La syntaxe suit les conventions Hydra. Deux différences notables d'emblée : pas
de variable `time` (la vitesse d'animation se contrôle par source via les
arguments) et le langage de script est Rhai, pas JavaScript.

## Ouvrir l'éditeur

Activez l'éditeur de visuels depuis le menu ou la liste des panneaux. Appuyez
sur Cmd+Entrée (macOS) ou Ctrl+Entrée pour évaluer.

Pour activer le fond visuel, activez Visuels dans les options d'apparence.

## Pipeline de base

Un pipeline commence par une source et se termine par `.out()` :

```
osc(60, 0.1).out()
```

Un motif oscillateur envoyé à l'écran. Les sources génèrent une couleur par
pixel en fonction des coordonnées et du temps.

On enchaîne des transformations entre la source et `.out()` :

```
osc(60, 0.1).rotate(0, 0.1).kaleid(4).out()
```

Si le script retourne un nœud sans appeler `.out()`, il est envoyé au buffer 0
automatiquement. Ces deux écritures sont équivalentes :

```
osc(60).rotate(0, 0.1)
```

```
osc(60).rotate(0, 0.1).out(o0)
```

## Sources

Chaque pipeline commence par une source. Tous les arguments possèdent des
valeurs par défaut — on peut appeler n'importe quelle source sans arguments.

`osc(freq, sync, offset)` — oscillateur sinusoïdal. `freq` contrôle la densité,
`sync` la vitesse d'animation, `offset` décale la phase.

```
osc(60, 0.1, 0.5).out()
```

`noise(scale, offset)` — bruit simplex. `scale` contrôle le zoom, `offset`
décale dans le temps.

```
noise(10, 0.1).out()
```

`voronoi(scale, speed, blending)` — cellules de Voronoï.

```
voronoi(5, 0.3, 0.3).out()
```

`shape(sides, radius, smoothing)` — polygone régulier.

```
shape(3, 0.5, 0.01).out()
```

`gradient(speed)` — dégradé UV.

`solid(r, g, b, a)` — couleur unie.

## Transformations géométriques

Modifient l'espace de coordonnées avant l'échantillonnage.

- `rotate(angle, speed)` — rotation
- `scale(amount, xMult, yMult, offsetX, offsetY)` — mise à l'échelle
- `kaleid(sides)` — kaléidoscope
- `pixelate(x, y)` — pixellisation
- `repeat(x, y, offsetX, offsetY)` — pavage
- `scroll(scrollX, scrollY, speedX, speedY)` — défilement
- `scrollX(amount, speed)` — défilement horizontal
- `scrollY(amount, speed)` — défilement vertical
- `repeatX(reps, offset)` — répétition horizontale
- `repeatY(reps, offset)` — répétition verticale

Un oscillateur pavé et en rotation :

```
osc(40).rotate(0, 0.05).repeat(3, 3).out()
```

Un kaléidoscope avec rotation lente :

```
noise(10).kaleid(6).rotate(0, 0.02).out()
```

## Transformations de couleur

Modifient la sortie colorimétrique d'une source.

- `color(r, g, b, a)` — multiplication de couleur
- `invert(amount)` — inversion
- `contrast(amount)` — contraste
- `brightness(amount)` — luminosité
- `saturate(amount)` — saturation
- `hue(shift)` — décalage de teinte
- `posterize(bins, gamma)` — postérisation
- `luma(threshold, tolerance)` — masque de luminance
- `colorama(amount)` — cycle HSV
- `shift(r, g, b, a)` — décalage de canaux
- `thresh(threshold, tolerance)` — seuillage
- `r(scale, offset)` — canal rouge
- `g(scale, offset)` — canal vert
- `b(scale, offset)` — canal bleu

Bruit désaturé à fort contraste :

```
noise(10).saturate(0.2).contrast(2.0).out()
```

Cycle de couleurs sur un oscillateur :

```
osc(30, 0.1).colorama(0.05).out()
```

## Mélange (blending)

Le mélange combine deux pipelines. Le second pipeline est passé en premier
argument :

```
osc(60).add(noise(10), 0.5).out()
```

- `add(source, amount)` — additif
- `mult(source, amount)` — multiplicatif
- `blend(source, amount)` — interpolation linéaire
- `diff(source)` — différence absolue
- `layer(source)` — superposition par alpha
- `mask(source)` — masque par luminance
- `sub(source, amount)` — soustractif

Une forme mêlée au bruit :

```
noise(5).mult(shape(4, 0.8), 0.7).out()
```

## Modulation

La modulation utilise la sortie d'un pipeline pour déformer les coordonnées d'un
autre. Même syntaxe que le mélange — le modulateur est passé en premier
argument :

```
osc(60).modulate(noise(10), 0.1).out()
```

- `modulate(source, amount)` — décalage XY
- `modulateScale(source, multiple, offset)` — modulation d'échelle
- `modulateRotate(source, multiple, offset)` — modulation de rotation
- `modulateRepeat(source, rX, rY, oX, oY)` — modulation de répétition
- `modulateRepeatX(source, reps, offset)` — modulation de répétition horizontale
- `modulateRepeatY(source, reps, offset)` — modulation de répétition verticale
- `modulateKaleid(source, sides)` — modulation kaléidoscope
- `modulateScrollX(source, amount, speed)` — modulation de défilement horizontal
- `modulateScrollY(source, amount, speed)` — modulation de défilement vertical
- `modulatePixelate(source, multiple, offset)` — modulation de pixellisation
- `modulateHue(source, amount)` — décalage par teinte

Cellules de Voronoï déformées par un oscillateur :

```
voronoi(10, 0.3).modulate(osc(40), 0.1).out()
```

Distorsion type feedback via auto-modulation par un buffer :

```
osc(60).modulate(src(o0), 0.05).out(o0)
```

## Buffers de sortie

Quatre buffers de sortie : `o0`, `o1`, `o2`, `o3`.

Par défaut, `.out()` envoie au buffer `o0`. On peut spécifier une cible :

```
osc(60).out(o0)
noise(10).out(o1)
```

Contrôle de l'affichage avec `render()` :

- `render()` — les 4 buffers en grille 2×2
- `render(o1)` — uniquement le buffer 1

```
osc(60).out(o0)
noise(10).out(o1)
shape(3).out(o2)
voronoi(5).out(o3)
render()
```

## Références croisées avec src()

`src()` lit depuis un buffer, ce qui permet d'injecter la sortie d'un buffer
dans un autre pipeline :

```
osc(60).out(o0)
src(o0).kaleid(4).out(o1)
render(o1)
```

Le buffer 1 prend l'oscillateur du buffer 0 et lui applique un kaléidoscope.

## Feedback

On route un buffer vers lui-même. Chaque image lit la sortie de l'image
précédente :

```
src(o0).colorama(0.01).scale(1.01).out(o0)
```

Les décalages de teinte et les changements d'échelle s'accumulent, créant des
traînées. On peut mélanger avec une source pour enrichir le feedback :

```
osc(60).blend(src(o0), 0.9).out(o0)
```

L'oscillateur se mélange avec sa propre image précédente — un écho doux.

## Différences avec Hydra navigateur

Le moteur de Sova est inspiré de Hydra, non un portage. Différences
principales :

Pas de variable `time`. Dans Hydra navigateur, on écrit
`osc(60, 0.1, () => time * 0.1)`. Dans Sova, l'animation est intégrée aux
arguments des sources : `osc(freq, sync)` où `sync` contrôle la vitesse
directement. Pas de callbacks, pas de fonctions fléchées.

Pas de réactivité `mouse`. La position de la souris n'est pas connectée au
moteur visuel.

Pas d'entrées externes. Caméra, vidéo et images ne sont pas disponibles.

Pas de `speed` global. La vitesse d'animation est propre à chaque source,
contrôlée via les arguments.

Scripts Rhai, pas JavaScript. Le langage de script est Rhai. Il supporte `let`,
`if`/`else`, `while`, `for`, `fn` et l'arithmétique de base. Pas de closures,
pas de fonctions fléchées. Voir la documentation Rhai pour la syntaxe.

GLSL 330. Les shaders ciblent le profil core OpenGL 3.3.
