use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LanguageSyntax {
    pub tokens: BTreeMap<String, String>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub enum LanguageElement {
    Word(String),
    Brackets(String, String),
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LanguageDocumentation {
    pub articles: Vec<(String, String)>,
    pub reference: BTreeMap<LanguageElement, String>
}

pub trait Language {

    fn name(&self) -> &str;

    fn version(&self) -> (usize, usize, usize);

    fn documentation(&self) -> LanguageDocumentation { Default::default() }

    fn syntax(&self) -> Option<LanguageSyntax> { None }

}
