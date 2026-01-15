//! Expression compilation for the Bob language.
//!
//! This module contains the main expression compiler and all related helper functions.
//! These functions have circular dependencies and must stay together in one module.

use crate::bali::bali_ast::constants::NOTE_MAP;
use crate::bob::bob_ast::{BobExpr, BobValue};
use crate::bob::context::{CompileContext, Label, LabeledInstr, resolve_labels};
use crate::bob::emit::emit_as_asm;
use crate::bob::emit_runtime::emit_map_var_as_asm;
use crate::bob::operators::find_operator;
use sova_core::vm::control_asm::ControlASM;
use sova_core::vm::event::Event;
use sova_core::vm::variable::{Variable, VariableValue};
use sova_core::vm::{EnvironmentFunc, Instruction};

const BREAK_EXIT_JUMP: usize = usize::MAX;

// ============================================================================
// Expression Compilation
// ============================================================================

pub(crate) fn may_contain_call(expr: &BobExpr) -> bool {
    match expr {
        BobExpr::Value(_) | BobExpr::Unit | BobExpr::Break | BobExpr::MapNew => false,
        BobExpr::FunctionCall(_, _) => true,
        BobExpr::Call(_, args) => args.iter().any(may_contain_call),
        BobExpr::Seq(a, b) => may_contain_call(a) || may_contain_call(b),
        BobExpr::Assign(_, e) => may_contain_call(e),
        BobExpr::List(elems) => elems.iter().any(may_contain_call),
        BobExpr::MapLiteral(pairs) => pairs.iter().any(|(_, v)| may_contain_call(v)),
        BobExpr::MapGet(a, b)
        | BobExpr::MapHas(a, b)
        | BobExpr::MapMerge(a, b)
        | BobExpr::Map(a, b)
        | BobExpr::Filter(a, b) => may_contain_call(a) || may_contain_call(b),
        BobExpr::MapLen(a) => may_contain_call(a),
        BobExpr::MapSet(a, b, c) | BobExpr::Reduce(a, b, c) | BobExpr::Ternary(a, b, c) => {
            may_contain_call(a) || may_contain_call(b) || may_contain_call(c)
        }
        BobExpr::Choose(opts) | BobExpr::Alt(opts) | BobExpr::Bytes(opts) => {
            opts.iter().any(may_contain_call)
        }
        BobExpr::If {
            condition,
            then_expr,
            else_expr,
        } => {
            may_contain_call(condition)
                || may_contain_call(then_expr)
                || may_contain_call(else_expr)
        }
        BobExpr::Switch {
            value,
            cases,
            default,
        } => {
            may_contain_call(value)
                || cases
                    .iter()
                    .any(|(a, b)| may_contain_call(a) || may_contain_call(b))
                || may_contain_call(default)
        }
        BobExpr::Prob {
            threshold,
            then_expr,
            else_expr,
        } => {
            may_contain_call(threshold)
                || may_contain_call(then_expr)
                || may_contain_call(else_expr)
        }
        BobExpr::Loop {
            start,
            end,
            step,
            body,
        } => {
            may_contain_call(start)
                || may_contain_call(end)
                || may_contain_call(step)
                || may_contain_call(body)
        }
        BobExpr::While { condition, body }
        | BobExpr::Every {
            period: condition,
            body,
        } => may_contain_call(condition) || may_contain_call(body),
        BobExpr::Do { count, body } | BobExpr::ForEach { list: count, body } => {
            may_contain_call(count) || may_contain_call(body)
        }
        BobExpr::Lambda { body, .. } | BobExpr::FunctionDef { body, .. } => may_contain_call(body),
        BobExpr::Emit(e) | BobExpr::Wait(e) | BobExpr::Dev(e) => may_contain_call(e),
        BobExpr::Fork { body } => may_contain_call(body),
        BobExpr::Euclidean {
            hits,
            steps,
            dur,
            hit_body,
            miss_body,
        } => {
            may_contain_call(hits)
                || may_contain_call(steps)
                || may_contain_call(dur)
                || may_contain_call(hit_body)
                || may_contain_call(miss_body)
        }
        BobExpr::Binary {
            pattern,
            dur,
            hit_body,
            miss_body,
        } => {
            may_contain_call(pattern)
                || may_contain_call(dur)
                || may_contain_call(hit_body)
                || may_contain_call(miss_body)
        }
    }
}

