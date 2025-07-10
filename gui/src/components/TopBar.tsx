import React, { useEffect, useState } from 'react';
import { Play, Square, Settings, LogOut } from 'lucide-react';
import { useLinkClock } from '../hooks/useLinkClock';

interface TopBarProps {
  isConnected: boolean;
  onConnect: () => void;
  onDisconnect: () => void;
  onToggleOptions: () => void;
  client: any;
  optionsPanelPosition?: 'left' | 'right' | 'bottom';
  onChangeOptionsPanelPosition?: (position: 'left' | 'right' | 'bottom') => void;
}

export const TopBar: React.FC<TopBarProps> = ({ 
  isConnected, 
  onConnect, 
  onDisconnect,
  onToggleOptions,
  client
}) => {
  const [isPlaying, setIsPlaying] = useState(false);
  const { phase, quantum } = useLinkClock(isPlaying);

  useEffect(() => {
    if (!client) return;

    const unsubscribe = client.onMessage((message: any) => {
      console.log('Received message:', message);
      if (message === 'TransportStarted') {
        console.log('Transport started');
        setIsPlaying(true);
      } else if (message === 'TransportStopped') {
        console.log('Transport stopped');
        setIsPlaying(false);
      } else if (message.Hello) {
        console.log('Hello message, is_playing:', message.Hello.is_playing);
        setIsPlaying(message.Hello.is_playing);
      }
    });

    return unsubscribe;
  }, [client]);

  const handlePlay = async () => {
    console.log('Play button clicked');
    try {
      await client.sendMessage({ TransportStart: "Immediate" });
      console.log('TransportStart message sent');
    } catch (error) {
      console.error('Failed to send TransportStart:', error);
    }
  };

  const handleStop = async () => {
    console.log('Stop button clicked');
    try {
      await client.sendMessage({ TransportStop: "Immediate" });
      console.log('TransportStop message sent');
    } catch (error) {
      console.error('Failed to send TransportStop:', error);
    }
  };
  return (
    <div className="h-12 border-b" style={{ backgroundColor: 'var(--color-surface)', borderColor: 'var(--color-border)' }}>
      <div className="flex items-center justify-between px-4 h-full">
        <div className="flex items-center">
          <h1 className="text-lg font-semibold" style={{ color: 'var(--color-text)' }}>
            Sova
          </h1>
        </div>
        
        <div className="flex items-center space-x-3">
          {isConnected && (
            <>
              <button
                onClick={handlePlay}
                className="p-2 rounded-md border-2 transition-all"
                style={{ 
                  borderColor: isPlaying ? 'var(--color-secondary)' : 'var(--color-border)',
                  backgroundColor: isPlaying ? 'var(--color-secondary)' : 'transparent',
                  color: isPlaying ? 'white' : 'var(--color-text)'
                }}
                title="Start playback"
              >
                <Play size={16} fill={isPlaying ? 'white' : 'none'} />
              </button>
              <button
                onClick={handleStop}
                className="p-2 rounded-md border-2 transition-all"
                style={{ 
                  borderColor: !isPlaying ? 'var(--color-muted)' : 'var(--color-border)',
                  backgroundColor: !isPlaying ? 'var(--color-muted)' : 'transparent',
                  color: !isPlaying ? 'white' : 'var(--color-text)'
                }}
                title="Stop playback"
              >
                <Square size={16} fill={!isPlaying ? 'white' : 'none'} />
              </button>
              <div className="w-40 h-6 rounded-sm overflow-hidden relative" style={{ backgroundColor: 'var(--color-background)' }}>
                <div 
                  className="h-full"
                  style={{ 
                    width: `${Math.max(0, Math.min(100, (phase / quantum) * 100))}%`,
                    backgroundColor: isPlaying ? 'var(--color-secondary)' : 'var(--color-muted)'
                  }}
                />
                <div className="absolute inset-0 flex items-center justify-center text-xs font-bold" style={{ color: 'var(--color-text)' }}>
                  {isPlaying ? '▶' : '■'} | {phase.toFixed(1)}/{quantum.toFixed(0)}
                </div>
              </div>
            </>
          )}
          
          <button
            onClick={onToggleOptions}
            className="p-2 rounded-md border-2 transition-all hover:opacity-80"
            style={{ 
              borderColor: 'var(--color-border)',
              backgroundColor: 'transparent',
              color: 'var(--color-text)'
            }}
            title="Options"
          >
            <Settings size={16} />
          </button>
          
          {isConnected ? (
            <button
              onClick={onDisconnect}
              className="p-2 transition-all hover:opacity-80"
              style={{ 
                color: 'var(--color-error)'
              }}
              title="Disconnect from server"
            >
              <LogOut size={16} />
            </button>
          ) : (
            <button
              onClick={onConnect}
              className="px-3 py-1.5 text-white text-sm rounded-md transition-colors hover:opacity-90"
              style={{ backgroundColor: 'var(--color-primary)' }}
              title="Connect to server"
            >
              Connect
            </button>
          )}
        </div>
      </div>
    </div>
  );
};