# Vim Mode Redesign

This document outlines the redesign of the vim mode implementation for elegant, composable, and maintainable vim emulation.

## Current Problems

1. **Monolithic Handler**: 750+ line function with deeply nested match statements
2. **Missing Grammar**: No modeling of vim's `[count][operator][motion]` structure
3. **Limited Composability**: Hardcoded operator-motion combinations instead of true composition
4. **Complex State**: Scattered state across multiple fields with unclear responsibilities
5. **Poor Extensibility**: Adding new motions/operators requires extensive changes

## Design Principles

1. **Grammar-Based Architecture**: Model vim's actual command grammar
2. **Composable Operations**: Any operator works with any motion/text-object
3. **Single Responsibility**: Each module handles one concern
4. **Stateless Execution**: Commands execute without hidden state mutations
5. **Focused Implementation**: Essential vim features done excellently, not comprehensive coverage

## Core Architecture

### Command Structure

```rust
pub struct Command {
    pub count: Option<u32>,
    pub operator: Option<Operator>,
    pub motion: Motion,
}

pub enum Operator {
    Delete,    // d
    Change,    // c  
    Yank,      // y
}

pub enum Motion {
    // Character motions
    Left, Right, Up, Down,
    
    // Word motions  
    WordForward,     // w
    WordBackward,    // b
    WordEnd,         // e
    
    // Line motions
    LineStart,       // 0
    LineEnd,         // $
    LineFirst,       // ^
    
    // Document motions
    Top,             // gg
    Bottom,          // G
    Line(u32),       // {n}G
    
    // Text objects
    InnerWord,       // iw
    AroundWord,      // aw
    InnerQuote(char), // i" i'
    AroundQuote(char), // a" a'
    InnerBracket(char), // i{ i[ i(
    AroundBracket(char), // a{ a[ a(
    
    // Search motions
    FindChar(char),     // f{char}
    TillChar(char),     // t{char}
    FindCharBack(char), // F{char}
    TillCharBack(char), // T{char}
}
```

### Mode System

```rust
pub enum Mode {
    Normal,
    Insert,
    Visual { line_wise: bool },
    Command,
    Search { forward: bool },
    OperatorPending,
}
```

### State Management

```rust
pub struct VimState {
    pub mode: Mode,
    pub parser: CommandParser,
    pub registers: RegisterMap,
    pub last_search: Option<String>,
}

pub struct CommandParser {
    count_buffer: String,
    operator: Option<Operator>,
    motion_buffer: String,
}
```

## Module Structure

### `vim/mod.rs` - Main Handler
- Simple dispatcher to mode-specific handlers
- State transitions
- ~50 lines total

### `vim/parser.rs` - Command Parser
- Incremental parsing of vim commands
- Grammar validation
- Count handling

```rust
impl CommandParser {
    pub fn push_key(&mut self, key: char) -> ParseResult {
        // Incremental parsing logic
    }
}

pub enum ParseResult {
    Incomplete,
    Complete(Command),
    Invalid,
}
```

### `vim/motion.rs` - Motion Engine
- Motion execution
- Text object resolution
- Range calculation

```rust
pub trait MotionExecutor {
    fn execute_motion(&mut self, motion: Motion, count: u32) -> TextRange;
    fn find_text_object(&self, object: TextObject) -> Option<TextRange>;
}
```

### `vim/operator.rs` - Operator Engine
- Operator application to ranges
- Register management
- Mode transitions after operations

```rust
pub fn execute_command(textarea: &mut TextArea, cmd: Command) -> VimResult {
    let range = resolve_motion_range(textarea, cmd.motion, cmd.count.unwrap_or(1))?;
    
    match cmd.operator {
        Some(Operator::Delete) => {
            textarea.delete_range(range);
            Ok(Mode::Normal)
        },
        Some(Operator::Change) => {
            textarea.delete_range(range);
            Ok(Mode::Insert)
        },
        Some(Operator::Yank) => {
            textarea.yank_range(range);
            Ok(Mode::Normal)
        },
        None => {
            textarea.move_to_range_start(range);
            Ok(Mode::Normal)
        }
    }
}
```

### `vim/modes/` - Mode Handlers
- `normal.rs` - Normal mode key handling
- `insert.rs` - Insert mode key handling  
- `visual.rs` - Visual mode key handling
- `command.rs` - Command mode key handling

## Implementation Strategy

### Phase 1: Foundation
1. Create module structure
2. Implement basic Command/Motion/Operator types
3. Replace monolithic handler with dispatcher

### Phase 2: Essential Motions
Focus on most-used motions that provide 80% of functionality:
- Character motions: h, j, k, l
- Word motions: w, b, e
- Line motions: 0, $, ^
- Document motions: gg, G
- Basic text objects: iw, aw

### Phase 3: Essential Operators
- Delete (d)
- Change (c)
- Yank (y)
- Full composition with all motions

### Phase 4: Visual Mode
- Character and line-wise visual selection
- Visual mode operators

## Key Benefits

1. **Simplicity**: Main handler reduces from 750+ lines to ~50 lines
2. **Elegance**: Code structure mirrors vim's actual grammar
3. **Precision**: Composable operators eliminate special cases
4. **Maintainability**: Single-responsibility modules are easy to understand
5. **Extensibility**: Adding motions/operators requires minimal changes

## Essential Feature Set

Rather than comprehensive vim emulation, focus on essential features:

**Normal Mode**:
- Movement: hjkl, w/b/e, 0/$^, gg/G
- Editing: x, dd, D, cc, C, yy, p/P
- Text objects: iw/aw, i"/a", i{/a{
- Operations: u (undo), Ctrl-r (redo)

**Insert Mode**:
- Text insertion
- Esc to normal mode

**Visual Mode**:
- Character/line selection
- Cut/copy/delete operations

**Command Mode**:
- Line numbers (:42)
- Basic commands (:q, :w if applicable)

**Search**:
- Forward/backward search (/, ?)
- Next/previous (n, N)

This focused approach delivers excellent vim experience without overwhelming complexity.