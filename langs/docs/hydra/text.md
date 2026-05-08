# Text

`text()` renders a string as a texture that can be used like any other Hydra source. The text is CPU-rasterized to a 512x512 bitmap, uploaded to the GPU, and sampled via `iText0`.

## Usage

```
text("SOVA").out()
```

The function takes a single string argument and returns a source node. White text on a transparent background, centered in the texture.

## Multi-line

Newlines in the string produce multiple lines, each centered independently. Font size auto-scales to fit.

```
text("LINE 1\nLINE 2").out()
```

## Chaining

`text()` is a source, so it chains with all geometry, color, and blending operations:

```
text("HELLO").rotate(0.0, 0.1).colorama(0.05).out()
```

```
osc(60.0, 0.1).layer(text("SOVA")).out()
```

## Limitations

- One text texture per evaluation (`iText0`). Multiple `text()` calls overwrite each other — only the last one is rendered.
- Fixed 512x512 texture resolution.
- Uses the built-in monospace font (Hack). No font selection.
- Filled text only, no outlines.
