import React, { useEffect, useState } from 'react';
import { useStore } from '@nanostores/react';
import { serverManagerStore, serverManagerActions, type ServerConfig } from '../stores/serverManagerStore';
import { serverConfigStore } from '../stores/serverConfigStore';
import { Monitor } from 'lucide-react';
import { Dropdown } from './Dropdown';

interface ServerConfigFormProps {
  onConfigChange?: (config: ServerConfig) => void;
  showSaveButton?: boolean;
  compact?: boolean;
}

export const ServerConfigForm: React.FC<ServerConfigFormProps> = ({
  onConfigChange,
  showSaveButton = true,
  compact = false
}) => {
  const serverState = useStore(serverManagerStore);
  const persistedConfig = useStore(serverConfigStore);
  const [localConfig, setLocalConfig] = useState<ServerConfig>(persistedConfig);
  const [audioDevices, setAudioDevices] = useState<string[]>([]);
  const [error, setError] = useState<string | null>(null);

  // Update local config when persisted config changes
  useEffect(() => {
    setLocalConfig(persistedConfig);
  }, [persistedConfig]);

  // Load audio devices
  useEffect(() => {
    const loadAudioDevices = async () => {
      const devices = await serverManagerActions.listAudioDevices();
      setAudioDevices(devices);
    };
    loadAudioDevices();
  }, []);

  // Notify parent of config changes
  useEffect(() => {
    if (onConfigChange) {
      onConfigChange(localConfig);
    }
  }, [localConfig, onConfigChange]);

  const handleConfigChange = (key: keyof ServerConfig, value: any) => {
    setLocalConfig(prev => ({ ...prev, [key]: value }));
  };

  const handleSaveConfig = async () => {
    try {
      await serverManagerActions.updateConfig(localConfig);
      setError(null);
    } catch (error) {
      setError(error instanceof Error ? error.message : 'Failed to save config');
    }
  };

  const isRunning = serverState.status === 'Running';
  const inputClass = compact ? 'w-full px-2 py-1 border text-sm' : 'w-full px-3 py-2 border';
  const labelClass = compact ? 'text-xs' : 'text-sm';
  const gridClass = compact ? 'grid-cols-2 gap-3' : 'grid-cols-1 gap-4';

  return (
    <div className={compact ? 'space-y-4' : 'space-y-6'}>
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
              value={localConfig.ip}
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
              value={localConfig.port}
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
                checked={localConfig.audio_engine}
                onChange={(e) => handleConfigChange('audio_engine', e.target.checked)}
              />
              <span style={{ color: 'var(--color-text)' }}>Enable Audio Engine</span>
            </label>
          </div>
          
          {localConfig.audio_engine && (
            <div className={`grid ${gridClass}`}>
              <div>
                <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
                  Sample Rate
                </label>
                <input
                  type="number"
                  value={localConfig.sample_rate}
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
                  value={localConfig.block_size}
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
                  value={localConfig.buffer_size}
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
                  value={localConfig.max_audio_buffers}
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
                  value={localConfig.max_voices}
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
                  value={localConfig.audio_priority}
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
                  value={localConfig.output_device || ''}
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
              value={localConfig.osc_host}
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
              value={localConfig.osc_port}
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

      {/* Relay Settings */}
      <div>
        <h4 className={`font-medium mb-3 ${compact ? 'text-base' : 'text-lg'}`} style={{ color: 'var(--color-text)' }}>
          Relay
        </h4>
        <div className="space-y-3">
          <div>
            <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
              Relay Address
            </label>
            <input
              type="text"
              value={localConfig.relay || ''}
              onChange={(e) => handleConfigChange('relay', e.target.value || undefined)}
              className={inputClass}
              style={{
                backgroundColor: 'var(--color-surface)',
                borderColor: 'var(--color-border)',
                color: 'var(--color-text)'
              }}
              placeholder="Optional relay server address"
            />
          </div>
          <div>
            <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
              Relay Token
            </label>
            <input
              type="text"
              value={localConfig.relay_token || ''}
              onChange={(e) => handleConfigChange('relay_token', e.target.value || undefined)}
              className={inputClass}
              style={{
                backgroundColor: 'var(--color-surface)',
                borderColor: 'var(--color-border)',
                color: 'var(--color-text)'
              }}
              placeholder="Optional relay token"
            />
          </div>
        </div>
      </div>

      {/* Advanced Settings */}
      <div>
        <h4 className={`font-medium mb-3 ${compact ? 'text-base' : 'text-lg'}`} style={{ color: 'var(--color-text)' }}>
          Advanced
        </h4>
        <div className="space-y-3">
          <div>
            <label className={`block ${labelClass} font-medium mb-1`} style={{ color: 'var(--color-muted)' }}>
              Instance Name
            </label>
            <input
              type="text"
              value={localConfig.instance_name}
              onChange={(e) => handleConfigChange('instance_name', e.target.value)}
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
              Audio Files Location
            </label>
            <input
              type="text"
              value={localConfig.audio_files_location}
              onChange={(e) => handleConfigChange('audio_files_location', e.target.value)}
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
              Timestamp Tolerance (ms)
            </label>
            <input
              type="number"
              value={localConfig.timestamp_tolerance_ms}
              onChange={(e) => handleConfigChange('timestamp_tolerance_ms', parseInt(e.target.value))}
              className={inputClass}
              style={{
                backgroundColor: 'var(--color-surface)',
                borderColor: 'var(--color-border)',
                color: 'var(--color-text)'
              }}
            />
          </div>
          <div>
            <label className={`flex items-center gap-2 ${labelClass}`}>
              <input
                type="checkbox"
                checked={localConfig.list_devices}
                onChange={(e) => handleConfigChange('list_devices', e.target.checked)}
              />
              <span style={{ color: 'var(--color-text)' }}>List Available Devices</span>
            </label>
          </div>
        </div>
      </div>

      {/* Save button */}
      {showSaveButton && (
        <div className="mt-6">
          <button
            onClick={handleSaveConfig}
            disabled={isRunning}
            className={`${compact ? 'px-4 py-2 text-sm' : 'px-6 py-3'} text-white disabled:opacity-50`}
            style={{ backgroundColor: 'var(--color-primary)' }}
          >
            Save Configuration
          </button>
          {isRunning && (
            <p className="text-xs mt-2" style={{ color: 'var(--color-muted)' }}>
              Stop the server to modify configuration
            </p>
          )}
        </div>
      )}
    </div>
  );
};