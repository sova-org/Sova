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
];
