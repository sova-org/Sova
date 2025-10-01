import React, { useEffect, useState } from 'react';
import { useStore } from '@nanostores/react';
import { serverConfigStore, configDirtyStore, updateConfig, saveConfig, loadConfig, type ServerConfig } from '../../stores/server/serverConfig';
import { serverManagerActions } from '../../stores/server/serverManager';
import { Monitor, Save, RotateCcw } from 'lucide-react';
import { Dropdown } from '../ui/Dropdown';

interface ServerConfigFormProps {
  onConfigChange?: (config: ServerConfig) => void;
  compact?: boolean;
}

export const ServerConfigForm: React.FC<ServerConfigFormProps> = ({
  onConfigChange,
  compact = false
}) => {
  const config = useStore(serverConfigStore);
  const isDirty = useStore(configDirtyStore);
  const [audioDevices, setAudioDevices] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    loadConfig();
  }, []);

  useEffect(() => {
    const loadAudioDevices = async () => {
      const devices = await serverManagerActions.listAudioDevices();
      setAudioDevices(devices);
    };
    loadAudioDevices();
  }, []);

  useEffect(() => {
    if (onConfigChange) {
      onConfigChange(config);
    }
  }, [config, onConfigChange]);

  const handleConfigChange = (key: keyof ServerConfig, value: any) => {
    updateConfig({ [key]: value });
  };

  const handleSave = async () => {
    setIsSaving(true);
    setError(null);
    try {
      await saveConfig();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save configuration');
    } finally {
      setIsSaving(false);
    }
  };

  const handleReload = async () => {
    setError(null);
    await loadConfig();
  };

  const inputClass = compact ? 'w-full px-2 py-1 border text-sm' : 'w-full px-3 py-2 border';
  const labelClass = compact ? 'text-xs' : 'text-sm';
  const gridClass = compact ? 'grid-cols-2 gap-3' : 'grid-cols-1 gap-4';

  return (
    <div className={compact ? 'space-y-4' : 'space-y-6'}>
      {/* Save/Reload buttons */}
      <div className="flex gap-2">
        <button
          onClick={handleSave}
          disabled={!isDirty || isSaving}
          className={`flex items-center gap-2 px-4 py-2 font-medium transition-colors ${
            isDirty && !isSaving
              ? 'bg-blue-600 text-white hover:bg-blue-700'
              : 'bg-gray-300 text-gray-500 cursor-not-allowed'
          }`}
          style={{
            backgroundColor: isDirty && !isSaving ? 'var(--color-primary)' : undefined,
          }}
        >
          <Save size={16} />
          {isSaving ? 'Saving...' : 'Save Configuration'}
        </button>
        <button
          onClick={handleReload}
          className="flex items-center gap-2 px-4 py-2 border font-medium hover:bg-gray-100 transition-colors"
          style={{
            borderColor: 'var(--color-border)',
            color: 'var(--color-text)',
          }}
        >
          <RotateCcw size={16} />
          Reload from File
        </button>
      </div>

      {/* Unsaved changes indicator */}
      {isDirty && (
        <div className="p-3 bg-yellow-100 border border-yellow-200 text-yellow-800 text-sm">
          You have unsaved changes. Click "Save Configuration" to persist them.
        </div>
      )}

      {/* Error display */}
      {error && (
        <div className="p-3 bg-red-100 border border-red-200 text-red-700 text-sm">
          {error}
        </div>
      )}

      {/* Network Settings */}
      <div>
        <h4 className={`font-medium mb-3 ${compact ? 'text-base' : 'text-lg'}`} style={{ color: 'var(--color-text)' }}>
          Network
        </h4>
        <div className={`grid ${gridClass}`}>
          <div>
            <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
              IP Address
            </label>
            <input
              type="text"
              value={config.ip}
              onChange={(e) => handleConfigChange('ip', e.target.value)}
              className={inputClass}
              style={{
                backgroundColor: 'var(--color-surface)',
                borderColor: 'var(--color-border)',
                color: 'var(--color-text)'
              }}
            />
          </div>
          <div>
            <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
              Port
            </label>
            <input
              type="number"
              value={config.port}
              onChange={(e) => handleConfigChange('port', parseInt(e.target.value))}
              className={inputClass}
              style={{
                backgroundColor: 'var(--color-surface)',
                borderColor: 'var(--color-border)',
                color: 'var(--color-text)'
              }}
            />
          </div>
        </div>
      </div>

      {/* Audio Engine */}
      <div>
        <h4 className={`font-medium mb-3 ${compact ? 'text-base' : 'text-lg'}`} style={{ color: 'var(--color-text)' }}>
          Audio Engine
        </h4>
        <div className="space-y-3">
          <div>
            <label className={`flex items-center gap-2 ${labelClass}`}>
              <input
                type="checkbox"
                checked={config.audio_engine}
                onChange={(e) => handleConfigChange('audio_engine', e.target.checked)}
              />
              <span style={{ color: 'var(--color-text)' }}>Enable Audio Engine</span>
            </label>
          </div>

          {config.audio_engine && (
            <div className={`grid ${gridClass}`}>
              <div>
                <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
                  Sample Rate
                </label>
                <input
                  type="number"
                  value={config.sample_rate}
                  onChange={(e) => handleConfigChange('sample_rate', parseInt(e.target.value))}
                  className={inputClass}
                  style={{
                    backgroundColor: 'var(--color-surface)',
                    borderColor: 'var(--color-border)',
                    color: 'var(--color-text)'
                  }}
                />
              </div>
              <div>
                <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
                  Block Size
                </label>
                <input
                  type="number"
                  value={config.block_size}
                  onChange={(e) => handleConfigChange('block_size', parseInt(e.target.value))}
                  className={inputClass}
                  style={{
                    backgroundColor: 'var(--color-surface)',
                    borderColor: 'var(--color-border)',
                    color: 'var(--color-text)'
                  }}
                />
              </div>
              <div>
                <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
                  Buffer Size
                </label>
                <input
                  type="number"
                  value={config.buffer_size}
                  onChange={(e) => handleConfigChange('buffer_size', parseInt(e.target.value))}
                  className={inputClass}
                  style={{
                    backgroundColor: 'var(--color-surface)',
                    borderColor: 'var(--color-border)',
                    color: 'var(--color-text)'
                  }}
                />
              </div>
              <div>
                <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
                  Max Audio Buffers
                </label>
                <input
                  type="number"
                  value={config.max_audio_buffers}
                  onChange={(e) => handleConfigChange('max_audio_buffers', parseInt(e.target.value))}
                  className={inputClass}
                  style={{
                    backgroundColor: 'var(--color-surface)',
                    borderColor: 'var(--color-border)',
                    color: 'var(--color-text)'
                  }}
                />
              </div>
              <div>
                <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
                  Max Voices
                </label>
                <input
                  type="number"
                  value={config.max_voices}
                  onChange={(e) => handleConfigChange('max_voices', parseInt(e.target.value))}
                  className={inputClass}
                  style={{
                    backgroundColor: 'var(--color-surface)',
                    borderColor: 'var(--color-border)',
                    color: 'var(--color-text)'
                  }}
                />
              </div>
              <div>
                <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
                  Audio Priority
                </label>
                <input
                  type="number"
                  value={config.audio_priority}
                  onChange={(e) => handleConfigChange('audio_priority', parseInt(e.target.value))}
                  className={inputClass}
                  style={{
                    backgroundColor: 'var(--color-surface)',
                    borderColor: 'var(--color-border)',
                    color: 'var(--color-text)'
                  }}
                />
              </div>
              <div>
                <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
                  Output Device
                </label>
                <Dropdown
                  value={config.output_device || ''}
                  options={[
                    { value: '', label: 'Default' },
                    ...audioDevices.map(device => ({ value: device, label: device }))
                  ]}
                  onChange={(value) => handleConfigChange('output_device', value || undefined)}
                  icon={<Monitor size={16} />}
                  title="Select audio output device"
                  width="full"
                />
                <p className="text-xs mt-1 opacity-60" style={{ color: 'var(--color-muted)' }}>
                  Devices marked with ✓ support 44.1kHz stereo output
                </p>
              </div>
              <div>
                <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
                  Audio Files Location
                </label>
                <input
                  type="text"
                  value={config.audio_files_location}
                  onChange={(e) => handleConfigChange('audio_files_location', e.target.value)}
                  className={inputClass}
                  style={{
                    backgroundColor: 'var(--color-surface)',
                    borderColor: 'var(--color-border)',
                    color: 'var(--color-text)'
                  }}
                />
              </div>
            </div>
          )}
        </div>
      </div>

      {/* OSC Settings */}
      <div>
        <h4 className={`font-medium mb-3 ${compact ? 'text-base' : 'text-lg'}`} style={{ color: 'var(--color-text)' }}>
          OSC
        </h4>
        <div className={`grid ${gridClass}`}>
          <div>
            <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
              OSC Host
            </label>
            <input
              type="text"
              value={config.osc_host}
              onChange={(e) => handleConfigChange('osc_host', e.target.value)}
              className={inputClass}
              style={{
                backgroundColor: 'var(--color-surface)',
                borderColor: 'var(--color-border)',
                color: 'var(--color-text)'
              }}
            />
          </div>
          <div>
            <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
              OSC Port
            </label>
            <input
              type="number"
              value={config.osc_port}
              onChange={(e) => handleConfigChange('osc_port', parseInt(e.target.value))}
              className={inputClass}
              style={{
                backgroundColor: 'var(--color-surface)',
                borderColor: 'var(--color-border)',
                color: 'var(--color-text)'
              }}
            />
          </div>
        </div>
      </div>

    </div>
  );
};
