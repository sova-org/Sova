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

impl TokenCategory {
    pub const COUNT: usize = 10;
    pub const ALL: [TokenCategory; Self::COUNT] = [
        TokenCategory::Keyword, TokenCategory::Builtin, TokenCategory::Operator,
        TokenCategory::Number, TokenCategory::String, TokenCategory::Comment,
        TokenCategory::Variable, TokenCategory::Symbol, TokenCategory::Special,
        TokenCategory::Punctuation,
    ];
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntaxRule {
    pub category: TokenCategory,
    pub pattern: String,
}

impl SyntaxRule {
    pub fn new(category: TokenCategory, pattern: &str) -> Self {
        Self { category, pattern: pattern.to_owned() }
    }
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
    pub signature: Option<String>,
    pub example: Option<String>,
    pub category: Option<String>,
    pub aliases: Vec<String>,
}

impl ReferenceEntry {
    pub fn new(description: impl Into<String>) -> Self {
        Self { description: description.into(), signature: None, example: None, category: None, aliases: Vec::new() }
    }
    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = Some(signature.into());
        self
    }
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.example = Some(example.into());
        self
    }
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
    pub fn with_aliases(mut self, aliases: &[&str]) -> Self {
        self.aliases = aliases.iter().map(|s| s.to_string()).collect();
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

impl LanguageDocumentation {

    pub fn is_empty(&self) -> bool {
        self.articles.is_empty() && self.reference.is_empty()
    }

}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LanguageDefinition {
    pub name: String, 
    pub documentation: LanguageDocumentation, 
    pub syntax: Option<LanguageSyntax> 
}

pub trait Language {

    fn name(&self) -> &str;

    fn version(&self) -> (usize, usize, usize);

    fn documentation(&self) -> LanguageDocumentation { Default::default() }

    fn syntax(&self) -> Option<LanguageSyntax> { None }

    fn definition(&self) -> LanguageDefinition {
        LanguageDefinition { 
            name: self.name().to_owned(), 
            documentation: self.documentation(), 
            syntax: self.syntax()
        }
    }

}
