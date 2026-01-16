//! Compiler for the Bob language.
//!
//! Transforms Bob AST into Sova VM bytecode instructions.
//! Bob is purely expression-oriented - everything is an expression.

use crate::compiler::{CompilationError, Compiler};
use crate::lang::bob::bob_ast::{BobExpr, BobProgram};
use crate::lang::bob::bob_grammar;
use crate::lang::bob::compile_expr::compile_expr;
use crate::lang::bob::context::CompileContext;
use crate::vm::Program;
use lalrpop_util::ParseError;
use std::collections::BTreeMap;

// ============================================================================
// Compiler
// ============================================================================

#[derive(Debug)]
pub struct BobCompiler;

impl Compiler for BobCompiler {
    fn name(&self) -> &str {
        "bob"
    }

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
                crate::lang::bob::context::FunctionInfo {
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
