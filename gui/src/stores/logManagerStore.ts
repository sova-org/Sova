import { atom, map } from 'nanostores';
import { invoke } from '@tauri-apps/api/core';
// import { readTextFile, exists } from '@tauri-apps/api/fs';

export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
  source: 'file' | 'network';
}

export enum LogSourceType {
  Local = 'local',
  Remote = 'remote',
  Hybrid = 'hybrid'
}

export interface LogManagerState {
  logs: LogEntry[];
  sourceType: LogSourceType;
  isMonitoring: boolean;
  lastFilePosition: number;
  localLogPath: string | null;
}

// Core store
export const logManagerStore = map<LogManagerState>({
  logs: [],
  sourceType: LogSourceType.Remote,
  isMonitoring: false,
  lastFilePosition: 0,
  localLogPath: null
});

// Derived atoms
export const $logs = atom<LogEntry[]>([]);
export const $sourceType = atom<LogSourceType>(LogSourceType.Remote);
export const $isMonitoring = atom<boolean>(false);

// Update derived atoms when main store changes
logManagerStore.subscribe((state) => {
  $logs.set(state.logs);
  $sourceType.set(state.sourceType);
  $isMonitoring.set(state.isMonitoring);
});

class LogManager {
  private fileMonitorInterval: ReturnType<typeof setInterval> | null = null;
  private maxLogs = 1000;
  private batchTimer: ReturnType<typeof setTimeout> | null = null;
  private pendingLogs: LogEntry[] = [];

  /**
   * Initialize the log manager based on connection type
   */
  async initialize(isLocalServer: boolean): Promise<void> {
    // Clear any existing logs when initializing
    logManagerStore.setKey('logs', []);
    logManagerStore.setKey('lastFilePosition', 0);
    
    if (isLocalServer) {
      // Try to get local log file path
      try {
        const localLogPath: string | null = await invoke('get_local_log_file_path');
        
        if (localLogPath) {
          // We have a local server with file logging
          logManagerStore.setKey('sourceType', LogSourceType.Hybrid);
          logManagerStore.setKey('localLogPath', localLogPath);
          await this.startFileMonitoring(localLogPath);
        } else {
          // Local server but no file logging available
          logManagerStore.setKey('sourceType', LogSourceType.Remote);
        }
      } catch (error) {
        console.warn('Failed to get local log file path:', error);
        logManagerStore.setKey('sourceType', LogSourceType.Remote);
      }
    } else {
      // Remote server - only network logs
      logManagerStore.setKey('sourceType', LogSourceType.Remote);
      logManagerStore.setKey('localLogPath', null);
    }
    
    logManagerStore.setKey('isMonitoring', true);
  }

  /**
   * Start monitoring the local log file
   */
  private async startFileMonitoring(filePath: string): Promise<void> {
    // Initial file read to get existing logs
    await this.readFileFromPosition(filePath, 0);
    
    // Set up periodic file monitoring
    this.fileMonitorInterval = setInterval(async () => {
      await this.readFileFromPosition(filePath, logManagerStore.get().lastFilePosition);
    }, 500); // Check every 500ms
  }

  /**
   * Read log file from a specific position
   */
  private async readFileFromPosition(filePath: string, position: number): Promise<void> {
    try {
      // Check if file exists
      const fileExists = await invoke<boolean>('fs_exists', { path: filePath });
      if (!fileExists) {
        return;
      }

      // Read file content
      const content = await invoke<string>('fs_read_text_file', { path: filePath });
      
      // Parse new content from position
      const newContent = content.slice(position);
      if (newContent.length === 0) {
        return;
      }

      // Parse log entries
      const lines = newContent.split('\n').filter((line: string) => line.trim().length > 0);
      const newLogs: LogEntry[] = [];

      for (const line of lines) {
        const logEntry = this.parseLogLine(line);
        if (logEntry) {
          logEntry.source = 'file';
          newLogs.push(logEntry);
        }
      }

      // Update logs and position
      this.addLogs(newLogs);
      logManagerStore.setKey('lastFilePosition', content.length);
      
    } catch (error) {
      console.error('Error reading log file:', error);
    }
  }

