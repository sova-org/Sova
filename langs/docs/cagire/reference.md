# Cagire Language Reference

Cagire is a stack-based language for live coding music. Values are pushed onto a stack; words consume stack values and produce results. The general pattern is: push parameters, name a sound, emit with `.`.

## Data Types

- **Integer**: `42`, `-10`, `0`
- **Float**: `3.14`, `-0.5`, `0.25`
- **String**: `"kick"`, `"sine"`, `"hello"`
- **Note name**: `c4` (60), `cs4`/`c#4` (61), `bb3` (58) — pushes MIDI number
- **Quotation**: `{ ... }` — deferred code block, first-class value

## Stack Operations

| Word | Stack | Description |
|------|-------|-------------|
| `dup` | `(a -- a a)` | Duplicate top |
| `dupn` | `(a n -- a..a)` | Duplicate a, n times |
| `drop` | `(a --)` | Remove top |
| `swap` | `(a b -- b a)` | Exchange top two |
| `over` | `(a b -- a b a)` | Copy second to top |
| `rot` | `(a b c -- b c a)` | Rotate top three |
| `nip` | `(a b -- b)` | Remove second |
| `tuck` | `(a b -- b a b)` | Copy top under second |
| `2dup` | `(a b -- a b a b)` | Duplicate top pair |
| `2drop` | `(a b --)` | Drop top pair |
| `2swap` | `(a b c d -- c d a b)` | Swap top two pairs |
| `2over` | `(a b c d -- a b c d a b)` | Copy second pair |
| `rev` | `(..n n -- ..n)` | Reverse top n items |
| `shuffle` | `(..n n -- ..n)` | Randomly shuffle top n |
| `sort` | `(..n n -- ..n)` | Sort top n ascending |
| `rsort` | `(..n n -- ..n)` | Sort top n descending |
| `sum` | `(..n n -- total)` | Sum top n items |
| `prod` | `(..n n -- product)` | Product of top n items |

## Arithmetic

| Word | Stack | Description |
|------|-------|-------------|
| `+` | `(a b -- a+b)` | Add |
| `-` | `(a b -- a-b)` | Subtract |
| `*` | `(a b -- a*b)` | Multiply |
| `/` | `(a b -- a/b)` | Divide |
| `mod` | `(a b -- a%b)` | Modulo |
| `neg` | `(a -- -a)` | Negate |
| `abs` | `(a -- \|a\|)` | Absolute value |
| `floor` | `(f -- n)` | Round down |
| `ceil` | `(f -- n)` | Round up |
| `round` | `(f -- n)` | Round to nearest |
| `min` | `(a b -- min)` | Minimum |
| `max` | `(a b -- max)` | Maximum |
| `pow` | `(a b -- a^b)` | Exponentiation |
| `sqrt` | `(a -- sqrt)` | Square root |
| `sin` | `(a -- sin)` | Sine (radians) |
| `cos` | `(a -- cos)` | Cosine (radians) |
| `log` | `(a -- ln)` | Natural logarithm |

## Comparison & Logic

| Word | Stack | Description |
|------|-------|-------------|
| `=` | `(a b -- bool)` | Equal |
| `!=` / `<>` | `(a b -- bool)` | Not equal |
| `lt` | `(a b -- bool)` | Less than |
| `gt` | `(a b -- bool)` | Greater than |
| `<=` | `(a b -- bool)` | Less or equal |
| `>=` | `(a b -- bool)` | Greater or equal |
| `and` | `(a b -- bool)` | Logical and |
| `or` | `(a b -- bool)` | Logical or |
| `not` | `(a -- bool)` | Logical not |
| `xor` | `(a b -- bool)` | Exclusive or |
| `nand` | `(a b -- bool)` | Not and |
| `nor` | `(a b -- bool)` | Not or |

## Control Flow

```
;; if / else / then
coin if "kick" s . else "snare" s . then

;; quotation conditionals
{ "kick" s . } coin ?       ;; execute if true
{ "snare" s . } coin !?     ;; execute if false

;; ifelse: ( true-quot false-quot bool -- )
{ 60 } { 72 } coin ifelse note .

;; pick: ( ..quots n -- ) execute nth quotation
{ 60 } { 64 } { 67 } step 3 mod pick note .

;; apply: execute quotation unconditionally
{ 2 * } apply

;; times: ( n quot -- ) repeat n times, @i = index
4 { @i 60 + note . } times
```

## Sound Pipeline

The core pattern: name a sound, set parameters, emit.

```
"kick" sound .              ;; play a sample
"sine" s 440 freq .         ;; play an oscillator
60 note 100 vel .           ;; MIDI note
clear                       ;; reset sound register
```

`sound` (alias `s`) sets the sound name. `.` emits the event. `clear` resets the register.

### Arpeggios

```
c4 e4 g4 arp note .         ;; wrap stack values as arpeggio list
```

## Sample Parameters

| Word | Description |
|------|-------------|
| `bank` | Sample bank suffix |
| `n` | Sample number |
| `time` | Time offset |
| `begin` / `end` | Sample start/end (0-1) |
| `speed` | Playback speed |
| `dur` | Duration |
| `gate` | Gate time |
| `repeat` | Repeat count |
| `voice` | Voice number |
| `orbit` | Orbit/bus |
| `cut` | Cut group |

