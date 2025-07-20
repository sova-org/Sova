import { persistentAtom } from '@nanostores/persistent';
import { ServerConfig } from './serverManagerStore';
import { updateStore } from '../utils/store-helpers';

// Default server configuration
export const DEFAULT_SERVER_CONFIG: ServerConfig = {
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

// Persistent configuration store
export const serverConfigStore = persistentAtom<ServerConfig>('serverConfig', DEFAULT_SERVER_CONFIG, {
  encode: JSON.stringify,
  decode: JSON.parse,
});

// Additional persistent settings
export const serverSettingsStore = persistentAtom<{
  autoStart: boolean;
  lastKnownStatus: string;
}>('serverSettings', {
  autoStart: false,
  lastKnownStatus: 'Stopped',
}, {
  encode: JSON.stringify,
  decode: JSON.parse,
});

// Helper functions
export const updateServerConfig = (config: Partial<ServerConfig>) => {
  updateStore(serverConfigStore, config);
};

export const getServerConfig = (): ServerConfig => {
  return serverConfigStore.get();
};

export const updateServerSettings = (settings: Partial<{ autoStart: boolean; lastKnownStatus: string }>) => {
  updateStore(serverSettingsStore, settings);
};

export const getServerSettings = () => {
  return serverSettingsStore.get();
};