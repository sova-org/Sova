import { writable, derived, type Writable, type Readable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SERVER_EVENTS } from '$lib/events';
import type { CompilationState, CompilationUpdatePayload, RemoveFramePayload } from '$lib/types/protocol';

// Track latest scriptId per frame position
interface FrameCompilation {
	scriptId: string;  // String to avoid JS precision loss for u64
	state: CompilationState;
}

// Compilation state per frame: Map<"lineId:frameId", FrameCompilation>
export const compilationStates: Writable<Map<string, FrameCompilation>> = writable(new Map());

// Pure helper to create key (without scriptId for simpler lookup)
function makeKey(lineId: number, frameId: number): string {
	return `${lineId}:${frameId}`;
}

// Helper to get compilation state for a specific frame (by position only)
export function getCompilationStateForFrame(
	lineId: number,
	frameId: number
): Readable<CompilationState | null> {
	return derived(compilationStates, ($states) =>
		$states.get(makeKey(lineId, frameId))?.state ?? null
	);
}

// Helper to check if any frame is currently compiling
export const isAnyCompiling: Readable<boolean> = derived(compilationStates, ($states) => {
	for (const { state } of $states.values()) {
		if (state === 'Compiling') return true;
	}
	return false;
});

// Helper to get all failed compilations
export const failedCompilations: Readable<Array<{
	key: string;
	lineId: number;
	frameId: number;
	scriptId: string;
	error: string;
}>> = derived(compilationStates, ($states) => {
	const failed = [];
	for (const [key, { scriptId, state }] of $states.entries()) {
		if (typeof state === 'object' && 'Error' in state) {
			const [lineId, frameId] = key.split(':').map(Number);
			failed.push({ key, lineId, frameId, scriptId, error: state.Error.info });
		}
	}
	return failed;
});

let unlistenFunctions: UnlistenFn[] = [];

export async function initializeCompilationStore(): Promise<void> {
	console.log('[CompilationStore] Initializing compilation store listener');

	// Listen for compilation updates
	unlistenFunctions.push(
		await listen<CompilationUpdatePayload>(SERVER_EVENTS.COMPILATION_UPDATE, (event) => {
			console.log('[CompilationStore] COMPILATION UPDATE RECEIVED:', event.payload);
			const { lineId, frameId, scriptId, state } = event.payload;
			compilationStates.update(($states) => {
				const key = makeKey(lineId, frameId);
				const newStates = new Map($states);
				newStates.set(key, { scriptId, state });
				return newStates;
			});
		})
	);

	// Clean up compilation state when frames are removed
	unlistenFunctions.push(
		await listen<RemoveFramePayload>(SERVER_EVENTS.REMOVE_FRAME, (event) => {
			const { lineId, frameId } = event.payload;
			compilationStates.update(($states) => {
				const newStates = new Map<string, FrameCompilation>();
				for (const [key, value] of $states.entries()) {
					const [lid, fid] = key.split(':').map(Number);
					if (lid === lineId) {
						if (fid < frameId) {
							newStates.set(key, value);
						} else if (fid > frameId) {
							// Shift index down
							newStates.set(makeKey(lid, fid - 1), value);
						}
						// fid === frameId: deleted, skip
					} else {
						newStates.set(key, value);
					}
				}
				return newStates;
			});
		})
	);

	// Clean up compilation state when lines are removed
	unlistenFunctions.push(
		await listen<number>(SERVER_EVENTS.REMOVE_LINE, (event) => {
			const removedLineId = event.payload;
			compilationStates.update(($states) => {
				const newStates = new Map<string, FrameCompilation>();
				for (const [key, value] of $states.entries()) {
					const [lid, fid] = key.split(':').map(Number);
					if (lid < removedLineId) {
						newStates.set(key, value);
					} else if (lid > removedLineId) {
						// Shift line index down
						newStates.set(makeKey(lid - 1, fid), value);
					}
					// lid === removedLineId: deleted, skip
				}
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
