use crate::{clock::{NEVER, SyncTime}, compiler::CompilationState, vm::{Instruction, Program, EvaluationContext, event::ConcreteEvent, interpreter::Interpreter}, scene::script::{ReturnInfo, Script}};

pub const DEFAULT_INSTRUCTION_BATCH_SIZE : usize = 16;

#[derive(Debug, Default, Clone)]
pub struct ASMInterpreter {
    prog: Program,
    instruction_index: usize,
    return_stack: Vec<ReturnInfo>,
    /// Optimization: allows to execute in the same iteration at most `instruction_block_size` control instructions
    pub instruction_batch_size: usize
}

impl ASMInterpreter {

    pub fn new(prog : Program) -> Self {
        Self {
            prog, 
            instruction_index: 0, 
            return_stack: Vec::new(), 
            instruction_batch_size: DEFAULT_INSTRUCTION_BATCH_SIZE
        }
    }

    #[inline]
    pub fn current_instruction(&self) -> &Instruction {
        &self.prog[self.instruction_index]
    }

    pub fn execute_control(&mut self, ctx : &mut EvaluationContext) {
        let Instruction::Control(control) = &self.prog[self.instruction_index] else {
            return;
        };
        match control.execute(
            ctx,
            &mut self.return_stack,
            self.instruction_index,
            &self.prog,
        ) {
            ReturnInfo::None => self.instruction_index += 1,
            ReturnInfo::IndexChange(index) => self.instruction_index = index,
            ReturnInfo::RelIndexChange(index_change) => {
                let mut index = self.instruction_index as i64;
                index += index_change;
                if index < 0 {
                    index = 0
                };
                self.instruction_index = index as usize;
            }
            ReturnInfo::ProgChange(index, prog) => {
                self.instruction_index = index;
                self.prog = prog.clone();
            }
        };
    }

}

impl From<Program> for ASMInterpreter  {
    fn from(value: Program) -> Self {
        ASMInterpreter::new(value)
    }
}

impl Interpreter for ASMInterpreter {

    fn execute_next(
        &mut self,
        ctx : &mut EvaluationContext
    ) -> (Option<ConcreteEvent>, SyncTime) {
        for _ in 0..self.instruction_batch_size {
            if self.has_terminated() {
                return (None, NEVER);
            }
            let current = &self.prog[self.instruction_index];
            match current {
                Instruction::Control(_) => self.execute_control(ctx),
                Instruction::Effect(event, var_time_span) => {
                    self.instruction_index += 1;
                    let wait = ctx
                        .evaluate(var_time_span)
                        .as_dur()
                        .as_micros(ctx.clock, ctx.frame_len);
                    let c_event = event.make_concrete(ctx);
                    // let res = (c_event, self.scheduled_time);
                    // self.scheduled_time += wait;
                    return (Some(c_event), wait)
                }
            }
        }
        (None, 0)
    }

    #[inline]
    fn stop(&mut self) {
        self.instruction_index = usize::MAX;
    }

    #[inline]
    fn has_terminated(&self) -> bool {
        self.instruction_index >= self.prog.len()
    }

}

#[derive(Debug, Default)]
pub struct ASMInterpreterFactory;

/// Does not behave the same as the factory trait, as it needs to pass forward the language name to the transcoder
impl ASMInterpreterFactory {

    pub fn make_instance(&self, script : &Script) -> Option<Box<dyn Interpreter>> {
        match &script.compiled {
            CompilationState::Compiled(prog) => Some(Box::new(ASMInterpreter::new(prog.clone()))),
            _ => None
        }
    }

}