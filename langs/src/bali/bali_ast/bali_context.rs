use crate::bali::bali_ast::expression::Expression;
use crate::bali::bali_ast::function::FunctionContent;
use sova_core::vm::Instruction;
use sova_core::vm::control_asm::{ControlASM, DEFAULT_CHAN, DEFAULT_DEVICE};
use sova_core::vm::variable::Variable;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct BaliContext {
    pub channel: Option<Expression>,
    pub device: Option<Expression>,
    pub velocity: Option<Expression>,
    pub duration: Option<Expression>,
}

impl Default for BaliContext {
    fn default() -> Self {
        Self::new()
    }
}

impl BaliContext {
    pub fn new() -> BaliContext {
        BaliContext {
            channel: None,
            device: None,
            velocity: None,
            duration: None,
        }
    }

    pub fn update(&self, above: &BaliContext) -> BaliContext {
        BaliContext {
            channel: self.channel.clone().or_else(|| above.channel.clone()),
            device: self.device.clone().or_else(|| above.device.clone()),
            velocity: self.velocity.clone().or_else(|| above.velocity.clone()),
            duration: self.duration.clone().or_else(|| above.duration.clone()),
        }
    }

    pub fn emit_channel(
        &self,
        target_var: &Variable,
        functions: &HashMap<String, FunctionContent>,
    ) -> Vec<Instruction> {
        self.emit_field(&self.channel, target_var, DEFAULT_CHAN, functions)
    }

    pub fn emit_device(
        &self,
        target_var: &Variable,
        functions: &HashMap<String, FunctionContent>,
    ) -> Vec<Instruction> {
        self.emit_field(&self.device, target_var, DEFAULT_DEVICE, functions)
    }

    pub fn emit_velocity(
        &self,
        target_var: &Variable,
        default: i64,
        functions: &HashMap<String, FunctionContent>,
    ) -> Vec<Instruction> {
        self.emit_field(&self.velocity, target_var, default, functions)
    }

    fn emit_field(
        &self,
        field: &Option<Expression>,
        target_var: &Variable,
        default: i64,
        functions: &HashMap<String, FunctionContent>,
    ) -> Vec<Instruction> {
        match field {
            Some(expr) => {
                let mut res = expr.as_asm(functions);
                res.push(Instruction::Control(ControlASM::Pop(target_var.clone())));
                res
            }
            None => {
                vec![Instruction::Control(ControlASM::Mov(
                    default.into(),
                    target_var.clone(),
                ))]
            }
        }
    }
}
