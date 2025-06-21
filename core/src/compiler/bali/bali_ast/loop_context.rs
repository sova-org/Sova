#[derive(Debug, Clone)]
pub struct LoopContext {
    pub negate: bool,
    pub reverse: bool,
    pub shift: Option<i64>,
    pub step_time: bool,
}

impl LoopContext {
    pub fn new() -> LoopContext {
        LoopContext {
            negate: false,
            reverse: false,
            shift: None,
            step_time: false,
        }
    }

    pub fn update(self, above: LoopContext) -> LoopContext {
        let mut b = LoopContext::new();
        b.negate = self.negate || above.negate;
        b.reverse = self.reverse || above.reverse;
        b.shift = match self.shift {
            Some(_) => self.shift,
            None => above.shift,
        };
        b.step_time = self.step_time || above.step_time;
        b
    }
}
