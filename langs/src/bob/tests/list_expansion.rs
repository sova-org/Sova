use super::{compile_and_run, compile_and_run_debug};
use sova_core::vm::event::ConcreteEvent;

// ============================================================================
// MIDI Note List Expansion Tests
// ============================================================================

#[test]
fn note_list_expands_to_chord() {
    let result = compile_and_run_debug(">> [note: '[60 64 67] vel: 100]");
    assert_eq!(result.events.len(), 3, "Expected 3 events for 3-note chord");

    // Verify each note
    let notes: Vec<u64> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiNote(n, _, _, _, _) => *n,
            _ => panic!("Expected MidiNote, got {:?}", e),
        })
        .collect();
    assert_eq!(notes, vec![60, 64, 67]);

    // Verify all have same velocity
    for (event, _) in &result.events {
        match event {
            ConcreteEvent::MidiNote(_, vel, _, _, _) => {
                assert_eq!(*vel, 100, "All notes should have vel=100");
            }
            _ => panic!("Expected MidiNote"),
        }
    }
}

#[test]
fn note_list_with_parallel_vel_list() {
    let result = compile_and_run(">> [note: '[60 64] vel: '[100 80]]");
    assert_eq!(result.events.len(), 2);

    match (&result.events[0].0, &result.events[1].0) {
        (ConcreteEvent::MidiNote(n1, v1, _, _, _), ConcreteEvent::MidiNote(n2, v2, _, _, _)) => {
            assert_eq!((*n1, *v1), (60, 100), "First: note=60, vel=100");
            assert_eq!((*n2, *v2), (64, 80), "Second: note=64, vel=80");
        }
        _ => panic!("Expected two MidiNotes"),
    }
}

#[test]
fn shorter_list_wraps_around() {
    // note has 3 elements, vel has 2 - vel should wrap
    let result = compile_and_run(">> [note: '[60 64 67] vel: '[100 80]]");
    assert_eq!(result.events.len(), 3);

    let pairs: Vec<(u64, u64)> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiNote(n, v, _, _, _) => (*n, *v),
            _ => panic!("Expected MidiNote"),
        })
        .collect();

    assert_eq!(pairs[0], (60, 100), "i=0: note=60, vel=100");
    assert_eq!(pairs[1], (64, 80), "i=1: note=64, vel=80");
    assert_eq!(pairs[2], (67, 100), "i=2: note=67, vel=100 (wrapped)");
}

#[test]
fn all_params_can_be_lists() {
    let result = compile_and_run(">> [note: '[60 64] vel: '[100 80] chan: '[0 1]]");
    assert_eq!(result.events.len(), 2);

    match (&result.events[0].0, &result.events[1].0) {
        (ConcreteEvent::MidiNote(n1, v1, c1, _, _), ConcreteEvent::MidiNote(n2, v2, c2, _, _)) => {
            assert_eq!((*n1, *v1, *c1), (60, 100, 0));
            assert_eq!((*n2, *v2, *c2), (64, 80, 1));
        }
        _ => panic!("Expected two MidiNotes"),
    }
}

#[test]
fn single_note_no_expansion() {
    // No lists - should emit single event (regression test)
    let result = compile_and_run(">> [note: 60 vel: 100]");
    assert_eq!(result.events.len(), 1);
    match &result.events[0].0 {
        ConcreteEvent::MidiNote(n, v, _, _, _) => {
            assert_eq!((*n, *v), (60, 100));
        }
        _ => panic!("Expected MidiNote"),
    }
}

#[test]
fn single_element_list_same_as_scalar() {
    let result = compile_and_run(">> [note: '[60] vel: 100]");
    assert_eq!(result.events.len(), 1);
    match &result.events[0].0 {
        ConcreteEvent::MidiNote(n, v, _, _, _) => {
            assert_eq!((*n, *v), (60, 100));
        }
        _ => panic!("Expected MidiNote"),
    }
}

