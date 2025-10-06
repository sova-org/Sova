use std::{borrow::Cow, collections::BTreeMap};

use crate::{
    compiler::{CompilationError, Compiler},
    lang::Program,
};

use crate::compiler::dummylang::dummygrammar;

#[derive(Debug)]
pub struct DummyCompiler;
impl Compiler for DummyCompiler {
    fn name(&self) -> &str {
        "dummy"
    }

    fn compile(&self, script: &str, _args: &BTreeMap<String, String>) -> Result<Program, CompilationError> {
        if let Ok(parsed) = dummygrammar::ProgParser::new().parse(script) {
            Ok(parsed.as_asm())
        } else {
            Err(CompilationError::default_error("dummylang".to_string()))
        }
    }

    fn syntax(&self) -> Option<Cow<'static, str>> {
        Some(Cow::Borrowed(include_str!(
            "../../static/syntaxes/dummy.sublime-syntax"
        )))
    }
}
