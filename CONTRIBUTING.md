# Contributing to Sova

Contributions are welcome. There are many ways to contribute beyond code:

- **Bug reports**: Open an issue describing the problem and steps to reproduce.
- **Feature requests**: Suggest new features or improvements.
- **Documentation**: Fix typos, clarify explanations, or add examples.
- **Testing**: Try Sova on different platforms, report issues.
- **Language design**: Propose new languages or improvements to existing ones.
- **Tutorials**: Write guides or share example sessions.
- **Community support**: Help others in issues and discussions.

## Prerequisites

Before you start, ensure you have:

- **Rust** (stable toolchain) - [rustup.rs](https://rustup.rs/)
- **Node.js** (v18+) and **pnpm** - [pnpm.io](https://pnpm.io/)
- **Tauri prerequisites** for your platform - [Tauri Prerequisites](https://v2.tauri.app/start/prerequisites/)

## Quick start

```sh
# GUI development (SvelteKit + Tauri)
cd gui && pnpm install && pnpm tauri dev

# Rust backend only
cargo build
cargo clippy

# Build language grammars (if modifying Bali/Boinx)
cd gui && pnpm grammar:build
```

The GUI runs `sova_server` as a sidecar binary. When you run `pnpm tauri dev`, the sidecar is automatically built. If you modify `core/`, the sidecar will be rebuilt on next dev/build.

## Project structure

- `core/` - Server, VM, scheduling, device management
- `gui/` - SvelteKit + Tauri desktop app
- `solo-tui/` - Terminal interface

## Code contributions

1. Fork the repository
2. Create a branch for your changes
3. Make your changes
4. Run `cargo clippy` and fix any warnings
5. Submit a pull request with a clear description of your changes

Please explain the reasoning behind your changes in the pull request. Document what problem you're solving and how your solution works. This helps reviewers understand your intent and speeds up the review process.

### Rust

- Run `cargo clippy` before submitting.
- Avoid cloning to satisfy the borrow checker - find a better solution.

### TypeScript/Svelte

- Use pnpm (not npm or yarn).
- Run `pnpm check` for type checking.

## Code of conduct

This project follows the [Contributor Covenant 2.1](https://www.contributor-covenant.org/version/2/1/code_of_conduct/). By participating, you agree to uphold its standards. We are committed to providing a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity, experience level, nationality, appearance, race, religion, or sexual identity.

**Expected behavior:**
- Demonstrate empathy and kindness
- Respect differing viewpoints and experiences
- Accept constructive feedback gracefully
- Focus on what's best for the community

**Unacceptable behavior:**
- Harassment, trolling, or personal attacks
- Sexualized language or unwanted advances
- Publishing others' private information
- Any conduct inappropriate in a professional setting

Report violations to the project maintainers. All complaints will be reviewed promptly and confidentially.

## License

By contributing, you agree that your contributions will be licensed under AGPLv3.
