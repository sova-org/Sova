import { persistentMap } from '@nanostores/persistent';

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
  connectionStore.setKey('username', settings.username ?? connectionStore.get().username);
  connectionStore.setKey('ip', settings.ip ?? connectionStore.get().ip);
  connectionStore.setKey('port', settings.port ?? connectionStore.get().port);
};

export const getConnectionSettings = (): ConnectionSettings => {
  return connectionStore.get();
};