Sova's visual engine is built on the ideas of
[Hydra](https://hydra.ojack.xyz), the live-codeable video synthesizer created by
[Olivia Jack](https://ojack.xyz). Hydra brought analog video synthesis thinking
into the browser — real-time pattern generation, transformation, and compositing
through code. It is one of the most influential tools in the live coding
community, used in performances, workshops, and classrooms worldwide. Its source
code is available on [GitHub](https://github.com/hydra-synth/hydra).

If you already know browser Hydra, read [Differences](hydra-differences) first — it covers what Sova does differently.

Sova implements a subset of Hydra's function vocabulary as a native shader
engine. The syntax follows Hydra's conventions: sources, geometry transforms,
color operations, blending, and modulation chain together and end with `.out()`.
Two notable differences up front: `time`, `beat`, `tempo`, and `phase` are
available as expression variables (not arrow functions as in browser Hydra), and
the scripting language is Rhai, not JavaScript. See [Differences](hydra-differences) for the full
list.

## Opening the editor

Toggle the visuals editor from the menu or panel list. Press Cmd+Enter (macOS)
or Ctrl+Enter to evaluate.

To enable the visual background, turn on Visuals in the Appearance options.
