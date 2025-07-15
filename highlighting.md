# Syntax Highlighting Implementation Analysis & Recommendations

## Current State Analysis

### TUI Implementation
- **Library**: `syntect` with `tui-textarea` for syntax highlighting
- **Data Flow**: Server sends `.sublime-syntax` files → `SyntaxSetBuilder` → `SyntaxHighlighter` → `TextArea`
- **Languages**: BaLi, Dummy, Boinx (server-provided via `syntax_definitions` in Hello message)
- **Architecture**: 
  - `EditorData.syntax_highlighter: Option<Arc<SyntaxHighlighter>>`
  - `EditorData.syntax_name_map: HashMap<String, String>` (compiler_name → syntax_name)
  - Dynamic language lookup: `scene.lines[line_idx].scripts[frame_idx].lang`

### GUI Implementation
- **Library**: CodeMirror 6 with `@uiw/react-codemirror` wrapper
- **Dependencies**: `@codemirror/language`, `@lezer/highlight`, `@codemirror/state`, `@codemirror/view`
- **Current State**: No language-specific syntax highlighting
- **Missing Components**: Language selection UI, language packages, syntax highlighting integration

## Existing Language Definitions

### BaLi Language (`bali.sublime-syntax`)
```yaml
name: BaLi
file_extensions: [bali]
scope: source.bali
```

**Token Types:**
- **Comments**: `;` semicolon-style line comments
- **Strings**: `"..."` double-quoted with escape sequences
- **Keywords**: 
  - Control flow: `loop`, `eucloop`, `binloop`, `spread`, `scatter`, `pick`, `with`, `withdirt`, `?`, `seq`, `for`, `if`
  - Effects: `def`, `note`, `prog`, `control`, `at`, `chanpress`, `osc`, `dirt`
  - Operators: `>>`, `<<`, `>`, `<`, `+`, `-`, `*`, `/`, `%`, `//`, `scale`, `clamp`, `min`, `max`, `quantize`, `sine`, `saw`, `triangle`, `isaw`, `randstep`, `ccin`
  - Logical: `&&`, `||`, `not`, `lt`, `leq`, `gt`, `geq`, `==`, `!=`
- **Context Elements**: `dev:`, `ch:`, `v:`, `dur:`, `sh:`, `-n`, `-r`
- **Dirt Parameters**: `:param_name` (prefixed with colon)
- **Numbers**: Integer and float literals
- **Identifiers**: Variables, musical notes (`c4`, `f#5`), reserved vars (`T`, `R`, `A`, `B`)

### Dummy Language (`dummy.sublime-syntax`)
```yaml
name: DummyLang
file_extensions: [dummy]
scope: source.dummy
```

**Token Types:**
- **Keywords**: `A`, `C`, `N` (control keywords)
- **Numbers**: Integer literals
- **Whitespace**: Explicit whitespace handling

## Recommended Implementation Strategy

### Approach: Direct Lezer Grammar Creation

Create custom CodeMirror 6 language packages using Lezer grammar for each language.

#### Phase 1: Language Package Structure

```typescript
// src/languages/types.ts
interface LanguageDefinition {
  name: string;
  extension: string;
  parser: LRLanguage;
  support: LanguageSupport;
}

// src/languages/registry.ts
const LANGUAGES: Record<string, LanguageDefinition> = {
  'bali': createBaliLanguage(),
  'dummy': createDummyLanguage(),
  'boinx': createBoinxLanguage(),
};

export function getLanguageSupport(languageName: string): LanguageSupport | null {
  return LANGUAGES[languageName]?.support || null;
}
```

#### Phase 2: BaLi Grammar Definition

