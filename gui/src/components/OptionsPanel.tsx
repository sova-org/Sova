import React, { useState } from 'react';
import { X, Palette, Settings as SettingsIcon, Info } from 'lucide-react';
import { useStore } from '@nanostores/react';
import { MaterialColorPalette } from './MaterialColorPalette';
import { editorSettingsStore, setFontSize, setTabSize, toggleVimMode } from '../stores/editorSettingsStore';

interface OptionsPanelProps {
  onClose: () => void;
}

type TabType = 'colors' | 'settings' | 'about';

export const OptionsPanel: React.FC<OptionsPanelProps> = ({ onClose }) => {
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
                  className="w-full p-2 border rounded-md"
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
                  className="w-full p-2 border rounded-md"
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
                    className="rounded"
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

  return (
    <div 
      className="w-80 h-full border-l flex flex-col"
      style={{ backgroundColor: 'var(--color-surface)', borderColor: 'var(--color-border)' }}
    >
      {/* Header */}
      <div 
        className="h-12 flex items-center justify-between px-4 border-b"
        style={{ borderColor: 'var(--color-border)' }}
      >
        <h2 className="font-semibold" style={{ color: 'var(--color-text)' }}>
          Options
        </h2>
        <button
          onClick={onClose}
          className="p-1 rounded-md hover:opacity-80 transition-opacity"
          style={{ color: 'var(--color-muted)' }}
        >
          <X size={16} />
        </button>
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
      <div className="flex-1 overflow-y-auto">
        {renderTabContent()}
      </div>
    </div>
  );
};