pub(crate) fn compile_expr(
    expr: &BobExpr,
    dest: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    match expr {
        BobExpr::Unit => vec![Instruction::Control(ControlASM::Mov(
            Variable::Constant(VariableValue::Integer(0)),
            dest.clone(),
        ))],

        BobExpr::Value(v) => vec![Instruction::Control(ControlASM::Mov(
            bob_value_to_variable(v),
            dest.clone(),
        ))],

        BobExpr::Seq(left, right) => {
            let discard = ctx.temp("_bob_discard");
            let mut instrs = compile_expr(left, &discard, ctx);
            instrs.extend(compile_expr(right, dest, ctx));
            instrs
        }

        BobExpr::Assign(target, value_expr) => {
            let target_var = bob_value_to_variable(target);
            let mut instrs = compile_expr(value_expr, &target_var, ctx);
            if &target_var != dest {
                instrs.push(Instruction::Control(ControlASM::Mov(
                    target_var,
                    dest.clone(),
                )));
            }
            instrs
        }

        BobExpr::Call(name, args) => compile_call(name, args, dest, ctx),
        BobExpr::FunctionCall(name, args) => compile_function_call(name, args, dest, ctx),

        BobExpr::MapNew => vec![Instruction::Control(ControlASM::MapEmpty(dest.clone()))],

        BobExpr::MapLiteral(pairs) => {
            let mut instrs = vec![Instruction::Control(ControlASM::MapEmpty(dest.clone()))];
            for (key, val_expr) in pairs {
                let val_temp = ctx.temp("_bob_map_v");
                instrs.extend(compile_expr(val_expr, &val_temp, ctx));
                instrs.push(Instruction::Control(ControlASM::MapInsert(
                    dest.clone(),
                    Variable::Constant(VariableValue::Str(key.clone())),
                    val_temp,
                    dest.clone(),
                )));
            }
            instrs
        }

        BobExpr::MapGet(map_expr, key_expr) => {
            let map_temp = ctx.temp("_bob_map");
            let key_temp = ctx.temp("_bob_key");
            let mut instrs = compile_expr(map_expr, &map_temp, ctx);
            instrs.extend(compile_expr(key_expr, &key_temp, ctx));
            instrs.push(Instruction::Control(ControlASM::MapGet(
                map_temp,
                key_temp,
                dest.clone(),
            )));
            instrs
        }

        BobExpr::MapHas(map_expr, key_expr) => {
            let map_temp = ctx.temp("_bob_map");
            let key_temp = ctx.temp("_bob_key");
            let mut instrs = compile_expr(map_expr, &map_temp, ctx);
            instrs.extend(compile_expr(key_expr, &key_temp, ctx));
            instrs.push(Instruction::Control(ControlASM::MapHas(
                map_temp,
                key_temp,
                dest.clone(),
            )));
            instrs
        }

        BobExpr::MapSet(map_expr, key_expr, val_expr) => {
            let map_temp = ctx.temp("_bob_map");
            let key_temp = ctx.temp("_bob_key");
            let val_temp = ctx.temp("_bob_val");
            let mut instrs = compile_expr(map_expr, &map_temp, ctx);
            instrs.extend(compile_expr(key_expr, &key_temp, ctx));
            instrs.extend(compile_expr(val_expr, &val_temp, ctx));
            instrs.push(Instruction::Control(ControlASM::MapInsert(
                map_temp.clone(),
                key_temp,
                val_temp,
                map_temp.clone(),
            )));
            instrs.push(Instruction::Control(ControlASM::Mov(
                map_temp,
                dest.clone(),
            )));
            instrs
        }

        BobExpr::MapMerge(a_expr, b_expr) => {
            let a_temp = ctx.temp("_bob_map_a");
            let b_temp = ctx.temp("_bob_map_b");
            let mut instrs = compile_expr(a_expr, &a_temp, ctx);
            instrs.extend(compile_expr(b_expr, &b_temp, ctx));
            // BitOr with b first means b's values win on conflict
            instrs.push(Instruction::Control(ControlASM::BitOr(
                b_temp,
                a_temp,
                dest.clone(),
            )));
            instrs
        }

        BobExpr::MapLen(map_expr) => {
            let map_temp = ctx.temp("_bob_map");
            let mut instrs = compile_expr(map_expr, &map_temp, ctx);
            instrs.push(Instruction::Control(ControlASM::MapLen(
                map_temp,
                dest.clone(),
            )));
            instrs
        }

        BobExpr::List(elems) => {
            let mut instrs = Vec::new();
            // Start with empty vector
            instrs.push(Instruction::Control(ControlASM::Mov(
                Variable::Constant(VariableValue::Vec(vec![])),
                dest.clone(),
            )));
            // Compile and push each element
            for elem in elems {
                let temp = ctx.temp("_bob_list");
                instrs.extend(compile_expr(elem, &temp, ctx));
                instrs.push(Instruction::Control(ControlASM::VecPush(
                    dest.clone(),
                    temp,
                    dest.clone(),
                )));
            }
            instrs
        }

        BobExpr::Map(fn_expr, list_expr) => compile_list_map(fn_expr, list_expr, dest, ctx),
        BobExpr::Filter(fn_expr, list_expr) => compile_filter(fn_expr, list_expr, dest, ctx),
        BobExpr::Reduce(fn_expr, init_expr, list_expr) => {
            compile_reduce(fn_expr, init_expr, list_expr, dest, ctx)
        }

        BobExpr::Choose(options) => compile_choose(options, dest, ctx),
        BobExpr::Alt(options) => compile_alt(options, dest, ctx),
        BobExpr::Bytes(_) => vec![],

        BobExpr::Ternary(cond, then_expr, else_expr) => {
            let cond_var = ctx.temp("_bob_tern_c");
            let else_label = ctx.new_label();
            let end_label = ctx.new_label();

            let mut labeled: Vec<LabeledInstr> = Vec::new();
            for instr in compile_expr(cond, &cond_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::JumpIfNot(cond_var, else_label.clone()));
            for instr in compile_expr(then_expr, dest, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Jump(end_label.clone()));
            labeled.push(LabeledInstr::Mark(else_label));
            for instr in compile_expr(else_expr, dest, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Mark(end_label));
            resolve_labels(labeled)
        }

        BobExpr::If {
            condition,
            then_expr,
            else_expr,
        } => {
            let cond_var = ctx.temp("_bob_if_c");
            let else_label = ctx.new_label();
            let end_label = ctx.new_label();

            let mut labeled: Vec<LabeledInstr> = Vec::new();
            for instr in compile_expr(condition, &cond_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::JumpIfNot(cond_var, else_label.clone()));
            for instr in compile_expr(then_expr, dest, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Jump(end_label.clone()));
            labeled.push(LabeledInstr::Mark(else_label));
            for instr in compile_expr(else_expr, dest, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Mark(end_label));
            resolve_labels(labeled)
        }

        BobExpr::Switch {
            value,
            cases,
            default,
        } => {
            let val_var = ctx.temp("_bob_switch_val");
            let case_var = ctx.temp("_bob_switch_case");
            let cond_var = ctx.temp("_bob_switch_cond");
            let end_label = ctx.new_label();
            let default_label = ctx.new_label();

            let mut labeled: Vec<LabeledInstr> = Vec::new();
            for instr in compile_expr(value, &val_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }

            // Create labels for each case body
            let case_body_labels: Vec<Label> = cases.iter().map(|_| ctx.new_label()).collect();

            // Check phase: compare value against each case, jump to body if match
            for (i, (case_val, _)) in cases.iter().enumerate() {
                let check_next_label = ctx.new_label();
                for instr in compile_expr(case_val, &case_var, ctx) {
                    labeled.push(LabeledInstr::Instr(instr));
                }
                labeled.push(LabeledInstr::Instr(Instruction::Control(
                    ControlASM::Equal(val_var.clone(), case_var.clone(), cond_var.clone()),
                )));
                // If match, jump to body; else continue checking
                labeled.push(LabeledInstr::JumpIf(
                    cond_var.clone(),
                    case_body_labels[i].clone(),
                ));
                labeled.push(LabeledInstr::Mark(check_next_label));
            }
            // No case matched, go to default
            labeled.push(LabeledInstr::Jump(default_label.clone()));

            // Body phase: each case body with jump to end
            for (i, (_, case_result)) in cases.iter().enumerate() {
                labeled.push(LabeledInstr::Mark(case_body_labels[i].clone()));
                for instr in compile_expr(case_result, dest, ctx) {
                    labeled.push(LabeledInstr::Instr(instr));
                }
                labeled.push(LabeledInstr::Jump(end_label.clone()));
            }

            labeled.push(LabeledInstr::Mark(default_label));
            for instr in compile_expr(default, dest, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Mark(end_label));
            resolve_labels(labeled)
        }

        BobExpr::Prob {
            threshold,
            then_expr,
            else_expr,
        } => {
            let thresh_var = ctx.temp("_bob_prob_thresh");
            let rand_var = ctx.temp("_bob_prob_rand");
            let cond_var = ctx.temp("_bob_prob_cond");
            let else_label = ctx.new_label();
            let end_label = ctx.new_label();

            let mut labeled: Vec<LabeledInstr> = Vec::new();
            for instr in compile_expr(threshold, &thresh_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                Variable::Environment(EnvironmentFunc::RandomUInt(100)),
                rand_var.clone(),
            ))));
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::LowerThan(rand_var, thresh_var, cond_var.clone()),
            )));
            labeled.push(LabeledInstr::JumpIfNot(cond_var, else_label.clone()));
            for instr in compile_expr(then_expr, dest, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Jump(end_label.clone()));
            labeled.push(LabeledInstr::Mark(else_label));
            for instr in compile_expr(else_expr, dest, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Mark(end_label));
            resolve_labels(labeled)
        }

        BobExpr::Loop {
            start,
            end,
            step,
            body,
        } => {
            let i_var = Variable::Instance("I".to_string());
            let end_var = ctx.temp("_bob_loop_end");
            let step_var = ctx.temp("_bob_loop_step");
            let cond_var = ctx.temp("_bob_loop_cond");
            let result_var = ctx.temp("_bob_loop_result");
            let elem_var = ctx.temp("_bob_loop_elem");
            let loop_cond = ctx.new_label();
            let loop_end = ctx.new_label();

            let mut labeled: Vec<LabeledInstr> = Vec::new();
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                Variable::Constant(VariableValue::Vec(vec![])),
                result_var.clone(),
            ))));
            for instr in compile_expr(start, &i_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            for instr in compile_expr(end, &end_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            for instr in compile_expr(step, &step_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Mark(loop_cond.clone()));
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::GreaterThan(i_var.clone(), end_var, cond_var.clone()),
            )));
            labeled.push(LabeledInstr::JumpIf(cond_var, loop_end.clone()));
            for instr in compile_expr(body, &elem_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::VecPush(result_var.clone(), elem_var, result_var.clone()),
            )));
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
                i_var.clone(),
                step_var,
                i_var,
            ))));
            labeled.push(LabeledInstr::Jump(loop_cond));
            labeled.push(LabeledInstr::Mark(loop_end));
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                result_var,
                dest.clone(),
            ))));
            resolve_labels(labeled)
        }

        BobExpr::While { condition, body } => {
            let cond_var = ctx.temp("_bob_cond");
            let result_var = ctx.temp("_bob_while_result");
            let loop_start = ctx.new_label();
            let loop_end = ctx.new_label();

            let mut labeled: Vec<LabeledInstr> = Vec::new();
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                Variable::Constant(VariableValue::Integer(0)),
                result_var.clone(),
            ))));
            labeled.push(LabeledInstr::Mark(loop_start.clone()));
            for instr in compile_expr(condition, &cond_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::JumpIfNot(cond_var, loop_end.clone()));
            for instr in compile_expr(body, &result_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Jump(loop_start));
            labeled.push(LabeledInstr::Mark(loop_end));
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                result_var,
                dest.clone(),
            ))));
            resolve_labels(labeled)
        }

        BobExpr::Do { count, body } => {
            let counter_var = ctx.temp("_bob_do_i");
            let limit_var = ctx.temp("_bob_do_n");
            let cond_var = ctx.temp("_bob_do_cond");
            let result_var = ctx.temp("_bob_do_result");
            let zero = Variable::Constant(VariableValue::Integer(0));
            let one = Variable::Constant(VariableValue::Integer(1));
            let loop_cond = ctx.new_label();
            let loop_end = ctx.new_label();

            let mut labeled: Vec<LabeledInstr> = Vec::new();
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                zero.clone(),
                counter_var.clone(),
            ))));
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                zero,
                result_var.clone(),
            ))));
            for instr in compile_expr(count, &limit_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Mark(loop_cond.clone()));
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::GreaterOrEqual(counter_var.clone(), limit_var, cond_var.clone()),
            )));
            labeled.push(LabeledInstr::JumpIf(cond_var, loop_end.clone()));
            for instr in compile_expr(body, &result_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
                counter_var.clone(),
                one,
                counter_var,
            ))));
            labeled.push(LabeledInstr::Jump(loop_cond));
            labeled.push(LabeledInstr::Mark(loop_end));
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                result_var,
                dest.clone(),
            ))));
            resolve_labels(labeled)
        }

        BobExpr::ForEach { list, body } => {
            let list_var = ctx.temp("_bob_foreach_list");
            let len_var = ctx.temp("_bob_foreach_len");
            let i_var = Variable::Instance("I".to_string());
            let e_var = Variable::Instance("E".to_string());
            let cond_var = ctx.temp("_bob_foreach_cond");
            let result_var = ctx.temp("_bob_foreach_result");
            let elem_var = ctx.temp("_bob_foreach_elem");
            let zero = Variable::Constant(VariableValue::Integer(0));
            let one = Variable::Constant(VariableValue::Integer(1));
            let loop_cond = ctx.new_label();
            let loop_end = ctx.new_label();

            let mut labeled: Vec<LabeledInstr> = Vec::new();
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                Variable::Constant(VariableValue::Vec(vec![])),
                result_var.clone(),
            ))));
            for instr in compile_expr(list, &list_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::VecLen(list_var.clone(), len_var.clone()),
            )));
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                zero,
                i_var.clone(),
            ))));
            labeled.push(LabeledInstr::Mark(loop_cond.clone()));
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::GreaterOrEqual(i_var.clone(), len_var, cond_var.clone()),
            )));
            labeled.push(LabeledInstr::JumpIf(cond_var, loop_end.clone()));
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::VecGet(list_var, i_var.clone(), e_var),
            )));
            for instr in compile_expr(body, &elem_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::VecPush(result_var.clone(), elem_var, result_var.clone()),
            )));
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
                i_var.clone(),
                one,
                i_var,
            ))));
            labeled.push(LabeledInstr::Jump(loop_cond));
            labeled.push(LabeledInstr::Mark(loop_end));
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                result_var,
                dest.clone(),
            ))));
            resolve_labels(labeled)
        }

        BobExpr::Every { period, body } => {
            let counter_var = ctx.line_temp("_bob_every");
            let period_var = ctx.temp("_bob_every_n");
            let mod_var = ctx.temp("_bob_every_mod");
            let cond_var = ctx.temp("_bob_every_cond");
            let one = Variable::Constant(VariableValue::Integer(1));
            let zero = Variable::Constant(VariableValue::Integer(0));
            let end_label = ctx.new_label();

            let mut labeled: Vec<LabeledInstr> = Vec::new();
            for instr in compile_expr(period, &period_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mod(
                counter_var.clone(),
                period_var.clone(),
                mod_var.clone(),
            ))));
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::Equal(mod_var, zero.clone(), cond_var.clone()),
            )));
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
                counter_var.clone(),
                one,
                counter_var.clone(),
            ))));
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mod(
                counter_var.clone(),
                period_var,
                counter_var,
            ))));
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                zero,
                dest.clone(),
            ))));
            labeled.push(LabeledInstr::JumpIfNot(cond_var, end_label.clone()));
            for instr in compile_expr(body, dest, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Mark(end_label));
            resolve_labels(labeled)
        }

        BobExpr::Lambda { args, body } => {
            let mut func_code: Vec<Instruction> = Vec::new();
            for arg in args.iter().rev() {
                func_code.push(Instruction::Control(ControlASM::Pop(Variable::Global(
                    arg.clone(),
                ))));
            }
            let mut func_ctx = CompileContext {
                functions: ctx.functions.clone(),
                default_dev: ctx.default_dev,
                temp_counter: ctx.temp_counter,
                label_counter: ctx.label_counter,
            };
            func_code.extend(compile_expr(body, &Variable::StackBack, &mut func_ctx));
            func_code.push(Instruction::Control(ControlASM::Return));
            ctx.temp_counter = func_ctx.temp_counter;
            vec![Instruction::Control(ControlASM::Mov(
                Variable::Constant(VariableValue::Func(func_code)),
                dest.clone(),
            ))]
        }

        BobExpr::FunctionDef { name, args, body } => {
            let mut func_code: Vec<Instruction> = Vec::new();
            for arg in args.iter().rev() {
                func_code.push(Instruction::Control(ControlASM::Pop(Variable::Global(
                    arg.clone(),
                ))));
            }
            let mut func_ctx = CompileContext {
                functions: ctx.functions.clone(),
                default_dev: ctx.default_dev,
                temp_counter: ctx.temp_counter,
                label_counter: ctx.label_counter,
            };
            func_code.extend(compile_expr(body, &Variable::StackBack, &mut func_ctx));
            func_code.push(Instruction::Control(ControlASM::Return));
            ctx.temp_counter = func_ctx.temp_counter;
            let func_var = Variable::Instance(format!("_func_{name}"));
            let mut instrs = vec![Instruction::Control(ControlASM::Mov(
                Variable::Constant(VariableValue::Func(func_code)),
                func_var,
            ))];
            instrs.push(Instruction::Control(ControlASM::Mov(
                Variable::Constant(VariableValue::Integer(0)),
                dest.clone(),
            )));
            instrs
        }

        BobExpr::Emit(inner) => {
            match inner.as_ref() {
                // Map literals: keys known at compile time, use smart dispatch
                BobExpr::MapLiteral(pairs) => {
                    let converted: Vec<(String, BobExpr)> = pairs
                        .iter()
                        .map(|(k, v)| (k.clone(), (**v).clone()))
                        .collect();
                    emit_as_asm(&converted, ctx.default_dev, ctx)
                }
                // Variables/expressions: emit as generic Dirt with _map param
                _ => {
                    let mut instrs = compile_expr(inner, dest, ctx);
                    instrs.extend(emit_map_var_as_asm(dest, ctx.default_dev, ctx));
                    instrs
                }
            }
        }

        BobExpr::Wait(dur) => {
            let wait_var = ctx.temp("_bob_wait");
            let frames_var = ctx.temp("_bob_frames");
            let mut instrs = compile_expr(dur, &wait_var, ctx);
            instrs.push(Instruction::Control(ControlASM::FloatAsFrames(
                wait_var,
                frames_var.clone(),
            )));
            instrs.push(Instruction::Effect(Event::Nop, frames_var));
            instrs.push(Instruction::Control(ControlASM::Mov(
                Variable::Constant(VariableValue::Integer(0)),
                dest.clone(),
            )));
            instrs
        }

        BobExpr::Dev(id) => {
            if let BobExpr::Value(BobValue::Int(i)) = id.as_ref() {
                ctx.default_dev = *i;
            }
            vec![Instruction::Control(ControlASM::Mov(
                Variable::Constant(VariableValue::Integer(0)),
                dest.clone(),
            ))]
        }

        BobExpr::Break => vec![Instruction::Control(ControlASM::Jump(BREAK_EXIT_JUMP))],

        BobExpr::Fork { body } => {
            let mut instrs = Vec::new();
            let time_var = ctx.temp("_bob_fork_time");

            // Time = 0 for immediate spawn
            instrs.push(Instruction::Control(ControlASM::FloatAsFrames(
                Variable::Constant(VariableValue::Float(0.0)),
                time_var.clone(),
            )));

            // Compile body as a separate program
            let branch_prog = compile_fork_branch(body, ctx);

            // Store program in variable
            let branch_var = ctx.temp("_bob_fork_branch");
            instrs.push(Instruction::Control(ControlASM::Mov(
                Variable::Constant(VariableValue::Func(branch_prog)),
                branch_var.clone(),
            )));

            // Emit StartProgram event to spawn the branch
            instrs.push(Instruction::Effect(
                Event::StartProgram(branch_var),
                time_var.clone(),
            ));

            // FORK returns 0 immediately
            instrs.push(Instruction::Control(ControlASM::Mov(
                Variable::Constant(VariableValue::Integer(0)),
                dest.clone(),
            )));
            instrs
        }

        BobExpr::Euclidean {
            hits,
            steps,
            dur,
            hit_body,
            miss_body,
        } => {
            // EU hits steps dur : hit_body ELSE : miss_body END
            //
            // Compiles to:
            //   _hits = <hits>
            //   _steps = <steps>
            //   _dur = <dur>
            //   I = 0
            //   loop_start:
            //     if I >= _steps: goto loop_end
            //     _product = I * _hits
            //     _mod = _product % _steps
            //     if _mod < _hits: goto hit_branch
            //     <miss_body>
            //     goto after_branch
            //   hit_branch:
            //     <hit_body>
            //   after_branch:
            //     WAIT _dur
            //     I = I + 1
            //     goto loop_start
            //   loop_end:

            let hits_var = ctx.temp("_bob_eu_hits");
            let steps_var = ctx.temp("_bob_eu_steps");
            let dur_var = ctx.temp("_bob_eu_dur");
            let i_var = Variable::Instance("I".to_string());
            let cond_var = ctx.temp("_bob_eu_cond");
            let product_var = ctx.temp("_bob_eu_prod");
            let mod_var = ctx.temp("_bob_eu_mod");
            let hit_cond_var = ctx.temp("_bob_eu_hit");
            let frames_var = ctx.temp("_bob_eu_frames");
            let body_result = ctx.temp("_bob_eu_body");

            let zero = Variable::Constant(VariableValue::Integer(0));
            let one = Variable::Constant(VariableValue::Integer(1));

            let loop_start = ctx.new_label();
            let loop_end = ctx.new_label();
            let hit_branch = ctx.new_label();
            let after_branch = ctx.new_label();

            let mut labeled: Vec<LabeledInstr> = Vec::new();

            // Evaluate hits, steps, dur once at start
            for instr in compile_expr(hits, &hits_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            for instr in compile_expr(steps, &steps_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            for instr in compile_expr(dur, &dur_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }

            // I = 0
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                zero.clone(),
                i_var.clone(),
            ))));

            // loop_start:
            labeled.push(LabeledInstr::Mark(loop_start.clone()));

            // if I >= _steps: goto loop_end
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::GreaterOrEqual(i_var.clone(), steps_var.clone(), cond_var.clone()),
            )));
            labeled.push(LabeledInstr::JumpIf(cond_var.clone(), loop_end.clone()));

            // Euclidean hit check: (I * hits) % steps < hits
            // _product = I * _hits
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mul(
                i_var.clone(),
                hits_var.clone(),
                product_var.clone(),
            ))));
            // _mod = _product % _steps
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mod(
                product_var,
                steps_var.clone(),
                mod_var.clone(),
            ))));
            // _hit_cond = _mod < _hits
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::LowerThan(mod_var, hits_var, hit_cond_var.clone()),
            )));
            // if hit: goto hit_branch
            labeled.push(LabeledInstr::JumpIf(hit_cond_var, hit_branch.clone()));

            // miss_body (or skip if Unit)
            for instr in compile_expr(miss_body, &body_result, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Jump(after_branch.clone()));

            // hit_branch:
            labeled.push(LabeledInstr::Mark(hit_branch));
            for instr in compile_expr(hit_body, &body_result, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }

            // after_branch:
            labeled.push(LabeledInstr::Mark(after_branch));

            // WAIT _dur
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::FloatAsFrames(dur_var, frames_var.clone()),
            )));
            labeled.push(LabeledInstr::Instr(Instruction::Effect(
                Event::Nop,
                frames_var,
            )));

            // I = I + 1
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
                i_var.clone(),
                one,
                i_var,
            ))));

            // goto loop_start
            labeled.push(LabeledInstr::Jump(loop_start));

            // loop_end:
            labeled.push(LabeledInstr::Mark(loop_end));

            // EU returns 0
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                zero,
                dest.clone(),
            ))));

            resolve_labels(labeled)
        }

        BobExpr::Binary {
            pattern,
            dur,
            hit_body,
            miss_body,
        } => {
            // BIN pattern dur : hit_body ELSE : miss_body END
            //
            // Compiles to:
            //   _pattern = <pattern>
            //   _dur = <dur>
            //   if _pattern == 0: goto loop_end  (0 steps)
            //   _steps = 64 - leading_zeros(_pattern)
            //   I = 0
            //   loop_start:
            //     if I >= _steps: goto loop_end
            //     _shift = _steps - 1 - I
            //     _bit = (_pattern >> _shift) & 1
            //     if _bit != 0: goto hit_branch
            //     <miss_body>
            //     goto after_branch
            //   hit_branch:
            //     <hit_body>
            //   after_branch:
            //     WAIT _dur
            //     I = I + 1
            //     goto loop_start
            //   loop_end:

            let pattern_var = ctx.temp("_bob_bin_pat");
            let dur_var = ctx.temp("_bob_bin_dur");
            let steps_var = ctx.temp("_bob_bin_steps");
            let i_var = Variable::Instance("I".to_string());
            let cond_var = ctx.temp("_bob_bin_cond");
            let shift_var = ctx.temp("_bob_bin_shift");
            let bit_var = ctx.temp("_bob_bin_bit");
            let frames_var = ctx.temp("_bob_bin_frames");
            let body_result = ctx.temp("_bob_bin_body");
            let lz_var = ctx.temp("_bob_bin_lz");
            let temp_var = ctx.temp("_bob_bin_tmp");

            let zero = Variable::Constant(VariableValue::Integer(0));
            let one = Variable::Constant(VariableValue::Integer(1));
            let sixty_four = Variable::Constant(VariableValue::Integer(64));

            let loop_start = ctx.new_label();
            let loop_end = ctx.new_label();
            let hit_branch = ctx.new_label();
            let after_branch = ctx.new_label();

            let mut labeled: Vec<LabeledInstr> = Vec::new();

            // Evaluate pattern and dur once at start
            for instr in compile_expr(pattern, &pattern_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            for instr in compile_expr(dur, &dur_var, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }

            // if pattern == 0: goto loop_end (0 steps case)
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::Equal(pattern_var.clone(), zero.clone(), cond_var.clone()),
            )));
            labeled.push(LabeledInstr::JumpIf(cond_var.clone(), loop_end.clone()));

            // _steps = 64 - leading_zeros(_pattern)
            // Use LeadingZeros instruction
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::LeadingZeros(pattern_var.clone(), lz_var.clone()),
            )));
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Sub(
                sixty_four,
                lz_var,
                steps_var.clone(),
            ))));

            // I = 0
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                zero.clone(),
                i_var.clone(),
            ))));

            // loop_start:
            labeled.push(LabeledInstr::Mark(loop_start.clone()));

            // if I >= _steps: goto loop_end
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::GreaterOrEqual(i_var.clone(), steps_var.clone(), cond_var.clone()),
            )));
            labeled.push(LabeledInstr::JumpIf(cond_var.clone(), loop_end.clone()));

            // Binary hit check: (pattern >> (steps - 1 - I)) & 1
            // _shift = _steps - 1 - I = (_steps - 1) - I
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Sub(
                steps_var.clone(),
                one.clone(),
                temp_var.clone(),
            ))));
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Sub(
                temp_var,
                i_var.clone(),
                shift_var.clone(),
            ))));
            // _bit = (_pattern >> _shift) & 1
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::ShiftRightL(pattern_var, shift_var, bit_var.clone()),
            )));
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::BitAnd(bit_var.clone(), one.clone(), bit_var.clone()),
            )));
            // if _bit != 0: goto hit_branch
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::Different(bit_var, zero.clone(), cond_var.clone()),
            )));
            labeled.push(LabeledInstr::JumpIf(cond_var, hit_branch.clone()));

            // miss_body (or skip if Unit)
            for instr in compile_expr(miss_body, &body_result, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }
            labeled.push(LabeledInstr::Jump(after_branch.clone()));

            // hit_branch:
            labeled.push(LabeledInstr::Mark(hit_branch));
            for instr in compile_expr(hit_body, &body_result, ctx) {
                labeled.push(LabeledInstr::Instr(instr));
            }

            // after_branch:
            labeled.push(LabeledInstr::Mark(after_branch));

            // WAIT _dur
            labeled.push(LabeledInstr::Instr(Instruction::Control(
                ControlASM::FloatAsFrames(dur_var, frames_var.clone()),
            )));
            labeled.push(LabeledInstr::Instr(Instruction::Effect(
                Event::Nop,
                frames_var,
            )));

            // I = I + 1
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
                i_var.clone(),
                one,
                i_var,
            ))));

            // goto loop_start
            labeled.push(LabeledInstr::Jump(loop_start));

            // loop_end:
            labeled.push(LabeledInstr::Mark(loop_end));

            // BIN returns 0
            labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
                zero,
                dest.clone(),
            ))));

            resolve_labels(labeled)
        }
    }
}

