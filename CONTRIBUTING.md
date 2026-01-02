# Contributing to Sova

Thank you for considering contributing to Sova! We welcome contributions from everyone. Here are some guidelines to help you get started.

## How to Contribute?

Contributing to Sova development doesn't necessarily require programming. You can help by reporting bugs, suggesting improvements, correcting spelling mistakes, translating documentation, etc. This repository is a collaborative workspace:

- **Programming:** Bug fixes, feature additions. See [the list of open issues]().
- **Documentation:** Improving guides, correcting mistakes, translating content.
- **Testing:** Test Sova, report encountered issues.
- **Ideas:** Propose new features or improvements.
- **Community:** Answer questions from other users.

## Prerequisites

Before you start, ensure you have:

- **Rust** (stable toolchain)
- **Node.js** and **pnpm**
- **Tauri prerequisites** for your platform - see [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)

## How to Contribute

1. **Fork the repository**: Click the "Fork" button at the top right of the repository page.

2. **Clone your fork**: 
    ```sh
    git clone https://github.com/your-username/sova.git
    ```

3. **Create a branch**: 
    ```sh
    git checkout -b your-branch-name
    ```

4. **Make your changes**: Implement your feature, bug fix, or documentation update.

5. **Commit your changes**: 
    ```sh
    git add .
    git commit -m "Description of your changes"
    ```

6. **Push to your fork**: 
    ```sh
    git push origin your-branch-name
    ```

7. **Create a Pull Request**: Go to the original repository and click the "New Pull Request" button.

## Quick Start

```sh
# GUI development (SvelteKit + Tauri)
cd gui && pnpm install && pnpm dev

# Rust backend
cargo build
cargo clippy

# Build language grammars (if modifying Bali/Boinx)
cd gui && pnpm grammar:build
```

The GUI runs `sova_server` as a sidecar binary. When you run `pnpm dev`, the sidecar is automatically built via `scripts/build-sidecar.sh`. If you modify `core/`, the sidecar will be rebuilt on next dev/build.

## Project Structure

- `core/` - Server, scheduling, device management
- `engine/` - Audio synthesis
- `gui/` - SvelteKit + Tauri desktop app
- `relay/` - Collaboration server
- `solo-tui/` - Terminal interface

## Code Style

- Follow the existing code style.
- Write clear, concise commit messages.
- Comments in English, keep them sparse.

### Rust
- Run `cargo clippy` before submitting a PR.
- Avoid cloning to satisfy the borrow checker - find a better solution.

### TypeScript
- Use pnpm (not npm or yarn).
- Run `pnpm check` for type checking.

## Reporting Issues

If you find a bug or have a feature request, please open an issue on GitHub. Provide as much detail as possible to help us understand and address the issue.
