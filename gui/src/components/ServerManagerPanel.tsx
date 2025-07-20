import React, { useEffect } from 'react';
import { useStore } from '@nanostores/react';
import { 
  serverManagerStore, 
  serverManagerUIStore, 
  serverManagerActions,
  getServerStatusText,
  getServerStatusColor
} from '../stores/serverManagerStore';
import { X, Settings } from 'lucide-react';
import { ServerControls } from './ServerControls';
import { ServerConfigForm } from './ServerConfigForm';
import { ServerLogsPanel } from './ServerLogsPanel';

export const ServerManagerPanel: React.FC = () => {
  const serverState = useStore(serverManagerStore);
  const uiState = useStore(serverManagerUIStore);

  // Initialize on mount
  useEffect(() => {
    serverManagerActions.initialize();
    return () => serverManagerActions.stopStatusPolling();
  }, []);

  if (!uiState.isVisible) return null;

  return (
    <div 
      className="fixed inset-0 flex items-center justify-center z-50"
      style={{
        background: 'rgba(0, 0, 0, 0.4)',
        backdropFilter: 'blur(2px)'
      }}
      onClick={() => serverManagerActions.hide()}
    >
      <div 
        className="shadow-2xl w-full max-w-7xl max-h-[90vh] flex flex-col"
        style={{ 
          backgroundColor: 'var(--color-surface)',
          color: 'var(--color-text)',
          border: '1px solid var(--color-border)'
        }}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b" style={{ borderColor: 'var(--color-border)' }}>
          <h2 className="text-lg font-semibold">Server Management</h2>
          <div className="flex items-center gap-2">
            <div className="flex items-center gap-2">
              <div 
                className="w-3 h-3"
                style={{ backgroundColor: getServerStatusColor(serverState.status) }}
              />
              <span className="text-sm">{getServerStatusText(serverState.status)}</span>
            </div>
            <button
              onClick={() => serverManagerActions.hide()}
              className="p-1 hover:bg-gray-100"
              style={{ backgroundColor: 'transparent' }}
            >
              <X size={16} />
            </button>
          </div>
        </div>

        {/* Action buttons */}
        <div className="p-4 border-b" style={{ borderColor: 'var(--color-border)' }}>
          <ServerControls layout="horizontal" size="medium" />
        </div>

        {/* Side by side content */}
        <div className="flex-1 flex overflow-hidden">
          {/* Configuration pane */}
          <div className="flex-1 overflow-y-auto border-r" style={{ borderColor: 'var(--color-border)' }}>
            <div className="p-4">
              <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
                <Settings size={18} />
                Configuration
              </h3>
              <ServerConfigForm compact={false} />
            </div>
          </div>
          
          {/* Logs pane */}
          <div className="flex-1 flex flex-col" style={{ minHeight: 0 }}>
            <div className="flex-1" style={{ minHeight: 0 }}>
              <ServerLogsPanel />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

