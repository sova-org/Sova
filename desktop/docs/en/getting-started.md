This guide walks through Sova's core workflow: connecting to a server, making your first sound, navigating the grid, and using timing and devices. See [About Sova](about) for context on the project and its design.

## Connect

When Sova opens, you land on the connection screen. Two things need to happen: start a server, then connect to it.

Click **Start Server** on the left side. This launches a local server with default settings (tempo 120 BPM, quantum 4 beats). The button turns green to confirm the server is running. To change these defaults before starting, click the gear icon next to the button to open the server configuration.

Enter a username in the **User** field — this identifies you in multiplayer sessions. Then click **Connect**. The connection screen disappears and the main interface appears: the scene grid in the center, the transport bar at the top.

## First sound

Press **Play** in the transport bar (top left). The sequencer starts running. The scene begins with one line containing one frame of 1 beat duration.

Double-click the frame cell in the grid to open the code editor. The editor opens in a floating window. The default language is Boinx — the current language is displayed in the top-left corner of the editor window.

For this tutorial, switch to Cagire: press Cmd+L (Ctrl+L on Linux/Windows) to open the language selector, then pick **cagire** from the list. Type:

```forth
kick snd .
```

Press Cmd+Enter (Ctrl+Enter on Linux/Windows) to evaluate. The editor flashes white to confirm. A kick drum sounds on every beat.

Try a melody:

```forth
0 0.25 0.5 0.75 at
c4 e4 g4 c5 arp note sine snd .5 decay .
```

Four notes spread across the beat. Replace `sine` with `saw` and re-evaluate — the change is immediate. This is the core loop of live coding: write, evaluate, listen, modify.

## The grid

The scene grid is your workspace. Each column is a **line** (runs in parallel). Each row in a column is a **frame** (runs in sequence). A line loops through its frames from top to bottom, then starts over.

Right-click on a frame cell to access context menu actions: insert or remove frames, edit duration and repetitions, cut/copy/paste, move frames up or down. You can also add new lines using the + button at the bottom left of the grid.

Each frame displays its duration (in beats), repetition count, and a preview of its code. A progress bar overlays the currently playing frame. A drum pattern in line 1, a bass in line 2, chords in line 3 — all running simultaneously.

See [The Scene](the-scene) for the full details on navigation and editing.

## Languages

Every frame has its own language. To change it, open the editor and press Cmd+L (Ctrl+L) or click the language name at the top of the editor window. A searchable list appears — select the language you want. The language applies to that frame only; other frames keep their own.

Four languages ship with Sova by default. More can be added — the environment is designed to host new languages without modifying the core. This extensibility is central to Sova's design: the VM, scheduler, and I/O layer are shared infrastructure, and each language is free to explore its own paradigm for musical expression. See [About Sova](about) for more on this philosophy.

**Cagire** — stack-based, Forth-like. Push values, apply words, emit with `.`. Suited to sound design and quick experimentation.

```forth
c4 min7 arp note 0.5 decay 0.4 verb sine snd .
```

**Bob** — imperative, Polish notation. Event maps, loops, explicit timing with WAIT. Suited to precise melodic sequences.

```
RANGE 0 3 :
  >> [note: ADD 60 MUL I 4 vel: 100]
  WAIT 0.25
END
```

**BaLi** — Lisp-like, expression-based. Nested S-expressions, loops and transforms compose naturally. Suited to algorithmic and generative composition.

```
(loop 4
  (note (+ 60 (* $i 3)) 90)
  1//4)
```

**Boinx** — declarative pattern notation. Sequences and simultaneity expressed visually with brackets. Suited to rhythmic patterns readable at a glance.

```
<s: 'kick'> | [. _ . _]
```

See [Languages](languages) for more detail, and click the language tabs (Bob, BaLi, Boinx, Cagire) in the documentation panel for full references and runnable examples.

## Timing

Each frame has a duration in beats. The sequencer runs the frame's script once per duration, then moves to the next frame. A 1-beat frame at 120 BPM runs every half second. Edit a frame's duration directly in the grid cell, or right-click and select **Edit Duration**.

Inside a frame, you can subdivide time. In Cagire, `at` places sounds at fractional positions within the duration. In Bob, `WAIT` advances the clock explicitly.

The transport bar at the top displays the current beat, tempo, and quantum. Click the tempo value to edit it (range: 20–300 BPM). The animated phase bar shows where you are in the current quantum cycle. See [Timing](timing).

## Your instruments

Sova sends events to device slots numbered 1 to 16. When the server starts, it creates a virtual MIDI port named "Sova" and assigns it to slot 1. Any code you write sends to slot 1 by default.

Open the Devices panel (Cmd+Shift+I / Ctrl+Shift+I) to see connected devices, add MIDI ports, create OSC endpoints, or enable the built-in audio engine (Doux). Each device can be assigned to a slot number. In your code, use `dev` to target a different slot:

```forth
2 dev c4 note 100 vel .
```

See [Devices](devices).

## Playing together

Other musicians connect to your server using your IP address and port. Everyone sees the same scene, edits in real time, and stays in sync via Ableton Link. Use different lines to avoid editing conflicts. The editor shows who else is editing the same frame. Chat is built in.

See [Multiplayer](multiplayer).

## Visuals

Sova includes a visual scripting engine for real-time graphics. See [Hydra](hydra-intro) in the documentation panel.

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

Press F1 to see all keybindings. The command palette (Cmd+K) lists every action with its shortcut.
