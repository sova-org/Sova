import { atom } from 'nanostores';
import { invoke } from '@tauri-apps/api/core';

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
}

const DEFAULT_CONFIG: ServerConfig = {
  ip: '0.0.0.0',
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
};

export const serverConfigStore = atom<ServerConfig>(DEFAULT_CONFIG);

export const configDirtyStore = atom<boolean>(false);

export const loadConfig = async (): Promise<void> => {
  try {
    const config = await invoke<ServerConfig>('get_server_config');
    serverConfigStore.set(config);
    configDirtyStore.set(false);
  } catch (error) {
    console.error('Failed to load config:', error);
  }
};

export const saveConfig = async (): Promise<void> => {
  try {
    const config = serverConfigStore.get();
    await invoke('save_server_config', { config });
    configDirtyStore.set(false);
  } catch (error) {
    console.error('Failed to save config:', error);
    throw error;
  }
};

export const updateConfig = (updates: Partial<ServerConfig>): void => {
  const current = serverConfigStore.get();
  serverConfigStore.set({ ...current, ...updates });
  configDirtyStore.set(true);
};

export const getConfigFilePath = async (): Promise<string> => {
  try {
    return await invoke<string>('get_config_file_path');
  } catch (error) {
    console.error('Failed to get config file path:', error);
    return 'unknown';
  }
};
