# Sova Languages

This crate contains all programming language implementations for Sova. Each language compiles to VM bytecode or implements the interpreter trait to interact with the virtual machine directly.

## Languages

| Language | Type | Description |
|----------|------|-------------|
| **Bali** | Compiled | Declarative language, looks like a LISP  |
| **Bob** | Compiled | Monome Teleype inspired imperative language |
| **Boinx** | Interpreted | Concise functional programming language |
| **Forth** | Interpreted | Stack-based concatenative language (WIP) |
| **Rhai** | Compiled | Rhai scripting language integration (WIP) |
| **Lua** | Compiled | Lua/Luau integration (WIP) |

## Architecture

Languages either implement the `Compiler` trait (source -> bytecode) or the `Interpreter` trait (direct execution). The `LanguageCenter` in core registers available compilers and interpreters.

- **Compiled languages** (Bali, Bob, Rhai, Lua): Transform source code into VM instructions
- **Interpreted languages** (Boinx, Forth): Execute directly, emitting events through the VM interface

## Building

```
cargo build -p langs
```

## Testing

```
cargo test -p langs
```

## Adding a Language

1. Create a new module in `src/`
2. Implement either `Compiler` or `Interpreter` trait from `sova_core`
3. Register in `src/lib.rs`
4. Add to `LanguageCenter` in consumer crates (server, solo-tui)
