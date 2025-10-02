
Sova is a software environment for collaborative musical live coding. It is composed of four components. Each of them can be installed and used independently, but they are designed to work together seamlessly:

<p align="center">
  <img src="./assets/diagrams/sova_architecture.svg">
</p>

| Component | Purpose | Key Features |
|-----------|---------|--------------|
| **Server** | Central hub | • Relay between musicians, the core and other components<br> • Receive messages, send messages, orchestrates the session |
| **Core** | Heart of Sova | • Host compilers/interpreters for live coding languages)<br>• Manage MIDI/OSC, audio I/O and world interaction<br>• Pre-configured OSC device for [SuperDirt](https://github.com/musikinformatik/SuperDirt) _(optional)_<br>• Spawn and control audio engine instance _(optional)_<br>• Synchronize musicians via [Ableton Link](https://ableton.github.io/link/) protocol<br>• Manage the shared _scene_ state (the jam session) |
| **GUI** | User interface | • Connect to Sova server: can spawn server instance<br>• Code editor with highlighting, error reporting, etc.<br>• Configure and control server instance <br>• Edit / Save / Load scene snapshots<br>• Real-time _scene_ display / edit<br>• Collaborative jamming!  |
| **Engine** | Audio engine | • Audio synthesis and sampling duties<br>• Controlled via OSC messages<br>• Portable and lightweight<br>• Easy to extend|
