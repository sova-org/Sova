use std::{cell::LazyCell, collections::BTreeMap};

use pest::Stack;
use sova_core::{compiler::{CompilationError, Compiler}, vm::{Instruction, Language, Program, control_asm::ControlASM, language::{LanguageDocumentation, LanguageSyntax}, resolve_gotos, variable::Variable}};

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
const FN_NAME_REG       : usize = 6;
const FN_SCOPE_REG      : usize = 7;
const LIST_REG          : usize = 8;
const COMPUTE_REG       : usize = 9;
const DUR_REG           : usize = 10;
const DEVICE_REG        : usize = 11;
const CHANNEL_REG       : usize = 12;

const EXECUTE_ELEM_ADDR : usize = 1;

const APPLY_CTX : LazyCell<Program> = LazyCell::new(|| {
    use ControlASM::*;
    use Variable::*;
    let reg = Variable::reg;
    vec![
        Delete(reg(COMPUTE_REG)).into(),
        Pop(reg(COMPUTE_REG)).into(),

        PushFront(reg(CONTEXT_REG)).into(),
        CallProcedure(EXECUTE_ELEM_ADDR).into(), // Execute first item
        PopFront(reg(CONTEXT_REG)).into(),

        Insert(reg(CONTEXT_REG), reg(COMPUTE_REG), StackBack, reg(CONTEXT_REG)).into(),

        CallProcedure(EXECUTE_ELEM_ADDR).into(),
    ]
});

const EXEC_PROG : LazyCell<Program> = LazyCell::new(|| {
    use ControlASM::*;
    use Variable::*;
    let reg = Variable::reg;
    let mut main = vec![
        // Assert element is vec,
        IsVec(StackBack, reg(FLAG_REG)).into(),
        RelJumpIfNot(reg(FLAG_REG), 2).into(),
        Return.into(), // If atomic item : just return

        PushFront(reg(LIST_LEN_REG)).into(),
        PushFront(reg(CONTEXT_REG)).into(),
        PushFront(reg(DUR_REG)).into(),
        PushFront(reg(CHANNEL_REG)).into(),
        PushFront(reg(DEVICE_REG)).into(),
        
        Len(StackBack, reg(LIST_LEN_REG)).into(),
        PushList(StackBack).into(),
        
        CallProcedure(EXECUTE_ELEM_ADDR).into(), // Execute first item

        IsMap(StackBack, reg(FLAG_REG)).into(), // assert new first item is a Word 
        RelJumpIfNot(reg(FLAG_REG), 3).into(),
        Contains(StackBack, "_sym".into(), reg(FLAG_REG)).into(),
        RelJumpIf(reg(FLAG_REG), 3).into(),
        PopList(reg(LIST_LEN_REG), StackBack).into(), // else 
        GoTo("quit".to_string()).into(),

        Pop(reg(FN_DESC_REG)).into(),
        Index(reg(FN_DESC_REG), "_sym".into(), reg(FN_NAME_REG)).into(),
        Index(reg(FN_DESC_REG), "_scope".into(), reg(FN_SCOPE_REG)).into(),

        Sub(reg(LIST_LEN_REG), 1.into(), reg(LIST_LEN_REG)).into(),
        PopList(reg(LIST_LEN_REG), StackBack).into(),

        DynIsSet(reg(FN_NAME_REG), Variable::reg(FN_SCOPE_REG), reg(FLAG_REG)).into(),
        RelJumpIfNot(reg(FLAG_REG), 5).into(),

        DynSrcMov(reg(FN_NAME_REG), reg(FN_SCOPE_REG), StackBack).into(),
        IsFunction(StackBack, reg(FLAG_REG)).into(),
        RelJumpIfNot(reg(FLAG_REG), 8).into(),
        Delete(reg(COMPUTE_REG)).into(),
        Pop(reg(COMPUTE_REG)).into(),
        Pop(reg(LIST_REG)).into(),

        GoTo("quit".to_string()).into(),

        Contains(reg(DICTIONARY_REG), reg(FN_NAME_REG), reg(FLAG_REG)).into(),
        RelJumpIfNot(reg(FLAG_REG), 3).into(),
        Error(vec!["Cannot find a value for word: ".into(), reg(FN_NAME_REG)]).into(),
        GoTo("quit".to_string()).into(),
        Index(reg(DICTIONARY_REG), reg(FN_NAME_REG), StackBack).into(),
        
        CallFunction(StackBack).into(),

        Symbol("quit".to_string()).into(),
        PopFront(reg(DEVICE_REG)).into(),
        PopFront(reg(CHANNEL_REG)).into(),
        PopFront(reg(DUR_REG)).into(),
        PopFront(reg(CONTEXT_REG)).into(),
        PopFront(reg(LIST_LEN_REG)).into(),
        Return.into()
    ];
    let size = main.len();
    main.insert(0, Jump(1 + size).into());
    resolve_gotos(main)
});

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
        let exec = EXEC_PROG;
        todo!()
    }
}