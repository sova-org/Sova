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

## Quick start

```sh
cargo build
cargo clippy
cargo run -p sova-desktop --release
```

## Project structure

- `core/` - Rust library: VM, scheduler, protocols (MIDI, OSC), and languages
- `langs/` - Language implementations (Bob, BaLi, Boinx, Cagire)
- `server/` - TCP server binary for multiplayer coordination
- `desktop/` - Primary desktop client (egui/eframe)
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
