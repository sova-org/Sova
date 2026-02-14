//! Compiler for the Bob language.
//!
//! Transforms Bob AST into Sova VM bytecode instructions.
//! Bob is purely expression-oriented - everything is an expression.

use crate::bob::bob_ast::{BobExpr, BobProgram};
use crate::bob::bob_grammar;
use crate::bob::compile_expr::compile_expr;
use crate::bob::context::CompileContext;
use lalrpop_util::ParseError;
use sova_core::compiler::{CompilationError, Compiler};
use sova_core::vm::{Language, Program};
use std::collections::BTreeMap;

// ============================================================================
// Compiler
// ============================================================================

#[derive(Debug)]
pub struct BobCompiler;

impl Language for BobCompiler {
    fn name(&self) -> &str {
        "bob"
    }
    fn version(&self) -> (usize, usize, usize) {
        (1,0,0)
    }
    fn documentation(&self) -> sova_core::vm::language::LanguageDocumentation {
        use sova_core::vm::language::{LanguageDocumentation, LanguageElement::*};
        let mut doc = LanguageDocumentation::default();
        doc.reference.insert(Word("ADD".into()), "Addition — ADD a b".into());
        doc.reference.insert(Word("SUB".into()), "Subtraction — SUB a b".into());
        doc.reference.insert(Word("MUL".into()), "Multiplication — MUL a b".into());
        doc.reference.insert(Word("DIV".into()), "Division — DIV a b".into());
        doc.reference.insert(Word("MOD".into()), "Modulo — MOD a b".into());
        doc.reference.insert(Word("RAND".into()), "Random float 0..1".into());
        doc.reference.insert(Word("RRAND".into()), "Random in range — RRAND lo hi".into());
        doc.reference.insert(Word("PLAY".into()), "Emit an event — PLAY [key: val, ...]".into());
        doc.reference.insert(Word("WAIT".into()), "Wait beats — WAIT n".into());
        doc.reference.insert(Word("DEV".into()), "Set target device — DEV n".into());
        doc.reference.insert(Word("L".into()), "Loop — L start end : body END".into());
        doc.reference.insert(Word("I".into()), "Loop index variable".into());
        doc.articles.push(("Introduction".into(), include_str!("../../docs/bob/intro.md").into()));
        doc
    }
}

impl Compiler for BobCompiler {
    
    fn compile(
        &self,
        script: &str,
        _args: &BTreeMap<String, String>,
    ) -> Result<Program, CompilationError> {
        let preprocessed = super::bob_preprocess::preprocess(script);
        match bob_grammar::ProgramParser::new().parse(&preprocessed) {
            Ok(parsed) => Ok(bob_as_asm(parsed)),
            Err(parse_error) => {
                let (from, to) = match &parse_error {
                    ParseError::InvalidToken { location } => (*location, *location),
                    ParseError::UnrecognizedEof { location, .. } => (*location, *location),
                    ParseError::UnrecognizedToken {
                        token: (f, _, t), ..
                    } => (*f, *t),
                    ParseError::ExtraToken { token: (f, _, t) } => (*f, *t),
                    ParseError::User { .. } => (0, 0),
                };
                Err(CompilationError {
                    lang: "Bob".to_string(),
                    info: parse_error.to_string(),
                    from,
                    to,
                })
            }
        }
    }
}

fn bob_as_asm(program: BobProgram) -> Program {
    let mut ctx = CompileContext::new();

    // First pass: collect function definitions
    collect_function_defs(&program, &mut ctx);

    // Second pass: compile expression
    let dest = ctx.temp("_bob_result");
    compile_expr(&program, &dest, &mut ctx)
}

fn collect_function_defs(expr: &BobExpr, ctx: &mut CompileContext) {
    match expr {
        BobExpr::Seq(left, right) => {
            collect_function_defs(left, ctx);
            collect_function_defs(right, ctx);
        }
        BobExpr::FunctionDef { name, args, .. } => {
            ctx.functions.insert(
                name.clone(),
                crate::bob::context::FunctionInfo {
                    arg_names: args.clone(),
                },
            );
        }
        BobExpr::Fork { body } => {
            collect_function_defs(body, ctx);
        }
        _ => {}
    }
}