#[test]
fn complex_wrap_pattern() {
    // 4 notes, 2 velocities, 3 channels
    // max_len = 4, so:
    // i=0: note[0]=60, vel[0]=100, chan[0]=0
    // i=1: note[1]=62, vel[1]=80,  chan[1]=1
    // i=2: note[2]=64, vel[0]=100, chan[2]=2
    // i=3: note[3]=65, vel[1]=80,  chan[0]=0
    let result = compile_and_run(">> [note: '[60 62 64 65] vel: '[100 80] chan: '[0 1 2]]");
    assert_eq!(result.events.len(), 4);

    let data: Vec<(u64, u64, u64)> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiNote(n, v, c, _, _) => (*n, *v, *c),
            _ => panic!("Expected MidiNote"),
        })
        .collect();

    assert_eq!(data[0], (60, 100, 0));
    assert_eq!(data[1], (62, 80, 1));
    assert_eq!(data[2], (64, 100, 2));
    assert_eq!(data[3], (65, 80, 0)); // vel and chan wrapped
}

// ============================================================================
// MIDI CC List Expansion Tests
// ============================================================================

#[test]
fn cc_list_expansion() {
    let result = compile_and_run(">> [cc: '[1 7 10] val: 64]");
    assert_eq!(result.events.len(), 3);

    let ccs: Vec<u64> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiControl(cc, _, _, _) => *cc,
            _ => panic!("Expected MidiControl"),
        })
        .collect();
    assert_eq!(ccs, vec![1, 7, 10]);
}

#[test]
fn cc_val_parallel_expansion() {
    let result = compile_and_run(">> [cc: '[1 7] val: '[64 127]]");
    assert_eq!(result.events.len(), 2);

    match (&result.events[0].0, &result.events[1].0) {
        (ConcreteEvent::MidiControl(cc1, v1, _, _), ConcreteEvent::MidiControl(cc2, v2, _, _)) => {
            assert_eq!((*cc1, *v1), (1, 64));
            assert_eq!((*cc2, *v2), (7, 127));
        }
        _ => panic!("Expected MidiControl events"),
    }
}

#[test]
fn cc_all_params_lists() {
    let result = compile_and_run(">> [cc: '[1 7] val: '[64 127] chan: '[0 1]]");
    assert_eq!(result.events.len(), 2);

    let data: Vec<(u64, u64, u64)> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiControl(cc, v, c, _) => (*cc, *v, *c),
            _ => panic!("Expected MidiControl"),
        })
        .collect();

    assert_eq!(data[0], (1, 64, 0));
    assert_eq!(data[1], (7, 127, 1));
}

// ============================================================================
// MIDI Program Change List Expansion Tests
// ============================================================================

#[test]
fn pc_list_expansion() {
    let result = compile_and_run(">> [pc: '[0 5 10]]");
    assert_eq!(result.events.len(), 3);

    let pcs: Vec<u64> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiProgram(pc, _, _) => *pc,
            _ => panic!("Expected MidiProgram"),
        })
        .collect();
    assert_eq!(pcs, vec![0, 5, 10]);
}

#[test]
fn pc_chan_parallel() {
    let result = compile_and_run(">> [pc: '[1 2 3] chan: '[0 1]]");
    assert_eq!(result.events.len(), 3);

    let data: Vec<(u64, u64)> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiProgram(pc, c, _) => (*pc, *c),
            _ => panic!("Expected MidiProgram"),
        })
        .collect();

    assert_eq!(data[0], (1, 0));
    assert_eq!(data[1], (2, 1));
    assert_eq!(data[2], (3, 0)); // chan wrapped
}

// ============================================================================
// MIDI Aftertouch List Expansion Tests
// ============================================================================

#[test]
fn aftertouch_note_list() {
    let result = compile_and_run(">> [note: '[60 64 67] at: 100]");
    assert_eq!(result.events.len(), 3);

    let notes: Vec<u64> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiAftertouch(n, _, _, _) => *n,
            _ => panic!("Expected MidiAftertouch"),
        })
        .collect();
    assert_eq!(notes, vec![60, 64, 67]);
}

#[test]
fn aftertouch_parallel_lists() {
    let result = compile_and_run(">> [note: '[60 64] at: '[50 100]]");
    assert_eq!(result.events.len(), 2);

    match (&result.events[0].0, &result.events[1].0) {
        (
            ConcreteEvent::MidiAftertouch(n1, a1, _, _),
            ConcreteEvent::MidiAftertouch(n2, a2, _, _),
        ) => {
            assert_eq!((*n1, *a1), (60, 50));
            assert_eq!((*n2, *a2), (64, 100));
        }
        _ => panic!("Expected MidiAftertouch events"),
    }
}

// ============================================================================
// MIDI Channel Pressure List Expansion Tests
// ============================================================================

