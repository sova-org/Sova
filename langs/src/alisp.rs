use std::collections::BTreeMap;

use sova_core::{compiler::{CompilationError, Compiler}, vm::{Instruction, Language, Program, control_asm::ControlASM, language::{LanguageDocumentation, LanguageSyntax}, variable::Variable}};

use crate::alisp::parser::parse_alisp;

mod parser;
mod ast;
mod words;

const CONTEXT_REG       : usize = 0;
const LIST_LEN_REG      : usize = 1;
const FLAG_REG          : usize = 2;
const INDEX_REG         : usize = 3;
const DICTIONARY_REG    : usize = 4;
const FN_DESC_REG       : usize = 5;

const EXECUTE_ELEM_ADDR : usize = 1;

#[derive(Debug, Clone)]
pub struct ALispCompiler;

impl Language for ALispCompiler {
    fn name(&self) -> &str {
        "alisp"
    }

    fn version(&self) -> (usize, usize, usize) {
        (1,0,0)
    }
    
    fn documentation(&self) -> LanguageDocumentation { Default::default() }
    
    fn syntax(&self) -> Option<LanguageSyntax> { None }
}

impl Compiler for ALispCompiler {
    fn compile(&self, text: &str, _args: &BTreeMap<String, String>) -> Result<Program, CompilationError> {
        use ControlASM::*;
        let ast = parse_alisp(text)?;
        let exec_prog : Vec<Instruction> = vec![
            IsVec(Variable::StackBack, Variable::reg(FLAG_REG)).into(),
            RelJumpIfNot(Variable::reg(FLAG_REG), 2).into(),
            Return.into(),
            PushList(Variable::StackBack).into(),
            CallProcedure(EXECUTE_ELEM_ADDR).into(),
            IsMap(Variable::StackBack, Variable::reg(FLAG_REG)).into(),
            RelJumpIfNot(Variable::reg(FLAG_REG), 4).into(),
            Contains(Variable::StackBack, String::from("_sym").into(), Variable::reg(FLAG_REG)).into(),
            RelJumpIfNot(Variable::reg(FLAG_REG), 2).into(),

            Pop(Variable::reg(FN_DESC_REG)).into(),


            CallFunction(Variable::StackBack).into(),
            Return.into()
        ];
        todo!()
    }
}