fn compile_fork_branch(body: &BobExpr, ctx: &mut CompileContext) -> Vec<Instruction> {
    // Create a context for the branch that shares function definitions
    // but gets fresh temp/label counters to avoid collisions
    let mut branch_ctx = CompileContext {
        functions: ctx.functions.clone(),
        default_dev: ctx.default_dev,
        temp_counter: ctx.temp_counter,
        label_counter: ctx.label_counter,
    };

    let result_var = Variable::Instance("_bob_branch_result".to_string());
    let prog = compile_expr(body, &result_var, &mut branch_ctx);

    // Sync counters back to parent to avoid collisions with future branches
    ctx.temp_counter = branch_ctx.temp_counter;
    ctx.label_counter = branch_ctx.label_counter;

    prog
}

// ============================================================================
// List Operations
// ============================================================================

fn compile_list_map(
    fn_expr: &BobExpr,
    list_expr: &BobExpr,
    dest: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let fn_var = ctx.temp("_bob_map_fn");
    let list_var = ctx.temp("_bob_map_src");
    let result_var = ctx.temp("_bob_map_res");
    let len_var = ctx.temp("_bob_map_len");
    let idx_var = ctx.temp("_bob_map_idx");
    let elem_var = ctx.temp("_bob_map_elem");
    let mapped_var = ctx.temp("_bob_map_mapped");
    let cond_var = ctx.temp("_bob_map_cond");
    let zero = Variable::Constant(VariableValue::Integer(0));
    let one = Variable::Constant(VariableValue::Integer(1));
    let loop_label = ctx.new_label();
    let end_label = ctx.new_label();

    let mut labeled: Vec<LabeledInstr> = Vec::new();
    for instr in compile_expr(fn_expr, &fn_var, ctx) {
        labeled.push(LabeledInstr::Instr(instr));
    }
    for instr in compile_expr(list_expr, &list_var, ctx) {
        labeled.push(LabeledInstr::Instr(instr));
    }
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        Variable::Constant(VariableValue::Vec(vec![])),
        result_var.clone(),
    ))));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::VecLen(list_var.clone(), len_var.clone()),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        zero,
        idx_var.clone(),
    ))));
    labeled.push(LabeledInstr::Mark(loop_label.clone()));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::GreaterOrEqual(idx_var.clone(), len_var.clone(), cond_var.clone()),
    )));
    labeled.push(LabeledInstr::JumpIf(cond_var, end_label.clone()));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::VecGet(list_var, idx_var.clone(), elem_var.clone()),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Push(
        elem_var,
    ))));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::CallFunction(fn_var),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Pop(
        mapped_var.clone(),
    ))));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::VecPush(result_var.clone(), mapped_var, result_var.clone()),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
        idx_var.clone(),
        one,
        idx_var,
    ))));
    labeled.push(LabeledInstr::Jump(loop_label));
    labeled.push(LabeledInstr::Mark(end_label));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        result_var,
        dest.clone(),
    ))));
    resolve_labels(labeled)
}