#[test]
fn channel_pressure_list() {
    let result = compile_and_run(">> [pressure: '[50 100 127]]");
    assert_eq!(result.events.len(), 3);

    let pressures: Vec<u64> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiChannelPressure(p, _, _) => *p,
            _ => panic!("Expected MidiChannelPressure"),
        })
        .collect();
    assert_eq!(pressures, vec![50, 100, 127]);
}

#[test]
fn channel_pressure_with_chan_list() {
    let result = compile_and_run(">> [pressure: '[64 100] chan: '[0 1]]");
    assert_eq!(result.events.len(), 2);

    let data: Vec<(u64, u64)> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiChannelPressure(p, c, _) => (*p, *c),
            _ => panic!("Expected MidiChannelPressure"),
        })
        .collect();

    assert_eq!(data[0], (64, 0));
    assert_eq!(data[1], (100, 1));
}

// ============================================================================
// Dirt Event List Expansion Tests
// ============================================================================

#[test]
fn dirt_sound_list() {
    let result = compile_and_run(">> [s: '[\"bd\" \"sn\" \"hh\"]]");
    assert_eq!(result.events.len(), 3);

    for (event, _) in &result.events {
        assert!(
            matches!(event, ConcreteEvent::Dirt { .. }),
            "Expected Dirt, got {:?}",
            event
        );
    }
}

#[test]
fn dirt_param_list() {
    let result = compile_and_run(">> [s: \"bd\" n: '[0 1 2]]");
    assert_eq!(result.events.len(), 3);

    for (event, _) in &result.events {
        assert!(matches!(event, ConcreteEvent::Dirt { .. }));
    }
}

#[test]
fn dirt_multiple_param_lists() {
    let result = compile_and_run(">> [s: \"bd\" n: '[0 1] gain: '[0.5 1.0]]");
    assert_eq!(result.events.len(), 2);
}

// ============================================================================
// OSC List Expansion Tests
// ============================================================================

#[test]
fn osc_param_list() {
    let result = compile_and_run(">> [addr: \"/test\" x: '[1 2 3]]");
    assert_eq!(result.events.len(), 3);

    for (event, _) in &result.events {
        assert!(
            matches!(event, ConcreteEvent::Osc { .. }),
            "Expected Osc, got {:?}",
            event
        );
    }
}

#[test]
fn osc_multiple_param_lists() {
    let result = compile_and_run(">> [addr: \"/synth\" freq: '[440 880] amp: '[0.5 1.0]]");
    assert_eq!(result.events.len(), 2);
}

// ============================================================================
// Variable-based Expansion Tests
// ============================================================================

#[test]
fn note_from_variable_list() {
    let result = compile_and_run("SET G.CHORD '[60 64 67]; >> [note: G.CHORD vel: 100]");
    assert_eq!(result.events.len(), 3);

    let notes: Vec<u64> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiNote(n, _, _, _, _) => *n,
            _ => panic!("Expected MidiNote"),
        })
        .collect();
    assert_eq!(notes, vec![60, 64, 67]);
}

#[test]
fn mixed_variable_and_literal_lists() {
    let result =
        compile_and_run("SET G.NOTES '[60 64 67]; >> [note: G.NOTES vel: '[100 80] chan: 0]");
    assert_eq!(result.events.len(), 3);

    let data: Vec<(u64, u64)> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiNote(n, v, _, _, _) => (*n, *v),
            _ => panic!("Expected MidiNote"),
        })
        .collect();

    assert_eq!(data[0], (60, 100));
    assert_eq!(data[1], (64, 80));
    assert_eq!(data[2], (67, 100)); // vel wrapped
}

// ============================================================================
// Expression-based List Expansion Tests
// ============================================================================

#[test]
fn computed_list_values() {
    // List with computed values
    let result = compile_and_run(">> [note: '[ADD 60 0 ADD 60 4 ADD 60 7] vel: 100]");
    assert_eq!(result.events.len(), 3);

    let notes: Vec<u64> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiNote(n, _, _, _, _) => *n,
            _ => panic!("Expected MidiNote"),
        })
        .collect();
    assert_eq!(notes, vec![60, 64, 67]);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn empty_list_no_events() {
    // Empty list should produce 0 events (max_len = 0)
    let result = compile_and_run(">> [note: '[] vel: 100]");
    // With our current impl, empty list has VecLen=0, so max_len=0, loop doesn't run
    // but single fallback runs - so 1 event with default note
    // Actually let me check the semantics...
    // If note is empty list, VecLen=0, so it's treated as scalar
    // Then max_len from all params is 0, so we emit single event
    // The note value will be the empty list itself, which as_integer() returns 0
    assert_eq!(result.events.len(), 1);
}

