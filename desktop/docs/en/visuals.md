# Visuals (Hydra)

Sova includes a built-in visual engine inspired by **Hydra**
(https://hydra.ojack.xyz), the live-codeable video synthesizer created by
Olivia Jack. You write short scripts that describe visual pipelines — sources,
transforms, color effects, blending, modulation — and Sova renders them as a
shader background behind the interface in real time.

The syntax follows Hydra's conventions closely. If you know Hydra, you already
know how to use Sova's visuals. If you don't, this article covers everything
you need.

## Opening the editor

Toggle the visuals editor from the menu or panel list. The editor window lets
you write Hydra-style code with syntax highlighting. Press **Cmd+Enter**
(macOS) or **Ctrl+Enter** to evaluate.

To enable the visual background, turn on **Visuals** in the Appearance options.

## Basic pipeline

A Hydra pipeline starts with a **source** and ends with `.out()`:

```
osc(60, 0.1).out()
```

This creates an oscillator pattern and sends it to the screen. Sources generate
a color for every pixel based on its coordinates and the current time.

You can chain **transforms** between the source and `.out()`:

```
osc(60, 0.1).rotate(0, 0.1).kaleid(4).out()
```

## Sources

Sources are the starting point of every pipeline. Each one takes pixel
coordinates and returns a color.

- `osc(freq, sync, offset)` — Sine-wave oscillator
- `noise(scale, offset)` — Simplex noise
- `voronoi(scale, speed, blending)` — Voronoi cells
- `shape(sides, radius, smoothing)` — Regular polygon
- `gradient(speed)` — UV gradient
- `solid(r, g, b, a)` — Flat color

All arguments have defaults — you can call any source with fewer arguments or
none at all: `osc()`, `noise()`, etc.

## Geometry transforms

Geometry transforms modify the coordinate space before the source is sampled.

- `rotate(angle, speed)` — Rotate
- `scale(amount, xMult, yMult, offsetX, offsetY)` — Scale
- `kaleid(sides)` — Kaleidoscope
- `pixelate(x, y)` — Pixelation
- `repeat(x, y, offsetX, offsetY)` — Tile repetition
- `scroll(scrollX, scrollY, speedX, speedY)` — Scroll / pan
- `scrollX(amount, speed)` — Horizontal scroll
- `scrollY(amount, speed)` — Vertical scroll
- `repeatX(reps, offset)` — Horizontal repeat
- `repeatY(reps, offset)` — Vertical repeat

## Color transforms

Color transforms modify the color output of a source.

- `color(r, g, b, a)` — Color multiply
- `invert(amount)` — Invert colors
- `contrast(amount)` — Adjust contrast
- `brightness(amount)` — Adjust brightness
- `saturate(amount)` — Adjust saturation
- `hue(shift)` — Shift hue
- `posterize(bins, gamma)` — Reduce color depth
- `luma(threshold, tolerance)` — Luminance mask
- `colorama(amount)` — HSV color cycling
- `shift(r, g, b, a)` — Shift color channels
- `thresh(threshold, tolerance)` — Threshold
- `r(scale, offset)` — Red channel adjust
- `g(scale, offset)` — Green channel adjust
- `b(scale, offset)` — Blue channel adjust

## Blending

Blending combines two pipelines into one. The second pipeline is passed as the
first argument:

```
osc(60).add(noise(10), 0.5).out()
```

- `add(source, amount)` — Additive blend
- `mult(source, amount)` — Multiplicative blend
- `blend(source, amount)` — Linear interpolation
- `diff(source)` — Absolute difference
- `layer(source)` — Alpha-based layering
- `mask(source)` — Luminance masking
- `sub(source, amount)` — Subtractive blend

## Modulation

Modulation uses one pipeline's output to distort another pipeline's
coordinates. Like blending, the modulator is passed as the first argument:

```
osc(60).modulate(noise(10), 0.1).out()
```

- `modulate(source, amount)` — XY offset
- `modulateScale(source, multiple, offset)` — Scale modulation
- `modulateRotate(source, multiple, offset)` — Rotation modulation
- `modulateRepeat(source, rX, rY, oX, oY)` — Repeat modulation
- `modulateRepeatX(source, reps, offset)` — Horizontal repeat mod
- `modulateRepeatY(source, reps, offset)` — Vertical repeat mod
- `modulateKaleid(source, sides)` — Kaleidoscope modulation
- `modulateScrollX(source, amount, speed)` — Horizontal scroll mod
- `modulateScrollY(source, amount, speed)` — Vertical scroll mod
- `modulatePixelate(source, multiple, offset)` — Pixelation modulation
- `modulateHue(source, amount)` — Hue-based offset

## Multiple output buffers

Sova's Hydra engine provides 4 output buffers: `o0`, `o1`, `o2`, `o3`. This
matches real Hydra's multi-buffer architecture and enables layered composition,
cross-referencing, and feedback loops.

### Routing to buffers

By default, `.out()` sends to buffer 0. You can specify a target buffer:

```
osc(60).out(o0)
noise(10).out(o1)
```

### Displaying buffers

By default, buffer 0 is displayed. Use `render()` to control what is shown:

- `render()` — display all 4 buffers in a 2x2 grid
- `render(o1)` — display only buffer 1

```
osc(60).out(o0)
noise(10).out(o1)
shape(3).out(o2)
voronoi(5).out(o3)
render()
```

### Cross-referencing with `src()`

`src()` reads from another buffer, enabling you to use one buffer's output as
the input to another pipeline:

```
osc(60).out(o0)
src(o0).kaleid(4).out(o1)
render(o1)
```

Here, buffer 1 takes buffer 0's oscillator and applies a kaleidoscope to it.

### Feedback loops

Route a buffer back into itself to create feedback — each frame reads the
previous frame's output:

```
src(o0).colorama(0.01).scale(1.01).out(o0)
```

This creates a trail effect: each frame shifts the hue slightly and scales up,
accumulating over time. Combine with other sources for richer feedback:

```
osc(60).blend(src(o0), 0.9).out(o0)
```

The oscillator blends with its own previous frame, creating a smooth echo.

## Backward compatibility

If your script returns a node without calling `.out()`, it is automatically
routed to buffer 0 and displayed. These two scripts are equivalent:

```
osc(60).rotate(0, 0.1)
```

```
osc(60).rotate(0, 0.1).out(o0)
```

## Differences from Hydra

Sova's visual engine is a Hydra-inspired implementation, not a direct port.
Key differences:

- **No `time` variable** — use time-based arguments on sources and transforms
  instead (e.g. `osc(freq, sync)` where `sync` controls animation speed).
- **No `mouse` reactivity** — mouse input is not yet connected.
- **No external inputs** — camera, video, and image sources are not supported.
- **No `speed` global** — animation speed is controlled per-source.
- **Rhai scripting** — the script engine uses Rhai, not JavaScript. Rhai
  supports `let`, `if`/`else`, `while`, `for`, `fn`, and basic arithmetic.
  See the Rhai documentation for details.
- **GLSL 330** — shaders target OpenGL 3.3 core profile.
