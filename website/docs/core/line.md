# Line

A Line is a sequence of frames that play in order. Each frame has a duration in beats. Lines support speed adjustment, looping, and frame range selection. When a line reaches the end of its frames, it loops back to the beginning, like on a regular sequencer.


| Property | Type | Description |
|----------|------|-------------|
| `frames` | `Frame[]` | Sequence of frames in the line |
| `speed_factor` | `number` | Playback speed multiplier (1.0 = nominal speed) |
| `vars` | `VariableStore` | Variables specific to this line |
| `start_frame` | `number \| null` | First frame to play (inclusive) |
| `end_frame` | `number \| null` | Last frame to play (inclusive) |
| `custom_length` | `number \| null` | Override total loop duration in beats |

- Speed factor affects playback: values less than 1.0 slow it down, greater than 1.0 speed it up.
- Setting `start_frame` and `end_frame` restricts which frames play during each loop. 