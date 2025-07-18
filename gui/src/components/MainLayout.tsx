import React, { useState, useEffect, useRef } from 'react';
import { TopBar } from './TopBar';
import { FooterBar } from './FooterBar';
import { CodeEditor } from './CodeEditor';
import { OptionsPanel } from './OptionsPanel';
import { Splash } from './Splash';
import { GridComponent } from './GridComponent';
import { CommandPalette } from './CommandPalette';
import { BuboCoreClient } from '../client';
import { handleServerMessage, peersStore, scriptEditorStore, sceneStore, setScriptLanguage, updateGridSelection } from '../stores/sceneStore';
import { clearRemoteLogs } from '../stores/remoteLogsStore';
import { updateConnectionState } from '../stores/connectionStateStore';
import { getAvailableLanguages } from '../languages';
import { optionsPanelStore, setOptionsPanelSize, setOptionsPanelPosition } from '../stores/optionsPanelStore';
// import { serverManagerStore } from '../stores/serverManagerStore';
// import { serverConfigStore } from '../stores/serverConfigStore';
import { ResizeHandle } from './ResizeHandle';
import { SplitResizeHandle } from './SplitResizeHandle';
import { useStore } from '@nanostores/react';
import { layoutStore, getSplitRatio } from '../stores/layoutStore';

