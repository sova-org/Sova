Blending combines two chains. The second chain is the first argument:

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
