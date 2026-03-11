# Events

Scripts produce events. An event is a message sent to a device: a MIDI note, a
control change, an OSC packet, an audio engine command. Each event carries
parameters that determine what sounds, when, and where.

## MIDI notes

A note event sends Note On, then Note Off after the specified duration. The
engine handles Note Off automatically.

Parameters: pitch (0–127), velocity (0–127), duration (beats), channel (1–16),
device (1–16). Defaults: velocity 100, channel 1, device 1. Duration defaults
vary by language — see the language tabs.

## Control Change

CC messages set continuous controller values on external instruments or DAWs.
Parameters: CC number (0–127), value (0–127), channel, device.

## Pitch Bend

At the VM level, pitch bend is a 14-bit integer: 0–16383, center at 8192 (no
bend). Some languages expose this as a float from -1.0 to 1.0 for convenience.
Parameters: bend value, channel, device.

## Program Change

Selects a patch or preset on the target device. Parameters: program number
(0–127), channel, device.

## Aftertouch

Polyphonic aftertouch applies pressure per note. Parameters: note (0–127),
pressure (0–127), channel, device.

## Channel Pressure

Channel pressure applies a single pressure value to the entire channel.
Parameters: pressure (0–127), channel, device.

## System Exclusive

SysEx sends raw byte data to a device. Used for vendor-specific messages,
firmware updates, or bulk dumps. Parameters: byte sequence, device.

## MIDI Transport

Transport messages synchronize external hardware sequencers and drum machines.
Types: Start, Stop, Continue, Clock, Reset. Each targets a specific device.
These carry no channel — they apply to the device as a whole.

## OSC

OSC events send UDP packets to SuperCollider, Max/MSP, Pure Data, or any
OSC-capable application. An OSC event carries an address path and a set of
arguments. Route it to an OSC device slot. See **Devices** for slot
configuration.

## Audio engine events

Audio events send key-value parameter maps to the Doux audio engine. Cagire has
the deepest integration with Doux — see the Cagire language tab for details.

## Device and channel routing

Every event carries a device slot and a MIDI channel. The device slot selects
the output (1–16). The channel selects the MIDI channel within that device.
Slot 0 is the log console — use it to inspect events without sending them to
hardware. Device and channel can change mid-script, routing different events to
different destinations from the same frame. See **Devices** for configuration.

## Chords and sequences

Without explicit timing offsets, all events in a script fire simultaneously.
This produces chords or layered sounds. To space events in time, use your
language's timing mechanism. See **Timing** for details.

## Reading MIDI input

Incoming CC values from hardware controllers can be read inside scripts. This
allows physical knobs and faders to drive parameters in real time. Connect a
MIDI input device in the Devices panel and use the language's input functions to
read values.

See the language tabs for the syntax of each event type.
