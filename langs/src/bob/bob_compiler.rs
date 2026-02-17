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
        use sova_core::vm::language::{LanguageDocumentation, LanguageElement::*, ReferenceEntry};
        let mut doc = LanguageDocumentation::default();
        doc.reference.insert(Word("ADD".into()), ReferenceEntry::new("Addition — ADD a b").with_example("PLAY [note: ADD 60 7]"));
        doc.reference.insert(Word("SUB".into()), ReferenceEntry::new("Subtraction — SUB a b").with_example("PLAY [note: SUB 72 12]"));
        doc.reference.insert(Word("MUL".into()), ReferenceEntry::new("Multiplication — MUL a b").with_example("PLAY [vel: MUL 45 2]"));
        doc.reference.insert(Word("DIV".into()), ReferenceEntry::new("Division — DIV a b"));
        doc.reference.insert(Word("MOD".into()), ReferenceEntry::new("Modulo — MOD a b"));
        doc.reference.insert(Word("RAND".into()), ReferenceEntry::new("Random float 0..1"));
        doc.reference.insert(Word("RRAND".into()), ReferenceEntry::new("Random in range — RRAND lo hi").with_example("PLAY [note: RRAND 48 72]"));
        doc.reference.insert(Word("PLAY".into()), ReferenceEntry::new("Emit an event — PLAY [key: val, ...]").with_example("PLAY [note: 60, vel: 100]"));
        doc.reference.insert(Word("WAIT".into()), ReferenceEntry::new("Wait beats — WAIT n").with_example("PLAY [note: 60]\nWAIT 1\nPLAY [note: 64]"));
        doc.reference.insert(Word("DEV".into()), ReferenceEntry::new("Set target device — DEV n").with_example("DEV 2\nPLAY [note: 60]"));
        doc.reference.insert(Word("L".into()), ReferenceEntry::new("Loop — L start end : body END").with_example("L 0 4 :\n  PLAY [note: ADD 60 I]\n  WAIT 1\nEND"));
        doc.reference.insert(Word("I".into()), ReferenceEntry::new("Loop index variable"));
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
