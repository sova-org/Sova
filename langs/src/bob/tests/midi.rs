use super::compile_and_run;
use sova_core::vm::event::ConcreteEvent;

// ============================================================================
// Helper macros for event type assertions
// ============================================================================

macro_rules! assert_midi_note {
    ($result:expr) => {
        assert_eq!($result.events.len(), 1);
        assert!(
            matches!($result.events[0].0, ConcreteEvent::MidiNote(..)),
            "Expected MidiNote, got {:?}",
            $result.events[0].0
        );
    };
    ($result:expr, $note:expr) => {
        assert_eq!($result.events.len(), 1);
        match &$result.events[0].0 {
            ConcreteEvent::MidiNote(note, _, _, _, _) => {
                assert_eq!(*note, $note, "Expected note {}, got {}", $note, note);
            }
            other => panic!("Expected MidiNote, got {:?}", other),
        }
    };
    ($result:expr, $note:expr, $vel:expr) => {
        assert_eq!($result.events.len(), 1);
        match &$result.events[0].0 {
            ConcreteEvent::MidiNote(note, vel, _, _, _) => {
                assert_eq!(*note, $note, "Expected note {}, got {}", $note, note);
                assert_eq!(*vel, $vel, "Expected vel {}, got {}", $vel, vel);
            }
            other => panic!("Expected MidiNote, got {:?}", other),
        }
    };
    ($result:expr, $note:expr, $vel:expr, $chan:expr) => {
        assert_eq!($result.events.len(), 1);
        match &$result.events[0].0 {
            ConcreteEvent::MidiNote(note, vel, chan, _, _) => {
                assert_eq!(*note, $note, "Expected note {}, got {}", $note, note);
                assert_eq!(*vel, $vel, "Expected vel {}, got {}", $vel, vel);
                assert_eq!(*chan, $chan, "Expected chan {}, got {}", $chan, chan);
            }
            other => panic!("Expected MidiNote, got {:?}", other),
        }
    };
}

macro_rules! assert_midi_cc {
    ($result:expr) => {
        assert_eq!($result.events.len(), 1);
        assert!(
            matches!($result.events[0].0, ConcreteEvent::MidiControl(..)),
            "Expected MidiControl, got {:?}",
            $result.events[0].0
        );
    };
    ($result:expr, $cc:expr, $val:expr, $chan:expr) => {
        assert_eq!($result.events.len(), 1);
        match &$result.events[0].0 {
            ConcreteEvent::MidiControl(cc, val, chan, _) => {
                assert_eq!(*cc, $cc, "Expected cc {}, got {}", $cc, cc);
                assert_eq!(*val, $val, "Expected val {}, got {}", $val, val);
                assert_eq!(*chan, $chan, "Expected chan {}, got {}", $chan, chan);
            }
            other => panic!("Expected MidiControl, got {:?}", other),
        }
    };
}

macro_rules! assert_midi_program {
    ($result:expr) => {
        assert_eq!($result.events.len(), 1);
        assert!(
            matches!($result.events[0].0, ConcreteEvent::MidiProgram(..)),
            "Expected MidiProgram, got {:?}",
            $result.events[0].0
        );
    };
    ($result:expr, $pc:expr, $chan:expr) => {
        assert_eq!($result.events.len(), 1);
        match &$result.events[0].0 {
            ConcreteEvent::MidiProgram(pc, chan, _) => {
                assert_eq!(*pc, $pc, "Expected pc {}, got {}", $pc, pc);
                assert_eq!(*chan, $chan, "Expected chan {}, got {}", $chan, chan);
            }
            other => panic!("Expected MidiProgram, got {:?}", other),
        }
    };
}

macro_rules! assert_midi_aftertouch {
    ($result:expr) => {
        assert_eq!($result.events.len(), 1);
        assert!(
            matches!($result.events[0].0, ConcreteEvent::MidiAftertouch(..)),
            "Expected MidiAftertouch, got {:?}",
            $result.events[0].0
        );
    };
    ($result:expr, $note:expr, $at:expr, $chan:expr) => {
        assert_eq!($result.events.len(), 1);
        match &$result.events[0].0 {
            ConcreteEvent::MidiAftertouch(note, at, chan, _) => {
                assert_eq!(*note, $note, "Expected note {}, got {}", $note, note);
                assert_eq!(*at, $at, "Expected at {}, got {}", $at, at);
                assert_eq!(*chan, $chan, "Expected chan {}, got {}", $chan, chan);
            }
            other => panic!("Expected MidiAftertouch, got {:?}", other),
        }
    };
}

