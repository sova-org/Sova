import {
  EditorView,
  lineNumbers,
  highlightActiveLineGutter,
  highlightSpecialChars,
  drawSelection,
  dropCursor,
  rectangularSelection,
  crosshairCursor,
  highlightActiveLine,
  keymap,
} from "@codemirror/view";
import { EditorState, Compartment, type Extension } from "@codemirror/state";
import {
  history,
  defaultKeymap,
  historyKeymap,
  indentWithTab,
} from "@codemirror/commands";
import {
  syntaxHighlighting,
  foldGutter,
  indentOnInput,
  bracketMatching,
  foldKeymap,
  indentUnit,
} from "@codemirror/language";
import {
  autocompletion,
  completionKeymap,
  closeBrackets,
  closeBracketsKeymap,
} from "@codemirror/autocomplete";
import { highlightSelectionMatches, searchKeymap } from "@codemirror/search";
import { lintKeymap } from "@codemirror/lint";
import { vim } from "@replit/codemirror-vim";
import { emacs } from "@replit/codemirror-emacs";
import { get } from "svelte/store";
import {
  editorConfig,
  currentThemeTransformed,
  currentTransparency,
  config,
} from "$lib/stores/config";
import { createHighlightStyle } from "$lib/themes";
import { hexToRgba } from "$lib/utils/colorUtils";
import type { Theme } from "$lib/themes";
import type { EditorConfig } from "$lib/stores/editorConfig";

const keymapCompartment = new Compartment();
const themeCompartment = new Compartment();
const highlightCompartment = new Compartment();
const lineNumbersCompartment = new Compartment();
const lineWrappingCompartment = new Compartment();
const highlightActiveLineCompartment = new Compartment();
const tabSizeCompartment = new Compartment();
const indentUnitCompartment = new Compartment();
const closeBracketsCompartment = new Compartment();
const bracketMatchingCompartment = new Compartment();
const autocompletionCompartment = new Compartment();
const rectangularSelectionCompartment = new Compartment();
const foldGutterCompartment = new Compartment();
const matchHighlightingCompartment = new Compartment();
const languageCompartment = new Compartment();

function getKeymapExtension(mode: string) {
  switch (mode) {
    case "vim":
      return vim();
    case "emacs":
      return emacs();
    case "normal":
      return [];
    default:
      return [];
  }
}

function getIndentUnit(config: EditorConfig): string {
  return config.use_tabs ? "\t" : " ".repeat(config.tab_size);
}

function createEditorTheme(
  fontSize: number,
  cursorBlinkRate: number,
  theme: Theme,
  transparency: number,
  fontFamily?: string,
) {
  const alpha = transparency / 100;
  const cfg = get(config);
  const appearanceFont = cfg?.appearance?.font_family || "monospace";
  const effectiveFont = fontFamily || appearanceFont;

  return EditorView.theme(
    {
      "&": {
        height: "100%",
        fontSize: `${fontSize}px`,
        fontFamily: effectiveFont,
        backgroundColor: hexToRgba(theme.editor.background, alpha),
        color: theme.editor.foreground,
      },
      ".cm-content": {
        caretColor: theme.editor.caret,
      },
      ".cm-cursor, .cm-dropCursor": {
        borderLeftColor: theme.editor.caret,
        animationDuration:
          cursorBlinkRate === 0 ? "0s" : `${cursorBlinkRate}ms`,
      },
      "&.cm-focused .cm-selectionBackground, ::selection": {
        backgroundColor: hexToRgba(theme.editor.selection, alpha),
      },
      ".cm-activeLine": {
        backgroundColor: hexToRgba(theme.editor.activeLine, alpha),
      },
      ".cm-gutters": {
        backgroundColor: hexToRgba(theme.editor.gutter, alpha),
        color: theme.editor.gutterText,
        border: "none",
      },
      ".cm-activeLineGutter": {
        backgroundColor: hexToRgba(theme.editor.activeLineGutter, alpha),
      },
      ".cm-lineNumbers .cm-gutterElement": {
        color: theme.editor.lineNumber,
      },
      ".cm-scroller": {
        overflow: "auto",
      },
    },
    { dark: theme.name !== "light" },
  );
}

