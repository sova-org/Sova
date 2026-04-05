Variables are used to store values that persist beyond a single event. They are similar to variables encountered in regular programming languages. They are used to share data between scripts, accumulate state and can be used to build patterns that evolve over time. Every variable lives in a scope that determines who can see it and how long it survives. Variables in Sova are key to having fun sessions with other musicians. You can share a lot of information, no matter the language you use. Some variables are shared globally in live in the virtual machine, meaning that you can tweak a variable in `Cagire` and your friend can use it while playing with `Boinx`.

## Scopes

There are four variable scopes, here sorted from from the narrowest to the widest:

- **Instance**: created fresh each time a script executes. It does not survives to the next execution. Use it for intermediate calculations that don't need to outlive a single run.

- **Frame**: each frame has its own store. Values survive across repetitions within that frame: each repetition reads what the previous one wrote. When the line advances to the next frame, that frame's own store takes over. A counter that increments on every repetition is the typical use.

- **Line**: shared across all frames in a line. One frame writes a value, another reads it later in the sequence. Set a root note in one frame, transpose from it in the next. Line variables persist as long as the line exists in the [scene](the-scene).

- **Global**: visible to every script in the session, across all lines and frames. In a [multiplayer](multiplayer) session, global variables are shared between all connected musicians. It should be best reserved for session-wide state: a root note, a shared counter, a mode flag, etc.

## Reading and writing

Each language has its own syntax for variables, telling you what scope you are currently assigning a variable to. The exact conventions differ per language: see the language tabs for syntax. Reading an undefined variable returns zero. There is no error, no `null` value, no exception. It is safe to read a variable before it has been used or defined. Variables are dynamically typed. The first assignment determines the type. Subsequent writes coerce the new value to match. Write a float into an integer variable and it truncates to an integer.

## Environment values

These variables are very special. They are read-only values injected by the runtime. Scripts can read them but not write them. At the VM level, six environment functions exist:

| Function | Returns |
|----------|---------|
| `GetTempo` | Current session tempo (BPM) |
| `RandomInt` | Unrestricted random integer |
| `RandomFloat` | Random float in [0, 1) |
| `RandomUInt(n)` | Random integer in [0, n) |
| `RandomDecInBounds(min, max)` | Random decimal in [min, max] |
| `FrameLen(line, frame)` | Duration of a specific frame in beats |

These names are internal to the VM. Each language wraps them in its own syntax and adds its own context values — tempo, loop index, random helpers, step position, and more. See the language tabs for the full list and how to access them.

## Visibility between frames

Within a script, you read back what you just wrote. But the scheduler guarantees isolation between concurrent frames: if two frames run in the same scheduling pass, each sees the other's previous values, not the current ones. The result does not depend on which frame the scheduler runs first. This prevents ordering surprises when multiple lines share global state.
