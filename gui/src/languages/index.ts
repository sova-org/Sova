import { registerLanguage } from './registry';
import { createDummyLanguage } from './dummy';
import { createBaliLanguage } from './bali';

// Register all available languages
export function initializeLanguages(): void {
  registerLanguage('dummy', createDummyLanguage());
  registerLanguage('bali', createBaliLanguage());
  // Add more languages here as they are implemented
}

// Re-export registry functions for convenience
export { getLanguageSupport, getAvailableLanguages, getLanguageDefinition } from './registry';