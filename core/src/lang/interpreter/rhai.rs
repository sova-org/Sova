use rhai::{AST, Engine};

use crate::{
    clock::SyncTime,
    compiler::{CompilationError, CompilationState},
    lang::{
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
}

impl Interpreter for RhaiInterpreter {
    fn execute_next(&mut self, ctx: &mut EvaluationContext) -> (Option<ConcreteEvent>, SyncTime) {
        todo!()
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
        engine.on_debug(|txt, src, pos| {
            let src = src.map(|s| format!("({s}) ")).unwrap_or_default();
            log_debug!("Rhai @ {src}{pos} : {txt}");
        }).on_print(|txt| {
            log_println!("{txt}");
        });
        match engine.compile(script.content()) {
            Ok(ast) => Ok(Box::new(RhaiInterpreter { engine, ast })),
            Err(e) => Err(e.to_string())
        }
    }

    fn check(&self, script: &Script) -> CompilationState {
        match Engine::new_raw().compile(script.content()) {
            Ok(ast) => CompilationState::Parsed(None),
            Err(e) => CompilationState::Error(CompilationError {
                lang: "rhai".to_owned(),
                info: e.to_string(),
                from: 0,
                to: 0,
            })
        }
    }
}
