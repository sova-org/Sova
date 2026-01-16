import { parser } from './bob.grammar.js';
import { LRLanguage, LanguageSupport } from '@codemirror/language';
import { styleTags, tags as t } from '@lezer/highlight';
import type { LanguageDefinition } from './types';

const bobLanguage = LRLanguage.define({
  parser: parser.configure({
    props: [
      styleTags({
        ControlKeyword: t.controlKeyword,
        FunctionKeyword: t.definitionKeyword,
        EventKeyword: t.special(t.keyword),
        SelectionKeyword: t.keyword,
        AssignKeyword: t.modifier,
        BuiltinVar: t.atom,
        OperatorKeyword: t.operatorKeyword,
        SymbolicOperator: t.operator,
        ScopedVar: t.special(t.variableName),
        Symbol: t.atom,
        ListStart: t.squareBracket,
        String: t.string,
        Comment: t.lineComment,
        Number: t.number,
        Identifier: t.variableName,
        Punctuation: t.punctuation,
        '( )': t.paren,
        '[ ]': t.squareBracket,
        '{ }': t.brace,
      }),
    ],
  }),
  languageData: {
    name: 'Bob',
    fileExtensions: ['bob'],
    commentTokens: { line: '#' },
  },
});

export function createBobLanguage(): LanguageDefinition {
  return {
    name: 'Bob',
    extension: 'bob',
    parser: bobLanguage,
    support: new LanguageSupport(bobLanguage),
  };
}