fn compile_filter(
    fn_expr: &BobExpr,
    list_expr: &BobExpr,
    dest: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let fn_var = ctx.temp("_bob_filter_fn");
    let list_var = ctx.temp("_bob_filter_src");
    let result_var = ctx.temp("_bob_filter_res");
    let len_var = ctx.temp("_bob_filter_len");
    let idx_var = ctx.temp("_bob_filter_idx");
    let elem_var = ctx.temp("_bob_filter_elem");
    let pred_var = ctx.temp("_bob_filter_pred");
    let cond_var = ctx.temp("_bob_filter_cond");
    let zero = Variable::Constant(VariableValue::Integer(0));
    let one = Variable::Constant(VariableValue::Integer(1));
    let loop_label = ctx.new_label();
    let skip_label = ctx.new_label();
    let end_label = ctx.new_label();

    let mut labeled: Vec<LabeledInstr> = Vec::new();
    for instr in compile_expr(fn_expr, &fn_var, ctx) {
        labeled.push(LabeledInstr::Instr(instr));
    }
    for instr in compile_expr(list_expr, &list_var, ctx) {
        labeled.push(LabeledInstr::Instr(instr));
    }
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        Variable::Constant(VariableValue::Vec(vec![])),
        result_var.clone(),
    ))));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::VecLen(list_var.clone(), len_var.clone()),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        zero,
        idx_var.clone(),
    ))));
    labeled.push(LabeledInstr::Mark(loop_label.clone()));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::GreaterOrEqual(idx_var.clone(), len_var.clone(), cond_var.clone()),
    )));
    labeled.push(LabeledInstr::JumpIf(cond_var, end_label.clone()));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::VecGet(list_var, idx_var.clone(), elem_var.clone()),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Push(
        elem_var.clone(),
    ))));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::CallFunction(fn_var),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Pop(
        pred_var.clone(),
    ))));
    labeled.push(LabeledInstr::JumpIfNot(pred_var, skip_label.clone()));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::VecPush(result_var.clone(), elem_var, result_var.clone()),
    )));
    labeled.push(LabeledInstr::Mark(skip_label));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
        idx_var.clone(),
        one,
        idx_var,
    ))));
    labeled.push(LabeledInstr::Jump(loop_label));
    labeled.push(LabeledInstr::Mark(end_label));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        result_var,
        dest.clone(),
    ))));
    resolve_labels(labeled)
}

