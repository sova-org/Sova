import { writable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SERVER_EVENTS } from '$lib/events';
import type { ScopeSamples } from '$lib/types/protocol';

export const scopeSamples = writable<ScopeSamples | null>(null);

let unlisten: UnlistenFn | null = null;

export async function initScopeListener(): Promise<void> {
	unlisten = await listen<ScopeSamples>(SERVER_EVENTS.SCOPE_DATA, (event) => {
		scopeSamples.set(event.payload);
	});
}

export function cleanupScopeListener(): void {
	if (unlisten) {
		unlisten();
		unlisten = null;
	}
	scopeSamples.set(null);
}
