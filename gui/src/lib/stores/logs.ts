import { writable, derived, type Writable, type Readable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SERVER_EVENTS } from '$lib/events';

export interface LogEntry {
	message: string;
	timestamp: number;
}

const MAX_LOGS = 1000; // Circular buffer size

// Log entries in chronological order (newest last)
export const logs: Writable<LogEntry[]> = writable([]);

// Derived store for recent logs
export const recentLogs: Readable<LogEntry[]> = derived(
	logs,
	($logs) => $logs.slice(-100) // Last 100 entries
);

// Pure function to add log with circular buffer logic
function addLog(currentLogs: LogEntry[], message: string): LogEntry[] {
	const newLog: LogEntry = { message, timestamp: Date.now() };
	const updated = [...currentLogs, newLog];

	// Keep only last MAX_LOGS entries
	return updated.length > MAX_LOGS ? updated.slice(-MAX_LOGS) : updated;
}

let unlistenFunctions: UnlistenFn[] = [];
let isInitialized = false;

export async function initializeLogsStore(): Promise<void> {
	if (isInitialized) {
		return;
	}

	unlistenFunctions.push(
		await listen<string>(SERVER_EVENTS.LOG, (event) => {
			logs.update(($logs) => addLog($logs, event.payload));
		})
	);

	isInitialized = true;
}

export function cleanupLogsStore(): void {
	for (const unlisten of unlistenFunctions) {
		unlisten();
	}
	unlistenFunctions = [];
	isInitialized = false;
}

// Export for testing
export { addLog };
