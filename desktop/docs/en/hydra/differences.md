Sova's engine is Hydra-inspired, not a port. Key differences:

`time`, `beat`, `tempo`, and `phase` are available as expression variables.
Unlike browser Hydra which uses arrow functions (`() => time * 0.1`), Sova
embeds these as GLSL uniform references. Write `osc(60.0, sin(time))` directly.
`beat` and `phase` sync to the musical clock via Ableton Link.

No `mouse` reactivity. Mouse position is not connected to the visual engine.

No external inputs. Camera, video, and image sources are not available.

No `speed` global. Animation speed is per-source, controlled through arguments.

Rhai scripting, not JavaScript. The script language is Rhai. It supports `let`,
`if`/`else`, `while`, `for`, `fn`, and basic arithmetic. No closures, no arrow
functions. See the Rhai documentation for syntax details.

GLSL 330. Shaders target OpenGL 3.3 core profile.
