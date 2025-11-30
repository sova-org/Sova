import { writable } from 'svelte/store';

export const isHelpModeActive = writable(false);

export function toggleHelpMode(): void {
	isHelpModeActive.update((active) => !active);
}

export function exitHelpMode(): void {
	isHelpModeActive.set(false);
}
