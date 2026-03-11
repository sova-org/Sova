A chain starts with a source and ends with `.out()`:

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
