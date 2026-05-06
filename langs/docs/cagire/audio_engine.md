Doux is the audio engine that ships with Sova. When you write `sine snd c4 note 0.5 gain .`, almost every word in that line is an instruction to Doux. The oscillators, the envelopes, the filters, the reverbs, the crushers, the compressors: they all live inside Doux. Understanding Doux is understanding what most of Cagire does. This article is a tour of what is in there and how Cagire talks to it.

For the mechanics of handing values to parameters and changing them over time, see the [Modulations](#) article. For sending notes to external gear instead of Doux, see [MIDI](#).

## The Sound Register

Cagire does not call Doux directly. It fills an internal accumulator called the *sound register*, then hands the whole thing over when you emit with `.`. Three words drive it:

```forth
sine sound      ;; open a new sound, name it "sine"
0.5 gain        ;; add a parameter
.               ;; emit: the register is packaged and sent to Doux
```

`sound` (or its shorthand `snd`) opens the register and sets the instrument or sample name. Parameter words like `gain`, `freq`, `lpf`, `verb`, `pan` attach values to it. Nothing reaches Doux until `.` fires. If you need to throw the register away without emitting, use `clear`:

```forth
kick snd 0.5 gain      ;; filling the register
clear                  ;; register is emptied
hat snd .              ;; only the hat plays
```

The order of parameters between `snd` and `.` does not matter. You can emit several times per frame.

## What Lives Inside Doux

Doux is organized into families of parameters. This section walks through them at a glance. Each family has its own word reference entries with examples.

### Sources

The first thing `snd` expects is a source name. Doux has a fixed set, grouped into a few families.

*Tonal oscillators* produce pitched waveforms: `sine`, `tri` (alias `triangle`), `saw` (alias `sawtooth`), `zaw` (naive aliased sawtooth), `pulse` (alias `square`), `pulze` (naive aliased pulse), `add` (32 partial additive), and `osc` (a morphing oscillator that blends sine, triangle, saw, and square).

```forth
sine snd c4 note 0.5 gain .
```

```forth
add snd c3 note 0.3 gain .
```

*Noise* comes in three colours: `white`, `pink`, `brown`.

```forth
pink snd 0.01 0.3 ad 0.4 gain .
```

*Drums* are procedural synths, not samples. Each one has its own internal recipe and responds to shaping parameters the same way tonal sources do: `kick`, `snare` (alias `sd`), `hat` (aliases `hh`, `hihat`), `tom`, `rim` (aliases `rimshot`, `rs`), `cowbell` (alias `cb`), `cymbal` (alias `cy`).

```forth
kick snd .
```

```forth
snare snd 0.7 gain .
```

*Sample playback* lives behind three source names. `sample` plays an audio file from disk with pitch tracking, `wt` treats a file as a morphable wavetable, and `gm` routes to a General MIDI soundfont.

*Live audio* from the input bus is available as `live` (alias `mic`).

Once the source is set, timbre words shape its character: `note` and `freq` set pitch, `detune` spreads voices, `pw` sets pulse width on pulse sources, `sub` and `subwave` add a sub oscillator, `warp`, `harmonics`, `timbre`, `morph`, and `wave` push the spectral character on sources that support them. FM is a modulation that applies to any oscillator through `fm`, `fmh`, `fm2`, `fm2h`, `fmalgo`, and `fmfb`. For `sample` and `wt`, `n` picks a file or slot, `begin` and `end` trim playback, `speed` retunes it, `slice` and `pick` chop it into grains.

```forth
sine snd c3 note 0.01 0.4 ad 3 fm 2 fmh .
```

### Envelopes

Every sound has an amplitude envelope. Cagire gives you the full ADSR:

```forth
saw snd c3 note 0.01 0.2 0.6 0.4 adsr .
```

And a shorthand for an attack and decay percussive shape:

```forth
saw snd c3 note 0.01 0.3 ad .
```

The individual words are `attack` / `att` / `a`, `decay` / `dec` / `d`, `sustain` / `sus` / `s`, `release` / `rel` / `r`, plus `envdelay`, `hold`, and `gate` for the sustain phase length. `gain` and `postgain` set level; `velocity` adjusts touch sensitivity.

### Filters

Four filter families sit in every voice.

The standard pair `lpf` / `hpf` / `bpf` with matching resonance `lpq` / `hpq` / `bpq` covers most needs:

```forth
saw snd c3 note 0.01 0.4 ad 1200 lpf 0.4 lpq .
```

The ladder variants `llpf` / `lhpf` / `lbpf` and their `l?q` pairs have more character and drive harder. A three band EQ uses `eqlo`, `eqmid`, `eqhi` with their matching `eqlofreq`, `eqmidfreq`, `eqhifreq`, plus `tilt` for a single knob that goes from bright to dark. Finally a comb filter family (`comb`, `combfreq`, `combfeedback`, `combdamp`) for metallic resonances. `ftype` switches between filter algorithms where multiple are available.

### Reverb

One reverb per voice, with a rich parameter set:

```forth
sine snd c4 note 0.4 verb .
```

The full palette: `verb` (mix), `verbdecay`, `verbdamp`, `verbpredelay`, `verbdiff` (diffusion), `verbchorus` and `verbchorusfreq` for modulation inside the tail, `verbprelow` and `verbprehigh` to filter the input, `verblowcut` / `verbhighcut` / `verblowgain` for tail shaping, and `size`. `verbtype` picks the algorithm.

### Delay

A standard delay and a richer feedback delay live side by side. The standard delay uses `delay`, `delaytime`, `delayfeedback`, `delaytype`. The feedback delay uses `fb` / `feedback`, `fbtime` / `fbt`, `fbdamp` / `fbd`, plus its own internal LFO via `fblfo`, `fblfodepth`, `fblfoshape`.

```forth
saw snd c3 note 0.01 0.3 ad 0.4 delay 0.375 delaytime .
```

### Modulation Effects

Phaser, flanger, chorus, and an allpass smear. Each has a mix and depth control with extras.

Phaser:

```forth
saw snd c3 note 0.5 phaser 0.6 phaserdepth 0.4 phasersweep .
```

Flanger:

```forth
saw snd c3 note 0.4 flanger 0.5 flangerdepth 0.5 flangerfeedback .
```

Chorus:

```forth
saw snd c3 note 0.5 chorus 0.6 chorusdepth 0.3 chorusdelay .
```

Smear:

```forth
saw snd c3 note 0.5 smear 800 smearfreq 0.4 smearfb .
```

### Dirt and Distortion

Hard color: `crush` (bit crush), `fold` (wave folding), `wrap` (wave wrapping), `distort` with `distortvol`.

```forth
saw snd c3 note 0.6 distort 0.3 distortvol 0.5 fold .
```

### Space and Stereo

`pan` moves a voice left to right between -1 and 1. `width` sets the stereo width (0 mono, 1 normal, 2 wider). `haas` adds a small delay between the two channels in milliseconds for a sense of stereo placement.

### Compressor and Sidechain

A single sidechain compressor sits on the output. `comp` sets the duck amount, `cattack` and `crelease` shape its response, `corbit` names which orbit triggers the duck.

## Orbits

Voices in Doux route through numbered effect buses called *orbits*. The `orbit` word picks which bus a voice goes through:

```forth
kick snd 0 orbit .
```

```forth
saw snd c4 note 0.5 verb 1 orbit .
```

Different voices sharing the same orbit also share that orbit's effect state, which is how you get a single long reverb tail that several voices pour into. Orbits are also what `corbit` targets when you build a sidechain, and how `orec` and `odub` capture audio in the [Recording](#) article.

## Skipping a Voice

`rest` pushes a silence value. Any sound or parameter that resolves to silence cancels its emission for that polyphony slot. Useful for chords that drop a voice on certain cycles:

```
< 60 rest > 67 note sine snd .
```

Here the first slot alternates between note 60 and silence, while the second slot keeps emitting note 67.

## Where to Go Next

Now that you know what is inside Doux, see [Modulations](#) for how values flow from the stack to the sound register, how to change them from one frame to the next, and how to hand Doux continuous movements that sweep parameters smoothly.
