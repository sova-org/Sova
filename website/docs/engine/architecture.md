### Voice

A **Voice** is a single sound instance in the engine, representing one _note_/_sound_ being played. Each voice has a unique ID for tracking and control, and contains one source (oscillator, sampler, etc.) along with a chain of local effects (filters, distortion, etc.). An ADSR envelope controls the voice's amplitude over time, while up to 16 modulation slots enable parameter automation. Each voice is assigned to a track for routing and mixing. This engine works with a fixed-size pool of voices (configurable between 64 and 512) that are recycled when finished.

```
Inactive
  → Triggered
  → Active (processing)
  → Envelope finished
Inactive (recycled)
```

### Track

A **Track** is an audio channel that mixes multiple voices and applies effects. Each track can host unlimited voices (up to the engine's maximum), and maintains its own buffer for mixing voice outputs. Tracks apply **global effects** such as reverb, delay, and echo using a **send architecture** that provides intuitive dry/wet control. The final output from each track is routed to the master output for final processing.

```
┌──────────────────────┐
│ PER-VOICE PROCESSING │
└──────────────────────┘
    ┌─────────────┐
    │   SOURCE    │ 
    │             │
    │ • sine      │
    │ • saw       │
    │ • etc.      │
    └──────┬──────┘
    ┌──────┴──────┐
    │ DC BLOCKER  │ 
    └──────┬──────┘
    ┌──────┴──────┐
    │LOCAL EFFECTS│ 
    │ • lowpass   │
    │ • dist      │
    │ • phaser    │
    │ • etc...    │
    └──────┬──────┘
    ┌──────┴──────┐
    │ SOFT LIMIT  │
    └──────┬──────┘
    ┌──────┴──────┐
    │  ENVELOPE   │ 
    └──────┬──────┘
    ┌──────┴──────┐
    │ AMP AND PAN │
    └──────┬──────┘
 ┌──────────────────┐
 │ TRACK PROCESSING │
 └──────────────────┘
    ┌─────────────┐
    │ TRACK MIXER │
    │             │
    │ Voice 0  ───┤
    │ Voice 1  ───┤
    │ Voice 2  ───┼──► Track buffer
    │ ...      ───┤
    │ Voice N  ───┤
    └──────┬──────┘
    ┌──────┴───────────────────┐
    │    GLOBAL EFFECTS (SEND) │
    │                          │
    │  For each active effect: │
    │  1. Copy to send buffer  │
    │  2. Process effect (wet) │
    │     • echo               │
    │     • reverb             │
    │  3. Mix back to track    │ 
    └──────────┬───────────────┘
        ┌──────┴──────┐
        │   MASTER    │
        │     SUM     │
        └──────┬──────┘
        ┌──────┴──────┐
        │  SOFT CLIP  │
        └──────┬──────┘
               ↓
        Audio Interface
```