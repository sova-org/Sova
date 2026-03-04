use rhai::{AST, ASTFlags, Expr, FnCallExpr, Position, Stmt};

use super::runtime::{Instruction, LoweredProgram};

pub fn lower_ast(ast: &AST) -> Result<LoweredProgram, String> {
    if ast.has_functions() {
        return Err(error_at(
            Position::NONE,
            "user-defined functions are not supported in Rhai v1",
        ));
    }

    let mut lowerer = Lowerer::default();
    for stmt in ast.statements() {
        lowerer.lower_stmt(stmt)?;
    }

    Ok(LoweredProgram {
        instructions: lowerer.instructions,
    })
}

#[derive(Default)]
struct Lowerer {
    instructions: Vec<Instruction>,
    loops: Vec<LoopFrame>,
    next_for_id: usize,
}

#[derive(Default)]
struct LoopFrame {
    continue_target: Option<usize>,
    continue_jumps: Vec<usize>,
    break_jumps: Vec<usize>,
}

impl Lowerer {
    fn lower_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Noop(..) => Ok(()),
            Stmt::Block(block) => {
                for stmt in block.iter() {
                    self.lower_stmt(stmt)?;
                }
                Ok(())
            }
            Stmt::If(flow, ..) => self.lower_if(flow),
            Stmt::While(flow, ..) => self.lower_while(flow),
            Stmt::For(for_info, pos) => self.lower_for(for_info, *pos),
            Stmt::Var(var_info, ..) => self.lower_var(var_info),
            Stmt::Assignment(assignment) => self.lower_assignment(assignment),
            Stmt::FnCall(call, pos) => {
                if is_emit_call(call) || is_delay_call(call) {
                    self.lower_event_stmt(call, *pos)
                } else {
                    validate_fn_call(call, false)?;
                    let expr = Expr::FnCall(call.clone(), *pos);
                    self.instructions.push(Instruction::EvalExpr(expr));
                    Ok(())
                }
            }
            Stmt::Expr(expr) => {
                if let Expr::FnCall(call, pos) = expr.as_ref() {
                    if is_emit_call(call) || is_delay_call(call) {
                        return self.lower_event_stmt(call, *pos);
                    }
                }
                validate_expr(expr, false)?;
                self.instructions
                    .push(Instruction::EvalExpr((**expr).clone()));
                Ok(())
            }
            Stmt::BreakLoop(value, flags, pos) => {
                if value.is_some() {
                    return Err(error_at(
                        *pos,
                        "break/continue values are not supported in Rhai v1",
                    ));
                }
                let jump_index = self.push_jump_placeholder();
                let Some(loop_frame) = self.loops.last_mut() else {
                    return Err(error_at(*pos, "break/continue outside of loop"));
                };
                if flags.intersects(ASTFlags::BREAK) {
                    loop_frame.break_jumps.push(jump_index);
                } else if let Some(target) = loop_frame.continue_target {
                    self.patch_jump(jump_index, target)?;
                } else {
                    loop_frame.continue_jumps.push(jump_index);
                }
                Ok(())
            }
            _ => Err(error_at(
                stmt.position(),
                "unsupported Rhai statement in v1",
            )),
        }
    }

    fn lower_if(&mut self, flow: &rhai::FlowControl) -> Result<(), String> {
        validate_expr(&flow.expr, false)?;
        let jump_if_false = self.instructions.len();
        self.instructions.push(Instruction::JumpIfFalse {
            cond: flow.expr.clone(),
            target: usize::MAX,
        });

        for stmt in flow.body.iter() {
            self.lower_stmt(stmt)?;
        }

        if flow.branch.is_empty() {
            let end = self.instructions.len();
            self.patch_jump(jump_if_false, end)?;
            return Ok(());
        }

        let jump_over_else = self.push_jump_placeholder();
        let else_start = self.instructions.len();
        self.patch_jump(jump_if_false, else_start)?;

        for stmt in flow.branch.iter() {
            self.lower_stmt(stmt)?;
        }

        let end = self.instructions.len();
        self.patch_jump(jump_over_else, end)?;
        Ok(())
    }

    fn lower_while(&mut self, flow: &rhai::FlowControl) -> Result<(), String> {
        let loop_start = self.instructions.len();

        let guard_jump = if matches!(flow.expr, Expr::Unit(..)) {
            None
        } else {
            validate_expr(&flow.expr, false)?;
            let index = self.instructions.len();
            self.instructions.push(Instruction::JumpIfFalse {
                cond: flow.expr.clone(),
                target: usize::MAX,
            });
            Some(index)
        };

        self.loops.push(LoopFrame {
            continue_target: Some(loop_start),
            continue_jumps: Vec::new(),
            break_jumps: Vec::new(),
        });

        for stmt in flow.body.iter() {
            self.lower_stmt(stmt)?;
        }

        self.instructions
            .push(Instruction::Jump { target: loop_start });
        let loop_end = self.instructions.len();

        let frame = self.loops.pop().unwrap_or_default();
        for jump in frame.continue_jumps {
            self.patch_jump(jump, loop_start)?;
        }
        for jump in frame.break_jumps {
            self.patch_jump(jump, loop_end)?;
        }

        if let Some(guard_jump) = guard_jump {
            self.patch_jump(guard_jump, loop_end)?;
        }

        Ok(())
    }

    fn lower_for(
        &mut self,
        for_info: &(rhai::Ident, Option<rhai::Ident>, rhai::FlowControl),
        pos: Position,
    ) -> Result<(), String> {
        let (iter_ident, counter_ident, flow) = for_info;

        if !flow.branch.is_empty() {
            return Err(error_at(pos, "unexpected for-loop branch body in Rhai v1"));
        }

        validate_expr(&flow.expr, false)?;
        let for_id = self.next_for_id;
        self.next_for_id += 1;

        let init_index = self.instructions.len();
        self.instructions.push(Instruction::ForInit {
            id: for_id,
            iterable: flow.expr.clone(),
            iter_var: iter_ident.as_str().to_string(),
            counter_var: counter_ident.as_ref().map(|x| x.as_str().to_string()),
            exit_target: usize::MAX,
        });

        self.loops.push(LoopFrame {
            continue_target: None,
            continue_jumps: Vec::new(),
            break_jumps: Vec::new(),
        });

        let body_start = self.instructions.len();
        for stmt in flow.body.iter() {
            self.lower_stmt(stmt)?;
        }

        let for_next = self.instructions.len();
        self.instructions.push(Instruction::ForNext {
            id: for_id,
            body_start,
        });

        let cleanup = self.instructions.len();
        self.instructions
            .push(Instruction::ForCleanup { id: for_id });

        self.patch_for_exit(init_index, cleanup)?;

        let frame = self.loops.pop().unwrap_or_default();
        for jump in frame.continue_jumps {
            self.patch_jump(jump, for_next)?;
        }
        for jump in frame.break_jumps {
            self.patch_jump(jump, cleanup)?;
        }

        Ok(())
    }

    fn lower_var(
        &mut self,
        var_info: &(rhai::Ident, Expr, Option<std::num::NonZeroUsize>),
    ) -> Result<(), String> {
        validate_expr(&var_info.1, false)?;
        self.instructions.push(Instruction::SetVar {
            name: var_info.0.as_str().to_string(),
            expr: var_info.1.clone(),
        });
        Ok(())
    }

    fn lower_assignment(
        &mut self,
        assignment: &(rhai::OpAssignment, rhai::BinaryExpr),
    ) -> Result<(), String> {
        let (target_name, indices) =
            collect_assignment_target(&assignment.1.lhs).ok_or_else(|| {
                error_at(
                    assignment.0.position(),
                    "unsupported assignment target (only variables and index chains are supported)",
                )
            })?;

        validate_expr(&assignment.1.rhs, false)?;
        for index in &indices {
            validate_expr(index, false)?;
        }

        let op = assignment
            .0
            .get_op_assignment_info()
            .map(|(_, _, _, _, _, op_syntax)| op_syntax.to_string());

        if indices.is_empty() {
            if let Some(op) = op {
                self.instructions.push(Instruction::SetVarOp {
                    name: target_name,
                    op,
                    expr: assignment.1.rhs.clone(),
                });
            } else {
                self.instructions.push(Instruction::SetVar {
                    name: target_name,
                    expr: assignment.1.rhs.clone(),
                });
            }
        } else if let Some(op) = op {
            self.instructions.push(Instruction::SetIndexOp {
                name: target_name,
                indices,
                op,
                expr: assignment.1.rhs.clone(),
            });
        } else {
            self.instructions.push(Instruction::SetIndex {
                name: target_name,
                indices,
                expr: assignment.1.rhs.clone(),
            });
        }

        Ok(())
    }

    fn lower_event_stmt(&mut self, call: &FnCallExpr, pos: Position) -> Result<(), String> {
        if is_emit_call(call) {
            if !(call.args.len() == 1 || call.args.len() == 2 || call.args.len() == 4) {
                return Err(error_at(pos, "EMIT supports only 1, 2, or 4 arguments"));
            }
            for arg in &call.args {
                validate_expr(arg, false)?;
            }
            let instr = Instruction::Emit {
                args: call.args[0].clone(),
                dur: call.args.get(1).cloned(),
                chan: if call.args.len() == 4 {
                    call.args.get(2).cloned()
                } else {
                    None
                },
                dev: if call.args.len() == 4 {
                    call.args.get(3).cloned()
                } else {
                    None
                },
            };
            self.instructions.push(instr);
            return Ok(());
        }

        if call.args.len() != 1 {
            return Err(error_at(pos, "DELAY expects exactly one argument"));
        }
        validate_expr(&call.args[0], false)?;
        self.instructions.push(Instruction::Delay {
            dur: call.args[0].clone(),
        });
        Ok(())
    }

    fn push_jump_placeholder(&mut self) -> usize {
        let index = self.instructions.len();
        self.instructions
            .push(Instruction::Jump { target: usize::MAX });
        index
    }

    fn patch_jump(&mut self, index: usize, target: usize) -> Result<(), String> {
        let Some(instruction) = self.instructions.get_mut(index) else {
            return Err("invalid jump patch index".to_string());
        };

        match instruction {
            Instruction::Jump { target: out_target } => {
                *out_target = target;
                Ok(())
            }
            Instruction::JumpIfFalse {
                target: out_target, ..
            } => {
                *out_target = target;
                Ok(())
            }
            _ => Err("attempted to patch a non-jump instruction".to_string()),
        }
    }

    fn patch_for_exit(&mut self, index: usize, target: usize) -> Result<(), String> {
        let Some(instruction) = self.instructions.get_mut(index) else {
            return Err("invalid for-exit patch index".to_string());
        };

        if let Instruction::ForInit { exit_target, .. } = instruction {
            *exit_target = target;
            Ok(())
        } else {
            Err("attempted to patch a non-for-init instruction".to_string())
        }
    }
}