macro_rules! assert_midi_channel_pressure {
    ($result:expr) => {
        assert_eq!($result.events.len(), 1);
        assert!(
            matches!($result.events[0].0, ConcreteEvent::MidiChannelPressure(..)),
            "Expected MidiChannelPressure, got {:?}",
            $result.events[0].0
        );
    };
    ($result:expr, $pressure:expr, $chan:expr) => {
        assert_eq!($result.events.len(), 1);
        match &$result.events[0].0 {
            ConcreteEvent::MidiChannelPressure(pressure, chan, _) => {
                assert_eq!(
                    *pressure, $pressure,
                    "Expected pressure {}, got {}",
                    $pressure, pressure
                );
                assert_eq!(*chan, $chan, "Expected chan {}, got {}", $chan, chan);
            }
            other => panic!("Expected MidiChannelPressure, got {:?}", other),
        }
    };
}

macro_rules! assert_midi_start {
    ($result:expr) => {
        assert_eq!($result.events.len(), 1);
        assert!(
            matches!($result.events[0].0, ConcreteEvent::MidiStart(_)),
            "Expected MidiStart, got {:?}",
            $result.events[0].0
        );
    };
}

macro_rules! assert_midi_stop {
    ($result:expr) => {
        assert_eq!($result.events.len(), 1);
        assert!(
            matches!($result.events[0].0, ConcreteEvent::MidiStop(_)),
            "Expected MidiStop, got {:?}",
            $result.events[0].0
        );
    };
}

macro_rules! assert_midi_reset {
    ($result:expr) => {
        assert_eq!($result.events.len(), 1);
        assert!(
            matches!($result.events[0].0, ConcreteEvent::MidiReset(_)),
            "Expected MidiReset, got {:?}",
            $result.events[0].0
        );
    };
}

macro_rules! assert_midi_continue {
    ($result:expr) => {
        assert_eq!($result.events.len(), 1);
        assert!(
            matches!($result.events[0].0, ConcreteEvent::MidiContinue(_)),
            "Expected MidiContinue, got {:?}",
            $result.events[0].0
        );
    };
}

macro_rules! assert_midi_clock {
    ($result:expr) => {
        assert_eq!($result.events.len(), 1);
        assert!(
            matches!($result.events[0].0, ConcreteEvent::MidiClock(_)),
            "Expected MidiClock, got {:?}",
            $result.events[0].0
        );
    };
}

macro_rules! assert_dirt {
    ($result:expr) => {
        assert_eq!($result.events.len(), 1);
        assert!(
            matches!($result.events[0].0, ConcreteEvent::Dirt { .. }),
            "Expected Dirt, got {:?}",
            $result.events[0].0
        );
    };
}

// ============================================================================
// MIDI Note Tests
// ============================================================================

#[test]
fn midi_note_basic() {
    let result = compile_and_run(">> [note: 60]");
    assert_midi_note!(result, 60);
}

#[test]
fn midi_note_with_velocity() {
    let result = compile_and_run(">> [note: 60 vel: 80]");
    assert_midi_note!(result, 60, 80);
}

#[test]
fn midi_note_all_params() {
    let result = compile_and_run(">> [note: 60 vel: 80 chan: 1 dur: 0.25 dev: 0]");
    assert_midi_note!(result, 60, 80, 1);
}

#[test]
fn midi_note_from_symbol() {
    // :c3 = MIDI note 60 (middle C)
    let result = compile_and_run(">> [note: :c3 vel: 100]");
    assert_midi_note!(result, 60, 100);
}

#[test]
fn midi_note_defaults() {
    // note defaults: vel=100, chan=0
    let result = compile_and_run(">> [note: 72]");
    assert_midi_note!(result, 72, 100, 0);
}

#[test]
fn midi_vel_only_triggers_note() {
    // vel alone should trigger MidiNote with default note=60
    let result = compile_and_run(">> [vel: 127]");
    assert_midi_note!(result, 60, 127, 0);
}

// ============================================================================
// MIDI CC Tests
// ============================================================================

#[test]
fn midi_cc() {
    let result = compile_and_run(">> [cc: 1 val: 64 chan: 0]");
    assert_midi_cc!(result, 1, 64, 0);
}

