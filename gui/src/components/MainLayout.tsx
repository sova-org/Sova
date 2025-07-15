import React, { useState, useEffect, useRef } from 'react';
import { TopBar } from './TopBar';
import { FooterBar } from './FooterBar';
import { CodeEditor } from './CodeEditor';
import { OptionsPanel } from './OptionsPanel';
import { Splash } from './Splash';
import { GridComponent } from './GridComponent';
import { CommandPalette } from './CommandPalette';
import { BuboCoreClient } from '../client';
import { handleServerMessage, peersStore, scriptEditorStore } from '../stores/sceneStore';
import { optionsPanelStore, setOptionsPanelSize, setOptionsPanelPosition } from '../stores/optionsPanelStore';
import { ResizeHandle } from './ResizeHandle';
import { useStore } from '@nanostores/react';

export const MainLayout: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [client] = useState(() => new BuboCoreClient());
  const [connectionError, setConnectionError] = useState<string>('');
  const [isOptionsPanelOpen, setIsOptionsPanelOpen] = useState(false);
  const [editorContent, setEditorContent] = useState('// Welcome to BuboCore Editor\n// Start typing your code here...\n');
  const [currentView, setCurrentView] = useState<'editor' | 'grid' | 'split'>('split');
  const optionsPanelState = useStore(optionsPanelStore);
  const [serverAddress, setServerAddress] = useState<string>('');
  const [username, setUsername] = useState<string>('User');
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  
  // Use reactive peer store instead of local state
  const peers = useStore(peersStore);
  const peerCount = peers.peerList.length;
  
  // Script editor state
  const scriptEditor = useStore(scriptEditorStore);
  
  const panelRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!client || !isConnected) return;

    const unsubscribe = client.onMessage(handleServerMessage);

    return unsubscribe;
  }, [client, isConnected]);
  
  // Update editor content when script is loaded
  useEffect(() => {
    if (scriptEditor.currentScript !== undefined) {
      setEditorContent(scriptEditor.currentScript);
    }
  }, [scriptEditor.currentScript]);

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
  
  const handleEvaluateScript = async () => {
    if (!client || !scriptEditor.selectedFrame) return;
    
    // Clear previous compilation error (like TUI does)
    scriptEditorStore.setKey('compilationError', null);
    
    const { line_idx, frame_idx } = scriptEditor.selectedFrame;
    try {
      await client.sendMessage({
        SetScript: [line_idx, frame_idx, editorContent, "Immediate"]
      });
      
      // Don't show intermediate message - wait for actual compilation result
      // The server will send either ScriptCompiled or CompilationErrorOccurred
    } catch (error) {
      console.error('Failed to update script:', error);
      // Create an error object similar to what server would send
      scriptEditorStore.setKey('compilationError', {
        lang: 'Network',
        info: 'Failed to send script to server: ' + error,
        from: 0,
        to: 0
      });
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
          currentView={currentView}
          onViewChange={setCurrentView}
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
                  value={editorContent}
                  onChange={setEditorContent}
                  className="flex-1"
                  onEvaluate={handleEvaluateScript}
                  showEvaluateButton={!!scriptEditor.selectedFrame}
                />
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
            className="fixed z-50 shadow-2xl transition-all duration-300 ease-in-out"
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
                  panelRef={panelRef}
                  onResizeEnd={(newWidth, newHeight) => {
                    setOptionsPanelSize(newWidth, newHeight);
                  }}
                />
              )}
              {optionsPanelState.position === 'left' && (
                <ResizeHandle
                  direction="horizontal"
                  position="right"
                  panelRef={panelRef}
                  onResizeEnd={(newWidth, newHeight) => {
                    setOptionsPanelSize(newWidth, newHeight);
                  }}
                />
              )}
              {optionsPanelState.position === 'bottom' && (
                <ResizeHandle
                  direction="vertical"
                  position="top"
                  panelRef={panelRef}
                  onResizeEnd={(newWidth, newHeight) => {
                    setOptionsPanelSize(newWidth, newHeight);
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