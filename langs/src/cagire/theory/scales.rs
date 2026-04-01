pub struct Scale {
    pub name: &'static str,
    pub pattern: &'static [i64],
}

pub static SCALES: &[Scale] = &[
    Scale {
        name: "major",
        pattern: &[0, 2, 4, 5, 7, 9, 11],
    },
    Scale {
        name: "minor",
        pattern: &[0, 2, 3, 5, 7, 8, 10],
    },
    Scale {
        name: "dorian",
        pattern: &[0, 2, 3, 5, 7, 9, 10],
    },
    Scale {
        name: "phrygian",
        pattern: &[0, 1, 3, 5, 7, 8, 10],
    },
    Scale {
        name: "lydian",
        pattern: &[0, 2, 4, 6, 7, 9, 11],
    },
    Scale {
        name: "mixolydian",
        pattern: &[0, 2, 4, 5, 7, 9, 10],
    },
    Scale {
        name: "aeolian",
        pattern: &[0, 2, 3, 5, 7, 8, 10],
    },
    Scale {
        name: "locrian",
        pattern: &[0, 1, 3, 5, 6, 8, 10],
    },
    Scale {
        name: "pentatonic",
        pattern: &[0, 2, 4, 7, 9],
    },
    Scale {
        name: "minpent",
        pattern: &[0, 3, 5, 7, 10],
    },
    Scale {
        name: "blues",
        pattern: &[0, 3, 5, 6, 7, 10],
    },
    Scale {
        name: "chromatic",
        pattern: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    },
    Scale {
        name: "wholetone",
        pattern: &[0, 2, 4, 6, 8, 10],
    },
    Scale {
        name: "harmonicminor",
        pattern: &[0, 2, 3, 5, 7, 8, 11],
    },
    Scale {
        name: "melodicminor",
        pattern: &[0, 2, 3, 5, 7, 9, 11],
    },
    Scale {
        name: "bebop",
        pattern: &[0, 2, 4, 5, 7, 9, 10, 11],
    },
    Scale {
        name: "bebopmaj",
        pattern: &[0, 2, 4, 5, 7, 8, 9, 11],
    },
    Scale {
        name: "bebopmin",
        pattern: &[0, 2, 3, 5, 7, 8, 9, 10],
    },
    Scale {
        name: "altered",
        pattern: &[0, 1, 3, 4, 6, 8, 10],
    },
    Scale {
        name: "lyddom",
        pattern: &[0, 2, 4, 6, 7, 9, 10],
    },
    Scale {
        name: "halfwhole",
        pattern: &[0, 1, 3, 4, 6, 7, 9, 10],
    },
    Scale {
        name: "wholehalf",
        pattern: &[0, 2, 3, 5, 6, 8, 9, 11],
    },
    Scale {
        name: "augmented",
        pattern: &[0, 3, 4, 7, 8, 11],
    },
    Scale {
        name: "tritone",
        pattern: &[0, 1, 4, 6, 7, 10],
    },
    Scale {
        name: "prometheus",
        pattern: &[0, 2, 4, 6, 9, 10],
    },
    Scale {
        name: "dorianb2",
        pattern: &[0, 1, 3, 5, 7, 9, 10],
    },
    Scale {
        name: "lydianaug",
        pattern: &[0, 2, 4, 6, 8, 9, 11],
    },
    Scale {
        name: "mixb6",
        pattern: &[0, 2, 4, 5, 7, 8, 10],
    },
    Scale {
        name: "locrian2",
        pattern: &[0, 2, 3, 5, 6, 8, 10],
    },
];

pub fn lookup(name: &str) -> Option<&'static [i64]> {
    SCALES.iter().find(|s| s.name == name).map(|s| s.pattern)
}
