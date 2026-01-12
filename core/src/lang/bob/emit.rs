//! Compile-time event emission for the Bob compiler.
//!
//! This module handles emitting events from map literals with known keys at compile time.
//! It includes list parameter expansion for broadcasting events across list values.

use crate::lang::bob::bob_ast::BobExpr;
use crate::lang::bob::context::{CompileContext, LabeledInstr, resolve_labels};
use crate::vm::Instruction;
use crate::vm::control_asm::ControlASM;
use crate::vm::event::Event;
use crate::vm::variable::{Variable, VariableValue};
use std::collections::HashMap;

// Import compile_expr and bob_value_to_variable from compile_expr module
use super::compile_expr::{bob_value_to_variable, compile_expr};

// ============================================================================
// Default Values
// ============================================================================

pub(crate) mod defaults {
    pub const MIDI_NOTE: i64 = 60;
    pub const MIDI_VEL: i64 = 100;
    pub const MIDI_DUR: f64 = 0.5;
    pub const MIDI_CHAN: i64 = 0;
    pub const MIDI_CC: i64 = 0;
    pub const MIDI_VAL: i64 = 0;
    pub const MIDI_PRESSURE: i64 = 0;
    pub const MIDI_AT: i64 = 0;
    pub const MIDI_PC: i64 = 0;
}

// ============================================================================
// Emit Helpers
// ============================================================================

pub(crate) fn emit_immediate(event: Event) -> Vec<Instruction> {
    let time_var = Variable::Instance("_bob_time".to_string());
    vec![
        Instruction::Control(ControlASM::FloatAsFrames(
            Variable::Constant(VariableValue::Float(0.0)),
            time_var.clone(),
        )),
        Instruction::Effect(event, time_var),
    ]
}

/// Generates labeled instructions that expand list values in params and emits multiple events.
///
/// Algorithm:
/// 1. For each param, check if it's a Vec (VecLen > 0)
/// 2. Compute max_len = max of all Vec lengths (scalars have len 0, treated as repeat)
/// 3. If max_len == 0, all are scalars -> emit single event
/// 4. Otherwise, loop 0..max_len, extracting value[i % len] for Vecs, direct for scalars
pub(crate) fn emit_with_expansion<F>(
    param_keys: &[&str],
    compiled: &HashMap<String, Variable>,
    ctx: &mut CompileContext,
    emit_single: F,
) -> Vec<Instruction>
where
    F: Fn(&HashMap<String, Variable>) -> Vec<Instruction>,
{
    let params_to_expand: Vec<(&str, Variable)> = param_keys
        .iter()
        .filter_map(|k| compiled.get(*k).map(|v| (*k, v.clone())))
        .collect();

    if params_to_expand.is_empty() {
        return emit_single(compiled);
    }

    let mut labeled: Vec<LabeledInstr> = Vec::new();

    // Labels
    let label_loop_start = ctx.new_label();
    let label_loop_end = ctx.new_label();
    let label_scalar_fallback = ctx.new_label();
    let label_done = ctx.new_label();

    // Variables
    let max_len_var = ctx.temp("_exp_max");
    let idx_var = ctx.temp("_exp_idx");
    let cond_var = ctx.temp("_exp_cond");
    let zero = Variable::Constant(VariableValue::Integer(0));
    let one = Variable::Constant(VariableValue::Integer(1));

    // Initialize max_len = 0
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        zero.clone(),
        max_len_var.clone(),
    ))));

    // For each param, compute VecLen and update max_len
    let mut len_vars: HashMap<&str, Variable> = HashMap::new();
    for (key, var) in &params_to_expand {
        let len_var = ctx.temp("_exp_len");
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::VecLen(var.clone(), len_var.clone()),
        )));

        // if len_var > max_len then max_len = len_var
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::GreaterThan(len_var.clone(), max_len_var.clone(), cond_var.clone()),
        )));
        let skip_update = ctx.new_label();
        labeled.push(LabeledInstr::JumpIfNot(
            cond_var.clone(),
            skip_update.clone(),
        ));
        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            len_var.clone(),
            max_len_var.clone(),
        ))));
        labeled.push(LabeledInstr::Mark(skip_update));

        len_vars.insert(*key, len_var);
    }

    // If max_len == 0, jump to scalar fallback
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Equal(max_len_var.clone(), zero.clone(), cond_var.clone()),
    )));
    labeled.push(LabeledInstr::JumpIf(
        cond_var.clone(),
        label_scalar_fallback.clone(),
    ));

    // Initialize idx = 0
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        zero.clone(),
        idx_var.clone(),
    ))));

    // LOOP START
    labeled.push(LabeledInstr::Mark(label_loop_start.clone()));

    // if idx >= max_len, exit loop
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::GreaterOrEqual(idx_var.clone(), max_len_var.clone(), cond_var.clone()),
    )));
    labeled.push(LabeledInstr::JumpIf(
        cond_var.clone(),
        label_loop_end.clone(),
    ));

    // Extract values for this iteration
    let mut extracted: HashMap<String, Variable> = compiled.clone();
    for (key, var) in &params_to_expand {
        let len_var = len_vars.get(key).unwrap();
        let extracted_var = ctx.temp("_exp_val");
        let wrapped_idx_var = ctx.temp("_exp_widx");

        // if len == 0 (scalar), use original value
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Equal(len_var.clone(), zero.clone(), cond_var.clone()),
        )));
        let use_vecget = ctx.new_label();
        let after_extract = ctx.new_label();
        labeled.push(LabeledInstr::JumpIfNot(
            cond_var.clone(),
            use_vecget.clone(),
        ));

        // Scalar path: copy original
        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            var.clone(),
            extracted_var.clone(),
        ))));
        labeled.push(LabeledInstr::Jump(after_extract.clone()));

        // VecGet path
        labeled.push(LabeledInstr::Mark(use_vecget));
        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mod(
            idx_var.clone(),
            len_var.clone(),
            wrapped_idx_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::VecGet(var.clone(), wrapped_idx_var, extracted_var.clone()),
        )));
        labeled.push(LabeledInstr::Mark(after_extract));

        extracted.insert(key.to_string(), extracted_var);
    }

    // Emit event with extracted values
    for instr in emit_single(&extracted) {
        labeled.push(LabeledInstr::Instr(instr));
    }

    // idx++
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
        idx_var.clone(),
        one,
        idx_var.clone(),
    ))));

    // Jump back to loop start
    labeled.push(LabeledInstr::Jump(label_loop_start));

    // LOOP END - skip scalar fallback
    labeled.push(LabeledInstr::Mark(label_loop_end));
    labeled.push(LabeledInstr::Jump(label_done.clone()));

    // SCALAR FALLBACK
    labeled.push(LabeledInstr::Mark(label_scalar_fallback));
    for instr in emit_single(compiled) {
        labeled.push(LabeledInstr::Instr(instr));
    }

    // DONE
    labeled.push(LabeledInstr::Mark(label_done));

    resolve_labels(labeled)
}

