//! Abstract Syntax Tree types for the Bob language.
//!
//! Bob is a purely expression-oriented language using Polish/prefix notation
//! where operators precede their operands:
//! - `ADD 2 8` → 10
//! - `ADD 2 MUL 3 4` → 14 (operators consume fixed arguments, right-to-left)
//!
//! Everything is an expression. Semicolons sequence expressions, discarding
//! the left value and returning the right: `G.X 1; G.Y 2; + G.X G.Y` → 3

/// A Bob program is a single expression (possibly a sequence).
pub type BobProgram = BobExpr;

/// An expression in Bob - everything is an expression.
///
/// Expressions use Polish notation: operator first, then operands.
/// Arity is fixed per operator, so no parentheses needed for nesting.
#[derive(Debug, Clone)]
pub enum BobExpr {
    /// A literal or variable reference.
    Value(BobValue),

    /// Sequence: `expr; expr` - evaluates left, discards, evaluates right, returns right.
    Seq(Box<BobExpr>, Box<BobExpr>),

    /// Assignment: `G.X 10` - assigns AND returns the value.
    Assign(BobValue, Box<BobExpr>),

    /// Built-in operator call: `ADD X Y`, `RAND 10`, `CLAMP val 0 100`
    Call(String, Vec<BobExpr>),

    /// User-defined function call: `(CALL FACTORIAL N)`
    FunctionCall(String, Vec<BobExpr>),

    /// List literal: `'[1 2 3]`
    List(Vec<BobExpr>),

    /// Map literal: `[note: 60, vel: 100]`
    MapLiteral(Vec<(String, Box<BobExpr>)>),

    /// Empty map constructor: `MNEW`
    MapNew,

    /// Map key access: `MGET mymap "key"`
    MapGet(Box<BobExpr>, Box<BobExpr>),

    /// Map key existence check: `MHAS mymap "key"`
    MapHas(Box<BobExpr>, Box<BobExpr>),

    /// Map key insertion: `MSET mymap "key" value` - returns the map
    MapSet(Box<BobExpr>, Box<BobExpr>, Box<BobExpr>),

    /// Map merge: `MMERGE map1 map2` - merge two maps, second wins on conflict
    MapMerge(Box<BobExpr>, Box<BobExpr>),

    /// Map length: `MLEN mymap` - returns number of keys
    MapLen(Box<BobExpr>),

    /// Map operation: `MAP fn list` - apply function to each element
    Map(Box<BobExpr>, Box<BobExpr>),

    /// Filter operation: `FILTER fn list` - keep elements matching predicate
    Filter(Box<BobExpr>, Box<BobExpr>),

    /// Reduce operation: `REDUCE fn init list` - fold list into single value
    Reduce(Box<BobExpr>, Box<BobExpr>, Box<BobExpr>),

    /// Random selection: `CHOOSE: a b c END` picks one at random.
    Choose(Vec<BobExpr>),

    /// Sequential cycling: `ALT: a b c END` returns next in sequence each call.
    Alt(Vec<BobExpr>),

    /// Byte array: `BYTES: 240 67 32 0 247 END` for SysEx data.
    Bytes(Vec<BobExpr>),

    /// Ternary conditional: `? cond then else`
    Ternary(Box<BobExpr>, Box<BobExpr>, Box<BobExpr>),

    /// IF expression: `IF cond : expr END` or `IF cond : expr ELSE : expr END`
    If {
        condition: Box<BobExpr>,
        then_expr: Box<BobExpr>,
        else_expr: Box<BobExpr>,
    },

    /// Switch expression: `SWITCH val : CASE 1 : expr CASE 2 : expr DEFAULT : expr END`
    Switch {
        value: Box<BobExpr>,
        cases: Vec<(BobExpr, BobExpr)>,
        default: Box<BobExpr>,
    },

