import { writable, type Writable } from "svelte/store";

export const availableLanguages: Writable<string[]> = writable([]);

export function setAvailableLanguages(languages: string[]): void {
  availableLanguages.set(languages);
}

export function cleanupLanguagesStore(): void {
  availableLanguages.set([]);
}
