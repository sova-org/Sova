import React, { useState } from 'react';
import { TopBar } from './TopBar';
import { CodeEditor } from './CodeEditor';
import { OptionsPanel } from './OptionsPanel';
import { Splash } from './Splash';
import { BuboCoreClient } from '../client';

export const MainLayout: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [client] = useState(() => new BuboCoreClient());
  const [connectionError, setConnectionError] = useState<string>('');
  const [isOptionsPanelOpen, setIsOptionsPanelOpen] = useState(false);
  const [editorContent, setEditorContent] = useState('// Welcome to BuboCore Editor\n// Start typing your code here...\n');

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
        onToggleOptions={() => setIsOptionsPanelOpen(!isOptionsPanelOpen)}
        client={client}
      />
      
      <div className="flex-1 flex overflow-hidden">
        <div className="flex-1 flex flex-col">
          <CodeEditor
            initialContent={editorContent}
            onChange={setEditorContent}
            className="flex-1"
          />
        </div>
        
        {isOptionsPanelOpen && (
          <OptionsPanel onClose={() => setIsOptionsPanelOpen(false)} />
        )}
      </div>
    </div>
  );
};