fn collect_assignment_target(expr: &Expr) -> Option<(String, Vec<Expr>)> {
    match expr {
        Expr::Variable(var, ..) => Some((var.1.to_string(), Vec::new())),
        Expr::Index(binary, ..) => {
            let (name, mut indices) = collect_assignment_target(&binary.lhs)?;
            indices.push(binary.rhs.clone());
            Some((name, indices))
        }
        _ => None,
    }
}

pub fn validate_expr(expr: &Expr, allow_special_stmt_calls: bool) -> Result<(), String> {
    match expr {
        Expr::DynamicConstant(..)
        | Expr::BoolConstant(..)
        | Expr::IntegerConstant(..)
        | Expr::FloatConstant(..)
        | Expr::CharConstant(..)
        | Expr::StringConstant(..)
        | Expr::Unit(..)
        | Expr::Variable(..) => Ok(()),
        Expr::InterpolatedString(parts, ..) => {
            for part in parts {
                validate_expr(part, false)?;
            }
            Ok(())
        }
        Expr::Array(items, ..) => {
            for item in items {
                validate_expr(item, false)?;
            }
            Ok(())
        }
        Expr::Map(map, ..) => {
            for (_, value) in map.0.iter() {
                validate_expr(value, false)?;
            }
            Ok(())
        }
        Expr::FnCall(call, ..) => validate_fn_call(call, allow_special_stmt_calls),
        Expr::Index(binary, ..) => {
            validate_expr(&binary.lhs, false)?;
            validate_expr(&binary.rhs, false)
        }
        Expr::And(items, ..) | Expr::Or(items, ..) | Expr::Coalesce(items, ..) => {
            for item in items.iter() {
                validate_expr(item, false)?;
            }
            Ok(())
        }
        _ => Err(error_at(
            expr.start_position(),
            "unsupported Rhai expression in v1",
        )),
    }
}

