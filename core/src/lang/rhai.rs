use std::collections::{BTreeMap, BTreeSet};

use rhai::{AST, ASTFlags, Engine, Expr, FnCallExpr, Stmt, StmtBlock, Token};

use crate::{
    clock::{NEVER, SyncTime}, compiler::{CompilationError, CompilationState, Compiler}, log_debug, log_println, scene::script::Script, vm::{
        EvaluationContext, Instruction, Program, control_asm::ControlASM, event::ConcreteEvent, interpreter::{Interpreter, InterpreterFactory}, variable::{Variable, VariableValue}
    }
};

pub const TEMP_REGISTER : usize = 1;
pub const RETURN_REGISTER : usize = 0;

#[derive(Debug)]
pub struct RhaiCompiler;

impl RhaiCompiler {

    pub fn write_fn_call(compiled: &mut Program, call: &FnCallExpr) {
        for arg in call.args.iter().rev() {
            Self::push_expr(compiled, arg, true);
        }
        match call.op_token {
            Some(Token::Plus) => {
                compiled.push(ControlASM::Add(Variable::StackBack, Variable::StackBack, Variable::StackBack).into());
            }
            Some(Token::Minus) => {
                compiled.push(ControlASM::Sub(Variable::StackBack, Variable::StackBack, Variable::StackBack).into());
            }
            Some(Token::Multiply) => {
                compiled.push(ControlASM::Mul(Variable::StackBack, Variable::StackBack, Variable::StackBack).into());
            }
            Some(Token::Divide) => {
                compiled.push(ControlASM::Div(Variable::StackBack, Variable::StackBack, Variable::StackBack).into());
            }
            Some(Token::Modulo) => {
                compiled.push(ControlASM::Mod(Variable::StackBack, Variable::StackBack, Variable::StackBack).into());
            }
            Some(Token::UnaryMinus) => {
                compiled.push(ControlASM::Neg(Variable::StackBack, Variable::StackBack).into());
            }

            Some(Token::And) => {
                compiled.push(ControlASM::And(Variable::StackBack, Variable::StackBack, Variable::StackBack).into());
            }
            Some(Token::Or) => {
                compiled.push(ControlASM::Or(Variable::StackBack, Variable::StackBack, Variable::StackBack).into());
            }
            Some(Token::XOr) => {
                compiled.push(ControlASM::Xor(Variable::StackBack, Variable::StackBack, Variable::StackBack).into());
            }
            Some(Token::Bang) => {
                compiled.push(ControlASM::Not(Variable::StackBack, Variable::StackBack).into());
            }

            Some(Token::Ampersand) => {
                compiled.push(ControlASM::BitAnd(Variable::StackBack, Variable::StackBack, Variable::StackBack).into());
            }
            Some(Token::Pipe) => {
                compiled.push(ControlASM::BitOr(Variable::StackBack, Variable::StackBack, Variable::StackBack).into());
            }
            
            _ => {
                let name = call.name.to_string();
                compiled.push(ControlASM::CallFunction(Variable::Instance(name)).into());
            }
        }
    }

    pub fn get_lhs(compiled: &mut Program, expr: &Expr) -> Variable {
        todo!()
    }

