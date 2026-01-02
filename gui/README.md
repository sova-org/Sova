# Sova GUI

A graphical user interface (GUI) designed to be the primary user interface for Sova. This application comes with all the bells and whistles we can provide. It can act as a standalone binary for Sova. This application adheres to the client/server architecture exposed by the Sova `Core`. Each instance of Sova GUI can be used to host a server and/or as a client to connect to a server on the local network. The GUI can be used to play alone or to host/join a collaborative music jam. 

## Views

The visual workspace consists of resizable [split panes](https://github.com/orefalo/svelte-splitpanes). Each pane hosts a specialized view used to manage the live session (projects, devices, chat, etc). The scene editor, among these views, is the central piece: it combines a visual timeline with a [CodeMirror](https://codemirror.net/)-based script editor. The interface uses a flexible split pane layout. Each pane can display one of the following views:

- **Scene**: Visual timeline that shows lines and frames in the scene.
- **Devices**: used to manage MIDI and OSC devices.
- **Projects**: Project lmanagement, saving and loading projects from disk.
- **Config**: Application settings.
- **Logs**: Server log viewer.
- **Chat**: Real-time messaging with peers.
- **Login**: Server connection.

## Command Palette

Open with `Ctrl/Cmd+K`. Navigate with arrows, execute with `Enter`.

| Command | Description |
|---------|-------------|
| `play` | Start transport |
| `pause` | Stop transport |
| `tempo <BPM>` | Set tempo |
| `scene` | Show scene view |
| `config` | Show config view |
| `devices` | Show devices view |
| `logs` | Show logs view |
| `projects` | Show projects view |
| `chat` | Show chat view |
| `login` | Show login view |
| `nickname <name>` | Set display name |
| `save` | Save current project |
| `load <name> [now\|end]` | Load project (immediate or end of line) |
| `split-horizontal` | Split pane horizontally |
| `split-vertical` | Split pane vertically |
| `close-pane` | Close current pane |
| `disconnect` | Disconnect from server |
| `exit` | Quit application |
| `help` | Toggle help mode |

## Collaboration

Multiple clients can connect to the same server:
- Shared scene state — edits between users are synced in real-time
- (**TODO**) Peer editing indicators — see who is editing which frame
- Live chat with timestamps and usernames
- Shared transport and device state
- Global variables (A-Z) visible to all

## Implementation

Built with Tauri 2.0 and SvelteKit 5. The Rust backend (`src-tauri/`) handles server process management, TCP client connections, and project persistence. The Svelte frontend (`src/`) provides the UI, made with Svelte. State management uses Svelte 5 runes. Styling is done using Tailwind CSS. The tech stack is voluntarily boring, using technologies that make it easy to understand and maintain for external contributors.

## Building

```
pnpm install
pnpm tauri build
```

## Development

```
pnpm tauri dev
```
