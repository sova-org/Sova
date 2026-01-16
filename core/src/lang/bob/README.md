# Bob Language Reference

Bob is a terse, Monome Teletype-inspired DSL for live coding music. It uses **Polish notation** (operator before operands) with **fixed arity** operators, eliminating the need for parentheses. Expressions nest naturally: `ADD 2 MUL 3 4` evaluates to 14 because `MUL 3 4` yields 12, then `ADD 2 12` yields 14. The language prioritizes brevity and immediacy: every keystroke counts when performing live.

## Data Types

Bob supports seven primary data types:

- **Integer**: Whole numbers. `42`, `-10`
- **Float**: Decimal numbers. `3.14`, `-0.5`
- **String**: Quoted text. `"hello"`
- **Symbol**: Colon-prefixed identifiers. `:kick`, `:synth`
- **Note Symbol**: MIDI note names. `:c3` (60), `:c#3` (61), `:db3` (61)
- **List**: Ordered collection. `'[1 2 3]`
- **Map**: Key-value pairs in brackets. `[note: 60 vel: 100]`
- **Bytes**: Raw byte sequences for SysEx. `BYTES: 240 67 247 END`

### Note Symbols

Note symbols provide readable MIDI note values:

```
:c3    # MIDI 60 (middle C)
:c#3   # MIDI 61 (C sharp)
:db3   # MIDI 61 (D flat)
:d3    # MIDI 62
:a4    # MIDI 69 (concert A)
```

## Variables

Bob has four variable scopes, distinguished by prefix:

- **Global** (`G.name`): Prefixed with `G.`. Shared across all scripts.
- **Frame** (`F.name`): Prefixed with `F.`. Persists for the current frame.
- **Line** (`L.name`): Prefixed with `L.`. Persists per source line across executions.
- **Instance** (`lowercase`): Lowercase word. Function parameters, read-only.

Four special read-only variables are available:

- `I`: Loop index, set by `L` loops and `EACH` (ForEach) loops.
- `E`: Current element, set by `EACH` (ForEach) loops.
- `T`: Current tempo.
- `R`: Random value 0-127, changes each read.

### Assignment

Assignment uses the `SET` keyword and returns the assigned value:

```
SET G.X 42            # Global
SET F.count 0         # Frame-scoped
SET L.phase 0.5       # Line-scoped
```

Note: Single uppercase letters (`A`-`Z`) can be read as globals, but assignment requires the `G.` prefix.

## Operators

### Arithmetic

Binary operations on two values:

- **`ADD`** or **`+`**: Addition. `ADD 2 3` → 5
- **`SUB`** or **`-`**: Subtraction. `SUB 10 3` → 7
- **`MUL`** or **`*`**: Multiplication. `MUL 4 5` → 20
- **`DIV`** or **`/`**: Division. `DIV 20 4` → 5
- **`MOD`** or **`%`**: Modulo. `MOD 17 5` → 2

Unary operations on one value:

- **`NEG`**: Negate. `NEG 5` → -5
- **`ABS`**: Absolute value. `ABS NEG 5` → 5

### Comparison

All return boolean (true/false):

- **`GT`** or **`>`**: Greater than. `GT 5 3` → true
- **`LT`** or **`<`**: Less than. `LT 2 8` → true
- **`GTE`** or **`>=`**: Greater or equal. `GTE 5 5` → true
- **`LTE`** or **`<=`**: Less or equal. `LTE 3 3` → true
- **`EQ`** or **`==`**: Equal. `EQ 4 4` → true
- **`NE`** or **`!=`**: Not equal. `NE 1 2` → true

### Logical

- **`AND`** or **`&&`**: Logical and. `AND 1 1` → true
- **`OR`** or **`||`**: Logical or. `OR 0 1` → true
- **`XOR`**: Logical exclusive or. `XOR 1 0` → true
- **`NOT`** or **`!`**: Logical not (unary). `NOT 0` → true

### Bitwise

