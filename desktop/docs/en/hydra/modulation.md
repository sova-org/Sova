Modulation uses one chain's output to distort another's coordinates. Same
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
