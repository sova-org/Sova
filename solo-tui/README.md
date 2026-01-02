# Solo TUI

A standalone TUI (_Terminal User Interface_) for Sova. Bypasses the client/server architecture entirely, embeds Sova Core as a library and communicates directly via channels. Scene editing, script composition, device routing, and transport control from a single interface. Built with [Ratatui](https://ratatui.rs/) and Crossterm. Event-driven architecture, running at 30 FPS. Solo TUI is a client used to interact with Sova Core directly, mostly aimed at developers and experienced users. **Note**: As the name suggests, Solo TUI is a "solo" editor for Sova. No collaboration, bypassing the server means no shared state with other clients.

## Features

The application is organized in views following a spatial layout. Switch between them with `Ctrl+Arrow`:

- **Scene Grid**: Navigate lines (tracks) and frames (steps). Visual compilation status per frame.
  - `↑` `↓` move between frames, `←` `→` move between lines
  - `i` insert frame after selection
  - `l` insert line after selection
  - `r` remove selected frame,`Ctrl+r` remove selected line
  - `d` set frame duration (beats)
  - `x` set frame repetitions
  - `m` toggle frame enabled/disabled
  - `y` duplicate frame, `Ctrl+y` duplicate line

- **Script Editor**: Edit code, switch between languages.
  - `Ctrl+s` upload script to scheduler
  - `Ctrl+l` switch language
  - `Ctrl+a` select all
  - `Ctrl+w` select word forward
  - `Ctrl+c` copy, `Ctrl+x` cut, `Ctrl+v` paste

- **Device Routing**: Assign MIDI/OSC devices to numbered slots.
  - `↑` `↓` navigate device list
  - `a` assign device to slot
  - `u` unassign device from slot
  - `o` create OSC output (name:ip:port)
  - `m` connect MIDI output

- **Transport**: Control playback.
  - `Space` play/pause
  - `↑` `↓` adjust tempo ±1 BPM
  - `t` set tempo, `q` set quantum
  - `s` toggle start/stop sync
  - `r` reset beat to zero

- **Logs**: Log viewer.
  - `↑` `↓` `←` `→` scroll

- **Persistence**: Save and load scenes as JSON snapshots.
  - `Ctrl+s` save scene to file
  - `Ctrl+l` load scene from file

There are some global keybindings you might want to be aware of:

- `Ctrl+Space`: play/pause (works on any page).
- `Ctrl+↑` `Ctrl+↓` `Ctrl+←` `Ctrl+→`: navigate between pages.
- `Esc`: quit the application (with confirmation modal).

## Building

```
cargo build --release
```

## Running

```
cargo run --release
```
