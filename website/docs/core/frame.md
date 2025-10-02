# Frame

A Frame is a duration. This duration is always associated with a _script_, a computer program that will run for this duration. It can be very short, it can be very long. A frame is defined by the following properties:

| Property | Type | Description |
|----------|------|-------------|
| `duration` | `number` | The length of time the frame runs |
| `enabled` | `boolean` | Whether the frame is active |
| `name` | `string \| null` | Optional name for the frame |
| `script` | `Script` | The program that executes during this frame |
| `repetitions` | `number` | Number of times the frame repeats |

Just like on a step sequencer, a frame can be enabled or disabled. If disabled, the frame is skipped during playback. Time will still pass, but the script will not run. A frame can also be set to repeat a certain number of times. 