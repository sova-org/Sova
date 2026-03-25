# Cagire Language Reference

Cagire is a stack-based language for live coding music. Values are pushed onto a stack; words consume stack values and produce results. The general pattern is: push parameters, name a sound, emit with `.`.

## Data Types

- **Integer**: `42`, `-10`, `0`
- **Float**: `3.14`, `-0.5`, `0.25` (leading zero optional: `.25`)
- **String**: `"kick"`, `"sine"`, `"hello"`
- **Note name**: `c4` (60), `cs4`/`c#4` (61), `bb3` (58) — pushes MIDI number. French solfège: `do4` (60), `mib3` (58), `sol#3` (56). Names: do/ut, ré/re, mi, fa, sol, la, si/ti
- **Interval**: `P5` (7), `M3` (4), `m7` (10) — adds semitones to top of stack
- **Quotation**: `( ... )` — deferred code block, first-class value

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
| `linmap` | `(val inlo inhi outlo outhi -- mapped)` | Linear map to output range |
| `expmap` | `(val lo hi -- mapped)` | Exponential map (0-1 to range) |

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
coin if "kick" snd . else "snare" snd . then

;; quotation conditionals
( "kick" snd . ) coin ?       ;; execute if true
( "snare" snd . ) coin !?     ;; execute if false

;; ifelse: ( true-quot false-quot bool -- )
( 60 ) ( 72 ) coin ifelse note .

;; select: execute nth quotation (0-indexed)
( 60 ) ( 64 ) ( 67 ) step 3 mod select note .

;; apply: execute quotation unconditionally
( 2 * ) apply

;; map: apply quotation to each element on the stack
1 2 3 ( 2 * ) map

