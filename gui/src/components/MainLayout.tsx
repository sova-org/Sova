import React, { useState } from 'react';
import { TopBar } from './TopBar';
import { MaterialColorPalette } from './MaterialColorPalette';
import { StylingLab } from './StylingLab';
import { Splash } from './Splash';
import { BuboCoreClient } from '../client';

export const MainLayout: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [client] = useState(() => new BuboCoreClient());
  const [connectionError, setConnectionError] = useState<string>('');

  const handleConnect = async (name: string, ip: string, port: number): Promise<void> => {
    setConnectionError('');
    await client.connect(ip, port);
    await client.sendMessage({ SetName: name });
    setIsConnected(true);
  };

  const handleDisconnect = async () => {
    try {
      await client.disconnect();
      setIsConnected(false);
    } catch (error) {
      console.error('Failed to disconnect:', error);
    }
  };

  if (!isConnected) {
    return <Splash onConnect={handleConnect} error={connectionError} />;
  }

  return (
    <div className="h-screen flex flex-col" style={{ backgroundColor: 'var(--color-background)' }}>
      <TopBar
        isConnected={isConnected}
        onConnect={() => handleConnect('User', '127.0.0.1', 8080)}
        onDisconnect={handleDisconnect}
        client={client}
      />
      
      <div className="flex-1 flex">
        <div className="flex-1 p-4">
          <MaterialColorPalette />
        </div>
        <div className="flex-1 p-4">
          <StylingLab />
        </div>
      </div>
    </div>
  );
};