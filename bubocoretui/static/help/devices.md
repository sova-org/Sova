The `Devices` view shows available MIDI and OSC devices and allows managing connections and assignments.

## Keybindings

### Navigation

*   `↑` / `↓` : Navigate through the device list.
*   `M` : Switch to MIDI device list tab.
*   `O` : Switch to OSC device list tab (Not fully implemented yet).

### General Actions (Normal Mode)

*   `Enter` : Connect / Disconnect the selected device.
*   `Ctrl` + `N` : Create a new MIDI virtual output port (opens naming prompt).
*   `s` : Assign a Slot ID to the selected device (opens assignment prompt).

### Virtual Port Naming Mode

*   `Enter` : Confirm the entered name and create the port.
*   `Esc` : Cancel virtual port creation.
*   `↑` / `↓` : Browse through recently used port names.
*   *Other keys* : Edit the port name.

### Slot Assignment Mode

*   `Enter` : Confirm the entered Slot ID (0 to unassign, 1-16 to assign).
*   `Esc` : Cancel slot assignment.
*   `0-9` : Enter the Slot ID number.
*   `Backspace` : Delete the last digit entered.
