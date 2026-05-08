Four output buffers: `o0`, `o1`, `o2`, `o3`.

By default, `.out()` sends to `o0`. Specify a target:

```
osc(60).out(o0)
noise(10).out(o1)
```

Display control with `render()`:

- `render()` — all 4 buffers in a 2x2 grid
- `render(o1)` — only buffer 1

```
osc(60).out(o0)
noise(10).out(o1)
shape(3).out(o2)
voronoi(5).out(o3)
render()
```

## Clearing with hush()

`hush()` stops all visuals immediately — clears every buffer and outputs a black screen.

```
hush()
```

## Cross-referencing with src()

`src()` reads from a buffer, allowing you to pipe one buffer's output into
another chain:

```
osc(60).out(o0)
src(o0).kaleid(4).out(o1)
render(o1)
```

Buffer 1 takes buffer 0's oscillator and applies a kaleidoscope.
