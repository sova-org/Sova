use crossbeam_channel::Sender;

/// A compiler is a trait that defines any piece of software that can compile
/// a textual representation of a program into a program.
use crate::compiler::{CompilationState, Compiler, CompilerCollection};
use crate::scene::script::Script;
use crate::scene::Line;
use crate::schedule::SchedulerMessage;
use crate::{log_eprintln, Scene};
use std::collections::BTreeMap;
use std::sync::Arc;
use std::thread;

fn compilation_job(compiler: &dyn Compiler, script: Script) -> CompilationState {
    match compiler.compile(script.content(), &script.args) {
        Ok(prog) => 
            CompilationState::Compiled(prog),
        Err(err) => 
            CompilationState::Error(err),
    }
}

/// The transcoder is a repository of compilers. It allows to add, remove and
/// compile programs in different languages.
#[derive(Debug, Default)]
pub struct Transcoder {
    pub compilers: CompilerCollection
}

impl Transcoder {
    /// Create a new transcoder with a set of compilers and an active compiler.
    /// If the active compiler is not in the set of compilers, it will be set to None.
    /// If no active compiler is set, the first added compiler will be set as active.
    ///
    /// # Arguments
    ///
    /// * `compilers` - A set of compilers to add to the transcoder.
    ///
    /// # Returns
    ///
    /// A new transcoder with the set of compilers.
    pub fn new(compilers: CompilerCollection) -> Self {
        Self {
            compilers
        }
    }

    /// Add a compiler to the transcoder.
    ///
    /// # Arguments
    ///
    /// * `compiler` - The compiler to add to the transcoder.
    ///
    /// # Returns
    ///
    /// The transcoder with the new compiler added.
    pub fn add_compiler(&mut self, compiler: impl Compiler + 'static) {
        let name : String = compiler.name().into();
        self.compilers.insert(name.clone(), Arc::new(compiler));
    }

    /// Remove a compiler from the transcoder.
    ///
    /// # Arguments
    ///
    /// * `lang` - The language of the compiler to remove.
    ///
    /// # Returns
    ///
    /// The removed compiler, or None if the compiler was not found.
    pub fn remove_compiler(&mut self, lang: &str) -> Option<Arc<dyn Compiler>> {
        self.compilers.remove(lang)
    }

    /// Compile a program from a string.
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the program to compile.
    /// * `lang` - The language of the compiler to use.
    ///
    /// # Returns
    ///
    /// The compiled program, or an error if the compiler was not found or the compilation failed.
    pub fn compile(&self, content: &str, lang: &str, args: &BTreeMap<String, String>) -> CompilationState {
        let Some(compiler) = self.compilers.get(lang) else {
            return CompilationState::NotCompiled;
        };
        match compiler.compile(content, args) {
            Ok(prog) => CompilationState::Compiled(prog),
            Err(err) => CompilationState::Error(err),
        }
    }

    pub fn compile_script(&self, script : &mut Script) -> bool {
        if let CompilationState::Compiled(prog) = self.compile(script.content(), script.lang(), &script.args) {
            script.compiled = CompilationState::Compiled(prog);
            true
        } else {
            log_eprintln!(
                "[!] Scheduler: unable to compile script !"
            );
            false
        }
    }

    /// Returns a list of names of the available compilers.
    pub fn available_compilers(&self) -> impl Iterator<Item = &str> {
        self.compilers.keys().map(String::as_str)
    }

    pub fn has_compiler(&self, lang : &str) -> bool {
        self.compilers.contains_key(lang)
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
        let Some(compiler) = self.compilers.get(script.lang()) else {
            return;
        };
        let id = script.id();
        let _ = notifier.send(SchedulerMessage::CompilationUpdate(
            line_id, frame_id, script.id(), CompilationState::Compiling)
        );
        let compiler = Arc::clone(compiler);
        let script = script.clone();
        thread::spawn(move || {
            let state = compilation_job(&*compiler, script);
            let _ = notifier.send(SchedulerMessage::CompilationUpdate(line_id, frame_id, id, state));
        });
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
