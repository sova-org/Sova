import React from 'react';
import { X, Palette, Settings as SettingsIcon, Monitor, FileText, ArrowLeft, ArrowRight, ArrowDown, Type, AlignLeft, FileType, Server, ScrollText } from 'lucide-react';
import { useStore } from '@nanostores/react';
import { MaterialColorPalette } from './MaterialColorPalette';
import { DevicesPanel } from './DevicesPanel';
import { FilesPanel } from './FilesPanel';
import { ServerConfigPanel } from './ServerConfigPanel';
import { ServerLogsPanel } from './ServerLogsPanel';
import { editorSettingsStore, setFontSize, setTabSize, toggleVimMode, setFontFamily } from '../stores/editorSettingsStore';
import { optionsPanelStore, setOptionsPanelActiveTab } from '../stores/optionsPanelStore';
import { Dropdown } from './Dropdown';

interface OptionsPanelProps {
  onClose: () => void;
  position?: 'left' | 'right' | 'bottom';
  onPositionChange?: (position: 'left' | 'right' | 'bottom') => void;
}

// type TabType = 'colors' | 'settings' | 'devices' | 'files';

export const OptionsPanel: React.FC<OptionsPanelProps> = ({ onClose, position = 'right', onPositionChange }) => {
  const editorSettings = useStore(editorSettingsStore);
  const optionsState = useStore(optionsPanelStore);
  const activeTab = optionsState.activeTab;

  const tabs = [
    { id: 'colors' as const, label: 'Colors', icon: Palette },
    { id: 'settings' as const, label: 'Settings', icon: SettingsIcon },
    { id: 'devices' as const, label: 'Devices', icon: Monitor },
    { id: 'files' as const, label: 'Files', icon: FileText },
    { id: 'server' as const, label: 'Server', icon: Server },
    { id: 'logs' as const, label: 'Logs', icon: ScrollText },
  ];

  const renderTabContent = () => {
    switch (activeTab) {
      case 'colors':
        return <MaterialColorPalette />;
      case 'settings':
        return (
          <div className="p-4">
            <h3 className="text-lg font-semibold mb-4" style={{ color: 'var(--color-text)', fontFamily: 'inherit' }}>
              Editor Settings
            </h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-2" style={{ color: 'var(--color-muted)' }}>
                  Font Size
                </label>
                <Dropdown
                  value={editorSettings.fontSize.toString()}
                  options={[
                    { value: '12', label: '12px' },
                    { value: '14', label: '14px' },
                    { value: '16', label: '16px' },
                    { value: '18', label: '18px' },
                    { value: '20', label: '20px' },
                  ]}
                  onChange={(value) => setFontSize(Number(value))}
                  icon={<Type size={16} />}
                  title="Select font size"
                  width="full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-2" style={{ color: 'var(--color-muted)' }}>
                  Font Family
                </label>
                <Dropdown
                  value={editorSettings.fontFamily}
                  options={[
                    { value: '"Cascadia Mono", monospace', label: 'Cascadia Mono' },
                    { value: '"Cascadia Mono NF", monospace', label: 'Cascadia Mono NF' },
                    { value: '"Comic Mono", monospace', label: 'Comic Mono' },
                    { value: '"Departure Mono", monospace', label: 'Departure Mono' },
                    { value: '"Fira Code", monospace', label: 'Fira Code' },
                    { value: '"IBM Plex Mono", monospace', label: 'IBM Plex Mono' },
                    { value: '"Iosevka Curly Slab", monospace', label: 'Iosevka Curly Slab' },
                    { value: '"JetBrains Mono", monospace', label: 'JetBrains Mono' },
                    { value: '"JGS7", monospace', label: 'JGS7' },
                    { value: '"Pixel Code", monospace', label: 'Pixel Code' },
                    { value: '"Victor Mono", monospace', label: 'Victor Mono' },
                  ]}
                  onChange={(value) => setFontFamily(value)}
                  icon={<FileType size={16} />}
                  title="Select font family"
                  width="full"
                />
              </div>
              <div>
                <label className="block text-sm font-medium mb-2" style={{ color: 'var(--color-muted)' }}>
                  Tab Size
                </label>
                <Dropdown
                  value={editorSettings.tabSize.toString()}
                  options={[
                    { value: '2', label: '2 spaces' },
                    { value: '4', label: '4 spaces' },
                    { value: '8', label: '8 spaces' },
                  ]}
                  onChange={(value) => setTabSize(Number(value))}
                  icon={<AlignLeft size={16} />}
                  title="Select tab size"
                  width="full"
                />
              </div>
              <div>
                <label className="flex items-center space-x-2">
                  <input 
                    type="checkbox" 
                    checked={editorSettings.vimMode}
                    onChange={toggleVimMode}
                    style={{ 
                      accentColor: 'var(--color-primary)'
                    }}
                  />
                  <span className="text-sm font-medium" style={{ color: 'var(--color-muted)' }}>
                    Vim Mode
                  </span>
                </label>
              </div>
            </div>
          </div>
        );
      case 'devices':
        return <DevicesPanel />;
      case 'files':
        return <FilesPanel />;
      case 'server':
        return <ServerConfigPanel />;
      case 'logs':
        return <ServerLogsPanel />;
      default:
        return null;
    }
  };

  const getBorderClass = () => {
    switch (position) {
      case 'left': return 'border-r';
      case 'right': return 'border-l';
      case 'bottom': return 'border-t';
      default: return 'border-l';
    }
  };

  return (
    <div 
      className={`w-full h-full ${getBorderClass()} flex flex-col overflow-hidden select-none`}
      style={{ 
        backgroundColor: 'var(--color-surface)', 
        borderColor: 'var(--color-border)',
        boxShadow: '0 4px 24px rgba(0, 0, 0, 0.15), 0 2px 8px rgba(0, 0, 0, 0.1)'
      }}
    >
      {/* Header */}
      <div 
        className="h-12 flex items-center justify-between px-4 border-b"
        style={{ borderColor: 'var(--color-border)' }}
      >
        <h2 className="font-semibold" style={{ color: 'var(--color-text)' }}>
          Options
        </h2>
        <div className="flex items-center space-x-2">
          {/* Position toggle buttons */}
          {onPositionChange && (
            <div className="flex space-x-1">
              <button
                onClick={() => onPositionChange('left')}
                className={`p-1.5 rounded transition-all ${
                  position === 'left' ? 'opacity-100' : 'opacity-50 hover:opacity-75'
                }`}
                style={{ 
                  backgroundColor: position === 'left' ? 'var(--color-primary)' : 'transparent',
                  color: position === 'left' ? 'var(--color-background)' : 'var(--color-muted)'
                }}
                title="Position on left"
              >
                <ArrowLeft size={14} />
              </button>
              <button
                onClick={() => onPositionChange('bottom')}
                className={`p-1.5 rounded transition-all ${
                  position === 'bottom' ? 'opacity-100' : 'opacity-50 hover:opacity-75'
                }`}
                style={{ 
                  backgroundColor: position === 'bottom' ? 'var(--color-primary)' : 'transparent',
                  color: position === 'bottom' ? 'var(--color-background)' : 'var(--color-muted)'
                }}
                title="Position at bottom"
              >
                <ArrowDown size={14} />
              </button>
              <button
                onClick={() => onPositionChange('right')}
                className={`p-1.5 rounded transition-all ${
                  position === 'right' ? 'opacity-100' : 'opacity-50 hover:opacity-75'
                }`}
                style={{ 
                  backgroundColor: position === 'right' ? 'var(--color-primary)' : 'transparent',
                  color: position === 'right' ? 'var(--color-background)' : 'var(--color-muted)'
                }}
                title="Position on right"
              >
                <ArrowRight size={14} />
              </button>
            </div>
          )}
          <button
            onClick={onClose}
            className="p-1 hover:opacity-80 transition-opacity ml-2"
            style={{ color: 'var(--color-muted)' }}
          >
            <X size={16} />
          </button>
        </div>
      </div>

      {/* Tabs */}
      <div 
        className="flex border-b overflow-x-auto scrollbar-hide"
        style={{ borderColor: 'var(--color-border)' }}
      >
        {tabs.map((tab) => {
          const Icon = tab.icon;
          return (
            <button
              key={tab.id}
              onClick={() => setOptionsPanelActiveTab(tab.id)}
              className={`flex-shrink-0 flex items-center justify-center space-x-2 py-3 px-4 text-sm font-medium transition-colors border-b-2 ${
                activeTab === tab.id ? 'border-current' : 'border-transparent'
              }`}
              style={{
                color: activeTab === tab.id ? 'var(--color-primary)' : 'var(--color-muted)',
                borderBottomColor: activeTab === tab.id ? 'var(--color-primary)' : 'transparent',
                minWidth: '80px'
              }}
            >
              <Icon size={16} />
              <span className="hidden sm:inline">{tab.label}</span>
            </button>
          );
        })}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto select-text">
        {renderTabContent()}
      </div>
    </div>
  );
};