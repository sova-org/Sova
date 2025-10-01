use super::{Motion, YankType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub count: Option<u32>,
    pub operator: Option<Operator>,
    pub motion: Motion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    Delete,
    Change,
    Yank,
}

impl Operator {
    pub fn from_char(c: char) -> Option<Self> {
        match c {
            'd' => Some(Self::Delete),
            'c' => Some(Self::Change),
            'y' => Some(Self::Yank),
            _ => None,
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Self::Delete => 'd',
            Self::Change => 'c',
            Self::Yank => 'y',
        }
    }

    pub fn yank_type_for_motion(self, motion: Motion) -> YankType {
        match motion {
            Motion::Line(_) | Motion::Top | Motion::Bottom => YankType::Linewise,
            _ => YankType::Characterwise,
        }
    }
}

impl Command {
    pub fn effective_count(&self) -> u32 {
        self.count.unwrap_or(1)
    }

    pub fn is_linewise(&self) -> bool {
        if let Some(operator) = self.operator {
            operator.yank_type_for_motion(self.motion) == YankType::Linewise
        } else {
            false
        }
    }
}
