use std::collections::BTreeMap;

use sova_core::{compiler::{CompilationError, Compiler}, vm::{Language, Program, language::{LanguageDocumentation, LanguageSyntax}}};

use crate::alisp::parser::parse_alisp;

mod parser;
mod ast;
mod words;

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
        todo!()
    }
}