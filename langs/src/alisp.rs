use std::{cell::LazyCell, collections::BTreeMap};

use pest::Stack;
use sova_core::{compiler::{CompilationError, Compiler}, sova_prog, vm::{Instruction, Language, Program, control_asm::ControlASM, language::{LanguageDocumentation, LanguageSyntax}, resolve_gotos, variable::{Variable, VariableValue}}};

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
    let reg = Variable::reg;
    sova_prog![
        Delete(reg(COMPUTE_REG)),
        Pop(reg(COMPUTE_REG)),

        PushFront(reg(CONTEXT_REG)),
        CallProcedure(EXECUTE_ELEM_ADDR), // Execute first item
        PopFront(reg(CONTEXT_REG)),

        Insert(reg(CONTEXT_REG), reg(COMPUTE_REG), StackBack, reg(CONTEXT_REG)),

        CallProcedure(EXECUTE_ELEM_ADDR)
    ]
});

const EXEC_PROG : LazyCell<Program> = LazyCell::new(|| {
    let reg = Variable::reg;
    let mut main = sova_prog![
        // Assert element is vec,
        IsVec(StackBack, reg(FLAG_REG)),
        RelJumpIfNot(reg(FLAG_REG), 2),
        Return, // If atomic item : just return

        PushFront(reg(LIST_LEN_REG)),
        PushFront(reg(CONTEXT_REG)),
        PushFront(reg(DUR_REG)),
        PushFront(reg(CHANNEL_REG)),
        PushFront(reg(DEVICE_REG)),
        
        Len(StackBack, reg(LIST_LEN_REG)),
        PushList(StackBack),
        
        CallProcedure(EXECUTE_ELEM_ADDR), // Execute first item

        IsMap(StackBack, reg(FLAG_REG)), // assert new first item is a Word 
        RelJumpIfNot(reg(FLAG_REG), 3),
        Contains(StackBack, "_sym".into(), reg(FLAG_REG)),
        RelJumpIf(reg(FLAG_REG), 9),     // If symbol, continue

        // Execute each element of the list, then push it and quit
        Mov(Vec::<VariableValue>::new().into(), reg(LIST_REG)), 
        VecPush(reg(LIST_REG), StackBack, reg(LIST_REG)),
        Sub(reg(LIST_LEN_REG), 1.into(), reg(LIST_LEN_REG)),
        RelJumpIfLessOrEqual(reg(LIST_LEN_REG), 0.into(), 3),
        CallProcedure(EXECUTE_ELEM_ADDR),
        RelJump(-3),
        Push(reg(LIST_REG)),
        GoTo("quit".to_string()),

        // If a symbol has been found
        Pop(reg(FN_DESC_REG)),
        Index(reg(FN_DESC_REG), "_sym".into(), reg(FN_NAME_REG)),
        Index(reg(FN_DESC_REG), "_scope".into(), reg(FN_SCOPE_REG)),

        Sub(reg(LIST_LEN_REG), 1.into(), reg(LIST_LEN_REG)),
        PopList(reg(LIST_LEN_REG), StackBack),

        DynIsSet(reg(FN_NAME_REG), Variable::reg(FN_SCOPE_REG), reg(FLAG_REG)),
        RelJumpIf(reg(FLAG_REG), 2),
        GoTo("lookup_fn".to_string()),

        DynSrcMov(reg(FN_NAME_REG), reg(FN_SCOPE_REG), StackBack),
        IsFunction(StackBack, reg(FLAG_REG)),
        RelJumpIfNot(reg(FLAG_REG), 8),
        Delete(reg(COMPUTE_REG)),
        Pop(reg(COMPUTE_REG)),
        Pop(reg(LIST_REG)),

        GoTo("quit".to_string()),

        Symbol("lookup_fn".to_string()),
        Contains(reg(DICTIONARY_REG), reg(FN_NAME_REG), reg(FLAG_REG)),
        RelJumpIfNot(reg(FLAG_REG), 3),
        Error(vec!["Cannot find a value for word: ".into(), reg(FN_NAME_REG)]),
        GoTo("quit".to_string()),
        Index(reg(DICTIONARY_REG), reg(FN_NAME_REG), StackBack),
        
        CallFunction(StackBack),

        Symbol("quit".to_string()),
        PopFront(reg(DEVICE_REG)),
        PopFront(reg(CHANNEL_REG)),
        PopFront(reg(DUR_REG)),
        PopFront(reg(CONTEXT_REG)),
        PopFront(reg(LIST_LEN_REG)),
        Return
    ];
    let size = main.len();
    main.insert(0, ControlASM::Jump(1 + size).into());
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
        let ast = parse_alisp(text)?;
        let exec = EXEC_PROG;
        todo!()
    }
}