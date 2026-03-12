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
