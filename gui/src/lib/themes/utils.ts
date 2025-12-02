import { HighlightStyle } from "@codemirror/language";
import { tags as t } from "@lezer/highlight";
import type { Theme } from "./types";

/**
 * Creates a CodeMirror HighlightStyle from a Theme object, mapping Lezer syntax
 * tags to the corresponding colors defined in the theme's syntax palette.
 */
export function createHighlightStyle(theme: Theme): HighlightStyle {
  return HighlightStyle.define([
    { tag: t.keyword, color: theme.syntax.keyword },
    {
      tag: [t.name, t.deleted, t.character, t.macroName],
      color: theme.syntax.variable,
    },
    {
      tag: [t.function(t.variableName), t.labelName],
      color: theme.syntax.function,
    },
    {
      tag: [t.color, t.constant(t.name), t.standard(t.name)],
      color: theme.syntax.constant,
    },
    { tag: [t.definition(t.name), t.separator], color: theme.syntax.variable },
    {
      tag: [
        t.typeName,
        t.className,
        t.number,
        t.changed,
        t.annotation,
        t.modifier,
        t.self,
        t.namespace,
      ],
      color: theme.syntax.class,
    },
    {
      tag: [
        t.operator,
        t.operatorKeyword,
        t.url,
        t.escape,
        t.regexp,
        t.link,
        t.special(t.string),
      ],
      color: theme.syntax.operator,
    },
    { tag: [t.meta, t.comment], color: theme.syntax.comment },
    { tag: t.strong, fontWeight: "bold" },
    { tag: t.emphasis, fontStyle: "italic" },
    { tag: t.strikethrough, textDecoration: "line-through" },
    { tag: t.link, color: theme.syntax.string, textDecoration: "underline" },
    { tag: t.heading, fontWeight: "bold", color: theme.syntax.keyword },
    {
      tag: [t.atom, t.bool, t.special(t.variableName)],
      color: theme.syntax.boolean,
    },
    {
      tag: [t.processingInstruction, t.string, t.inserted],
      color: theme.syntax.string,
    },
    { tag: t.invalid, color: theme.editor.foreground },
    { tag: t.number, color: theme.syntax.number },
    { tag: [t.propertyName], color: theme.syntax.property },
    { tag: [t.typeName], color: theme.syntax.type },
    { tag: [t.tagName], color: theme.syntax.tag },
    { tag: [t.attributeName], color: theme.syntax.attribute },
  ]);
}
