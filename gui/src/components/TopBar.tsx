import React, { useEffect, useState } from 'react';
import { Play, Square, Settings, LogOut, Grid3X3, Code, SplitSquareHorizontal, HelpCircle } from 'lucide-react';
import { useLinkClock } from '../hooks/useLinkClock';
import { useStore } from '@nanostores/react';
import { playbackStore } from '../stores/sceneStore';
import { SplitToggleButton } from './SplitToggleButton';
import { layoutStore } from '../stores/layoutStore';

interface TopBarProps {
  isConnected: boolean;
  onConnect: () => void;
  onDisconnect: () => void;
  onToggleOptions: () => void;
  client: any;
  optionsPanelPosition?: 'left' | 'right' | 'bottom';
  onChangeOptionsPanelPosition?: (position: 'left' | 'right' | 'bottom') => void;
  currentView: 'editor' | 'grid' | 'split';
  onViewChange: (view: 'editor' | 'grid' | 'split') => void;
  isHelpOpen?: boolean;
  onToggleHelp?: () => void;
}

export const TopBar: React.FC<TopBarProps> = ({ 
  isConnected, 
  onConnect, 
  onDisconnect,
  onToggleOptions,
  client,
  currentView,
  onViewChange,
  isHelpOpen,
  onToggleHelp
}) => {
  const playback = useStore(playbackStore);
  const isPlaying = playback.isPlaying;
  const layout = useStore(layoutStore);
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
        <div className="flex items-center space-x-4">
          <h1 className="text-lg font-semibold" style={{ color: 'var(--color-text)', fontFamily: 'inherit !important' }}>
            Sova
          </h1>
          
          {/* View switching buttons */}
          {isConnected && (
            <div className="flex items-center border-2" style={{ borderColor: 'var(--color-border)' }}>
              <button
                onClick={() => onViewChange('editor')}
                className="p-2 transition-all"
                style={{ 
                  backgroundColor: currentView === 'editor' ? 'var(--color-primary)' : 'transparent',
                  color: currentView === 'editor' ? 'white' : 'var(--color-text)'
                }}
                title="Editor only"
              >
                <Code size={16} />
              </button>
              <button
                onClick={() => onViewChange('split')}
                className="p-2 border-x-2 transition-all"
                style={{ 
                  backgroundColor: currentView === 'split' ? 'var(--color-primary)' : 'transparent',
                  color: currentView === 'split' ? 'white' : 'var(--color-text)',
                  borderColor: 'var(--color-border)'
                }}
                title="Split view"
              >
                <SplitSquareHorizontal size={16} />
              </button>
              <button
                onClick={() => onViewChange('grid')}
                className="p-2 transition-all"
                style={{ 
                  backgroundColor: currentView === 'grid' ? 'var(--color-primary)' : 'transparent',
                  color: currentView === 'grid' ? 'white' : 'var(--color-text)'
                }}
                title="Grid only"
              >
                <Grid3X3 size={16} />
              </button>
            </div>
          )}
          
          {/* Split orientation toggle - only shown in split view */}
          {isConnected && currentView === 'split' && (
            <SplitToggleButton 
              orientation={layout.splitOrientation}
              className="border-2" 
              style={{ borderColor: 'var(--color-border)', color: 'var(--color-text)' }}
            />
          )}
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
                        opacity: hoverSide === 'left' ? 1 : 0.5,
                        fontFamily: 'inherit'
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
                        opacity: hoverSide === 'right' ? 1 : 0.5,
                        fontFamily: 'inherit'
                      }}
                    >
                      +
                    </div>
                  </>
                )}
                
                <div className="absolute inset-0 flex items-center justify-center text-xs font-bold pointer-events-none" style={{ 
                  color: 'var(--color-text)',
                  textShadow: '0 0 2px rgba(0,0,0,0.5)',
                  fontFamily: 'inherit'
                }}>
                  {isPlaying ? '▶' : '■'} | {tempo.toFixed(0)} BPM
                </div>
              </div>
            </>
          )}
          
          <button
            onClick={onToggleHelp}
            className="p-2 border-2 transition-all hover:opacity-80"
            style={{ 
              borderColor: isHelpOpen ? 'var(--color-primary)' : 'var(--color-border)',
              backgroundColor: isHelpOpen ? 'var(--color-primary)' : 'transparent',
              color: isHelpOpen ? 'white' : 'var(--color-text)'
            }}
            title="Help"
          >
            <HelpCircle size={16} />
          </button>
          
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
              style={{ backgroundColor: 'var(--color-primary)', fontFamily: 'inherit' }}
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