- **`BAND`** or **`&`**: Bitwise and. `BAND 12 10` → 8
- **`BOR`** or **`|`**: Bitwise or. `BOR 12 10` → 14
- **`BXOR`** or **`^`**: Bitwise exclusive or. `BXOR 12 10` → 6
- **`BNOT`** or **`~`**: Bitwise not (unary). `BNOT 0` → -1
- **`SHL`** or **`<<`**: Shift left. `SHL 1 4` → 16
- **`SHR`**: Shift right. `SHR 16 2` → 4

### Utility

- **`MIN`**: Minimum of two values. `MIN 3 7` → 3
- **`MAX`**: Maximum of two values. `MAX 3 7` → 7
- **`CLAMP`**: Constrain value to range. `CLAMP 15 0 10` → 10
- **`WRAP`**: Wrap value around range. `WRAP 12 0 10` → 2
- **`SCALE`**: Remap from one range to another. `SCALE 5 0 10 0 100` → 50
- **`QT`**: Quantize to nearest step. `QT 7 4` → 8

### Random

- **`TOSS`**: Random 0 or 1 (no arguments). `TOSS` → 0 or 1
- **`RAND`**: Random integer from 0 to n (inclusive). `RAND 10` → 0-10
- **`RRAND`**: Random integer in range (inclusive). `RRAND 5 10` → 5-10
- **`DRUNK`**: Brownian walk from current value. `DRUNK G.X 2` → G.X ± 0-2

## Control Flow

All control structures use `:` to open a body and `END` to close. Alternatively, braces `{ }` can be used instead.

### Conditional

```
IF cond : body END

IF cond : body ELSE : body END

# No ELIF - use nested IF/ELSE
IF cond1 : body1 ELSE : IF cond2 : body2 ELSE : body3 END END
```

Brace style (ELSE requires colon style for both branches):
```
IF cond { body } ELSE { body }
```

### Ternary

Inline conditional expression:
```
? cond then else
SET G.X ? GT G.Y 10 100 0    # G.X = 100 if G.Y > 10, else 0
```

### Loops

**Counted loop** with iterator `I`:
```
RANGE start end : body END
RANGE start end step : body END
RANGE 1 4 : >> [note: ADD 60 I] END    # I = 1, 2, 3, 4
RANGE 0 10 2 : >> [note: I] END        # I = 0, 2, 4, 6, 8, 10
```

**Repeat N times** (no iterator):
```
DO n : body END
DO 4 : >> [note: 60] END
```

**While loop**:
```
WHILE cond : body END
```

### ForEach Loop

Iterate over each element with `EACH`. Inside the body, `E` is the current element and `I` is the index:

```
EACH list : body END

EACH '[60 64 67] : >> [note: E] END                # play each note
EACH '[60 64 67] : >> [note: E] WAIT 0.25 END      # with timing

# Use I for index-based calculations
EACH '[60 64 67] : >> [note: E vel: SUB 127 MUL I 20] END

# With variable
SET G.NOTES '[48 55 60 64]
EACH G.NOTES : >> [note: E vel: 100] END
```

### Periodic Execution

Execute every Nth iteration (counter persists per-line):
```
EVERY n : body END
RANGE 0 7 : EVERY 2 : >> [note: 60] END END    # triggers on 0, 2, 4, 6
```

### Euclidean Rhythm

Distribute hits evenly across steps using the Euclidean algorithm:
```
EU hits steps dur : body END
EU hits steps dur : body ELSE : miss_body END
```

- `hits`: Number of hits to distribute
- `steps`: Total number of steps
- `dur`: Duration of each step in beats
- `I`: Step index (0 to steps-1) available in body
- Body executes on hit positions, ELSE body on misses

```
EU 3 8 0.125 : >> [note: 60] END                    # classic tresillo
EU 5 8 0.125 : >> [note: 60] END                    # cinquillo
EU 3 8 0.125 : >> [note: 60] ELSE : >> [note: 60 vel: 20] END  # with ghost notes

# Use I for velocity curves
EU 4 8 0.125 : >> [note: 60 vel: - 127 * I 10] END

# Evolution - hits can be a variable or expression
EU G.HITS 8 0.125 : >> [note: 60] END
EU + 2 G.X 8 0.125 : >> [note: 60] END
```

