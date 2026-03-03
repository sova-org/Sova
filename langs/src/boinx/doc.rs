use sova_core::vm::language::{LanguageDocumentation, LanguageElement, ReferenceEntry};

pub fn make_documentation() -> LanguageDocumentation {
    let mut doc = LanguageDocumentation::default();

    doc.articles.push((
        "Introduction".into(),
        include_str!("../../docs/boinx/intro.md").into(),
    ));
    doc.articles.push((
        "Language Reference".into(),
        include_str!("../../docs/boinx/reference.md").into(),
    ));

    let entry = ReferenceEntry::new("A placeholder.").with_category("Items");
    doc.reference
        .insert(LanguageElement::Word(".".to_owned()), entry);
    let entry = ReferenceEntry::new("A mute.").with_category("Items");
    doc.reference
        .insert(LanguageElement::Word("_".to_owned()), entry);
    let entry = ReferenceEntry::new("An UNORDERED key-value set.")
        .with_example("<s: 'saw' note: C4 lpf: 2000>")
        .with_category("Items");
    doc.reference.insert(
        LanguageElement::Brackets("<".to_owned(), ">".to_owned()),
        entry,
    );
    let entry = ReferenceEntry::new("An evenly spaced sequence in time.")
        .with_example("[C4 E4 G4]")
        .with_category("Sets");
    doc.reference.insert(
        LanguageElement::Brackets("[".to_owned(), "]".to_owned()),
        entry,
    );
    let entry = ReferenceEntry::new("Simultaneous set of elements.")
        .with_example("(C4 E4 G4)")
        .with_category("Sets");
    doc.reference.insert(
        LanguageElement::Brackets("(".to_owned(), ")".to_owned()),
        entry,
    );

    let entry = ReferenceEntry::new(
        "This operator is the simplest: it sends the LHS into every *slot* in the RHS.",
    )
    .with_example("<s: 'kick'> | [..]")
    .with_category("Operators");
    doc.reference
        .insert(LanguageElement::Word("|".to_owned()), entry);

    let entry = ReferenceEntry::new(
        "This operator iterates (cycling) over every item of the LHS and into every *slot* in the RHS, *from left to right*, not regarding about depth."
    ).with_example("(C4 C5) ° [.._[..]]").with_category("Operators");
    doc.reference
        .insert(LanguageElement::Word("°".to_owned()), entry);

    let entry = ReferenceEntry::new(
        "This operator iterates (cycling) over every item of the LHS and the RHS and perform each time a *Compose* operator between the item yielded from the LHS and the one yielded from the RHS."
    ).with_example("[C E A G] ! [(. .+4)(. .+3)(. .+3)(. .+4)]").with_category("Operators");
    doc.reference
        .insert(LanguageElement::Word("!".to_owned()), entry);

    let entry = ReferenceEntry::new(
        "This operator applies a *Compose* operator between each item of the LHS, and the RHS, and replaces each LHS item with the result of its composition."
    ).with_example("[<s: 'kick'> <s: 'hh'>] ~ [..]").with_category("Operators");
    doc.reference
        .insert(LanguageElement::Word("~".to_owned()), entry);

    let entry = ReferenceEntry::new(
        "This operator applies a *Compose* operator between each *atomic item* of the LHS, and the RHS, and replaces each of the LHS items with the result of its composition."
    ).with_example("['bd' 'sn' ['bd' 'bd'] 'sn'] # \"s\"").with_category("Operators");
    doc.reference
        .insert(LanguageElement::Word("#".to_owned()), entry);

    doc
}
