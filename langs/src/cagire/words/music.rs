use std::sync::LazyLock;

use crate::cagire::theory;

use super::{Word, WordCompile::*};

const SCALE_EXAMPLE: &str = include_str!("../../../docs/cagire/examples/deg.cagire");
const CHORD_EXAMPLE: &str = include_str!("../../../docs/cagire/examples/chord.cagire");

const BASE_WORDS: &[Word] = &[
    Word {
        name: "mtof",
        aliases: &[],
        category: "Music",
        stack: "(midi -- hz)",
        desc: "MIDI note to frequency",
        example: include_str!("../../../docs/cagire/examples/mtof.cagire"),
        compile: Simple,
        varargs: false,
    },
    Word {
        name: "ftom",
        aliases: &[],
        category: "Music",
        stack: "(hz -- midi)",
        desc: "Frequency to MIDI note",
        example: include_str!("../../../docs/cagire/examples/ftom.cagire"),
        compile: Simple,
        varargs: false,
    },
    Word {
        name: "edo",
        aliases: &[],
        category: "Scale",
        stack: "(n -- tuning)",
        desc: "Build an equal-division tuning over 2/1",
        example: include_str!("../../../docs/cagire/examples/edo.cagire"),
        compile: Simple,
        varargs: false,
    },
    Word {
        name: "tuning",
        aliases: &[],
        category: "Scale",
        stack: "([c1 c2 ...] period -- tuning)",
        desc: "Build a tuning from cents offsets within a period",
        example: include_str!("../../../docs/cagire/examples/tuning.cagire"),
        compile: Simple,
        varargs: false,
    },
    Word {
        name: "scale",
        aliases: &[],
        category: "Scale",
        stack: "([i1 i2 ...] tuning -- scale)",
        desc: "Build a scale by selecting tuning step indices",
        example: include_str!("../../../docs/cagire/examples/scale.cagire"),
        compile: Simple,
        varargs: false,
    },
    Word {
        name: "mode",
        aliases: &[],
        category: "Scale",
        stack: "(n scale -- scale)",
        desc: "Rotate a scale's degree ordering",
        example: include_str!("../../../docs/cagire/examples/mode.cagire"),
        compile: Simple,
        varargs: false,
    },
    Word {
        name: "deg",
        aliases: &[],
        category: "Scale",
        stack: "(root scale degree -- hz)",
        desc: "Resolve a scale degree to frequency",
        example: include_str!("../../../docs/cagire/examples/deg.cagire"),
        compile: Simple,
        varargs: false,
    },
    Word {
        name: "chord",
        aliases: &[],
        category: "Chord",
        stack: "(quality --)",
        desc: "Set active chord quality for note playback",
        example: CHORD_EXAMPLE,
        compile: Simple,
        varargs: false,
    },
];

pub(super) static WORDS: LazyLock<Vec<Word>> = LazyLock::new(|| {
    let mut words = Vec::with_capacity(
        BASE_WORDS.len() + theory::scales::SCALES.len() + theory::chords::CHORDS.len(),
    );
    words.extend_from_slice(BASE_WORDS);
    for scale in theory::scales::SCALES {
        words.push(Word {
            name: scale.name,
            aliases: &[],
            category: "Scale",
            stack: "(-- scale)",
            desc: "Built-in 12-EDO scale",
            example: SCALE_EXAMPLE,
            compile: BuiltinScale(scale.degrees),
            varargs: false,
        });
    }
    for chord in theory::chords::CHORDS {
        words.push(Word {
            name: chord.name,
            aliases: &[],
            category: "Chord",
            stack: "(-- quality)",
            desc: chord.desc,
            example: CHORD_EXAMPLE,
            compile: BuiltinChordQuality(chord.intervals),
            varargs: false,
        });
    }
    words
});
