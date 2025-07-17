import React, { useEffect, useRef, useState, useCallback } from 'react';
import { useStore } from '@nanostores/react';
import { serverManagerStore, serverManagerActions } from '../stores/serverManagerStore';
import { serverConfigStore } from '../stores/serverConfigStore';
import { remoteLogsStore, clearRemoteLogs } from '../stores/remoteLogsStore';
import { connectionStateStore, getLogDisplayMode } from '../stores/connectionStateStore';
import { $logs, $sourceType, LogSourceType, clearLogs } from '../stores/logManagerStore';
import { Trash2, Server, Globe, Link2, File, Network, ChevronDown } from 'lucide-react';


export const ServerLogsPanel: React.FC = () => {
  const serverState = useStore(serverManagerStore);
  const serverConfig = useStore(serverConfigStore);
  const remoteLogs = useStore(remoteLogsStore);
  const connectionState = useStore(connectionStateStore);
  const hybridLogs = useStore($logs);
  const sourceType = useStore($sourceType);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const logsContainerRef = useRef<HTMLDivElement>(null);
  const [isUserScrolling, setIsUserScrolling] = useState(false);
  const [showScrollToBottom, setShowScrollToBottom] = useState(false);
  const lastScrollTop = useRef(0);
  
  const displayMode = getLogDisplayMode();
  
  // Prevent unused variable warnings
  void isUserScrolling;
  void showScrollToBottom;

  useEffect(() => {
    // Refresh logs when component mounts
    serverManagerActions.refreshLogs();
  }, []);

  // Auto-scroll to bottom when new logs arrive, but only if user hasn't scrolled up
  useEffect(() => {
    // DISABLED for now to fix scroll issue
    // if (!isUserScrolling && logsEndRef.current) {
    //   setTimeout(() => {
    //     if (!isUserScrolling && logsEndRef.current) {
    //       logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    //     }
    //   }, 50);
    // }
  }, [hybridLogs.length]);

  // Handle scroll detection
  const handleScroll = useCallback((e: React.UIEvent<HTMLDivElement>) => {
    const container = e.currentTarget;
    const { scrollTop, scrollHeight, clientHeight } = container;
    
    // Detect if user is scrolling up (intentional scroll)
    const isScrollingUp = scrollTop < lastScrollTop.current;
    lastScrollTop.current = scrollTop;
    
    // Consider user is at bottom if within 5px of bottom (very strict)
    const isAtBottom = scrollHeight - scrollTop - clientHeight < 5;
    
    // If user scrolls up, disable auto-scroll
    if (isScrollingUp && !isAtBottom) {
      setIsUserScrolling(true);
      setShowScrollToBottom(true);
    } else if (isAtBottom) {
      // Only re-enable auto-scroll if user is truly at bottom
      setIsUserScrolling(false);
      setShowScrollToBottom(false);
    }
  }, []);

  // Scroll to bottom function
  const scrollToBottom = useCallback(() => {
    if (logsEndRef.current) {
      // Immediately set states to prevent interference during scroll
      setIsUserScrolling(false);
      setShowScrollToBottom(false);
      
      // Scroll to bottom
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, []);

  // Removed mergedLogs - use hybridLogs directly for better performance

  // Auto-scroll logic completely disabled - user controls scroll manually
  // useEffect(() => {
  //   if (!isUserScrolling && logsEndRef.current) {
  //     setTimeout(() => {
  //       if (!isUserScrolling && logsEndRef.current) {
  //         logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
  //       }
  //     }, 50);
  //   }
  // }, [mergedLogs, remoteLogs, serverState.logs, isUserScrolling]);

  const getLevelColor = (level: string) => {
    switch (level.toLowerCase()) {
      case 'error':
        return 'var(--color-error)';
      case 'warn':
      case 'warning':
        return 'var(--color-warning)';
      case 'info':
        return 'var(--color-primary)';
      case 'debug':
        return 'var(--color-muted)';
      default:
        return 'var(--color-text)';
    }
  };

  const handleClearLogs = () => {
    clearRemoteLogs();
    clearLogs(); // Clear hybrid logs
  };

  const renderLogEntry = useCallback((log: any, index: number, showSource: boolean = false) => {
    const timestamp = log.timestamp ? new Date(log.timestamp).toLocaleTimeString() : '';
    const source = log.source || 'network';
    const sourceIcon = source === 'file' ? <File size={12} /> : <Network size={12} />;
    
    return (
      <div key={index} className="flex gap-2">
        <span className="text-xs opacity-60 flex-shrink-0" style={{ color: 'var(--color-muted)' }}>
          {timestamp}
        </span>
        <span 
          className="text-xs font-medium flex-shrink-0 min-w-[50px]"
          style={{ color: getLevelColor(log.level) }}
        >
          [{log.level.toUpperCase()}]
        </span>
        {showSource && (
          <span 
            className="text-xs flex-shrink-0 flex items-center gap-1"
            style={{ color: 'var(--color-muted)' }}
          >
            {sourceIcon}
            [{source === 'file' ? 'F' : 'N'}]
          </span>
        )}
        <span className="flex-1 break-words">
          {log.message}
        </span>
      </div>
    );
  }, []);

  // Render log container with scroll management
  const renderLogContainer = useCallback((logs: any[], showSourceIcons: boolean = false) => {
    const maxVisibleItems = 100;
    const visibleLogs = logs.length <= maxVisibleItems ? logs : logs.slice(-maxVisibleItems);
    
    return (
      <div className="flex-1 relative">
        <div 
          ref={logsContainerRef}
          className="h-full overflow-y-auto p-3 font-mono text-sm border"
          style={{ 
            backgroundColor: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            color: 'var(--color-text)'
          }}
          onScroll={handleScroll}
        >
          {logs.length === 0 ? (
            <div className="text-center py-8" style={{ color: 'var(--color-muted)' }}>
              No logs available.
            </div>
          ) : (
            <div className="space-y-1">
              {visibleLogs.map((log, index) => renderLogEntry(log, index, showSourceIcons))}
              <div ref={logsEndRef} />
            </div>
          )}
        </div>
        
        {/* Scroll to bottom button - ALWAYS VISIBLE FOR NOW */}
        <button
          onClick={scrollToBottom}
          className="absolute bottom-4 right-4 flex items-center gap-1 px-3 py-2 text-xs border rounded-lg shadow-lg hover:opacity-90 transition-opacity"
          style={{ 
            borderColor: 'var(--color-border)',
            color: 'var(--color-text)',
            backgroundColor: 'var(--color-surface)'
          }}
        >
          <ChevronDown size={12} />
          Scroll to bottom
        </button>
      </div>
    );
  }, [handleScroll, scrollToBottom, renderLogEntry]);

  // Render different layouts based on display mode
  if (displayMode === 'local-only') {
    if (connectionState.isConnectedToLocalServer) {
      // Case 1: Connected to local spawned server - show merged logs
      return (
        <div className="p-4 h-full flex flex-col">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-2">
              <h3 className="text-lg font-semibold" style={{ color: 'var(--color-text)', fontFamily: 'inherit' }}>
                Server Logs
              </h3>
              <div className="flex items-center gap-1 px-2 py-1 text-xs border rounded" 
                style={{ borderColor: 'var(--color-border)', color: 'var(--color-muted)' }}>
                {sourceType === LogSourceType.Hybrid ? (
                  <>
                    <Link2 size={12} />
                    Hybrid (File + Network)
                  </>
                ) : sourceType === LogSourceType.Local ? (
                  <>
                    <File size={12} />
                    File Only
                  </>
                ) : (
                  <>
                    <Network size={12} />
                    Network Only
                  </>
                )}
              </div>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={handleClearLogs}
                className="flex items-center gap-1 px-2 py-1 text-xs border rounded hover:opacity-80 transition-opacity"
                style={{ 
                  borderColor: 'var(--color-border)',
                  color: 'var(--color-muted)',
                  backgroundColor: 'transparent'
                }}
              >
                <Trash2 size={12} />
                Clear
              </button>
            </div>
          </div>
          
          {renderLogContainer(hybridLogs, sourceType === LogSourceType.Hybrid)}
        </div>
      );
    } else {
      // Just local server running, not connected
      return (
        <div className="p-4 h-full flex flex-col">
          <div className="flex items-center justify-between mb-4">
            <div className="flex items-center gap-2">
              <h3 className="text-lg font-semibold" style={{ color: 'var(--color-text)', fontFamily: 'inherit' }}>
                Server Logs
              </h3>
              <div className="flex items-center gap-1 px-2 py-1 text-xs border rounded" 
                style={{ borderColor: 'var(--color-border)', color: 'var(--color-muted)' }}>
                <Server size={12} />
                Local Only
              </div>
            </div>
          </div>
          
          {renderLogContainer(serverState.logs, false)}
        </div>
      );
    }
  }

  if (displayMode === 'remote-only') {
    // Case 2: Connected to remote server, no local server
    return (
      <div className="p-4 h-full flex flex-col">
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <h3 className="text-lg font-semibold" style={{ color: 'var(--color-text)', fontFamily: 'inherit' }}>
              Server Logs
            </h3>
            <div className="flex items-center gap-1 px-2 py-1 text-xs border rounded" 
              style={{ borderColor: 'var(--color-border)', color: 'var(--color-muted)' }}>
              <Globe size={12} />
              Remote Server
            </div>
          </div>
          <button
            onClick={handleClearLogs}
            className="flex items-center gap-1 px-2 py-1 text-xs border rounded hover:opacity-80 transition-opacity"
            style={{ 
              borderColor: 'var(--color-border)',
              color: 'var(--color-muted)',
              backgroundColor: 'transparent'
            }}
          >
            <Trash2 size={12} />
            Clear
          </button>
        </div>
        
        {renderLogContainer(remoteLogs, false)}
      </div>
    );
  }

  // Case 3: Connected to remote server AND running local server - show both
  return (
    <div className="p-4 h-full flex flex-col">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold" style={{ color: 'var(--color-text)', fontFamily: 'inherit' }}>
          Server Logs
        </h3>
        <div className="flex items-center gap-2 text-xs" style={{ color: 'var(--color-muted)' }}>
          <Server size={12} />
          Local + 
          <Globe size={12} />
          Remote
        </div>
      </div>
      
      {/* Remote Server Logs (Top Half) */}
      <div className="flex-1 flex flex-col min-h-0">
        <div className="flex items-center justify-between mb-2">
          <h4 className="text-sm font-medium flex items-center gap-2" style={{ color: 'var(--color-text)' }}>
            <Globe size={14} />
            Remote Server ({connectionState.connectedIp}:{connectionState.connectedPort})
          </h4>
          <button
            onClick={handleClearLogs}
            className="flex items-center gap-1 px-2 py-1 text-xs border rounded hover:opacity-80 transition-opacity"
            style={{ 
              borderColor: 'var(--color-border)',
              color: 'var(--color-muted)',
              backgroundColor: 'transparent'
            }}
          >
            <Trash2 size={12} />
            Clear
          </button>
        </div>
        
        {renderLogContainer(remoteLogs, false)}
      </div>

      {/* Local Server Logs (Bottom Half) */}
      <div className="flex-1 flex flex-col min-h-0 mt-4">
        <h4 className="text-sm font-medium mb-2 flex items-center gap-2" style={{ color: 'var(--color-text)' }}>
          <Server size={14} />
          Local Server (127.0.0.1:{serverConfig.port})
        </h4>
        
        {renderLogContainer(serverState.logs, false)}
      </div>
    </div>
  );
};