import { writable, derived, type Writable, type Readable } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { SERVER_EVENTS } from "$lib/events";
import type { LogMessage, Severity } from "$lib/types/protocol";

export interface LogEntry {
  message: string;
  level: Severity;
  timestamp: number;
}

const MAX_LOGS = 10_000;

// Log entries in chronological order (newest last)
export const logs: Writable<LogEntry[]> = writable([]);

// Filter settings - which severity levels to show (default: all except Debug)
export const showFatal = writable(true);
export const showError = writable(true);
export const showWarn = writable(true);
export const showInfo = writable(true);
export const showDebug = writable(false);

// Derived store for filtered logs
export const filteredLogs: Readable<LogEntry[]> = derived(
  [logs, showFatal, showError, showWarn, showInfo, showDebug],
  ([$logs, $showFatal, $showError, $showWarn, $showInfo, $showDebug]) => {
    return $logs.filter((log) => {
      switch (log.level) {
        case "Fatal":
          return $showFatal;
        case "Error":
          return $showError;
        case "Warn":
          return $showWarn;
        case "Info":
          return $showInfo;
        case "Debug":
          return $showDebug;
        default:
          return true;
      }
    });
  },
);

// Derived store for recent filtered logs
export const recentLogs: Readable<LogEntry[]> = derived(
  filteredLogs,
  ($filteredLogs) => $filteredLogs.slice(-100),
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
function addLogBatch(messages: LogMessage[]): void {
  const timestamp = Date.now();
  for (const msg of messages) {
    pendingLogs.push({
      message: msg.msg,
      level: msg.level,
      timestamp,
    });
  }
  scheduleFlush();
}

// Add a single log message (from server:log event - backward compatibility)
function addLog(logMessage: LogMessage): void {
  pendingLogs.push({
    message: logMessage.msg,
    level: logMessage.level,
    timestamp: Date.now(),
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
      addLogBatch(event.payload);
    }),
  );

  // Listen for individual logs (from remote server via client_manager)
  unlistenFunctions.push(
    await listen<LogMessage>(SERVER_EVENTS.LOG, (event) => {
      addLog(event.payload);
    }),
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
