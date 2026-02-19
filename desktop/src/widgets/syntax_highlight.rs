use std::ops::Range;

use eframe::egui::Color32;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sova_core::vm::language::{LanguageSyntax, TokenCategory};

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
pub enum SyntaxThemePref {
    #[default]
    OneDark,
    Solarized,
    Phosphor,
}

pub struct CompiledSyntax {
    regex: Regex,
    group_names: Vec<String>,
    group_categories: Vec<TokenCategory>,
}

impl CompiledSyntax {
    pub fn new(syntax: &LanguageSyntax) -> Option<Self> {
        if syntax.rules.is_empty() {
            return None;
        }
        let mut categories = Vec::with_capacity(syntax.rules.len());
        let mut names = Vec::with_capacity(syntax.rules.len());
        let mut parts = Vec::with_capacity(syntax.rules.len());
        for (i, rule) in syntax.rules.iter().enumerate() {
            let name = format!("g{i}");
            parts.push(format!("(?P<{name}>{})", rule.pattern));
            names.push(name);
            categories.push(rule.category);
        }
        let combined = parts.join("|");
        let regex = Regex::new(&combined).ok()?;
        Some(Self { regex, group_names: names, group_categories: categories })
    }

    pub fn tokenize<'a>(&'a self, text: &'a str) -> impl Iterator<Item = (Range<usize>, TokenCategory)> + 'a {
        self.regex.captures_iter(text).filter_map(|caps| {
            for (i, cat) in self.group_categories.iter().enumerate() {
                if let Some(m) = caps.name(&self.group_names[i]) {
                    return Some((m.start()..m.end(), *cat));
                }
            }
            None
        })
    }
}

pub struct SyntaxTheme {
    colors: [Color32; TokenCategory::COUNT],
}

impl SyntaxTheme {
    pub fn from_pref(pref: SyntaxThemePref) -> Self {
        match pref {
            SyntaxThemePref::OneDark => Self::one_dark(),
            SyntaxThemePref::Solarized => Self::solarized(),
            SyntaxThemePref::Phosphor => Self::phosphor(),
        }
    }

    fn one_dark() -> Self {
        Self {
            colors: [
                Color32::from_rgb(198, 120, 221), // Keyword — purple
                Color32::from_rgb(97, 175, 239),  // Builtin — blue
                Color32::from_rgb(86, 182, 194),  // Operator — cyan
                Color32::from_rgb(209, 154, 102), // Number — orange
                Color32::from_rgb(152, 195, 121), // String — green
                Color32::from_rgb(92, 99, 112),   // Comment — gray
                Color32::from_rgb(229, 192, 123), // Variable — gold
                Color32::from_rgb(224, 148, 120), // Symbol — salmon
                Color32::from_rgb(224, 208, 120), // Special — bright yellow
                Color32::from_rgb(140, 140, 140), // Punctuation — dim
            ],
        }
    }

    fn solarized() -> Self {
        Self {
            colors: [
                Color32::from_rgb(181, 137, 0),   // Keyword — yellow
                Color32::from_rgb(38, 139, 210),   // Builtin — blue
                Color32::from_rgb(133, 153, 0),    // Operator — green
                Color32::from_rgb(211, 54, 130),   // Number — magenta
                Color32::from_rgb(42, 161, 152),   // String — cyan
                Color32::from_rgb(88, 110, 117),   // Comment — base01 (dim)
                Color32::from_rgb(203, 75, 22),    // Variable — orange
                Color32::from_rgb(108, 113, 196),  // Symbol — violet
                Color32::from_rgb(220, 50, 47),    // Special — red
                Color32::from_rgb(101, 123, 131),  // Punctuation — base00
            ],
        }
    }

    fn phosphor() -> Self {
        Self {
            colors: [
                Color32::from_rgb(80, 255, 80),    // Keyword — bright green
                Color32::from_rgb(57, 210, 57),    // Builtin — green
                Color32::from_rgb(40, 150, 40),    // Operator — dim green
                Color32::from_rgb(80, 255, 80),    // Number — bright green
                Color32::from_rgb(140, 230, 80),   // String — green-yellow
                Color32::from_rgb(30, 100, 30),    // Comment — dark green
                Color32::from_rgb(80, 255, 80),    // Variable — bright green
                Color32::from_rgb(120, 220, 60),   // Symbol — yellow-green
                Color32::from_rgb(220, 255, 220),  // Special — white
                Color32::from_rgb(30, 100, 30),    // Punctuation — dark green
            ],
        }
    }

    pub fn color(&self, cat: TokenCategory) -> Color32 {
        self.colors[cat as usize]
    }
}