## Oscillator Parameters

| Word | Description |
|------|-------------|
| `note` | MIDI note number |
| `freq` | Frequency (Hz) |
| `detune` | Detune amount |
| `glide` | Portamento |
| `pw` | Pulse width |
| `spread` | Stereo spread |
| `mult` | Multiplier |
| `harmonics` | Harmonics count |
| `timbre` | Timbre |
| `morph` | Morph |
| `sub` | Sub oscillator level |
| `suboct` | Sub octave |
| `subwave` | Sub waveform |

## Envelope

| Word | Description |
|------|-------------|
| `gain` | Volume (0-1) |
| `velocity` / `vel` | Velocity |
| `attack` / `att` | Attack time |
| `decay` / `dec` | Decay time |
| `sustain` / `sus` | Sustain level |
| `release` / `rel` | Release time |
| `adsr` | `(a d s r --)` Set all four |
| `ad` | `(a d --)` Attack + decay (sustain=0) |

## Filter

Lowpass (`lpf`), highpass (`hpf`), bandpass (`bpf`), ladder variants (`llpf`, `lhpf`, `lbpf`).

Each filter has frequency, resonance (Q), and envelope controls:

```
2000 lpf 0.5 lpq .
100 hpf .
0.5 lpe 0.01 lpa 0.1 lpd .   ;; filter envelope
```

EQ: `eqlo`, `eqmid`, `eqhi`, `tilt`. Comb: `comb`, `combfreq`, `combfeedback`, `combdamp`.

## Effects

### Reverb

```
0.3 verb 0.75 verbdecay .
```

Words: `verb`, `verbdecay`, `verbdamp`, `verbpredelay`, `verbdiff`, `verbtype`, `verbchorus`, `size`.

### Delay

```
0.3 delay 0.25 delaytime 0.5 delayfeedback .
```

Words: `delay`, `delaytime`, `delayfeedback`, `delaytype`.

### Distortion & Lo-fi

```
8 crush .
0.5 distort .
2 fold .
```

Words: `crush`, `fold`, `wrap`, `distort`, `distortvol`.

### Stereo

```
-0.5 pan .
0 width .
```

Words: `pan`, `width`, `haas`.

### Modulation FX

Phaser: `phaser`, `phaserdepth`, `phasersweep`, `phasercenter`.
Flanger: `flanger`, `flangerdepth`, `flangerfeedback`.
Chorus: `chorus`, `chorusdepth`, `chorusdelay`.
Feedback delay: `feedback`/`fb`, `fbtime`/`fbt`, `fbdamp`/`fbd`, `fblfo`, `fblfodepth`, `fblfoshape`.
Smear: `smear`, `smearfreq`, `smearfb`.

### FM Synthesis

```
200 fm 2 fmh 0.5 fme .
```

Words: `fm`, `fmh`, `fmshape`, `fme`, `fma`, `fmd`, `fms`, `fmr`, `fm2`, `fm2h`, `fmalgo`, `fmfb`.

### Vibrato & Ring Mod

Vibrato: `vib`, `vibmod`, `vibshape`. AM: `am`, `amdepth`, `amshape`. RM: `rm`, `rmdepth`, `rmshape`.

### Wavetable

```
0.5 scan 2048 wtlen .
```

Words: `scan`, `wtlen`, `scanlfo`, `scandepth`, `scanshape`.

## Probability

| Word | Stack | Description |
|------|-------|-------------|
| `coin` | `(-- bool)` | 50/50 random boolean |
| `rand` | `(min max -- n)` | Random in range |
| `exprand` | `(lo hi -- f)` | Exponential random (biased low) |
| `logrand` | `(lo hi -- f)` | Exponential random (biased high) |
| `seed` | `(n --)` | Set random seed |
| `chance` | `(quot prob --)` | Execute quotation with probability 0.0-1.0 |
| `prob` | `(quot pct --)` | Execute quotation with probability 0-100 |
| `choose` | `(..n n -- val)` | Random pick from n items |
| `wchoose` | `(v1 w1 .. n -- val)` | Weighted random pick |
| `always` | `(quot --)` | Always execute (100%) |
| `almostAlways` | `(quot --)` | 90% |
| `often` | `(quot --)` | 75% |
| `sometimes` | `(quot --)` | 50% |
| `rarely` | `(quot --)` | 25% |
| `almostNever` | `(quot --)` | 10% |
| `never` | `(quot --)` | Never execute (0%) |

## Sequencing

| Word | Stack | Description |
|------|-------|-------------|
| `cycle` | `(v..n n -- val)` | Cycle through n items by step |
| `pcycle` | `(v..n n -- val)` | Cycle through n items by pattern |
| `bounce` | `(v..n n -- val)` | Ping-pong cycle |
| `every` | `(quot n --)` | Execute every nth iteration |
| `bjork` | `(quot k n --)` | Euclidean distribution by step |
| `pbjork` | `(quot k n --)` | Euclidean distribution by pattern |
| `loop` | `(n --)` | Fit sample to n beats |
| `at` | `(v..n --)` | Set delta timing for emit |
| `chain` | `(bank pattern --)` | Chain to next pattern |

