import React, { useEffect, useRef, useMemo } from 'react';
import { useStore } from '@nanostores/react';
import { serverManagerStore, serverManagerActions } from '../stores/serverManagerStore';
import { remoteLogsStore, clearRemoteLogs, RemoteLogEntry } from '../stores/remoteLogsStore';
import { connectionStateStore, getLogDisplayMode } from '../stores/connectionStateStore';
import { Trash2, Server, Globe, Link2 } from 'lucide-react';

interface CombinedLogEntry {
  timestamp: Date;
  timestampStr: string;
  level: string;
  message: string;
  source: 'local' | 'remote';
}

export const ServerLogsPanel: React.FC = () => {
  const serverState = useStore(serverManagerStore);
  const remoteLogs = useStore(remoteLogsStore);
  const connectionState = useStore(connectionStateStore);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const displayMode = getLogDisplayMode();

  useEffect(() => {
    // Refresh logs when component mounts
    serverManagerActions.refreshLogs();
  }, []);

  // Combine and sort logs when in merged mode
  const mergedLogs = useMemo(() => {
    if (displayMode !== 'local-only' || !connectionState.isConnectedToLocalServer) {
      return [];
    }

    const combined: CombinedLogEntry[] = [];

    // Add local server logs
    serverState.logs.forEach(log => {
      combined.push({
        timestamp: new Date(log.timestamp),
        timestampStr: log.timestamp,
        level: log.level,
        message: log.message,
        source: 'local'
      });
    });

    // Add remote logs
    remoteLogs.forEach(log => {
      combined.push({
        timestamp: log.timestamp,
        timestampStr: log.timestamp.toISOString(),
        level: log.level,
        message: log.message,
        source: 'remote'
      });
    });

    // Sort by timestamp
    return combined.sort((a, b) => a.timestamp.getTime() - b.timestamp.getTime());
  }, [serverState.logs, remoteLogs, displayMode, connectionState.isConnectedToLocalServer]);

  useEffect(() => {
    // Auto-scroll to bottom when new logs arrive
    if (logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [mergedLogs, remoteLogs, serverState.logs]);

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
    // Could also clear local logs if needed
  };

  const renderLogEntry = (log: CombinedLogEntry | RemoteLogEntry, index: number, showSource: boolean = false) => {
    const timestamp = 'timestamp' in log && log.timestamp instanceof Date 
      ? log.timestamp.toLocaleTimeString() 
      : (log as any).timestampStr || '';
    
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
        {showSource && 'source' in log && (
          <span 
            className="text-xs flex-shrink-0"
            style={{ color: 'var(--color-muted)' }}
          >
            [{log.source === 'local' ? 'L' : 'R'}]
          </span>
        )}
        <span className="flex-1 break-words">
          {log.message}
        </span>
      </div>
    );
  };

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
                <Link2 size={12} />
                Local Server
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
          
          <div 
            className="flex-1 overflow-y-auto p-3 font-mono text-sm border"
            style={{ 
              backgroundColor: 'var(--color-surface)',
              borderColor: 'var(--color-border)',
              color: 'var(--color-text)'
            }}
          >
            {mergedLogs.length === 0 ? (
              <div className="text-center py-8" style={{ color: 'var(--color-muted)' }}>
                No logs available.
              </div>
            ) : (
              <div className="space-y-1">
                {mergedLogs.map((log, index) => renderLogEntry(log, index))}
                <div ref={logsEndRef} />
              </div>
            )}
          </div>
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
          
          <div 
            className="flex-1 overflow-y-auto p-3 font-mono text-sm border"
            style={{ 
              backgroundColor: 'var(--color-surface)',
              borderColor: 'var(--color-border)',
              color: 'var(--color-text)'
            }}
          >
            {serverState.logs.length === 0 ? (
              <div className="text-center py-8" style={{ color: 'var(--color-muted)' }}>
                No local server logs. Start a local server to see logs.
              </div>
            ) : (
              <div className="space-y-1">
                {serverState.logs.map((log, index) => (
                  <div key={index} className="flex gap-2">
                    <span className="text-xs opacity-60 flex-shrink-0" style={{ color: 'var(--color-muted)' }}>
                      {log.timestamp}
                    </span>
                    <span 
                      className="text-xs font-medium flex-shrink-0 min-w-[50px]"
                      style={{ color: getLevelColor(log.level) }}
                    >
                      [{log.level.toUpperCase()}]
                    </span>
                    <span className="flex-1 break-words">
                      {log.message}
                    </span>
                  </div>
                ))}
                <div ref={logsEndRef} />
              </div>
            )}
          </div>
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
        
        <div 
          className="flex-1 overflow-y-auto p-3 font-mono text-sm border"
          style={{ 
            backgroundColor: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            color: 'var(--color-text)'
          }}
        >
          {remoteLogs.length === 0 ? (
            <div className="text-center py-8" style={{ color: 'var(--color-muted)' }}>
              {connectionState.isConnected 
                ? 'Waiting for server logs...' 
                : 'No remote server logs. Connect to a server to see logs.'}
            </div>
          ) : (
            <div className="space-y-1">
              {remoteLogs.map((log, index) => renderLogEntry(log, index))}
              <div ref={logsEndRef} />
            </div>
          )}
        </div>
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
        
        <div 
          className="flex-1 overflow-y-auto p-3 font-mono text-sm border min-h-0"
          style={{ 
            backgroundColor: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            color: 'var(--color-text)'
          }}
        >
          {remoteLogs.length === 0 ? (
            <div className="text-center py-4" style={{ color: 'var(--color-muted)' }}>
              Waiting for remote server logs...
            </div>
          ) : (
            <div className="space-y-1">
              {remoteLogs.map((log, index) => renderLogEntry(log, index))}
              <div ref={logsEndRef} />
            </div>
          )}
        </div>
      </div>

      {/* Local Server Logs (Bottom Half) */}
      <div className="flex-1 flex flex-col min-h-0 mt-4">
        <h4 className="text-sm font-medium mb-2 flex items-center gap-2" style={{ color: 'var(--color-text)' }}>
          <Server size={14} />
          Local Server (127.0.0.1:{serverState.config.port})
        </h4>
        
        <div 
          className="flex-1 overflow-y-auto p-3 font-mono text-sm border min-h-0"
          style={{ 
            backgroundColor: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            color: 'var(--color-text)'
          }}
        >
          {serverState.logs.length === 0 ? (
            <div className="text-center py-4" style={{ color: 'var(--color-muted)' }}>
              Waiting for local server logs...
            </div>
          ) : (
            <div className="space-y-1">
              {serverState.logs.map((log, index) => (
                <div key={index} className="flex gap-2">
                  <span className="text-xs opacity-60 flex-shrink-0" style={{ color: 'var(--color-muted)' }}>
                    {log.timestamp}
                  </span>
                  <span 
                    className="text-xs font-medium flex-shrink-0 min-w-[50px]"
                    style={{ color: getLevelColor(log.level) }}
                  >
                    [{log.level.toUpperCase()}]
                  </span>
                  <span className="flex-1 break-words">
                    {log.message}
                  </span>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};