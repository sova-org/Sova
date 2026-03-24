//! Runtime event emission for the Bob compiler.
//!
//! This module handles emitting events from map variables at runtime,
//! where keys are not known until execution time.

use crate::bob::context::{CompileContext, LabeledInstr, resolve_labels};
use crate::bob::emit::{
    defaults, emit_midi_aftertouch_single, emit_midi_channel_pressure_single,
    emit_midi_control_single, emit_midi_note_single, emit_midi_program_single, emit_with_expansion,
    is_expandable,
};
use sova_core::vm::Instruction;
use sova_core::vm::control_asm::ControlASM;
use sova_core::vm::event::Event;
use sova_core::vm::variable::{Variable, VariableValue};
use std::collections::HashMap;

// ============================================================================
// Helpers
// ============================================================================

/// Extract a key from a map that is known to exist (dispatch already checked).
/// Pattern: dest = default; dest = map[key] (overwrites default)
fn extract_required(
    labeled: &mut Vec<LabeledInstr>,
    map_var: &Variable,
    key: &Variable,
    dest: &Variable,
    default: &Variable,
) {
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        default.clone(),
        dest.clone(),
    ))));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Index(map_var.clone(), key.clone(), dest.clone()),
    )));
}

/// Extract a key from a map that may or may not exist.
/// Pattern: dest = default; if map has key: dest = map[key]
fn extract_optional(
    labeled: &mut Vec<LabeledInstr>,
    map_var: &Variable,
    key: &Variable,
    dest: &Variable,
    default: &Variable,
    cond: &Variable,
    ctx: &mut CompileContext,
) {
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        default.clone(),
        dest.clone(),
    ))));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Contains(map_var.clone(), key.clone(), cond.clone()),
    )));
    let skip = ctx.new_label();
    labeled.push(LabeledInstr::JumpIfNot(cond.clone(), skip.clone()));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Index(map_var.clone(), key.clone(), dest.clone()),
    )));
    labeled.push(LabeledInstr::Mark(skip));
}

/// Emit a MIDI event with expansion, appending to labeled instructions.
fn emit_expanded<F>(
    labeled: &mut Vec<LabeledInstr>,
    keys: &[&str],
    params: HashMap<String, Variable>,
    ctx: &mut CompileContext,
    emit_fn: F,
) where
    F: Fn(&HashMap<String, Variable>) -> Vec<Instruction>,
{
    let expanded = emit_with_expansion(keys, &params, ctx, emit_fn);
    for instr in expanded {
        labeled.push(LabeledInstr::Instr(instr));
    }
}

// ============================================================================
// Runtime Event Emission
// ============================================================================

