import React, { useEffect, useRef } from 'react';
import { useStore } from '@nanostores/react';
import { serverManagerStore, serverManagerActions } from '../stores/serverManagerStore';

export const ServerLogsPanel: React.FC = () => {
  const serverState = useStore(serverManagerStore);
  const logsEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    // Auto-scroll to bottom when new logs arrive
    if (logsEndRef.current) {
      logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
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

  return (
    <div className="p-4 h-full flex flex-col">
      <h3 className="text-lg font-semibold mb-4" style={{ color: 'var(--color-text)', fontFamily: 'inherit' }}>
        Server Logs
      </h3>
      
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
            No logs available. Start the server to see logs.
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
};