fn compile_reduce(
    fn_expr: &BobExpr,
    init_expr: &BobExpr,
    list_expr: &BobExpr,
    dest: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let fn_var = ctx.temp("_bob_reduce_fn");
    let list_var = ctx.temp("_bob_reduce_src");
    let acc_var = ctx.temp("_bob_reduce_acc");
    let len_var = ctx.temp("_bob_reduce_len");
    let idx_var = ctx.temp("_bob_reduce_idx");
    let elem_var = ctx.temp("_bob_reduce_elem");
    let cond_var = ctx.temp("_bob_reduce_cond");
    let zero = Variable::Constant(VariableValue::Integer(0));
    let one = Variable::Constant(VariableValue::Integer(1));
    let loop_label = ctx.new_label();
    let end_label = ctx.new_label();

    let mut labeled: Vec<LabeledInstr> = Vec::new();
    for instr in compile_expr(fn_expr, &fn_var, ctx) {
        labeled.push(LabeledInstr::Instr(instr));
    }
    for instr in compile_expr(init_expr, &acc_var, ctx) {
        labeled.push(LabeledInstr::Instr(instr));
    }
    for instr in compile_expr(list_expr, &list_var, ctx) {
        labeled.push(LabeledInstr::Instr(instr));
    }
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::VecLen(list_var.clone(), len_var.clone()),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        zero,
        idx_var.clone(),
    ))));
    labeled.push(LabeledInstr::Mark(loop_label.clone()));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::GreaterOrEqual(idx_var.clone(), len_var.clone(), cond_var.clone()),
    )));
    labeled.push(LabeledInstr::JumpIf(cond_var, end_label.clone()));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::VecGet(list_var, idx_var.clone(), elem_var.clone()),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Push(
        acc_var.clone(),
    ))));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Push(
        elem_var,
    ))));
    labeled.push(LabeledInstr::Instr(Instruction::Control(
        ControlASM::CallFunction(fn_var),
    )));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Pop(
        acc_var.clone(),
    ))));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
        idx_var.clone(),
        one,
        idx_var,
    ))));
    labeled.push(LabeledInstr::Jump(loop_label));
    labeled.push(LabeledInstr::Mark(end_label));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        acc_var,
        dest.clone(),
    ))));
    resolve_labels(labeled)
}