### Binary Rhythm

Use an integer's binary representation to define a rhythm pattern:
```
BIN pattern dur : body END
BIN pattern dur : body ELSE : miss_body END
```

- `pattern`: Integer whose bits determine hits (1) and misses (0)
- `dur`: Duration of each step in beats
- Steps = number of significant bits (64 - leading zeros), 0 if pattern is 0
- MSB first (left-to-right reading)
- `I`: Step index (0 to steps-1) available in body

```
BIN 5 0.125 : >> [note: 60] END       # 5 = 101: 3 steps, hits at 0 and 2
BIN 7 0.125 : >> [note: 60] END       # 7 = 111: 3 steps, all hits
BIN 170 0.125 : >> [note: 60] END     # 170 = 10101010: 8 steps, alternating
BIN 255 0.125 : >> [note: 60] END     # 255 = 11111111: 8 steps, all hits

# With ELSE for ghost notes
BIN 5 0.125 : >> [note: 60 vel: 100] ELSE : >> [note: 60 vel: 20] END

# Pattern from variable (can evolve over time)
SET G.PAT 5
BIN G.PAT 0.125 : >> [note: 60] END

# Use I for velocity curves
BIN 15 0.125 : >> [note: 60 vel: - 127 * I 20] END
```

Common patterns:
- `1` = single hit
- `5` = 101 (tresillo-like)
- `9` = 1001 (four-on-floor kick)
- `170` = 10101010 (offbeats)
- `255` = 11111111 (all 8th notes)

### Probabilistic

Execute with percentage probability:
```
PROB percent : body END
PROB percent : body ELSE : body END

PROB 50 : >> [note: 60] END    # 50% chance
```

### Switch

```
SWITCH expr :
    CASE val : body
    CASE val : body
    DEFAULT : body
END
```

### Break

Exit script immediately:
```
BREAK
```

## Concurrency

### Fork

Spawn a concurrent execution branch. The main script continues immediately while the forked branch runs in parallel:

```
FORK : body END
FORK { body }

FORK : DO 4 : >> [note: 60] WAIT 0.25 END END
SET G.X 99    # continues immediately
```

## Functions

### Definition

Function names must be uppercase (2+ characters). Arguments are single uppercase letters:

```
FUNC NAME A B :
    body
END
```

Implicit return (last expression):
```
FUNC DOUBLE X : MUL X 2 END
```

The last expression in the function body is the return value:
```
FUNC GCD A B :
    WHILE NE B 0 :
        SET G.T B;
        SET B MOD A B;
        SET A G.T
    END;
    A
END
```

### Call

Function calls require parentheses with CALL:
```
SET G.Y (CALL DOUBLE 5)       # G.Y = 10
SET G.Z (CALL GCD 48 18)      # G.Z = 6
```

### Lambda

Anonymous functions stored in variables:
```
SET G.F FN X : MUL X 2 END
SET G.Y (CALL G.F 5)          # G.Y = 10

# Multi-argument
SET G.G FN A B : ADD A B END

# With body
SET G.H FN X :
    SET G.temp MUL X X;
    ADD G.temp 1
END
```

## Selection

### Random Choice

Pick one option randomly each evaluation:
```
SET G.X CHOOSE: 1 2 3 4 END
```

### Alternating Cycle

Cycle through options sequentially (state persists per-line):
```
RANGE 0 5 : SET G.X ALT: 10 20 30 END END    # G.X cycles: 10, 20, 30, 10, 20, 30
```

## Maps

### Creation

```
SET G.M MNEW                       # empty map
SET G.M [note: 60 vel: 100]        # literal
SET G.M [a: ADD 1 2 b: MUL 3 4]    # with expressions
```

### Operations

