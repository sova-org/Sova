mod compile;
mod core;
mod effects;
mod midi;
mod music;
mod osc;
mod sequencing;
mod sound;

use std::collections::HashMap;
use std::sync::LazyLock;

pub(crate) use compile::compile_word;

#[derive(Clone, Copy)]
pub enum WordCompile {
    Simple,
    BuiltinScale(&'static [usize]),
    BuiltinChordQuality(&'static [i64]),
    Context(&'static str),
    Param,
    Probability(f64),
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct Word {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub category: &'static str,
    pub stack: &'static str,
    pub desc: &'static str,
    pub example: &'static str,
    pub compile: WordCompile,
    pub varargs: bool,
}

pub static WORDS: LazyLock<Vec<Word>> = LazyLock::new(|| {
    let mut words = Vec::new();
    words.extend_from_slice(self::core::WORDS);
    words.extend_from_slice(sound::WORDS);
    words.extend_from_slice(effects::WORDS);
    words.extend_from_slice(sequencing::WORDS);
    words.extend_from_slice(music::WORDS.as_slice());
    words.extend_from_slice(midi::WORDS);
    words.extend_from_slice(osc::WORDS);
    words
});

static WORD_MAP: LazyLock<HashMap<&'static str, &'static Word>> = LazyLock::new(|| {
    let mut map = HashMap::with_capacity(WORDS.len() * 2);
    for word in WORDS.iter() {
        map.insert(word.name, word);
        for alias in word.aliases {
            map.insert(alias, word);
        }
    }
    map
});

pub fn lookup_word(name: &str) -> Option<&'static Word> {
    WORD_MAP.get(name).copied()
}
