import React, { useState, useEffect } from 'react';
import { Music, Wifi, Plus, Trash2, Hash, Play, Square, Tag } from 'lucide-react';
import { createBuboClient } from '../client';
import type { DeviceInfo, ClientMessage, ServerMessage } from '../types';

const client = createBuboClient();

interface DevicesState {
  devices: DeviceInfo[];
  activeTab: 'midi' | 'osc';
  selectedMidiIndex: number;
  selectedOscIndex: number;
  isConnected: boolean;
  
  // Input modes
  isCreatingOsc: boolean;
  editingDeviceName: string | null;
  isCreatingNewDevice: boolean;
  
  // Input values
  slotEditValue: string;
  newDeviceInput: string;
  oscStep: number;
  oscName: string;
  oscIp: string;
  oscPort: string;
  
  // UI state
  statusMessage: string;
}

export const DevicesPanel: React.FC = () => {
  const [state, setState] = useState<DevicesState>({
    devices: [],
    activeTab: 'midi',
    selectedMidiIndex: 0,
    selectedOscIndex: 0,
    isConnected: false,
    
    isCreatingOsc: false,
    editingDeviceName: null,
    isCreatingNewDevice: false,
    
    slotEditValue: '',
    newDeviceInput: '',
    oscStep: 0,
    oscName: '',
    oscIp: '',
    oscPort: '',
    
    statusMessage: '',
  });

  const filteredDevices = state.devices
    .filter(device => {
      if (state.activeTab === 'midi') {
        return device.kind === 'Midi';
      } else {
        return device.kind === 'Osc';
      }
    })
    .sort((a, b) => a.name.localeCompare(b.name));

  const currentSelectedIndex = state.activeTab === 'midi' ? state.selectedMidiIndex : state.selectedOscIndex;
  const validSelectedIndex = Math.min(currentSelectedIndex, filteredDevices.length - 1);
  const selectedDevice = filteredDevices[validSelectedIndex >= 0 ? validSelectedIndex : 0];
  
  // Debug logging
  console.log('Current state:', {
    activeTab: state.activeTab,
    totalDevices: state.devices.length,
    filteredDevices: filteredDevices.length,
    currentSelectedIndex,
    validSelectedIndex,
    selectedDevice: selectedDevice?.name
  });

  useEffect(() => {
    checkConnection();
    
    const unsubscribe = client.onMessage((message) => {
      const serverMessage = message as ServerMessage;
      if (typeof serverMessage === 'object' && serverMessage !== null && 'Hello' in serverMessage) {
        const helloMessage = serverMessage as { Hello: { devices: DeviceInfo[] } };
        console.log('Hello message devices:', helloMessage.Hello.devices);
        setState(prev => ({ 
          ...prev, 
          devices: helloMessage.Hello.devices,
          selectedMidiIndex: 0,
          selectedOscIndex: 0
        }));
      } else if (typeof serverMessage === 'object' && serverMessage !== null && 'DeviceList' in serverMessage) {
        const deviceListMessage = serverMessage as { DeviceList: DeviceInfo[] };
        console.log('DeviceList message devices:', deviceListMessage.DeviceList);
        setState(prev => ({ 
          ...prev, 
          devices: deviceListMessage.DeviceList,
          selectedMidiIndex: 0,
          selectedOscIndex: 0
        }));
      } else if (serverMessage === 'Success') {
        setState(prev => ({ ...prev, statusMessage: 'Operation successful' }));
        requestDeviceList();
      } else if (typeof serverMessage === 'object' && serverMessage !== null && 'InternalError' in serverMessage) {
        const errorMessage = serverMessage as { InternalError: string };
        setState(prev => ({ ...prev, statusMessage: `Error: ${errorMessage.InternalError}` }));
      }
    });

    return () => unsubscribe();
  }, []);

  const checkConnection = async () => {
    try {
      const connected = await client.isConnected();
      setState(prev => ({ ...prev, isConnected: connected }));
      if (connected) {
        requestDeviceList();
      }
    } catch (error) {
      console.error('Failed to check connection:', error);
    }
  };

  const requestDeviceList = async () => {
    try {
      await client.sendMessage("RequestDeviceList");
    } catch (error) {
      console.error('Failed to request device list:', error);
    }
  };

  const sendClientMessage = async (message: ClientMessage) => {
    try {
      await client.sendMessage(message);
    } catch (error) {
      console.error('Failed to send message:', error);
      setState(prev => ({ ...prev, statusMessage: 'Failed to send message' }));
    }
  };

  const handleDeviceConnect = (device: DeviceInfo) => {
    if (device.is_connected) {
      sendClientMessage({ DisconnectMidiDeviceByName: device.name });
    } else {
      sendClientMessage({ ConnectMidiDeviceByName: device.name });
    }
  };

  const handleSlotClick = (device: DeviceInfo) => {
    setState(prev => ({ 
      ...prev, 
      editingDeviceName: device.name,
      slotEditValue: device.id === 0 ? '' : device.id.toString()
    }));
  };

  const handleSlotEditKeyDown = (e: React.KeyboardEvent, device: DeviceInfo) => {
    if (e.key === 'Enter') {
      confirmSlotEdit(device);
    } else if (e.key === 'Escape') {
      setState(prev => ({ ...prev, editingDeviceName: null }));
    }
  };

  const confirmSlotEdit = (device: DeviceInfo) => {
    const slotNum = parseInt(state.slotEditValue);
    
    if (state.slotEditValue === '' || slotNum === 0) {
      sendClientMessage({ UnassignDeviceFromSlot: device.id });
    } else if (isNaN(slotNum) || slotNum < 1 || slotNum > 16) {
      setState(prev => ({ ...prev, statusMessage: 'Invalid slot number (1-16)' }));
      return;
    } else {
      sendClientMessage({ AssignDeviceToSlot: [slotNum, device.name] });
    }
    
    setState(prev => ({ ...prev, editingDeviceName: null }));
  };

  const handleCreateNewDevice = () => {
    setState(prev => ({ ...prev, isCreatingNewDevice: true, newDeviceInput: '' }));
  };

  const confirmNewDeviceCreation = () => {
    if (!state.newDeviceInput.trim()) {
      setState(prev => ({ ...prev, statusMessage: 'Please enter a device name' }));
      return;
    }
    
    if (state.activeTab === 'midi') {
      sendClientMessage({ CreateVirtualMidiOutput: state.newDeviceInput.trim() });
    }
    
    setState(prev => ({ ...prev, isCreatingNewDevice: false, newDeviceInput: '' }));
  };

  const handleNewDeviceKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      confirmNewDeviceCreation();
    } else if (e.key === 'Escape') {
      setState(prev => ({ ...prev, isCreatingNewDevice: false, newDeviceInput: '' }));
    }
  };

  const handleCreateOsc = () => {
    setState(prev => ({ 
      ...prev, 
      isCreatingOsc: true, 
      oscStep: 0, 
      oscName: '', 
      oscIp: '127.0.0.1', 
      oscPort: '57120' 
    }));
  };

  const handleCancelOscCreation = () => {
    setState(prev => ({ 
      ...prev, 
      isCreatingOsc: false, 
      oscStep: 0, 
      oscName: '', 
      oscIp: '127.0.0.1', 
      oscPort: '57120' 
    }));
  };

  const handleOscStepNext = () => {
    if (state.oscStep === 0 && !state.oscName.trim()) {
      setState(prev => ({ ...prev, statusMessage: 'Please enter a name' }));
      return;
    }
    if (state.oscStep === 1 && !state.oscIp.trim()) {
      setState(prev => ({ ...prev, statusMessage: 'Please enter an IP address' }));
      return;
    }
    if (state.oscStep === 2) {
      const port = parseInt(state.oscPort);
      if (isNaN(port) || port < 1 || port > 65535) {
        setState(prev => ({ ...prev, statusMessage: 'Invalid port number' }));
        return;
      }
      
      sendClientMessage({ CreateOscDevice: [state.oscName.trim(), state.oscIp.trim(), port] });
      handleCancelOscCreation();
      return;
    }
    
    setState(prev => ({ ...prev, oscStep: prev.oscStep + 1 }));
  };

  const handleOscFieldKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleOscStepNext();
    } else if (e.key === 'Escape') {
      handleCancelOscCreation();
    }
  };


  const handleRemoveOsc = (device: DeviceInfo) => {
    if (device.kind !== 'Osc') return;
    sendClientMessage({ RemoveOscDevice: device.name });
  };


  const renderDeviceTable = () => {
    const isMidiTab = state.activeTab === 'midi';
    
    return (
      <div className="overflow-hidden">
        <div className={`grid gap-2 p-2 border-b font-medium text-sm ${
          isMidiTab ? 'grid-cols-3' : 'grid-cols-4'
        }`}
             style={{ borderColor: 'var(--color-border)', color: 'var(--color-text)' }}>
          <div className="flex items-center">
            <Hash size={14} className="mr-2" />
            <span>Slot</span>
          </div>
          <div className="flex items-center">
            <div className="w-2 h-2 rounded-full bg-current mr-2 opacity-60" />
            <span>Status</span>
          </div>
          <div className="flex items-center">
            <Tag size={14} className="mr-2" />
            <span>Name</span>
          </div>
          {!isMidiTab && (
            <div className="flex items-center">
              <Wifi size={14} className="mr-2" />
              <span>Address</span>
            </div>
          )}
        </div>
        
        <div className="flex-1 overflow-y-auto">
          {filteredDevices.map((device, index) => {
            const isSelected = index === validSelectedIndex;
            const isEditingSlot = state.editingDeviceName === device.name;
            
            return (
              <div
                key={device.name}
                className={`grid gap-2 p-2 border-b cursor-pointer transition-colors duration-150 ${
                  isMidiTab ? 'grid-cols-3' : 'grid-cols-4'
                }`}
                style={{ 
                  borderColor: 'var(--color-border)',
                  backgroundColor: isSelected ? 'var(--color-primary)' : undefined,
                  color: isSelected ? 'var(--color-background)' : 'var(--color-text)'
                }}
                onMouseEnter={(e) => {
                  if (!isSelected) {
                    e.currentTarget.style.backgroundColor = 'var(--color-surface)';
                  }
                }}
                onMouseLeave={(e) => {
                  if (!isSelected) {
                    e.currentTarget.style.backgroundColor = 'transparent';
                  }
                }}
                onClick={() => setState(prev => ({ 
                  ...prev, 
                  selectedMidiIndex: state.activeTab === 'midi' ? index : prev.selectedMidiIndex,
                  selectedOscIndex: state.activeTab === 'osc' ? index : prev.selectedOscIndex
                }))}
              >
                {/* Slot Column - Inline Editing */}
                <div className="text-sm">
                  {isEditingSlot ? (
                    <input
                      type="text"
                      value={state.slotEditValue}
                      onChange={(e) => setState(prev => ({ ...prev, slotEditValue: e.target.value }))}
                      onKeyDown={(e) => handleSlotEditKeyDown(e, device)}
                      onBlur={() => confirmSlotEdit(device)}
                      className="w-12 px-2 py-1 text-xs border focus:outline-none focus:ring-1 text-center"
                      style={{ 
                        borderColor: 'var(--color-primary)', 
                        backgroundColor: 'var(--color-background)', 
                        color: 'var(--color-text)',
                        boxShadow: `0 0 0 1px var(--color-primary)`
                      }}
                      placeholder=""
                      autoFocus
                      onClick={(e) => e.stopPropagation()}
                    />
                  ) : (
                    <span 
                      className="cursor-pointer px-2 py-1 transition-colors duration-150 inline-block min-w-[24px] text-center"
                      style={{
                        backgroundColor: isSelected ? 'rgba(255,255,255,0.1)' : undefined,
                        color: isSelected ? 'var(--color-background)' : 'var(--color-text)'
                      }}
                      onMouseEnter={(e) => {
                        if (!isSelected) {
                          e.currentTarget.style.backgroundColor = 'var(--color-primary-100)';
                        }
                      }}
                      onMouseLeave={(e) => {
                        if (!isSelected) {
                          e.currentTarget.style.backgroundColor = 'transparent';
                        }
                      }}
                      onClick={(e) => {
                        e.stopPropagation();
                        handleSlotClick(device);
                      }}
                      title="Click to edit slot assignment (1-16)"
                    >
                      {device.id === 0 ? '--' : device.id.toString()}
                    </span>
                  )}
                </div>
                
                {/* Status Column */}
                <div className="text-sm flex items-center space-x-2">
                  {isMidiTab ? (
                    <>
                      <div 
                        className="w-2 h-2 rounded-full flex-shrink-0" 
                        style={{
                          backgroundColor: device.is_connected ? 'var(--color-success)' : 'var(--color-warning)'
                        }}
                      />
                      <span className="truncate">{device.is_connected ? 'Connected' : 'Available'}</span>
                    </>
                  ) : (
                    <>
                      <div 
                        className="w-2 h-2 rounded-full flex-shrink-0" 
                        style={{ backgroundColor: 'var(--color-info)' }}
                      />
                      <span className="truncate">Active</span>
                    </>
                  )}
                </div>
                
                {/* Name Column with Connect/Disconnect Button */}
                <div className="text-sm flex items-center justify-between min-w-0">
                  <span className="truncate flex-1 mr-2">{device.name}</span>
                  <div className="flex items-center space-x-1 flex-shrink-0">
                    {isMidiTab && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDeviceConnect(device);
                        }}
                        className="p-1.5 transition-all duration-200 flex items-center justify-center"
                        style={{ 
                          backgroundColor: device.is_connected ? 'var(--color-error)' : 'var(--color-success)',
                          color: 'white',
                          border: '1px solid transparent'
                        }}
                        onMouseEnter={(e) => {
                          e.currentTarget.style.transform = 'scale(1.1)';
                          e.currentTarget.style.opacity = '0.9';
                        }}
                        onMouseLeave={(e) => {
                          e.currentTarget.style.transform = 'scale(1)';
                          e.currentTarget.style.opacity = '1';
                        }}
                        title={device.is_connected ? 'Disconnect' : 'Connect'}
                      >
                        {device.is_connected ? <Square size={14} fill="white" /> : <Play size={14} fill="white" />}
                      </button>
                    )}
                    {!isMidiTab && (
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleRemoveOsc(device);
                        }}
                        className="p-1.5 transition-all duration-200 flex items-center justify-center"
                        style={{ 
                          backgroundColor: 'var(--color-error)',
                          color: 'white',
                          border: '1px solid transparent'
                        }}
                        onMouseEnter={(e) => {
                          e.currentTarget.style.transform = 'scale(1.1)';
                          e.currentTarget.style.opacity = '0.9';
                        }}
                        onMouseLeave={(e) => {
                          e.currentTarget.style.transform = 'scale(1)';
                          e.currentTarget.style.opacity = '1';
                        }}
                        title="Remove OSC Device"
                      >
                        <Trash2 size={14} color="white" />
                      </button>
                    )}
                  </div>
                </div>
                
                {/* Address Column (OSC only) */}
                {!isMidiTab && (
                  <div className="text-sm truncate flex items-center">
                    <span>{device.address || 'N/A'}</span>
                  </div>
                )}
              </div>
            );
          })}
          
          {/* Inline Device Creation Row */}
          <div className={`grid gap-2 p-2 transition-colors duration-150 ${
            isMidiTab ? 'grid-cols-3' : 'grid-cols-4'
          }`}
               style={{ 
                 backgroundColor: (state.isCreatingNewDevice || state.isCreatingOsc) ? 'var(--color-surface)' : 'transparent',
                 borderTop: '1px dashed var(--color-border)',
                 marginTop: '4px'
               }}
               onMouseEnter={(e) => {
                 if (!state.isCreatingNewDevice && !state.isCreatingOsc) {
                   e.currentTarget.style.backgroundColor = 'var(--color-primary-50)';
                 }
               }}
               onMouseLeave={(e) => {
                 if (!state.isCreatingNewDevice && !state.isCreatingOsc) {
                   e.currentTarget.style.backgroundColor = 'transparent';
                 }
               }}>
              
              {/* Slot Column */}
              <div className="text-sm flex items-center justify-center">
                <span className="text-center min-w-[24px]" style={{ color: 'var(--color-muted)' }}>--</span>
              </div>
              
              {/* Status Column */}
              <div className="text-sm flex items-center">
                <div className="flex items-center space-x-1">
                  <div className="w-2 h-2 rounded-full" style={{ backgroundColor: 'var(--color-muted)' }}></div>
                  <span style={{ color: 'var(--color-muted)' }}>
                    {(state.isCreatingNewDevice || state.isCreatingOsc) ? 'Creating' : 'New'}
                  </span>
                </div>
              </div>
              
              {/* Name Column */}
              <div className="text-sm flex items-center justify-between">
                {(state.isCreatingNewDevice || state.isCreatingOsc) ? (
                  <input
                    type="text"
                    value={isMidiTab ? state.newDeviceInput : 
                           state.oscStep === 0 ? state.oscName :
                           state.oscStep === 1 ? state.oscIp : state.oscPort}
                    onChange={(e) => {
                      if (isMidiTab) {
                        setState(prev => ({ ...prev, newDeviceInput: e.target.value }));
                      } else {
                        if (state.oscStep === 0) {
                          setState(prev => ({ ...prev, oscName: e.target.value }));
                        } else if (state.oscStep === 1) {
                          setState(prev => ({ ...prev, oscIp: e.target.value }));
                        } else {
                          setState(prev => ({ ...prev, oscPort: e.target.value }));
                        }
                      }
                    }}
                    onKeyDown={isMidiTab ? handleNewDeviceKeyDown : handleOscFieldKeyDown}
                    onBlur={() => {
                      if (isMidiTab) {
                        if (state.newDeviceInput.trim()) {
                          confirmNewDeviceCreation();
                        } else {
                          setState(prev => ({ ...prev, isCreatingNewDevice: false }));
                        }
                      } else {
                        // For OSC, only cancel if we're at step 0 and no name is entered
                        if (state.oscStep === 0 && !state.oscName.trim()) {
                          handleCancelOscCreation();
                        }
                      }
                    }}
                    className="flex-1 px-2 py-1 text-sm border focus:outline-none focus:ring-1"
                    style={{ 
                      borderColor: 'var(--color-primary)', 
                      backgroundColor: 'var(--color-background)', 
                      color: 'var(--color-text)',
                      boxShadow: `0 0 0 1px var(--color-primary)`
                    }}
                    placeholder={isMidiTab ? "Enter device name" : 
                                state.oscStep === 0 ? "Device name" :
                                state.oscStep === 1 ? "IP Address (e.g., 127.0.0.1)" : "Port (e.g., 57120)"}
                    autoFocus
                  />
                ) : (
                  <div className="flex items-center w-full">
                    <button
                      onClick={isMidiTab ? handleCreateNewDevice : handleCreateOsc}
                      className="flex items-center space-x-2 px-3 py-1 text-sm transition-colors duration-150 w-full justify-center"
                      style={{ 
                        color: 'var(--color-muted)',
                        backgroundColor: 'transparent',
                        border: `1px dashed var(--color-border)`
                      }}
                      onMouseEnter={(e) => {
                        e.currentTarget.style.backgroundColor = 'var(--color-primary-50)';
                        e.currentTarget.style.color = 'var(--color-primary)';
                        e.currentTarget.style.borderColor = 'var(--color-primary)';
                      }}
                      onMouseLeave={(e) => {
                        e.currentTarget.style.backgroundColor = 'transparent';
                        e.currentTarget.style.color = 'var(--color-muted)';
                        e.currentTarget.style.borderColor = 'var(--color-border)';
                      }}
                    >
                      <Plus size={16} />
                      <span>{isMidiTab ? 'Add Virtual MIDI' : 'Add OSC output'}</span>
                    </button>
                  </div>
                )}
              </div>
              
            {/* Address Column (OSC only) */}
            {!isMidiTab && (
              <div className="text-sm flex items-center">
                {state.isCreatingOsc ? (
                  <div className="flex items-center space-x-2 text-xs" style={{ color: 'var(--color-muted)' }}>
                    <span>Step {state.oscStep + 1}/3:</span>
                    <span>
                      {state.oscStep === 0 ? 'Name' : 
                       state.oscStep === 1 ? 'IP Address' : 'Port'}
                    </span>
                  </div>
                ) : (
                  <span style={{ color: 'var(--color-muted)' }}>New Connection</span>
                )}
              </div>
            )}
          </div>
        </div>
      </div>
    );
  };



  if (!state.isConnected) {
    return (
      <div className="p-4 text-center">
        <div className="text-gray-500">Not connected to server</div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Tab Navigation */}
      <div className="flex border-b" style={{ borderColor: 'var(--color-border)' }}>
        <button
          onClick={() => setState(prev => ({ ...prev, activeTab: 'midi' }))}
          className={`flex-1 flex items-center justify-center py-3 px-4 transition-colors duration-150 ${
            state.activeTab === 'midi' ? 'border-b-2' : ''
          }`}
          style={{
            color: state.activeTab === 'midi' ? 'var(--color-primary)' : 'var(--color-muted)',
            borderBottomColor: state.activeTab === 'midi' ? 'var(--color-primary)' : 'transparent',
          }}
        >
          <div className="flex items-center space-x-2">
            <Music size={18} />
            <span className="font-medium">MIDI</span>
          </div>
        </button>
        <button
          onClick={() => setState(prev => ({ ...prev, activeTab: 'osc' }))}
          className={`flex-1 flex items-center justify-center py-3 px-4 transition-colors duration-150 ${
            state.activeTab === 'osc' ? 'border-b-2' : ''
          }`}
          style={{
            color: state.activeTab === 'osc' ? 'var(--color-primary)' : 'var(--color-muted)',
            borderBottomColor: state.activeTab === 'osc' ? 'var(--color-primary)' : 'transparent',
          }}
        >
          <div className="flex items-center space-x-2">
            <Wifi size={18} />
            <span className="font-medium">OSC</span>
          </div>
        </button>
      </div>

      {/* Device Table */}
      <div className="flex-1 overflow-hidden">
        {renderDeviceTable()}
      </div>


      {/* Status Message */}
      {state.statusMessage && (
        <div className="p-2 text-sm text-center" style={{ color: 'var(--color-muted)' }}>
          {state.statusMessage}
        </div>
      )}

    </div>
  );
};