use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TokenCategory {
    Keyword,
    Builtin,
    Operator,
    Number,
    String,
    Comment,
    Variable,
    Symbol,
    Special,
    Punctuation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxRule {
    pub category: TokenCategory,
    pub pattern: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LanguageSyntax {
    pub rules: Vec<SyntaxRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum LanguageElement {
    Word(String),
    Brackets(String, String),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReferenceEntry {
    pub description: String,
    pub example: Option<String>,
}

impl ReferenceEntry {
    pub fn new(description: impl Into<String>) -> Self {
        Self { description: description.into(), example: None }
    }
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.example = Some(example.into());
        self
    }
}

impl From<String> for ReferenceEntry {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for ReferenceEntry {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LanguageDocumentation {
    pub articles: Vec<(String, String)>,
    pub reference: BTreeMap<LanguageElement, ReferenceEntry>,
    pub escape: Vec<(String, String)>
}

pub trait Language {

    fn name(&self) -> &str;

    fn version(&self) -> (usize, usize, usize);

    fn documentation(&self) -> LanguageDocumentation { Default::default() }

    fn syntax(&self) -> Option<LanguageSyntax> { None }

}
