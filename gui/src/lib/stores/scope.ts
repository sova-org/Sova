import { writable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SERVER_EVENTS } from '$lib/events';
import type { ScopePeaks } from '$lib/types/protocol';

export const scopePeaks = writable<ScopePeaks | null>(null);

let unlisten: UnlistenFn | null = null;

export async function initScopeListener(): Promise<void> {
	unlisten = await listen<ScopePeaks>(SERVER_EVENTS.SCOPE_DATA, (event) => {
		scopePeaks.set(event.payload);
	});
}

export function cleanupScopeListener(): void {
	if (unlisten) {
		unlisten();
		unlisten = null;
	}
	scopePeaks.set(null);
}
