import { writable, type Writable } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import type { ExecutionMode } from '$lib/types/protocol';
import { ListenerGroup } from './helpers';
import { SERVER_EVENTS } from '$lib/events';
import { scene } from './scene';

export const globalMode: Writable<ExecutionMode | null> = writable(null);

const listeners = new ListenerGroup();

export async function initializeExecutionModeStore(): Promise<void> {
	await listeners.add(() =>
		listen<ExecutionMode | null>(SERVER_EVENTS.GLOBAL_MODE, (event) => {
			const newMode = event.payload;
			globalMode.set(newMode);

			// When global mode is set, update all lines to match
			if (newMode !== null) {
				scene.update((currentScene) => {
					if (!currentScene) return currentScene;
					return {
						...currentScene,
						lines: currentScene.lines.map((line) => ({
							...line,
							execution_mode: {
								...line.execution_mode,
								looping: newMode.looping,
							},
						})),
					};
				});
			}
		})
	);
}

export function cleanupExecutionModeStore(): void {
	listeners.cleanup();
	globalMode.set(null);
}
