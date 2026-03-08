# Visuels (Hydra)

Sova inclut un moteur visuel intégré inspiré de **Hydra**
(https://hydra.ojack.xyz), le synthétiseur vidéo live-codable créé par Olivia
Jack. Vous écrivez de courts scripts décrivant des pipelines visuels — sources,
transformations, effets de couleur, mélange, modulation — et Sova les affiche
en temps réel comme fond derrière l'interface.

La syntaxe suit de près les conventions de Hydra. Si vous connaissez Hydra,
vous savez déjà utiliser les visuels de Sova. Sinon, cet article couvre tout ce
qu'il faut savoir.

## Ouvrir l'éditeur

Activez l'éditeur de visuels depuis le menu ou la liste des panneaux. La
fenêtre d'édition permet d'écrire du code de style Hydra avec coloration
syntaxique. Appuyez sur **Cmd+Entrée** (macOS) ou **Ctrl+Entrée** pour
évaluer.

Pour activer le fond visuel, activez **Visuels** dans les options d'apparence.

## Pipeline de base

Un pipeline Hydra commence par une **source** et se termine par `.out()` :

```
osc(60, 0.1).out()
```

Cela crée un motif oscillateur et l'envoie à l'écran. Les sources génèrent une
couleur pour chaque pixel en fonction de ses coordonnées et du temps courant.

Vous pouvez enchaîner des **transformations** entre la source et `.out()` :

```
osc(60, 0.1).rotate(0, 0.1).kaleid(4).out()
```

## Sources

Les sources sont le point de départ de chaque pipeline. Chacune prend les
coordonnées du pixel et retourne une couleur.

- `osc(freq, sync, offset)` — Oscillateur sinusoïdal
- `noise(scale, offset)` — Bruit simplex
- `voronoi(scale, speed, blending)` — Cellules de Voronoï
- `shape(sides, radius, smoothing)` — Polygone régulier
- `gradient(speed)` — Dégradé UV
- `solid(r, g, b, a)` — Couleur unie

Tous les arguments ont des valeurs par défaut — vous pouvez appeler n'importe
quelle source avec moins d'arguments ou aucun : `osc()`, `noise()`, etc.

## Transformations géométriques

Les transformations géométriques modifient l'espace de coordonnées avant
l'échantillonnage de la source.

- `rotate(angle, speed)` — Rotation
- `scale(amount, xMult, yMult, offsetX, offsetY)` — Mise à l'échelle
- `kaleid(sides)` — Kaléidoscope
- `pixelate(x, y)` — Pixellisation
- `repeat(x, y, offsetX, offsetY)` — Pavage
- `scroll(scrollX, scrollY, speedX, speedY)` — Défilement
- `scrollX(amount, speed)` — Défilement horizontal
- `scrollY(amount, speed)` — Défilement vertical
- `repeatX(reps, offset)` — Répétition horizontale
- `repeatY(reps, offset)` — Répétition verticale

## Transformations de couleur

Les transformations de couleur modifient la sortie colorimétrique d'une source.

- `color(r, g, b, a)` — Multiplication de couleur
- `invert(amount)` — Inversion
- `contrast(amount)` — Contraste
- `brightness(amount)` — Luminosité
- `saturate(amount)` — Saturation
- `hue(shift)` — Décalage de teinte
- `posterize(bins, gamma)` — Postérisation
- `luma(threshold, tolerance)` — Masque de luminance
- `colorama(amount)` — Cycle HSV
- `shift(r, g, b, a)` — Décalage de canaux
- `thresh(threshold, tolerance)` — Seuillage
- `r(scale, offset)` — Ajustement du canal rouge
- `g(scale, offset)` — Ajustement du canal vert
- `b(scale, offset)` — Ajustement du canal bleu

## Mélange (blending)

Le mélange combine deux pipelines en un. Le second pipeline est passé en
premier argument :

```
osc(60).add(noise(10), 0.5).out()
```

- `add(source, amount)` — Mélange additif
- `mult(source, amount)` — Mélange multiplicatif
- `blend(source, amount)` — Interpolation linéaire
- `diff(source)` — Différence absolue
- `layer(source)` — Superposition par alpha
- `mask(source)` — Masque par luminance
- `sub(source, amount)` — Mélange soustractif

## Modulation

La modulation utilise la sortie d'un pipeline pour déformer les coordonnées
d'un autre pipeline. Comme le mélange, le modulateur est passé en premier
argument :

```
osc(60).modulate(noise(10), 0.1).out()
```

- `modulate(source, amount)` — Décalage XY
- `modulateScale(source, multiple, offset)` — Modulation d'échelle
- `modulateRotate(source, multiple, offset)` — Modulation de rotation
- `modulateRepeat(source, rX, rY, oX, oY)` — Modulation de répétition
- `modulateRepeatX(source, reps, offset)` — Modulation de rép. horizontale
- `modulateRepeatY(source, reps, offset)` — Modulation de rép. verticale
- `modulateKaleid(source, sides)` — Modulation kaléidoscope
- `modulateScrollX(source, amount, speed)` — Modulation de défilement H
- `modulateScrollY(source, amount, speed)` — Modulation de défilement V
- `modulatePixelate(source, multiple, offset)` — Modulation de pixellisation
- `modulateHue(source, amount)` — Décalage par teinte

## Buffers de sortie multiples

Le moteur Hydra de Sova fournit 4 buffers de sortie : `o0`, `o1`, `o2`, `o3`.
Cela correspond à l'architecture multi-buffer de Hydra et permet la
composition en couches, les références croisées et les boucles de rétroaction.

### Routage vers les buffers

Par défaut, `.out()` envoie au buffer 0. Vous pouvez spécifier un buffer
cible :

```
osc(60).out(o0)
noise(10).out(o1)
```

### Affichage des buffers

Par défaut, le buffer 0 est affiché. Utilisez `render()` pour contrôler ce qui
est montré :

- `render()` — affiche les 4 buffers en grille 2x2
- `render(o1)` — affiche uniquement le buffer 1

```
osc(60).out(o0)
noise(10).out(o1)
shape(3).out(o2)
voronoi(5).out(o3)
render()
```

### Références croisées avec `src()`

`src()` lit depuis un autre buffer, vous permettant d'utiliser la sortie d'un
buffer comme entrée d'un autre pipeline :

```
osc(60).out(o0)
src(o0).kaleid(4).out(o1)
render(o1)
```

Ici, le buffer 1 prend l'oscillateur du buffer 0 et lui applique un
kaléidoscope.

### Boucles de rétroaction (feedback)

Routez un buffer vers lui-même pour créer du feedback — chaque image lit la
sortie de l'image précédente :

```
src(o0).colorama(0.01).scale(1.01).out(o0)
```

Cela crée un effet de traînée : chaque image décale légèrement la teinte et
agrandit, s'accumulant au fil du temps. Combinez avec d'autres sources pour
un feedback plus riche :

```
osc(60).blend(src(o0), 0.9).out(o0)
```

L'oscillateur se mélange avec sa propre image précédente, créant un écho doux.

## Compatibilité ascendante

Si votre script retourne un nœud sans appeler `.out()`, il est automatiquement
routé vers le buffer 0 et affiché. Ces deux scripts sont équivalents :

```
osc(60).rotate(0, 0.1)
```

```
osc(60).rotate(0, 0.1).out(o0)
```

## Différences avec Hydra

Le moteur visuel de Sova est une implémentation inspirée de Hydra, pas un port
direct. Différences principales :

- **Pas de variable `time`** — utilisez les arguments temporels des sources et
  transformations (ex. `osc(freq, sync)` où `sync` contrôle la vitesse).
- **Pas de réactivité souris** — l'entrée souris n'est pas encore connectée.
- **Pas d'entrées externes** — caméra, vidéo et images ne sont pas supportées.
- **Pas de `speed` global** — la vitesse d'animation est contrôlée par source.
- **Scripts Rhai** — le moteur de script utilise Rhai, pas JavaScript. Rhai
  supporte `let`, `if`/`else`, `while`, `for`, `fn`, et l'arithmétique de base.
- **GLSL 330** — les shaders ciblent le profil core OpenGL 3.3.
