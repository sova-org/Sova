use crate::lang::bali::bali_ast::{expression::Expression, function::FunctionContent};
use crate::vm::{Instruction, control_asm::ControlASM, variable::Variable};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum BooleanExpression {
    And(Box<BooleanExpression>, Box<BooleanExpression>),
    Or(Box<BooleanExpression>, Box<BooleanExpression>),
    Not(Box<BooleanExpression>),
    Lower(Box<Expression>, Box<Expression>),
    LowerOrEqual(Box<Expression>, Box<Expression>),
    Greater(Box<Expression>, Box<Expression>),
    GreaterOrEqual(Box<Expression>, Box<Expression>),
    Equal(Box<Expression>, Box<Expression>),
    Different(Box<Expression>, Box<Expression>),
}

impl BooleanExpression {
    pub fn as_asm(&self, functions: &HashMap<String, FunctionContent>) -> Vec<Instruction> {
        let bvar_1 = Variable::Instance("_bexp1".to_owned());
        let bvar_2 = Variable::Instance("_bexp2".to_owned());
        let evar_1 = Variable::Instance("_exp1".to_owned());
        let evar_2 = Variable::Instance("_exp2".to_owned());
        let bvar_out = Variable::Instance("_bres".to_owned());
        let mut res = match self {
            BooleanExpression::And(e1, e2) | BooleanExpression::Or(e1, e2) => {
                let mut e1 = e1.as_asm(functions);
                e1.extend(e2.as_asm(functions));
                e1.push(Instruction::Control(ControlASM::Pop(bvar_2.clone())));
                e1.push(Instruction::Control(ControlASM::Pop(bvar_1.clone())));
                e1
            }
            BooleanExpression::Not(e) => {
                let mut e = e.as_asm(functions);
                e.push(Instruction::Control(ControlASM::Pop(bvar_1.clone())));
                e
            }
            BooleanExpression::Lower(e1, e2)
            | BooleanExpression::LowerOrEqual(e1, e2)
            | BooleanExpression::Greater(e1, e2)
            | BooleanExpression::GreaterOrEqual(e1, e2)
            | BooleanExpression::Equal(e1, e2)
            | BooleanExpression::Different(e1, e2) => {
                let mut e1 = e1.as_asm(functions);
                e1.extend(e2.as_asm(functions));
                e1.push(Instruction::Control(ControlASM::Pop(evar_2.clone())));
                e1.push(Instruction::Control(ControlASM::Pop(evar_1.clone())));
                e1
            }
        };
        match self {
            BooleanExpression::And(_, _) => {
                res.push(Instruction::Control(ControlASM::And(
                    bvar_1.clone(),
                    bvar_2.clone(),
                    bvar_out.clone(),
                )));
            }
            BooleanExpression::Or(_, _) => {
                res.push(Instruction::Control(ControlASM::Or(
                    bvar_1.clone(),
                    bvar_2.clone(),
                    bvar_out.clone(),
                )));
            }
            BooleanExpression::Not(_) => {
                res.push(Instruction::Control(ControlASM::Not(
                    bvar_1.clone(),
                    bvar_out.clone(),
                )));
            }
            BooleanExpression::Lower(_, _) => res.push(Instruction::Control(
                ControlASM::LowerThan(evar_1.clone(), evar_2.clone(), bvar_out.clone()),
            )),
            BooleanExpression::LowerOrEqual(_, _) => res.push(Instruction::Control(
                ControlASM::LowerOrEqual(evar_1.clone(), evar_2.clone(), bvar_out.clone()),
            )),
            BooleanExpression::Greater(_, _) => res.push(Instruction::Control(
                ControlASM::GreaterThan(evar_1.clone(), evar_2.clone(), bvar_out.clone()),
            )),
            BooleanExpression::GreaterOrEqual(_, _) => res.push(Instruction::Control(
                ControlASM::GreaterOrEqual(evar_1.clone(), evar_2.clone(), bvar_out.clone()),
            )),
            BooleanExpression::Equal(_, _) => res.push(Instruction::Control(ControlASM::Equal(
                evar_1.clone(),
                evar_2.clone(),
                bvar_out.clone(),
            ))),
            BooleanExpression::Different(_, _) => res.push(Instruction::Control(
                ControlASM::Different(evar_1.clone(), evar_2.clone(), bvar_out.clone()),
            )),
        };

        res.push(Instruction::Control(ControlASM::Push(bvar_out.clone())));
        res
    }
}
