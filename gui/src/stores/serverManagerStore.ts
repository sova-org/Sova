import { map } from 'nanostores';
import { invoke } from '@tauri-apps/api/core';
import { serverConfigStore, updateServerConfig, updateServerSettings } from './serverConfigStore';

// Types matching the Rust backend
export interface ServerConfig {
  ip: string;
  port: number;
  audio_engine: boolean;
  sample_rate: number;
  block_size: number;
  buffer_size: number;
  max_audio_buffers: number;
  max_voices: number;
  output_device?: string;
  osc_port: number;
  osc_host: string;
  timestamp_tolerance_ms: number;
  audio_files_location: string;
  audio_priority: number;
  relay?: string;
  instance_name: string;
  relay_token?: string;
  list_devices: boolean;
}

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

  // Server detection
  async detectRunningServer() {
    try {
      const isRunning = await invoke<boolean>('detect_running_server');
      if (isRunning) {
        await this.refreshState();
        this.startStatusPolling();
      }
      return isRunning;
    } catch (error) {
      console.error('Failed to detect running server:', error);
      return false;
    }
  },

  // Server state actions
  async refreshState() {
    try {
      const state = await invoke<ServerState & { config: ServerConfig }>('get_server_state');
      // Update runtime state
      const runtimeState: ServerState = {
        status: state.status,
        logs: state.logs,
      };
      if (state.process_id !== undefined) {
        runtimeState.process_id = state.process_id;
      }
      serverManagerStore.set(runtimeState);
      // Update persistent config if server is running
      if (state.config) {
        updateServerConfig(state.config);
      }
      // Update last known status
      updateServerSettings({ lastKnownStatus: typeof state.status === 'string' ? state.status : 'Error' });
    } catch (error) {
      console.error('Failed to refresh server state:', error);
    }
  },

  async updateConfig(config: Partial<ServerConfig>) {
    try {
      const currentConfig = serverConfigStore.get();
      const newConfig = { ...currentConfig, ...config };
      
      // Update persistent store first
      updateServerConfig(config);
      
      // Then update backend
      await invoke('update_server_config', { config: newConfig });
      
      // Refresh state to ensure sync
      await this.refreshState();
    } catch (error) {
      console.error('Failed to update config:', error);
      throw error;
    }
  },

  async startServer() {
    try {
      this.setLoading(true);
      this.setError(null);
      
      // Just start the server directly
      await invoke('start_server');
      
      // Poll for status updates
      await this.refreshState();
      
      // Start polling for status updates
      this.startStatusPolling();
    } catch (error) {
      console.error('Failed to start server:', error);
      this.setError(error instanceof Error ? error.message : 'Failed to start server');
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
      
      // Update last known status
      updateServerSettings({ lastKnownStatus: 'Stopped' });
      
      // Refresh state
      await this.refreshState();
      
      // Stop polling
      this.stopStatusPolling();
    } catch (error) {
      console.error('Failed to stop server:', error);
      this.setError(error instanceof Error ? error.message : 'Failed to stop server');
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
      this.setError(error instanceof Error ? error.message : 'Failed to restart server');
      throw error;
    } finally {
      this.setLoading(false);
    }
  },

  async refreshLogs() {
    try {
      const logs = await invoke<LogEntry[]>('get_server_logs', { limit: 100 });
      serverManagerStore.setKey('logs', logs);
    } catch (error) {
      console.error('Failed to refresh logs:', error);
    }
  },

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
        await this.refreshLogs();
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