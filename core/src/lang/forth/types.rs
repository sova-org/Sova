pub type ForthValue = f64;

#[derive(Clone)]
pub enum Word {
    Builtin(BuiltinWord),
    UserDefined(Vec<String>),
}

#[derive(Clone, Copy)]
pub struct BuiltinWord(pub fn(&mut ForthState));

pub struct ForthState {
    pub data_stack: Vec<ForthValue>,
    pub return_stack: Vec<usize>,
}

impl Default for ForthState {
    fn default() -> Self {
        Self {
            data_stack: Vec::new(),
            return_stack: Vec::new(),
        }
    }
}

impl ForthState {
    pub fn push(&mut self, val: ForthValue) {
        self.data_stack.push(val);
    }

    pub fn pop(&mut self) -> ForthValue {
        self.data_stack.pop().unwrap_or(0.0)
    }

    pub fn peek(&self) -> ForthValue {
        self.data_stack.last().copied().unwrap_or(0.0)
    }
}
