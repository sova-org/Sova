import { writable, derived, type Writable, type Readable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SERVER_EVENTS } from '$lib/events';
import type { LogMessage, Severity } from '$lib/types/protocol';

export type LogSource = 'client' | 'server';

export interface LogEntry {
	message: string;
	level: Severity;
	timestamp: number;
	source: LogSource;
}

const MAX_LOGS = 10_000;

// Log entries in chronological order (newest last)
export const logs: Writable<LogEntry[]> = writable([]);

// Filter settings - which severity levels to show (default: all except Debug)
export interface LogFilters {
	fatal: boolean;
	error: boolean;
	warn: boolean;
	info: boolean;
	debug: boolean;
}

export const logFilters: Writable<LogFilters> = writable({
	fatal: true,
	error: true,
	warn: true,
	info: true,
	debug: false,
});

// Tab selection for log source filtering
export type LogTab = 'all' | 'output' | 'server';
export const activeLogTab: Writable<LogTab> = writable('output');

// Severity to filter key mapping for O(1) lookup
const severityToKey: Record<Severity, keyof LogFilters> = {
	Fatal: 'fatal',
	Error: 'error',
	Warn: 'warn',
	Info: 'info',
	Debug: 'debug',
};

// Derived store for filtered logs
export const filteredLogs: Readable<LogEntry[]> = derived(
	[logs, logFilters, activeLogTab],
	([$logs, $filters, $tab]) => {
		const result: LogEntry[] = [];
		for (const log of $logs) {
			// Tab filter
			if ($tab === 'output' && log.source !== 'client') continue;
			if ($tab === 'server' && log.source !== 'server') continue;
			// Severity filter
			const key = severityToKey[log.level];
			if (key === undefined || $filters[key]) {
				result.push(log);
			}
		}
		return result;
	}
);

// RAF-based batching for high-throughput log ingestion
let pendingLogs: LogEntry[] = [];
let rafId: number | null = null;

function flushPendingLogs(): void {
	rafId = null;
	if (pendingLogs.length === 0) return;

	const batch = pendingLogs;
	pendingLogs = [];

	logs.update((current) => {
		const combined = current.concat(batch);
		return combined.length > MAX_LOGS ? combined.slice(-MAX_LOGS) : combined;
	});
}

function scheduleFlush(): void {
	if (rafId === null) {
		rafId = requestAnimationFrame(flushPendingLogs);
	}
}

// Add a batch of log messages (from server:log-batch event)
function addLogBatch(messages: LogMessage[], source: LogSource): void {
	const timestamp = Date.now();
	for (const msg of messages) {
		if (!msg?.msg || msg.msg.trim().length === 0) {
			continue;
		}
		pendingLogs.push({
			message: msg.msg,
			level: msg.level ?? 'Info',
			timestamp,
			source,
		});
	}
	scheduleFlush();
}

// Add a single log message
function addLog(logMessage: LogMessage, source: LogSource): void {
	if (!logMessage?.msg || logMessage.msg.trim().length === 0) {
		return;
	}
	pendingLogs.push({
		message: logMessage.msg,
		level: logMessage.level ?? 'Info',
		timestamp: Date.now(),
		source,
	});
	scheduleFlush();
}

let unlistenFunctions: UnlistenFn[] = [];
let isInitialized = false;

export async function initializeLogsStore(): Promise<void> {
	if (isInitialized) {
		return;
	}

	// Listen for batched logs (high-performance path from local server)
	unlistenFunctions.push(
		await listen<LogMessage[]>(SERVER_EVENTS.LOG_BATCH, (event) => {
			addLogBatch(event.payload, 'client');
		})
	);

	// Listen for individual logs (from remote server via client_manager)
	unlistenFunctions.push(
		await listen<LogMessage>(SERVER_EVENTS.LOG, (event) => {
			addLog(event.payload, 'client');
		})
	);

	// Listen for server subprocess logs
	unlistenFunctions.push(
		await listen<LogMessage>(SERVER_EVENTS.SERVER_LOG, (event) => {
			addLog(event.payload, 'server');
		})
	);

	isInitialized = true;
}

export function cleanupLogsStore(): void {
	// Cancel any pending RAF
	if (rafId !== null) {
		cancelAnimationFrame(rafId);
		rafId = null;
	}
	pendingLogs = [];

	for (const unlisten of unlistenFunctions) {
		unlisten();
	}
	unlistenFunctions = [];
	isInitialized = false;
}

// Export for testing
export { addLog, addLogBatch };
