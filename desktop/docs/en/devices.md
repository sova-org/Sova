# Devices

Sova can send MIDI and OSC, with more protocols to come. Notes, control changes, pitch bends and other messages can reach external synthesizers, drum machines, DAWs. Anything that listens can play with Sova. The built-in audio engine is able to produce sound without any external gear. All output flows through numbered slots: code targets a slot, and the device assigned to that slot handles the message. The Devices panel is where connections are made and slots are assigned.

## Slots

There are sixteen device slots by default, numbered from 1 to 16. Each slot holds one device. This device can be a MIDI port, an OSC endpoint or the audio engine itself. A device can only occupy one slot at a time; assigning it elsewhere clears the previous assignment. Events that do not specify a device go to slot 1. Each language has its own syntax for targeting a slot; see [Events](events). There is a special slot, invisible to you: `Slot 0` is the `Log` device used for debugging. It is always present, cannot be reassigned.

## MIDI output

System MIDI ports are discovered automatically and listed in the Devices panel. Click `Connect` to open a port and assign it to a slot. MIDI channels range from 1 to 16; the default is channel 1. One port addresses all 16 channels, so a single connection can drive multiple instruments. Sova tracks active notes per channel. A duplicate `Note On` for a note already sounding on the same channel is silently dropped. `Note Off` is only sent for notes that were tracked as active: no stuck notes, no redundant messages.

## Virtual MIDI ports

Virtual MIDI ports are software ports created by Sova that can appear as inputs in other applications on the same machine. This feature is macOS and Linux only. Windows does not support them (yet), but this feature is being worked on by Microsoft. You can create a virtual midi port in the Devices panel by entering its name first. Both an input and an output port are created under that name. Other applications see the output as an available MIDI source; the input receives data back.

## OSC output

You can create an OSC endpoint by specifying a name, IP address, and port number. Messages are sent as UDP packets to that address, in a _fire and forget_ manner. This is how Sova talks to SuperCollider, Max, Pure Data, and similar environments. See [Events](events) for the message format.

## Audio engine

Doux, the internal audio engine, occupies a slot like any other device. It is a bit special though. When a Sova session starts, the audio engine will already be assigned automatically to take slot 1. If slot 1 was already occupied, the previous device moves to slot 2. See [Audio Engine](audio-engine) for synthesis details.

## MIDI input

MIDI input devices do not occupy a slot. They feed data into the system: incoming CC values are stored per channel and controller number. Scripts can read the last received value for any CC on any channel. See the language tabs for the syntax used by all languages to receive and play with CC events.

## Latency

There are two mechanisms currently used to compensate for timing delays:

**Per-device latency** is a user-adjustable offset, 20 ms by default. It shifts event timestamps forward to compensate for hardware response time. Adjust it per device in the Devices panel.

**Protocol lookahead** is a fixed offset applied by the world thread for dispatch precision. MIDI messages are sent 2 ms early. OSC and audio engine messages are sent 20 ms early. These values are not user-adjustable.

The two combine: per-device latency accounts for your hardware, protocol lookahead ensures the world thread hands off messages early enough for the transport layer to deliver them accurately.

## Multiplayer

Device slot assignments are shared state. The server owns the slot map. When one musician assigns a device to a slot, all clients see the change. But MIDI and OSC connections are local to each machine. Each musician configures their own physical outputs. See [Multiplayer](multiplayer). It means that you can have one synth assigned for a slot and somebody could have a different synth assigned.

## Restoration

When a session is restored, device assignments are preserved. Physical MIDI ports are re-discovered automatically. Virtual MIDI ports and OSC endpoints are re-created. If a device from the snapshot is unavailable, it appears as disconnected but keeps its slot assignment.