```lezer
// src/languages/bali.grammar
@top Program { expression* }

expression {
  List |
  Atom
}

List {
  "(" expression* ")"
}

Atom {
  ControlKeyword |
  EffectKeyword |
  ArithmeticOperator |
  LogicalOperator |
  FlowOperator |
  ContextElement |
  DirtParam |
  String |
  Float |
  Integer |
  Identifier |
  Comment
}

ControlKeyword {
  "loop" | "eucloop" | "binloop" | "spread" | "scatter" | "pick" | 
  "with" | "withdirt" | "?" | "seq" | "for" | "if"
}

EffectKeyword {
  "def" | "note" | "prog" | "control" | "at" | "chanpress" | "osc" | "dirt"
}

ArithmeticOperator {
  "+" | "-" | "*" | "/" | "%" | "//" | "scale" | "clamp" | "min" | "max" | 
  "quantize" | "sine" | "saw" | "triangle" | "isaw" | "randstep" | "ccin"
}

LogicalOperator {
  "&&" | "||" | "not" | "lt" | "leq" | "gt" | "geq" | "==" | "!="
}

FlowOperator {
  ">>" | "<<" | ">" | "<"
}

ContextElement {
  ("dev" | "ch" | "v" | "dur") ":" |
  ("sh") ":" |
  ("-n" | "-r")
}

DirtParam {
  ":" identifier
}

String {
  '"' (![\\\n"] | "\\" _)* '"'
}

Comment {
  ";" ![\n]* 
}

Float {
  digit+ "." digit+
}

Integer {
  digit+
}

Identifier {
  (letter | "_") (letter | digit | "_" | "#" | "-")*
}

@tokens {
  letter { std.letter }
  digit { std.digit }
  whitespace { std.whitespace }
}

@skip { whitespace }
```

#### Phase 3: Syntax Highlighting Configuration

```typescript
// src/languages/bali.ts
import { parser } from "./bali.grammar";
import { LRLanguage, LanguageSupport } from "@codemirror/language";
import { styleTags, tags as t } from "@lezer/highlight";

const baliLanguage = LRLanguage.define({
  parser: parser.configure({
    props: [
      styleTags({
        ControlKeyword: t.controlKeyword,
        EffectKeyword: t.keyword,
        ArithmeticOperator: t.operator,
        LogicalOperator: t.logicOperator,
        FlowOperator: t.operator,
        ContextElement: t.tagName,
        DirtParam: t.attributeName,
        String: t.string,
        Comment: t.lineComment,
        Float: t.float,
        Integer: t.integer,
        Identifier: t.variableName,
        "( )": t.paren,
      }),
    ],
  }),
  languageData: {
    name: "BaLi",
    extensions: ["bali"],
    commentTokens: { line: ";" },
  },
});

export function createBaliLanguage(): LanguageDefinition {
  return {
    name: "BaLi",
    extension: "bali",
    parser: baliLanguage,
    support: new LanguageSupport(baliLanguage),
  };
}
```

#### Phase 4: Dummy Language (Minimal Implementation)

```lezer
// src/languages/dummy.grammar
@top Program { statement* }

statement {
  ControlKeyword |
  Integer |
  whitespace
}

ControlKeyword {
  "A" | "C" | "N"
}

Integer {
  digit+
}

@tokens {
  digit { std.digit }
  whitespace { std.whitespace }
}

@skip { whitespace }
```

```typescript
// src/languages/dummy.ts
export function createDummyLanguage(): LanguageDefinition {
  return {
    name: "Dummy",
    extension: "dummy", 
    parser: dummyLanguage,
    support: new LanguageSupport(dummyLanguage),
  };
}
```

#### Phase 5: CodeEditor Integration

```typescript
// src/components/CodeEditor.tsx
import { getLanguageSupport } from '../languages/registry';

export const CodeEditor: React.FC<CodeEditorProps> = ({ ... }) => {
  // Get current language from scene data
  const currentLanguage = useMemo(() => {
    // Extract language from current frame's script
    return scene?.lines[activeLineIndex]?.scripts
      ?.find(script => script.index === activeFrameIndex)?.lang || 'bali';
  }, [scene, activeLineIndex, activeFrameIndex]);

  const extensions = useMemo(() => {
    const baseExtensions = [];
    
    // Add vim mode
    if (editorSettings.vimMode) {
      baseExtensions.push(vim());
    }
    
    // Add language support
    const languageSupport = getLanguageSupport(currentLanguage);
    if (languageSupport) {
      baseExtensions.push(languageSupport);
    }
    
    // Add other extensions
    baseExtensions.push(flashField);
    if (onEvaluate) {
      baseExtensions.push(evalKeymap({ onEvaluate, flashColor }));
    }
    
    return baseExtensions;
  }, [currentLanguage, editorSettings.vimMode, onEvaluate, palette.warning]);

  return (
    <div className={`h-full w-full relative ${className}`}>
      <CodeMirror
        value={value}
        height="100%"
        theme={currentTheme}
        extensions={extensions}
        onChange={(value) => onChange?.(value)}
        basicSetup={{
          lineNumbers: true,
          foldGutter: true,
          dropCursor: false,
          allowMultipleSelections: false,
          indentOnInput: true,
          bracketMatching: true,
          closeBrackets: true,
          autocompletion: true,
          highlightSelectionMatches: false,
          searchKeymap: true,
        }}
      />
    </div>
  );
};
```

