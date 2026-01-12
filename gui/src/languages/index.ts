import { registerLanguage } from './registry';
import { createDummyLanguage } from './dummy';
import { createBaliLanguage } from './bali';
import { createBobLanguage } from './bob';

// Register all available languages
export function initializeLanguages(): void {
	registerLanguage('dummy', createDummyLanguage());
	registerLanguage('bali', createBaliLanguage());
	registerLanguage('bob', createBobLanguage());
}

// Re-export registry functions for convenience
export {
	getLanguageSupport,
	getAvailableLanguages,
	getLanguageDefinition,
} from './registry';
