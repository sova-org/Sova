# Rhai Reference (SOVA v1 subset)

## Builtins

`EMIT(args)`
- Emits `ConcreteEvent::Generic(args, 0, "", 1)`.

`EMIT(args, dur)`
- Emits with explicit duration and defaults `chan = ""`, `dev = 1`.

`EMIT(args, dur, chan, dev)`
- Emits with explicit duration/channel/device.

`DELAY(dur)`
- Yields a wait duration in microseconds.

`beats(x)`, `frames(x)`, `micros(x)`
- Build `TimeSpan` values.

## Control Flow

Supported:

- `if / else`
- `while`
- `for`
- `break`
- `continue`

## Variables and Assignment

Supported:

- `let` declarations
- simple assignment (`=`)
- op-assignment (`+=`, `-=`, `*=`, `/=`, `%=` ...)
- index assignment (including op-assignment): `a[i] = x`, `m["k"] += 1`

## Notes

- `EMIT` and `DELAY` are statements, not expressions.
- Duration arguments for `DELAY` and `EMIT(..., dur, ...)` must be duration values.
- Runtime errors are pushed to `ctx.errors`, then interpreter terminates.
