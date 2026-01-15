use crate::bali::bali_ast::{
    AltVariableGenerator, LocalChoiceVariableGenerator, bali_context::BaliContext,
    constants::FUNCTION_PREFIX, expression::Expression, toplevel_effect::TopLevelEffect,
};

use sova_core::vm::{
    Instruction,
    control_asm::ControlASM,
    variable::{Variable, VariableValue},
};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FunctionContent {
    pub arg_list: Vec<String>,
    pub return_expression: Box<Expression>,
    pub function_program: Vec<TopLevelEffect>,
}

impl FunctionContent {
    pub fn as_asm(
        &self,
        function_name: String,
        local_choice_vars: &mut LocalChoiceVariableGenerator,
        local_alt_vars: &mut AltVariableGenerator,
        functions: &HashMap<String, FunctionContent>,
    ) -> Instruction {
        let mut function_code = Vec::new();

        // get arguments from the stack
        for arg in self.arg_list.clone().into_iter().rev() {
            let instance_var = Variable::Instance(arg);
            function_code.push(Instruction::Control(ControlASM::Pop(instance_var)));
        }

        // apply the effects
        for effect in &self.function_program {
            function_code.extend(effect.as_asm(
                BaliContext::new(),
                local_choice_vars,
                local_alt_vars,
                functions,
            ));
        }

        // compute the return value and put it on the stack
        function_code.extend(self.return_expression.as_asm(functions));

        // return from function
        function_code.push(Instruction::Control(ControlASM::Return));

        let var_name = format!("{}{}", FUNCTION_PREFIX, function_name);

        Instruction::Control(ControlASM::Mov(
            Variable::Constant(VariableValue::Func(function_code)),
            Variable::Instance(var_name),
        ))
    }
}