// ============================================================================
// Event Emission
// ============================================================================

pub(crate) fn emit_as_asm(
    pairs: &[(String, BobExpr)],
    default_dev: i64,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let mut instrs = Vec::new();
    let mut compiled: HashMap<String, Variable> = HashMap::new();

    for (i, (key, expr)) in pairs.iter().enumerate() {
        match expr {
            BobExpr::Value(v) => {
                compiled.insert(key.clone(), bob_value_to_variable(v));
            }
            _ => {
                let temp = Variable::Instance(format!("_bob_emit_{i}"));
                instrs.extend(compile_expr(expr, &temp, ctx));
                compiled.insert(key.clone(), temp);
            }
        }
    }

    let device_id = compiled
        .get("dev")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(default_dev)));

    let keys: Vec<&str> = pairs.iter().map(|(k, _)| k.as_str()).collect();

    let is_truthy = |key: &str| -> bool {
        compiled
            .get(key)
            .is_some_and(|v| !matches!(v, Variable::Constant(VariableValue::Integer(0))))
    };

    // Priority-based dispatch (first match wins)
    // 1. Transport events (require truthy value)
    if keys.contains(&"start") && is_truthy("start") {
        instrs.extend(emit_midi_start(&device_id));
    } else if keys.contains(&"stop") && is_truthy("stop") {
        instrs.extend(emit_midi_stop(&device_id));
    } else if keys.contains(&"reset") && is_truthy("reset") {
        instrs.extend(emit_midi_reset(&device_id));
    } else if keys.contains(&"continue") && is_truthy("continue") {
        instrs.extend(emit_midi_continue(&device_id));
    } else if keys.contains(&"clock") && is_truthy("clock") {
        instrs.extend(emit_midi_clock(&device_id));
    }
    // 2. SysEx (needs original expression for BYTES)
    else if keys.contains(&"sysex") {
        if let Some((_, sysex_expr)) = pairs.iter().find(|(k, _)| k == "sysex") {
            instrs.extend(emit_midi_sysex(sysex_expr, &device_id, ctx));
        }
    }
    // 3. CC
    else if keys.contains(&"cc") {
        instrs.extend(emit_midi_control(&compiled, &device_id, ctx));
    }
    // 4. Program Change
    else if keys.contains(&"pc") {
        instrs.extend(emit_midi_program(&compiled, &device_id, ctx));
    }
    // 5. Polyphonic Aftertouch (requires both at AND note)
    else if keys.contains(&"at") && keys.contains(&"note") {
        instrs.extend(emit_midi_aftertouch(&compiled, &device_id, ctx));
    }
    // 6. Channel Pressure
    else if keys.contains(&"pressure") {
        instrs.extend(emit_midi_channel_pressure(&compiled, &device_id, ctx));
    }
    // 7. OSC
    else if keys.contains(&"addr") {
        instrs.extend(emit_osc(pairs, &compiled, &device_id, ctx));
    }
    // 8. MIDI Note
    else if keys.iter().any(|k| *k == "note" || *k == "vel") {
        instrs.extend(emit_midi_note(&compiled, &device_id, ctx));
    }
    // 9. Dirt with sound
    else if keys.iter().any(|k| *k == "sound" || *k == "s") {
        instrs.extend(emit_dirt(&compiled, &device_id, ctx));
    }
    // 10. Dirt generic
    else {
        instrs.extend(emit_dirt_generic(&compiled, &device_id, ctx));
    }

    instrs
}

