use sova_core::vm::language::{LanguageSyntax, SyntaxRule, TokenCategory};

pub fn syntax() -> LanguageSyntax {
    use TokenCategory::*;
    LanguageSyntax {
        rules: vec![
            SyntaxRule::new(Comment, r"//[^\n]*"),
            SyntaxRule::new(Comment, r"/\*[^*]*\*+(?:[^/*][^*]*\*+)*/"),
            SyntaxRule::new(Number, r"\b\d+(\.\d+)?\b"),
            SyntaxRule::new(
                Builtin,
                r"\b(osc|noise|voronoi|shape|gradient|solid|rings|checker|src|text)\b",
            ),
            SyntaxRule::new(
                Keyword,
                r"\b(add|mult|blend|diff|layer|mask|sub|modulate|modulateScale|modulateRotate|modulateRepeat|modulateRepeatX|modulateRepeatY|modulateKaleid|modulateScrollX|modulateScrollY|modulatePixelate|modulateHue)\b",
            ),
            SyntaxRule::new(
                Operator,
                r"\b(rotate|scale|scroll|kaleid|pixelate|repeat|scrollX|scrollY|repeatX|repeatY|polar|cart|fold)\b",
            ),
            SyntaxRule::new(
                Variable,
                r"\b(color|invert|contrast|brightness|saturate|hue|posterize|luma|colorama|shift|thresh)\b",
            ),
            SyntaxRule::new(
                Special,
                r"\b(out|r|g|b|render|o0|o1|o2|o3|time|beat|tempo|phase|fast|smooth)\b",
            ),
            SyntaxRule::new(
                Symbol,
                r"\b(let|const|if|else|while|loop|for|in|fn|return|true|false)\b",
            ),
            SyntaxRule::new(Operator, r"[+\-*/%]=?|[=!<>]=|&&|\|\||!"),
            SyntaxRule::new(Punctuation, r"[.(),;]"),
        ],
    }
}
