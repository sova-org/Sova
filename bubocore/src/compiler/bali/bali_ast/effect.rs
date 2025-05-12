use crate::{
    lang::{
        Instruction,
        control_asm::ControlASM,
        variable::{Variable, VariableValue},
        event::Event,
    },
    protocol::osc::{Argument as OscArgument, OSCMessage},
    compiler::bali::bali_ast::{
        bali_context::BaliContext,
        value::Value,
        expression::Expression,
        fraction::Fraction,
        constants::{
            DEFAULT_VELOCITY,
            DEFAULT_CHAN,
            DEFAULT_DEVICE,
            DEFAULT_DURATION
        },
    },
};

#[derive(Debug, Clone)]
pub enum Effect {
    Definition(Value, Box<Expression>),
    Note(Box<Expression>, BaliContext),
    ProgramChange(Box<Expression>, BaliContext),
    ControlChange(Box<Expression>, Box<Expression>, BaliContext),
    Osc(Value, Vec<Expression>, BaliContext),
    Dirt(Box<Expression>, Vec<(String, Box<Expression>)>, BaliContext), // Changed Box<Expression> to Fraction
    Aftertouch(Box<Expression>, Box<Expression>, BaliContext),
    ChannelPressure(Box<Expression>, BaliContext),
}

impl Effect {
    pub fn as_asm(&self, context: BaliContext) -> Vec<Instruction> {
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
            Effect::Definition(v, expr) => {
                res.extend(expr.as_asm());
                if let Value::Variable(v) = v {
                    res.push(Instruction::Control(ControlASM::Pop(Value::as_variable(v))));
                }
            }
            Effect::Note(n, c) => {
                let context = c.clone().update(context);
                res.extend(n.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(note_var.clone())));

                if let Some(v) = context.velocity {
                    res.extend(v.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(velocity_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(
                        DEFAULT_VELOCITY.into(),
                        velocity_var.clone(),
                    )))
                }

                if let Some(ch) = context.channel {
                    res.extend(ch.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(
                        DEFAULT_CHAN.into(),
                        chan_var.clone(),
                    )))
                }

                if let Some(d) = context.duration {
                    res.extend(d.as_asm());
                } else {
                    res.extend(
                        Fraction {
                            numerator: Box::new(Expression::Value(Value::Number(1))),
                            denominator: Box::new(Expression::Value(Value::Number(
                                DEFAULT_DURATION,
                            ))),
                        }
                        .as_asm(),
                    );
                }
                res.push(Instruction::Control(ControlASM::Pop(duration_var.clone())));
                res.push(Instruction::Control(ControlASM::FloatAsFrames(
                    duration_var.clone(),
                    duration_time_var.clone(),
                )));

