use crate::bob::BobCompiler;
use sova_core::compiler::Compiler;
use sova_core::vm::Instruction;
use sova_core::vm::control_asm::ControlASM;
use sova_core::vm::variable::Variable;
use std::collections::BTreeMap;

fn compile(source: &str) -> Vec<Instruction> {
    let compiler = BobCompiler;
    compiler
        .compile(source, &BTreeMap::new())
        .expect("compilation failed")
}

fn find_get_midi_cc(instrs: &[Instruction]) -> Option<&ControlASM> {
    instrs.iter().find_map(|i| match i {
        Instruction::Control(c @ ControlASM::GetMidiCC(..)) => Some(c),
        _ => None,
    })
}

#[test]
fn ccin_context_form() {
    let instrs = compile("SET G.X CCIN 1");
    let cc = find_get_midi_cc(&instrs).expect("expected GetMidiCC instruction");
    match cc {
        ControlASM::GetMidiCC(dev, chan, _ctrl, _dest) => {
            assert_eq!(
                *dev,
                Variable::Instance("_use_context_device".to_string()),
                "context form should use _use_context_device"
            );
            assert_eq!(
                *chan,
                Variable::Instance("_use_context_channel".to_string()),
                "context form should use _use_context_channel"
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn ccin_explicit_form() {
    let instrs = compile("SET G.X (CCIN 7 2 1)");
    let cc = find_get_midi_cc(&instrs).expect("expected GetMidiCC instruction");
    match cc {
        ControlASM::GetMidiCC(dev, chan, _ctrl, _dest) => {
            // Explicit form should NOT use context placeholders
            assert_ne!(
                *dev,
                Variable::Instance("_use_context_device".to_string()),
                "explicit form should not use context device"
            );
            assert_ne!(
                *chan,
                Variable::Instance("_use_context_channel".to_string()),
                "explicit form should not use context channel"
            );
        }
        _ => unreachable!(),
    }
}

#[test]
fn ccin_in_emit() {
    let instrs = compile(">> [cc: 1 val: CCIN 1]");
    assert!(
        find_get_midi_cc(&instrs).is_some(),
        "CCIN should compile to GetMidiCC when used in emit"
    );
}

#[test]
fn ccin_with_variable_arg() {
    let instrs = compile("SET G.ctrl 7; SET G.X CCIN G.ctrl");
    assert!(
        find_get_midi_cc(&instrs).is_some(),
        "CCIN should work with variable arguments"
    );
}
