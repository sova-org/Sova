Doux is Sova's built-in synthesizer. It is a general-purpose audio engine that covers subtractive, additive, frequency modulation, wavetable, and sample-based synthesis in a single voice architecture. Think of it as a synth that does a bit of everything. Doux started as a Rust port of [Dough](https://dough.strudel.cc/), a C engine by Felix Roos from the [TidalCycles](https://tidalcycles.org)/[Strudel](https://strudel.cc) ecosystem. It then grew into its own project with extensions tailored for Sova. 

Doux runs inside the server and produces audio directly. No external software or hardware is needed to play music with Sova. When the server starts, the engine starts with it and occupies [device](devices) slot 1. There is nothing to install or configure to begin making sound. All [languages](languages) can send events to it. The Doux tab in the documentation panel covers every parameter in detail.

Doux has a lot of merits. Thanks to Doux, Sova can produce sound as a standalone application. The engine is small / performant enough to be used on a large range of machines, from potato to high-end computers. Doux combines a lot of sound synthesis and sound processing techniques, meaning that exploring all its parameters can lead you to very different regions of the sound/timbre space.

## Sound sources

A `source` is the raw material of a voice. Start by picking one: an oscillator, a sample, a soundfont sample, a live microphone. The source then enters a shared pipeline of filters, envelope, modulation and effects. The source determines the starting timbre; everything downstream applies identically regardless of which source you chose. Sources are raw sound materials that you will shape and transform by coding.

### Oscillators

Oscillators are the simplest sources you can work with, and they are available everywhere, all the time. Six basic waveforms are available as of now: sine, triangle, band-limited sawtooth, raw sawtooth (aliased), pulse with variable width, and raw pulse. Even these basic shapes go further than you might expect. Three phase-shaping modifiers (warp, mirror, and size) transform the waveform without changing pitch. A sub oscillator adds a tone one to three octaves below in triangle, sine, or square. Unison spread layers seven detuned copies across the stereo field. Oscillators can quickly go from very simple tone generators to complex timbre generators, especially when modulated.

Three specialized oscillators also extend the engine. The additive oscillator builds timbres from one to thirty-two sine partials. Timbre, morph, and harmonics are parameters that you can use to shape the spectrum continuously. FM synthesis lets oscillators modulate each other: single or dual operator, with three routing algorithms and feedback on the topmost operator. The wavetable oscillator reads any audio sample as a looping waveform with a scan parameter that sweeps through its cycles. These special oscillators can cover a wide range of timbres expected from a modern synth.

### Samples, SoundFonts and Synth Drums

Instead of generating waveforms, you can start to play using audio samples. Doux will then become a very capable sampler that can do looping, stretching, slicing, etc. Sample banks for Doux should be organized in folders on disk, containing as many files per folder as you need. Each sample can be sliced with start and end points and modulated: playback speed, pitch-independent time stretching, choke groups for mutual voice silencing, etc. WAV, FLAC, OGG, and MP3 files are supported. We also offer an experimental soundfont playback support that loads SF2 files (`.sf2`) and exposes 128 General MIDI presets selectable by name or program number. Sample bank size is not limited whatsoever, you can load 30GB of audio samples if you feel like it, and they will be lazily loaded when required.

The engine also provides seven synthesized drum instruments: `kick`, `snare`, `hi-hat`, `tom`, `rimshot`, `cowbell`, `cymbal`. Each drum model comes with parameters for pitch, tone, and noise content. These models are not sample-based. The engine generates them from oscillators and noise, so they respond to the same shaping pipeline as any other source.

### Live input

A microphone or line-in can serve as the source, routing external audio through the full pipeline. By default, live input is monophonic but you can spawn one voice per audio input, meaning that you can capture 5 or 6 instruments if you feel like it. The engine then becomes a real-time effects processor. You can live code your guitar sound or record and sample a friend playing the trumpet. Any source feeds into the same filters, effects, and routing described below.

## Sound shaping

Once you have defined a source, you enter in a different stage, where you can specify what effects to apply to a voice. It is useful to remind yourself that there are three ways to modulate a parameter and to play with audio engine parameters:

