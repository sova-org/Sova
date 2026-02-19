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
    Dracula,
    Monokai,
    Gruvbox,
    Nord,
    Catppuccin,
    TokyoNight,
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
            SyntaxThemePref::Dracula => Self::dracula(),
            SyntaxThemePref::Monokai => Self::monokai(),
            SyntaxThemePref::Gruvbox => Self::gruvbox(),
            SyntaxThemePref::Nord => Self::nord(),
            SyntaxThemePref::Catppuccin => Self::catppuccin(),
            SyntaxThemePref::TokyoNight => Self::tokyo_night(),
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

    fn dracula() -> Self {
        Self {
            colors: [
                Color32::from_rgb(189, 147, 249), // Keyword — purple
                Color32::from_rgb(80, 250, 123),  // Builtin — green
                Color32::from_rgb(255, 121, 198), // Operator — pink
                Color32::from_rgb(139, 233, 253), // Number — cyan
                Color32::from_rgb(241, 250, 140), // String — yellow
                Color32::from_rgb(98, 114, 164),  // Comment — muted blue
                Color32::from_rgb(255, 184, 108), // Variable — orange
                Color32::from_rgb(255, 85, 85),   // Symbol — red
                Color32::from_rgb(255, 255, 128), // Special — bright yellow
                Color32::from_rgb(148, 148, 148), // Punctuation — dim
            ],
        }
    }

    fn monokai() -> Self {
        Self {
            colors: [
                Color32::from_rgb(249, 38, 114),  // Keyword — red-pink
                Color32::from_rgb(102, 217, 239), // Builtin — blue
                Color32::from_rgb(249, 38, 114),  // Operator — red-pink
                Color32::from_rgb(174, 129, 255), // Number — purple
                Color32::from_rgb(230, 219, 116), // String — yellow
                Color32::from_rgb(117, 113, 94),  // Comment — gray-olive
                Color32::from_rgb(166, 226, 46),  // Variable — green
                Color32::from_rgb(253, 151, 31),  // Symbol — orange
                Color32::from_rgb(248, 248, 242), // Special — white
                Color32::from_rgb(136, 136, 136), // Punctuation — dim
            ],
        }
    }

    fn gruvbox() -> Self {
        Self {
            colors: [
                Color32::from_rgb(251, 73, 52),   // Keyword — red
                Color32::from_rgb(142, 192, 124), // Builtin — aqua
                Color32::from_rgb(254, 128, 25),  // Operator — orange
                Color32::from_rgb(211, 134, 155), // Number — purple
                Color32::from_rgb(184, 187, 38),  // String — green
                Color32::from_rgb(146, 131, 116), // Comment — gray
                Color32::from_rgb(250, 189, 47),  // Variable — yellow
                Color32::from_rgb(131, 165, 152), // Symbol — blue
                Color32::from_rgb(254, 128, 25),  // Special — bright orange
                Color32::from_rgb(168, 153, 132), // Punctuation — fg3
            ],
        }
    }

    fn nord() -> Self {
        Self {
            colors: [
                Color32::from_rgb(129, 161, 193), // Keyword — blue
                Color32::from_rgb(136, 192, 208), // Builtin — cyan
                Color32::from_rgb(143, 188, 187), // Operator — frost
                Color32::from_rgb(180, 142, 173), // Number — purple
                Color32::from_rgb(163, 190, 140), // String — green
                Color32::from_rgb(76, 86, 106),   // Comment — dark gray
                Color32::from_rgb(216, 222, 233), // Variable — snowstorm
                Color32::from_rgb(208, 135, 112), // Symbol — orange
                Color32::from_rgb(235, 203, 139), // Special — yellow
                Color32::from_rgb(107, 112, 137), // Punctuation — dim
            ],
        }
    }

    fn catppuccin() -> Self {
        Self {
            colors: [
                Color32::from_rgb(203, 166, 247), // Keyword — mauve
                Color32::from_rgb(137, 180, 250), // Builtin — blue
                Color32::from_rgb(137, 220, 235), // Operator — sky
                Color32::from_rgb(250, 179, 135), // Number — peach
                Color32::from_rgb(166, 227, 161), // String — green
                Color32::from_rgb(108, 112, 134), // Comment — overlay0
                Color32::from_rgb(249, 226, 175), // Variable — yellow
                Color32::from_rgb(242, 205, 205), // Symbol — flamingo
                Color32::from_rgb(245, 194, 231), // Special — pink
                Color32::from_rgb(147, 153, 178), // Punctuation — surface2
            ],
        }
    }

    fn tokyo_night() -> Self {
        Self {
            colors: [
                Color32::from_rgb(157, 124, 216), // Keyword — purple
                Color32::from_rgb(122, 162, 247), // Builtin — blue
                Color32::from_rgb(137, 221, 255), // Operator — cyan
                Color32::from_rgb(255, 158, 100), // Number — orange
                Color32::from_rgb(158, 206, 106), // String — green
                Color32::from_rgb(86, 95, 137),   // Comment — dark blue-gray
                Color32::from_rgb(224, 175, 104), // Variable — yellow
                Color32::from_rgb(187, 154, 247), // Symbol — magenta
                Color32::from_rgb(247, 118, 142), // Special — red
                Color32::from_rgb(105, 112, 145), // Punctuation — dim
            ],
        }
    }

    pub fn color(&self, cat: TokenCategory) -> Color32 {
        self.colors[cat as usize]
    }
}
