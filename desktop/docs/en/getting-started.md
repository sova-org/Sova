Sova is a live coding sequencer designed as a musical instrument in its own
right — both technical and poetical. You write code, and the environment turns
it into MIDI notes, OSC messages, and audio, all synchronized to a shared clock
via Ableton Link. Four built-in languages each offer a different approach to
musical expression. Multiple musicians can connect and perform on the same
scene. No prerequisites are needed to get started.

## Connect

Open the Server panel and click Start. The app runs a local server and connects
to it automatically.

## First sound

Double-click any frame cell in the grid to open the code editor. The default
language is Cagire. Type:

```forth
kick snd .
```

Press Cmd+Enter (Ctrl+Enter on Linux/Windows) to evaluate. A kick sounds on
every beat.

A melody:

```forth
0 0.25 0.5 0.75 at
c4 e4 g4 c5 arp note sine snd .
```

Four notes spread across the frame. Replace `sine` with `saw` and re-evaluate.
The change is immediate.

## The grid

The scene grid is your workspace. Each column is a **line** (plays in parallel).
Each row in a column is a **frame** (plays in sequence). A line loops through its
frames top to bottom, then starts over.

A drum pattern in line 1, a bass in line 2, chords in line 3 — all running
simultaneously. See **The Grid** for navigation and editing.

## Four languages

Every frame has its own language. Pick the one that fits your musical intention.

**Cagire** — stack-based, Forth-like. Push values, apply words, emit with `.`.
Suited to sound design and quick experimentation.

```forth
c4 min7 arp note 0.5 decay 0.4 verb sine snd .
```

**Bob** — imperative, Polish notation. Event maps, loops, explicit timing with
WAIT. Suited to precise melodic sequences.

```
RANGE 0 3 :
  >> [note: ADD 60 MUL I 4 vel: 100]
  WAIT 0.25
END
```

**BaLi** — Lisp-like, expression-based. Nested S-expressions, loops and
transforms compose naturally. Suited to algorithmic and generative composition.

```
(loop 4
  (note (+ 60 (* $i 3)) 90)
  1//4)
```

**Boinx** — declarative pattern notation. Sequences and simultaneity expressed
visually with brackets. Suited to rhythmic patterns readable at a glance.

```
<s: 'kick'> | [. _ . _]
```

To change a frame's language, open the editor and select from the dropdown at
the top, or press Cmd+L (Ctrl+L). See **Languages** for more detail, and click
the language tabs (Bob, BaLi, Boinx, Cagire) for full references.

## Timing

Each frame has a duration in beats. The sequencer runs the frame's script once
per duration, then moves to the next. A 1-beat frame at 120 BPM runs every
half second.

Inside a frame, you can subdivide time. In Cagire, `at` places sounds at
fractional positions within the duration. In Bob, `WAIT` advances the clock
explicitly.

Set tempo and quantum in the transport bar at the top. See **Timing**.

## Your instruments

Sova sends events to device slots numbered 1 to 16. Open the Devices panel to
connect MIDI ports, create OSC endpoints, or enable the built-in audio engine
(Doux). Slot 1 is the default. In your code, use `dev` to target a slot:

```forth
2 dev c4 note 100 vel .
```

See **Devices**.

## Playing together

Start a server. Other musicians connect to your IP and port. Everyone sees the
same scene, edits in real time, and stays in sync via Ableton Link. Use
different lines to avoid editing conflicts. Chat is built in. See
**Multiplayer**.

## Visuals

Sova includes a shader engine inspired by [Hydra](https://hydra.ojack.xyz). Write visual pipelines —
oscillators, noise, kaleidoscopes, feedback loops — and they render behind the
interface in real time.

```
osc(60, 0.1).rotate(0, 0.1).kaleid(4).out()
```

See **Visuals (Hydra)**.

## Shortcuts

| Action | macOS | Linux/Windows |
|--------|-------|---------------|
| Evaluate code | Cmd+Enter | Ctrl+Enter |
| Command palette | Cmd+K | Ctrl+K |
| Play / Stop | Cmd+Shift+Space | Ctrl+Shift+Space |
| Save scene | Cmd+S | Ctrl+S |
| Load scene | Cmd+O | Ctrl+O |
| Server panel | Cmd+Shift+S | Ctrl+Shift+S |
| Devices panel | Cmd+Shift+I | Ctrl+Shift+I |
| Documentation | Cmd+Shift+H | Ctrl+Shift+H |
| Visuals | Cmd+Shift+V | Ctrl+Shift+V |
| Change language | Cmd+L | Ctrl+L |

Press F1 to see all keybindings. The command palette (Cmd+K) lists every action
with its shortcut.
