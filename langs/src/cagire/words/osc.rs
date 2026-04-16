use super::{Word, WordCompile::*};

pub(super) const WORDS: &[Word] = &[
    Word {
        name: "address",
        aliases: &["addr"],
        category: "OSC",
        stack: "(v.. --)",
        desc: "Set OSC address for raw message",
        example: include_str!("../../../docs/cagire/examples/address.cagire"),
        compile: Param,
        varargs: true,
    },
    Word {
        name: "oscin",
        aliases: &[],
        category: "OSC",
        stack: "(route idx -- val)",
        desc: "Read value from OSC input device",
        example: include_str!("../../../docs/cagire/examples/oscin.cagire"),
        compile: Simple,
        varargs: false,
    },
];
