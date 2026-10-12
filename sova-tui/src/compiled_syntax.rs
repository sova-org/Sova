use regex::Regex;
use sova_core::vm::language::{LanguageDefinition, SyntaxRule, TokenCategory};

use crate::event::{AppEvent, EventSender};

#[derive(Debug, Clone)]
pub struct CompiledSyntax {
    regex: Regex,
    pub rules: Vec<SyntaxRule>
}

impl CompiledSyntax {

    pub fn split(&self, text: &str) -> Vec<(String, Option<TokenCategory>)> {
        let mut tokens = Vec::new();
        let mut begin = 0;
        while begin < text.len() && let Some(cap) = self.regex.captures_at(text, begin) {
            let start = cap.get_match().start();
            if start > begin {
                let to_add = text[begin..start].to_string();
                tokens.push((to_add, None));
            }
            begin = cap.get_match().end();
            for (i, rule) in self.rules.iter().enumerate() {
                if let Some(group) = cap.name(&format!("rule{i}")) {
                    tokens.push((group.as_str().to_string(), Some(rule.category)));
                }
            }
        }
        if begin < text.len() {
            tokens.push((text[begin..].to_string(), None));
        }
        tokens
    }

}

pub fn start_syntax_compilation(langs: &Vec<LanguageDefinition>, mut sender: EventSender) {
    let langs : Vec<LanguageDefinition> = langs.iter().map(|lang| {
        LanguageDefinition { name: lang.name.clone(), documentation: Default::default(), syntax: lang.syntax.clone() }
    }).collect();
    tokio::spawn(async move {
        for lang in langs {
            if lang.syntax.rules.is_empty() {
                continue;
            }
            let groups : Vec<String> = lang.syntax.rules
                .iter()
                .enumerate()
                .map(|(i, r)| format!("(?<rule{i}>{})", r.pattern.clone()))
                .collect();
            let groups = groups.join("|");
            if let Ok(regex) = Regex::new(&groups) {
                let compiled = CompiledSyntax {
                    regex,
                    rules: lang.syntax.rules
                };
                sender.send(AppEvent::UpdateSyntax(lang.name, compiled));
            } else {
                sender.send(AppEvent::Negative(format!("Unable to compile syntax for {} !", lang.name)));
            }
        }
    });
}