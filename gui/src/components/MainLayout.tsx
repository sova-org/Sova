import React, { useState, useEffect, useRef } from 'react';
import { TopBar } from './TopBar';
import { FooterBar } from './FooterBar';
import { CodeEditor } from './CodeEditor';
import { OptionsPanel } from './OptionsPanel';
import { Splash } from './Splash';
import { GridComponent } from './GridComponent';
import { CommandPalette } from './CommandPalette';
import { BuboCoreClient } from '../client';
import { handleServerMessage, peersStore } from '../stores/sceneStore';
import { optionsPanelStore, setOptionsPanelSize, setOptionsPanelPosition } from '../stores/optionsPanelStore';
import { ResizeHandle } from './ResizeHandle';
import { useStore } from '@nanostores/react';
import { Grid3X3, Code, SplitSquareHorizontal } from 'lucide-react';

export const MainLayout: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [client] = useState(() => new BuboCoreClient());
  const [connectionError, setConnectionError] = useState<string>('');
  const [isOptionsPanelOpen, setIsOptionsPanelOpen] = useState(false);
  const [editorContent, setEditorContent] = useState('// Welcome to BuboCore Editor\n// Start typing your code here...\n');
  const [currentView, setCurrentView] = useState<'editor' | 'grid' | 'split'>('editor');
  const optionsPanelState = useStore(optionsPanelStore);
  const [serverAddress, setServerAddress] = useState<string>('');
  const [username, setUsername] = useState<string>('User');
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  
  // Use reactive peer store instead of local state
  const peers = useStore(peersStore);
  const peerCount = peers.peerList.length;
  
  // Track original size during resize
  const [originalSize, setOriginalSize] = useState({ width: 0, height: 0 });
  const [isResizing, setIsResizing] = useState(false);
  const panelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!client || !isConnected) return;

    const unsubscribe = client.onMessage(handleServerMessage);

    return unsubscribe;
  }, [client, isConnected]);

  const handleConnect = async (name: string, ip: string, port: number): Promise<void> => {
    setConnectionError('');
    setServerAddress(`${ip}:${port}`);
    setUsername(name);
    await client.connect(ip, port);
    await client.sendMessage({ SetName: name });
    setIsConnected(true);
  };

  const handleDisconnect = async () => {
    try {
      await client.disconnect();
      setIsConnected(false);
      setServerAddress('');
    } catch (error) {
      console.error('Failed to disconnect:', error);
    }
  };

  if (!isConnected) {
    return <Splash onConnect={handleConnect} error={connectionError} />;
  }

  const getMainContentWidth = () => {
    const baseWidth = window.innerWidth;
    return currentView === 'split' ? baseWidth / 2 : baseWidth;
  };

  const getMainContentHeight = () => {
    return window.innerHeight - 48 - 24; // Account for topbar (48px) and footer (24px)
  };

  return (
    <>
      <div className="h-screen flex flex-col" style={{ backgroundColor: 'var(--color-background)' }}>
        <TopBar
          isConnected={isConnected}
          onConnect={() => handleConnect('User', '127.0.0.1', 8080)}
          onDisconnect={handleDisconnect}
          onToggleOptions={() => setIsOptionsPanelOpen(!isOptionsPanelOpen)}
          client={client}
        />
        
        
        <div className="flex-1 flex overflow-hidden">
          <div className="flex-1 flex">
            {/* Editor View */}
            {(currentView === 'editor' || currentView === 'split') && (
              <div 
                className="relative flex flex-col"
                style={{ 
                  width: currentView === 'split' ? '50%' : '100%'
                }}
              >
                <CodeEditor
                  initialContent={editorContent}
                  onChange={setEditorContent}
                  className="flex-1"
                />
                
                {/* Floating Action Buttons */}
                <div className="absolute top-4 right-4 flex flex-col space-y-2 z-10">
                  {currentView === 'editor' && (
                    <>
                      <button
                        onClick={() => setCurrentView('split')}
                        className="w-10 h-10 shadow-lg hover:shadow-xl transition-shadow flex items-center justify-center"
                        style={{
                          backgroundColor: 'var(--color-primary)',
                          color: 'var(--color-background)'
                        }}
                        title="Split View"
                      >
                        <SplitSquareHorizontal size={16} />
                      </button>
                      <button
                        onClick={() => setCurrentView('grid')}
                        className="w-8 h-8 shadow-md hover:shadow-lg transition-shadow flex items-center justify-center"
                        style={{
                          backgroundColor: 'var(--color-surface)',
                          color: 'var(--color-text)',
                          border: '1px solid var(--color-border)'
                        }}
                        title="Grid Only"
                      >
                        <Grid3X3 size={14} />
                      </button>
                    </>
                  )}
                  {currentView === 'split' && (
                    <button
                      onClick={() => setCurrentView('editor')}
                      className="w-10 h-10 shadow-lg hover:shadow-xl transition-shadow flex items-center justify-center"
                      style={{
                        backgroundColor: 'var(--color-secondary)',
                        color: 'var(--color-background)'
                      }}
                      title="Editor Only"
                    >
                      <Code size={16} />
                    </button>
                  )}
                </div>
              </div>
            )}
            
            {/* Grid View */}
            {(currentView === 'grid' || currentView === 'split') && (
              <div 
                className="relative flex flex-col"
                style={{ 
                  width: currentView === 'split' ? '50%' : '100%'
                }}
              >
                <GridComponent
                  width={getMainContentWidth()}
                  height={getMainContentHeight()}
                  client={client}
                />
                
                {/* Floating Action Buttons */}
                <div className="absolute top-4 right-4 flex flex-col space-y-2 z-10">
                  {currentView === 'grid' && (
                    <>
                      <button
                        onClick={() => setCurrentView('split')}
                        className="w-10 h-10 shadow-lg hover:shadow-xl transition-shadow flex items-center justify-center"
                        style={{
                          backgroundColor: 'var(--color-primary)',
                          color: 'var(--color-background)'
                        }}
                        title="Split View"
                      >
                        <SplitSquareHorizontal size={16} />
                      </button>
                      <button
                        onClick={() => setCurrentView('editor')}
                        className="w-8 h-8 shadow-md hover:shadow-lg transition-shadow flex items-center justify-center"
                        style={{
                          backgroundColor: 'var(--color-surface)',
                          color: 'var(--color-text)',
                          border: '1px solid var(--color-border)'
                        }}
                        title="Editor Only"
                      >
                        <Code size={14} />
                      </button>
                    </>
                  )}
                  {currentView === 'split' && (
                    <button
                      onClick={() => setCurrentView('grid')}
                      className="w-10 h-10 shadow-lg hover:shadow-xl transition-shadow flex items-center justify-center"
                      style={{
                        backgroundColor: 'var(--color-secondary)',
                        color: 'var(--color-background)'
                      }}
                      title="Grid Only"
                    >
                      <Grid3X3 size={16} />
                    </button>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>
        
        {/* Footer Bar */}
        <FooterBar 
          isConnected={isConnected}
          peerCount={peerCount}
          serverAddress={serverAddress}
          username={username}
          onUsernameChange={async (newUsername) => {
            setUsername(newUsername);
            if (isConnected) {
              await client.sendMessage({ SetName: newUsername });
            }
          }}
        />
      </div>
      
      {/* Options Panel - Overlay with positioning */}
      {isOptionsPanelOpen && (
        <>
          <div 
            className="fixed inset-0 z-40"
            onClick={() => setIsOptionsPanelOpen(false)}
            style={{ backgroundColor: 'transparent' }}
          />
          <div 
            ref={panelRef}
            className={`fixed z-50 shadow-2xl ${isResizing ? '' : 'transition-all duration-300 ease-in-out'}`}
            style={{
              top: optionsPanelState.position === 'bottom' ? 'auto' : '48px',
              right: optionsPanelState.position === 'right' ? 0 : optionsPanelState.position === 'bottom' ? 0 : 'auto',
              bottom: optionsPanelState.position === 'bottom' ? 0 : 'auto',
              left: optionsPanelState.position === 'left' ? 0 : optionsPanelState.position === 'bottom' ? 0 : 'auto',
              width: optionsPanelState.position === 'bottom' ? '100%' : `${optionsPanelState.width}px`,
              height: optionsPanelState.position === 'bottom' ? `${optionsPanelState.height}px` : 'calc(100% - 48px - 24px)',
              minWidth: optionsPanelState.position === 'bottom' ? '100%' : '300px',
              maxWidth: optionsPanelState.position === 'bottom' ? '100%' : '80vw',
              minHeight: optionsPanelState.position === 'bottom' ? '200px' : 'auto',
              maxHeight: optionsPanelState.position === 'bottom' ? '60vh' : 'calc(100% - 48px - 24px)',
            }}
          >
            <div className="relative w-full h-full">
              <OptionsPanel 
                onClose={() => setIsOptionsPanelOpen(false)}
                position={optionsPanelState.position}
                onPositionChange={setOptionsPanelPosition}
              />
              
              {/* Resize Handles */}
              {optionsPanelState.position === 'right' && (
                <ResizeHandle
                  direction="horizontal"
                  position="left"
                  onResizeStart={() => {
                    const currentSize = { width: optionsPanelState.width, height: optionsPanelState.height };
                    setOriginalSize(currentSize);
                    setIsResizing(true);
                  }}
                  onResize={(delta) => {
                    if (panelRef.current) {
                      const maxWidth = window.innerWidth * 0.8;
                      const newWidth = Math.max(300, Math.min(maxWidth, originalSize.width + delta));
                      panelRef.current.style.width = `${newWidth}px`;
                    }
                  }}
                  onResizeEnd={() => {
                    if (panelRef.current) {
                      const currentWidth = parseInt(panelRef.current.style.width) || optionsPanelState.width;
                      const maxWidth = window.innerWidth * 0.8;
                      const newWidth = Math.max(300, Math.min(maxWidth, currentWidth));
                      setOptionsPanelSize(newWidth, optionsPanelState.height);
                    }
                    setIsResizing(false);
                  }}
                />
              )}
              {optionsPanelState.position === 'left' && (
                <ResizeHandle
                  direction="horizontal"
                  position="right"
                  onResizeStart={() => {
                    const currentSize = { width: optionsPanelState.width, height: optionsPanelState.height };
                    setOriginalSize(currentSize);
                    setIsResizing(true);
                  }}
                  onResize={(delta) => {
                    if (panelRef.current) {
                      const maxWidth = window.innerWidth * 0.8;
                      const newWidth = Math.max(300, Math.min(maxWidth, originalSize.width + delta));
                      panelRef.current.style.width = `${newWidth}px`;
                    }
                  }}
                  onResizeEnd={() => {
                    if (panelRef.current) {
                      const currentWidth = parseInt(panelRef.current.style.width) || optionsPanelState.width;
                      const maxWidth = window.innerWidth * 0.8;
                      const newWidth = Math.max(300, Math.min(maxWidth, currentWidth));
                      setOptionsPanelSize(newWidth, optionsPanelState.height);
                    }
                    setIsResizing(false);
                  }}
                />
              )}
              {optionsPanelState.position === 'bottom' && (
                <ResizeHandle
                  direction="vertical"
                  position="top"
                  onResizeStart={() => {
                    const currentSize = { width: optionsPanelState.width, height: optionsPanelState.height };
                    setOriginalSize(currentSize);
                    setIsResizing(true);
                  }}
                  onResize={(delta) => {
                    if (panelRef.current) {
                      const maxHeight = window.innerHeight * 0.6;
                      const newHeight = Math.max(200, Math.min(maxHeight, originalSize.height + delta));
                      panelRef.current.style.height = `${newHeight}px`;
                    }
                  }}
                  onResizeEnd={() => {
                    if (panelRef.current) {
                      const currentHeight = parseInt(panelRef.current.style.height) || optionsPanelState.height;
                      const maxHeight = window.innerHeight * 0.6;
                      const newHeight = Math.max(200, Math.min(maxHeight, currentHeight));
                      setOptionsPanelSize(optionsPanelState.width, newHeight);
                    }
                    setIsResizing(false);
                  }}
                />
              )}
            </div>
          </div>
        </>
      )}
      
      {/* Command Palette */}
      <CommandPalette
        open={isCommandPaletteOpen}
        onOpenChange={setIsCommandPaletteOpen}
        client={client}
        onViewChange={setCurrentView}
        currentView={currentView}
        isConnected={isConnected}
        onConnect={() => handleConnect('User', '127.0.0.1', 8080)}
        onDisconnect={handleDisconnect}
      />
    </>
  );
};