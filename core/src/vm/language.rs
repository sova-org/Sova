use std::collections::BTreeMap;

pub struct LanguageSyntax {
    pub tokens: BTreeMap<String, String>
}

pub trait Language {

    fn documentation(&self) -> BTreeMap<String, String> { Default::default() }

    fn syntax(&self) -> Option<LanguageSyntax> { None }

}
