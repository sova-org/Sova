use crate::{
    compiler::bali::bali_ast::{
        constants::FUNCTION_PREFIX, function::FunctionContent, value::Value,
    },
    lang::{
        Instruction,
        control_asm::ControlASM,
        variable::Variable,
        environment_func::EnvironmentFunc,
    },
    log_println,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Expression {
    Addition(Box<Expression>, Box<Expression>),
    Multiplication(Box<Expression>, Box<Expression>),
    Subtraction(Box<Expression>, Box<Expression>),
    Division(Box<Expression>, Box<Expression>),
    Modulo(Box<Expression>, Box<Expression>),
    Function(String, Vec<Box<Expression>>),
    RandomFrac(Box<Expression>, Box<Expression>),
    Scale(
        Box<Expression>,
        Box<Expression>,
        Box<Expression>,
        Box<Expression>,
        Box<Expression>,
    ), // value, old_min, old_max, new_min, new_max
    Clamp(Box<Expression>, Box<Expression>, Box<Expression>), // value, min, max
    Min(Box<Expression>, Box<Expression>),
    Max(Box<Expression>, Box<Expression>),
    Quantize(Box<Expression>, Box<Expression>), // value, step
    Sine(Box<Expression>),                      // speed
    Saw(Box<Expression>),                       // speed
    Triangle(Box<Expression>),                  // speed
    ISaw(Box<Expression>),                      // speed (inverted saw)
    RandStep(Box<Expression>),                  // speed (random step LFO)
    MidiCC(
        Box<Expression>,
        Option<Box<Expression>>,
        Option<Box<Expression>>,
    ),
    Value(Value),
}

impl Expression {
    pub fn as_asm(&self, functions: &HashMap<String, FunctionContent>) -> Vec<Instruction> {
        // Standard temporary variables for expression evaluation
        let var_1 = Variable::Instance("_exp1".to_owned());
        let var_2 = Variable::Instance("_exp2".to_owned());
        let var_3 = Variable::Instance("_exp3".to_owned());
        let var_4 = Variable::Instance("_exp4".to_owned());
        let var_5 = Variable::Instance("_exp5".to_owned());
        let speed_var = Variable::Instance("_osc_speed".to_owned());
        let var_out = Variable::Instance("_res".to_owned());

        let mut res_asm =
            match self {
                // Binary operations: Evaluate operands, pop into temps, execute operation into var_out
                Expression::Addition(e1, e2)
                | Expression::Multiplication(e1, e2)
                | Expression::Subtraction(e1, e2)
                | Expression::Division(e1, e2)
                | Expression::Modulo(e1, e2)
                | Expression::Min(e1, e2)
                | Expression::Max(e1, e2)
                | Expression::Quantize(e1, e2)
                | Expression::RandomFrac(e1, e2) => {
                    let mut asm = e1.as_asm(functions);
                    asm.extend(e2.as_asm(functions));
                    asm.push(Instruction::Control(ControlASM::Pop(var_2.clone())));
                    asm.push(Instruction::Control(ControlASM::Pop(var_1.clone())));
                    match self {
                        Expression::Addition(_, _) => asm.push(Instruction::Control(
                            ControlASM::Add(var_1.clone(), var_2.clone(), var_out.clone()),
                        )),
                        Expression::Multiplication(_, _) => asm.push(Instruction::Control(
                            ControlASM::Mul(var_1.clone(), var_2.clone(), var_out.clone()),
                        )),
                        Expression::Subtraction(_, _) => asm.push(Instruction::Control(
                            ControlASM::Sub(var_1.clone(), var_2.clone(), var_out.clone()),
                        )),
                        Expression::Division(_, _) => asm.push(Instruction::Control(
                            ControlASM::Div(var_1.clone(), var_2.clone(), var_out.clone()),
                        )),
                        Expression::Modulo(_, _) => asm.push(Instruction::Control(
                            ControlASM::Mod(var_1.clone(), var_2.clone(), var_out.clone()),
                        )),
                        Expression::Min(_, _) => asm.push(Instruction::Control(ControlASM::Min(
                            var_1.clone(),
                            var_2.clone(),
                            var_out.clone(),
                        ))),
                        Expression::Max(_, _) => asm.push(Instruction::Control(ControlASM::Max(
                            var_1.clone(),
                            var_2.clone(),
                            var_out.clone(),
                        ))),
                        Expression::Quantize(_, _) => asm.push(Instruction::Control(
                            ControlASM::Quantize(var_1.clone(), var_2.clone(), var_out.clone()),
                        )),
                        Expression::RandomFrac(_, _) => asm.push(Instruction::Control(ControlASM::Mov(
                            Variable::Environment(EnvironmentFunc::RandomDecInBounds(Box::new(var_1.clone()), Box::new(var_2.clone()))),
                            var_out.clone(),
                        ))),
                        _ => unreachable!(), // Should not happen due to outer match
                    }
                    asm
                }
                Expression::Function(name, args) => {
                    log_println!("Call function {} with args {:?}", name, args);

                    let mut asm = Vec::new();

                    // check function existence and number of arguments
                    let function = functions.get(name);

                    if let Some(function) = function {
                        if function.arg_list.len() == args.len() {
                            // put each variable corresponding to an argument name of the function on the stack
                            let mut arg_order = Vec::new();
                            for arg in &function.arg_list {
                                let arg_var = Variable::Instance(arg.to_string());
                                arg_order.push(arg_var.clone());
                                asm.push(Instruction::Control(ControlASM::Push(arg_var)));
                            }

                            // compute each argument and put it on stack
                            for arg in args {
                                asm.extend(arg.as_asm(functions));
                            }

                            // call function
                            let func_var =
                                Variable::Instance(format!("{}{}", FUNCTION_PREFIX, name));
                            asm.push(Instruction::Control(ControlASM::CallFunction(func_var)));

                            // get result from stack
                            asm.push(Instruction::Control(ControlASM::Pop(var_out.clone())));

                            // restore each variable corresponding to an argument name of the function from the stack
                            while let Some(arg_var) = arg_order.pop() {
                                asm.push(Instruction::Control(ControlASM::Pop(arg_var)));
                            }
                        } else {
                            asm.push(Instruction::Control(ControlASM::Mov(
                                0.into(),
                                var_out.clone(),
                            )));
                        }
                    } else {
                        asm.push(Instruction::Control(ControlASM::Mov(
                            0.into(),
                            var_out.clone(),
                        )));
                    }

                    asm
                }
                Expression::Scale(val, old_min, old_max, new_min, new_max) => {
                    let mut asm = val.as_asm(functions);
                    asm.extend(old_min.as_asm(functions));
                    asm.extend(old_max.as_asm(functions));
                    asm.extend(new_min.as_asm(functions));
                    asm.extend(new_max.as_asm(functions));
                    asm.push(Instruction::Control(ControlASM::Pop(var_5.clone())));
                    asm.push(Instruction::Control(ControlASM::Pop(var_4.clone())));
                    asm.push(Instruction::Control(ControlASM::Pop(var_3.clone())));
                    asm.push(Instruction::Control(ControlASM::Pop(var_2.clone())));
                    asm.push(Instruction::Control(ControlASM::Pop(var_1.clone())));
                    asm.push(Instruction::Control(ControlASM::Scale(
                        var_1.clone(),
                        var_2.clone(),
                        var_3.clone(),
                        var_4.clone(),
                        var_5.clone(),
                        var_out.clone(),
                    )));
                    asm
                }
                Expression::Clamp(val, min, max) => {
                    let mut asm = val.as_asm(functions);
                    asm.extend(min.as_asm(functions));
                    asm.extend(max.as_asm(functions));
                    asm.push(Instruction::Control(ControlASM::Pop(var_3.clone())));
                    asm.push(Instruction::Control(ControlASM::Pop(var_2.clone())));
                    asm.push(Instruction::Control(ControlASM::Pop(var_1.clone())));
                    asm.push(Instruction::Control(ControlASM::Clamp(
                        var_1.clone(),
                        var_2.clone(),
                        var_3.clone(),
                        var_out.clone(),
                    )));
                    asm
                }
                Expression::Sine(speed_expr)
                | Expression::Saw(speed_expr)
                | Expression::Triangle(speed_expr)
                | Expression::ISaw(speed_expr)
                | Expression::RandStep(speed_expr) => {
                    let mut asm = speed_expr.as_asm(functions);
                    asm.push(Instruction::Control(ControlASM::Pop(speed_var.clone())));
                    match self {
                        Expression::Sine(_) => asm.push(Instruction::Control(ControlASM::GetSine(
                            speed_var.clone(),
                            var_out.clone(),
                        ))),
                        Expression::Saw(_) => asm.push(Instruction::Control(ControlASM::GetSaw(
                            speed_var.clone(),
                            var_out.clone(),
                        ))),
                        Expression::Triangle(_) => asm.push(Instruction::Control(
                            ControlASM::GetTriangle(speed_var.clone(), var_out.clone()),
                        )),
                        Expression::ISaw(_) => asm.push(Instruction::Control(ControlASM::GetISaw(
                            speed_var.clone(),
                            var_out.clone(),
                        ))),
                        Expression::RandStep(_) => asm.push(Instruction::Control(
                            ControlASM::GetRandStep(speed_var.clone(), var_out.clone()),
                        )),
                        _ => unreachable!(),
                    }
                    asm
                }
                // MidiCC: Evaluate control expression, pop into midi_cc_ctrl_var, execute GetMidiCCFromContext into var_out
                Expression::MidiCC(ctrl_expr, device_expr_opt, channel_expr_opt) => {
                    let mut asm = Vec::new();
                    // Temporary variables for specific CC lookup, used only if provided
                    let ccin_device_id_var = Variable::Instance("_ccin_device_id".to_owned());
                    let ccin_chan_var = Variable::Instance("_ccin_chan".to_owned());
                    // Variable for control number
                    let ccin_ctrl_var = Variable::Instance("_ccin_ctrl".to_owned());
                    // Placeholder variables to signal using context
                    let use_context_device_var =
                        Variable::Instance("_use_context_device".to_owned());
                    let use_context_channel_var =
                        Variable::Instance("_use_context_channel".to_owned());

                    // 1. Evaluate the control number expression first
                    asm.extend(ctrl_expr.as_asm(functions));
                    asm.push(Instruction::Control(ControlASM::Pop(ccin_ctrl_var.clone())));

                    // 2. Determine and evaluate Device Variable
                    let device_var_to_pass = if let Some(device_expr) = device_expr_opt {
                        // Evaluate specific device expression
                        asm.extend(device_expr.as_asm(functions));
                        asm.push(Instruction::Control(ControlASM::Pop(
                            ccin_device_id_var.clone(),
                        )));
                        ccin_device_id_var // Pass the variable holding the evaluated result
                    } else {
                        use_context_device_var // Pass the placeholder to signal using context
                    };

                    // 3. Determine and evaluate Channel Variable
                    let channel_var_to_pass = if let Some(channel_expr) = channel_expr_opt {
                        // Evaluate specific channel expression
                        asm.extend(channel_expr.as_asm(functions));
                        asm.push(Instruction::Control(ControlASM::Pop(ccin_chan_var.clone())));
                        ccin_chan_var // Pass the variable holding the evaluated result
                    } else {
                        use_context_channel_var // Pass the placeholder to signal using context
                    };

                    // 4. Always generate the single GetMidiCC instruction
                    asm.push(Instruction::Control(ControlASM::GetMidiCC(
                        device_var_to_pass,  // Either specific var or context placeholder
                        channel_var_to_pass, // Either specific var or context placeholder
                        ccin_ctrl_var.clone(),
                        var_out.clone(), // Standard result variable
                    )));

                    asm
                }
                Expression::Value(v) => {
                    vec![
                        v.as_asm(),                                             // Push the value onto stack
                        Instruction::Control(ControlASM::Pop(var_out.clone())), // Pop it into the result variable
                    ]
                }
            };

        // Common final step for all expressions: Push the computed result (`var_out`)
        // onto the stack for the *next* operation or effect to use.
        res_asm.push(Instruction::Control(ControlASM::Push(var_out.clone())));
        res_asm
    }
}