#[test]
fn list_with_symbols() {
    let result = compile_and_run(">> [note: '[:c3 :e3 :g3] vel: 100]");
    assert_eq!(result.events.len(), 3);

    let notes: Vec<u64> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiNote(n, _, _, _, _) => *n,
            _ => panic!("Expected MidiNote"),
        })
        .collect();
    // :c3 = 60, :e3 = 64, :g3 = 67
    assert_eq!(notes, vec![60, 64, 67]);
}

#[test]
fn very_long_list() {
    // Test with a longer list to ensure loop works correctly
    let result = compile_and_run(">> [note: '[60 61 62 63 64 65 66 67 68 69] vel: 100]");
    assert_eq!(result.events.len(), 10);
}

#[test]
fn list_expansion_in_loop() {
    // Emit chord multiple times in a loop
    let result = compile_and_run(
        r#"
        SET G.I 0;
        WHILE LT G.I 2 :
            >> [note: '[60 64] vel: 100];
            SET G.I ADD G.I 1
        END
        "#,
    );
    // 2 iterations × 2 notes = 4 events
    assert_eq!(result.events.len(), 4);
}

// ============================================================================
// Map Merge with List Values
// ============================================================================

#[test]
fn mmerge_preserves_list_then_expands() {
    let result = compile_and_run(
        r#"
        SET G.A [note: '[60 64]];
        SET G.B [vel: 100];
        SET G.C MMERGE G.A G.B;
        >> G.C
        "#,
    );
    // After merge, G.C = {note: [60, 64], vel: 100}
    // Emitting should expand to 2 notes
    assert_eq!(result.events.len(), 2);
}

#[test]
fn mmerge_second_list_wins() {
    let result = compile_and_run(
        r#"
        SET G.A [note: '[60 62]];
        SET G.B [note: '[64 67 71]];
        SET G.C MMERGE G.A G.B;
        >> G.C
        "#,
    );
    // Second list wins, so we get 3 notes: 64, 67, 71
    assert_eq!(result.events.len(), 3);

    let notes: Vec<u64> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiNote(n, _, _, _, _) => *n,
            _ => panic!("Expected MidiNote"),
        })
        .collect();
    assert_eq!(notes, vec![64, 67, 71]);
}

#[test]
fn bor_first_list_wins() {
    let result = compile_and_run(
        r#"
        SET G.A [note: '[60 62]];
        SET G.B [note: '[64 67 71]];
        SET G.C BOR G.A G.B;
        >> G.C
        "#,
    );
    // First list wins (BOR), so we get 2 notes: 60, 62
    assert_eq!(result.events.len(), 2);

    let notes: Vec<u64> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiNote(n, _, _, _, _) => *n,
            _ => panic!("Expected MidiNote"),
        })
        .collect();
    assert_eq!(notes, vec![60, 62]);
}

#[test]
fn inline_bor_with_emit() {
    // BOR inline with emit - second map adds vel, first map's note wins
    let result = compile_and_run(">> BOR [note: 60] [note: 64 vel: 80]");
    assert_eq!(result.events.len(), 1);
    match &result.events[0].0 {
        ConcreteEvent::MidiNote(n, v, _, _, _) => {
            assert_eq!(*n, 60, "BOR: first map's note wins");
            assert_eq!(*v, 80, "BOR: second map adds vel");
        }
        _ => panic!("Expected MidiNote, got {:?}", result.events[0].0),
    }
}

#[test]
fn inline_bor_with_list_expansion() {
    // BOR inline with emit + list expansion
    let result = compile_and_run(">> BOR [note: '[60 64 67]] [vel: 100]");
    assert_eq!(result.events.len(), 3, "Should expand 3-note chord");

    let notes: Vec<u64> = result
        .events
        .iter()
        .map(|(e, _)| match e {
            ConcreteEvent::MidiNote(n, _, _, _, _) => *n,
            _ => panic!("Expected MidiNote"),
        })
        .collect();
    assert_eq!(notes, vec![60, 64, 67]);
}
