pub struct Chord {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub desc: &'static str,
    pub intervals: &'static [i64],
}

pub static CHORDS: &[Chord] = &[
    Chord {
        name: "maj",
        aliases: &[],
        desc: "Major triad quality",
        intervals: &[0, 4, 7],
    },
    Chord {
        name: "m",
        aliases: &[],
        desc: "Minor triad quality",
        intervals: &[0, 3, 7],
    },
    Chord {
        name: "dim",
        aliases: &[],
        desc: "Diminished triad quality",
        intervals: &[0, 3, 6],
    },
    Chord {
        name: "aug",
        aliases: &[],
        desc: "Augmented triad quality",
        intervals: &[0, 4, 8],
    },
    Chord {
        name: "sus2",
        aliases: &[],
        desc: "Suspended 2nd quality",
        intervals: &[0, 2, 7],
    },
    Chord {
        name: "sus4",
        aliases: &[],
        desc: "Suspended 4th quality",
        intervals: &[0, 5, 7],
    },
    Chord {
        name: "maj7",
        aliases: &[],
        desc: "Major 7th quality",
        intervals: &[0, 4, 7, 11],
    },
    Chord {
        name: "min7",
        aliases: &[],
        desc: "Minor 7th quality",
        intervals: &[0, 3, 7, 10],
    },
    Chord {
        name: "dom7",
        aliases: &["7"],
        desc: "Dominant 7th quality",
        intervals: &[0, 4, 7, 10],
    },
    Chord {
        name: "dim7",
        aliases: &[],
        desc: "Diminished 7th quality",
        intervals: &[0, 3, 6, 9],
    },
    Chord {
        name: "m7b5",
        aliases: &[],
        desc: "Half-diminished quality",
        intervals: &[0, 3, 6, 10],
    },
    Chord {
        name: "minmaj7",
        aliases: &[],
        desc: "Minor-major 7th quality",
        intervals: &[0, 3, 7, 11],
    },
    Chord {
        name: "aug7",
        aliases: &[],
        desc: "Augmented 7th quality",
        intervals: &[0, 4, 8, 10],
    },
    Chord {
        name: "maj6",
        aliases: &["6"],
        desc: "Major 6th quality",
        intervals: &[0, 4, 7, 9],
    },
    Chord {
        name: "min6",
        aliases: &[],
        desc: "Minor 6th quality",
        intervals: &[0, 3, 7, 9],
    },
    Chord {
        name: "dom9",
        aliases: &["9"],
        desc: "Dominant 9th quality",
        intervals: &[0, 4, 7, 10, 14],
    },
    Chord {
        name: "maj9",
        aliases: &[],
        desc: "Major 9th quality",
        intervals: &[0, 4, 7, 11, 14],
    },
    Chord {
        name: "min9",
        aliases: &[],
        desc: "Minor 9th quality",
        intervals: &[0, 3, 7, 10, 14],
    },
    Chord {
        name: "dom11",
        aliases: &["11"],
        desc: "Dominant 11th quality",
        intervals: &[0, 4, 7, 10, 14, 17],
    },
    Chord {
        name: "min11",
        aliases: &[],
        desc: "Minor 11th quality",
        intervals: &[0, 3, 7, 10, 14, 17],
    },
    Chord {
        name: "dom13",
        aliases: &["13"],
        desc: "Dominant 13th quality",
        intervals: &[0, 4, 7, 10, 14, 21],
    },
    Chord {
        name: "add9",
        aliases: &[],
        desc: "Add 9 quality",
        intervals: &[0, 4, 7, 14],
    },
    Chord {
        name: "add11",
        aliases: &[],
        desc: "Add 11 quality",
        intervals: &[0, 4, 7, 17],
    },
    Chord {
        name: "madd9",
        aliases: &[],
        desc: "Minor add 9 quality",
        intervals: &[0, 3, 7, 14],
    },
    Chord {
        name: "dom7b9",
        aliases: &[],
        desc: "Dominant 7th flat 9 quality",
        intervals: &[0, 4, 7, 10, 13],
    },
    Chord {
        name: "dom7s9",
        aliases: &[],
        desc: "Dominant 7th sharp 9 quality",
        intervals: &[0, 4, 7, 10, 15],
    },
    Chord {
        name: "dom7b5",
        aliases: &[],
        desc: "Dominant 7th flat 5 quality",
        intervals: &[0, 4, 6, 10],
    },
    Chord {
        name: "dom7s5",
        aliases: &[],
        desc: "Dominant 7th sharp 5 quality",
        intervals: &[0, 4, 8, 10],
    },
    Chord {
        name: "pwr",
        aliases: &[],
        desc: "Power chord quality",
        intervals: &[0, 7],
    },
    Chord {
        name: "7sus4",
        aliases: &[],
        desc: "Suspended 4th 7th quality",
        intervals: &[0, 5, 7, 10],
    },
    Chord {
        name: "9sus4",
        aliases: &[],
        desc: "Suspended 4th 9th quality",
        intervals: &[0, 5, 7, 10, 14],
    },
    Chord {
        name: "augmaj7",
        aliases: &[],
        desc: "Augmented major 7th quality",
        intervals: &[0, 4, 8, 11],
    },
    Chord {
        name: "maj69",
        aliases: &[],
        desc: "Major 6/9 quality",
        intervals: &[0, 4, 7, 9, 14],
    },
    Chord {
        name: "min69",
        aliases: &[],
        desc: "Minor 6/9 quality",
        intervals: &[0, 3, 7, 9, 14],
    },
    Chord {
        name: "maj11",
        aliases: &[],
        desc: "Major 11th quality",
        intervals: &[0, 4, 7, 11, 14, 17],
    },
    Chord {
        name: "maj13",
        aliases: &[],
        desc: "Major 13th quality",
        intervals: &[0, 4, 7, 11, 14, 21],
    },
    Chord {
        name: "min13",
        aliases: &[],
        desc: "Minor 13th quality",
        intervals: &[0, 3, 7, 10, 14, 21],
    },
    Chord {
        name: "dom7s11",
        aliases: &[],
        desc: "Dominant 7th sharp 11 quality",
        intervals: &[0, 4, 7, 10, 18],
    },
];

pub fn lookup(name: &str) -> Option<&'static Chord> {
    CHORDS
        .iter()
        .find(|chord| chord.name == name || chord.aliases.iter().any(|alias| *alias == name))
}

pub fn lookup_numeric(alias: i64) -> Option<&'static Chord> {
    match alias {
        6 => lookup("6"),
        7 => lookup("7"),
        9 => lookup("9"),
        11 => lookup("11"),
        13 => lookup("13"),
        _ => None,
    }
}
