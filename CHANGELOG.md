# Changelog

All notable changes to Sova will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

This changelog was introduced on 2026-02-06. No changelog existed prior to this date.

## [Unreleased]

### Added

- New `desktop` crate — egui/eframe native GUI scaffolding with app_id, window title, sizing constraints, centered positioning, and `widgets` module
- `desktop` added to workspace members
- Speed factor control in Track header — displays `{speed}x`, double-click to edit, clamped 0.1–10.0, accent-colored when not default
- Log viewer panel in desktop GUI with Server/Client tabs, severity-colored entries, and auto-scroll
- Devices panel in desktop app for MIDI/OSC/Audio device management (listing, connecting/disconnecting, virtual MIDI creation, OSC output creation, slot assignment, removal)
- Reusable confirmation dialog widget in desktop app
- File menu with Exit option and close confirmation when server is running

### Changed

- Connection form is now the central view when disconnected (moved from floating window)
- Disconnect button moved to bottom bar
- Aligned GUI frontend types with new core execution model (`ExecutionMode` is now a string union, `Scene.mode` replaces `Scene.global_mode`, `Line` has direct `looping`/`trailing` booleans)
- Renamed `globalMode` store to `sceneMode`, `setGlobalMode` API to `setSceneMode`
- Replaced global loop toggle in TopBar with scene mode cycler (Free/AtQuantum/LongestLine) with per-mode color styling
- Added per-line trailing toggle (`T` button) to track controls
- `server/src/main.rs` now creates `AudioEngineProxy` locally and bridges to doux-sova via a converter thread, decoupling the two type systems
- Removed all `[patch]` sections from root `Cargo.toml` and `gui/src-tauri/Cargo.toml` (doux and core now use published git versions)
- Global egui style with zero corner radius on all windows, widgets, and menus

### Fixed

- Timeline trailing playback now highlights all concurrent playing frames, not just the first
- Restored `audio` as default feature in `server/Cargo.toml` (was accidentally set to `[]` after merge, causing server exit code 2 when GUI passed audio CLI args)
- Improved staleness check in `gui/scripts/build-sidecar-dev.sh` to include `langs/src` and `server/Cargo.toml`
- Hide native number input spin buttons globally to prevent accidental value changes from misclicked spinner arrows in the compact UI

### Removed

- Client window toggle from View menu and context menu
- `PanelVisibility.client` field
- `setLineExecutionMode` and `setLineCustomLength` API functions (fields no longer exist in core)
- All GitHub Actions CI workflows (`.github/workflows/build-release.yml`, `solo-tui/.github/workflows/ci.yml`) and Dependabot config (`solo-tui/.github/dependabot.yml`)
- `sova_core` dependency from `doux-sova` crate — bridge types (`SyncTime`, `ParamValue`, `AudioPayload`) now live in `doux-sova/src/types.rs`
