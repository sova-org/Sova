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
- `polar()` — Cartesian to polar coordinates (angle→X, radius→Y)
- `cart()` — polar back to Cartesian (inverse of `polar`)
- `fold(amount)` — fold UV space back on itself, creating hard-edged symmetry

A tiled, rotating oscillator:

```
osc(40).rotate(0, 0.05).repeat(3, 3).out()
```

A kaleidoscope with slow spin:

```
noise(10).kaleid(6).rotate(0, 0.02).out()
```

Polar coordinate transform — stripes become rings:

```
osc(20).polar().out()
```

Radial tiling — repeat in polar space creates angular symmetry:

```
osc(10).polar().repeatX(4).out()
```

Folded noise — fractal-like hard edges:

```
noise(3, 0.1).fold(2).out()
```
