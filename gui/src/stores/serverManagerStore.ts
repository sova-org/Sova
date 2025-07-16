import { map } from 'nanostores';
import { invoke } from '@tauri-apps/api/core';

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
  config: ServerConfig;
  logs: LogEntry[];
}

// Default configuration
const DEFAULT_CONFIG: ServerConfig = {
  ip: '127.0.0.1',
  port: 8080,
  audio_engine: false,
  sample_rate: 44100,
  block_size: 512,
  buffer_size: 1024,
  max_audio_buffers: 2048,
  max_voices: 128,
  osc_port: 12345,
  osc_host: '127.0.0.1',
  timestamp_tolerance_ms: 1000,
  audio_files_location: './samples',
  audio_priority: 80,
  instance_name: 'local',
  list_devices: false,
};

// Store
export const serverManagerStore = map<ServerState>({
  status: 'Stopped',
  config: DEFAULT_CONFIG,
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

  async updateConfig(config: Partial<ServerConfig>) {
    try {
      const currentState = serverManagerStore.get();
      const newConfig = { ...currentState.config, ...config };
      
      await invoke('update_server_config', { config: newConfig });
      
      // Refresh state to get updated config
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