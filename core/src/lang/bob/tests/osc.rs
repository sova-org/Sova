use super::compile_and_run;
use crate::vm::event::ConcreteEvent;

macro_rules! assert_osc {
    ($result:expr) => {
        assert_eq!($result.events.len(), 1);
        assert!(
            matches!($result.events[0].0, ConcreteEvent::Osc { .. }),
            "Expected Osc, got {:?}",
            $result.events[0].0
        );
    };
}

#[test]
fn osc_basic() {
    let result = compile_and_run(">> [addr: \"/synth\"]");
    assert_osc!(result);
}

#[test]
fn osc_with_params() {
    let result = compile_and_run(">> [addr: \"/synth\" freq: 440 amp: 0.5]");
    assert_osc!(result);
}

#[test]
fn osc_with_device() {
    let result = compile_and_run(">> [addr: \"/ctrl\" dev: 2 x: 100]");
    assert_osc!(result);
}

#[test]
fn osc_with_expressions() {
    let result = compile_and_run("SET G.X 440; >> [addr: \"/synth\" freq: MUL G.X 2]");
    assert_osc!(result);
}

#[test]
fn osc_priority_over_note() {
    // addr takes priority over note - sends OSC, not MIDI
    let result = compile_and_run(">> [addr: \"/synth\" note: 60]");
    assert_osc!(result);
}

#[test]
fn osc_multiple_params() {
    let result = compile_and_run(">> [addr: \"/fx\" delay: 0.25 feedback: 0.7 mix: 0.5]");
    assert_osc!(result);
}