// ============================================================================
// Selection Operations
// ============================================================================

fn compile_choose(
    options: &[BobExpr],
    dest: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    if options.len() == 1 {
        return compile_expr(&options[0], dest, ctx);
    }
    let n = options.len();
    let rand_var = ctx.temp("_bob_choose_rand");
    let cond_var = ctx.temp("_bob_choose_cond");
    let end_label = ctx.new_label();
    let option_labels: Vec<Label> = (0..n).map(|_| ctx.new_label()).collect();

    let mut labeled: Vec<LabeledInstr> = Vec::new();
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        Variable::Environment(EnvironmentFunc::RandomUInt(n as u64)),
        rand_var.clone(),
    ))));
    for i in 0..n {
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Equal(
                rand_var.clone(),
                Variable::Constant(VariableValue::Integer(i as i64)),
                cond_var.clone(),
            ),
        )));
        labeled.push(LabeledInstr::JumpIf(
            cond_var.clone(),
            option_labels[i].clone(),
        ));
    }
    for (i, opt) in options.iter().enumerate() {
        labeled.push(LabeledInstr::Mark(option_labels[i].clone()));
        for instr in compile_expr(opt, dest, ctx) {
            labeled.push(LabeledInstr::Instr(instr));
        }
        if i < n - 1 {
            labeled.push(LabeledInstr::Jump(end_label.clone()));
        }
    }
    labeled.push(LabeledInstr::Mark(end_label));
    resolve_labels(labeled)
}