fn validate_fn_call(call: &FnCallExpr, allow_special_stmt_calls: bool) -> Result<(), String> {
    if call.capture_parent_scope {
        return Err(error_at(
            call.args
                .first()
                .map(Expr::start_position)
                .unwrap_or(Position::NONE),
            "capturing parent scope is not supported in Rhai v1",
        ));
    }

    let name = call.name.as_str();

    if is_emit_call(call) {
        if !allow_special_stmt_calls {
            return Err(error_at(
                call.args
                    .first()
                    .map(Expr::start_position)
                    .unwrap_or(Position::NONE),
                "EMIT can only be used as a statement",
            ));
        }
        if !(call.args.len() == 1 || call.args.len() == 2 || call.args.len() == 4) {
            return Err("EMIT supports only 1, 2, or 4 arguments".to_string());
        }
    } else if is_delay_call(call) {
        if !allow_special_stmt_calls {
            return Err(error_at(
                call.args
                    .first()
                    .map(Expr::start_position)
                    .unwrap_or(Position::NONE),
                "DELAY can only be used as a statement",
            ));
        }
        if call.args.len() != 1 {
            return Err("DELAY expects exactly one argument".to_string());
        }
    } else if is_duration_builder(name) {
        if call.args.len() != 1 {
            return Err(format!("{name} expects exactly one argument"));
        }
    } else if !is_supported_operator(name, call.args.len()) {
        return Err(error_at(
            call.args
                .first()
                .map(Expr::start_position)
                .unwrap_or(Position::NONE),
            &format!(
                "unsupported function call '{name}' (user-defined functions are disabled in v1)"
            ),
        ));
    }

    for arg in &call.args {
        validate_expr(arg, false)?;
    }

    Ok(())
}

