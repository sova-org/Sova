use std::sync::Arc;

use crate::{clock::SyncTime, lang::{evaluation_context::EvaluationContext, event::ConcreteEvent, interpreter::{Interpreter, InterpreterFactory}, variable::VariableStore, Instruction, Program}, scene::{line::Line, script::ReturnInfo}, transcoder::Transcoder};

pub struct ASMInterpreter {
    prog: Program,
    instruction_index: usize,
    return_stack: Vec<ReturnInfo>,
}

impl ASMInterpreter {

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

impl Interpreter for ASMInterpreter {

    fn execute_next(
        &mut self,
        ctx : &mut EvaluationContext
    ) -> (Option<ConcreteEvent>, Option<SyncTime>) {
        if self.has_terminated() {
            return (None, None);
        }
        let current = &self.prog[self.instruction_index];
        //print!("Executing this instruction: {:?}\n", current);
        match current {
            Instruction::Control(_) => {
                self.execute_control(ctx);
                (None, None)
            }
            Instruction::Effect(event, var_time_span) => {
                self.instruction_index += 1;
                let wait = ctx
                    .evaluate(var_time_span)
                    .as_dur()
                    .as_micros(ctx.clock, ctx.frame_len());
                let c_event = event.make_concrete(ctx);
                // let res = (c_event, self.scheduled_time);
                // self.scheduled_time += wait;
                (Some(c_event), Some(wait))
            }
        }
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

pub struct ASMInterpreterFactory {
    pub transcoder : Transcoder
}

impl ASMInterpreterFactory {

    pub fn new(transcoder : Transcoder) -> Self {
        ASMInterpreterFactory { transcoder }
    }

}

impl InterpreterFactory for ASMInterpreterFactory {

    fn name(&self) -> String {
        todo!()
    }

    fn make_instance(&self, content : String) -> Box<dyn Interpreter> {
        todo!()
    }

}