fn compile_alt(options: &[BobExpr], dest: &Variable, ctx: &mut CompileContext) -> Vec<Instruction> {
    if options.len() == 1 {
        return compile_expr(&options[0], dest, ctx);
    }
    let n = options.len();
    let counter_var = ctx.line_temp("_bob_alt");
    let cond_var = ctx.temp("_bob_alt_cond");
    let temp_counter = ctx.temp("_bob_alt_tmp");
    let increment_label = ctx.new_label();
    let option_labels: Vec<Label> = (0..n).map(|_| ctx.new_label()).collect();

    let mut labeled: Vec<LabeledInstr> = Vec::new();
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        counter_var.clone(),
        temp_counter.clone(),
    ))));
    for i in 0..n {
        labeled.push(LabeledInstr::Instr(Instruction::Control(
            ControlASM::Equal(
                temp_counter.clone(),
                Variable::Constant(VariableValue::Integer(i as i64)),
                cond_var.clone(),
            ),
        )));
        labeled.push(LabeledInstr::JumpIf(
            cond_var.clone(),
            option_labels[i].clone(),
        ));
    }
    for (i, opt) in options.iter().enumerate() {
        labeled.push(LabeledInstr::Mark(option_labels[i].clone()));
        for instr in compile_expr(opt, dest, ctx) {
            labeled.push(LabeledInstr::Instr(instr));
        }
        if i < n - 1 {
            labeled.push(LabeledInstr::Jump(increment_label.clone()));
        }
    }
    labeled.push(LabeledInstr::Mark(increment_label));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Add(
        temp_counter.clone(),
        Variable::Constant(VariableValue::Integer(1)),
        temp_counter.clone(),
    ))));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mod(
        temp_counter.clone(),
        Variable::Constant(VariableValue::Integer(n as i64)),
        temp_counter.clone(),
    ))));
    labeled.push(LabeledInstr::Instr(Instruction::Control(ControlASM::Mov(
        temp_counter,
        counter_var,
    ))));
    resolve_labels(labeled)
}

