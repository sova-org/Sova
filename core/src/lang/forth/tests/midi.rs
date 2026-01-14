use super::run_forth;
use crate::vm::event::ConcreteEvent;

#[test]
fn test_note() {
    let result = run_forth("60 note");
    assert_eq!(result.events.len(), 1);
    match &result.events[0].0 {
        ConcreteEvent::MidiNote(note, vel, ch, _dur, dev) => {
            assert_eq!(*note, 60);
            assert_eq!(*vel, 90);  // default velocity
            assert_eq!(*ch, 1);    // default channel
            assert_eq!(*dev, 1);   // default device
        }
        _ => panic!("Expected MidiNote event"),
    }
}

#[test]
fn test_note_with_velocity() {
    let result = run_forth("127 vel! 60 note");
    assert_eq!(result.events.len(), 1);
    match &result.events[0].0 {
        ConcreteEvent::MidiNote(note, vel, _ch, _dur, _dev) => {
            assert_eq!(*note, 60);
            assert_eq!(*vel, 127);
        }
        _ => panic!("Expected MidiNote event"),
    }
}

#[test]
fn test_note_with_channel() {
    let result = run_forth("10 ch! 36 note");
    assert_eq!(result.events.len(), 1);
    match &result.events[0].0 {
        ConcreteEvent::MidiNote(note, _vel, ch, _dur, _dev) => {
            assert_eq!(*note, 36);
            assert_eq!(*ch, 10);
        }
        _ => panic!("Expected MidiNote event"),
    }
}

#[test]
fn test_note_with_device() {
    let result = run_forth("2 dev! 60 note");
    assert_eq!(result.events.len(), 1);
    match &result.events[0].0 {
        ConcreteEvent::MidiNote(_note, _vel, _ch, _dur, dev) => {
            assert_eq!(*dev, 2);
        }
        _ => panic!("Expected MidiNote event"),
    }
}

#[test]
fn test_cc() {
    let result = run_forth("1 64 cc");  // CC1 = 64 (controller value cc)
    assert_eq!(result.events.len(), 1);
    match &result.events[0].0 {
        ConcreteEvent::MidiControl(ctrl, val, ch, dev) => {
            assert_eq!(*ctrl, 1);
            assert_eq!(*val, 64);
            assert_eq!(*ch, 1);
            assert_eq!(*dev, 1);
        }
        _ => panic!("Expected MidiControl event"),
    }
}

#[test]
fn test_pc() {
    let result = run_forth("5 pc");  // Program change to 5
    assert_eq!(result.events.len(), 1);
    match &result.events[0].0 {
        ConcreteEvent::MidiProgram(prog, ch, dev) => {
            assert_eq!(*prog, 5);
            assert_eq!(*ch, 1);
            assert_eq!(*dev, 1);
        }
        _ => panic!("Expected MidiProgram event"),
    }
}

#[test]
fn test_multiple_notes() {
    let result = run_forth("60 note 64 note 67 note");
    assert_eq!(result.events.len(), 3);
}

#[test]
fn test_wait() {
    let result = run_forth("60 note 1 wait 64 note");
    assert_eq!(result.events.len(), 2);
    // Second event should have accumulated time
    assert!(result.events[1].1 > result.events[0].1);
}

#[test]
fn test_drum_pattern() {
    // Kick on channel 10
    let result = run_forth("10 ch! 36 note 38 note 36 note 38 note");
    assert_eq!(result.events.len(), 4);
    for event in &result.events {
        match &event.0 {
            ConcreteEvent::MidiNote(_note, _vel, ch, _dur, _dev) => {
                assert_eq!(*ch, 10);
            }
            _ => panic!("Expected MidiNote event"),
        }
    }
}

#[test]
fn test_context_getters() {
    let result = run_forth("5 ch! ch@ note");
    assert_eq!(result.events.len(), 1);
    match &result.events[0].0 {
        ConcreteEvent::MidiNote(note, _vel, ch, _dur, _dev) => {
            assert_eq!(*note, 5);  // ch@ pushed 5, which became the note
            assert_eq!(*ch, 5);
        }
        _ => panic!("Expected MidiNote event"),
    }
}

#[test]
fn test_word_with_note() {
    let result = run_forth(": kick 36 note ; : snare 38 note ; kick snare kick");
    assert_eq!(result.events.len(), 3);

    let notes: Vec<u64> = result.events.iter().map(|(e, _)| {
        match e {
            ConcreteEvent::MidiNote(n, _, _, _, _) => *n,
            _ => panic!("Expected MidiNote"),
        }
    }).collect();

    assert_eq!(notes, vec![36, 38, 36]);
}

#[test]
fn test_loop_with_notes() {
    // Play 4 notes using DO LOOP
    let result = run_forth("4 0 do 60 i + note loop");
    assert_eq!(result.events.len(), 4);

    let notes: Vec<u64> = result.events.iter().map(|(e, _)| {
        match e {
            ConcreteEvent::MidiNote(n, _, _, _, _) => *n,
            _ => panic!("Expected MidiNote"),
        }
    }).collect();

    assert_eq!(notes, vec![60, 61, 62, 63]);
}
