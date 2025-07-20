import { LRLanguage, LanguageSupport } from '@codemirror/language';

export interface LanguageDefinition {
  name: string;
  extension: string;
  parser: LRLanguage;
  support: LanguageSupport;
}