# Installation

<link href="https://fonts.googleapis.com/icon?family=Material+Icons" rel="stylesheet">

<style>
.download-button {
  flex: 1;
  min-width: 150px;
  padding: 1.5rem 1rem;
  background: #0F131A;
  color: #BFBDB6;
  text-align: center;
  text-decoration: none !important;
  border: 1px solid #565B66 !important;
  border-bottom: 1px solid #565B66 !important;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  box-sizing: border-box;
  margin-bottom: 1rem;
}

.download-button:hover {
  border-color: #59C2FF;
  color: #59C2FF;
  transform: scale(1.05);
  text-decoration: none !important;
}

.download-button .material-icons {
  font-size: 3rem;
  margin-bottom: 0.5rem;
}
</style>

## Release versions

Sova is a modular live coding environment. The easiest way to get started is to download the pre-built binary for your platform. Simply download the software and run it! If you are a developer or want to contribute to the project, you can also build Sova from source.

<div style="display: flex; gap: 1rem; margin: 2rem 0 3rem 0; flex-wrap: wrap; padding-bottom: 1rem;">
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

## From source

Before installing Sova, make sure to install the following dependencies on your system:

- **Rust** (1.80 or later): [Install Rust](https://rustup.rs/).
- **Node.js** (20 or later): [Install Node.js](https://nodejs.org/).
- **pnpm**: Install with `npm install -g pnpm`.

Some additional dependencies are required depending on your operating system:

<!-- tabs:start -->

#### **macOS**

No additional dependencies required. Audio support via CoreAudio is built-in.

#### **Linux**

Install audio and MIDI dependencies:
```bash
# Debian/Ubuntu
sudo apt-get install libasound2-dev libjack-jackd2-dev

# Fedora
sudo dnf install alsa-lib-devel jack-audio-connection-kit-devel

# Arch
sudo pacman -S alsa-lib jack2
```

#### **Windows**

Audio support via WASAPI is built-in. No additional dependencies required.

<!-- tabs:end -->

You can now safely clone the Sova repository and proceed with the build:

```bash
git clone https://github.com/Bubobubobubobubo/sova.git
cd Sova
```

## Modules

<!-- tabs:start -->

#### **GUI (Recommended)**

Build and run the GUI application:

```bash
cd gui
pnpm install
pnpm tauri dev
```

The GUI will launch immediately. If you want to test the production build, run:

```bash
pnpm tauri build
```

The app will be in `gui/src-tauri/target/release/bundle/`.

#### **Core + Server**

```bash
cd core
cargo build --release
```

Run the server:

```bash
./target/release/sova_server
```

The server listens on default port 8000 and handles client connections. Please take a look at the [server documentation](/docs/server/server.md) for more details about flags and options.

#### **Audio Engine**

The audio engine can be used independently from the rest of the Sova environment. It is a lightweight and portable audio synthesis engine.

```bash
cd engine
cargo build --release
```

Run the engine:

```bash
./target/release/sova_engine
```

The engine will automatically detect available audio devices. Please take a look at the [engine documentation](/docs/engine/engine.md) for more details about flags and options.

<!-- tabs:end -->

