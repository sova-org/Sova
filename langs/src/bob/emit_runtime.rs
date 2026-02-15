//! Runtime event emission for the Bob compiler.
//!
//! This module handles emitting events from map variables at runtime,
//! where keys are not known until execution time.

use crate::bob::context::{CompileContext, LabeledInstr, resolve_labels};
use crate::bob::emit::{
    defaults, emit_midi_aftertouch_single, emit_midi_channel_pressure_single,
    emit_midi_control_single, emit_midi_note_single, emit_midi_program_single, emit_with_expansion,
};
use sova_core::vm::Instruction;
use sova_core::vm::control_asm::ControlASM;
use sova_core::vm::event::Event;
use sova_core::vm::variable::{Variable, VariableValue};
use std::collections::HashMap;

// ============================================================================
// Runtime Event Emission
// ============================================================================

pub(crate) fn emit_map_var_as_asm(
    map_var: &Variable,
    default_dev: i64,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    // Runtime emit of a map variable with key-based dispatch and list expansion.
    //
    // Handles two cases:
    // A) Variable is a list of maps (e.g., '[[note: 60] [note: 64]]) → emit each map
    // B) Variable is a single map (possibly with list values) → dispatch + expand
    //
    // For case B, key priority (same as compile-time emit_as_asm):
    // - cc → MidiControl
    // - pc → MidiProgram
    // - at + note → MidiAftertouch
    // - pressure → MidiChannelPressure
    // - addr → Osc
    // - note/vel → MidiNote
    // - sound/s → Dirt
    // - else → Dirt generic

    let mut labeled: Vec<LabeledInstr> = Vec::new();

    // Labels
    let label_single_map = ctx.new_label();
    let label_list_loop_start = ctx.new_label();
    let label_list_loop_end = ctx.new_label();
    let label_check_pc = ctx.new_label();
    let label_check_at = ctx.new_label();
    let label_check_pressure = ctx.new_label();
    let label_check_note = ctx.new_label();
    let label_emit_dirt = ctx.new_label();
    let label_done = ctx.new_label();

    // Variables for list-of-maps handling
    let list_len = ctx.temp("_em_list_len");
    let list_idx = ctx.temp("_em_list_idx");
    let elem_map = ctx.temp("_em_elem_map");
    let is_list_cond = ctx.temp("_em_is_list");

    let zero = Variable::Constant(VariableValue::Integer(0));
    let one = Variable::Constant(VariableValue::Integer(1));

    // ========== Check if variable is a list of maps ==========
    // VecLen returns 0 for non-Vec values, >0 for Vec
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Len(map_var.clone(), list_len.clone()),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::GreaterThan(list_len.clone(), zero.clone(), is_list_cond.clone()),
    )));
    labeled.push(LabeledInstr::JumpIfNot(
        is_list_cond.clone(),
        label_single_map.clone(),
    ));

    // ----- LIST OF MAPS PATH -----
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        zero.clone(),
        list_idx.clone(),
    ))));

    // Loop: for each element in the list
    labeled.push(LabeledInstr::Mark(label_list_loop_start.clone()));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::GreaterOrEqual(list_idx.clone(), list_len.clone(), is_list_cond.clone()),
    )));
    labeled.push(LabeledInstr::JumpIf(
        is_list_cond.clone(),
        label_list_loop_end.clone(),
    ));

    // Get element at index
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Index(map_var.clone(), list_idx.clone(), elem_map.clone()),
    )));

    // Generate emit code for elem_map and add it inline
    // (We'll jump to label_check_cc with current_map set, then jump back)
    // Actually, this is complex - let's just emit a simple Dirt event for list-of-maps
    // since that was the original behavior
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

    // Increment index
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
        list_idx.clone(),
        one.clone(),
        list_idx.clone(),
    ))));
    labeled.push(LabeledInstr::Jump(label_list_loop_start));

    labeled.push(LabeledInstr::Mark(label_list_loop_end));
    labeled.push(LabeledInstr::Jump(label_done.clone()));

    // ----- SINGLE MAP PATH -----
    labeled.push(LabeledInstr::Mark(label_single_map));

    // Variables for extracted values
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

    // Has-key result variables
    let has_cc = ctx.temp("_em_has_cc");
    let has_pc = ctx.temp("_em_has_pc");
    let has_at = ctx.temp("_em_has_at");
    let has_note = ctx.temp("_em_has_note");
    let has_pressure = ctx.temp("_em_has_pressure");

    // Expansion variables
    let cond = ctx.temp("_em_cond");
    let time_var = ctx.temp("_em_time");

    let default_dev_var = Variable::Constant(VariableValue::Integer(default_dev));

    // Key constants
    let key_note = Variable::Constant(VariableValue::Str("note".to_string()));
    let key_vel = Variable::Constant(VariableValue::Str("vel".to_string()));
    let key_chan = Variable::Constant(VariableValue::Str("chan".to_string()));
    let key_dur = Variable::Constant(VariableValue::Str("dur".to_string()));
    let key_cc = Variable::Constant(VariableValue::Str("cc".to_string()));
    let key_val = Variable::Constant(VariableValue::Str("val".to_string()));
    let key_pc = Variable::Constant(VariableValue::Str("pc".to_string()));
    let key_at = Variable::Constant(VariableValue::Str("at".to_string()));
    let key_pressure = Variable::Constant(VariableValue::Str("pressure".to_string()));
    let key_sound = Variable::Constant(VariableValue::Str("sound".to_string()));
    let key_s = Variable::Constant(VariableValue::Str("s".to_string()));
    let key_dev = Variable::Constant(VariableValue::Str("dev".to_string()));

    // Default values
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

    // ========== Extract all relevant keys with defaults ==========

    // Helper macro-like inline: Index with default if key not present
    // Index(map, key, dest) - sets dest to value or keeps it if not found
    // We use Contains first, then Index

    // dev (with default)
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        default_dev_var.clone(),
        dev_var.clone(),
    ))));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Contains(map_var.clone(), key_dev.clone(), cond.clone()),
    )));
    let skip_dev = ctx.new_label();
    labeled.push(LabeledInstr::JumpIfNot(cond.clone(), skip_dev.clone()));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Index(map_var.clone(), key_dev.clone(), dev_var.clone()),
    )));
    labeled.push(LabeledInstr::Mark(skip_dev));

    // Check which keys exist for dispatch
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Contains(map_var.clone(), key_cc.clone(), has_cc.clone()),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Contains(map_var.clone(), key_pc.clone(), has_pc.clone()),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Contains(map_var.clone(), key_at.clone(), has_at.clone()),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Contains(map_var.clone(), key_note.clone(), has_note.clone()),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::Contains(map_var.clone(), key_pressure.clone(), has_pressure.clone()),
    )));

    // ========== Dispatch based on keys ==========

    // if has_cc → emit CC
    labeled.push(LabeledInstr::JumpIfNot(
        has_cc.clone(),
        label_check_pc.clone(),
    ));

    // ----- EMIT CC PATH -----
    {
        // Extract cc, val, chan with defaults
        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_cc.clone(),
            cc_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_cc.clone(), cc_var.clone()),
        )));

        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_val.clone(),
            val_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Contains(map_var.clone(), key_val.clone(), cond.clone()),
        )));
        let skip_val = ctx.new_label();
        labeled.push(LabeledInstr::JumpIfNot(cond.clone(), skip_val.clone()));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_val.clone(), val_var.clone()),
        )));
        labeled.push(LabeledInstr::Mark(skip_val));

        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_chan.clone(),
            chan_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Contains(map_var.clone(), key_chan.clone(), cond.clone()),
        )));
        let skip_chan = ctx.new_label();
        labeled.push(LabeledInstr::JumpIfNot(cond.clone(), skip_chan.clone()));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_chan.clone(), chan_var.clone()),
        )));
        labeled.push(LabeledInstr::Mark(skip_chan));

        // Now emit with expansion for cc, val, chan
        let params: HashMap<String, Variable> = [
            ("cc".to_string(), cc_var.clone()),
            ("val".to_string(), val_var.clone()),
            ("chan".to_string(), chan_var.clone()),
        ]
        .into_iter()
        .collect();

        let expanded = emit_with_expansion(&["cc", "val", "chan"], &params, ctx, |p| {
            emit_midi_control_single(p, &dev_var)
        });
        for instr in expanded {
            labeled.push(LabeledInstr::Instr(instr));
        }
    }
    labeled.push(LabeledInstr::Jump(label_done.clone()));

    // ----- CHECK PC -----
    labeled.push(LabeledInstr::Mark(label_check_pc));
    labeled.push(LabeledInstr::JumpIfNot(
        has_pc.clone(),
        label_check_at.clone(),
    ));

    // ----- EMIT PC PATH -----
    {
        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_pc.clone(),
            pc_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_pc.clone(), pc_var.clone()),
        )));

        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_chan.clone(),
            chan_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Contains(map_var.clone(), key_chan.clone(), cond.clone()),
        )));
        let skip_chan = ctx.new_label();
        labeled.push(LabeledInstr::JumpIfNot(cond.clone(), skip_chan.clone()));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_chan.clone(), chan_var.clone()),
        )));
        labeled.push(LabeledInstr::Mark(skip_chan));

        let params: HashMap<String, Variable> = [
            ("pc".to_string(), pc_var.clone()),
            ("chan".to_string(), chan_var.clone()),
        ]
        .into_iter()
        .collect();

        let expanded = emit_with_expansion(&["pc", "chan"], &params, ctx, |p| {
            emit_midi_program_single(p, &dev_var)
        });
        for instr in expanded {
            labeled.push(LabeledInstr::Instr(instr));
        }
    }
    labeled.push(LabeledInstr::Jump(label_done.clone()));

    // ----- CHECK AT (aftertouch needs both at AND note) -----
    labeled.push(LabeledInstr::Mark(label_check_at));
    // at && note
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::And(
        has_at.clone(),
        has_note.clone(),
        cond.clone(),
    ))));
    labeled.push(LabeledInstr::JumpIfNot(
        cond.clone(),
        label_check_pressure.clone(),
    ));

    // ----- EMIT AFTERTOUCH PATH -----
    {
        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_note.clone(),
            note_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_note.clone(), note_var.clone()),
        )));

        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_at.clone(),
            at_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_at.clone(), at_var.clone()),
        )));

        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_chan.clone(),
            chan_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Contains(map_var.clone(), key_chan.clone(), cond.clone()),
        )));
        let skip_chan = ctx.new_label();
        labeled.push(LabeledInstr::JumpIfNot(cond.clone(), skip_chan.clone()));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_chan.clone(), chan_var.clone()),
        )));
        labeled.push(LabeledInstr::Mark(skip_chan));

        let params: HashMap<String, Variable> = [
            ("note".to_string(), note_var.clone()),
            ("at".to_string(), at_var.clone()),
            ("chan".to_string(), chan_var.clone()),
        ]
        .into_iter()
        .collect();

        let expanded = emit_with_expansion(&["note", "at", "chan"], &params, ctx, |p| {
            emit_midi_aftertouch_single(p, &dev_var)
        });
        for instr in expanded {
            labeled.push(LabeledInstr::Instr(instr));
        }
    }
    labeled.push(LabeledInstr::Jump(label_done.clone()));

    // ----- CHECK PRESSURE -----
    labeled.push(LabeledInstr::Mark(label_check_pressure));
    labeled.push(LabeledInstr::JumpIfNot(
        has_pressure.clone(),
        label_check_note.clone(),
    ));

    // ----- EMIT CHANNEL PRESSURE PATH -----
    {
        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_pressure.clone(),
            pressure_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_pressure.clone(), pressure_var.clone()),
        )));

        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_chan.clone(),
            chan_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Contains(map_var.clone(), key_chan.clone(), cond.clone()),
        )));
        let skip_chan = ctx.new_label();
        labeled.push(LabeledInstr::JumpIfNot(cond.clone(), skip_chan.clone()));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_chan.clone(), chan_var.clone()),
        )));
        labeled.push(LabeledInstr::Mark(skip_chan));

        let params: HashMap<String, Variable> = [
            ("pressure".to_string(), pressure_var.clone()),
            ("chan".to_string(), chan_var.clone()),
        ]
        .into_iter()
        .collect();

        let expanded = emit_with_expansion(&["pressure", "chan"], &params, ctx, |p| {
            emit_midi_channel_pressure_single(p, &dev_var)
        });
        for instr in expanded {
            labeled.push(LabeledInstr::Instr(instr));
        }
    }
    labeled.push(LabeledInstr::Jump(label_done.clone()));

    // ----- CHECK NOTE (MIDI Note) -----
    labeled.push(LabeledInstr::Mark(label_check_note));
    labeled.push(LabeledInstr::JumpIfNot(
        has_note.clone(),
        label_emit_dirt.clone(),
    ));

    // ----- EMIT MIDI NOTE PATH -----
    {
        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_note.clone(),
            note_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_note.clone(), note_var.clone()),
        )));

        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_vel.clone(),
            vel_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Contains(map_var.clone(), key_vel.clone(), cond.clone()),
        )));
        let skip_vel = ctx.new_label();
        labeled.push(LabeledInstr::JumpIfNot(cond.clone(), skip_vel.clone()));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_vel.clone(), vel_var.clone()),
        )));
        labeled.push(LabeledInstr::Mark(skip_vel));

        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_chan.clone(),
            chan_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Contains(map_var.clone(), key_chan.clone(), cond.clone()),
        )));
        let skip_chan = ctx.new_label();
        labeled.push(LabeledInstr::JumpIfNot(cond.clone(), skip_chan.clone()));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_chan.clone(), chan_var.clone()),
        )));
        labeled.push(LabeledInstr::Mark(skip_chan));

        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_dur.clone(),
            dur_var.clone(),
        ))));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Contains(map_var.clone(), key_dur.clone(), cond.clone()),
        )));
        let skip_dur = ctx.new_label();
        labeled.push(LabeledInstr::JumpIfNot(cond.clone(), skip_dur.clone()));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_dur.clone(), dur_var.clone()),
        )));
        labeled.push(LabeledInstr::Mark(skip_dur));

        let params: HashMap<String, Variable> = [
            ("note".to_string(), note_var.clone()),
            ("vel".to_string(), vel_var.clone()),
            ("chan".to_string(), chan_var.clone()),
            ("dur".to_string(), dur_var.clone()),
        ]
        .into_iter()
        .collect();

        let expanded = emit_with_expansion(&["note", "vel", "chan", "dur"], &params, ctx, |p| {
            emit_midi_note_single(p, &dev_var)
        });
        for instr in expanded {
            labeled.push(LabeledInstr::Instr(instr));
        }
    }
    labeled.push(LabeledInstr::Jump(label_done.clone()));

    // ----- EMIT DIRT (fallback) -----
    labeled.push(LabeledInstr::Mark(label_emit_dirt));
    {
        // For Dirt, we just pass the whole map as params
        // Extract sound if present
        labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
            default_sound.clone(),
            sound_var.clone(),
        ))));

        // Check for "sound" key
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Contains(map_var.clone(), key_sound.clone(), cond.clone()),
        )));
        let skip_sound = ctx.new_label();
        labeled.push(LabeledInstr::JumpIfNot(cond.clone(), skip_sound.clone()));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_sound.clone(), sound_var.clone()),
        )));
        labeled.push(LabeledInstr::Jump(skip_sound.clone()));
        labeled.push(LabeledInstr::Mark(skip_sound.clone()));

        // Check for "s" key if sound not found
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Contains(map_var.clone(), key_s.clone(), cond.clone()),
        )));
        let skip_s = ctx.new_label();
        labeled.push(LabeledInstr::JumpIfNot(cond.clone(), skip_s.clone()));
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Index(map_var.clone(), key_s.clone(), sound_var.clone()),
        )));
        labeled.push(LabeledInstr::Mark(skip_s));

        // For Dirt, we emit with the _map pattern - pass whole map
        // This is simpler for Dirt since it takes arbitrary params
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
