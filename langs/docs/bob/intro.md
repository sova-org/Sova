# Bob

Bob is an expression-oriented language for sequencing musical events.
Everything in Bob is an expression that evaluates to a value.

## Basics

Write sequences of commands separated by spaces. Use `WAIT` to advance time
and `PLAY` to emit events:

```
PLAY [note: 60, vel: 100]
WAIT 1
PLAY [note: 64, vel: 80]
WAIT 1
```

## Loops

Use `L` to repeat a block. The loop index is available as `I`:

```
L 0 4 :
  PLAY [note: ADD 60 I, vel: 100]
  WAIT 0.5
END
```

## Arithmetic

`ADD`, `SUB`, `MUL`, `DIV`, `MOD` — all take two arguments, prefix style:

```
ADD 3 4       -- 7
MUL 2 SUB 5 1 -- 8
```

`RAND` returns a random float between 0 and 1.
`RRAND lo hi` returns a random value in the given range.