#### Phase 6: Language Selection UI

```typescript
// src/components/LanguageSelector.tsx
interface LanguageSelectorProps {
  currentLanguage: string;
  availableLanguages: string[];
  onLanguageChange: (language: string) => void;
}

export const LanguageSelector: React.FC<LanguageSelectorProps> = ({
  currentLanguage,
  availableLanguages,
  onLanguageChange,
}) => {
  return (
    <select 
      value={currentLanguage} 
      onChange={(e) => onLanguageChange(e.target.value)}
      className="language-selector"
    >
      {availableLanguages.map(lang => (
        <option key={lang} value={lang}>
          {lang.charAt(0).toUpperCase() + lang.slice(1)}
        </option>
      ))}
    </select>
  );
};
```

## Build Configuration

### Package.json Dependencies
```json
{
  "dependencies": {
    "@codemirror/language": "^6.11.2",
    "@lezer/highlight": "^1.2.1",
    "@codemirror/state": "^6.5.2",
    "@codemirror/view": "^6.38.0"
  },
  "devDependencies": {
    "@lezer/generator": "^1.7.1"
  }
}
```

### Build Process
```bash
# Generate parser from grammar
npx lezer-generator src/languages/bali.grammar

# Build process will handle .grammar files automatically
npm run build
```

## Integration Points

### Message Protocol
- **Existing**: `SetScriptLanguage` message type in `types.ts`
- **Server Response**: Language change reflected in scene data
- **Client Action**: Send message when user changes language

### Theme Integration
- **Current**: `createCustomTheme()` with palette-based theming
- **Extension**: Syntax highlighting uses existing theme tokens
- **Fallback**: Default highlighting tags if theme doesn't specify

### Error Handling
- **Compilation Errors**: Include language identifier in error display
- **Grammar Errors**: Graceful fallback to plain text if parser fails
- **Missing Languages**: Default to plain text, log warning

## Performance Considerations

### Incremental Parsing
- **Lezer Benefit**: Incremental parsing with error recovery
- **Memory**: Shared parser instances via language registry
- **Caching**: Language support objects cached per language

### Loading Strategy
- **Lazy Loading**: Load language packages on demand
- **Bundle Splitting**: Separate chunks for each language
- **Fallback**: Immediate plain text, upgrade to syntax highlighting

## Testing Strategy

### Unit Tests
- **Grammar**: Test parser output for language constructs
- **Highlighting**: Verify correct token classification
- **Integration**: Test language switching and theme application

### Development Testing
1. Start with Dummy language (simplest grammar)
2. Validate highlighting and theme integration
3. Implement BaLi with comprehensive grammar
4. Add language selection UI
5. Test with existing editor features (vim mode, evaluation)

## Migration Path

### Phase 1: Foundation
- Create language registry and type definitions
- Implement Dummy language for testing
- Add basic language support to CodeEditor

### Phase 2: Core Languages
- Implement comprehensive BaLi grammar
- Add language selection UI component
- Integrate with existing message protocol

### Phase 3: Enhancement
- Add Boinx language support
- Implement language-specific features (autocompletion)
- Add language indicator to editor UI

### Phase 4: Polish
- Optimize performance and bundle size
- Add comprehensive error handling
- Implement advanced syntax highlighting features

## Key Benefits

1. **Performance**: Incremental parsing with error recovery
2. **Extensibility**: Easy to add new languages and features
3. **Consistency**: Unified theming with existing CodeMirror setup
4. **Maintainability**: Type-safe language definitions
5. **Future-Proof**: Foundation for language-specific features

## File Structure

```
src/
├── languages/
│   ├── types.ts              # Language definition interfaces
│   ├── registry.ts           # Language registry and utilities
│   ├── bali.grammar          # BaLi Lezer grammar
│   ├── bali.ts               # BaLi language implementation
│   ├── dummy.grammar         # Dummy Lezer grammar
│   ├── dummy.ts              # Dummy language implementation
│   └── boinx.ts              # Boinx language placeholder
└── components/
    ├── CodeEditor.tsx        # Updated with language support
    └── LanguageSelector.tsx  # New language selection UI
```