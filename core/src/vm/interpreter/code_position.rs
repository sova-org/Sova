#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CodePosition {
    pub line_start: usize,
    pub line_end: Option<usize>,
    pub col_start: Option<usize>,
    pub col_end: Option<usize>
}

impl CodePosition {

    pub fn line(i: usize) -> Self {
        CodePosition { line_start: i, ..Default::default() }
    }
    
}
