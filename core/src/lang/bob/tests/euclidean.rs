use super::compile_and_run;
use crate::vm::event::ConcreteEvent;
use crate::vm::variable::VariableValue;

// ============================================================================
// Basic Euclidean Pattern Tests
// ============================================================================

#[test]
fn eu_3_8_produces_3_hits() {
    // E(3,8) should produce exactly 3 MIDI notes
    let result = compile_and_run("EU 3 8 0.125 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 3, "E(3,8) should produce 3 hits");
}

#[test]
fn eu_5_8_produces_5_hits() {
    // E(5,8) should produce exactly 5 MIDI notes
    let result = compile_and_run("EU 5 8 0.125 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 5, "E(5,8) should produce 5 hits");
}

#[test]
fn eu_0_8_produces_no_hits() {
    // E(0,8) should produce no hits
    let result = compile_and_run("EU 0 8 0.125 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 0, "E(0,8) should produce 0 hits");
}

#[test]
fn eu_8_8_produces_8_hits() {
    // E(8,8) should produce 8 hits (every step)
    let result = compile_and_run("EU 8 8 0.125 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 8, "E(8,8) should produce 8 hits");
}

#[test]
fn eu_1_4_produces_1_hit() {
    // E(1,4) should produce exactly 1 hit
    let result = compile_and_run("EU 1 4 0.25 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 1, "E(1,4) should produce 1 hit");
}

// ============================================================================
// ELSE Branch Tests
// ============================================================================

#[test]
fn eu_with_else_branch() {
    // E(2,4) with ELSE should produce 2 hits and 2 misses
    let result =
        compile_and_run("EU 2 4 0.25 : >> [note: 60 vel: 100] ELSE : >> [note: 36 vel: 30] END");
    let events: Vec<_> = result
        .events
        .iter()
        .filter_map(|(e, _)| match e {
            ConcreteEvent::MidiNote(note, vel, _, _, _) => Some((*note, *vel)),
            _ => None,
        })
        .collect();

    // Should have 4 total events
    assert_eq!(events.len(), 4, "E(2,4) with ELSE should produce 4 events");

    // Count hits (note 60) and misses (note 36)
    let hits = events.iter().filter(|(n, _)| *n == 60).count();
    let misses = events.iter().filter(|(n, _)| *n == 36).count();
    assert_eq!(hits, 2, "Should have 2 hits");
    assert_eq!(misses, 2, "Should have 2 misses");
}

// ============================================================================
// Index Variable (I) Tests
// ============================================================================

#[test]
fn eu_index_available_in_body() {
    // I should be available and increment 0..steps-1
    let result = compile_and_run("SET G.SUM 0; EU 8 8 0.125 : SET G.SUM + G.SUM I END");
    // Sum of 0+1+2+3+4+5+6+7 = 28
    assert_eq!(
        result.global_vars.get("SUM"),
        Some(&VariableValue::Integer(28))
    );
}

#[test]
fn eu_index_for_velocity_curve() {
    // Use I for velocity curve
    let result = compile_and_run("EU 4 4 0.25 : >> [note: 60 vel: * I 30] END");
    let vels: Vec<u64> = result
        .events
        .iter()
        .filter_map(|(e, _)| match e {
            ConcreteEvent::MidiNote(_, vel, _, _, _) => Some(*vel),
            _ => None,
        })
        .collect();
    // I = 0, 1, 2, 3 → vel = 0, 30, 60, 90
    assert_eq!(vels, vec![0, 30, 60, 90]);
}

// ============================================================================
// Timing Tests
// ============================================================================

#[test]
fn eu_timing_advances_each_step() {
    // Each step should advance time by dur
    // At 120 BPM, 0.125 beats = 62500 μs
    let result = compile_and_run("EU 8 8 0.125 : >> [note: 60] END");
    let times: Vec<_> = result
        .events
        .iter()
        .filter_map(|(e, t)| match e {
            ConcreteEvent::MidiNote(..) => Some(*t),
            _ => None,
        })
        .collect();

    assert_eq!(times.len(), 8);
    // Events should be at 0, 62500, 125000, ... (multiples of 62500)
    for (i, time) in times.iter().enumerate() {
        let expected = (i as u64) * 62500;
        assert_eq!(
            *time, expected,
            "Event {} should be at time {}",
            i, expected
        );
    }
}

#[test]
fn eu_total_duration() {
    // EU 3 8 0.125 should take 8 * 0.125 = 1 beat = 500000 μs at 120 BPM
    let result = compile_and_run("EU 3 8 0.125 : >> [note: 60] END");
    assert_eq!(result.total_time, 500000, "Total duration should be 1 beat");
}

// ============================================================================
// Expression Arguments Tests
// ============================================================================

#[test]
fn eu_with_variable_hits() {
    // hits can be a variable
    let result = compile_and_run("SET G.H 3; EU G.H 8 0.125 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 3);
}

#[test]
fn eu_with_computed_hits() {
    // hits can be a computed expression
    let result = compile_and_run("EU + 1 2 8 0.125 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 3, "1+2=3 hits");
}

#[test]
fn eu_with_variable_steps() {
    // steps can be a variable
    let result = compile_and_run("SET G.S 4; EU 2 G.S 0.25 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 2);
}

// ============================================================================
// Brace Syntax Tests
// ============================================================================

#[test]
fn eu_brace_syntax_with_else() {
    // Brace style requires ELSE - E(2,4) = 2 hits, 2 misses = 4 MIDI events
    let result = compile_and_run("EU 2 4 0.25 { >> [note: 60] } ELSE { >> [note: 36] }");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 4, "E(2,4) with ELSE should produce 4 notes");
}

// ============================================================================
// Complex Body Tests
// ============================================================================

#[test]
fn eu_body_with_multiple_statements() {
    // Body can have multiple statements
    let result = compile_and_run("SET G.X 0; EU 3 8 0.125 : SET G.X + G.X 1; >> [note: 60] END");
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(3))
    );
}

#[test]
fn eu_body_with_internal_wait() {
    // Body can contain additional WAIT
    let result = compile_and_run("EU 2 2 0.25 : >> [note: 60]; WAIT 0.1; >> [note: 72] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 4, "2 hits × 2 notes each = 4 notes");
}

// ============================================================================
// Binary Rhythm Tests
// ============================================================================

#[test]
fn bin_5_is_101_produces_2_hits() {
    // 5 = 0b101 = 3 steps, hits at positions 0 and 2
    let result = compile_and_run("BIN 5 0.125 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 2, "5 = 101 should produce 2 hits");
}

#[test]
fn bin_7_is_111_produces_3_hits() {
    // 7 = 0b111 = 3 steps, all hits
    let result = compile_and_run("BIN 7 0.125 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 3, "7 = 111 should produce 3 hits");
}

#[test]
fn bin_0_produces_no_steps() {
    // 0 has no significant bits, so 0 steps
    let result = compile_and_run("BIN 0 0.125 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 0, "0 should produce 0 steps");
    assert_eq!(result.total_time, 0, "0 pattern should take no time");
}

#[test]
fn bin_1_is_single_hit() {
    // 1 = 0b1 = 1 step, 1 hit
    let result = compile_and_run("BIN 1 0.25 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 1, "1 should produce 1 hit");
}

#[test]
fn bin_170_alternating_pattern() {
    // 170 = 0b10101010 = 8 steps, hits at even positions (0,2,4,6)
    let result = compile_and_run("BIN 170 0.125 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 4, "170 = 10101010 should produce 4 hits");
}

#[test]
fn bin_with_else_branch() {
    // 5 = 101: hit, miss, hit (3 steps total)
    let result =
        compile_and_run("BIN 5 0.125 : >> [note: 60 vel: 100] ELSE : >> [note: 36 vel: 30] END");
    let events: Vec<_> = result
        .events
        .iter()
        .filter_map(|(e, _)| match e {
            ConcreteEvent::MidiNote(note, vel, _, _, _) => Some((*note, *vel)),
            _ => None,
        })
        .collect();

    assert_eq!(events.len(), 3, "5 = 101 should produce 3 events");
    let hits = events.iter().filter(|(n, _)| *n == 60).count();
    let misses = events.iter().filter(|(n, _)| *n == 36).count();
    assert_eq!(hits, 2, "Should have 2 hits");
    assert_eq!(misses, 1, "Should have 1 miss");
}

#[test]
fn bin_index_available_in_body() {
    // I should be available as step index (0..steps-1)
    // 7 = 111, 3 steps, all hits
    let result = compile_and_run("SET G.SUM 0; BIN 7 0.125 : SET G.SUM + G.SUM I END");
    // Sum of 0+1+2 = 3
    assert_eq!(
        result.global_vars.get("SUM"),
        Some(&VariableValue::Integer(3))
    );
}

#[test]
fn bin_timing_advances_each_step() {
    // 7 = 111, 3 steps at 0.125 beats each
    let result = compile_and_run("BIN 7 0.125 : >> [note: 60] END");
    let times: Vec<_> = result
        .events
        .iter()
        .filter_map(|(e, t)| match e {
            ConcreteEvent::MidiNote(..) => Some(*t),
            _ => None,
        })
        .collect();

    assert_eq!(times.len(), 3);
    for (i, time) in times.iter().enumerate() {
        let expected = (i as u64) * 62500;
        assert_eq!(
            *time, expected,
            "Event {} should be at time {}",
            i, expected
        );
    }
}

#[test]
fn bin_total_duration() {
    // 15 = 1111, 4 steps at 0.25 = 1 beat = 500000 μs at 120 BPM
    let result = compile_and_run("BIN 15 0.25 : >> [note: 60] END");
    assert_eq!(result.total_time, 500000, "Total duration should be 1 beat");
}

#[test]
fn bin_with_variable_pattern() {
    // pattern can be a variable
    let result = compile_and_run("SET G.P 5; BIN G.P 0.125 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 2, "G.P=5=101 should produce 2 hits");
}

#[test]
fn bin_brace_syntax_with_else() {
    // 5 = 101: 2 hits, 1 miss
    let result = compile_and_run("BIN 5 0.125 { >> [note: 60] } ELSE { >> [note: 36] }");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 3, "5 = 101 with ELSE should produce 3 notes");
}

#[test]
fn bin_msb_first_order() {
    // Verify MSB-first ordering: 4 = 100, first step is hit, then two misses
    // Use I to verify order
    let result = compile_and_run("SET G.X 99; BIN 4 0.125 : SET G.X I END");
    // 4 = 100, hit at I=0
    assert_eq!(
        result.global_vars.get("X"),
        Some(&VariableValue::Integer(0)),
        "4 = 100 should hit at index 0 (MSB first)"
    );
}

#[test]
fn bin_large_pattern() {
    // Test a larger pattern: 255 = 11111111 (8 bits, 8 hits)
    let result = compile_and_run("BIN 255 0.125 : >> [note: 60] END");
    let note_count = result
        .events
        .iter()
        .filter(|(e, _)| matches!(e, ConcreteEvent::MidiNote(..)))
        .count();
    assert_eq!(note_count, 8, "255 = 11111111 should produce 8 hits");
}
