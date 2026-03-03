# Rhai in SOVA

`rhai` is an interpreted, resumable language for SOVA.

Execution is event-based and timed:

- `EMIT(...)` yields a `Generic` event.
- `DELAY(...)` yields a scheduler wait time.

The interpreter resumes exactly after the yielding statement on the next `execute_next` call.

## Variable Scopes

Rhai variables are bridged to `EvaluationContext` stores with prefixes:

- `g_name` -> `global_vars["name"]`
- `l_name` -> `line_vars["name"]`
- `f_name` -> `frame_vars["name"]`
- `name` -> `instance_vars["name"]`

Missing variables read as `VariableValue::default()` (`0`).

## Time Helpers

- `beats(x)` -> `TimeSpan::Beats(x)`
- `frames(x)` -> `TimeSpan::Frames(x)`
- `micros(x)` -> `TimeSpan::Micros(max(0, x))`

Use these with `DELAY` and `EMIT`.