;; times: ( n quot -- ) repeat n times, @i = index
4 ( @i 60 + note . ) times
```

## Sound Pipeline

The core pattern: name a sound, set parameters, emit.

```
"kick" sound .              ;; play a sample
"sine" snd 440 freq .       ;; play an oscillator
60 note 100 velocity .      ;; MIDI note
clear                       ;; reset sound register
```

`sound` (alias `snd`) sets the sound name. `.` emits the event. `clear` resets the register.

| Word | Stack | Description |
|------|-------|-------------|
| `sound` / `snd` | `(name --)` | Set sound name |
| `.` | `(--)` | Emit current event |
| `clear` | `(--)` | Reset sound register |
| `all` | `(--)` | Apply current params to all subsequent sounds |
| `noall` | `(--)` | Clear global params set by `all` |

### Recording

| Word | Stack | Description |
|------|-------|-------------|
| `rec` | `(name --)` | Toggle recording audio to named sample |
| `overdub` / `dub` | `(name --)` | Toggle overdub layering |
| `orec` | `(name orbit --)` | Toggle recording a single orbit |
| `odub` | `(name orbit --)` | Toggle overdub on a single orbit |

## Sample Parameters

| Word | Description |
|------|-------------|
| `bank` | Sample bank suffix |
| `n` | Sample number |
| `time` | Time offset |
| `begin` / `end` | Sample start/end (0-1) |
| `speed` | Playback speed |
| `gate` | Gate duration (total note length, 0 = infinite sustain) |
| `voice` | Voice number |
| `orbit` | Orbit/bus |
| `cut` | Cut group |
| `reset` | Reset parameter |
| `stretch` | Time stretch factor (pitch-independent) |
| `slice` | Divide sample into N equal slices |
| `pick` | Select which slice to play (0-indexed, wraps) |

## Oscillator Parameters

| Word | Description |
|------|-------------|
| `note` | MIDI note number |
| `freq` | Frequency (Hz) |
| `detune` | Detune amount |
| `pw` | Pulse width |
| `spread` | Stereo spread |
| `mult` | Multiplier |
| `coarse` | Coarse tune (semitones) |
| `wave` / `waveform` | Oscillator waveform |
| `mirror` | Mirror |
| `warp` | Warp amount |
| `partials` | Number of active harmonics (additive source) |
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
| `postgain` | Post gain |
| `velocity` | Velocity |
| `attack` / `att` | Attack time |
| `decay` / `dec` | Decay time |
| `sustain` / `sus` | Sustain level |
| `release` / `rel` | Release time |
| `envdelay` / `envdly` | Envelope delay time |
| `hold` / `hld` | Envelope hold time |
| `adsr` | `(a d s r --)` Set all four |
| `ad` | `(a d --)` Attack + decay (sustain=0) |

## Filter

Lowpass (`lpf`), highpass (`hpf`), bandpass (`bpf`), ladder variants (`llpf`, `lhpf`, `lbpf`).

Each filter has frequency and resonance (Q) controls:

```
2000 lpf 0.5 lpq .
100 hpf .
```

### Lowpass

| Word | Description |
|------|-------------|
| `lpf` | Lowpass frequency |
| `lpq` | Lowpass resonance |

### Highpass

| Word | Description |
|------|-------------|
| `hpf` | Highpass frequency |
| `hpq` | Highpass resonance |

### Bandpass

| Word | Description |
|------|-------------|
| `bpf` | Bandpass frequency |
| `bpq` | Bandpass resonance |

### Ladder Filters

| Word | Description |
|------|-------------|
| `llpf` / `llpq` | Ladder lowpass frequency / resonance |
| `lhpf` / `lhpq` | Ladder highpass frequency / resonance |
| `lbpf` / `lbpq` | Ladder bandpass frequency / resonance |

### EQ & Comb

`eqlo`, `eqmid`, `eqhi`, `tilt`, `ftype`. Comb: `comb`, `combfreq`, `combfeedback`, `combdamp`.

## Effects

### Reverb

```
0.3 verb 0.75 verbdecay .
```

Words: `verb`, `verbdecay`, `verbdamp`, `verbpredelay`, `verbdiff`, `verbtype`, `verbchorus`, `verbchorusfreq`, `verbprelow`, `verbprehigh`, `verblowcut`, `verbhighcut`, `verblowgain`, `size`.

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

### Compressor

Sidechain compression routed by orbit.

```
0.8 comp 0 comporbit .
```

| Word | Description |
|------|-------------|
| `comp` | Sidechain duck amount (0-1) |
| `compattack` / `cattack` | Compressor attack time |
| `comprelease` / `crelease` | Compressor release time |
| `comporbit` / `corbit` | Sidechain source orbit |

### FM Synthesis

```
200 fm 2 fmh .
```

Words: `fm`, `fmh`, `fmshape`, `fm2`, `fm2h`, `fmalgo`, `fmfb`.

### Vibrato & Ring Mod

Vibrato: `vib`, `vibmod`, `vibshape`. AM: `am`, `amdepth`, `amshape`. RM: `rm`, `rmdepth`, `rmshape`.

### Wavetable

```
0.5 scan 2048 wtlen .
```

Words: `scan`, `wtlen`.

## Probability

| Word | Stack | Description |
|------|-------|-------------|
| `coin` | `(-- bool)` | 50/50 random boolean |
| `rand` | `(min max -- n\|f)` | Random in range |
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
| `cycle` | `(v..n n -- val)` | Cycle through n items by frame runs |
| `pcycle` | `(v..n n -- val)` | Cycle through n items by line iteration |
| `bounce` | `(v..n n -- val)` | Ping-pong cycle by frame runs |
| `pbounce` | `(v..n n -- val)` | Ping-pong cycle by line iteration |
| `index` | `(v..n n idx -- val)` | Select item at explicit index |
| `every` | `(quot n --)` | Execute every nth iteration |
| `except` | `(quot n --)` | Execute on all iterations except every nth |
| `every+` | `(quot n offset --)` | Every nth iteration with phase offset |
| `except+` | `(quot n offset --)` | Skip every nth iteration with phase offset |
| `bjork` | `(quot k n --)` | Euclidean distribution by frame runs |
| `pbjork` | `(quot k n --)` | Euclidean distribution by line iteration |
| `loop` | `(n --)` | Fit sample to n beats |
| `at` | `(v..n --)` | Looping block: re-executes body per delta. Close with `.` or `done` |
| `tempo!` | `(bpm --)` | Set global tempo |
| `speed!` | `(multiplier --)` | Set line speed multiplier |

## Generators

| Word | Stack | Description |
|------|-------|-------------|
| `..` | `(start end -- start..end)` | Integer sequence |
| `.,` | `(start end step -- ...)` | Stepped sequence |
| `gen` | `(quot n -- results..)` | Execute quotation n times |
| `geom..` | `(start ratio count -- ...)` | Geometric sequence |
| `euclid` | `(k n -- positions..)` | Euclidean rhythm as normalized positions (0.0-1.0) |
| `euclidrot` | `(k n r -- positions..)` | Euclidean positions with rotation |

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
200 8000 0.01 0.1 ead lpf . ;; percussive envelope on filter
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
| `islide` | `(start end dur -- str)` | Swell transition (slow start, fast finish) |
| `oslide` | `(start end dur -- str)` | Pluck transition (fast attack, slow settle) |
| `pslide` | `(start end dur -- str)` | Stair transition (8 discrete steps) |
| `jit` | `(min max period -- str)` | Random hold |
| `sjit` | `(min max period -- str)` | Smooth random |
| `drunk` | `(min max period -- str)` | Drunk walk |
| `ead` | `(min max a d -- str)` | Percussive envelope (AD) |
| `eadr` | `(min max a d r -- str)` | Percussive envelope with release (ADR) |
| `eadsr` / `env` | `(min max a d s r -- str)` | DAHDSR envelope modulation |
| `lpg` | `(min max depth --)` | Low pass gate (pairs amp envelope with lpf) |

## Context Variables

Read-only words that push current execution state:

| Word | Description |
|------|-------------|
| `step` | Current frame index |
| `beat` | Current beat position |
| `pattern` | Current line index |
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
| `velocity` / `vel` | `(v.. --)` | Set velocity |
| `chan` | `(v.. --)` | Set MIDI channel 1-16 |
| `device` / `dev` | `(v.. --)` | Set device slot 1-16 |
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

### Scales & Diatonic Harmony

Set a tonal center with `key!`, then use a scale word followed by `triad` or `seventh` to build diatonic chords from scale degrees:

```
c4 key! 0 major triad note sine snd .    ;; C major triad
c4 key! 4 minor seventh note sine snd .  ;; 5th degree minor seventh
```

| Word | Stack | Description |
|------|-------|-------------|
| `key!` | `(root --)` | Set tonal center |
| `triad` | `(degree -- n1 n2 n3)` | Diatonic triad from scale degree |
| `seventh` | `(degree -- n1 n2 n3 n4)` | Diatonic seventh from scale degree |
| `tp` | `(n --)` | Transpose all integers on stack by N semitones |

### Voicings

| Word | Stack | Description |
|------|-------|-------------|
| `inv` | `(a b c.. -- b c.. a+12)` | Inversion: bottom note up an octave |
| `dinv` | `(a b.. z -- z-12 a b..)` | Down inversion: top note down an octave |
| `drop2` | `(a b c d -- b-12 a c d)` | Drop-2 voicing |
| `drop3` | `(a b c d -- c-12 a b d)` | Drop-3 voicing |

### Chords

Push a root note, then a chord word to expand into intervals:

```
c4 maj .            ;; C major: 60 64 67
c4 min7 .           ;; C minor 7: 60 63 67 70
```

Triads: `maj`, `m`, `dim`, `aug`, `sus2`, `sus4`, `pwr`.
Sevenths: `maj7`, `min7`, `dom7`, `dim7`, `m7b5`, `minmaj7`, `aug7`, `augmaj7`, `7sus4`.
Extended: `dom9`, `maj9`, `min9`, `dom11`, `maj11`, `min11`, `dom13`, `maj13`, `min13`, `9sus4`.
Added: `add9`, `add11`, `madd9`.
Altered: `dom7b9`, `dom7s9`, `dom7b5`, `dom7s5`, `dom7s11`.
Sixths: `maj6`, `min6`, `maj69`, `min69`.

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

: kick "kick" snd 0.9 gain . ;
kick                ;; call defined word

"kick" forget       ;; remove definition
```

### Scoped Variables

Variables are Instance-scoped by default (local to the script). Use prefixes to share data across scripts:

```
!G.x  @G.x  ,G.x   ;; Global — all scripts in the session
!L.x  @L.x  ,L.x   ;; Line — all frames in the same line
!F.x  @F.x  ,F.x   ;; Frame — persists across runs of this frame
```

See the **Variables** article for details.

## Debug

```
42 print            ;; print top of stack to log
```
