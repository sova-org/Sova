import { writable, derived, type Writable, type Readable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SERVER_EVENTS } from '$lib/events';
import type { VariableValue } from '$lib/types/protocol';

// Global variables (A-Z single-letter variables)
export const globalVariables: Writable<Record<string, VariableValue>> = writable({});

// Helper to get a specific variable
export function getGlobalVariable(name: string): Readable<VariableValue | null> {
	return derived(globalVariables, ($vars) => $vars[name] ?? null);
}

// Helper to get all variable names
export const globalVariableNames: Readable<string[]> = derived(
	globalVariables,
	($vars) => Object.keys($vars).sort()
);

let unlistenFunctions: UnlistenFn[] = [];

export async function initializeGlobalVariablesStore(): Promise<void> {
	unlistenFunctions.push(
		await listen<Record<string, VariableValue>>(SERVER_EVENTS.GLOBAL_VARIABLES, (event) => {
			globalVariables.set(event.payload);
		})
	);
}

export function cleanupGlobalVariablesStore(): void {
	for (const unlisten of unlistenFunctions) {
		unlisten();
	}
	unlistenFunctions = [];
	globalVariables.set({});
}
