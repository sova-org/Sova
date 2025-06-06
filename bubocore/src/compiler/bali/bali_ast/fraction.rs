use crate::{
    lang::{
        Instruction,
        control_asm::ControlASM,
        variable::Variable,
    },
    compiler::bali::bali_ast::{
        value::Value,
        expression::Expression,
        concrete_fraction::ConcreteFraction,
        function::FunctionContent,
    },
};

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Fraction {
    pub numerator: Box<Expression>,
    pub denominator: Box<Expression>,
}

impl Fraction {
    pub fn from_dec_string(dec: String) -> Fraction {
        let concrete = ConcreteFraction::from_dec_string(dec);
        Fraction {
            numerator: Box::new(Expression::Value(Value::Number(concrete.numerator))),
            denominator: Box::new(Expression::Value(Value::Number(concrete.denominator))),
        }
    }

    pub fn as_asm(&self, functions: &HashMap<String, FunctionContent>) -> Vec<Instruction> {
        let var_1 = Variable::Instance("_exp1_frac".to_owned());
        let var_2 = Variable::Instance("_exp2_frac".to_owned());
        let var_out = Variable::Instance("_res_frac".to_owned());
        let mut e1 = vec![
            Instruction::Control(ControlASM::Mov(0.0.into(), var_1.clone())),
            Instruction::Control(ControlASM::Mov(0.0.into(), var_2.clone())),
            Instruction::Control(ControlASM::Mov(0.0.into(), var_out.clone())),
        ];
        e1.extend(self.numerator.as_asm(&functions));
        e1.extend(self.denominator.as_asm(&functions));
        e1.push(Instruction::Control(ControlASM::Pop(var_2.clone())));
        e1.push(Instruction::Control(ControlASM::Pop(var_1.clone())));
        e1.push(Instruction::Control(ControlASM::Div(
            var_1.clone(),
            var_2.clone(),
            var_out.clone(),
        )));
        e1.push(Instruction::Control(ControlASM::Push(var_out.clone())));
        e1
    }
}