import React, { useState, useEffect } from 'react';
import { useStore } from '@nanostores/react';
import { connectionStore, updateConnectionSettings } from '../stores/connectionStore';
import { serverManagerStore, serverManagerActions, getServerStatusColor } from '../stores/serverManagerStore';
import { serverConfigStore } from '../stores/serverConfigStore';
import { Settings, Play, Square, Loader2 } from 'lucide-react';

interface SplashProps {
  onConnect: (name: string, ip: string, port: number) => Promise<void>;
  error?: string;
}

export const Splash: React.FC<SplashProps> = ({ onConnect, error: externalError }) => {
  const savedSettings = useStore(connectionStore);
  const serverState = useStore(serverManagerStore);
  const serverConfig = useStore(serverConfigStore);
  const [name, setName] = useState(savedSettings.username || 'User');
  const [ip, setIp] = useState(savedSettings.ip || '127.0.0.1');
  const [port, setPort] = useState(savedSettings.port || '8080');
  const [error, setError] = useState('');
  const [isConnecting, setIsConnecting] = useState(false);
  const [isServerLoading, setIsServerLoading] = useState(false);

  // Update local state when store changes
  useEffect(() => {
    setName(savedSettings.username || 'User');
    // If server is running, use its actual IP and port
    if (serverState.status === 'Running') {
      setIp(serverConfig.ip || '127.0.0.1');
      setPort(String(serverConfig.port) || '8080');
    } else {
      setIp(savedSettings.ip || '127.0.0.1');
      setPort(savedSettings.port || '8080');
    }
  }, [savedSettings, serverState.status, serverConfig.ip, serverConfig.port]);

  // Show external error if any
  useEffect(() => {
    if (externalError) {
      setError(externalError);
    }
  }, [externalError]);

  const validateIp = (ip: string): boolean => {
    if (!ip || typeof ip !== 'string') return false;
    const parts = ip.split('.');
    if (parts.length !== 4) return false;
    return parts.every(part => {
      const num = parseInt(part);
      return !isNaN(num) && num >= 0 && num <= 255;
    });
  };

  const validatePort = (port: string): number | null => {
    if (!port || typeof port !== 'string') return null;
    const num = parseInt(port);
    if (isNaN(num) || num < 1 || num > 65535) return null;
    return num;
  };

  const validateName = (name: string): boolean => {
    if (!name || typeof name !== 'string') return false;
    return name.length > 0 && /^[a-zA-Z0-9-]+$/.test(name);
  };

  const handleConnect = async () => {
    setError('');
    setIsConnecting(true);
    console.log('Attempting to connect with:', { name, ip, port });

    if (!validateName(name)) {
      setError('Username must contain only letters, numbers, or hyphens');
      setIsConnecting(false);
      return;
    }

    if (!validateIp(ip)) {
      setError('IP must have 4 octets (xxx.xxx.xxx.xxx)');
      setIsConnecting(false);
      return;
    }

    const portNum = validatePort(port);
    if (!portNum) {
      setError('Port must be a valid number between 1-65535');
      setIsConnecting(false);
      return;
    }

    // Save settings before connecting
    updateConnectionSettings({ username: name, ip, port });
    console.log('Calling onConnect with:', name, ip, portNum);
    try {
      await onConnect(name, ip, portNum);
    } catch (err) {
      console.error('Connect failed in Splash:', err);
      setError(err instanceof Error ? err.message : 'Connection failed');
    } finally {
      setIsConnecting(false);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleConnect();
    }
  };

  const handleServerToggle = async () => {
    setIsServerLoading(true);
    try {
      if (serverState.status === 'Running') {
        await serverManagerActions.stopServer();
      } else {
        await serverManagerActions.startServer();
      }
    } catch (err) {
      console.error('Server toggle failed:', err);
      setError(err instanceof Error ? err.message : 'Server operation failed');
    } finally {
      setIsServerLoading(false);
    }
  };

  // Initialize server manager on mount
  useEffect(() => {
    serverManagerActions.initialize();
  }, []);

  const isServerRunning = serverState.status === 'Running';
  const isServerTransitioning = serverState.status === 'Starting' || serverState.status === 'Stopping' || isServerLoading;

  return (
    <div className="h-screen flex items-center justify-center" style={{ backgroundColor: 'var(--color-background)' }}>
      <div className="w-full max-w-md px-8">
        <h1 className="text-6xl font-bold text-center mb-8" style={{ color: 'var(--color-text)' }}>
          BuboCore
        </h1>

        {/* Server Status Indicator */}
        <div className="flex items-center justify-center gap-2 mb-8">
          <div 
            className="w-3 h-3 rounded-full"
            style={{ backgroundColor: getServerStatusColor(serverState.status) }}
          />
          <span className="text-sm" style={{ color: 'var(--color-muted)' }}>
            Server: {typeof serverState.status === 'string' ? serverState.status : 'Error'}
          </span>
        </div>

        <div className="space-y-4">
          <div>
            <label className="block text-sm font-medium mb-1" style={{ color: 'var(--color-muted)' }}>
              Username
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              onKeyDown={handleKeyDown}
              className="w-full px-3 py-2 border-2 focus:outline-none focus:ring-2"
              style={{
                backgroundColor: 'var(--color-surface)',
                borderColor: 'var(--color-border)',
                color: 'var(--color-text)'
              }}
              placeholder="Enter username"
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-1" style={{ color: 'var(--color-muted)' }}>
              IP Address
            </label>
            <input
              type="text"
              value={ip}
              onChange={(e) => setIp(e.target.value)}
              onKeyDown={handleKeyDown}
              className="w-full px-3 py-2 border-2 focus:outline-none focus:ring-2"
              style={{
                backgroundColor: 'var(--color-surface)',
                borderColor: 'var(--color-border)',
                color: 'var(--color-text)'
              }}
              placeholder="127.0.0.1"
            />
          </div>

          <div>
            <label className="block text-sm font-medium mb-1" style={{ color: 'var(--color-muted)' }}>
              Port
            </label>
            <input
              type="text"
              value={port}
              onChange={(e) => setPort(e.target.value)}
              onKeyDown={handleKeyDown}
              className="w-full px-3 py-2 border-2 focus:outline-none focus:ring-2"
              style={{
                backgroundColor: 'var(--color-surface)',
                borderColor: 'var(--color-border)',
                color: 'var(--color-text)'
              }}
              placeholder="8080"
            />
          </div>

          {(error || externalError) && (
            <div className="text-sm text-red-500 text-center">
              {error || externalError}
            </div>
          )}

          <button
            onClick={handleConnect}
            disabled={isConnecting}
            className="w-full py-2 px-4 text-white font-medium transition-colors hover:opacity-90 mt-6 disabled:opacity-50"
            style={{ backgroundColor: 'var(--color-primary)' }}
          >
            {isConnecting ? 'Connecting...' : 'Connect'}
          </button>

          <div className="flex gap-2 mt-2">
            <button
              onClick={handleServerToggle}
              disabled={isServerTransitioning}
              className="flex-1 py-2 px-4 border font-medium transition-colors hover:opacity-90 flex items-center justify-center gap-2"
              style={{ 
                borderColor: isServerRunning ? 'var(--color-error)' : 'var(--color-success)',
                color: isServerRunning ? 'var(--color-error)' : 'var(--color-success)',
                backgroundColor: 'transparent'
              }}
            >
              {isServerTransitioning ? (
                <Loader2 size={16} className="animate-spin" />
              ) : isServerRunning ? (
                <Square size={16} />
              ) : (
                <Play size={16} />
              )}
              {isServerTransitioning ? 'Processing...' : isServerRunning ? 'Stop Server' : 'Start Server'}
            </button>

            <button
              onClick={() => serverManagerActions.show()}
              className="py-2 px-4 border font-medium transition-colors hover:opacity-90 flex items-center justify-center gap-2"
              style={{ 
                borderColor: 'var(--color-border)',
                color: 'var(--color-text)',
                backgroundColor: 'transparent'
              }}
            >
              <Settings size={16} />
              Configure
            </button>
          </div>

          <p className="text-sm text-center mt-4" style={{ color: 'var(--color-muted)' }}>
            Press TAB to switch fields, ENTER to connect
          </p>
        </div>
      </div>
    </div>
  );
};