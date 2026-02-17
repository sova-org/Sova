# Getting Started with Bob

Bob is a terse, expression-oriented language for live coding music. It uses **Polish notation** (operator before operands): `ADD 2 3` instead of `2 + 3`. Everything is an expression, expressions nest naturally, and every keystroke counts when performing live.

## Your first sequence

Emit a MIDI note with `>>` and advance time with `WAIT`:

```
>> [note: 60 vel: 100]
WAIT 1
>> [note: 64 vel: 80]
WAIT 1
>> [note: 67 vel: 100]
```

`>>` sends an event map to the current device. `WAIT 1` pauses for one beat. Without `WAIT`, all events fire simultaneously.

## Variables

Bob has four scopes, distinguished by prefix:

```
SET G.root 60       -- global: shared across all scripts
SET F.count 0       -- frame: persists for the current frame
SET L.phase 0       -- line: persists across executions
```

Instance variables (lowercase words) are read-only function parameters.

## Arithmetic

All operators are prefix with fixed arity. They nest without parentheses:

```
ADD 3 4             -- 7
MUL 2 SUB 5 1      -- 8 (SUB 5 1 = 4, then MUL 2 4)
>> [note: ADD G.root 7 vel: 100]
```

## Loops

`RANGE` iterates from start to end. `I` is the loop index:

```
RANGE 0 3 :
  >> [note: ADD 60 I vel: 100]
  WAIT 0.5
END
```

`DO` repeats N times without an index:

```
DO 4 : >> [note: 60] WAIT 0.25 END
```

## Euclidean rhythms

Distribute hits evenly across steps:

```
EU 3 8 0.125 : >> [note: 60] END
```

## Device selection

Route events to a specific MIDI/OSC output:

```
DEV 1
>> [note: 60]
DEV 2
>> [note: 48]
```

## Lists and iteration

```
SET G.NOTES '[60 64 67 72]
EACH G.NOTES : >> [note: E] WAIT 0.25 END
```

## Random

```
>> [note: RRAND 48 72 vel: RRAND 60 127]
PROB 50 : >> [note: 60] END
```

## Next steps

See the **Language Reference** article for complete documentation of all keywords, data types, control flow, events, and more.
