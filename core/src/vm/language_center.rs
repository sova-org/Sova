use std::thread;

use crossbeam_channel::Sender;

use crate::{Scene, compiler::CompilationState, vm::{Transcoder, interpreter::InterpreterDirectory}, scene::{Line, script::Script}, schedule::SchedulerMessage};

#[derive(Debug, Default)]
pub struct LanguageCenter {
    pub transcoder: Transcoder,
    pub interpreters: InterpreterDirectory,
}

impl LanguageCenter {

    pub fn languages(&self) -> impl Iterator<Item = &str> {
        self.transcoder.available_compilers().chain(self.interpreters.available_interpreters())
    }

    pub fn blocking_process(
        &self, 
        script: &mut Script, 
    ) {
        if script.is_empty() {
            return;
        }
        let lang = script.lang();
        let state = if let Some(compiler) = self.transcoder.get_compiler(lang) {
            let script = script.clone();
            match compiler.compile(script.content(), &script.args) {
                Ok(prog) => 
                    CompilationState::Compiled(prog),
                Err(err) => 
                    CompilationState::Error(err),
            }
        } else if let Some(factory) = self.interpreters.get_factory(lang) {
            let script = script.clone();
            factory.check(&script)
        } else {
            CompilationState::NotCompiled
        };
        script.compiled = state;
    }

    pub fn process_script(
        &self, 
        line_id: usize, 
        frame_id: usize, 
        script: &Script, 
        notifier: Sender<SchedulerMessage>
    ) {
        if script.is_empty() {
            return;
        }
        let lang = script.lang();
        let id = script.id();
        let _ = notifier.send(SchedulerMessage::CompilationUpdate(
            line_id, frame_id, script.id(), CompilationState::Compiling)
        );
        if let Some(compiler) = self.transcoder.get_compiler(lang) {
            let script = script.clone();
            thread::spawn(move || {
                let state = match compiler.compile(script.content(), &script.args) {
                    Ok(prog) => 
                        CompilationState::Compiled(prog),
                    Err(err) => 
                        CompilationState::Error(err),
                };
                let _ = notifier.send(SchedulerMessage::CompilationUpdate(line_id, frame_id, id, state));
            });
        } else if let Some(factory) = self.interpreters.get_factory(lang) {
            let script = script.clone();
            thread::spawn(move || {
                let state = factory.check(&script);
                let _ = notifier.send(SchedulerMessage::CompilationUpdate(line_id, frame_id, id, state));
            });
        } else {
            let _ = notifier.send(SchedulerMessage::CompilationUpdate(
                line_id, frame_id, script.id(), CompilationState::NotCompiled)
            );
        }
    }

    pub fn process_line(&self, line_id: usize, line : &Line, notifier: Sender<SchedulerMessage>) {
        for (frame_id, frame) in line.frames.iter().enumerate() {
            self.process_script(line_id, frame_id, frame.script(), notifier.clone());
        }
    }

    pub fn process_scene(&self, scene : &Scene, notifier: Sender<SchedulerMessage>) {
        for (line_id, line) in scene.lines.iter().enumerate() {
            self.process_line(line_id, line, notifier.clone());
        }
    }

}