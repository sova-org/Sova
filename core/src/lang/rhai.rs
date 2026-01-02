use rhai::{AST, Engine, NativeCallContext, Scope, Stmt};

use crate::{
    clock::{NEVER, SyncTime},
    compiler::{CompilationError, CompilationState},
    vm::{
        EvaluationContext,
        event::ConcreteEvent,
        interpreter::{Interpreter, InterpreterFactory},
    },
    log_debug, log_println,
    scene::script::Script,
};

pub struct RhaiInterpreter {
    engine: Engine,
    ast: AST,
    scope: Scope<'static>,
}

impl RhaiInterpreter {
    pub fn initialize_context_watcher(&mut self, ctx: &mut EvaluationContext) {
        for var in ctx.global_vars.iter() {}
    }
}

impl Interpreter for RhaiInterpreter {
    fn execute_next(&mut self, ctx: &mut EvaluationContext) -> (Option<ConcreteEvent>, SyncTime) {
        let statements = self.ast.statements();
        self.engine.eval_ast::<i64>(&self.ast);
        (None, NEVER)
    }

    fn has_terminated(&self) -> bool {
        todo!()
    }

    fn stop(&mut self) {
        todo!()
    }
}

pub struct RhaiInterpreterFactory;

impl InterpreterFactory for RhaiInterpreterFactory {
    fn name(&self) -> &str {
        "rhai"
    }

    fn make_instance(&self, script: &Script) -> Result<Box<dyn Interpreter>, String> {
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
        match engine.compile(script.content()) {
            Ok(ast) => Ok(Box::new(RhaiInterpreter {
                engine,
                ast,
                scope: Scope::new(),
            })),
            Err(e) => Err(e.to_string()),
        }
    }

    fn check(&self, script: &Script) -> CompilationState {
        match Engine::new_raw().compile(script.content()) {
            Ok(_ast) => CompilationState::Parsed(None),
            Err(e) => {
                let line = e.1.line().unwrap_or(0);
                CompilationState::Error(CompilationError {
                    lang: self.name().to_owned(),
                    info: e.0.to_string(),
                    from: line,
                    to: line,
                })
            }
        }
    }
}