    pub fn push_expr(compiled: &mut Program, expr: &Expr, force_push: bool) -> Variable {
        let mut ret = Variable::StackBack;
        match expr {
            Expr::DynamicConstant(_, _) => unimplemented!(),
            Expr::BoolConstant(b, _) => ret = (*b).into(),
            Expr::IntegerConstant(i, _) => ret = (*i).into(),
            Expr::FloatConstant(f, _) => ret = (**f).into(),
            Expr::CharConstant(c, _) => ret = String::from(*c).into(),
            Expr::StringConstant(string, _) => ret = string.to_string().into(),
            Expr::InterpolatedString(_, _) => todo!(),
            Expr::Array(x, _) => {
                let temp = Variable::reg(TEMP_REGISTER);
                compiled.push(ControlASM::Mov(VariableValue::Vec(Default::default()).into(), temp.clone()).into());
                for v in x.iter() {
                    let value = Self::push_expr(compiled, v, false);
                    compiled.push(ControlASM::VecPush(temp.clone(), value, temp.clone()).into());
                }
                compiled.push(ControlASM::Push(temp).into());
            }
            Expr::Map(x, _) => {
                let temp = Variable::reg(TEMP_REGISTER);
                compiled.push(ControlASM::Mov(VariableValue::Map(Default::default()).into(), temp.clone()).into());
                for (k,v) in x.0.iter() {
                    let key = k.name.to_string();
                    let value = Self::push_expr(compiled, v, false);
                    compiled.push(ControlASM::MapInsert(temp.clone(), key.into(), value, temp.clone()).into());
                }
                compiled.push(ControlASM::Push(temp).into());
            }
            Expr::Unit(_) => 
                ret = Default::default(),
            Expr::Variable(ident, _non_zero, _) => 
                ret = Variable::Instance(ident.1.to_string()),
            Expr::Property(ident, _) => {

            },
            Expr::MethodCall(fn_call_expr, _) => todo!(),
            Expr::Stmt(block) => {
                Self::write_stmt_block(compiled, block.iter());
                compiled.push(ControlASM::Push(Variable::reg(RETURN_REGISTER)).into());
            }
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
            Expr::ThisPtr(_) => unimplemented!(),
            _ => unimplemented!(),
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
        compiled_block.push(ControlASM::Mov(Default::default(), Variable::reg(RETURN_REGISTER)).into());
        for stmt in block {
            match stmt {
                Stmt::Noop(_) => (),
                Stmt::If(flow_control, _) => {
                    let mut body = Program::new();
                    let mut branch = Program::new();
                    let ret1 = Self::write_stmt_block(&mut body, flow_control.body.iter());
                    let ret2 = Self::write_stmt_block(&mut branch, flow_control.branch.iter());
                    let branch_size = branch.len() as i64;
                    body.push(ControlASM::RelJump(branch_size + 1).into());
                    let body_size = body.len() as i64;
                    let expr = Self::push_expr(&mut compiled_block, &flow_control.expr, false);
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
                        let expr = Self::push_expr(&mut compiled_block, &flow_control.expr, false);
                        compiled_block.push(ControlASM::RelJumpIfNot(expr, body_size + 1).into());
                    }
                    compiled_block.append(&mut body);
                }
                Stmt::Do(flow_control, astflags, _) => {
                    let size_before = compiled_block.len();
                    let mut body = Program::new();
                    Self::write_stmt_block(&mut body, flow_control.body.iter());
                    compiled_block.append(&mut body);
                    let expr = Self::push_expr(&mut compiled_block, &flow_control.expr, false);
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
                    let res = Self::push_expr(&mut compiled_block, &def.1, false);
                    compiled_block.push(match res {
                        Variable::StackBack => ControlASM::Pop(Variable::Instance(name)),
                        res => ControlASM::Mov(res, Variable::Instance(name))
                    }.into()); 
                }
                Stmt::Assignment(assign) => {
                    let rhs = Self::push_expr(&mut compiled_block, &assign.1.rhs, false);
                    let lhs = Self::get_lhs(&mut compiled_block, &assign.1.lhs);
                    if assign.0.is_op_assignment() {
                        match rhs {
                            Variable::StackBack => compiled_block.push(ControlASM::Pop(lhs).into()),
                            rhs => compiled_block.push(ControlASM::Mov(rhs, lhs).into())
                        }
                    } else {
                        match assign.0.get_op_assignment_info().unwrap().2 {
                            Token::PlusAssign => compiled_block.push(ControlASM::Add(lhs.clone(), rhs, lhs).into()),
                            Token::MinusAssign => compiled_block.push(ControlASM::Sub(lhs.clone(), rhs, lhs).into()),
                            Token::MultiplyAssign => compiled_block.push(ControlASM::Mul(lhs.clone(), rhs, lhs).into()),
                            Token::DivideAssign => compiled_block.push(ControlASM::Div(lhs.clone(), rhs, lhs).into()),
                            Token::ModuloAssign => compiled_block.push(ControlASM::Mod(lhs.clone(), rhs, lhs).into()),
                            Token::AndAssign => compiled_block.push(ControlASM::And(lhs.clone(), rhs, lhs).into()),
                            Token::OrAssign => compiled_block.push(ControlASM::Or(lhs.clone(), rhs, lhs).into()),
                            Token::XOrAssign => compiled_block.push(ControlASM::Xor(lhs.clone(), rhs, lhs).into()),
                            _ => unimplemented!()
                        }
                    }
                }
                Stmt::FnCall(call, _) => {
                    Self::write_fn_call(&mut compiled_block, call);
                }
                Stmt::Block(sub_block) => {
                    Self::write_stmt_block(&mut compiled_block, sub_block.iter());
                }
                Stmt::Expr(expr) => {
                    let res = Self::push_expr(&mut compiled_block, &expr, false);
                    todo!();
                }
                Stmt::BreakLoop(expr, astflags, _) => {
                    if let Some(expr) = expr {
                        Self::push_expr(&mut compiled_block, &expr, true);
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
                        Self::push_expr(&mut compiled_block, &expr, true);
                    }
                    compiled_block.push(ControlASM::Return.into());
                }
                Stmt::TryCatch(_flow_control, _) => unimplemented!(),
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
        engine.disable_symbol("try");
        engine.disable_symbol("throw");
        engine.disable_symbol("this");
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
