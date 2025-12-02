import { parser } from './bali.grammar.js';
import { LRLanguage, LanguageSupport } from '@codemirror/language';
import { styleTags, tags as t } from '@lezer/highlight';
import type { LanguageDefinition } from './types';

const baliLanguage = LRLanguage.define({
  parser: parser.configure({
    props: [
      styleTags({
        Keyword: t.keyword,
        ContextKeyword: t.tagName,
        DirtParam: t.attributeName,
        String: t.string,
        Comment: t.lineComment,
        Number: t.number,
        Identifier: t.variableName,
        Operator: t.operator,
        "( )": t.paren,
      }),
    ],
  }),
  languageData: {
    name: 'BaLi',
    fileExtensions: ['bali'],
    commentTokens: { line: ';' },
  },
});

export function createBaliLanguage(): LanguageDefinition {
  return {
    name: 'BaLi',
    extension: 'bali',
    parser: baliLanguage,
    support: new LanguageSupport(baliLanguage),
  };
}