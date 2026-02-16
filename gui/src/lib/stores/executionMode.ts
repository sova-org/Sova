import { writable, type Writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import type { ExecutionMode } from '$lib/types/protocol';
import { ListenerGroup } from './helpers';
import { SERVER_EVENTS } from '$lib/events';

export const sceneMode: Writable<ExecutionMode> = writable('AtQuantum');

const listeners = new ListenerGroup();

export async function initializeExecutionModeStore(): Promise<void> {
	await listeners.add(() =>
		listen<ExecutionMode>(SERVER_EVENTS.GLOBAL_MODE, (event) => {
			sceneMode.set(event.payload);
		})
	);
}

export function cleanupExecutionModeStore(): void {
	listeners.cleanup();
	sceneMode.set('AtQuantum');
}
