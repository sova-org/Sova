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
