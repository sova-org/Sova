import { writable, type Writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import type { ExecutionMode } from '$lib/types/protocol';
import { ListenerGroup } from './helpers';
import { SERVER_EVENTS } from '$lib/events';

export const globalMode: Writable<ExecutionMode | null> = writable(null);

const listeners = new ListenerGroup();

export async function initializeExecutionModeStore(): Promise<void> {
	await listeners.add(() =>
		listen<ExecutionMode | null>(SERVER_EVENTS.GLOBAL_MODE, (event) => {
			globalMode.set(event.payload);
		})
	);
}

export function cleanupExecutionModeStore(): void {
	listeners.cleanup();
	globalMode.set(null);
}
