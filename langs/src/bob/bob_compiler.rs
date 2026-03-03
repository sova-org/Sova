//! Compiler for the Bob language.
//!
//! Transforms Bob AST into Sova VM bytecode instructions.
//! Bob is purely expression-oriented - everything is an expression.

use crate::bob::bob_ast::{BobExpr, BobProgram};
use crate::bob::bob_grammar;
use crate::bob::compile_expr::compile_expr;
use crate::bob::context::CompileContext;
use lalrpop_util::ParseError;
use sova_core::compiler::{CompilationError, Compiler};
use sova_core::vm::language::LanguageSyntax;
use sova_core::vm::{Language, Program};
use std::collections::BTreeMap;

// ============================================================================
// Compiler
// ============================================================================

#[derive(Debug)]
pub struct BobCompiler;

impl Language for BobCompiler {
    fn name(&self) -> &str {
        "bob"
    }
    fn version(&self) -> (usize, usize, usize) {
        (1, 0, 0)
    }
    fn documentation(&self) -> sova_core::vm::language::LanguageDocumentation {
        use sova_core::vm::language::{LanguageDocumentation, LanguageElement::*, ReferenceEntry};
        let mut doc = LanguageDocumentation::default();

        // -- Output & Timing --
        doc.reference.insert(
            Word("PLAY".into()),
            ReferenceEntry::new(
                "Emit an event. The map keys determine event type (note, cc, addr, etc).",
            )
            .with_category("Output & Timing")
            .with_aliases(&[">>", "@"])
            .with_example(">> [note: 60 vel: 100 dur: 0.25]\nWAIT 0.5\n>> [note: 64 vel: 80]"),
        );
        doc.reference.insert(Word("WAIT".into()), ReferenceEntry::new(
            "Advance time by a duration in beats. Without WAIT, all events fire simultaneously."
        ).with_category("Output & Timing").with_example(">> [note: 60]\nWAIT 0.5\n>> [note: 64]\nWAIT 0.5\n>> [note: 67]"));
        doc.reference.insert(Word("DEV".into()), ReferenceEntry::new(
            "Set the default output device for subsequent events. Individual events can override with the dev: key."
        ).with_category("Output & Timing").with_example("DEV 1\n>> [note: 60]\nDEV 2\n>> [note: 48]"));
        doc.reference.insert(
            Word("PRINT".into()),
            ReferenceEntry::new("Print a value to the log device.")
                .with_category("Output & Timing")
                .with_aliases(&["P"])
                .with_example("PRINT \"hello\"\nP ADD 2 3"),
        );
        doc.reference.insert(
            Word("BREAK".into()),
            ReferenceEntry::new("Exit the current script immediately.")
                .with_category("Output & Timing")
                .with_example("IF GT G.X 10 : BREAK END\n>> [note: 60]"),
        );
        doc.reference.insert(Word("SET".into()), ReferenceEntry::new(
            "Assign a value to a variable. Returns the assigned value. Scopes: G. (global), F. (frame), L. (line)."
        ).with_category("Output & Timing").with_example("SET G.root 60\n>> [note: G.root]\nWAIT 0.5\n>> [note: ADD G.root 7]"));

        // -- Special Variables --
        doc.reference.insert(
            Word("I".into()),
            ReferenceEntry::new("Loop index variable. Set by RANGE, EACH, EU, and BIN loops.")
                .with_category("Special Variables")
                .with_example("RANGE 0 3 :\n  >> [note: ADD 60 MUL I 4]\n  WAIT 0.25\nEND"),
        );
        doc.reference.insert(
            Word("E".into()),
            ReferenceEntry::new("Current element variable. Set by EACH (forEach) loops.")
                .with_category("Special Variables")
                .with_example("EACH '[60 64 67] :\n  >> [note: E]\n  WAIT 0.25\nEND"),
        );
        doc.reference.insert(
            Word("T".into()),
            ReferenceEntry::new("Current tempo in BPM. Read-only.")
                .with_category("Special Variables")
                .with_example(">> [note: ? GT T 120 72 60]"),
        );
        doc.reference.insert(
            Word("R".into()),
            ReferenceEntry::new("Random value 0-127. Changes on each read.")
                .with_category("Special Variables")
                .with_example(">> [note: ADD 48 MOD R 24 vel: R]"),
        );

        // -- Arithmetic --
        doc.reference.insert(Word("ADD".into()), ReferenceEntry::new(
            "Addition. ADD a b. Also works on maps (recursive merge, values added for matching keys)."
        ).with_category("Arithmetic").with_example(">> [note: ADD 60 7 vel: 100]\nWAIT 0.5\n>> [note: ADD 60 12]"));
        doc.reference.insert(
            Word("SUB".into()),
            ReferenceEntry::new("Subtraction. SUB a b.")
                .with_category("Arithmetic")
                .with_example(">> [note: SUB 72 12]\nWAIT 0.5\n>> [vel: SUB 127 MUL I 20]"),
        );
        doc.reference.insert(
            Word("MUL".into()),
            ReferenceEntry::new("Multiplication. MUL a b.")
                .with_category("Arithmetic")
                .with_example("RANGE 0 3 :\n  >> [note: ADD 60 MUL I 4]\n  WAIT 0.25\nEND"),
        );
        doc.reference.insert(
            Word("DIV".into()),
            ReferenceEntry::new("Division. DIV a b.")
                .with_category("Arithmetic")
                .with_example("WAIT DIV 1 8\n>> [note: 60]"),
        );
        doc.reference.insert(
            Word("MOD".into()),
            ReferenceEntry::new("Modulo. MOD a b. Returns remainder of division.")
                .with_category("Arithmetic")
                .with_example("RANGE 0 7 :\n  >> [note: ADD 60 MOD MUL I 7 12]\n  WAIT 0.125\nEND"),
        );
        doc.reference.insert(
            Word("NEG".into()),
            ReferenceEntry::new("Negate a value. NEG x. Unary.")
                .with_category("Arithmetic")
                .with_example("SET G.X 5\nP NEG G.X"),
        );
        doc.reference.insert(
            Word("ABS".into()),
            ReferenceEntry::new("Absolute value. ABS x. Unary.")
                .with_category("Arithmetic")
                .with_example(">> [vel: ABS SUB 64 R]"),
        );

        // -- Comparison --
        doc.reference.insert(
            Word("GT".into()),
            ReferenceEntry::new("Greater than. GT a b. Returns boolean.")
                .with_category("Comparison")
                .with_example("IF GT G.X 10 :\n  >> [note: 72]\nEND"),
        );
        doc.reference.insert(
            Word("LT".into()),
            ReferenceEntry::new("Less than. LT a b. Returns boolean.")
                .with_category("Comparison")
                .with_example("IF LT R 64 :\n  >> [note: 60 vel: 40]\nEND"),
        );
        doc.reference.insert(
            Word("GTE".into()),
            ReferenceEntry::new("Greater than or equal. GTE a b. Returns boolean.")
                .with_category("Comparison")
                .with_example("IF GTE G.count 4 :\n  SET G.count 0\nEND"),
        );
        doc.reference.insert(
            Word("LTE".into()),
            ReferenceEntry::new("Less than or equal. LTE a b. Returns boolean.")
                .with_category("Comparison")
                .with_example("SET G.vel CLAMP R 0 127\nIF LTE G.vel 10 : SET G.vel 10 END"),
        );
        doc.reference.insert(Word("EQ".into()), ReferenceEntry::new(
            "Equal. EQ a b. Returns boolean."
        ).with_category("Comparison").with_example("IF EQ MOD I 4 0 :\n  >> [note: 60 vel: 127]\nELSE :\n  >> [note: 60 vel: 60]\nEND"));
        doc.reference.insert(
            Word("NE".into()),
            ReferenceEntry::new("Not equal. NE a b. Returns boolean.")
                .with_category("Comparison")
                .with_example("IF NE G.last G.current :\n  >> [note: G.current]\nEND"),
        );

        // -- Logical --
        doc.reference.insert(
            Word("AND".into()),
            ReferenceEntry::new("Logical AND. AND a b. Returns boolean.")
                .with_category("Logical")
                .with_example("IF AND GT I 2 LT I 6 :\n  >> [note: 60]\nEND"),
        );
        doc.reference.insert(
            Word("OR".into()),
            ReferenceEntry::new("Logical OR. OR a b. Returns boolean.")
                .with_category("Logical")
                .with_example("IF OR EQ I 0 EQ I 4 :\n  >> [note: 60 vel: 127]\nEND"),
        );
        doc.reference.insert(
            Word("XOR".into()),
            ReferenceEntry::new("Logical exclusive OR. XOR a b.")
                .with_category("Logical")
                .with_example("IF XOR TOSS TOSS :\n  >> [note: 72]\nEND"),
        );
        doc.reference.insert(
            Word("NOT".into()),
            ReferenceEntry::new("Logical NOT. NOT x. Unary.")
                .with_category("Logical")
                .with_example("IF NOT GT I 4 :\n  >> [note: 60]\nEND"),
        );

        // -- Bitwise --
        doc.reference.insert(Word("BAND".into()), ReferenceEntry::new(
            "Bitwise AND. BAND a b. Also acts as map union (first wins on conflict) when used on maps."
        ).with_category("Bitwise").with_example(">> [note: ADD 60 BAND I 3]"));
        doc.reference.insert(
            Word("BOR".into()),
            ReferenceEntry::new(
                "Bitwise OR. BOR a b. On maps: union where first map wins on conflict.",
            )
            .with_category("Bitwise")
            .with_example(">> [note: ADD 48 BOR I 7]"),
        );
        doc.reference.insert(
            Word("BXOR".into()),
            ReferenceEntry::new("Bitwise exclusive OR. BXOR a b.")
                .with_category("Bitwise")
                .with_example(">> [note: ADD 60 BXOR I 5]"),
        );
        doc.reference.insert(
            Word("BNOT".into()),
            ReferenceEntry::new("Bitwise NOT. BNOT x. Unary.")
                .with_category("Bitwise")
                .with_example("P BNOT 0"),
        );
        doc.reference.insert(
            Word("SHL".into()),
            ReferenceEntry::new("Shift left. SHL value bits.")
                .with_category("Bitwise")
                .with_example(">> [vel: SHL 1 I]"),
        );
        doc.reference.insert(
            Word("SHR".into()),
            ReferenceEntry::new("Shift right. SHR value bits.")
                .with_category("Bitwise")
                .with_example(">> [vel: SHR 127 I]"),
        );

        // -- Utility --
        doc.reference.insert(
            Word("MIN".into()),
            ReferenceEntry::new("Minimum of two values. MIN a b.")
                .with_category("Utility")
                .with_example(">> [vel: MIN R 100]"),
        );
        doc.reference.insert(
            Word("MAX".into()),
            ReferenceEntry::new("Maximum of two values. MAX a b.")
                .with_category("Utility")
                .with_example(">> [vel: MAX R 40]"),
        );
        doc.reference.insert(
            Word("CLAMP".into()),
            ReferenceEntry::new("Constrain value to range. CLAMP value min max.")
                .with_category("Utility")
                .with_example(">> [note: CLAMP ADD 60 R 48 84]"),
        );
        doc.reference.insert(
            Word("WRAP".into()),
            ReferenceEntry::new("Wrap value around range. WRAP value min max.")
                .with_category("Utility")
                .with_example(
                    "RANGE 0 15 :\n  >> [note: ADD 60 WRAP MUL I 7 0 12]\n  WAIT 0.125\nEND",
                ),
        );
        doc.reference.insert(
            Word("SCALE".into()),
            ReferenceEntry::new(
                "Remap value from one range to another. SCALE value in_min in_max out_min out_max.",
            )
            .with_category("Utility")
            .with_example(">> [vel: SCALE I 0 7 40 127]"),
        );
        doc.reference.insert(
            Word("QT".into()),
            ReferenceEntry::new("Quantize to nearest step. QT value step.")
                .with_category("Utility")
                .with_example(">> [note: QT RRAND 48 84 4]"),
        );

        // -- Random --
        doc.reference.insert(
            Word("TOSS".into()),
            ReferenceEntry::new("Random 0 or 1. No arguments. Coin flip.")
                .with_category("Random")
                .with_example("IF TOSS :\n  >> [note: 60]\nELSE :\n  >> [note: 72]\nEND"),
        );
        doc.reference.insert(
            Word("RAND".into()),
            ReferenceEntry::new("Random integer from 0 to n (inclusive). RAND n.")
                .with_category("Random")
                .with_example(">> [note: ADD 48 RAND 24 vel: ADD 60 RAND 67]"),
        );
        doc.reference.insert(
            Word("RRAND".into()),
            ReferenceEntry::new("Random integer in range (inclusive). RRAND lo hi.")
                .with_category("Random")
                .with_example(">> [note: RRAND 48 72 vel: RRAND 60 127]\nWAIT 0.25"),
        );
        doc.reference.insert(Word("DRUNK".into()), ReferenceEntry::new(
            "Brownian walk. DRUNK variable max_step. Mutates the variable by a random amount up to +/- max_step."
        ).with_category("Random").with_example("SET G.N 60\nRANGE 0 7 :\n  SET G.N CLAMP DRUNK G.N 2 48 84\n  >> [note: G.N]\n  WAIT 0.125\nEND"));

        // -- Control Flow --
        doc.reference.insert(
            Word("IF".into()),
            ReferenceEntry::new("Conditional. IF cond : body END. Supports ELSE : body END.")
                .with_category("Control Flow")
                .with_example(
                    "IF GT R 64 :\n  >> [note: 72 vel: 100]\nELSE :\n  >> [note: 60 vel: 60]\nEND",
                ),
        );
        doc.reference.insert(Word("SWITCH".into()), ReferenceEntry::new(
            "Multi-way branch. SWITCH expr : CASE val : body ... DEFAULT : body END."
        ).with_category("Control Flow").with_example("SWITCH MOD I 4 :\n  CASE 0 : >> [note: 60]\n  CASE 2 : >> [note: 64]\n  DEFAULT : >> [note: 67]\nEND\nWAIT 0.25"));
        doc.reference.insert(Word("PROB".into()), ReferenceEntry::new(
            "Execute body with a given percentage probability. PROB percent : body END. Supports ELSE."
        ).with_category("Control Flow").with_example(">> [note: 60]\nPROB 30 : >> [note: 72 vel: 40] END\nWAIT 0.5"));
        doc.reference.insert(Word("RANGE".into()), ReferenceEntry::new(
            "Counted loop with index I. RANGE start end : body END. Optional step: RANGE start end step : body END."
        ).with_category("Control Flow").with_example("RANGE 0 7 :\n  >> [note: ADD 60 WRAP MUL I 7 0 12 vel: SUB 127 MUL I 10]\n  WAIT 0.125\nEND"));
        doc.reference.insert(
            Word("DO".into()),
            ReferenceEntry::new("Repeat N times with no index variable. DO n : body END.")
                .with_category("Control Flow")
                .with_example("DO 4 :\n  >> [note: RRAND 60 72]\n  WAIT 0.25\nEND"),
        );
        doc.reference.insert(Word("WHILE".into()), ReferenceEntry::new(
            "Loop while condition is true. WHILE cond : body END."
        ).with_category("Control Flow").with_example("SET G.N 60\nWHILE LT G.N 72 :\n  >> [note: G.N]\n  WAIT 0.25\n  SET G.N ADD G.N RRAND 1 3\nEND"));
        doc.reference.insert(Word("EACH".into()), ReferenceEntry::new(
            "Iterate over list elements. E is the current element, I is the index. EACH list : body END."
        ).with_category("Control Flow").with_example("EACH '[60 64 67 72] :\n  >> [note: E vel: SUB 127 MUL I 20]\n  WAIT 0.25\nEND"));
        doc.reference.insert(Word("EVERY".into()), ReferenceEntry::new(
            "Execute every Nth iteration. Counter persists per-line. EVERY n : body END."
        ).with_category("Control Flow").with_example("RANGE 0 7 :\n  >> [note: 60]\n  EVERY 2 : >> [note: 72 vel: 40] END\n  WAIT 0.125\nEND"));

        // -- Ternary --
        doc.reference.insert(
            Word("?".into()),
            ReferenceEntry::new(
                "Ternary conditional expression. ? cond then else. Inline alternative to IF/ELSE.",
            )
            .with_category("Control Flow")
            .with_example(">> [note: ? TOSS 60 72 vel: ? GT I 4 100 60]\nWAIT 0.25"),
        );

        // -- Rhythm --
        doc.reference.insert(Word("EU".into()), ReferenceEntry::new(
            "Euclidean rhythm. Distribute hits evenly across steps. EU hits steps dur : body END. Supports ELSE for misses. I is the step index."
        ).with_category("Rhythm").with_example("EU 3 8 0.125 :\n  >> [note: 60 vel: SUB 127 MUL I 10]\nELSE :\n  >> [note: 60 vel: 20]\nEND"));
        doc.reference.insert(Word("BIN".into()), ReferenceEntry::new(
            "Binary rhythm. An integer's bits define the pattern (1=hit, 0=miss, MSB first). BIN pattern dur : body END. Supports ELSE."
        ).with_category("Rhythm").with_example("BIN 170 0.125 :\n  >> [note: 60 vel: 100]\nELSE :\n  >> [note: 60 vel: 20]\nEND"));

        // -- Concurrency --
        doc.reference.insert(Word("FORK".into()), ReferenceEntry::new(
            "Spawn a concurrent branch. Main script continues immediately. FORK : body END."
        ).with_category("Concurrency").with_example("FORK :\n  DO 4 : >> [note: 72] WAIT 0.25 END\nEND\nDO 4 : >> [note: 48] WAIT 0.5 END"));

        // -- Functions --
        doc.reference.insert(Word("FUNC".into()), ReferenceEntry::new(
            "Define a named function. Names must be uppercase (2+ chars). Args are single uppercase letters. FUNC NAME A B : body END."
        ).with_category("Functions").with_example("FUNC ARP N :\n  >> [note: N] WAIT 0.125\n  >> [note: ADD N 4] WAIT 0.125\n  >> [note: ADD N 7] WAIT 0.125\nEND\n(CALL ARP 60)\n(CALL ARP 65)"));
        doc.reference.insert(
            Word("FN".into()),
            ReferenceEntry::new(
                "Lambda (anonymous function). Stored in variables. FN args : body END.",
            )
            .with_category("Functions")
            .with_example("SET G.F FN X : MUL X 2 END\nSET G.Y (CALL G.F 5)\nP G.Y"),
        );
        doc.reference.insert(
            Word("CALL".into()),
            ReferenceEntry::new(
                "Call a function. Must be wrapped in parentheses. (CALL name args...).",
            )
            .with_category("Functions")
            .with_example("FUNC DOUBLE X : MUL X 2 END\n>> [note: (CALL DOUBLE 30)]"),
        );

        // -- Selection --
        doc.reference.insert(
            Word("CHOOSE".into()),
            ReferenceEntry::new(
                "Pick one option randomly each evaluation. CHOOSE: val val ... END.",
            )
            .with_category("Selection")
            .with_example(">> [note: CHOOSE: 60 64 67 72 END vel: 100]\nWAIT 0.5"),
        );
        doc.reference.insert(Word("ALT".into()), ReferenceEntry::new(
            "Cycle through options sequentially. State persists per-line. ALT: val val ... END."
        ).with_category("Selection").with_example("RANGE 0 7 :\n  >> [note: ALT: 60 64 67 72 END]\n  WAIT 0.125\nEND"));

        // -- Lists --
        doc.reference.insert(Brackets("'[".into(), "]".into()), ReferenceEntry::new(
            "List literal. Ordered collection of values. '[val val ...]. Prefix quote distinguishes from maps."
        ).with_category("Lists").with_example("SET G.NOTES '[60 64 67 72]\nEACH G.NOTES :\n  >> [note: E]\n  WAIT 0.25\nEND"));
        doc.reference.insert(
            Word("LEN".into()),
            ReferenceEntry::new("Get list length. LEN list.")
                .with_category("Lists")
                .with_example("SET G.L '[60 64 67]\nP LEN G.L"),
        );
        doc.reference.insert(Word("GET".into()), ReferenceEntry::new(
            "Get element at index (wraps around). GET list index. Negative indices wrap from end."
        ).with_category("Lists").with_example("SET G.NOTES '[60 64 67 72]\nRANGE 0 7 :\n  >> [note: GET G.NOTES I]\n  WAIT 0.125\nEND"));
        doc.reference.insert(
            Word("PICK".into()),
            ReferenceEntry::new("Pick a random element from a list. PICK list.")
                .with_category("Lists")
                .with_example(
                    "DO 4 :\n  >> [note: PICK '[60 64 67 72] vel: 100]\n  WAIT 0.25\nEND",
                ),
        );
        doc.reference.insert(
            Word("CYCLE".into()),
            ReferenceEntry::new(
                "Cycle through list elements sequentially. State persists per-line. CYCLE list.",
            )
            .with_category("Lists")
            .with_example("RANGE 0 7 :\n  >> [note: CYCLE '[60 64 67 72]]\n  WAIT 0.125\nEND"),
        );
        doc.reference.insert(Word("MAP".into()), ReferenceEntry::new(
            "Apply a function to each list element, returning a new list. MAP fn list."
        ).with_category("Lists").with_example("SET G.X MAP FN A : ADD A 12 END '[60 64 67]\nEACH G.X : >> [note: E] WAIT 0.25 END"));
        doc.reference.insert(Word("FILTER".into()), ReferenceEntry::new(
            "Keep elements where predicate returns true. FILTER fn list."
        ).with_category("Lists").with_example("SET G.X FILTER FN A : GT A 60 END '[48 60 72 84]\nEACH G.X : >> [note: E] WAIT 0.25 END"));
        doc.reference.insert(
            Word("REDUCE".into()),
            ReferenceEntry::new("Fold a list into a single value. REDUCE fn initial list.")
                .with_category("Lists")
                .with_example("SET G.SUM REDUCE FN A B : ADD A B END 0 '[1 2 3 4]\nP G.SUM"),
        );

        // -- Maps --
        doc.reference.insert(
            Brackets("[".into(), "]".into()),
            ReferenceEntry::new(
                "Map literal. Key-value pairs. [key: val key: val ...]. Values can be expressions.",
            )
            .with_category("Maps")
            .with_example("SET G.M [note: 60 vel: ADD 50 RAND 77]\n>> G.M"),
        );
        doc.reference.insert(
            Word("MNEW".into()),
            ReferenceEntry::new("Create an empty map. MNEW.")
                .with_category("Maps")
                .with_example("SET G.M MNEW\nSET G.M MSET G.M \"note\" 60"),
        );
        doc.reference.insert(
            Word("MGET".into()),
            ReferenceEntry::new("Get a value from a map by key. MGET map key.")
                .with_category("Maps")
                .with_example("SET G.M [note: 60 vel: 100]\nP MGET G.M \"note\""),
        );
        doc.reference.insert(
            Word("MSET".into()),
            ReferenceEntry::new("Set a key in a map. Returns a new map. MSET map key value.")
                .with_category("Maps")
                .with_example("SET G.M [note: 60]\nSET G.M MSET G.M \"vel\" 100\n>> G.M"),
        );
        doc.reference.insert(
            Word("MHAS".into()),
            ReferenceEntry::new("Check if a key exists in a map. Returns boolean. MHAS map key.")
                .with_category("Maps")
                .with_example("SET G.M [note: 60]\nIF MHAS G.M \"vel\" : P \"has vel\" END"),
        );
        doc.reference.insert(
            Word("MMERGE".into()),
            ReferenceEntry::new(
                "Merge two maps. Second map wins on key conflict. MMERGE map1 map2.",
            )
            .with_category("Maps")
            .with_example(
                "SET G.BASE [note: 60 vel: 100]\nSET G.M MMERGE G.BASE [vel: 40 chan: 1]\n>> G.M",
            ),
        );
        doc.reference.insert(
            Word("MLEN".into()),
            ReferenceEntry::new("Get the number of keys in a map. MLEN map.")
                .with_category("Maps")
                .with_example("SET G.M [note: 60 vel: 100 chan: 0]\nP MLEN G.M"),
        );

        // -- MIDI Input --
        doc.reference.insert(Word("CCIN".into()), ReferenceEntry::new(
            "Read MIDI CC value (0-127) from an input device. CCIN ctrl uses context device/channel. (CCIN ctrl dev chan) reads from a specific device and channel."
        ).with_category("MIDI Input").with_example(">> [note: 60 vel: CCIN 1]\nSET G.vol (CCIN 7 2 1)"));

        // -- Special --
        doc.reference.insert(Word("BYTES".into()), ReferenceEntry::new(
            "Raw byte sequence for SysEx messages. BYTES: val val ... END. Values can be expressions."
        ).with_category("Special").with_example(">> [sysex: BYTES: 240 67 32 0 247 END]"));

        // -- Articles --
        doc.articles.push((
            "Introduction".into(),
            include_str!("../../docs/bob/intro.md").into(),
        ));
        doc.articles.push((
            "Language Reference".into(),
            include_str!("../../docs/bob/reference.md").into(),
        ));
        doc
    }

