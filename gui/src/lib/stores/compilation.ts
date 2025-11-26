import { writable, derived, type Writable, type Readable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SERVER_EVENTS } from '$lib/events';
import type { CompilationState, CompilationUpdatePayload } from '$lib/types/protocol';

// Compilation state per frame: Map<"lineId:frameId:scriptId", CompilationState>
export const compilationStates: Writable<Map<string, CompilationState>> = writable(new Map());

// Pure helper to create key
function makeKey(lineId: number, frameId: number, scriptId: number): string {
	return `${lineId}:${frameId}:${scriptId}`;
}

// Helper to get compilation state for a specific frame
export function getCompilationState(
	lineId: number,
	frameId: number,
	scriptId: number
): Readable<CompilationState | null> {
	return derived(compilationStates, ($states) =>
		$states.get(makeKey(lineId, frameId, scriptId)) ?? null
	);
}

// Helper to check if any frame is currently compiling
export const isAnyCompiling: Readable<boolean> = derived(compilationStates, ($states) => {
	for (const state of $states.values()) {
		if (state === 'Compiling') return true;
	}
	return false;
});

// Helper to get all failed compilations
export const failedCompilations: Readable<Array<{
	key: string;
	lineId: number;
	frameId: number;
	scriptId: number;
	error: string;
}>> = derived(compilationStates, ($states) => {
	const failed = [];
	for (const [key, state] of $states.entries()) {
		if (typeof state === 'object' && 'Error' in state) {
			const [lineId, frameId, scriptId] = key.split(':').map(Number);
			failed.push({ key, lineId, frameId, scriptId, error: state.Error });
		}
	}
	return failed;
});

let unlistenFunctions: UnlistenFn[] = [];

export async function initializeCompilationStore(): Promise<void> {
	unlistenFunctions.push(
		await listen<CompilationUpdatePayload>(SERVER_EVENTS.COMPILATION_UPDATE, (event) => {
			const { lineId, frameId, scriptId, state } = event.payload;
			compilationStates.update(($states) => {
				const newStates = new Map($states);
				newStates.set(makeKey(lineId, frameId, scriptId), state);
				return newStates;
			});
		})
	);
}

export function cleanupCompilationStore(): void {
	for (const unlisten of unlistenFunctions) {
		unlisten();
	}
	unlistenFunctions = [];
	compilationStates.set(new Map());
}
