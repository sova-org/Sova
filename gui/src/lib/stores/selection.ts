import { writable } from 'svelte/store';

export interface Selection {
	lineId: number;
	frameId: number;
}

export const selection = writable<Selection | null>(null);

export function selectFrame(lineId: number, frameId: number): void {
	selection.set({ lineId, frameId });
}

export function clearSelection(): void {
	selection.set(null);
}
