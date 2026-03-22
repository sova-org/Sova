Every chain starts with a source. All arguments have defaults — you can call
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

`rings(freq, speed)` — concentric rings radiating from center. `freq` controls
the number of rings, `speed` controls outward expansion.

```
rings(8, 0.1).out()
```

`checker(cols, rows)` — checkerboard pattern. `cols` and `rows` set the grid
density.

```
checker(4, 4).out()
```
