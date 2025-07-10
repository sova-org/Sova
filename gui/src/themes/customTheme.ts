import { createTheme } from '@uiw/codemirror-themes';
import { tags as t } from '@lezer/highlight';
import { MaterialPalette, ThemeMode } from '../hooks/useMaterialPalette';

export interface CustomThemeOptions {
  palette: MaterialPalette;
  themeMode: ThemeMode;
}

export const createCustomTheme = ({ palette, themeMode }: CustomThemeOptions) => {
  const isDark = themeMode === 'dark';
  
  return createTheme({
    theme: themeMode,
    settings: {
      background: palette.background,
      backgroundImage: '',
      foreground: palette.text,
      caret: palette.primary,
      selection: isDark ? `${palette.primary}40` : `${palette.primary}30`,
      selectionMatch: isDark ? `${palette.primary}30` : `${palette.primary}20`,
      lineHighlight: isDark ? `${palette.surface}80` : `${palette.surface}60`,
      gutterBackground: palette.surface,
      gutterForeground: palette.muted,
      gutterActiveForeground: palette.text,
      gutterBorder: palette.border,
      fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace',
    },
    styles: [
      // Comments
      { tag: [t.comment, t.lineComment, t.blockComment], color: palette.muted, fontStyle: 'italic' },
      
      // Keywords
      { tag: [t.keyword, t.controlKeyword, t.operatorKeyword], color: palette.primary, fontWeight: 'bold' },
      
      // Strings
      { tag: [t.string], color: palette.success },
      
      // Numbers
      { tag: [t.number, t.integer, t.float], color: palette.warning },
      
      // Booleans, null
      { tag: [t.bool, t.null], color: palette.error, fontWeight: 'bold' },
      
      // Function names
      { tag: [t.function(t.variableName), t.function(t.propertyName), t.definition(t.function(t.variableName))], color: palette.secondary, fontWeight: 'bold' },
      
      // Variable names
      { tag: [t.variableName, t.propertyName], color: palette.text },
      
      // Class names, type names
      { tag: [t.className, t.typeName, t.namespace], color: palette.info, fontWeight: 'bold' },
      
      // Operators
      { tag: [t.operator, t.arithmeticOperator, t.logicOperator, t.compareOperator], color: palette.primary600 },
      
      // Punctuation
      { tag: [t.punctuation, t.bracket, t.paren, t.brace], color: palette.muted },
      
      // Attributes
      { tag: [t.attributeName, t.attributeValue], color: palette.secondary600 },
      
      // Tags (HTML/XML)
      { tag: [t.tagName, t.angleBracket], color: palette.primary },
      
      // Special
      { tag: [t.special(t.variableName), t.special(t.propertyName)], color: palette.warning },
      
      // Invalid
      { tag: t.invalid, color: palette.error, textDecoration: 'underline' },
      
      // Headings (Markdown)
      { tag: [t.heading1, t.heading2, t.heading3, t.heading4, t.heading5, t.heading6], color: palette.primary, fontWeight: 'bold' },
      
      // Emphasis (Markdown)
      { tag: [t.emphasis], fontStyle: 'italic' },
      
      // Strong (Markdown)
      { tag: [t.strong], fontWeight: 'bold' },
      
      // Links (Markdown)
      { tag: [t.link, t.url], color: palette.info, textDecoration: 'underline' },
      
      // Code (Markdown)
      { tag: [t.monospace], color: palette.text, backgroundColor: isDark ? `${palette.surface}80` : `${palette.surface}60` },
    ],
  });
};