- **initial rate**: the simplest, you don't modulate anything. You specify a fixed value for a parameter, and that value is a constant. Using initial rate values is good to learn how the engine works. However, it can lead to stagnation, as timbres will never evolve.
- **control rate**: use your programming language, on the event side, to pick a different value for an engine parameter depending on what you do: sequences, randomness, etc. This is the way live coders generally live code their synthesis parameters.
- **audio rate**: most Doux parameters can be modulated audio rate, meaning that you can have very precise modulators (envelopes, low frequency oscillators, ramps) that will change the value of an engine parameter over time. This leads to very fluid textures and continuously evolving timbres.

Once you are aware of that, you can start digging into the engine and discover the range of effects available:

- **Filters**: Three filter families. State variable filters provide lowpass,
highpass, and bandpass modes with resonance. Moog-style ladder filters add a
four-pole response with nonlinear saturation and self-oscillation. Biquad
filters cover eight modes: lowpass, highpass, bandpass, notch, allpass,
peaking, low shelf, and high shelf.

- **Envelope**: A six-stage DAHDSR envelope controls amplitude: delay, attack,
hold, decay, sustain, release. Attack and decay/release stages have independent
curve shapes.

- **Modulation**: Vibrato applies a pitch LFO. Amplitude modulation and ring
modulation shape the signal level and spectrum respectively. LFO waveforms
include sine, triangle, sawtooth, square, and sample-and-hold.

- **Effects**: The effects chain splits into two stages. Per-voice insert effects
process each voice independently. Modulation inserts include phaser, flanger,
three-voice stereo chorus, and allpass resonance. Distortion covers soft-clip
saturation, Serge-style wavefolding, wavewrapping, bit crushing, and sample
rate reduction. A three-band parametric EQ and a tilt EQ handle tone shaping.
Stereo placement uses the Haas effect and a width control ranging from mono to
exaggerated stereo.

After the insert effects, voices send to shared orbit buses for reverb, delay, comb filtering, and dynamics. Two reverb algorithms are available: a Dattorro plate and a modern variant (taken from Vital) with built-in chorus modulation, shelf EQ, and pre-filtering. Delay comes in four flavors: standard, ping-pong, tape with saturation, and multitap — plus a separate feedback delay with LFO-modulated time. A resonant comb filter and a sidechain compressor for ducking between orbits round out the bus effects.

## Polyphony and routing

Doux is a polyphonic synthesizer/audio engine. You start with polyphony set at 32 simultaneous voices. This voice limit is very conservative and can be freely configured in the options. Eight independent effect bus orbits let you group voices and apply different send-effect settings to each group. One orbit for drums with a short reverb, another for pads with a long tail, for instance, etc. The engine runs inside the server. In a [multiplayer](multiplayer) session, all connected musicians share the same engine instance. Any client can trigger sound. When idle, the engine consumes no significant resources.

## Recording and Live Sampling

The engine can record its own master output or a single orbit output into named samples. These samples can be recorded up to 60 seconds each. An overdub mode layers new audio on top of existing recordings. Captured samples are immediately available for playback through the full engine, with all effects and time stretching. See the language tabs for the recording syntax.

This feature is a bit experimental but also super fun. It allows you to do live resampling of your own output, which opens up an interesting space for experimentation, especially when coupled with live input and/or when you play with rich modulated sound sources.

## Monitoring

The Audio panel configures the engine: output device, input device, sample
paths, voice limit, buffer size, and audio channels. When the engine is
running, the panel also displays live status — active voices, peak voice count,
CPU load, and sample rate. A restart button applies configuration changes
without restarting the server.

Four visualization panels monitor the audio output in real time. All four
can be toggled/untoggled only when the engine is active.

- **Scope** — waveform display with configurable smoothing, stroke width, and
  fill transparency. Detachable as a separate window.
- **Spectrum** — frequency analyzer spanning 20 Hz to 20 kHz across 128 bands,
  with smoothing, bar gap, and gradient controls. Also detachable.
- **VU meter** — peak level meters for each output channel, displayed as a side
  panel. Three color zones (green, yellow, red) mark safe, caution, and
  clipping ranges. Peaks hold briefly before decaying.
- **Scope bar** — a compact waveform strip at the bottom of the screen, useful
  as a persistent visual indicator without opening the full scope.

All visualization settings (smoothing, sizes, detached state) persist across
sessions.
