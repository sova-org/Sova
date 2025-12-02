import type {
  CompletionContext,
  CompletionResult,
} from "@codemirror/autocomplete";
import { themes } from "$lib/themes";

export function tomlThemeCompletion(
  context: CompletionContext,
): CompletionResult | null {
  const line = context.state.doc.lineAt(context.pos);
  const textBefore = line.text.slice(0, context.pos - line.from);

  const match = textBefore.match(/theme\s*=\s*["']?(\w*)$/);
  if (!match) return null;

  const typed = match[1];
  const startPos = context.pos - typed.length;

  const themeNames = Object.keys(themes);
  const filtered = typed
    ? themeNames.filter((name) =>
        name.toLowerCase().startsWith(typed.toLowerCase()),
      )
    : themeNames;

  return {
    from: startPos,
    options: filtered.map((name) => ({
      label: name,
      type: "constant",
    })),
  };
}
