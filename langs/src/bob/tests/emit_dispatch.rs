use super::compile_and_run;
use sova_core::vm::event::ConcreteEvent;

#[test]
fn sound_takes_precedence_over_note() {
    // When both sound: and note: are present, should emit Dirt, not MidiNote
    let result = compile_and_run(">> [sound: \"saw\" note: 60]");
    assert_eq!(result.events.len(), 1);
    assert!(
        matches!(result.events[0].0, ConcreteEvent::Dirt { .. }),
        "Expected Dirt event when sound: is present, got {:?}",
        result.events[0].0
    );
}

#[test]
fn sound_with_note_symbol() {
    // Same test but with note symbol syntax
    let result = compile_and_run(">> [sound: \"square\" note: :c4]");
    assert_eq!(result.events.len(), 1);
    assert!(
        matches!(result.events[0].0, ConcreteEvent::Dirt { .. }),
        "Expected Dirt event when sound: is present, got {:?}",
        result.events[0].0
    );
}

#[test]
fn bare_note_emits_midi() {
    // Without sound:, note: should emit MIDI
    let result = compile_and_run(">> [note: 60 vel: 100]");
    assert_eq!(result.events.len(), 1);
    assert!(
        matches!(result.events[0].0, ConcreteEvent::MidiNote(..)),
        "Expected MidiNote event when only note: is present, got {:?}",
        result.events[0].0
    );
}

#[test]
fn s_shorthand_takes_precedence_over_note() {
    // s: is shorthand for sound:
    let result = compile_and_run(">> [s: \"bd\" note: 48]");
    assert_eq!(result.events.len(), 1);
    assert!(
        matches!(result.events[0].0, ConcreteEvent::Dirt { .. }),
        "Expected Dirt event when s: is present, got {:?}",
        result.events[0].0
    );
}
