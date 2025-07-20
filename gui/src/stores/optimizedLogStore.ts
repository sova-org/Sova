import { atom } from 'nanostores';

export interface LogEntry {
  id: string;
  timestamp: string;
  level: string;
  message: string;
  source: 'file' | 'network';
}

// Single optimized log store
export const logStore = atom<LogEntry[]>([]);

class OptimizedLogManager {
  private logs: LogEntry[] = [];
  private logSet = new Set<string>();
  private maxLogs = 1000;
  private pendingLogs: LogEntry[] = [];
  private flushTimer: number | null = null;

  addLog(level: string, message: string, source: 'file' | 'network' = 'network') {
    const timestamp = new Date().toISOString();
    const id = `${timestamp}-${message}-${source}`;
    
    // O(1) duplicate check
    if (this.logSet.has(id)) return;
    
    const entry: LogEntry = { id, timestamp, level, message, source };
    
    // Add to pending buffer
    this.pendingLogs.push(entry);
    this.logSet.add(id);
    
    this.scheduleFlush();
  }

  private scheduleFlush() {
    if (this.flushTimer !== null) return;
    
    // Flush immediately for first log, then batch subsequent ones
    if (this.logs.length === 0) {
      this.flushPendingLogs();
      return;
    }
    
    // For subsequent logs, use a short delay to batch without blocking
    this.flushTimer = setTimeout(() => {
      this.flushPendingLogs();
      this.flushTimer = null;
    }, 150) as unknown as number; // 150ms batching window
  }
  
  private flushPendingLogs() {
    if (this.pendingLogs.length === 0) return;
    
    // Process all pending logs at once
    this.logs.push(...this.pendingLogs);
    this.pendingLogs = [];
    
    // Trim if needed
    if (this.logs.length > this.maxLogs) {
      const excess = this.logs.length - this.maxLogs;
      const removed = this.logs.splice(0, excess);
      removed.forEach(log => this.logSet.delete(log.id));
    }
    
    // Update store immediately - no blocking
    logStore.set([...this.logs]);
  }

  clear() {
    this.logs = [];
    this.logSet.clear();
    this.pendingLogs = [];
    if (this.flushTimer !== null) {
      clearTimeout(this.flushTimer);
      this.flushTimer = null;
    }
    logStore.set([]);
  }

  getLogs() {
    return this.logs;
  }
}

export const optimizedLogManager = new OptimizedLogManager();
export const addLog = (level: string, message: string, source: 'file' | 'network' = 'network') => 
  optimizedLogManager.addLog(level, message, source);
export const clearLogs = () => optimizedLogManager.clear();