pub(crate) fn emit_midi_note_single(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
) -> Vec<Instruction> {
    let note = compiled
        .get("note")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(
            defaults::MIDI_NOTE,
        )));

    let vel = compiled
        .get("vel")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(
            defaults::MIDI_VEL,
        )));

    let chan = compiled
        .get("chan")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(
            defaults::MIDI_CHAN,
        )));

    let dur = compiled
        .get("dur")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Float(defaults::MIDI_DUR)));

    let dur_frames_var = Variable::Instance("_bob_dur".to_string());
    let time_var = Variable::Instance("_bob_time".to_string());

    let event = Event::MidiNote(note, vel, chan, dur_frames_var.clone(), device_id.clone());

    vec![
        Instruction::Control(ControlASM::FloatAsFrames(dur, dur_frames_var)),
        Instruction::Control(ControlASM::FloatAsFrames(
            Variable::Constant(VariableValue::Float(0.0)),
            time_var.clone(),
        )),
        Instruction::Effect(event, time_var),
    ]
}

fn emit_midi_note(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let device_id = device_id.clone();
    emit_with_expansion(
        &["note", "vel", "chan", "dur"],
        compiled,
        ctx,
        move |params| emit_midi_note_single(params, &device_id),
    )
}

pub(crate) fn emit_midi_control_single(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
) -> Vec<Instruction> {
    let cc = compiled
        .get("cc")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(
            defaults::MIDI_CC,
        )));

    let val = compiled
        .get("val")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(
            defaults::MIDI_VAL,
        )));

    let chan = compiled
        .get("chan")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(
            defaults::MIDI_CHAN,
        )));

    emit_immediate(Event::MidiControl(cc, val, chan, device_id.clone()))
}

fn emit_midi_control(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let device_id = device_id.clone();
    emit_with_expansion(&["cc", "val", "chan"], compiled, ctx, move |params| {
        emit_midi_control_single(params, &device_id)
    })
}

pub(crate) fn emit_midi_program_single(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
) -> Vec<Instruction> {
    let pc = compiled
        .get("pc")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(
            defaults::MIDI_PC,
        )));

    let chan = compiled
        .get("chan")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(
            defaults::MIDI_CHAN,
        )));

    emit_immediate(Event::MidiProgram(pc, chan, device_id.clone()))
}

fn emit_midi_program(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let device_id = device_id.clone();
    emit_with_expansion(&["pc", "chan"], compiled, ctx, move |params| {
        emit_midi_program_single(params, &device_id)
    })
}

pub(crate) fn emit_midi_aftertouch_single(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
) -> Vec<Instruction> {
    let note = compiled
        .get("note")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(
            defaults::MIDI_NOTE,
        )));

    let at = compiled
        .get("at")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(
            defaults::MIDI_AT,
        )));

    let chan = compiled
        .get("chan")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(
            defaults::MIDI_CHAN,
        )));

    emit_immediate(Event::MidiAftertouch(note, at, chan, device_id.clone()))
}

fn emit_midi_aftertouch(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let device_id = device_id.clone();
    emit_with_expansion(&["note", "at", "chan"], compiled, ctx, move |params| {
        emit_midi_aftertouch_single(params, &device_id)
    })
}

pub(crate) fn emit_midi_channel_pressure_single(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
) -> Vec<Instruction> {
    let pressure = compiled
        .get("pressure")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(
            defaults::MIDI_PRESSURE,
        )));

    let chan = compiled
        .get("chan")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Integer(
            defaults::MIDI_CHAN,
        )));

    emit_immediate(Event::MidiChannelPressure(
        pressure,
        chan,
        device_id.clone(),
    ))
}

