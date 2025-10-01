import { map } from 'nanostores';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { addLog } from '../logs';

export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

export type ServerStatus = 
  | 'Stopped'
  | 'Starting'
  | 'Running'
  | 'Stopping'
  | { Error: string };

export interface ServerState {
  status: ServerStatus;
  process_id?: number;
  logs: LogEntry[];
}

// Runtime state store (without config)
export const serverManagerStore = map<ServerState>({
  status: 'Stopped',
  logs: [],
});

// UI state
export const serverManagerUIStore = map({
  isVisible: false,
  activeTab: 'config' as 'config' | 'logs',
  isLoading: false,
  error: null as string | null,
});

// Actions
export const serverManagerActions = {
  // UI actions
  show: () => serverManagerUIStore.setKey('isVisible', true),
  hide: () => serverManagerUIStore.setKey('isVisible', false),
  setActiveTab: (tab: 'config' | 'logs') => serverManagerUIStore.setKey('activeTab', tab),
  setLoading: (loading: boolean) => serverManagerUIStore.setKey('isLoading', loading),
  setError: (error: string | null) => serverManagerUIStore.setKey('error', error),


  // Server state actions
  async refreshState() {
    try {
      const state = await invoke<ServerState>('get_server_state');
      serverManagerStore.set(state);
    } catch (error) {
      console.error('Failed to refresh server state:', error);
    }
  },

  async startServer() {
    try {
      this.setLoading(true);
      this.setError(null);
      
      // Clear any existing logs before starting
      serverManagerStore.setKey('logs', []);
      
      // Just start the server directly
      await invoke('start_server');
      
      // Poll for status updates
      await this.refreshState();
      
      // Initialize log manager for local server
      
      // Start polling for status updates
      this.startStatusPolling();
    } catch (error) {
      console.error('Failed to start server:', error);
      this.setError(error instanceof Error ? error.message : String(error));
      throw error;
    } finally {
      this.setLoading(false);
    }
  },

  async stopServer() {
    try {
      this.setLoading(true);
      this.setError(null);

      await invoke('stop_server');

      // Refresh state
      await this.refreshState();
      
      // Stop polling
      this.stopStatusPolling();
    } catch (error) {
      console.error('Failed to stop server:', error);
      this.setError(error instanceof Error ? error.message : String(error));
      throw error;
    } finally {
      this.setLoading(false);
    }
  },

  async restartServer() {
    try {
      this.setLoading(true);
      this.setError(null);
      
      await invoke('restart_server');
      
      // Refresh state
      await this.refreshState();
    } catch (error) {
      console.error('Failed to restart server:', error);
      this.setError(error instanceof Error ? error.message : String(error));
      throw error;
    } finally {
      this.setLoading(false);
    }
  },

  // Removed refreshLogs - now using only real-time log streaming
  // to prevent replacing existing logs with limited snapshots

  async listAudioDevices() {
    try {
      const devices = await invoke<string[]>('list_audio_devices');
      return devices;
    } catch (error) {
      console.error('Failed to list audio devices:', error);
      return [];
    }
  },

  // Status polling
  _pollingInterval: null as number | null,

  startStatusPolling() {
    if (this._pollingInterval) return;
    
    this._pollingInterval = window.setInterval(async () => {
      const state = serverManagerStore.get();
      if (state.status === 'Running' || state.status === 'Starting') {
        await this.refreshState();
        // Don't refresh logs - rely on real-time streaming only
      }
    }, 1000);
  },

  stopStatusPolling() {
    if (this._pollingInterval) {
      window.clearInterval(this._pollingInterval);
      this._pollingInterval = null;
    }
  },

  // Initialize
  async initialize() {
    // Just refresh the state - keep it simple
    await this.refreshState();
    
    // Start polling if server is running
    const state = serverManagerStore.get();
    if (state.status === 'Running') {
      this.startStatusPolling();
    }
    
    // Listen for real-time server logs
    this.setupLogListener();
  },
  
  // Set up real-time log listener
  setupLogListener() {
    listen<LogEntry>('server-log', (event) => {
      const logEntry = event.payload;
      
      // Forward to the optimized log manager
      addLog(logEntry.level, logEntry.message, 'file');
      
      // Also maintain backward compatibility by updating this store
      const currentState = serverManagerStore.get();
      const newLogs = [...currentState.logs, logEntry];
      
      // Keep only last 1000 logs
      if (newLogs.length > 1000) {
        newLogs.splice(0, newLogs.length - 1000);
      }
      
      serverManagerStore.setKey('logs', newLogs);
    });
  },
};

// Helper functions
export const getServerStatusText = (status: ServerStatus): string => {
  if (typeof status === 'string') {
    return status;
  } else if (typeof status === 'object' && status.Error) {
    return `Error: ${status.Error}`;
  }
  return 'Unknown';
};

export const getServerStatusColor = (status: ServerStatus): string => {
  if (typeof status === 'string') {
    switch (status) {
      case 'Running':
        return 'var(--color-success)';
      case 'Stopped':
        return 'var(--color-muted)';
      case 'Starting':
      case 'Stopping':
        return 'var(--color-warning)';
      default:
        return 'var(--color-muted)';
    }
  } else if (typeof status === 'object' && status.Error) {
    return 'var(--color-error)';
  }
  return 'var(--color-muted)';
};