import { persistentMap } from '@nanostores/persistent';
import { batchUpdateMap } from '../utils/store-helpers';

export interface ConnectionSettings {
  username: string;
  ip: string;
  port: string;
  [key: string]: string | undefined;
}

// Create a persistent store with default values
export const connectionStore = persistentMap<ConnectionSettings>('connection:', {
  username: 'User',
  ip: '127.0.0.1',
  port: '8080'
});

// Helper functions
export const updateConnectionSettings = (settings: Partial<ConnectionSettings>) => {
  batchUpdateMap(connectionStore, settings);
};

export const getConnectionSettings = (): ConnectionSettings => {
  return connectionStore.get();
};