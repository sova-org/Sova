import React, { useEffect, useRef, useState, useCallback } from 'react';
import { useStore } from '@nanostores/react';
import { serverManagerStore, serverManagerActions } from '../stores/serverManagerStore';
import { remoteLogsStore, clearRemoteLogs } from '../stores/remoteLogsStore';
import { $logs, clearLogs } from '../stores/logManagerStore';
import { 
  Trash2, 
  Link2, 
  File, 
  Network, 
  ChevronDown,
  AlertTriangle,
  Info,
  AlertCircle,
  Bug,
  Clock
} from 'lucide-react';


interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
  source?: 'file' | 'network';
}


export const ServerLogsPanel: React.FC = () => {
  const serverState = useStore(serverManagerStore);
  const remoteLogs = useStore(remoteLogsStore);
  const hybridLogs = useStore($logs);
  const logsEndRef = useRef<HTMLDivElement>(null);
  const logsContainerRef = useRef<HTMLDivElement>(null);
  const [isUserScrolling, setIsUserScrolling] = useState(false);
  const [showScrollToBottom, setShowScrollToBottom] = useState(false);
  const lastScrollTop = useRef(0);
  
  const maxLogCount = 1000;

  useEffect(() => {
    // Initialize server manager when component mounts
    serverManagerActions.initialize();
  }, []);

  // Combine all logs for rich hybrid view
  const allLogs = React.useMemo(() => {
    const combined: LogEntry[] = [];
    
    // Add server logs
    serverState.logs.forEach(log => {
      combined.push({
        timestamp: typeof log.timestamp === 'string' ? log.timestamp : new Date(log.timestamp).toISOString(),
        level: log.level,
        message: log.message,
        source: 'file' as const
      });
    });

    // Add remote logs  
    remoteLogs.forEach(log => {
      combined.push({
        timestamp: typeof log.timestamp === 'string' ? log.timestamp : new Date(log.timestamp).toISOString(),
        level: log.level,
        message: log.message,
        source: 'network' as const
      });
    });

    // Add hybrid logs
    hybridLogs.forEach(log => {
      if (!combined.some(existing => 
        existing.timestamp === log.timestamp && 
        existing.message === log.message &&
        existing.level === log.level
      )) {
        combined.push({
          timestamp: log.timestamp,
          level: log.level,
          message: log.message,
          source: log.source || 'network' as const
        });
      }
    });

    // Sort by timestamp
    return combined.sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime());
  }, [serverState.logs, remoteLogs, hybridLogs]);

  // Keep recent logs only
  const displayLogs = React.useMemo(() => {
    return allLogs.slice(-maxLogCount);
  }, [allLogs, maxLogCount]);

  // Auto-scroll to bottom when new logs arrive, but only if user hasn't scrolled up
  useEffect(() => {
    if (!isUserScrolling && logsEndRef.current) {
      setTimeout(() => {
        if (!isUserScrolling && logsEndRef.current) {
          logsEndRef.current.scrollIntoView({ behavior: 'smooth' });
        }
      }, 50);
    }
  }, [displayLogs.length, isUserScrolling, showScrollToBottom]);

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

  const getLevelIcon = (level: string) => {
    switch (level.toLowerCase()) {
      case 'error':
        return <AlertCircle size={12} />;
      case 'warn':
      case 'warning':
        return <AlertTriangle size={12} />;
      case 'info':
        return <Info size={12} />;
      case 'debug':
        return <Bug size={12} />;
      default:
        return null;
    }
  };

  const getSourceIcon = (source: string) => {
    return source === 'file' ? <File size={12} /> : <Network size={12} />;
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleTimeString() + '.' + date.getMilliseconds().toString().padStart(3, '0');
  };

  const renderLogEntry = useCallback((log: LogEntry, index: number) => {
    const isError = log.level.toLowerCase() === 'error';
    const isWarning = log.level.toLowerCase() === 'warn' || log.level.toLowerCase() === 'warning';
    
    return (
      <div 
        key={index} 
        className={`flex gap-2 py-1 px-2 rounded-sm ${isError ? 'bg-red-500 bg-opacity-10' : isWarning ? 'bg-yellow-500 bg-opacity-10' : ''}`}
      >
        <span className="text-xs opacity-60 flex-shrink-0 font-mono" style={{ color: 'var(--color-muted)' }}>
          <Clock size={10} className="inline mr-1" />
          {formatTimestamp(log.timestamp)}
        </span>
        
        <span 
          className="text-xs font-medium flex-shrink-0 min-w-[60px] flex items-center gap-1"
          style={{ color: getLevelColor(log.level) }}
        >
          {getLevelIcon(log.level)}
          [{log.level.toUpperCase()}]
        </span>
        
        <span 
          className="text-xs flex-shrink-0 flex items-center gap-1 opacity-70"
          style={{ color: 'var(--color-muted)' }}
        >
          {getSourceIcon(log.source || 'network')}
          [{log.source === 'file' ? 'FILE' : 'NET'}]
        </span>
        
        <span className="flex-1 break-words font-mono text-sm" style={{ color: 'var(--color-text)' }}>
          {log.message}
        </span>
      </div>
    );
  }, []);

  // Render log container with scroll management
  const renderLogContainer = useCallback((logs: LogEntry[]) => {
    const maxVisibleItems = 100;
    const visibleLogs = logs.length <= maxVisibleItems ? logs : logs.slice(-maxVisibleItems);
    
    return (
      <div className="flex-1 relative" style={{ minHeight: 0 }}>
        <div 
          ref={logsContainerRef}
          className="absolute inset-0 overflow-y-auto p-3 font-mono text-sm border"
          style={{ 
            backgroundColor: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            color: 'var(--color-text)'
          }}
          onScroll={handleScroll}
        >
          {visibleLogs.length === 0 ? (
            <div className="text-center py-8" style={{ color: 'var(--color-muted)' }}>
              No logs available.
            </div>
          ) : (
            <div className="space-y-1">
              {visibleLogs.map((log, index) => renderLogEntry(log, index))}
              <div ref={logsEndRef} />
            </div>
          )}
        </div>
        
        {/* Scroll to bottom button */}
        {showScrollToBottom && (
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
        )}
      </div>
    );
  }, [handleScroll, scrollToBottom, renderLogEntry]);

  // Always render simple hybrid view for server configuration modal
  return (
    <div className="p-4 h-full flex flex-col" style={{ minHeight: 0 }}>
      {/* Header with title and hybrid mode indicator */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <h3 className="text-lg font-semibold" style={{ color: 'var(--color-text)', fontFamily: 'inherit' }}>
            Server Logs
          </h3>
          <div className="flex items-center gap-1 px-2 py-1 text-xs border rounded" 
            style={{ borderColor: 'var(--color-primary)', color: 'var(--color-primary)' }}>
            <Link2 size={12} />
            <span className="font-semibold">HYBRID</span>
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

      {/* Logs container */}
      {renderLogContainer(displayLogs)}
    </div>
  );
};