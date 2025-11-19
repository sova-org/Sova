#[derive(Default)]
pub enum PopupValue {
    #[default]
    None,
    Bool(bool),
    Text(String),
    Float(f64),
    Int(i64),
    Choice(usize, Vec<String>)
}

impl PopupValue {
    pub fn is_float(&self) -> bool {
        match self {
            Self::Float(_) => true,
            _ => false
        }
    }
    pub fn is_bool(&self) -> bool {
        match self {
            Self::Bool(_) => true,
            _ => false
        }
    }
    pub fn is_int(&self) -> bool {
        match self {
            Self::Int(_) => true,
            _ => false
        }
    }
    pub fn is_text(&self) -> bool {
        match self {
            Self::Text(_) => true,
            _ => false
        }
    }
    pub fn is_choice(&self) -> bool {
        match self {
            Self::Choice(_,_) => true,
            _ => false
        }
    }

    fn float(self) -> f64 {
        match self {
            Self::Float(x) => x,
            _ => Default::default()
        }
    }
    fn bool(self) -> bool {
        match self {
            Self::Bool(x) => x,
            _ => Default::default()
        }
    }
    fn int(self) -> i64 {
        match self {
            Self::Int(x) => x,
            _ => Default::default()
        }
    }
    fn text(self) -> String {
        match self {
            Self::Text(x) => x,
            Self::Choice(i, mut v) if i < v.len() => {
                v.remove(i)
            }
            _ => Default::default()
        }
    }
}

impl From<PopupValue> for String {
    fn from(value: PopupValue) -> Self {
        value.text()
    }
}
impl From<PopupValue> for bool {
    fn from(value: PopupValue) -> Self {
        value.bool()
    }
}
impl From<PopupValue> for i64 {
    fn from(value: PopupValue) -> Self {
        value.int()
    }
}
impl From<PopupValue> for f64 {
    fn from(value: PopupValue) -> Self {
        value.float()
    }
}

impl From<bool> for PopupValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<i64> for PopupValue {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}
impl From<f64> for PopupValue {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}
impl From<String> for PopupValue {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}
impl From<Vec<String>> for PopupValue {
    fn from(value: Vec<String>) -> Self {
        Self::Choice(0, value)
    }
}