    fn syntax(&self) -> Option<LanguageSyntax> {
        Some(super::bob_syntax::syntax())
    }
}

impl Compiler for BobCompiler {
    fn compile(
        &self,
        script: &str,
        _args: &BTreeMap<String, String>,
    ) -> Result<Program, CompilationError> {
        let preprocessed = super::bob_preprocess::preprocess(script);
        match bob_grammar::ProgramParser::new().parse(&preprocessed) {
            Ok(parsed) => Ok(bob_as_asm(parsed)),
            Err(parse_error) => {
                let (from, to) = match &parse_error {
                    ParseError::InvalidToken { location } => (*location, *location),
                    ParseError::UnrecognizedEof { location, .. } => (*location, *location),
                    ParseError::UnrecognizedToken {
                        token: (f, _, t), ..
                    } => (*f, *t),
                    ParseError::ExtraToken { token: (f, _, t) } => (*f, *t),
                    ParseError::User { .. } => (0, 0),
                };
                Err(CompilationError {
                    lang: "Bob".to_string(),
                    info: parse_error.to_string(),
                    from,
                    to,
                })
            }
        }
    }
}

fn bob_as_asm(program: BobProgram) -> Program {
    let mut ctx = CompileContext::new();

    // First pass: collect function definitions
    collect_function_defs(&program, &mut ctx);

    // Second pass: compile expression
    let dest = ctx.temp("_bob_result");
    compile_expr(&program, &dest, &mut ctx)
}

fn collect_function_defs(expr: &BobExpr, ctx: &mut CompileContext) {
    match expr {
        BobExpr::Seq(left, right) => {
            collect_function_defs(left, ctx);
            collect_function_defs(right, ctx);
        }
        BobExpr::FunctionDef { name, args, .. } => {
            ctx.functions.insert(
                name.clone(),
                crate::bob::context::FunctionInfo {
                    arg_names: args.clone(),
                },
            );
        }
        BobExpr::Fork { body } => {
            collect_function_defs(body, ctx);
        }
        _ => {}
    }
}