// ============================================================================
// Function Calls
// ============================================================================

fn compile_function_call(
    name: &str,
    args: &[BobExpr],
    dest: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let mut instrs = Vec::new();
    let arg_var_names: Vec<String> = ctx
        .functions
        .get(name)
        .map(|f| f.arg_names.clone())
        .unwrap_or_default();

    for arg_name in &arg_var_names {
        instrs.push(Instruction::Control(ControlASM::Push(Variable::Global(
            arg_name.clone(),
        ))));
    }

    let mut arg_temps: Vec<Variable> = Vec::new();
    for arg in args {
        let temp = ctx.temp("_bob_t");
        if may_contain_call(arg) && !arg_temps.is_empty() {
            for t in &arg_temps {
                instrs.push(Instruction::Control(ControlASM::Push(t.clone())));
            }
            instrs.extend(compile_expr(arg, &temp, ctx));
            for t in arg_temps.iter().rev() {
                instrs.push(Instruction::Control(ControlASM::Pop(t.clone())));
            }
        } else {
            instrs.extend(compile_expr(arg, &temp, ctx));
        }
        instrs.push(Instruction::Control(ControlASM::Push(temp.clone())));
        arg_temps.push(temp);
    }

    let func_var = if ctx.functions.contains_key(name) {
        Variable::Instance(format!("_func_{name}"))
    } else if name.len() == 1 && name.as_bytes()[0].is_ascii_uppercase() {
        Variable::Global(name.to_string())
    } else {
        Variable::Global(name.to_string())
    };
    instrs.push(Instruction::Control(ControlASM::CallFunction(func_var)));

    let need_restore = !arg_var_names.is_empty();
    let result_var = if need_restore && matches!(dest, Variable::StackBack) {
        ctx.temp("_bob_ret")
    } else {
        dest.clone()
    };
    instrs.push(Instruction::Control(ControlASM::Pop(result_var.clone())));

    for arg_name in arg_var_names.iter().rev() {
        instrs.push(Instruction::Control(ControlASM::Pop(Variable::Global(
            arg_name.clone(),
        ))));
    }

    if need_restore && matches!(dest, Variable::StackBack) {
        instrs.push(Instruction::Control(ControlASM::Mov(
            result_var,
            dest.clone(),
        )));
    }
    instrs
}

fn compile_call(
    name: &str,
    args: &[BobExpr],
    dest: &Variable,
    ctx: &mut CompileContext,
) -> Vec<Instruction> {
    let mut instrs = Vec::new();
    let mut temps: Vec<Variable> = Vec::new();

    for arg in args {
        let temp = ctx.temp("_bob_t");
        if may_contain_call(arg) && !temps.is_empty() {
            for t in &temps {
                instrs.push(Instruction::Control(ControlASM::Push(t.clone())));
            }
            instrs.extend(compile_expr(arg, &temp, ctx));
            for t in temps.iter().rev() {
                instrs.push(Instruction::Control(ControlASM::Pop(t.clone())));
            }
        } else {
            instrs.extend(compile_expr(arg, &temp, ctx));
        }
        temps.push(temp);
    }

    if name == "CYCLE" && temps.len() == 1 {
        let counter_var = ctx.line_temp("_bob_cycle");
        let len_var = ctx.temp("_bob_cycle_len");
        let idx_var = ctx.temp("_bob_cycle_idx");
        let cond_var = ctx.temp("_bob_cycle_cond");
        let zero = Variable::Constant(VariableValue::Integer(0));
        let one = Variable::Constant(VariableValue::Integer(1));
        // Get length
        instrs.push(Instruction::Control(ControlASM::VecLen(
            temps[0].clone(),
            len_var.clone(),
        )));
        // Check if len > 0
        instrs.push(Instruction::Control(ControlASM::GreaterThan(
            len_var.clone(),
            zero.clone(),
            cond_var.clone(),
        )));
        // Jump over cycle body (4 instructions: Mod, VecGet, Add, RelJump) to fallback if empty
        instrs.push(Instruction::Control(ControlASM::RelJumpIfNot(cond_var, 5)));
        // idx = counter % len
        instrs.push(Instruction::Control(ControlASM::Mod(
            counter_var.clone(),
            len_var,
            idx_var.clone(),
        )));
        // dest = vec[idx]
        instrs.push(Instruction::Control(ControlASM::VecGet(
            temps[0].clone(),
            idx_var,
            dest.clone(),
        )));
        // counter = counter + 1
        instrs.push(Instruction::Control(ControlASM::Add(
            counter_var.clone(),
            one,
            counter_var,
        )));
        // Jump over the fallback
        instrs.push(Instruction::Control(ControlASM::RelJump(2)));
        // Fallback: return 0
        instrs.push(Instruction::Control(ControlASM::Mov(zero, dest.clone())));
        return instrs;
    }

    if let Some(op) = find_operator(name, temps.len()) {
        instrs.extend((op.compile)(&temps, dest));
    } else {
        instrs.push(Instruction::Control(ControlASM::Mov(
            Variable::Constant(VariableValue::Integer(0)),
            dest.clone(),
        )));
    }
    instrs
}

// ============================================================================
// Value Conversion
// ============================================================================

pub(crate) fn bob_value_to_variable(value: &BobValue) -> Variable {
    match value {
        BobValue::Int(i) => Variable::Constant(VariableValue::Integer(*i)),
        BobValue::Float(f) => Variable::Constant(VariableValue::Float(*f)),
        BobValue::Str(s) => Variable::Constant(VariableValue::Str(s.clone())),
        BobValue::Symbol(s) => {
            if let Some(&midi_val) = NOTE_MAP.get(s) {
                Variable::Constant(VariableValue::Integer(midi_val))
            } else {
                Variable::Constant(VariableValue::Str(s.clone()))
            }
        }
        BobValue::GlobalVar(name) => Variable::Global(name.clone()),
        BobValue::FrameVar(name) => Variable::Frame(name.clone()),
        BobValue::LineVar(name) => Variable::Line(name.clone()),
        BobValue::InstanceVar(name) => Variable::Instance(name.clone()),
        BobValue::EnvTempo => Variable::Environment(EnvironmentFunc::GetTempo),
        BobValue::EnvRandom => Variable::Environment(EnvironmentFunc::RandomUInt(128)),
    }
}
