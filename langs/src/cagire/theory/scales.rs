pub struct Scale {
    pub name: &'static str,
    pub degrees: &'static [usize],
}

pub static SCALES: &[Scale] = &[
    Scale {
        name: "major",
        degrees: &[0, 2, 4, 5, 7, 9, 11],
    },
    Scale {
        name: "minor",
        degrees: &[0, 2, 3, 5, 7, 8, 10],
    },
    Scale {
        name: "dorian",
        degrees: &[0, 2, 3, 5, 7, 9, 10],
    },
    Scale {
        name: "phrygian",
        degrees: &[0, 1, 3, 5, 7, 8, 10],
    },
    Scale {
        name: "lydian",
        degrees: &[0, 2, 4, 6, 7, 9, 11],
    },
    Scale {
        name: "mixolydian",
        degrees: &[0, 2, 4, 5, 7, 9, 10],
    },
    Scale {
        name: "aeolian",
        degrees: &[0, 2, 3, 5, 7, 8, 10],
    },
    Scale {
        name: "locrian",
        degrees: &[0, 1, 3, 5, 6, 8, 10],
    },
    Scale {
        name: "pentatonic",
        degrees: &[0, 2, 4, 7, 9],
    },
    Scale {
        name: "minpent",
        degrees: &[0, 3, 5, 7, 10],
    },
    Scale {
        name: "blues",
        degrees: &[0, 3, 5, 6, 7, 10],
    },
    Scale {
        name: "chromatic",
        degrees: &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
    },
    Scale {
        name: "wholetone",
        degrees: &[0, 2, 4, 6, 8, 10],
    },
    Scale {
        name: "harmonicminor",
        degrees: &[0, 2, 3, 5, 7, 8, 11],
    },
    Scale {
        name: "melodicminor",
        degrees: &[0, 2, 3, 5, 7, 9, 11],
    },
    Scale {
        name: "bebop",
        degrees: &[0, 2, 4, 5, 7, 9, 10, 11],
    },
    Scale {
        name: "bebopmaj",
        degrees: &[0, 2, 4, 5, 7, 8, 9, 11],
    },
    Scale {
        name: "bebopmin",
        degrees: &[0, 2, 3, 5, 7, 8, 9, 10],
    },
    Scale {
        name: "altered",
        degrees: &[0, 1, 3, 4, 6, 8, 10],
    },
    Scale {
        name: "lyddom",
        degrees: &[0, 2, 4, 6, 7, 9, 10],
    },
    Scale {
        name: "halfwhole",
        degrees: &[0, 1, 3, 4, 6, 7, 9, 10],
    },
    Scale {
        name: "wholehalf",
        degrees: &[0, 2, 3, 5, 6, 8, 9, 11],
    },
    Scale {
        name: "augmented",
        degrees: &[0, 3, 4, 7, 8, 11],
    },
    Scale {
        name: "tritone",
        degrees: &[0, 1, 4, 6, 7, 10],
    },
    Scale {
        name: "prometheus",
        degrees: &[0, 2, 4, 6, 9, 10],
    },
    Scale {
        name: "dorianb2",
        degrees: &[0, 1, 3, 5, 7, 9, 10],
    },
    Scale {
        name: "lydianaug",
        degrees: &[0, 2, 4, 6, 8, 9, 11],
    },
    Scale {
        name: "mixb6",
        degrees: &[0, 2, 4, 5, 7, 8, 10],
    },
    Scale {
        name: "locrian2",
        degrees: &[0, 2, 3, 5, 6, 8, 10],
    },
];
