/** Complete theme definition including UI colors, editor styling, syntax highlighting, and ANSI terminal colors. */
export interface Theme {
  name: string;
  colors: {
    background: string;
    surface: string;
    border: string;
    text: string;
    textSecondary: string;
    accent: string;
    accentHover: string;
    input: string;
    inputBorder: string;
    button: string;
    buttonHover: string;
    danger: string;
    dangerHover: string;
  };
  editor: {
    background: string;
    foreground: string;
    caret: string;
    selection: string;
    activeLine: string;
    gutter: string;
    gutterText: string;
    activeLineGutter: string;
    lineNumber: string;
  };
  syntax: {
    keyword: string;
    operator: string;
    string: string;
    number: string;
    boolean: string;
    comment: string;
    function: string;
    class: string;
    variable: string;
    property: string;
    constant: string;
    type: string;
    tag: string;
    attribute: string;
  };
  ansi: {
    black: string;
    red: string;
    green: string;
    yellow: string;
    blue: string;
    magenta: string;
    cyan: string;
    white: string;
    brightBlack: string;
    brightRed: string;
    brightGreen: string;
    brightYellow: string;
    brightBlue: string;
    brightMagenta: string;
    brightCyan: string;
    brightWhite: string;
  };
}

/** Represents a font available for selection in the UI. */
export interface FontOption {
  value: string;
  label: string;
  isBundled?: boolean;
}

export const bundledFonts: FontOption[] = [
  { value: "Departure Mono", label: "Departure Mono", isBundled: true },
  { value: "Victor Mono", label: "Victor Mono", isBundled: true },
  { value: "Space Mono", label: "Space Mono", isBundled: true },
  { value: "IBM Plex Mono", label: "IBM Plex Mono", isBundled: true },
  { value: "Comic Mono", label: "Comic Mono", isBundled: true },
  { value: "JGS5", label: "JGS5 (ASCII Art)", isBundled: true },
  { value: "JGS7", label: "JGS7 (ASCII Art)", isBundled: true },
  { value: "PICO-8", label: "PICO-8", isBundled: true },
  { value: "monospace", label: "System Monospace", isBundled: true },
];

export const fontFamilies = bundledFonts;

/** Merges bundled fonts with system fonts, excluding duplicates. */
export function combineWithSystemFonts(systemFonts: string[]): FontOption[] {
  const bundledFontValues = new Set(bundledFonts.map((f) => f.value));

  const systemFontOptions: FontOption[] = systemFonts
    .filter((name) => !bundledFontValues.has(name))
    .map((name) => ({ value: name, label: name, isBundled: false }));

  return [...bundledFonts, ...systemFontOptions];
}