fn emit_midi_channel_pressure(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let device_id = device_id.clone();
    emit_with_expansion(&["pressure", "chan"], compiled, ctx, move |params| {
        emit_midi_channel_pressure_single(params, &device_id)
    })
}

pub(crate) fn emit_midi_start(device_id: &Variable) -> Vec<Instruction> {
    emit_immediate(Event::MidiStart(device_id.clone()))
}

pub(crate) fn emit_midi_stop(device_id: &Variable) -> Vec<Instruction> {
    emit_immediate(Event::MidiStop(device_id.clone()))
}

pub(crate) fn emit_midi_reset(device_id: &Variable) -> Vec<Instruction> {
    emit_immediate(Event::MidiReset(device_id.clone()))
}

pub(crate) fn emit_midi_continue(device_id: &Variable) -> Vec<Instruction> {
    emit_immediate(Event::MidiContinue(device_id.clone()))
}

pub(crate) fn emit_midi_clock(device_id: &Variable) -> Vec<Instruction> {
    emit_immediate(Event::MidiClock(device_id.clone()))
}

pub(crate) fn emit_midi_sysex(
    sysex_expr: &BobExpr,
    device_id: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let mut instrs = Vec::new();
    let mut data_vars: Vec<Variable> = Vec::new();

    if let BobExpr::Bytes(bytes) = sysex_expr {
        for byte_expr in bytes.iter() {
            let temp = ctx.temp("_bob_sysex");
            instrs.extend(compile_expr(byte_expr, &temp, ctx));
            data_vars.push(temp);
        }
    }

    instrs.extend(emit_immediate(Event::MidiSystemExclusive(
        data_vars,
        device_id.clone(),
    )));
    instrs
}

fn emit_osc_single(
    pairs: &[(String, BobExpr)],
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
) -> Vec<Instruction> {
    let addr = compiled
        .get("addr")
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Str(
            "/default".to_string(),
        )));

    let mut args: Vec<Variable> = Vec::new();
    for (key, _) in pairs {
        if key != "addr" && key != "dev" {
            if let Some(var) = compiled.get(key) {
                args.push(Variable::Constant(VariableValue::Str(key.clone())));
                args.push(var.clone());
            }
        }
    }

    emit_immediate(Event::Osc {
        addr,
        args,
        device_id: device_id.clone(),
    })
}

fn emit_osc(
    pairs: &[(String, BobExpr)],
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let device_id = device_id.clone();
    let pairs = pairs.to_vec();

    // Get all keys that should be checked for expansion (all except dev)
    let expand_keys: Vec<&str> = compiled
        .keys()
        .filter(|k| *k != "dev")
        .map(|s| s.as_str())
        .collect();

    emit_with_expansion(&expand_keys, compiled, ctx, move |params| {
        emit_osc_single(&pairs, params, &device_id)
    })
}

fn emit_dirt_single(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
) -> Vec<Instruction> {
    let sound = compiled
        .get("sound")
        .or_else(|| compiled.get("s"))
        .cloned()
        .unwrap_or(Variable::Constant(VariableValue::Str("bd".to_string())));

    let mut params: HashMap<String, Variable> = HashMap::new();
    for (key, var) in compiled {
        if key != "sound" && key != "s" && key != "dev" {
            params.insert(key.clone(), var.clone());
        }
    }

    emit_immediate(Event::Dirt {
        sound,
        params,
        device_id: device_id.clone(),
    })
}

fn emit_dirt(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let device_id = device_id.clone();

    // Get all keys that should be checked for expansion (all except dev)
    let expand_keys: Vec<&str> = compiled
        .keys()
        .filter(|k| *k != "dev")
        .map(|s| s.as_str())
        .collect();

    emit_with_expansion(&expand_keys, compiled, ctx, move |params| {
        emit_dirt_single(params, &device_id)
    })
}

fn emit_dirt_generic_single(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
) -> Vec<Instruction> {
    let mut params: HashMap<String, Variable> = HashMap::new();
    for (key, var) in compiled {
        if key != "dev" {
            params.insert(key.clone(), var.clone());
        }
    }

    emit_immediate(Event::Dirt {
        sound: Variable::Constant(VariableValue::Str(String::new())),
        params,
        device_id: device_id.clone(),
    })
}

fn emit_dirt_generic(
    compiled: &HashMap<String, Variable>,
    device_id: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let device_id = device_id.clone();

    // Get all keys that should be checked for expansion (all except dev)
    let expand_keys: Vec<&str> = compiled
        .keys()
        .filter(|k| *k != "dev")
        .map(|s| s.as_str())
        .collect();

    emit_with_expansion(&expand_keys, compiled, ctx, move |params| {
        emit_dirt_generic_single(params, &device_id)
    })
}
