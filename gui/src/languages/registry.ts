import type { LanguageSupport } from '@codemirror/language';
import type { LanguageDefinition } from './types';

const LANGUAGES: Record<string, LanguageDefinition> = {};

export function registerLanguage(id: string, definition: LanguageDefinition): void {
  LANGUAGES[id] = definition;
}

export function getLanguageSupport(languageName: string): LanguageSupport | null {
  return LANGUAGES[languageName]?.support || null;
}

export function getAvailableLanguages(): string[] {
  return Object.keys(LANGUAGES);
}

export function getLanguageDefinition(languageName: string): LanguageDefinition | null {
  return LANGUAGES[languageName] || null;
}