fn is_emit_call(call: &FnCallExpr) -> bool {
    call.name.as_str() == "EMIT"
}

fn is_delay_call(call: &FnCallExpr) -> bool {
    call.name.as_str() == "DELAY"
}

fn is_duration_builder(name: &str) -> bool {
    matches!(name, "beats" | "frames" | "micros")
}

fn is_supported_operator(name: &str, arity: usize) -> bool {
    matches!(
        (name, arity),
        ("+", 1)
            | ("+", 2)
            | ("-", 1)
            | ("-", 2)
            | ("!", 1)
            | ("~", 1)
            | ("*", 2)
            | ("/", 2)
            | ("%", 2)
            | ("**", 2)
            | ("==", 2)
            | ("!=", 2)
            | ("<", 2)
            | ("<=", 2)
            | (">", 2)
            | (">=", 2)
            | ("&&", 2)
            | ("||", 2)
            | ("&", 2)
            | ("|", 2)
            | ("^", 2)
            | ("<<", 2)
            | (">>", 2)
            | ("??", 2)
    )
}

fn error_at(pos: Position, message: &str) -> String {
    if let Some(line) = pos.line() {
        let col = pos.position().unwrap_or(0);
        format!("line {line}, col {col}: {message}")
    } else {
        message.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhai::Engine;

    #[test]
    fn rejects_user_defined_functions() {
        let engine = Engine::new();
        let ast = engine
            .compile("fn foo() { 1 }\nlet x = 1;")
            .expect("failed to compile script");
        let err = lower_ast(&ast).expect_err("expected function definitions to be rejected");
        assert!(
            err.contains("user-defined functions"),
            "unexpected error text: {err}"
        );
    }

    #[test]
    fn lowers_for_loops_and_op_assignments() {
        let engine = Engine::new();
        let ast = engine
            .compile(
                r#"
                let acc = 0;
                for (x, i) in [1, 2, 3] {
                    acc += x;
                    if i == 1 { continue; }
                    acc -= 1;
                }
                "#,
            )
            .expect("failed to compile script");

        let lowered = lower_ast(&ast).expect("lowering failed");
        assert!(
            lowered
                .instructions
                .iter()
                .any(|inst| matches!(inst, Instruction::ForInit { .. })),
            "expected at least one for-init instruction"
        );
        assert!(
            lowered
                .instructions
                .iter()
                .any(|inst| matches!(inst, Instruction::SetVarOp { op, .. } if op == "+")),
            "expected += lowering into SetVarOp"
        );
    }

    #[test]
    fn rejects_method_calls() {
        let engine = Engine::new();
        let ast = engine
            .compile("let x = [1,2,3]; let y = x.len();")
            .expect("failed to compile script");
        let err = lower_ast(&ast).expect_err("expected method call to be rejected");
        assert!(
            err.contains("unsupported Rhai expression"),
            "unexpected error text: {err}"
        );
    }
}