pub(crate) fn emit_map_var_as_asm(
    map_var: &Variable,
    default_dev: i64,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let mut labeled: Vec<LabeledInstr> = Vec::new();

    // Labels for dispatch chain
    let label_single_map = ctx.new_label();
    let label_list_loop_start = ctx.new_label();
    let label_list_loop_end = ctx.new_label();
    let label_check_pc = ctx.new_label();
    let label_check_at = ctx.new_label();
    let label_check_pressure = ctx.new_label();
    let label_check_note = ctx.new_label();
    let label_emit_dirt = ctx.new_label();
    let label_done = ctx.new_label();

    // Shared variables
    let list_len = ctx.temp("_em_list_len");
    let list_idx = ctx.temp("_em_list_idx");
    let elem_map = ctx.temp("_em_elem_map");
    let cond = ctx.temp("_em_cond");
    let zero = Variable::Constant(VariableValue::Integer(0));
    let one = Variable::Constant(VariableValue::Integer(1));

    // ========== Check if variable is a list of maps ==========
    if !is_expandable(map_var) {
        labeled.push(LabeledInstr::Jump(label_single_map.clone()));
    } else {
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Len(map_var.clone(), list_len.clone()),
        )));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::GreaterThan(list_len.clone(), zero.clone(), cond.clone()),
        )));
        labeled.push(LabeledInstr::JumpIfNot(cond.clone(), label_single_map.clone()));
    }

    // ----- LIST OF MAPS PATH -----
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        zero.clone(),
        list_idx.clone(),
    ))));
    labeled.push(LabeledInstr::Mark(label_list_loop_start.clone()));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::GreaterOrEqual(list_idx.clone(), list_len.clone(), cond.clone()),
    )));
    labeled.push(LabeledInstr::JumpIf(cond.clone(), label_list_loop_end.clone()));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Index(map_var.clone(), list_idx.clone(), elem_map.clone()),
    )));
    {
        let time_var = ctx.temp("_em_lom_time");
        let dev_var = Variable::Constant(VariableValue::Integer(default_dev));
        let mut params: HashMap<String, Variable> = HashMap::new();
        params.insert("_map".to_string(), elem_map.clone());
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::FloatAsFrames(
                Variable::Constant(VariableValue::Float(0.0)),
                time_var.clone(),
            ),
        )));
        labeled.push(LabeledInstr::Instr(Instruction::Effect(
            Event::Dirt {
                sound: Variable::Constant(VariableValue::Str(String::new())),
                params,
                device_id: dev_var,
            },
            time_var,
        )));
    }
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
        list_idx.clone(),
        one,
        list_idx.clone(),
    ))));
    labeled.push(LabeledInstr::Jump(label_list_loop_start));
    labeled.push(LabeledInstr::Mark(label_list_loop_end));
    labeled.push(LabeledInstr::Jump(label_done.clone()));

    // ----- SINGLE MAP PATH -----
    labeled.push(LabeledInstr::Mark(label_single_map));

    // Allocate destination variables for extracted values
    let note_var = ctx.temp("_em_note");
    let vel_var = ctx.temp("_em_vel");
    let chan_var = ctx.temp("_em_chan");
    let dur_var = ctx.temp("_em_dur");
    let cc_var = ctx.temp("_em_cc");
    let val_var = ctx.temp("_em_val");
    let pc_var = ctx.temp("_em_pc");
    let at_var = ctx.temp("_em_at");
    let pressure_var = ctx.temp("_em_pressure");
    let sound_var = ctx.temp("_em_sound");
    let dev_var = ctx.temp("_em_dev");

    // Has-key checks for dispatch
    let has_cc = ctx.temp("_em_has_cc");
    let has_pc = ctx.temp("_em_has_pc");
    let has_at = ctx.temp("_em_has_at");
    let has_note = ctx.temp("_em_has_note");
    let has_pressure = ctx.temp("_em_has_pressure");

    // Key and default constants
    let key = |s: &str| Variable::Constant(VariableValue::Str(s.to_string()));
    let default_dev_var = Variable::Constant(VariableValue::Integer(default_dev));
    let default_note = Variable::Constant(VariableValue::Integer(defaults::MIDI_NOTE));
    let default_vel = Variable::Constant(VariableValue::Integer(defaults::MIDI_VEL));
    let default_chan = Variable::Constant(VariableValue::Integer(defaults::MIDI_CHAN));
    let default_dur = Variable::Constant(VariableValue::Float(defaults::MIDI_DUR));
    let default_cc = Variable::Constant(VariableValue::Integer(defaults::MIDI_CC));
    let default_val = Variable::Constant(VariableValue::Integer(defaults::MIDI_VAL));
    let default_pc = Variable::Constant(VariableValue::Integer(defaults::MIDI_PC));
    let default_at = Variable::Constant(VariableValue::Integer(defaults::MIDI_AT));
    let default_pressure = Variable::Constant(VariableValue::Integer(defaults::MIDI_PRESSURE));
    let default_sound = Variable::Constant(VariableValue::Str("bd".to_string()));

    let key_note = key("note");
    let key_vel = key("vel");
    let key_chan = key("chan");
    let key_dur = key("dur");
    let key_cc = key("cc");
    let key_val = key("val");
    let key_pc = key("pc");
    let key_at = key("at");
    let key_pressure = key("pressure");
    let key_sound = key("sound");
    let key_s = key("s");
    let key_dev = key("dev");

    // Extract dev (common to all paths)
    extract_optional(&mut labeled, map_var, &key_dev, &dev_var, &default_dev_var, &cond, ctx);

    // Check which keys exist for dispatch
    for (k, dest) in [
        (&key_cc, &has_cc), (&key_pc, &has_pc), (&key_at, &has_at),
        (&key_note, &has_note), (&key_pressure, &has_pressure),
    ] {
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Contains(map_var.clone(), k.clone(), dest.clone()),
        )));
    }

    // ========== CC ==========
    labeled.push(LabeledInstr::JumpIfNot(has_cc.clone(), label_check_pc.clone()));
    extract_required(&mut labeled, map_var, &key_cc, &cc_var, &default_cc);
    extract_optional(&mut labeled, map_var, &key_val, &val_var, &default_val, &cond, ctx);
    extract_optional(&mut labeled, map_var, &key_chan, &chan_var, &default_chan, &cond, ctx);
    emit_expanded(&mut labeled, &["cc", "val", "chan"], HashMap::from([
        ("cc".into(), cc_var.clone()), ("val".into(), val_var.clone()), ("chan".into(), chan_var.clone()),
    ]), ctx, |p| emit_midi_control_single(p, &dev_var));
    labeled.push(LabeledInstr::Jump(label_done.clone()));

    // ========== PC ==========
    labeled.push(LabeledInstr::Mark(label_check_pc));
    labeled.push(LabeledInstr::JumpIfNot(has_pc.clone(), label_check_at.clone()));
    extract_required(&mut labeled, map_var, &key_pc, &pc_var, &default_pc);
    extract_optional(&mut labeled, map_var, &key_chan, &chan_var, &default_chan, &cond, ctx);
    emit_expanded(&mut labeled, &["pc", "chan"], HashMap::from([
        ("pc".into(), pc_var.clone()), ("chan".into(), chan_var.clone()),
    ]), ctx, |p| emit_midi_program_single(p, &dev_var));
    labeled.push(LabeledInstr::Jump(label_done.clone()));

    // ========== Aftertouch (requires both at AND note) ==========
    labeled.push(LabeledInstr::Mark(label_check_at));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::And(
        has_at.clone(), has_note.clone(), cond.clone(),
    ))));
    labeled.push(LabeledInstr::JumpIfNot(cond.clone(), label_check_pressure.clone()));
    extract_required(&mut labeled, map_var, &key_note, &note_var, &default_note);
    extract_required(&mut labeled, map_var, &key_at, &at_var, &default_at);
    extract_optional(&mut labeled, map_var, &key_chan, &chan_var, &default_chan, &cond, ctx);
    emit_expanded(&mut labeled, &["note", "at", "chan"], HashMap::from([
        ("note".into(), note_var.clone()), ("at".into(), at_var.clone()), ("chan".into(), chan_var.clone()),
    ]), ctx, |p| emit_midi_aftertouch_single(p, &dev_var));
    labeled.push(LabeledInstr::Jump(label_done.clone()));

    // ========== Channel Pressure ==========
    labeled.push(LabeledInstr::Mark(label_check_pressure));
    labeled.push(LabeledInstr::JumpIfNot(has_pressure.clone(), label_check_note.clone()));
    extract_required(&mut labeled, map_var, &key_pressure, &pressure_var, &default_pressure);
    extract_optional(&mut labeled, map_var, &key_chan, &chan_var, &default_chan, &cond, ctx);
    emit_expanded(&mut labeled, &["pressure", "chan"], HashMap::from([
        ("pressure".into(), pressure_var.clone()), ("chan".into(), chan_var.clone()),
    ]), ctx, |p| emit_midi_channel_pressure_single(p, &dev_var));
    labeled.push(LabeledInstr::Jump(label_done.clone()));

    // ========== MIDI Note ==========
    labeled.push(LabeledInstr::Mark(label_check_note));
    labeled.push(LabeledInstr::JumpIfNot(has_note.clone(), label_emit_dirt.clone()));
    extract_required(&mut labeled, map_var, &key_note, &note_var, &default_note);
    extract_optional(&mut labeled, map_var, &key_vel, &vel_var, &default_vel, &cond, ctx);
    extract_optional(&mut labeled, map_var, &key_chan, &chan_var, &default_chan, &cond, ctx);
    extract_optional(&mut labeled, map_var, &key_dur, &dur_var, &default_dur, &cond, ctx);
    emit_expanded(&mut labeled, &["note", "vel", "chan", "dur"], HashMap::from([
        ("note".into(), note_var.clone()), ("vel".into(), vel_var.clone()),
        ("chan".into(), chan_var.clone()), ("dur".into(), dur_var.clone()),
    ]), ctx, |p| emit_midi_note_single(p, &dev_var));
    labeled.push(LabeledInstr::Jump(label_done.clone()));

    // ========== Dirt (fallback) ==========
    labeled.push(LabeledInstr::Mark(label_emit_dirt));
    extract_optional(&mut labeled, map_var, &key_sound, &sound_var, &default_sound, &cond, ctx);
    // Also check "s" as alias for "sound"
    extract_optional(&mut labeled, map_var, &key_s, &sound_var, &sound_var, &cond, ctx);
    {
        let time_var = ctx.temp("_em_time");
        let mut params: HashMap<String, Variable> = HashMap::new();
        params.insert("_map".to_string(), map_var.clone());
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::FloatAsFrames(
                Variable::Constant(VariableValue::Float(0.0)),
                time_var.clone(),
            ),
        )));
        labeled.push(LabeledInstr::Instr(Instruction::Effect(
            Event::Dirt {
                sound: sound_var.clone(),
                params,
                device_id: dev_var.clone(),
            },
            time_var,
        )));
    }

    // ----- DONE -----
    labeled.push(LabeledInstr::Mark(label_done));
    resolve_labels(labeled)
}
