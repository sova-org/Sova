pub struct Chord {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    #[allow(dead_code)]
    pub desc: &'static str,
    pub intervals: &'static [i64],
}

pub static CHORDS: &[Chord] = &[
    // ----- Triads -----
    Chord {
        name: "maj",
        aliases: &["M", "major"],
        desc: "Major triad",
        intervals: &[0, 4, 7],
    },
    Chord {
        name: "m",
        aliases: &["min", "minor"],
        desc: "Minor triad",
        intervals: &[0, 3, 7],
    },
    Chord {
        name: "dim",
        aliases: &["o"],
        desc: "Diminished triad",
        intervals: &[0, 3, 6],
    },
    Chord {
        name: "aug",
        aliases: &["+"],
        desc: "Augmented triad",
        intervals: &[0, 4, 8],
    },
    Chord {
        name: "sus2",
        aliases: &[],
        desc: "Suspended 2nd",
        intervals: &[0, 2, 7],
    },
    Chord {
        name: "sus4",
        aliases: &["sus"],
        desc: "Suspended 4th",
        intervals: &[0, 5, 7],
    },
    // ----- Power chord -----
    Chord {
        name: "pwr",
        aliases: &["5", "power"],
        desc: "Power chord (root + fifth)",
        intervals: &[0, 7],
    },
    // ----- Sixth chords -----
    Chord {
        name: "maj6",
        aliases: &["6", "M6"],
        desc: "Major 6th",
        intervals: &[0, 4, 7, 9],
    },
    Chord {
        name: "min6",
        aliases: &["m6"],
        desc: "Minor 6th",
        intervals: &[0, 3, 7, 9],
    },
    Chord {
        name: "maj69",
        aliases: &["M69", "6/9"],
        desc: "Major 6/9",
        intervals: &[0, 4, 7, 9, 14],
    },
    Chord {
        name: "min69",
        aliases: &["m69", "m6/9"],
        desc: "Minor 6/9",
        intervals: &[0, 3, 7, 9, 14],
    },
    // ----- Seventh chords -----
    Chord {
        name: "maj7",
        aliases: &["M7", "Δ7"],
        desc: "Major 7th",
        intervals: &[0, 4, 7, 11],
    },
    Chord {
        name: "min7",
        aliases: &["m7"],
        desc: "Minor 7th",
        intervals: &[0, 3, 7, 10],
    },
    Chord {
        name: "dom7",
        aliases: &["7"],
        desc: "Dominant 7th",
        intervals: &[0, 4, 7, 10],
    },
    Chord {
        name: "dim7",
        aliases: &["o7"],
        desc: "Diminished 7th",
        intervals: &[0, 3, 6, 9],
    },
    Chord {
        name: "m7b5",
        aliases: &["min7b5", "h7", "hdim7", "halfdim7", "ø7"],
        desc: "Half-diminished (minor 7 flat 5)",
        intervals: &[0, 3, 6, 10],
    },
    // ----- Minor-major sevenths -----
    Chord {
        name: "minmaj7",
        aliases: &["mM7", "mmaj7"],
        desc: "Minor-major 7th",
        intervals: &[0, 3, 7, 11],
    },
    // ----- Augmented sevenths -----
    Chord {
        name: "aug7",
        aliases: &["+7"],
        desc: "Augmented 7th",
        intervals: &[0, 4, 8, 10],
    },
    Chord {
        name: "augmaj7",
        aliases: &["+M7", "+maj7"],
        desc: "Augmented major 7th",
        intervals: &[0, 4, 8, 11],
    },
    // ----- Ninth chords -----
    Chord {
        name: "dom9",
        aliases: &["9"],
        desc: "Dominant 9th",
        intervals: &[0, 4, 7, 10, 14],
    },
    Chord {
        name: "maj9",
        aliases: &["M9"],
        desc: "Major 9th",
        intervals: &[0, 4, 7, 11, 14],
    },
    Chord {
        name: "min9",
        aliases: &["m9"],
        desc: "Minor 9th",
        intervals: &[0, 3, 7, 10, 14],
    },
    // ----- Eleventh chords -----
    Chord {
        name: "dom11",
        aliases: &["11"],
        desc: "Dominant 11th",
        intervals: &[0, 4, 7, 10, 14, 17],
    },
    Chord {
        name: "maj11",
        aliases: &["M11"],
        desc: "Major 11th",
        intervals: &[0, 4, 7, 11, 14, 17],
    },
    Chord {
        name: "min11",
        aliases: &["m11"],
        desc: "Minor 11th",
        intervals: &[0, 3, 7, 10, 14, 17],
    },
    // ----- Thirteenth chords -----
    Chord {
        name: "dom13",
        aliases: &["13"],
        desc: "Dominant 13th",
        intervals: &[0, 4, 7, 10, 14, 21],
    },
    Chord {
        name: "maj13",
        aliases: &["M13"],
        desc: "Major 13th",
        intervals: &[0, 4, 7, 11, 14, 21],
    },
    Chord {
        name: "min13",
        aliases: &["m13"],
        desc: "Minor 13th",
        intervals: &[0, 3, 7, 10, 14, 21],
    },
    // ----- Add chords -----
    Chord {
        name: "add9",
        aliases: &["add2"],
        desc: "Add 9 (major triad + 9)",
        intervals: &[0, 4, 7, 14],
    },
    Chord {
        name: "add11",
        aliases: &["add4"],
        desc: "Add 11 (major triad + 11)",
        intervals: &[0, 4, 7, 17],
    },
    Chord {
        name: "madd9",
        aliases: &["madd2"],
        desc: "Minor add 9",
        intervals: &[0, 3, 7, 14],
    },
    // ----- Suspended-extended -----
    Chord {
        name: "7sus4",
        aliases: &["sus47"],
        desc: "Dominant 7 suspended 4",
        intervals: &[0, 5, 7, 10],
    },
    Chord {
        name: "9sus4",
        aliases: &["sus49"],
        desc: "Dominant 9 suspended 4",
        intervals: &[0, 5, 7, 10, 14],
    },
    Chord {
        name: "7sus2",
        aliases: &["sus27"],
        desc: "Dominant 7 suspended 2",
        intervals: &[0, 2, 7, 10],
    },
    // ----- Altered dominants -----
    Chord {
        name: "dom7b9",
        aliases: &["7b9"],
        desc: "Dominant 7 flat 9",
        intervals: &[0, 4, 7, 10, 13],
    },
    Chord {
        name: "dom7s9",
        aliases: &["7s9", "7#9"],
        desc: "Dominant 7 sharp 9",
        intervals: &[0, 4, 7, 10, 15],
    },
    Chord {
        name: "dom7b5",
        aliases: &["7b5"],
        desc: "Dominant 7 flat 5",
        intervals: &[0, 4, 6, 10],
    },
    Chord {
        name: "dom7s5",
        aliases: &["7s5", "7#5"],
        desc: "Dominant 7 sharp 5",
        intervals: &[0, 4, 8, 10],
    },
    Chord {
        name: "dom7s11",
        aliases: &["7s11", "7#11"],
        desc: "Dominant 7 sharp 11",
        intervals: &[0, 4, 7, 10, 18],
    },
    Chord {
        name: "dom7b9b5",
        aliases: &["7b9b5"],
        desc: "Dominant 7 flat 9 flat 5",
        intervals: &[0, 4, 6, 10, 13],
    },
    Chord {
        name: "dom7s9b5",
        aliases: &["7s9b5", "7#9b5"],
        desc: "Dominant 7 sharp 9 flat 5",
        intervals: &[0, 4, 6, 10, 15],
    },
    Chord {
        name: "dom7b9s5",
        aliases: &["7b9s5", "7b9#5"],
        desc: "Dominant 7 flat 9 sharp 5",
        intervals: &[0, 4, 8, 10, 13],
    },
    Chord {
        name: "dom7s9s5",
        aliases: &["7s9s5", "7#9#5"],
        desc: "Dominant 7 sharp 9 sharp 5",
        intervals: &[0, 4, 8, 10, 15],
    },
    Chord {
        name: "alt",
        aliases: &["dom7alt", "7alt"],
        desc: "Altered dominant (root, 3, b7, b9, #9)",
        intervals: &[0, 4, 10, 13, 15],
    },
    // ----- Major sharp 11 (Lydian) -----
    Chord {
        name: "maj7s11",
        aliases: &["M7s11", "maj7#11", "M7#11"],
        desc: "Major 7 sharp 11 (Lydian)",
        intervals: &[0, 4, 7, 11, 18],
    },
    Chord {
        name: "maj9s11",
        aliases: &["M9s11", "maj9#11", "M9#11"],
        desc: "Major 9 sharp 11",
        intervals: &[0, 4, 7, 11, 14, 18],
    },
    // ----- Minor-major extensions -----
    Chord {
        name: "minmaj9",
        aliases: &["mM9", "mmaj9"],
        desc: "Minor-major 9th",
        intervals: &[0, 3, 7, 11, 14],
    },
    Chord {
        name: "minmaj11",
        aliases: &["mM11", "mmaj11"],
        desc: "Minor-major 11th",
        intervals: &[0, 3, 7, 11, 14, 17],
    },
    Chord {
        name: "minmaj13",
        aliases: &["mM13", "mmaj13"],
        desc: "Minor-major 13th",
        intervals: &[0, 3, 7, 11, 14, 21],
    },
];

pub fn lookup(name: &str) -> Option<&'static Chord> {
    CHORDS
        .iter()
        .find(|chord| chord.name == name || chord.aliases.iter().any(|alias| *alias == name))
}

pub fn lookup_numeric(alias: i64) -> Option<&'static Chord> {
    match alias {
        5 => lookup("5"),
        6 => lookup("6"),
        7 => lookup("7"),
        9 => lookup("9"),
        11 => lookup("11"),
        13 => lookup("13"),
        _ => None,
    }
}