                if let Some(device_id) = context.device {
                    res.extend(device_id.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(
                        target_device_id_var.clone(),
                    )));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(
                        DEFAULT_DEVICE.into(),
                        target_device_id_var.clone(),
                    )));
                }

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
                let context = c.clone().update(context);
                res.extend(p.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(program_var.clone())));

                if let Some(ch) = context.channel {
                    res.extend(ch.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(
                        DEFAULT_CHAN.into(),
                        chan_var.clone(),
                    )))
                }

                if let Some(device_id) = context.device {
                    res.extend(device_id.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(
                        target_device_id_var.clone(),
                    )));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(
                        DEFAULT_DEVICE.into(),
                        target_device_id_var.clone(),
                    )));
                }

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
                let context = c.clone().update(context);
                res.extend(con.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(control_var.clone())));
                res.extend(v.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(value_var.clone())));

                if let Some(ch) = context.channel {
                    res.extend(ch.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(
                        DEFAULT_CHAN.into(),
                        chan_var.clone(),
                    )))
                }

                if let Some(device_id) = context.device {
                    res.extend(device_id.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(
                        target_device_id_var.clone(),
                    )));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(
                        DEFAULT_DEVICE.into(),
                        target_device_id_var.clone(),
                    )));
                }

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
                let context = osc_context.clone().update(context);
                let target_device_id_var = Variable::Instance("_target_device_id".to_string());
                let osc_addr_var = Variable::Instance("_osc_addr".to_string());
                let mut osc_args: Vec<OscArgument> = Vec::new();
                let mut arg_instrs: Vec<Instruction> = Vec::new();

                // Generate instructions to evaluate the address
                res.push(addr.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(osc_addr_var.clone())));

                // Generate instructions to evaluate dynamic arguments
                // and store them in temporary variables.
                let mut temp_arg_vars: Vec<Variable> = Vec::new();
                for (i, arg_expr) in args.iter().enumerate() {
                    let temp_var_name = match arg_expr {
                        Expression::Value(Value::String(_)) => format!("_osc_string_arg_{}", i),
                        _ => format!("_osc_float_arg_{}", i),
                    };
                    let temp_var = Variable::Instance(temp_var_name.to_string());
                    arg_instrs.extend(arg_expr.as_asm());
                    arg_instrs.push(Instruction::Control(ControlASM::Pop(temp_var.clone())));
                    temp_arg_vars.push(temp_var);
                }
                res.extend(arg_instrs); // Add evaluation instructions

                // Determine target device ID
                if let Some(device_id_expr) = context.device {
                    res.extend(device_id_expr.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(
                        target_device_id_var.clone(),
                    )));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(
                        DEFAULT_DEVICE.into(),
                        target_device_id_var.clone(),
                    )));
                }

                // Create the Event::Osc
                let event = Event::Osc {
                    addr: osc_addr_var.clone(),
                    args: temp_arg_vars,
                    device_id: target_device_id_var.clone(), // Event::Osc takes Variable
                };

                // Add the final effect instruction using the event directly
                res.push(Instruction::Effect(event, 0.0.into()));
            }
            Effect::Dirt(sound_expr, params, dirt_context) => {
                let context = dirt_context.clone().update(context);
                let target_device_id_var = Variable::Instance("_target_device_id".to_string());
                let dirt_data_var = Variable::Instance("_dirt_data".to_string());
                let mut eval_instrs: Vec<Instruction> = Vec::new();

                // --- Instructions to build the data map ---
                // 1. Create an empty map variable
                let map_init_var = Variable::Instance("_dirt_map_init".to_string());
                eval_instrs.push(Instruction::Control(ControlASM::MapEmpty(
                    map_init_var.clone(),
                )));

                // 2. Evaluate sound expression and add as "s"
                let sound_value_var = Variable::Instance("_dirt_sound_val".to_string());
                // --- Start Sound Handling Fix (Restored from previous version) ---
                match **sound_expr {
                    // Dereference Box<Expression>
                    Expression::Value(Value::String(ref s)) => {
                        // Sound is a literal string, insert it directly
                        let string_const_var = Variable::Constant(VariableValue::Str(s.clone()));
                        eval_instrs.push(Instruction::Control(ControlASM::MapInsert(
                            map_init_var.clone(),
                            VariableValue::Str("s".to_string()), // Key "s"
                            string_const_var, // Pass the Constant Variable holding the string
                            map_init_var.clone(), // Store back in the same map var
                        )));
                    }
                    _ => {
                        // Sound is a variable or complex expression, evaluate it
                        eval_instrs.extend(sound_expr.as_asm());
                        eval_instrs.push(Instruction::Control(ControlASM::Pop(
                            sound_value_var.clone(),
                        )));
                        eval_instrs.push(Instruction::Control(ControlASM::MapInsert(
                            map_init_var.clone(),
                            VariableValue::Str("s".to_string()), // Key "s"
                            sound_value_var, // Value (Variable holding evaluated sound)
                            map_init_var.clone(), // Store back in the same map var
                        )));
                    }
                }
                // --- End Sound Handling Fix ---

                // 3. Evaluate parameters and add to map
                for (key, value_frac) in params.iter() {
                    // Keep parameter handling as Fraction
                    let param_value_var = Variable::Instance(format!("_dirt_param_{}_val", key));
                    eval_instrs.extend(value_frac.as_asm()); // Use Fraction's as_asm
                    eval_instrs.push(Instruction::Control(ControlASM::Pop(
                        param_value_var.clone(),
                    )));
                    eval_instrs.push(Instruction::Control(ControlASM::MapInsert(
                        map_init_var.clone(),
                        VariableValue::Str(key.clone()), // Key
                        param_value_var,                 // Value (Variable holding evaluated param)
                        map_init_var.clone(),            // Store back
                    )));
                }
                // --- End map building ---

                // 4. Push the final map onto the stack and pop into dirt_data_var
                eval_instrs.push(Instruction::Control(ControlASM::Push(map_init_var.clone())));
                eval_instrs.push(Instruction::Control(ControlASM::Pop(dirt_data_var.clone())));

                // 5. Evaluate device context
                if let Some(device_id_expr) = context.device {
                    eval_instrs.extend(device_id_expr.as_asm());
                    eval_instrs.push(Instruction::Control(ControlASM::Pop(
                        target_device_id_var.clone(),
                    )));
                } else {
                    eval_instrs.push(Instruction::Control(ControlASM::Mov(
                        DEFAULT_DEVICE.into(),
                        target_device_id_var.clone(),
                    )));
                }

                // Add evaluation instructions first
                res.extend(eval_instrs);

                // 6. Create Event::Dirt using the variables holding the map and device ID
                let event = Event::Dirt {
                    data: dirt_data_var,             // Variable holding the map
                    device_id: target_device_id_var, // Variable holding the device ID
                };

                // 7. Add the final effect instruction
                res.push(Instruction::Effect(event, 0.0.into()));
            }
            Effect::Aftertouch(note_expr, value_expr, c) => {
                let context = c.clone().update(context);
                let note_var = Variable::Instance("_at_note".to_owned());
                let value_var = Variable::Instance("_at_value".to_owned());

                res.extend(note_expr.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(note_var.clone())));
                res.extend(value_expr.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(value_var.clone())));

                if let Some(ch) = context.channel {
                    res.extend(ch.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(
                        DEFAULT_CHAN.into(),
                        chan_var.clone(),
                    )))
                }

                if let Some(device_id) = context.device {
                    res.extend(device_id.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(
                        target_device_id_var.clone(),
                    )));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(
                        DEFAULT_DEVICE.into(),
                        target_device_id_var.clone(),
                    )));
                }

                res.push(Instruction::Effect(
                    Event::MidiAftertouch(
                        note_var,
                        value_var,
                        chan_var.clone(),
                        target_device_id_var.clone(),
                    ),
                    0.0.into(),
                ));
            }
            Effect::ChannelPressure(value_expr, c) => {
                let context = c.clone().update(context);
                let value_var = Variable::Instance("_chanpress_value".to_owned());

                res.extend(value_expr.as_asm());
                res.push(Instruction::Control(ControlASM::Pop(value_var.clone())));

                if let Some(ch) = context.channel {
                    res.extend(ch.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(chan_var.clone())));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(
                        DEFAULT_CHAN.into(),
                        chan_var.clone(),
                    )))
                }

                if let Some(device_id) = context.device {
                    res.extend(device_id.as_asm());
                    res.push(Instruction::Control(ControlASM::Pop(
                        target_device_id_var.clone(),
                    )));
                } else {
                    res.push(Instruction::Control(ControlASM::Mov(
                        DEFAULT_DEVICE.into(),
                        target_device_id_var.clone(),
                    )));
                }

                res.push(Instruction::Effect(
                    Event::MidiChannelPressure(
                        value_var,
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
