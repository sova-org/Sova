import type { Theme } from './types';

export function generateThemeCss(theme: Theme): string {
	return `
/* ${theme.name} Theme */

/* UI Colors */
--bg-color: ${theme.colors.background};
--surface-color: ${theme.colors.surface};
--border-color: ${theme.colors.border};
--text-color: ${theme.colors.text};
--text-secondary: ${theme.colors.textSecondary};
--accent-color: ${theme.colors.accent};
--accent-hover: ${theme.colors.accentHover};
--input-bg: ${theme.colors.input};
--input-border: ${theme.colors.inputBorder};
--button-bg: ${theme.colors.button};
--button-hover: ${theme.colors.buttonHover};
--danger-color: ${theme.colors.danger};
--danger-hover: ${theme.colors.dangerHover};

/* Editor Colors */
--editor-background: ${theme.editor.background};
--editor-foreground: ${theme.editor.foreground};
--editor-caret: ${theme.editor.caret};
--editor-selection: ${theme.editor.selection};
--editor-active-line: ${theme.editor.activeLine};
--editor-gutter: ${theme.editor.gutter};
--editor-gutter-text: ${theme.editor.gutterText};
--editor-active-line-gutter: ${theme.editor.activeLineGutter};
--editor-line-number: ${theme.editor.lineNumber};

/* Syntax Highlighting */
--syntax-keyword: ${theme.syntax.keyword};
--syntax-operator: ${theme.syntax.operator};
--syntax-string: ${theme.syntax.string};
--syntax-number: ${theme.syntax.number};
--syntax-boolean: ${theme.syntax.boolean};
--syntax-comment: ${theme.syntax.comment};
--syntax-function: ${theme.syntax.function};
--syntax-class: ${theme.syntax.class};
--syntax-variable: ${theme.syntax.variable};
--syntax-property: ${theme.syntax.property};
--syntax-constant: ${theme.syntax.constant};
--syntax-type: ${theme.syntax.type};
--syntax-tag: ${theme.syntax.tag};
--syntax-attribute: ${theme.syntax.attribute};

/* ANSI Colors */
--ansi-black: ${theme.ansi.black};
--ansi-red: ${theme.ansi.red};
--ansi-green: ${theme.ansi.green};
--ansi-yellow: ${theme.ansi.yellow};
--ansi-blue: ${theme.ansi.blue};
--ansi-magenta: ${theme.ansi.magenta};
--ansi-cyan: ${theme.ansi.cyan};
--ansi-white: ${theme.ansi.white};
--ansi-bright-black: ${theme.ansi.brightBlack};
--ansi-bright-red: ${theme.ansi.brightRed};
--ansi-bright-green: ${theme.ansi.brightGreen};
--ansi-bright-yellow: ${theme.ansi.brightYellow};
--ansi-bright-blue: ${theme.ansi.brightBlue};
--ansi-bright-magenta: ${theme.ansi.brightMagenta};
--ansi-bright-cyan: ${theme.ansi.brightCyan};
--ansi-bright-white: ${theme.ansi.brightWhite};
`.trim();
}
