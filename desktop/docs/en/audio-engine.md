# Audio Engine

Doux is Sova's built-in synthesizer — a general-purpose audio engine that
covers subtractive, additive, FM, wavetable, and sample-based synthesis in a
single voice architecture. Think of it as a synth that does a bit of
everything. It runs inside the server and produces audio directly, no external
software or hardware needed. When the server starts, Doux starts with it and
occupies [device](devices) slot 1. There is nothing to install or configure to
begin making sound. All four [languages](languages) can send events to it.
Cagire offers direct control over every synthesis parameter; Bob, BaLi, and
Boinx route note events through the same engine with the default voice. The
Doux tab in the documentation panel covers every parameter in detail.

## I. Sound sources

**Oscillators.** Sine, triangle, band-limited sawtooth, raw sawtooth, pulse
with variable width, and raw pulse. Three phase-shaping modifiers reshape the
waveform without changing pitch: warp applies a power curve, mirror reflects
the phase at a configurable position, and size quantizes it into discrete steps
for lo-fi textures. A sub oscillator adds a secondary tone one to three octaves
below the main pitch in triangle, sine, or square. Unison spread layers seven
detuned copies across the stereo field.

**Additive synthesis.** Builds timbres by stacking one to thirty-two sine
partials. Three controls shape the spectrum: timbre tilts the balance between
low and high partials, morph shifts the weight between even and odd harmonics,
and harmonics stretches the partial spacing away from the harmonic series.

**FM synthesis.** Single or dual operator frequency modulation with three
routing algorithms: cascade, parallel, and branch. The topmost operator
supports feedback. Each modulator can use sine, triangle, sawtooth, square, or
sample-and-hold as its waveform.

**Wavetable.** Any audio sample becomes a wavetable oscillator. A scan
parameter sweeps through the waveform cycles, and the cycle length is
configurable.

**Samples.** Folder-based sample banks loaded from disk. Slicing with start and
end points, playback speed control, pitch-independent time stretching, and
choke groups for mutual voice silencing. Supports WAV, FLAC, OGG, and MP3.

**Soundfont.** SF2 file playback with 128 General MIDI presets, selectable by
name or program number. Place an SF2 file in a sample directory and the engine
picks it up.

**Drums.** Seven synthesized percussion instruments — kick, snare, hi-hat, tom,
rimshot, cowbell, cymbal — each with tailored parameters for pitch, tone, and
noise content. Not sample-based: the engine generates them from oscillators and
noise.

**Live input.** Routes microphone or line-in audio through the full effects
chain, turning the engine into a real-time processor for external sound.

Between these sources the engine covers a continuous range from pure sine tones
to dense spectral textures, from clean sample playback to mangled live audio
processing — without any external gear.

## II. Sound shaping

**Filters.** Three filter families. State variable filters provide lowpass,
highpass, and bandpass modes with resonance. Moog-style ladder filters add a
four-pole response with nonlinear saturation and self-oscillation. Biquad
filters cover eight modes: lowpass, highpass, bandpass, notch, allpass,
peaking, low shelf, and high shelf.

**Envelope.** A six-stage DAHDSR envelope controls amplitude: delay, attack,
hold, decay, sustain, release. Attack and decay/release stages have independent
curve shapes.

**Modulation.** Vibrato applies a pitch LFO. Amplitude modulation and ring
modulation shape the signal level and spectrum respectively. LFO waveforms
include sine, triangle, sawtooth, square, and sample-and-hold.

**Effects.** The effects chain splits into two stages. Per-voice insert effects
process each voice independently. Modulation inserts include phaser, flanger,
three-voice stereo chorus, and allpass resonance. Distortion covers soft-clip
saturation, Serge-style wavefolding, wavewrapping, bit crushing, and sample
rate reduction. A three-band parametric EQ and a tilt EQ handle tone shaping.
Stereo placement uses the Haas effect and a width control ranging from mono to
exaggerated stereo.

After the inserts, voices send to shared orbit buses for reverb, delay, comb
filtering, and dynamics. Two reverb algorithms are available: a Dattorro plate
and a modern variant with built-in chorus modulation, shelf EQ, and
pre-filtering. Delay comes in four flavors: standard, ping-pong, tape with
saturation, and multitap — plus a separate feedback delay with LFO-modulated
time. A resonant comb filter and a sidechain compressor for ducking between
orbits round out the bus effects.

## III. Polyphony and routing

The engine is polyphonic with up to 32 simultaneous voices. The voice limit is
configurable. Eight independent effect bus orbits let you group voices and
apply different send-effect settings to each group — one orbit for drums with
a short reverb, another for pads with a long tail, for instance. The engine
runs inside the server. In a [multiplayer](multiplayer) session, all connected
musicians share the same engine instance. Any client can trigger sound. When
idle, the engine consumes no significant resources.

## IV. Recording and monitoring

**Recording.** The engine records its own master output — or a single orbit —
into named samples, up to 60 seconds each. An overdub mode layers new audio on
top of existing recordings. Captured samples are immediately available for
playback through the full engine, with all effects and time stretching. See the
language tabs for the recording syntax.

**Monitoring.** The Audio panel selects the output device and configures voice
count and sample paths. Three visualization panels track the output in real
time: the scope shows the waveform, the spectrum shows frequency content, and
the VU meter shows signal level. Scope and spectrum are detachable as separate
windows.
