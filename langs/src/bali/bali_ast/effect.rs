use crate::bali::bali_ast::{
    bali_context::BaliContext,
    constants::{DEFAULT_DURATION, DEFAULT_VELOCITY},
    expression::Expression,
    fraction::Fraction,
    function::FunctionContent,
    value::Value,
};
use sova_core::vm::{Instruction, control_asm::ControlASM, event::Event, variable::Variable};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Effect {
    Definition(Value, Box<Expression>),
    Note(Box<Expression>, BaliContext),
    ProgramChange(Box<Expression>, BaliContext),
    ControlChange(Box<Expression>, Box<Expression>, BaliContext),
    Osc(Value, Vec<Expression>, BaliContext),
    Dirt(Value, Vec<(String, Box<Expression>)>, BaliContext),

    Aftertouch(Box<Expression>, Box<Expression>, BaliContext),
    ChannelPressure(Box<Expression>, BaliContext),
    Nop,
}

impl Effect {
    pub fn as_asm(
        &self,
        context: BaliContext,
        functions: &HashMap<String, FunctionContent>,
    ) -> Vec<Instruction> {
        //let time_var = Variable::Instance("_time".to_owned());
        let note_var = Variable::Instance("_note".to_owned());
        let velocity_var = Variable::Instance("_velocity".to_owned());
        let chan_var = Variable::Instance("_chan".to_owned());
        let duration_var = Variable::Instance("_duration".to_owned());
        let duration_time_var = Variable::Instance("_duration_time".to_owned());
        let program_var = Variable::Instance("_program".to_owned());
        let control_var = Variable::Instance("_control".to_owned());
        let value_var = Variable::Instance("_control_value".to_owned());
        let target_device_id_var = Variable::Instance("_target_device_id".to_string());

        let mut res = Vec::new();
        //let mut res = vec![Instruction::Control(ControlASM::FloatAsFrames(delay.into(), time_var.clone()))];

        match self {
            Effect::Nop => {}
            Effect::Definition(v, expr) => {
                res.extend(expr.as_asm(functions));
                if let Value::Variable(v) = v {
                    res.push(Instruction::Control(ControlASM::Pop(Value::as_variable(v))));
                }
            }
            Effect::Note(n, c) => {
                let context = c.update(&context);
                res.extend(n.as_asm(functions));
                res.push(Instruction::Control(ControlASM::Pop(note_var.clone())));

                res.extend(context.emit_velocity(&velocity_var, DEFAULT_VELOCITY, functions));
                res.extend(context.emit_channel(&chan_var, functions));

                if let Some(ref d) = context.duration {
                    res.extend(d.as_asm(functions));
                } else {
                    res.extend(
                        Fraction {
                            numerator: Box::new(Expression::Value(Value::Number(1))),
                            denominator: Box::new(Expression::Value(Value::Number(
                                DEFAULT_DURATION,
                            ))),
                        }
                        .as_asm(functions),
                    );
                }
                res.push(Instruction::Control(ControlASM::Pop(duration_var.clone())));
                res.push(Instruction::Control(ControlASM::FloatAsFrames(
                    duration_var.clone(),
                    duration_time_var.clone(),
                )));

                res.extend(context.emit_device(&target_device_id_var, functions));

                res.push(Instruction::Effect(
                    Event::MidiNote(
                        note_var.clone(),
                        velocity_var.clone(),
                        chan_var.clone(),
                        duration_time_var.clone(),
                        target_device_id_var.clone(),
                    ),
                    0.0.into(),
                ));
            }
            Effect::ProgramChange(p, c) => {
                let context = c.update(&context);
                res.extend(p.as_asm(functions));
                res.push(Instruction::Control(ControlASM::Pop(program_var.clone())));

                res.extend(context.emit_channel(&chan_var, functions));
                res.extend(context.emit_device(&target_device_id_var, functions));

                res.push(Instruction::Effect(
                    Event::MidiProgram(
                        program_var.clone(),
                        chan_var.clone(),
                        target_device_id_var.clone(),
                    ),
                    0.0.into(),
                ));
            }
            Effect::ControlChange(con, v, c) => {
                let context = c.update(&context);
                res.extend(con.as_asm(functions));
                res.push(Instruction::Control(ControlASM::Pop(control_var.clone())));
                res.extend(v.as_asm(functions));
                res.push(Instruction::Control(ControlASM::Pop(value_var.clone())));

                res.extend(context.emit_channel(&chan_var, functions));
                res.extend(context.emit_device(&target_device_id_var, functions));

                res.push(Instruction::Effect(
                    Event::MidiControl(
                        control_var.clone(),
                        value_var.clone(),
                        chan_var.clone(),
                        target_device_id_var.clone(),
                    ),
                    0.0.into(),
                ));
            }
            Effect::Osc(addr, args, osc_context) => {
                let context = osc_context.update(&context);
                let osc_addr_var = Variable::Instance("_osc_addr".to_string());

                res.push(addr.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(osc_addr_var.clone())));

                let mut temp_arg_vars: Vec<Variable> = Vec::new();
                for (i, arg_expr) in args.iter().enumerate() {
                    let temp_var_name = match arg_expr {
                        Expression::Value(Value::String(_)) => format!("_osc_string_arg_{}", i),
                        _ => format!("_osc_float_arg_{}", i),
                    };
                    let temp_var = Variable::Instance(temp_var_name.to_string());

                    match arg_expr {
                        Expression::Value(Value::String(s)) => {
                            res.push(Instruction::Control(ControlASM::Mov(
                                Variable::Constant(s.clone().into()),
                                temp_var.clone(),
                            )));
                        }
                        _ => {
                            res.extend(arg_expr.as_asm(functions));
                            res.push(Instruction::Control(ControlASM::Pop(temp_var.clone())));
                        }
                    }
                    temp_arg_vars.push(temp_var);
                }

                res.extend(context.emit_device(&target_device_id_var, functions));

                let event = Event::Osc {
                    addr: osc_addr_var.clone(),
                    args: temp_arg_vars,
                    device_id: target_device_id_var.clone(),
                };
                res.push(Instruction::Effect(event, 0.0.into()));
            }
            Effect::Dirt(sound, params, dirt_context) => {
                let context = dirt_context.update(&context);
                let dirt_sound_var = Variable::Instance("_dirt_sound".to_string());

                res.push(sound.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(
                    dirt_sound_var.clone(),
                )));

                let mut params_map = HashMap::new();
                for (key, val) in params.iter() {
                    let param_value_var = Variable::Instance(format!("_dirt_param_{}_val", key));

                    match val.as_ref() {
                        Expression::Value(Value::String(s)) => {
                            res.push(Instruction::Control(ControlASM::Mov(
                                Variable::Constant(s.clone().into()),
                                param_value_var.clone(),
                            )));
                        }
                        _ => {
                            res.extend(val.as_asm(functions));
                            res.push(Instruction::Control(ControlASM::Pop(
                                param_value_var.clone(),
                            )));
                        }
                    }
                    params_map.insert(key.clone(), param_value_var);
                }

                res.extend(context.emit_device(&target_device_id_var, functions));

                let event = Event::Dirt {
                    sound: dirt_sound_var,
                    params: params_map,
                    device_id: target_device_id_var.clone(),
                };
                res.push(Instruction::Effect(event, 0.0.into()));
            }
            Effect::Aftertouch(note_expr, value_expr, c) => {
                let context = c.update(&context);
                let at_note_var = Variable::Instance("_at_note".to_owned());
                let at_value_var = Variable::Instance("_at_value".to_owned());

                res.extend(note_expr.as_asm(functions));
                res.push(Instruction::Control(ControlASM::Pop(at_note_var.clone())));
                res.extend(value_expr.as_asm(functions));
                res.push(Instruction::Control(ControlASM::Pop(at_value_var.clone())));

                res.extend(context.emit_channel(&chan_var, functions));
                res.extend(context.emit_device(&target_device_id_var, functions));

                res.push(Instruction::Effect(
                    Event::MidiAftertouch(
                        at_note_var,
                        at_value_var,
                        chan_var.clone(),
                        target_device_id_var.clone(),
                    ),
                    0.0.into(),
                ));
            }
            Effect::ChannelPressure(value_expr, c) => {
                let context = c.update(&context);
                let chanpress_value_var = Variable::Instance("_chanpress_value".to_owned());

                res.extend(value_expr.as_asm(functions));
                res.push(Instruction::Control(ControlASM::Pop(
                    chanpress_value_var.clone(),
                )));

                res.extend(context.emit_channel(&chan_var, functions));
                res.extend(context.emit_device(&target_device_id_var, functions));

                res.push(Instruction::Effect(
                    Event::MidiChannelPressure(
                        chanpress_value_var,
                        chan_var.clone(),
                        target_device_id_var.clone(),
                    ),
                    0.0.into(),
                ));
            }
        }

        res
    }
}
