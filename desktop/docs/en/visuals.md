# Visuals (Hydra)

Sova includes a visual engine inspired by [Hydra](https://hydra.ojack.xyz),
Olivia Jack's live-codeable video synthesizer. Write short scripts that describe
visual pipelines and Sova renders them as a shader background behind the
interface. Code and image merge in a single act of audiovisual performance.

The syntax follows Hydra's conventions. Two notable differences up front: there
is no `time` variable (animation speed is controlled per-source via arguments),
and the scripting language is Rhai, not JavaScript.

## Opening the editor

Toggle the visuals editor from the menu or panel list. Press Cmd+Enter (macOS)
or Ctrl+Enter to evaluate.

To enable the visual background, turn on Visuals in the Appearance options.

## Basic pipeline

A pipeline starts with a source and ends with `.out()`:

```
osc(60, 0.1).out()
```

An oscillator pattern sent to the screen. Sources generate a color per pixel
based on coordinates and time.

Chain transforms between source and `.out()`:

```
osc(60, 0.1).rotate(0, 0.1).kaleid(4).out()
```

If your script returns a node without calling `.out()`, it goes to buffer 0
automatically. These two are equivalent:

```
osc(60).rotate(0, 0.1)
```

```
osc(60).rotate(0, 0.1).out(o0)
```

## Sources

Every pipeline starts with a source. All arguments have defaults — you can call
any source with no arguments.

`osc(freq, sync, offset)` — sine-wave oscillator. `freq` controls density,
`sync` controls animation speed, `offset` shifts the phase.

```
osc(60, 0.1, 0.5).out()
```

`noise(scale, offset)` — simplex noise. `scale` controls zoom, `offset` shifts
over time.

```
noise(10, 0.1).out()
```

`voronoi(scale, speed, blending)` — Voronoi cells.

```
voronoi(5, 0.3, 0.3).out()
```

`shape(sides, radius, smoothing)` — regular polygon.

```
shape(3, 0.5, 0.01).out()
```

`gradient(speed)` — UV gradient.

`solid(r, g, b, a)` — flat color.

## Geometry transforms

Modify the coordinate space before sampling.

- `rotate(angle, speed)` — rotation
- `scale(amount, xMult, yMult, offsetX, offsetY)` — scaling
- `kaleid(sides)` — kaleidoscope
- `pixelate(x, y)` — pixelation
- `repeat(x, y, offsetX, offsetY)` — tile repetition
- `scroll(scrollX, scrollY, speedX, speedY)` — scroll / pan
- `scrollX(amount, speed)` — horizontal scroll
- `scrollY(amount, speed)` — vertical scroll
- `repeatX(reps, offset)` — horizontal repeat
- `repeatY(reps, offset)` — vertical repeat

A tiled, rotating oscillator:

```
osc(40).rotate(0, 0.05).repeat(3, 3).out()
```

A kaleidoscope with slow spin:

```
noise(10).kaleid(6).rotate(0, 0.02).out()
```

## Color transforms

Modify the color output of a source.

- `color(r, g, b, a)` — color multiply
- `invert(amount)` — invert
- `contrast(amount)` — contrast
- `brightness(amount)` — brightness
- `saturate(amount)` — saturation
- `hue(shift)` — hue shift
- `posterize(bins, gamma)` — reduce color depth
- `luma(threshold, tolerance)` — luminance mask
- `colorama(amount)` — HSV color cycling
- `shift(r, g, b, a)` — shift channels
- `thresh(threshold, tolerance)` — threshold
- `r(scale, offset)` — red channel
- `g(scale, offset)` — green channel
- `b(scale, offset)` — blue channel

Desaturated, high-contrast noise:

```
noise(10).saturate(0.2).contrast(2.0).out()
```

Color cycling on an oscillator:

```
osc(30, 0.1).colorama(0.05).out()
```

## Blending

Blending combines two pipelines. The second pipeline is the first argument:

```
osc(60).add(noise(10), 0.5).out()
```

- `add(source, amount)` — additive
- `mult(source, amount)` — multiplicative
- `blend(source, amount)` — linear interpolation
- `diff(source)` — absolute difference
- `layer(source)` — alpha layering
- `mask(source)` — luminance masking
- `sub(source, amount)` — subtractive

A shape blended over noise:

```
noise(5).mult(shape(4, 0.8), 0.7).out()
```

## Modulation

Modulation uses one pipeline's output to distort another's coordinates. Same
syntax as blending — the modulator is the first argument:

```
osc(60).modulate(noise(10), 0.1).out()
```

- `modulate(source, amount)` — XY offset
- `modulateScale(source, multiple, offset)` — scale modulation
- `modulateRotate(source, multiple, offset)` — rotation modulation
- `modulateRepeat(source, rX, rY, oX, oY)` — repeat modulation
- `modulateRepeatX(source, reps, offset)` — horizontal repeat modulation
- `modulateRepeatY(source, reps, offset)` — vertical repeat modulation
- `modulateKaleid(source, sides)` — kaleidoscope modulation
- `modulateScrollX(source, amount, speed)` — horizontal scroll modulation
- `modulateScrollY(source, amount, speed)` — vertical scroll modulation
- `modulatePixelate(source, multiple, offset)` — pixelation modulation
- `modulateHue(source, amount)` — hue-based offset

Voronoi cells warped by an oscillator:

```
voronoi(10, 0.3).modulate(osc(40), 0.1).out()
```

Feedback-like distortion using self-modulation through a buffer:

```
osc(60).modulate(src(o0), 0.05).out(o0)
```

## Output buffers

Four output buffers: `o0`, `o1`, `o2`, `o3`.

By default, `.out()` sends to `o0`. Specify a target:

```
osc(60).out(o0)
noise(10).out(o1)
```

Display control with `render()`:

- `render()` — all 4 buffers in a 2×2 grid
- `render(o1)` — only buffer 1

```
osc(60).out(o0)
noise(10).out(o1)
shape(3).out(o2)
voronoi(5).out(o3)
render()
```

## Cross-referencing with src()

`src()` reads from a buffer, allowing you to pipe one buffer's output into
another pipeline:

```
osc(60).out(o0)
src(o0).kaleid(4).out(o1)
render(o1)
```

Buffer 1 takes buffer 0's oscillator and applies a kaleidoscope.

## Feedback

Route a buffer back into itself. Each frame reads the previous frame's output:

```
src(o0).colorama(0.01).scale(1.01).out(o0)
```

Hue shifts and scale changes accumulate over time, creating trails. Blend with
a source for richer feedback:

```
osc(60).blend(src(o0), 0.9).out(o0)
```

The oscillator blends with its own previous frame — a smooth echo.

## Differences from browser Hydra

Sova's engine is Hydra-inspired, not a port. Key differences:

No `time` variable. In browser Hydra you write `osc(60, 0.1, () => time * 0.1)`.
In Sova, animation is baked into source arguments: `osc(freq, sync)` where
`sync` controls speed directly. No callbacks, no arrow functions.

No `mouse` reactivity. Mouse position is not connected to the visual engine.

No external inputs. Camera, video, and image sources are not available.

No `speed` global. Animation speed is per-source, controlled through arguments.

Rhai scripting, not JavaScript. The script language is Rhai. It supports `let`,
`if`/`else`, `while`, `for`, `fn`, and basic arithmetic. No closures, no arrow
functions. See the Rhai documentation for syntax details.

GLSL 330. Shaders target OpenGL 3.3 core profile.
