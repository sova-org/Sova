# Devices

Devices are the outputs that carry your music to the outside world. Every event
your code produces is routed to a device — a MIDI port, an OSC endpoint, the
built-in audio engine, or the log console.

## The device map

Sova uses a **device map** with 16 numbered slots:

- **Slot 0** — the Log device. Always present, not user-assignable. Events sent
  here appear in the Log panel. Useful for debugging.
- **Slots 1–16** — user-assignable. You place your MIDI ports, OSC endpoints,
  and audio engine connections in these slots.

When your code emits an event, it targets a **slot number**. The device sitting
in that slot receives the event. If the slot is empty, the event is silently
dropped.

The **default device slot is 1** — if your code doesn't specify a device, events
go to slot 1.

## Device types

| Type | Description |
|------|-------------|
| MIDI output | A hardware or software MIDI port on your system |
| Virtual MIDI output | A virtual MIDI port created by Sova (appears in other apps) |
| OSC output | A UDP endpoint (IP address + port) for Open Sound Control |
| Audio engine | The built-in Doux synthesizer (see Audio Engine article) |
| Log | The debug console (slot 0, always present) |

MIDI input devices can also be connected for receiving external MIDI, but they
don't occupy device slots — they feed into the system differently.

## The Devices panel

Open the Devices panel to manage your connections:

- **Connect MIDI**: lists available MIDI ports on your system. Click to connect.
- **Create virtual MIDI**: creates a new virtual MIDI port that other
  applications can see and receive from.
- **Create OSC output**: specify a name, target IP address, and port number.
- **Assign to slot**: drag or assign connected devices to slots 1–16.
- **Unassign**: remove a device from its slot without disconnecting it.

## Routing events from code

In your scripts, you control which device receives events by setting the device
variable. The exact syntax depends on the language — see each language's
reference for details. The general idea:

- Set the device to a slot number (1–16) before emitting events.
- Events inherit the current device setting.
- You can change the device mid-script to route different events to different
  outputs.

For example, slot 1 might be your synth, slot 2 your drum machine, and slot 3
an OSC connection to a visual program. A single script can address all three.

## MIDI channels

MIDI channels are **1-based** in Sova's user-facing interface and code (1–16),
matching the standard MIDI convention. The default channel is 1.

Each event can target a specific channel independently of the device slot. This
means one MIDI port (one device slot) can address all 16 MIDI channels.

## Tips

- Keep slot assignments consistent across sessions — your code refers to slot
  numbers, so shuffling devices between slots will break routing.
- Use the Log device (slot 0) while developing to see exactly what events your
  code produces before sending them to a real output.
- Virtual MIDI outputs are the easiest way to route Sova into a DAW on the same
  machine.
