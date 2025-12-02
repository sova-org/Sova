import { parser } from './dummy.grammar.js';
import { LRLanguage, LanguageSupport } from '@codemirror/language';
import { styleTags, tags as t } from '@lezer/highlight';
import type { LanguageDefinition } from './types';

const dummyLanguage = LRLanguage.define({
  parser: parser.configure({
    props: [
      styleTags({
        ControlKeyword: t.keyword,
        Integer: t.integer,
      }),
    ],
  }),
  languageData: {
    name: 'Dummy',
    fileExtensions: ['dummy'],
  },
});

export function createDummyLanguage(): LanguageDefinition {
  return {
    name: 'Dummy',
    extension: 'dummy',
    parser: dummyLanguage,
    support: new LanguageSupport(dummyLanguage),
  };
}