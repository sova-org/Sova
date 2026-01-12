use super::compile_and_run;
use crate::vm::event::ConcreteEvent;

macro_rules! assert_sysex {
    ($result:expr) => {
        assert_eq!($result.events.len(), 1);
        assert!(
            matches!($result.events[0].0, ConcreteEvent::MidiSystemExclusive(..)),
            "Expected MidiSystemExclusive, got {:?}",
            $result.events[0].0
        );
    };
}

#[test]
fn sysex_basic() {
    let result = compile_and_run(">> [sysex: BYTES: 240 67 32 0 247 END]");
    assert_sysex!(result);
}

#[test]
fn sysex_with_device() {
    let result = compile_and_run(">> [sysex: BYTES: 240 67 0 247 END dev: 2]");
    assert_sysex!(result);
}

#[test]
fn sysex_with_variables() {
    let result =
        compile_and_run("SET G.X 67; SET G.Y 32; >> [sysex: BYTES: 240 G.X G.Y 0 247 END]");
    assert_sysex!(result);
}

#[test]
fn sysex_with_expressions() {
    let result = compile_and_run(">> [sysex: BYTES: 240 ADD 60 7 MUL 2 16 247 END]");
    assert_sysex!(result);
}

#[test]
fn sysex_minimal() {
    let result = compile_and_run(">> [sysex: BYTES: 240 247 END]");
    assert_sysex!(result);
}
