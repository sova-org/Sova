use std::collections::BTreeMap;

use rhai::{AST, Engine, Expr, Stmt, StmtBlock};

use crate::{
    clock::{NEVER, SyncTime}, compiler::{CompilationError, CompilationState, Compiler}, log_debug, log_println, scene::script::Script, vm::{
        EvaluationContext, Instruction, Program, control_asm::ControlASM, event::ConcreteEvent, interpreter::{Interpreter, InterpreterFactory}, variable::Variable
    }
};

#[derive(Debug)]
pub struct RhaiCompiler;

impl RhaiCompiler {

    pub fn push_expr(compiled: &mut Program, expr: &Expr, lhs: bool) -> Variable {
        let mut ret = Variable::StackBack;
        match expr {
            Expr::DynamicConstant(dynamic, _) => todo!(),
            Expr::BoolConstant(b, _) => ret = Variable::Constant((*b).into()),
            Expr::IntegerConstant(i, _) => ret = Variable::Constant((*i).into()),
            Expr::FloatConstant(f, _) => ret = Variable::Constant((**f).into()),
            Expr::CharConstant(c, _) => ret = Variable::Constant(String::from(*c).into()),
            Expr::StringConstant(string, _) => ret = Variable::Constant(string.to_string().into()),
            Expr::InterpolatedString(thin_vec, _) => todo!(),
            Expr::Array(thin_vec, _) => todo!(),
            Expr::Map(_, _) => todo!(),
            Expr::Unit(_) => 
                compiled.push(ControlASM::Push(Variable::Constant(Default::default())).into()),
            Expr::Variable(ident, _non_zero, _) => 
                compiled.push(ControlASM::Push(Variable::Instance(ident.1.to_string())).into()),
            Expr::ThisPtr(_) => todo!(),
            Expr::Property(ident, _) => {

            },
            Expr::MethodCall(fn_call_expr, _) => todo!(),
            Expr::Stmt(stmt_block) => todo!(),
            Expr::FnCall(fn_call_expr, _) => todo!(),
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
        ret
    }

    pub fn write_stmt_block<'a>(compiled: &'a mut Program, block: impl Iterator<Item = &'a Stmt>) {
        for stmt in block {
            match stmt {
                Stmt::Noop(_) => compiled.push(ControlASM::Noop.into()),
                Stmt::If(flow_control, _) => todo!(),
                Stmt::Switch(control, _) => todo!(),
                Stmt::While(flow_control, _) => todo!(),
                Stmt::Do(flow_control, astflags, _) => todo!(),
                Stmt::For(control, _) => todo!(),
                Stmt::Var(def, _astflags, _) => {
                    let name = def.0.name.to_string();
                    let res = Self::push_expr(compiled, &def.1, false);
                    compiled.push(match res {
                        Variable::StackBack => ControlASM::Pop(Variable::Instance(name)),
                        res => ControlASM::Mov(res, Variable::Instance(name))
                    }.into());
                }
                Stmt::Assignment(assign) => todo!(),
                Stmt::FnCall(fn_call_expr, _) => todo!(),
                Stmt::Block(stmt_block) => todo!(),
                Stmt::TryCatch(flow_control, _) => todo!(),
                Stmt::Expr(expr) => todo!(),
                Stmt::BreakLoop(expr, astflags, _) => todo!(),
                Stmt::Return(expr, _astflags, _) => {
                    if let Some(expr) = expr {
                        let res = Self::push_expr(compiled, &expr, false);
                        if res != Variable::StackBack {
                            compiled.push(ControlASM::Push(res).into());
                        }
                    }
                    compiled.push(ControlASM::Return.into());
                }
                Stmt::Share(small_vec) => todo!(),
                _ => todo!(),
            }
        }
    }

    pub fn link_calls(compiled: &mut Program, calls: Vec<(usize, String)>, defs: &BTreeMap<String, usize>) 
        -> Result<(), CompilationError>
    {
        for (index, name) in calls {
            let Some(call_index) = defs.get(&name) else {
                return Err(CompilationError { 
                    lang: "rhai".to_owned(), 
                    info: format!("Linker error: Unkown function {name}"), 
                    from: 0, 
                    to: 0
                })
            };
            compiled[index] = ControlASM::CallProcedure(*call_index).into()
        }
        Ok(())
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