  /**
   * Parse a single log line
   */
  private parseLogLine(line: string): LogEntry | null {
    // Try to parse different log formats
    const patterns = [
      // ISO timestamp format: 2025-07-17T14:32:26.696615Z [INFO] message
      /^(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)\s+\[(\w+)\]\s+(.+)$/,
      // Simple format: [INFO] message
      /^\[(\w+)\]\s+(.+)$/,
      // Bare format: just message
      /^(.+)$/
    ];

    for (const pattern of patterns) {
      const match = line.match(pattern);
      if (match) {
        if (match.length === 4) {
          // Full timestamp format
          return {
            timestamp: match[1]!,
            level: match[2]!,
            message: match[3]!,
            source: 'file'
          };
        } else if (match.length === 3) {
          // Simple format
          return {
            timestamp: new Date().toISOString(),
            level: match[1]!,
            message: match[2]!,
            source: 'file'
          };
        } else {
          // Bare format
          return {
            timestamp: new Date().toISOString(),
            level: 'INFO',
            message: match[1]!,
            source: 'file'
          };
        }
      }
    }

    return null;
  }

  /**
   * Handle network log message with batching
   */
  handleNetworkLog(logData: any): void {
    const logEntry: LogEntry = {
      timestamp: logData.timestamp || new Date().toISOString(),
      level: logData.level || 'INFO',
      message: logData.message || logData.toString(),
      source: 'network'
    };

    this.batchLog(logEntry);
  }

  /**
   * Add logs to the store (optimized with batch processing)
   */
  private addLogs(newLogs: LogEntry[]): void {
    if (newLogs.length === 0) return;
    
    const currentLogs = logManagerStore.get().logs;
    const sourceType = logManagerStore.get().sourceType;
    
    // Use Map for O(1) duplicate detection
    const logMap = new Map<string, LogEntry>();
    
    // Add existing logs to map
    currentLogs.forEach(log => {
      const key = `${log.timestamp}-${log.message}-${log.source}`;
      logMap.set(key, log);
    });
    
    // Add new logs with deduplication
    newLogs.forEach(newLog => {
      const key = `${newLog.timestamp}-${newLog.message}-${newLog.source}`;
      
      // Check for exact duplicates first
      if (logMap.has(key)) {
        return;
      }
      
      // Check for cross-source duplicates in hybrid mode
      if (sourceType === LogSourceType.Hybrid) {
        const altKey = `${newLog.timestamp}-${newLog.message}-${newLog.source === 'file' ? 'network' : 'file'}`;
        if (logMap.has(altKey)) {
          return;
        }
      }
      
      logMap.set(key, newLog);
    });
    
    // Convert back to array and sort
    let updatedLogs = Array.from(logMap.values())
      .sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime());
    
    // Trim to max size
    if (updatedLogs.length > this.maxLogs) {
      updatedLogs = updatedLogs.slice(-this.maxLogs);
    }
    
    logManagerStore.setKey('logs', updatedLogs);
  }

  /**
   * Clear all logs
   */
  clearLogs(): void {
    logManagerStore.setKey('logs', []);
    logManagerStore.setKey('lastFilePosition', 0);
  }

  /**
   * Stop monitoring
   */
  stop(): void {
    if (this.fileMonitorInterval) {
      clearInterval(this.fileMonitorInterval);
      this.fileMonitorInterval = null;
    }
    
    if (this.batchTimer) {
      clearTimeout(this.batchTimer);
      this.batchTimer = null;
    }
    
    // Process any remaining pending logs
    if (this.pendingLogs.length > 0) {
      this.addLogs([...this.pendingLogs]);
      this.pendingLogs = [];
    }
    
    logManagerStore.setKey('isMonitoring', false);
  }

  /**
   * Get current logs
   */
  getLogs(): LogEntry[] {
    return logManagerStore.get().logs;
  }

  /**
   * Get source type
   */
  getSourceType(): LogSourceType {
    return logManagerStore.get().sourceType;
  }
  
  /**
   * Batch log processing to reduce UI updates
   */
  private batchLog(logEntry: LogEntry): void {
    this.pendingLogs.push(logEntry);
    
    if (this.batchTimer) {
      clearTimeout(this.batchTimer);
    }
    
    this.batchTimer = setTimeout(() => {
      if (this.pendingLogs.length > 0) {
        this.addLogs([...this.pendingLogs]);
        this.pendingLogs = [];
      }
      this.batchTimer = null;
    }, 50); // Batch updates every 50ms
  }
}

// Export singleton instance
export const logManager = new LogManager();

// Export helper functions
export const initializeLogManager = (isLocalServer: boolean) => logManager.initialize(isLocalServer);
export const handleNetworkLog = (logData: any) => logManager.handleNetworkLog(logData);
export const clearLogs = () => logManager.clearLogs();
export const stopLogManager = () => logManager.stop();