function buildExtensions(
  config: EditorConfig,
  theme: Theme,
  transparency: number,
  language: Extension,
  extraKeymaps: Extension[] = [],
) {
  return [
    keymapCompartment.of(getKeymapExtension(config.mode)),
    ...extraKeymaps,
    keymap.of([
      indentWithTab,
      ...closeBracketsKeymap,
      ...defaultKeymap,
      ...searchKeymap,
      ...historyKeymap,
      ...foldKeymap,
      ...completionKeymap,
      ...lintKeymap,
    ]),
    history(),
    drawSelection(),
    dropCursor(),
    indentOnInput(),
    highlightCompartment.of(syntaxHighlighting(createHighlightStyle(theme))),
    highlightSpecialChars(),
    lineNumbersCompartment.of(config.show_line_numbers ? lineNumbers() : []),
    lineWrappingCompartment.of(
      config.line_wrapping ? EditorView.lineWrapping : [],
    ),
    highlightActiveLineCompartment.of(
      config.highlight_active_line
        ? [highlightActiveLine(), highlightActiveLineGutter()]
        : [],
    ),
    tabSizeCompartment.of(EditorState.tabSize.of(config.tab_size)),
    indentUnitCompartment.of(indentUnit.of(getIndentUnit(config))),
    closeBracketsCompartment.of(config.close_brackets ? closeBrackets() : []),
    bracketMatchingCompartment.of(
      config.bracket_matching ? bracketMatching() : [],
    ),
    autocompletionCompartment.of(config.autocomplete ? autocompletion() : []),
    rectangularSelectionCompartment.of(
      config.rectangular_selection
        ? [rectangularSelection(), crosshairCursor()]
        : [],
    ),
    foldGutterCompartment.of(config.fold_gutter ? foldGutter() : []),
    matchHighlightingCompartment.of(
      config.match_highlighting ? highlightSelectionMatches() : [],
    ),
    languageCompartment.of(language),
    themeCompartment.of(
      createEditorTheme(
        config.font_size,
        config.cursor_blink_rate,
        theme,
        transparency,
        config.font_family,
      ),
    ),
  ];
}

export function createEditor(
  container: HTMLElement,
  initialDoc: string,
  language: Extension,
  config: EditorConfig,
  extraKeymaps: Extension[] = [],
): EditorView {
  const theme = get(currentThemeTransformed);
  const transparency = get(currentTransparency);

  const startState = EditorState.create({
    doc: initialDoc,
    extensions: buildExtensions(
      config,
      theme,
      transparency,
      language,
      extraKeymaps,
    ),
  });

  return new EditorView({
    state: startState,
    parent: container,
  });
}

export function createEditorSubscriptions(view: EditorView): () => void {
  let config: EditorConfig | null = get(editorConfig);
  let theme: Theme = get(currentThemeTransformed);
  let transparency: number = get(currentTransparency);

  const unsubscribeConfig = editorConfig.subscribe((newConfig) => {
    if (!view || !newConfig) return;
    config = newConfig;

    view.dispatch({
      effects: [
        keymapCompartment.reconfigure(getKeymapExtension(newConfig.mode)),
        themeCompartment.reconfigure(
          createEditorTheme(
            newConfig.font_size,
            newConfig.cursor_blink_rate,
            theme,
            transparency,
            newConfig.font_family,
          ),
        ),
        lineNumbersCompartment.reconfigure(
          newConfig.show_line_numbers ? lineNumbers() : [],
        ),
        lineWrappingCompartment.reconfigure(
          newConfig.line_wrapping ? EditorView.lineWrapping : [],
        ),
        highlightActiveLineCompartment.reconfigure(
          newConfig.highlight_active_line
            ? [highlightActiveLine(), highlightActiveLineGutter()]
            : [],
        ),
        tabSizeCompartment.reconfigure(
          EditorState.tabSize.of(newConfig.tab_size),
        ),
        indentUnitCompartment.reconfigure(indentUnit.of(getIndentUnit(newConfig))),
        closeBracketsCompartment.reconfigure(
          newConfig.close_brackets ? closeBrackets() : [],
        ),
        bracketMatchingCompartment.reconfigure(
          newConfig.bracket_matching ? bracketMatching() : [],
        ),
        autocompletionCompartment.reconfigure(
          newConfig.autocomplete ? autocompletion() : [],
        ),
        rectangularSelectionCompartment.reconfigure(
          newConfig.rectangular_selection
            ? [rectangularSelection(), crosshairCursor()]
            : [],
        ),
        foldGutterCompartment.reconfigure(
          newConfig.fold_gutter ? foldGutter() : [],
        ),
        matchHighlightingCompartment.reconfigure(
          newConfig.match_highlighting ? highlightSelectionMatches() : [],
        ),
      ],
    });
  });

  const unsubscribeTheme = currentThemeTransformed.subscribe((newTheme) => {
    if (!view || !config) return;
    theme = newTheme;

    view.dispatch({
      effects: [
        themeCompartment.reconfigure(
          createEditorTheme(
            config.font_size,
            config.cursor_blink_rate,
            newTheme,
            transparency,
            config.font_family,
          ),
        ),
        highlightCompartment.reconfigure(
          syntaxHighlighting(createHighlightStyle(newTheme)),
        ),
      ],
    });
  });

  const unsubscribeTransparency = currentTransparency.subscribe(
    (newTransparency) => {
      if (!view || !config) return;
      transparency = newTransparency;

      view.dispatch({
        effects: [
          themeCompartment.reconfigure(
            createEditorTheme(
              config.font_size,
              config.cursor_blink_rate,
              theme,
              newTransparency,
              config.font_family,
            ),
          ),
        ],
      });
    },
  );

  return () => {
    unsubscribeConfig();
    unsubscribeTheme();
    unsubscribeTransparency();
  };
}

export function reconfigureLanguage(
  view: EditorView,
  language: Extension,
): void {
  view.dispatch({
    effects: languageCompartment.reconfigure(language),
  });
}
