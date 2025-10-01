import { atom } from 'nanostores';
import { persistentMap } from '@nanostores/persistent';
import { batchUpdateMap } from '../../utils/store-helpers';
import { serverManagerStore } from '../server/serverManager';
import { serverConfigStore } from '../server/serverConfig';

// Connection Settings (persisted)
export interface ConnectionSettings {
  username: string;
  ip: string;
  port: string;
  [key: string]: string | undefined;
}

export const connectionStore = persistentMap<ConnectionSettings>('connection:', {
  username: 'User',
  ip: '127.0.0.1',
  port: '8080'
});

export const updateConnectionSettings = (settings: Partial<ConnectionSettings>) => {
  batchUpdateMap(connectionStore, settings);
};

export const getConnectionSettings = (): ConnectionSettings => {
  return connectionStore.get();
};

// Connection State (runtime)
export interface ConnectionState {
  isConnected: boolean;
  connectedIp: string | null;
  connectedPort: number | null;
  isConnectedToLocalServer: boolean;
}

export const connectionStateStore = atom<ConnectionState>({
  isConnected: false,
  connectedIp: null,
  connectedPort: null,
  isConnectedToLocalServer: false,
});

export const updateConnectionState = async (connected: boolean, ip?: string, port?: number) => {
  if (!connected) {
    connectionStateStore.set({
      isConnected: false,
      connectedIp: null,
      connectedPort: null,
      isConnectedToLocalServer: false,
    });
    return;
  }

  const serverState = serverManagerStore.get();
  const serverConfig = serverConfigStore.get();

  const isLocalServer = serverState.status === 'Running' &&
    serverState.process_id !== undefined &&
    (ip === '127.0.0.1' || ip === 'localhost') &&
    port === serverConfig.port;

  connectionStateStore.set({
    isConnected: true,
    connectedIp: ip || null,
    connectedPort: port || null,
    isConnectedToLocalServer: isLocalServer,
  });
};

export type LogDisplayMode = 'local-only' | 'remote-only' | 'split';

export const getLogDisplayMode = (): LogDisplayMode => {
  const connectionState = connectionStateStore.get();
  const serverState = serverManagerStore.get();

  if (connectionState.isConnectedToLocalServer) {
    return 'local-only';
  }

  if (connectionState.isConnected && serverState.status !== 'Running') {
    return 'remote-only';
  }

  if (connectionState.isConnected && serverState.status === 'Running') {
    return 'split';
  }

  if (serverState.status === 'Running') {
    return 'local-only';
  }

  return 'remote-only';
};
