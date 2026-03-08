Sova is a live coding sequencer. You write code, Sova turns it into MIDI notes,
OSC messages, and audio -- all synchronized to a shared clock via Ableton Link.
Four built-in languages, each with its own way of thinking about music. Multiple
players can connect and perform on the same scene simultaneously.

## Connect

Open the Server panel and click Start. The app runs a local server and connects
to it. You're in.

## First sound

Double-click any frame cell in the grid to open the code editor. The default
language is Cagire. Type:

```forth
kick snd .
```

Press Cmd+Enter (Ctrl+Enter on Linux/Windows) to evaluate. You hear a kick on
every beat.

Try a melody:

```forth
0 0.25 0.5 0.75 at
c4 e4 g4 c5 arp note sine snd .
```

Four notes, evenly spaced across the frame. Change `sine` to `saw`, re-evaluate.
Instant difference.

## The grid

The scene grid is your workspace. Each column is a **line** (plays in parallel).
Each row in a column is a **frame** (plays in sequence). A line loops through its
frames top to bottom, then starts over.

You can have a drum pattern in line 1, a bass in line 2, and chords in line 3,
all running at the same time. See the **The Grid** article for navigation and
editing.

## Four languages

Every frame has its own language. Pick the one that fits what you're doing.

**Cagire** -- stack-based, Forth-like. Push values, apply words, emit with `.`.
Best for sound design and quick experimentation.

```forth
c4 min7 arp note 0.5 decay 0.4 verb sine snd .
```

**Bob** -- imperative, Polish notation. Event maps, loops, explicit timing with
WAIT. Best for precise melodic sequences.

```
RANGE 0 3 :
  >> [note: ADD 60 MUL I 4 vel: 100]
  WAIT 0.25
END
```

**BaLi** -- Lisp-like, expression-based. Nested S-expressions, loops and
transforms compose naturally. Best for algorithmic and generative patterns.

```
(loop 4
  (note (+ 60 (* $i 3)) 90)
  1//4)
```

**Boinx** -- declarative pattern notation. Sequences and simultaneity expressed
visually with brackets. Best for rhythmic patterns you can see at a glance.

```
<s: 'kick'> | [. _ . _]
```

To change a frame's language, open the editor and select from the dropdown at the
top, or press Cmd+L (Ctrl+L). See the **Languages** article for more on each
one, and click the language tabs (Bob, bali, Boinx, Cagire) for full references.

## Timing

Each frame has a duration in beats. The sequencer plays the frame's script once
per duration, then moves to the next frame. A 1-beat frame at 120 BPM runs every
half second.

Inside a frame, you can subdivide time. In Cagire, `at` places sounds at
fractional positions within the beat. In Bob, `WAIT` advances the clock
explicitly.

Set tempo and quantum in the transport bar at the top. See the **Timing**
article.

## Your gear

Sova sends events to device slots numbered 1 to 16. Open the Devices panel to
connect MIDI ports, create OSC endpoints, or enable the built-in audio engine
(Doux). Slot 1 is the default. In your code, use `dev` to target a slot:

```forth
2 dev c4 note 100 vel .
```

See the **Devices** article.

## Playing together

Start a server. Other players connect to your IP and port. Everyone sees the same
scene, edits in real time, and stays in sync via Ableton Link. Use different
lines to avoid stepping on each other's code. Chat is built in. See the
**Multiplayer** article.

## Visuals

Sova has a built-in shader engine inspired by Hydra. Write visual pipelines --
oscillators, noise, kaleidoscopes, feedback loops -- and they render behind the
interface in real time.

```
osc(60, 0.1).rotate(0, 0.1).kaleid(4).out()
```

See the **Visuals (Hydra)** article.

## Shortcuts

| Action | macOS | Linux/Windows |
|--------|-------|---------------|
| Evaluate code | Cmd+Enter | Ctrl+Enter |
| Command palette | Cmd+K | Ctrl+K |
| Play / Stop | Cmd+Shift+Space | Ctrl+Shift+Space |
| Save scene | Cmd+S | Ctrl+S |
| Load scene | Cmd+O | Ctrl+O |
| Toggle server | Cmd+Shift+S | Ctrl+Shift+S |
| Toggle devices | Cmd+Shift+I | Ctrl+Shift+I |
| Toggle docs | Cmd+Shift+H | Ctrl+Shift+H |
| Toggle visuals | Cmd+Shift+V | Ctrl+Shift+V |
| Change language | Cmd+L | Ctrl+L |

Press F1 to see all keybindings. The command palette (Cmd+K) lists every action
with its shortcut.
