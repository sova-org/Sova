import React, { useEffect, useState } from 'react';
import { Play, Square, Settings, LogOut } from 'lucide-react';
import { useLinkClock } from '../hooks/useLinkClock';
import { useStore } from '@nanostores/react';
import { playbackStore } from '../stores/sceneStore';

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
  const playback = useStore(playbackStore);
  const isPlaying = playback.isPlaying;
  const { phase, quantum, tempo, setTempo } = useLinkClock(isPlaying);
  const [isHovering, setIsHovering] = useState(false);
  const [hoverSide, setHoverSide] = useState<'left' | 'right'>('left');

  // Watch playback state changes
  useEffect(() => {
    // Playback state changed
  }, [isPlaying, phase, quantum, tempo, playback]);

  const handleTempoBarMouseMove = (e: React.MouseEvent<HTMLDivElement>) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const width = rect.width;
    const isRightSide = x > width / 2;
    setHoverSide(isRightSide ? 'right' : 'left');
  };

  const handleTempoBarClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const isShiftPressed = e.shiftKey;
    const increment = isShiftPressed ? 5 : 1;
    const newTempo = hoverSide === 'right' 
      ? Math.min(200, tempo + increment)  // Cap at 200 BPM
      : Math.max(60, tempo - increment);   // Minimum 60 BPM
    
    setTempo(newTempo);
  };

  const handlePlay = async () => {
    try {
      await client.sendMessage({ TransportStart: "Immediate" });
    } catch (error) {
      console.error('Failed to send TransportStart:', error);
    }
  };

  const handleStop = async () => {
    try {
      await client.sendMessage({ TransportStop: "Immediate" });
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
                className="p-2 border-2 transition-all"
                style={{ 
                  borderColor: isPlaying ? 'var(--color-success)' : 'var(--color-border)',
                  backgroundColor: isPlaying ? 'var(--color-success)' : 'transparent',
                  color: isPlaying ? 'white' : 'var(--color-text)'
                }}
                title="Start playback"
              >
                <Play size={16} fill={isPlaying ? 'white' : 'none'} />
              </button>
              <button
                onClick={handleStop}
                className="p-2 border-2 transition-all"
                style={{ 
                  borderColor: !isPlaying ? 'var(--color-error)' : 'var(--color-border)',
                  backgroundColor: !isPlaying ? 'var(--color-error)' : 'transparent',
                  color: !isPlaying ? 'white' : 'var(--color-text)'
                }}
                title="Stop playback"
              >
                <Square size={16} fill={!isPlaying ? 'white' : 'none'} />
              </button>
              <div 
                className="w-40 h-6 overflow-hidden relative border cursor-pointer select-none" 
                style={{ 
                  backgroundColor: 'var(--color-background)',
                  borderColor: isPlaying ? 'var(--color-success)' : 'var(--color-error)'
                }}
                onMouseEnter={() => setIsHovering(true)}
                onMouseLeave={() => setIsHovering(false)}
                onMouseMove={handleTempoBarMouseMove}
                onClick={handleTempoBarClick}
                title={`Click to adjust tempo (${hoverSide === 'right' ? '+' : '-'}1 BPM, Shift for ${hoverSide === 'right' ? '+' : '-'}5)`}
              >
                <div 
                  className="h-full transition-colors duration-300"
                  style={{ 
                    width: `${Math.max(0, Math.min(100, (phase / quantum) * 100))}%`,
                    backgroundColor: isPlaying ? 'var(--color-success)' : 'var(--color-error)'
                  }}
                />
                
                {/* Hover indicators */}
                {isHovering && (
                  <>
                    {/* Left button - aligned with phase bar edges */}
                    <div 
                      className="absolute top-0 left-0 bottom-0 w-6 flex items-center justify-center text-sm font-bold transition-all"
                      style={{ 
                        backgroundColor: hoverSide === 'left' ? 'var(--color-surface)' : 'transparent',
                        color: 'var(--color-text)',
                        border: hoverSide === 'left' ? '1px solid var(--color-border)' : 'none',
                        opacity: hoverSide === 'left' ? 1 : 0.5
                      }}
                    >
                      −
                    </div>
                    {/* Right button - aligned with phase bar edges */}
                    <div 
                      className="absolute top-0 right-0 bottom-0 w-6 flex items-center justify-center text-sm font-bold transition-all"
                      style={{ 
                        backgroundColor: hoverSide === 'right' ? 'var(--color-surface)' : 'transparent',
                        color: 'var(--color-text)',
                        border: hoverSide === 'right' ? '1px solid var(--color-border)' : 'none',
                        opacity: hoverSide === 'right' ? 1 : 0.5
                      }}
                    >
                      +
                    </div>
                  </>
                )}
                
                <div className="absolute inset-0 flex items-center justify-center text-xs font-bold pointer-events-none" style={{ 
                  color: 'var(--color-text)',
                  textShadow: '0 0 2px rgba(0,0,0,0.5)'
                }}>
                  {isPlaying ? '▶' : '■'} | {tempo.toFixed(0)} BPM
                </div>
              </div>
            </>
          )}
          
          <button
            onClick={onToggleOptions}
            className="p-2 border-2 transition-all hover:opacity-80"
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
              className="px-3 py-1.5 text-white text-sm transition-colors hover:opacity-90"
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