```
SET G.V MGET G.M "key"             # get value
SET G.B MHAS G.M "key"             # check existence (bool)
SET G.M MSET G.M "key" value       # set key (returns new map)
SET G.L MLEN G.M                   # get map length
SET G.M MMERGE G.M1 G.M2           # merge maps (second wins on conflict)
```

### Map Operations with BOR and ADD

```
# BOR: union (first wins on conflict)
SET G.M BOR [x: 1] [x: 2 y: 3]     # x=1, y=3

# ADD: recursive merge (values added for matching keys)
SET G.M ADD [x: 1] [x: 2]          # x=3
```

## Lists

Lists are ordered collections of values. The syntax uses a leading quote to distinguish from maps.

### Creation

```
SET G.L '[1 2 3]                   # list of integers
SET G.L '[60 64 67 72]             # MIDI notes for a chord
SET G.L '["a" "b" "c"]             # list of strings
SET G.L '[ADD 1 2 MUL 3 4]         # with expressions: '[3 12]
SET G.L '[]                        # empty list
```

### Operations

- **`LEN`**: Get list length. `LEN '[1 2 3]` → 3
- **`GET`**: Get element at index (wraps). `GET '[10 20 30] 1` → 20
- **`PICK`**: Random element. `PICK '[1 2 3]` → random element
- **`CYCLE`**: Sequential cycling (state persists per-line). `CYCLE '[1 2 3]` → 1, 2, 3, 1, 2, ...

Index wrapping: negative indices wrap from end, positive indices wrap with modulo.
```
GET '[10 20 30] -1             # 30 (last element)
GET '[10 20 30] 3              # 10 (wraps to index 0)
```

### MAP

Apply a function to each element, returning a new list:

```
MAP fn list

SET G.X MAP FN A : MUL A 2 END '[1 2 3]        # '[2 4 6]

FUNC OCTAVE N : ADD N 12 END
SET G.Y (CALL MAP OCTAVE '[60 64 67])          # '[72 76 79]
```

### FILTER

Keep only elements where the predicate returns true:

```
FILTER fn list

SET G.X FILTER FN A : GT A 60 END '[48 60 72 84]    # '[72 84]
```

### REDUCE

Fold a list into a single value:

```
REDUCE fn init list

SET G.X REDUCE FN A B : ADD A B END 0 '[1 2 3]    # 6 (sum)
SET G.X REDUCE FN A B : MUL A B END 1 '[1 2 3 4]  # 24 (product)
```

### With Loops

```
# Using CYCLE for sequential patterns
RANGE 0 3 : >> [note: CYCLE '[60 64 67 72]] END

# Using GET with loop index
SET G.NOTES '[60 64 67]
RANGE 0 2 : >> [note: GET G.NOTES I] END
```

## Events

Events are emitted using `>>`, `@`, or `PLAY` followed by a map. The map keys determine the event type. Event dispatch follows a priority order: transport events are checked first, then SysEx, CC, program change, aftertouch, channel pressure, OSC, MIDI note, and finally generic events.

### Emit Syntax

```
>> [key: value ...]            # standard
>> [key: value ...]            # alternative
@ [key: value ...]             # alternative
PLAY [key: value ...]          # alternative
>> G.M                         # from map variable
```

### Device Selection

The `dev` key specifies which output device receives the event. If omitted, events go to the default device (0) or the device set by `DEV`:

```
DEV 1                          # set default device for this frame
>> [note: 60]                  # goes to device 1
>> [note: 72 dev: 2]           # override: goes to device 2
```

### MIDI Note

Triggered when the map contains `note` or `vel`.

| Key | Description | Default |
|-----|-------------|---------|
| `note` | MIDI note number (0-127) or note symbol | 60 |
| `vel` | Velocity (0-127) | 100 |
| `chan` | MIDI channel (0-15) | 0 |
| `dur` | Note duration in beats | 0.5 |
| `dev` | Output device | 0 |

```
>> [note: 60 vel: 100 chan: 0 dur: 0.25]
>> [note: :c3 vel: 100]        # using note symbol
```

### MIDI Control Change