export const MainLayout: React.FC = () => {
  const [isConnected, setIsConnected] = useState(false);
  const [client] = useState(() => new BuboCoreClient());
  const [connectionError, setConnectionError] = useState<string>('');
  const [isOptionsPanelOpen, setIsOptionsPanelOpen] = useState(false);
  const [editorContent, setEditorContent] = useState('// Welcome to BuboCore Editor\n// Start typing your code here...\n');
  const [currentView, setCurrentView] = useState<'editor' | 'grid' | 'split'>('split');
  const optionsPanelState = useStore(optionsPanelStore);
  const layout = useStore(layoutStore);
  const [serverAddress, setServerAddress] = useState<string>('');
  const [username, setUsername] = useState<string>('User');
  const [isCommandPaletteOpen, setIsCommandPaletteOpen] = useState(false);
  
  // Window dimensions state
  const [windowDimensions, setWindowDimensions] = useState({
    width: window.innerWidth,
    height: window.innerHeight
  });
  
  // Use reactive peer store instead of local state
  const peers = useStore(peersStore);
  const peerCount = peers.peerList.length;
  
  // Script editor state
  const scriptEditor = useStore(scriptEditorStore);
  const scene = useStore(sceneStore);
  
  // Server manager state
  // const serverState = useStore(serverManagerStore);
  // const serverConfig = useStore(serverConfigStore);
  
  const panelRef = useRef<HTMLDivElement>(null);
  const mainContentRef = useRef<HTMLDivElement>(null);

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

  // Handle window resize
  useEffect(() => {
    const handleResize = () => {
      setWindowDimensions({
        width: window.innerWidth,
        height: window.innerHeight
      });
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  // DISABLED: Auto-connect to local server when it starts
  // Commented out as requested - users prefer to connect manually
  /*
  useEffect(() => {
    // Only auto-connect if:
    // 1. We're not already connected
    // 2. A local server is running
    // 3. We have a process ID (meaning we started it)
    if (!isConnected && serverState.status === 'Running' && serverState.process_id) {
      const autoConnect = async () => {
        try {
          console.log('Auto-connecting to local server...');
          await handleConnect(username, serverConfig.ip, serverConfig.port);
          console.log('Auto-connected successfully');
        } catch (error) {
          console.error('Auto-connect failed:', error);
          setConnectionError('Failed to auto-connect to local server');
        }
      };
      
      // Small delay to ensure server is fully ready
      const timeout = setTimeout(autoConnect, 1000);
      return () => clearTimeout(timeout);
    }
    
    // Return undefined if no cleanup needed
    return undefined;
  }, [isConnected, serverState.status, serverState.process_id, serverConfig.ip, serverConfig.port, username]);
  */

  const handleConnect = async (name: string, ip: string, port: number): Promise<void> => {
    setConnectionError('');
    setServerAddress(`${ip}:${port}`);
    setUsername(name);
    await client.connect(ip, port);
    await client.sendMessage({ SetName: name });
    setIsConnected(true);
    updateConnectionState(true, ip, port);
  };

  // Auto-focus on first available tile after scene data is loaded
  useEffect(() => {
    if (scene && isConnected && !scriptEditor.selectedFrame) {
      const firstLine = scene.lines[0];
      if (firstLine && firstLine.frames.length > 0) {
        // Update grid selection to show first frame as selected
        updateGridSelection({
          start: [0, 0],
          end: [0, 0]
        });
        
        // Request script content for the first frame
        const handleFirstFrameSelection = async () => {
          try {
            await client.sendMessage({ GetScript: [0, 0] });
          } catch (error) {
            console.error('Failed to auto-select first frame:', error);
          }
        };
        
        handleFirstFrameSelection();
      }
    }
  }, [scene, isConnected, scriptEditor.selectedFrame, client]);

  const handleDisconnect = async () => {
    try {
      await client.disconnect();
      setIsConnected(false);
      setServerAddress('');
      updateConnectionState(false);
      clearRemoteLogs(); // Clear remote server logs when disconnecting
    } catch (error) {
      console.error('Failed to disconnect:', error);
    }
  };

  // Monitor connection health
  useEffect(() => {
    if (!isConnected) return;

    const checkConnection = async () => {
      try {
        const connected = await client.isConnected();
        if (!connected) {
          console.log('Server connection lost, returning to splash');
          // Ensure proper cleanup of the client connection
          try {
            await client.disconnect();
          } catch (error) {
            console.error('Failed to disconnect client during connection check:', error);
          }
          setIsConnected(false);
          setServerAddress('');
          setConnectionError('Connection to server lost');
          updateConnectionState(false);
          clearRemoteLogs(); // Clear remote server logs when connection is lost
        }
      } catch (error) {
        console.error('Connection check failed:', error);
        // Ensure proper cleanup of the client connection
        try {
          await client.disconnect();
        } catch (disconnectError) {
          console.error('Failed to disconnect client during connection check:', disconnectError);
        }
        setIsConnected(false);
        setServerAddress('');
        setConnectionError('Connection to server lost');
        clearRemoteLogs(); // Clear remote server logs when connection check fails
      }
    };

    // Check connection every 2 seconds
    const interval = setInterval(checkConnection, 2000);
    
    return () => clearInterval(interval);
  }, [isConnected, client]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (isConnected) {
        client.disconnect().catch(error => {
          console.error('Failed to disconnect client during cleanup:', error);
        });
      }
    };
  }, [client, isConnected]);
  
  // Get current language from the selected frame
  const currentLanguage = (() => {
    if (!scene || !scriptEditor.selectedFrame) return 'bali';
    const { line_idx, frame_idx } = scriptEditor.selectedFrame;
    const line = scene.lines[line_idx];
    if (!line) return 'bali';
    const script = line.scripts.find(s => s.index === frame_idx);
    return script?.lang || 'bali';
  })();
  
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

  const handleLanguageChange = async (language: string) => {
    if (!client || !scriptEditor.selectedFrame) return;
    
    const { line_idx, frame_idx } = scriptEditor.selectedFrame;
    try {
      const message = setScriptLanguage(line_idx, frame_idx, language);
      await client.sendMessage(message);
    } catch (error) {
      console.error('Failed to set script language:', error);
    }
  };

  if (!isConnected) {
    return <Splash onConnect={handleConnect} error={connectionError} />;
  }

  const getGridWidth = () => {
    const baseWidth = windowDimensions.width;
    if (currentView === 'split') {
      const ratio = getSplitRatio();
      return layout.splitOrientation === 'horizontal' ? baseWidth : baseWidth * (1 - ratio);
    }
    return baseWidth;
  };

  const getGridHeight = () => {
    const baseHeight = windowDimensions.height - 48 - 24; // Account for topbar (48px) and footer (24px)
    if (currentView === 'split') {
      const ratio = getSplitRatio();
      return layout.splitOrientation === 'horizontal' ? baseHeight * (1 - ratio) : baseHeight;
    }
    return baseHeight;
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
        
        
        <div 
          className="flex-1 flex overflow-hidden relative"
          ref={mainContentRef}
          style={{
            flexDirection: currentView === 'split' && layout.splitOrientation === 'horizontal' ? 'column' : 'row'
          }}
        >
          {/* Editor View */}
          {(currentView === 'editor' || currentView === 'split') && (
            <div 
              className="relative flex flex-col"
              style={{ 
                width: currentView === 'split' ? 
                  (layout.splitOrientation === 'horizontal' ? '100%' : `${getSplitRatio() * 100}%`) : 
                  '100%',
                height: currentView === 'split' ? 
                  (layout.splitOrientation === 'horizontal' ? `${getSplitRatio() * 100}%` : '100%') : 
                  '100%'
              }}
            >
              <CodeEditor
                value={editorContent}
                onChange={setEditorContent}
                className="flex-1"
                onEvaluate={handleEvaluateScript}
                showEvaluateButton={!!scriptEditor.selectedFrame}
                language={currentLanguage}
                availableLanguages={getAvailableLanguages()}
                onLanguageChange={handleLanguageChange}
              />
            </div>
          )}
          
          {/* Resize Handle */}
          {currentView === 'split' && (
            <SplitResizeHandle
              orientation={layout.splitOrientation}
              containerRef={mainContentRef}
              className="z-20"
            />
          )}
          
          {/* Grid View */}
          {(currentView === 'grid' || currentView === 'split') && (
            <div 
              className="relative flex flex-col"
              style={{ 
                width: currentView === 'split' ? 
                  (layout.splitOrientation === 'horizontal' ? '100%' : `${(1 - getSplitRatio()) * 100}%`) : 
                  '100%',
                height: currentView === 'split' ? 
                  (layout.splitOrientation === 'horizontal' ? `${(1 - getSplitRatio()) * 100}%` : '100%') : 
                  '100%'
              }}
            >
              <GridComponent
                width={getGridWidth()}
                height={getGridHeight()}
                client={client}
              />
            </div>
          )}
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
          {!optionsPanelState.isPinned && (
            <div 
              className="fixed inset-0 z-40"
              onClick={() => setIsOptionsPanelOpen(false)}
              style={{ backgroundColor: 'transparent' }}
            />
          )}
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