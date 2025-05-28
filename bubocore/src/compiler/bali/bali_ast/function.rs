use crate::compiler::bali::bali_ast::{
    toplevel_effect::TopLevelEffect,
    expression::Expression,
};

#[derive(Debug)]
pub struct FunctionContent {
    pub arg_list: Vec<String>,
    pub return_expression: Box<Expression>,
    pub function_program: Vec<TopLevelEffect>,
}