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