Triggered when the map contains `cc`.

| Key | Description | Default |
|-----|-------------|---------|
| `cc` | Controller number (0-127) | required |
| `val` | Controller value (0-127) | 0 |
| `chan` | MIDI channel (0-15) | 0 |
| `dev` | Output device | 0 |

```
>> [cc: 1 val: 64 chan: 0]
```

### MIDI Program Change

Triggered when the map contains `pc`.

| Key | Description | Default |
|-----|-------------|---------|
| `pc` | Program number (0-127) | required |
| `chan` | MIDI channel (0-15) | 0 |
| `dev` | Output device | 0 |

```
>> [pc: 5 chan: 2]
```

### MIDI Aftertouch

Polyphonic aftertouch. Triggered when the map contains both `at` and `note`.

| Key | Description | Default |
|-----|-------------|---------|
| `at` | Pressure value (0-127) | required |
| `note` | Note number (0-127) | required |
| `chan` | MIDI channel (0-15) | 0 |
| `dev` | Output device | 0 |

```
>> [at: 100 note: 60 chan: 0]
```

### MIDI Channel Pressure

Triggered when the map contains `pressure`.

| Key | Description | Default |
|-----|-------------|---------|
| `pressure` | Pressure value (0-127) | required |
| `chan` | MIDI channel (0-15) | 0 |
| `dev` | Output device | 0 |

```
>> [pressure: 80 chan: 0]
```

### MIDI Transport

Transport events require a truthy value (non-zero). They have the highest priority.

| Key | Description |
|-----|-------------|
| `start` | Send MIDI Start |
| `stop` | Send MIDI Stop |
| `continue` | Send MIDI Continue |
| `clock` | Send MIDI Clock |
| `reset` | Send MIDI Reset |

```
>> [start: 1]
>> [stop: 1]
>> [continue: 1]
>> [clock: 1]
>> [reset: 1]
```

### MIDI SysEx

Triggered when the map contains `sysex`. The value must be a `BYTES` block.

| Key | Description | Default |
|-----|-------------|---------|
| `sysex` | Byte sequence (BYTES block) | required |
| `dev` | Output device | 0 |

```
>> [sysex: BYTES: 240 67 32 0 247 END]
>> [sysex: BYTES: 240 G.X G.Y 247 END dev: 2]
```

### OSC

Triggered when the map contains `addr`. All other keys (except `dev`) are sent as OSC arguments in alternating key/value format.

| Key | Description | Default |
|-----|-------------|---------|
| `addr` | OSC address path | required |
| `dev` | Output device | 0 |
| *other* | Sent as OSC arguments | |

```
>> [addr: "/synth" freq: 440 amp: 0.5]
>> [addr: "/fx" dev: 2 delay: 0.25 feedback: 0.7]
```

### Generic Events

If no specific event type is matched, all key/value pairs are sent as a generic Dirt-style event.

```
>> [sound: "kick" speed: 1.5]
>> [s: "bd" gain: 0.8]
```

## Timing

Bob scripts execute within a frame-based timing system. Time advances only through explicit `WAIT` statements. Without `WAIT`, all events in a script fire simultaneously at the start of the frame.

### WAIT

The `WAIT` statement advances time by a duration expressed in beats:

```
WAIT 1                         # wait one beat
WAIT 0.5                       # wait half a beat
WAIT 0.25                      # wait quarter beat
WAIT DIV 1 8                   # wait eighth beat (computed)
```

### Sequencing Events

Events fire at the current time position. Use `WAIT` between events to create sequences:

```
>> [note: 60]                  # fires at t=0
WAIT 0.5
>> [note: 64]                  # fires at t=0.5
WAIT 0.5
>> [note: 67]                  # fires at t=1.0
```

### DEV

The `DEV` statement sets the default output device for subsequent events in the current frame:

```
DEV 1                          # all following events go to device 1
>> [note: 60]
>> [note: 64]
DEV 2                          # switch to device 2
>> [note: 48]
```

Individual events can override the default with the `dev` key.
