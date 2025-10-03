# Installation

<link href="https://fonts.googleapis.com/icon?family=Material+Icons" rel="stylesheet">

<style>
.download-button {
  flex: 1;
  min-width: 150px;
  padding: 1rem;
  background: var(--bg-secondary, #2d2d2d);
  color: var(--text-primary, white);
  text-align: center;
  text-decoration: none !important;
  border: none;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  transition: transform 0.2s ease;
}

.download-button:hover {
  transform: scale(1.05);
  text-decoration: none !important;
}

.download-button .material-icons {
  font-size: 3rem;
  margin-bottom: 0.5rem;
}
</style>

<div style="display: flex; gap: 1rem; margin: 2rem 0; flex-wrap: wrap;">
  <a href="#" class="download-button">
    <span class="material-icons">terminal</span>
    <div style="font-weight: bold;">Download for Linux</div>
  </a>
  <a href="#" class="download-button">
    <span class="material-icons">laptop_mac</span>
    <div style="font-weight: bold;">Download for macOS</div>
  </a>
  <a href="#" class="download-button">
    <span class="material-icons">window</span>
    <div style="font-weight: bold;">Download for Windows</div>
  </a>
</div>

Sova is a modular live coding environment. You can install components independently or get the full experience with the GUI. This guide will walk you through setting up Sova on your system.

## Prerequisites

### System Requirements

Before installing Sova, make sure you have:

- **Rust** (1.80 or later): [Install Rust](https://rustup.rs/)
- **Node.js** (20 or later): [Install Node.js](https://nodejs.org/)
- **pnpm**: Install with `npm install -g pnpm`

### Platform-Specific Dependencies

#### macOS
No additional dependencies required. Audio support via CoreAudio is built-in.

#### Linux
Install audio and MIDI dependencies:
```bash
# Debian/Ubuntu
sudo apt-get install libasound2-dev libjack-jackd2-dev

# Fedora
sudo dnf install alsa-lib-devel jack-audio-connection-kit-devel

# Arch
sudo pacman -S alsa-lib jack2
```

#### Windows
Audio support via WASAPI is built-in. No additional dependencies required.

## Quick Start: GUI Installation

The GUI is the easiest way to get started with Sova. It bundles the server and provides an intuitive interface for live coding.

### Clone the Repository

```bash
git clone https://github.com/Bubobubobubobubo/sova.git
cd Sova
```

### Build and Run the GUI

```bash
cd gui
pnpm install
pnpm tauri dev
```

The GUI will launch and automatically handle server management for you.

### Building a Standalone App

For a distributable application:

```bash
pnpm tauri build
```

The app will be in `gui/src-tauri/target/release/bundle/`.

## Component Installation

### Core + Server

The core is Sova's heart: it compiles live coding languages, manages MIDI/OSC, and orchestrates sessions.

```bash
cd core
cargo build --release
```

Run the server:

```bash
./target/release/sova_server
```

The server listens on default port 8000 and handles client connections.

### Audio Engine

The engine provides audio synthesis and sampling, controlled via OSC messages.

```bash
cd engine
cargo build --release
```

Run the engine:

```bash
./target/release/sova_engine
```

The engine will automatically detect available audio devices.

## Getting Ready to Play

### 1. Start the GUI

Launch the GUI application. It will automatically spawn a local server instance.

### 2. Configure Audio/MIDI

- The engine auto-detects audio devices
- MIDI devices are listed in the GUI settings
- OSC devices can be configured for external synths (e.g., SuperDirt)

### 3. Create Your First Script

1. Open the code editor in the GUI
2. Write your first pattern using Bali or Boinx syntax
3. Execute with `Ctrl+Enter` or the play button
4. Watch your code come to life in the scene grid

### 4. (Optional) Enable Collaboration

To jam with friends:

1. One person runs the relay server (or use a hosted relay)
2. All participants connect to the relay in GUI settings
3. Changes sync in real-time across all instances

### 5. Sync with Ableton Link

Enable Link synchronization in the GUI to sync tempo with other Link-enabled applications.

## Verify Installation

Test each component:

```bash
# Test server
./core/target/release/sova_server --help

# Test engine
./engine/target/release/sova_engine --help

# Test relay
./relay/target/release/sova-relay --help
```

## Next Steps

- Explore the [Getting Started](./getting_started.md) guide
- Learn about [Bali and Boinx](./core/languages.md) live coding languages
- Dive into [Engine synthesis](./engine/synths.md) capabilities