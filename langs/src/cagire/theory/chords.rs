pub struct Chord {
    pub name: &'static str,
    pub intervals: &'static [i64],
}

pub static CHORDS: &[Chord] = &[
    Chord { name: "maj", intervals: &[0, 4, 7] },
    Chord { name: "m", intervals: &[0, 3, 7] },
    Chord { name: "dim", intervals: &[0, 3, 6] },
    Chord { name: "aug", intervals: &[0, 4, 8] },
    Chord { name: "sus2", intervals: &[0, 2, 7] },
    Chord { name: "sus4", intervals: &[0, 5, 7] },
    Chord { name: "maj7", intervals: &[0, 4, 7, 11] },
    Chord { name: "min7", intervals: &[0, 3, 7, 10] },
    Chord { name: "dom7", intervals: &[0, 4, 7, 10] },
    Chord { name: "dim7", intervals: &[0, 3, 6, 9] },
    Chord { name: "m7b5", intervals: &[0, 3, 6, 10] },
    Chord { name: "minmaj7", intervals: &[0, 3, 7, 11] },
    Chord { name: "aug7", intervals: &[0, 4, 8, 10] },
    Chord { name: "maj6", intervals: &[0, 4, 7, 9] },
    Chord { name: "min6", intervals: &[0, 3, 7, 9] },
    Chord { name: "dom9", intervals: &[0, 4, 7, 10, 14] },
    Chord { name: "maj9", intervals: &[0, 4, 7, 11, 14] },
    Chord { name: "min9", intervals: &[0, 3, 7, 10, 14] },
    Chord { name: "dom11", intervals: &[0, 4, 7, 10, 14, 17] },
    Chord { name: "min11", intervals: &[0, 3, 7, 10, 14, 17] },
    Chord { name: "dom13", intervals: &[0, 4, 7, 10, 14, 21] },
    Chord { name: "add9", intervals: &[0, 4, 7, 14] },
    Chord { name: "add11", intervals: &[0, 4, 7, 17] },
    Chord { name: "madd9", intervals: &[0, 3, 7, 14] },
    Chord { name: "dom7b9", intervals: &[0, 4, 7, 10, 13] },
    Chord { name: "dom7s9", intervals: &[0, 4, 7, 10, 15] },
    Chord { name: "dom7b5", intervals: &[0, 4, 6, 10] },
    Chord { name: "dom7s5", intervals: &[0, 4, 8, 10] },
];

pub fn lookup(name: &str) -> Option<&'static [i64]> {
    CHORDS.iter().find(|c| c.name == name).map(|c| c.intervals)
}
