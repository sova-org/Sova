use std::collections::{BTreeMap, BTreeSet};

use rhai::{AST, ASTFlags, Engine, Expr, FnCallExpr, Stmt, StmtBlock, Token};

use crate::{
    clock::{NEVER, SyncTime}, compiler::{CompilationError, CompilationState, Compiler}, log_debug, log_println, scene::script::Script, vm::{
        EvaluationContext, Instruction, Program, control_asm::ControlASM, event::ConcreteEvent, interpreter::{Interpreter, InterpreterFactory}, variable::Variable
    }
};

#[derive(Debug)]
pub struct RhaiCompiler;

impl RhaiCompiler {

    pub fn write_fn_call(compiled: &mut Program, call: &FnCallExpr) {
        match call.op_token {
            Some(Token::Plus) => {}
            _ => {
                let name = call.name.to_string();
                for arg in call.args.iter().rev() {
                    Self::push_expr(compiled, arg, false, true);
                }
                compiled.push(ControlASM::CallFunction(Variable::Instance(name)).into());
            }
        }
    }

    pub fn push_expr(compiled: &mut Program, expr: &Expr, lhs: bool, force_push: bool) -> Variable {
        let mut ret = Variable::StackBack;
        match expr {
            Expr::DynamicConstant(dynamic, _) => todo!(),
            Expr::BoolConstant(b, _) => ret = (*b).into(),
            Expr::IntegerConstant(i, _) => ret = (*i).into(),
            Expr::FloatConstant(f, _) => ret = (**f).into(),
            Expr::CharConstant(c, _) => ret = String::from(*c).into(),
            Expr::StringConstant(string, _) => ret = string.to_string().into(),
            Expr::InterpolatedString(thin_vec, _) => todo!(),
            Expr::Array(thin_vec, _) => todo!(),
            Expr::Map(_, _) => todo!(),
            Expr::Unit(_) => 
                ret = Default::default(),
            Expr::Variable(ident, _non_zero, _) => 
                ret = Variable::Instance(ident.1.to_string()),
            Expr::ThisPtr(_) => todo!(),
            Expr::Property(ident, _) => {

            },
            Expr::MethodCall(fn_call_expr, _) => todo!(),
            Expr::Stmt(stmt_block) => todo!(),
            Expr::FnCall(call, _) => {
                Self::write_fn_call(compiled, call);
            }
            Expr::Dot(binary_expr, astflags, _) => {
                if let (Expr::Variable(ident, _, _), Expr::Property(prop, _)) = (&binary_expr.lhs, &binary_expr.rhs) {
                    match ident.1.as_str() {
                        "global" => compiled.push(ControlASM::Push(Variable::Global(prop.2.to_string())).into()),
                        "line" => compiled.push(ControlASM::Push(Variable::Line(prop.2.to_string())).into()),
                        "frame" => compiled.push(ControlASM::Push(Variable::Frame(prop.2.to_string())).into()),
                        _ => {
                            todo!()
                        }
                    }
                }
            }
            Expr::Index(binary_expr, astflags, _) => todo!(),
            Expr::And(small_vec, _) => todo!(),
            Expr::Or(small_vec, _) => todo!(),
            Expr::Coalesce(small_vec, _) => todo!(),
            Expr::Custom(custom_expr, _) => todo!(),
            _ => todo!(),
        };
        if ret != Variable::StackBack && force_push {
            compiled.push(ControlASM::Push(ret).into());
            ret = Variable::StackBack;
        }
        ret
    }

