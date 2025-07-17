import React, { useEffect, useRef } from 'react';
import { useStore } from '@nanostores/react';
import { serverManagerStore, serverManagerActions } from '../stores/serverManagerStore';
import { remoteLogsStore, clearRemoteLogs } from '../stores/remoteLogsStore';
import { Trash2 } from 'lucide-react';

export const ServerLogsPanel: React.FC = () => {
  const serverState = useStore(serverManagerStore);
  const remoteLogs = useStore(remoteLogsStore);
  const remoteLogsEndRef = useRef<HTMLDivElement>(null);
  const localLogsEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Auto-scroll to bottom when new remote logs arrive
    if (remoteLogsEndRef.current) {
      remoteLogsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [remoteLogs]);

  useEffect(() => {
    // Auto-scroll to bottom when new local logs arrive
    if (localLogsEndRef.current) {
      localLogsEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [serverState.logs]);

  useEffect(() => {
    // Refresh logs when component mounts
    serverManagerActions.refreshLogs();
  }, []);

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

  const handleClearRemoteLogs = () => {
    clearRemoteLogs();
  };

  return (
    <div className="p-4 h-full flex flex-col">
      <h3 className="text-lg font-semibold mb-4" style={{ color: 'var(--color-text)', fontFamily: 'inherit' }}>
        Server Logs
      </h3>
      
      {/* Remote Server Logs (Top Half) */}
      <div className="flex-1 flex flex-col min-h-0">
        <div className="flex items-center justify-between mb-2">
          <h4 className="text-sm font-medium" style={{ color: 'var(--color-text)' }}>
            Remote Server Logs
          </h4>
          <button
            onClick={handleClearRemoteLogs}
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
              No remote server logs. Connect to a server to see logs.
            </div>
          ) : (
            <div className="space-y-1">
              {remoteLogs.map((log, index) => (
                <div key={index} className="flex gap-2">
                  <span className="text-xs opacity-60 flex-shrink-0" style={{ color: 'var(--color-muted)' }}>
                    {log.timestamp.toLocaleTimeString()}
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
              <div ref={remoteLogsEndRef} />
            </div>
          )}
        </div>
      </div>

      {/* Local Server Logs (Bottom Half) */}
      <div className="flex-1 flex flex-col min-h-0 mt-4">
        <h4 className="text-sm font-medium mb-2" style={{ color: 'var(--color-text)' }}>
          Local Server Logs
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
              <div ref={localLogsEndRef} />
            </div>
          )}
        </div>
      </div>
    </div>
  );
};