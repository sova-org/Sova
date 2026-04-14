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
        Some(Self {
            regex,
            group_names: names,
            group_categories: categories,
        })
    }

    pub fn tokenize<'a>(
        &'a self,
        text: &'a str,
    ) -> impl Iterator<Item = (Range<usize>, TokenCategory)> + 'a {
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

// Rows: OneDark, Solarized, Phosphor, Dracula, Monokai, Gruvbox, Nord, Catppuccin, TokyoNight.
// Columns: Keyword, Builtin, Operator, Number, String, Comment, Variable, Symbol, Special, Punctuation.
const THEME_COLORS: [[[u8; 3]; TokenCategory::COUNT]; 9] = [
    [[198, 120, 221], [97, 175, 239],  [86, 182, 194],  [209, 154, 102], [152, 195, 121], [92, 99, 112],   [229, 192, 123], [224, 148, 120], [224, 208, 120], [140, 140, 140]],
    [[181, 137, 0],   [38, 139, 210],  [133, 153, 0],   [211, 54, 130],  [42, 161, 152],  [88, 110, 117],  [203, 75, 22],   [108, 113, 196], [220, 50, 47],   [101, 123, 131]],
    [[80, 255, 80],   [57, 210, 57],   [40, 150, 40],   [80, 255, 80],   [140, 230, 80],  [30, 100, 30],   [80, 255, 80],   [120, 220, 60],  [220, 255, 220], [30, 100, 30]],
    [[189, 147, 249], [80, 250, 123],  [255, 121, 198], [139, 233, 253], [241, 250, 140], [98, 114, 164],  [255, 184, 108], [255, 85, 85],   [255, 255, 128], [148, 148, 148]],
    [[249, 38, 114],  [102, 217, 239], [249, 38, 114],  [174, 129, 255], [230, 219, 116], [117, 113, 94],  [166, 226, 46],  [253, 151, 31],  [248, 248, 242], [136, 136, 136]],
    [[251, 73, 52],   [142, 192, 124], [254, 128, 25],  [211, 134, 155], [184, 187, 38],  [146, 131, 116], [250, 189, 47],  [131, 165, 152], [254, 128, 25],  [168, 153, 132]],
    [[129, 161, 193], [136, 192, 208], [143, 188, 187], [180, 142, 173], [163, 190, 140], [76, 86, 106],   [216, 222, 233], [208, 135, 112], [235, 203, 139], [107, 112, 137]],
    [[203, 166, 247], [137, 180, 250], [137, 220, 235], [250, 179, 135], [166, 227, 161], [108, 112, 134], [249, 226, 175], [242, 205, 205], [245, 194, 231], [147, 153, 178]],
    [[157, 124, 216], [122, 162, 247], [137, 221, 255], [255, 158, 100], [158, 206, 106], [86, 95, 137],   [224, 175, 104], [187, 154, 247], [247, 118, 142], [105, 112, 145]],
];

// Guard: if the enum gains a variant or is reordered, this will fail at compile time.
const _: () = assert!(SyntaxThemePref::TokyoNight as usize == 8);

pub struct SyntaxTheme {
    colors: [Color32; TokenCategory::COUNT],
}

impl SyntaxTheme {
    pub fn from_pref(pref: SyntaxThemePref) -> Self {
        let rgbs = &THEME_COLORS[pref as usize];
        Self {
            colors: std::array::from_fn(|i| Color32::from_rgb(rgbs[i][0], rgbs[i][1], rgbs[i][2])),
        }
    }

    pub fn color(&self, cat: TokenCategory) -> Color32 {
        self.colors[cat as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Snapshot the first color (Keyword) of every theme. If the THEME_COLORS table row order
    // drifts from the SyntaxThemePref enum order, at least one assertion here will catch it.
    #[test]
    fn theme_first_colors_match_originals() {
        let cases: &[(SyntaxThemePref, [u8; 3])] = &[
            (SyntaxThemePref::OneDark,     [198, 120, 221]),
            (SyntaxThemePref::Solarized,   [181, 137,   0]),
            (SyntaxThemePref::Phosphor,    [ 80, 255,  80]),
            (SyntaxThemePref::Dracula,     [189, 147, 249]),
            (SyntaxThemePref::Monokai,     [249,  38, 114]),
            (SyntaxThemePref::Gruvbox,     [251,  73,  52]),
            (SyntaxThemePref::Nord,        [129, 161, 193]),
            (SyntaxThemePref::Catppuccin,  [203, 166, 247]),
            (SyntaxThemePref::TokyoNight,  [157, 124, 216]),
        ];
        for (pref, [r, g, b]) in cases {
            let theme = SyntaxTheme::from_pref(*pref);
            let c = theme.colors[0];
            assert_eq!(
                (c.r(), c.g(), c.b()),
                (*r, *g, *b),
                "keyword color mismatch for theme index {}",
                *pref as usize
            );
        }
    }

    #[test]
    fn all_themes_have_full_color_array() {
        use SyntaxThemePref::*;
        for pref in [OneDark, Solarized, Phosphor, Dracula, Monokai, Gruvbox, Nord, Catppuccin, TokyoNight] {
            let theme = SyntaxTheme::from_pref(pref);
            assert_eq!(theme.colors.len(), TokenCategory::COUNT);
        }
    }
}