#[test]
fn midi_cc_defaults() {
    // CC with only cc number - val and chan default to 0
    let result = compile_and_run(">> [cc: 7]");
    assert_midi_cc!(result, 7, 0, 0);
}

#[test]
fn midi_cc_modwheel() {
    let result = compile_and_run(">> [cc: 1 val: 127]");
    assert_midi_cc!(result, 1, 127, 0);
}

// ============================================================================
// MIDI Program Change Tests
// ============================================================================

#[test]
fn midi_program_change() {
    let result = compile_and_run(">> [pc: 5]");
    assert_midi_program!(result, 5, 0);
}

#[test]
fn midi_program_change_with_channel() {
    let result = compile_and_run(">> [pc: 10 chan: 2]");
    assert_midi_program!(result, 10, 2);
}

// ============================================================================
// MIDI Aftertouch Tests
// ============================================================================

#[test]
fn midi_aftertouch() {
    // Polyphonic aftertouch: requires both at and note
    let result = compile_and_run(">> [at: 100 note: 60]");
    assert_midi_aftertouch!(result, 60, 100, 0);
}

#[test]
fn midi_aftertouch_with_channel() {
    let result = compile_and_run(">> [at: 80 note: 72 chan: 5]");
    assert_midi_aftertouch!(result, 72, 80, 5);
}

#[test]
fn midi_aftertouch_without_note_fallthrough() {
    // at without note falls through to Dirt generic (no MIDI aftertouch)
    let result = compile_and_run(">> [at: 100]");
    assert_dirt!(result);
}

// ============================================================================
// MIDI Channel Pressure Tests
// ============================================================================

#[test]
fn midi_channel_pressure() {
    let result = compile_and_run(">> [pressure: 80]");
    assert_midi_channel_pressure!(result, 80, 0);
}

#[test]
fn midi_channel_pressure_with_channel() {
    let result = compile_and_run(">> [pressure: 64 chan: 3]");
    assert_midi_channel_pressure!(result, 64, 3);
}

// ============================================================================
// MIDI Transport Tests
// ============================================================================

#[test]
fn midi_start() {
    let result = compile_and_run(">> [start: 1]");
    assert_midi_start!(result);
}

#[test]
fn midi_start_zero_no_event() {
    // MIDI start with 0 is falsy, falls through to Dirt generic
    let result = compile_and_run(">> [start: 0]");
    assert_dirt!(result);
}

#[test]
fn midi_stop() {
    let result = compile_and_run(">> [stop: 1]");
    assert_midi_stop!(result);
}

#[test]
fn midi_reset() {
    let result = compile_and_run(">> [reset: 1]");
    assert_midi_reset!(result);
}

#[test]
fn midi_continue() {
    let result = compile_and_run(">> [continue: 1]");
    assert_midi_continue!(result);
}

#[test]
fn midi_clock() {
    let result = compile_and_run(">> [clock: 1]");
    assert_midi_clock!(result);
}

#[test]
fn midi_conditional_transport() {
    // Transport with variable condition
    let result = compile_and_run("SET G.X 1; >> [start: G.X]");
    assert_midi_start!(result);
}

// ============================================================================
// MIDI Priority Tests
// ============================================================================

#[test]
fn midi_priority_cc_over_note() {
    // cc takes priority over note - should emit CC, not note
    let result = compile_and_run(">> [cc: 1 note: 60]");
    assert_midi_cc!(result);
}

#[test]
fn midi_priority_transport_over_cc() {
    // Transport takes priority over CC
    let result = compile_and_run(">> [start: 1 cc: 1]");
    assert_midi_start!(result);
}

#[test]
fn midi_priority_transport_over_note() {
    // Transport takes priority over note
    let result = compile_and_run(">> [stop: 1 note: 60]");
    assert_midi_stop!(result);
}

#[test]
fn midi_priority_aftertouch_over_note() {
    // at+note triggers aftertouch, not note
    let result = compile_and_run(">> [at: 100 note: 60]");
    assert_midi_aftertouch!(result);
}

#[test]
fn midi_priority_pc_over_note() {
    // pc takes priority over note
    let result = compile_and_run(">> [pc: 5 note: 60]");
    assert_midi_program!(result);
}

#[test]
fn midi_priority_pressure_over_note() {
    // pressure takes priority over note
    let result = compile_and_run(">> [pressure: 80 note: 60]");
    assert_midi_channel_pressure!(result);
}
