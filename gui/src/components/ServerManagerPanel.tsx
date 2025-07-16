import React, { useEffect } from 'react';
import { useStore } from '@nanostores/react';
import { 
  serverManagerStore, 
  serverManagerUIStore, 
  serverManagerActions,
  getServerStatusText,
  getServerStatusColor
} from '../stores/serverManagerStore';
import { X, Settings, FileText } from 'lucide-react';
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
    >
      <div 
        className="shadow-2xl w-full max-w-4xl max-h-[90vh] overflow-hidden"
        style={{ 
          backgroundColor: 'var(--color-surface)',
          color: 'var(--color-text)',
          border: '1px solid var(--color-border)'
        }}
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

        {/* Tabs */}
        <div className="flex border-b" style={{ borderColor: 'var(--color-border)' }}>
          <button
            onClick={() => serverManagerActions.setActiveTab('config')}
            className={`flex items-center gap-2 px-4 py-2 border-b-2 ${
              uiState.activeTab === 'config' ? 'border-blue-500' : 'border-transparent'
            }`}
          >
            <Settings size={16} />
            Configuration
          </button>
          <button
            onClick={() => serverManagerActions.setActiveTab('logs')}
            className={`flex items-center gap-2 px-4 py-2 border-b-2 ${
              uiState.activeTab === 'logs' ? 'border-blue-500' : 'border-transparent'
            }`}
          >
            <FileText size={16} />
            Logs
          </button>
        </div>

        {/* Content */}
        <div className="overflow-y-auto max-h-[60vh]">
          {uiState.activeTab === 'config' && (
            <div className="p-4">
              <ServerConfigForm compact={false} />
            </div>
          )}
          
          {uiState.activeTab === 'logs' && (
            <ServerLogsPanel />
          )}
        </div>
      </div>
    </div>
  );
};

