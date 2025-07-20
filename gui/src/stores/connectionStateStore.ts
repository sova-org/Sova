import { atom } from 'nanostores';
import { serverManagerStore } from './serverManagerStore';
import { serverConfigStore } from './serverConfigStore';
import { initializeLogManager } from './logManagerStore';

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

  // Check if we're connected to the local spawned server
  const serverState = serverManagerStore.get();
  const serverConfig = serverConfigStore.get();
  
  // Only consider it a local server if:
  // 1. Our GUI actually spawned/started a server AND  
  // 2. The connection details match our spawned server AND
  // 3. We have a process_id (indicating we started it)
  const isLocalServer = serverState.status === 'Running' &&
    serverState.process_id !== undefined &&  // We have a PID (we started it)
    (ip === '127.0.0.1' || ip === 'localhost') &&
    port === serverConfig.port;

  connectionStateStore.set({
    isConnected: true,
    connectedIp: ip || null,
    connectedPort: port || null,
    isConnectedToLocalServer: isLocalServer,
  });

  // Initialize log manager based on connection type
  await initializeLogManager(isLocalServer);
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