    /// Probabilistic expression: `PROB 50 : expr END` or `PROB 50 : expr ELSE : expr END`
    Prob {
        threshold: Box<BobExpr>,
        then_expr: Box<BobExpr>,
        else_expr: Box<BobExpr>,
    },

    /// Loop expression (list comprehension): `L start end : expr END`
    /// Returns list of collected values.
    Loop {
        start: Box<BobExpr>,
        end: Box<BobExpr>,
        step: Box<BobExpr>,
        body: Box<BobExpr>,
    },

    /// While loop: `WHILE cond : expr END` - returns last value or 0
    While {
        condition: Box<BobExpr>,
        body: Box<BobExpr>,
    },

    /// Repeat N times: `DO n : expr END` - returns last value or 0
    Do {
        count: Box<BobExpr>,
        body: Box<BobExpr>,
    },

    /// For-each: `EACH list : expr END` - returns list of collected values
    ForEach {
        list: Box<BobExpr>,
        body: Box<BobExpr>,
    },

    /// Execute every Nth call: `EVERY n : expr END` - returns expr or 0
    Every {
        period: Box<BobExpr>,
        body: Box<BobExpr>,
    },

    /// Lambda expression: `FN args : expr END`
    Lambda {
        args: Vec<String>,
        body: Box<BobExpr>,
    },

    /// Function definition: `FUNC NAME args : expr END` - defines and returns 0
    FunctionDef {
        name: String,
        args: Vec<String>,
        body: Box<BobExpr>,
    },

    /// Emit expression: `>> [note: 60]` - emits event AND returns the map.
    Emit(Box<BobExpr>),

    /// Wait expression: `WAIT 0.5` - advances time, returns 0
    Wait(Box<BobExpr>),

    /// Device selection: `DEV 1` - sets output device, returns 0
    Dev(Box<BobExpr>),

    /// Break: exits script, returns 0
    Break,

    /// Unit/empty expression (for empty programs)
    Unit,

    /// Fork expression: `FORK: body END`
    /// Spawns a concurrent execution branch with shared state (G.*, F.*).
    /// Returns 0 immediately after spawning.
    Fork { body: Box<BobExpr> },

    /// Euclidean rhythm: `EU hits steps dur : body END` or with ELSE
    /// Iterates `steps` times, executing body on euclidean-distributed hits.
    /// Each step advances time by `dur` beats. `I` is the step index.
    Euclidean {
        hits: Box<BobExpr>,
        steps: Box<BobExpr>,
        dur: Box<BobExpr>,
        hit_body: Box<BobExpr>,
        miss_body: Box<BobExpr>,
    },

    /// Binary rhythm: `BIN pattern dur : body END` or with ELSE
    /// Iterates over bits of `pattern` (MSB first), executing body on 1-bits.
    /// Steps = significant bits (64 - leading_zeros), 0 if pattern == 0.
    /// Each step advances time by `dur` beats. `I` is the step index.
    Binary {
        pattern: Box<BobExpr>,
        dur: Box<BobExpr>,
        hit_body: Box<BobExpr>,
        miss_body: Box<BobExpr>,
    },
}

/// A value in Bob - literals and variable references.
#[derive(Debug, Clone)]
pub enum BobValue {
    /// Integer literal: `42`, `-10`
    Int(i64),

    /// Float literal: `3.14`, `-0.5`
    Float(f64),

    /// String literal: `"hello"`
    Str(String),

    /// Symbol: `:mysymbol` or note like `:c4`
    Symbol(String),

    /// Global variable: `G.X`, `G.name`
    GlobalVar(String),

    /// Frame-scoped variable: `F.counter`
    FrameVar(String),

    /// Line-scoped variable: `L.phase`
    LineVar(String),

    /// Instance variable: `temp`, `myvar` (read-only)
    InstanceVar(String),

    /// Environment: tempo (read-only)
    EnvTempo,

    /// Environment: random 0-127 (read-only)
    EnvRandom,
}