    pub fn write_stmt_block<'a>(compiled: &'a mut Program, block: impl Iterator<Item = &'a Stmt>) {
        let mut redefinitions : Vec<String> = Vec::new();
        let mut compiled_block = Program::new();
        let mut breaks : Vec<usize> = Vec::new();
        let mut continues : Vec<usize> = Vec::new();
        for stmt in block {
            match stmt {
                Stmt::Noop(_) => compiled_block.push(ControlASM::Nop.into()),
                Stmt::If(flow_control, _) => {
                    let mut body = Program::new();
                    let mut branch = Program::new();
                    Self::write_stmt_block(&mut body, flow_control.body.iter());
                    Self::write_stmt_block(&mut branch, flow_control.branch.iter());
                    let branch_size = branch.len() as i64;
                    body.push(ControlASM::RelJump(branch_size + 1).into());
                    let body_size = body.len() as i64;
                    let expr = Self::push_expr(&mut compiled_block, &flow_control.expr, false, false);
                    compiled_block.push(ControlASM::RelJumpIfNot(expr, body_size + 1).into());
                    compiled_block.append(&mut body);
                    compiled_block.append(&mut branch);
                }
                Stmt::Switch(_control, _) => unimplemented!(),
                Stmt::While(flow_control, _) => {
                    let mut body = Program::new();
                    Self::write_stmt_block(&mut body, flow_control.body.iter());
                    let body_size = body.len() as i64;
                    body.push(ControlASM::RelJump(-1 - body_size).into());
                    if !flow_control.expr.is_unit() {
                        let body_size = body.len() as i64;
                        let expr = Self::push_expr(&mut compiled_block, &flow_control.expr, false, false);
                        compiled_block.push(ControlASM::RelJumpIfNot(expr, body_size + 1).into());
                    }
                    compiled_block.append(&mut body);
                }
                Stmt::Do(flow_control, astflags, _) => {
                    let size_before = compiled_block.len();
                    let mut body = Program::new();
                    Self::write_stmt_block(&mut body, flow_control.body.iter());
                    compiled_block.append(&mut body);
                    let expr = Self::push_expr(&mut compiled_block, &flow_control.expr, false, false);
                    let size_after = compiled_block.len();
                    let jump = (size_after - size_before) as i64 + 1;
                    if astflags.intersects(ASTFlags::NEGATED) {
                        body.push(ControlASM::RelJumpIfNot(expr, jump).into());
                    } else {
                        body.push(ControlASM::RelJumpIf(expr, jump).into());
                    }
                }
                Stmt::For(control, _) => todo!(),
                Stmt::Var(def, _astflags, _) => {
                    let name = def.0.name.to_string();
                    if def.2.is_some() { // Redefinition
                        compiled_block.push(ControlASM::Push(Variable::Instance(name.clone())).into());
                        redefinitions.push(name.clone());
                    }
                    let res = Self::push_expr(&mut compiled_block, &def.1, false, false);
                    compiled_block.push(match res {
                        Variable::StackBack => ControlASM::Pop(Variable::Instance(name)),
                        res => ControlASM::Mov(res, Variable::Instance(name))
                    }.into()); 
                }
                Stmt::Assignment(assign) => todo!(),
                Stmt::FnCall(call, _) => {
                    Self::write_fn_call(&mut compiled_block, call);
                }
                Stmt::Block(sub_block) => {
                    Self::write_stmt_block(&mut compiled_block, sub_block.iter());
                }
                Stmt::TryCatch(_flow_control, _) => unimplemented!(),
                Stmt::Expr(expr) => {
                    Self::push_expr(&mut compiled_block, &expr, false, true);
                }
                Stmt::BreakLoop(expr, astflags, _) => {
                    if let Some(expr) = expr {
                        Self::push_expr(&mut compiled_block, &expr, false, true);
                    }
                    let line = compiled.len();
                    if astflags.intersects(ASTFlags::BREAK) {
                        breaks.push(line);
                    } else {
                        continues.push(line);
                    }
                    compiled_block.push(Default::default());
                }
                Stmt::Return(expr, _astflags, _) => {
                    if let Some(expr) = expr {
                        Self::push_expr(&mut compiled_block, &expr, false, true);
                    }
                    compiled_block.push(ControlASM::Return.into());
                }
                Stmt::Share(small_vec) => todo!(),
                _ => todo!(),
            }
        }
        while let Some(redef) = redefinitions.pop() {
            compiled_block.push(ControlASM::Pop(Variable::Instance(redef)).into());
        }
        if compiled_block.is_empty() {
            compiled_block.push(ControlASM::Nop.into());
        }
        compiled.append(&mut compiled_block);
    }

    pub fn compile_functions(compiled: &mut Program, ast: &AST) -> Result<(), CompilationError> {
        for function in ast.iter_fn_def() {
            let mut fun_code = Program::new();
            let name = function.name.to_string();
            for arg in function.params.iter() {
                fun_code.push(ControlASM::Pop(Variable::Instance(arg.to_string())).into());
            }
            Self::write_stmt_block(&mut fun_code, function.body.iter());
            compiled.push(ControlASM::Mov(fun_code.into(), Variable::Instance(name)).into());
        }
        Ok(())
    }

    pub fn compile_ast(ast: AST) -> Result<Program, CompilationError> {
        let mut compiled = Program::new();
        Self::compile_functions(&mut compiled, &ast)?;
        Self::write_stmt_block(&mut compiled, ast.statements().iter());
        Ok(compiled)
    }
}

impl Compiler for RhaiCompiler {

    fn name(&self) -> &str {
        "rhai"
    }

    fn compile(&self, text: &str, _args: &BTreeMap<String, String>) -> Result<Program, CompilationError> {
        let mut engine = Engine::new_raw();
        engine.set_fast_operators(false);
        engine
            .on_debug(|txt, src, pos| {
                let src = src.map(|s| format!("({s}) ")).unwrap_or_default();
                log_debug!("Rhai @ {src}{pos} : {txt}");
            })
            .on_print(|txt| {
                log_println!("{txt}");
            });
        match engine.compile(text) {
            Ok(ast) => Self::compile_ast(ast),
            Err(e) => Err(CompilationError {
                lang: "rhai".to_owned(),
                info: e.0.to_string(),
                from: e.1.line().unwrap_or_default(),
                to: e.1.line().unwrap_or_default(),
            }),
        }
    }

}

