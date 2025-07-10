import React, { useState } from 'react';
import { X, Palette, Settings as SettingsIcon, Info, ArrowLeft, ArrowRight, ArrowDown } from 'lucide-react';
import { useStore } from '@nanostores/react';
import { MaterialColorPalette } from './MaterialColorPalette';
import { editorSettingsStore, setFontSize, setTabSize, toggleVimMode } from '../stores/editorSettingsStore';

interface OptionsPanelProps {
  onClose: () => void;
  position?: 'left' | 'right' | 'bottom';
  onPositionChange?: (position: 'left' | 'right' | 'bottom') => void;
}

type TabType = 'colors' | 'settings' | 'about';

export const OptionsPanel: React.FC<OptionsPanelProps> = ({ onClose, position = 'right', onPositionChange }) => {
  const [activeTab, setActiveTab] = useState<TabType>('colors');
  const editorSettings = useStore(editorSettingsStore);

  const tabs = [
    { id: 'colors' as const, label: 'Colors', icon: Palette },
    { id: 'settings' as const, label: 'Settings', icon: SettingsIcon },
    { id: 'about' as const, label: 'About', icon: Info },
  ];

  const renderTabContent = () => {
    switch (activeTab) {
      case 'colors':
        return <MaterialColorPalette />;
      case 'settings':
        return (
          <div className="p-4">
            <h3 className="text-lg font-semibold mb-4" style={{ color: 'var(--color-text)' }}>
              Editor Settings
            </h3>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-2" style={{ color: 'var(--color-muted)' }}>
                  Font Size
                </label>
                <select 
                  className="w-full p-2 border"
                  style={{ 
                    borderColor: 'var(--color-border)', 
                    backgroundColor: 'var(--color-surface)', 
                    color: 'var(--color-text)' 
                  }}
                  value={editorSettings.fontSize}
                  onChange={(e) => setFontSize(Number(e.target.value))}
                >
                  <option value="12">12px</option>
                  <option value="14">14px</option>
                  <option value="16">16px</option>
                  <option value="18">18px</option>
                  <option value="20">20px</option>
                </select>
              </div>
              <div>
                <label className="block text-sm font-medium mb-2" style={{ color: 'var(--color-muted)' }}>
                  Tab Size
                </label>
                <select 
                  className="w-full p-2 border"
                  style={{ 
                    borderColor: 'var(--color-border)', 
                    backgroundColor: 'var(--color-surface)', 
                    color: 'var(--color-text)' 
                  }}
                  value={editorSettings.tabSize}
                  onChange={(e) => setTabSize(Number(e.target.value))}
                >
                  <option value="2">2 spaces</option>
                  <option value="4">4 spaces</option>
                  <option value="8">8 spaces</option>
                </select>
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
      case 'about':
        return (
          <div className="p-4">
            <h3 className="text-lg font-semibold mb-4" style={{ color: 'var(--color-text)' }}>
              About BuboCore
            </h3>
            <div className="space-y-3 text-sm" style={{ color: 'var(--color-muted)' }}>
              <p>BuboCore GUI is a modern code editor interface built with React and Tauri.</p>
              <p>Features:</p>
              <ul className="list-disc pl-5 space-y-1">
                <li>CodeMirror 6 editor with syntax highlighting</li>
                <li>Material Design color palette system</li>
                <li>Real-time collaboration features</li>
                <li>Cross-platform desktop application</li>
              </ul>
            </div>
          </div>
        );
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
        className="flex border-b"
        style={{ borderColor: 'var(--color-border)' }}
      >
        {tabs.map((tab) => {
          const Icon = tab.icon;
          return (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`flex-1 flex items-center justify-center space-x-2 py-3 px-2 text-sm font-medium transition-colors border-b-2 ${
                activeTab === tab.id ? 'border-current' : 'border-transparent'
              }`}
              style={{
                color: activeTab === tab.id ? 'var(--color-primary)' : 'var(--color-muted)',
                borderBottomColor: activeTab === tab.id ? 'var(--color-primary)' : 'transparent',
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