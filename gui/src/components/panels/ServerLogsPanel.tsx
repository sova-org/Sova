import React, { useEffect, useRef, useState } from 'react';
import { useStore } from '@nanostores/react';
import { logStore, clearLogs } from '../../stores/logs';
import { Trash2, ChevronDown } from 'lucide-react';

export const ServerLogsPanel: React.FC = () => {
  const logs = useStore(logStore);
  const containerRef = useRef<HTMLDivElement>(null);
  const [isUserScrolling, setIsUserScrolling] = useState(false);
  const [showScrollToBottom, setShowScrollToBottom] = useState(false);
  const lastScrollTop = useRef(0);
  
  const visibleCount = 25;
  const displayLogs = logs.slice(-visibleCount);

  // Fast DOM rendering with smart updates
  useEffect(() => {
    if (!containerRef.current) return;
    
    const container = containerRef.current;
    const shouldAutoScroll = !isUserScrolling;
    
    // Check if content actually changed using log IDs
    const currentLogIds = displayLogs.map(log => log.id).join(',');
    const lastLogIds = container.dataset['lastLogIds'] || '';
    
    if (currentLogIds === lastLogIds && displayLogs.length > 0) {
      // Just scroll if needed
      if (shouldAutoScroll && container.parentElement) {
        container.parentElement.scrollTop = container.parentElement.scrollHeight;
      }
      return;
    }
    
    container.dataset['lastLogIds'] = currentLogIds;
    
    if (displayLogs.length === 0) {
      container.innerHTML = '<div class="text-center py-8" style="color: var(--color-muted)">No logs available.</div>';
      return;
    }
    
    // Pre-compute styles for better performance
    const levelColors: Record<string, string> = {
      error: 'var(--color-error)',
      warn: 'var(--color-warning)', 
      warning: 'var(--color-warning)',
      info: 'var(--color-primary)',
      debug: 'var(--color-muted)'
    };
    
    // Build HTML in one pass
    let html = '';
    for (const log of displayLogs) {
      const date = new Date(log.timestamp);
      const time = date.toLocaleTimeString() + '.' + date.getMilliseconds().toString().padStart(3, '0');
      const levelColor = levelColors[log.level.toLowerCase()] || 'var(--color-text)';
      const bgClass = log.level.toLowerCase() === 'error' ? ' bg-red-500 bg-opacity-10' : 
                     log.level.toLowerCase().includes('warn') ? ' bg-yellow-500 bg-opacity-10' : '';
      
      html += `<div class="flex gap-2 py-1 px-2 rounded-sm${bgClass}"><span class="text-xs opacity-60 flex-shrink-0 font-mono" style="color:var(--color-muted)">${time}</span><span class="text-xs font-medium flex-shrink-0 min-w-[60px]" style="color:${levelColor}">[${log.level.toUpperCase()}]</span><span class="text-xs flex-shrink-0 opacity-70" style="color:var(--color-muted)">[${log.source.toUpperCase()}]</span><span class="flex-1 break-words font-mono text-sm" style="color:var(--color-text)">${log.message}</span></div>`;
    }
    
    container.innerHTML = html;
    
    // Instant scroll
    if (shouldAutoScroll && container.parentElement) {
      container.parentElement.scrollTop = container.parentElement.scrollHeight;
    }
  }, [displayLogs, isUserScrolling]);

  // Handle scroll detection
  const handleScroll = (e: React.UIEvent<HTMLDivElement>) => {
    const container = e.currentTarget;
    const { scrollTop, scrollHeight, clientHeight } = container;
    
    const isScrollingUp = scrollTop < lastScrollTop.current;
    lastScrollTop.current = scrollTop;
    const isAtBottom = scrollHeight - scrollTop - clientHeight < 10;
    
    if (isScrollingUp && !isAtBottom) {
      setIsUserScrolling(true);
      setShowScrollToBottom(true);
    } else if (isAtBottom) {
      setIsUserScrolling(false);
      setShowScrollToBottom(false);
    }
  };

  const scrollToBottom = () => {
    if (containerRef.current?.parentElement) {
      setIsUserScrolling(false);
      setShowScrollToBottom(false);
      const scrollContainer = containerRef.current.parentElement;
      scrollContainer.scrollTop = scrollContainer.scrollHeight;
    }
  };

  return (
    <div className="p-4 h-full flex flex-col" style={{ minHeight: 0 }}>
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold" style={{ color: 'var(--color-text)' }}>Server Logs</h3>
        <button
          onClick={clearLogs}
          className="flex items-center gap-1 px-2 py-1 text-xs border rounded hover:opacity-80"
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

      <div className="flex-1 relative" style={{ minHeight: 0 }}>
        <div 
          className="absolute inset-0 overflow-y-auto p-3 font-mono text-sm border space-y-1"
          style={{ 
            backgroundColor: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            color: 'var(--color-text)'
          }}
          onScroll={handleScroll}
        >
          <div ref={containerRef} />
        </div>
        
        {showScrollToBottom && (
          <button
            onClick={scrollToBottom}
            className="absolute bottom-4 right-4 flex items-center gap-1 px-3 py-2 text-xs border rounded-lg shadow-lg"
            style={{ 
              borderColor: 'var(--color-border)',
              color: 'var(--color-text)',
              backgroundColor: 'var(--color-surface)'
            }}
          >
            <ChevronDown size={12} />
            ↓
          </button>
        )}
      </div>
    </div>
  );
};