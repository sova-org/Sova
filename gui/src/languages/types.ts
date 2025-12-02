import type { LRLanguage, LanguageSupport } from '@codemirror/language';

export type LanguageDefinition = {
  name: string;
  extension: string;
  parser: LRLanguage;
  support: LanguageSupport;
}