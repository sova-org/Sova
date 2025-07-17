import { atom } from 'nanostores';
import { serverManagerStore } from './serverManagerStore';
import { serverConfigStore } from './serverConfigStore';

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

// Update connection state when connecting
export const updateConnectionState = (connected: boolean, ip?: string, port?: number) => {
  if (!connected) {
    connectionStateStore.set({
      isConnected: false,
      connectedIp: null,
      connectedPort: null,
      isConnectedToLocalServer: false,
    });
    return;
  }

  // Check if we're connected to the local spawned server
  const serverState = serverManagerStore.get();
  const serverConfig = serverConfigStore.get();
  
  const isLocalServer = serverState.status === 'Running' &&
    (ip === '127.0.0.1' || ip === 'localhost') &&
    port === serverConfig.port;

  connectionStateStore.set({
    isConnected: true,
    connectedIp: ip || null,
    connectedPort: port || null,
    isConnectedToLocalServer: isLocalServer,
  });
};

// Get the current log display mode based on connection state
export type LogDisplayMode = 'local-only' | 'remote-only' | 'split';

export const getLogDisplayMode = (): LogDisplayMode => {
  const connectionState = connectionStateStore.get();
  const serverState = serverManagerStore.get();
  
  // Case 1: Connected to local spawned server - show merged logs
  if (connectionState.isConnectedToLocalServer) {
    return 'local-only'; // We'll merge remote logs into local
  }
  
  // Case 2: Connected to remote server, no local server - show remote only
  if (connectionState.isConnected && serverState.status !== 'Running') {
    return 'remote-only';
  }
  
  // Case 3: Connected to remote server AND running local server - show both
  if (connectionState.isConnected && serverState.status === 'Running') {
    return 'split';
  }
  
  // Not connected, local server may or may not be running
  if (serverState.status === 'Running') {
    return 'local-only';
  }
  
  // Nothing running
  return 'remote-only';
};