## Generators

| Word | Stack | Description |
|------|-------|-------------|
| `..` | `(start end -- start..end)` | Integer sequence |
| `.,` | `(start end step -- ...)` | Stepped sequence |
| `gen` | `(quot n -- results..)` | Execute quotation n times |
| `geom..` | `(start ratio count -- ...)` | Geometric sequence |
| `euclid` | `(k n -- indices..)` | Euclidean rhythm indices |
| `euclidrot` | `(k n r -- indices..)` | Euclidean indices with rotation |

## LFO & Ramps

| Word | Stack | Description |
|------|-------|-------------|
| `ramp` | `(freq curve -- val)` | Ramp 0-1 |
| `linramp` | `(freq -- val)` | Linear ramp (curve=1) |
| `expramp` | `(freq -- val)` | Exponential ramp (curve=3) |
| `logramp` | `(freq -- val)` | Logarithmic ramp (curve=0.3) |
| `triangle` | `(freq -- val)` | Triangle wave 0-1 |
| `perlin` | `(freq -- val)` | Perlin noise 0-1 |
| `range` | `(val min max -- scaled)` | Scale 0-1 to min-max |

## Audio-rate Modulation

These words produce modulation strings that can be passed to any parameter:

```
200 4000 2 lfo lpf .        ;; sine LFO on filter
0 1 0.01 slide gain .       ;; fade in
```

| Word | Stack | Description |
|------|-------|-------------|
| `lfo` | `(min max period -- str)` | Sine oscillation |
| `tlfo` | `(min max period -- str)` | Triangle oscillation |
| `wlfo` | `(min max period -- str)` | Sawtooth oscillation |
| `qlfo` | `(min max period -- str)` | Square oscillation |
| `slide` | `(start end dur -- str)` | Linear transition |
| `expslide` | `(start end dur -- str)` | Exponential transition |
| `sslide` | `(start end dur -- str)` | Smooth transition |
| `jit` | `(min max period -- str)` | Random hold |
| `sjit` | `(min max period -- str)` | Smooth random |
| `drunk` | `(min max period -- str)` | Drunk walk |
| `env` | `(start t1 d1 ... -- str)` | Multi-segment envelope |

## Context Variables

Read-only words that push current execution state:

| Word | Description |
|------|-------------|
| `step` | Current frame index |
| `beat` | Current beat position |
| `pattern` | Current line index |
| `pbank` | Current pattern bank |
| `tempo` | Current BPM |
| `phase` | Phase in bar (0-1) |
| `slot` | Current line index |
| `runs` | Times this frame has triggered |
| `iter` | Line iteration count |
| `stepdur` | Frame duration in seconds |
| `fill` | Fill toggle |

## MIDI

| Word | Stack | Description |
|------|-------|-------------|
| `note` | `(v.. --)` | Set MIDI note |
| `vel` / `velocity` | `(v.. --)` | Set velocity |
| `chan` | `(v.. --)` | Set MIDI channel 1-16 |
| `dev` | `(v.. --)` | Set device slot 1-16 |
| `ccnum` | `(v.. --)` | Set CC number |
| `ccout` | `(v.. --)` | Set CC value |
| `ccval` | `(cc chan -- val)` | Read CC from MIDI input |
| `bend` | `(v.. --)` | Pitch bend (-1.0 to 1.0) |
| `pressure` | `(v.. --)` | Channel pressure 0-127 |
| `program` | `(v.. --)` | Program change 0-127 |
| `mclock` | `(--)` | Send MIDI clock |
| `mstart` | `(--)` | Send MIDI start |
| `mstop` | `(--)` | Send MIDI stop |
| `mcont` | `(--)` | Send MIDI continue |

## Music Theory

### Chords

Push a root note, then a chord word to expand into intervals:

```
c4 maj .            ;; C major: 60 64 67
c4 min7 .           ;; C minor 7: 60 63 67 70
```

Triads: `maj`, `m`, `dim`, `aug`, `sus2`, `sus4`.
Sevenths: `maj7`, `min7`, `dom7`, `dim7`, `m7b5`, `minmaj7`, `aug7`.
Extended: `dom9`, `maj9`, `min9`, `dom11`, `min11`, `dom13`.
Added: `add9`, `add11`, `madd9`.
Altered: `dom7b9`, `dom7s9`, `dom7b5`, `dom7s5`.
Sixths: `maj6`, `min6`.

### Conversion

| Word | Stack | Description |
|------|-------|-------------|
| `mtof` | `(midi -- hz)` | MIDI to frequency |
| `ftom` | `(hz -- midi)` | Frequency to MIDI |

## Variables & Definitions

```
440 !freq           ;; store (consumes)
@freq               ;; recall
440 ,freq           ;; store (keeps on stack)

: kick "kick" s 0.9 gain . ;
kick                ;; call defined word

"kick" forget       ;; remove definition
```

## Debug

```
42 print            ;; print top of stack to log
```
