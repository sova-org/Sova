<script lang="ts">
  import { onMount } from 'svelte';
  import { Play, Square, X, Trash2 } from 'lucide-svelte';
  import { devices, midiDevices, oscDevices } from '$lib/stores';
  import {
    requestDeviceList,
    connectMidiDevice,
    disconnectMidiDevice,
    createVirtualMidiOutput,
    assignDeviceToSlot,
    unassignDeviceFromSlot,
    createOscDevice,
    removeOscDevice
  } from '$lib/api/client';
  import type { DeviceInfo } from '$lib/types/protocol';

  // Request device list when view mounts
  onMount(() => {
    requestDeviceList();
  });

  let activeTab = $state<'midi' | 'osc'>('midi');
  let creatingVirtualMidi = $state(false);
  let newMidiName = $state('');
  let creatingOsc = $state(false);
  let oscStep = $state(0);
  let oscName = $state('');
  let oscIp = $state('127.0.0.1');
  let oscPort = $state('57120');
  let editingSlot = $state<string | null>(null);
  let slotEditValue = $state('');

  function handleConnectToggle(device: DeviceInfo) {
    if (device.is_connected) {
      disconnectMidiDevice(device.name);
    } else {
      connectMidiDevice(device.name);
    }
  }

  function startVirtualMidiCreation() {
    creatingVirtualMidi = true;
    newMidiName = '';
  }

  function handleCreateVirtualMidi() {
    if (newMidiName.trim()) {
      createVirtualMidiOutput(newMidiName.trim());
      creatingVirtualMidi = false;
      newMidiName = '';
    }
  }

  function cancelVirtualMidiCreation() {
    creatingVirtualMidi = false;
    newMidiName = '';
  }

  function startSlotEdit(deviceName: string, currentSlot: number) {
    editingSlot = deviceName;
    slotEditValue = currentSlot === 0 ? '' : String(currentSlot);
  }

  function handleSlotUpdate(deviceName: string) {
    const slotNum = parseInt(slotEditValue);

    if (slotEditValue === '' || slotNum === 0) {
      // Get device's current slot
      const device = $devices.find(d => d.name === deviceName);
      if (device && device.id !== 0) {
        unassignDeviceFromSlot(device.id);
      }
    } else if (!isNaN(slotNum) && slotNum >= 1 && slotNum <= 16) {
      assignDeviceToSlot(slotNum, deviceName);
    }

    editingSlot = null;
    slotEditValue = '';
  }

  function cancelSlotEdit() {
    editingSlot = null;
    slotEditValue = '';
  }

  function startOscCreation() {
    creatingOsc = true;
    oscStep = 0;
    oscName = '';
    oscIp = '127.0.0.1';
    oscPort = '57120';
  }

  function handleOscNext() {
    if (oscStep === 0) {
      if (!oscName.trim()) return;
      oscStep = 1;
    } else if (oscStep === 1) {
      if (!oscIp.trim()) return;
      oscStep = 2;
    } else if (oscStep === 2) {
      const port = parseInt(oscPort);
      if (isNaN(port) || port < 1 || port > 65535) return;
      createOscDevice(oscName.trim(), oscIp.trim(), port);
      creatingOsc = false;
    }
  }

  function cancelOscCreation() {
    creatingOsc = false;
    oscStep = 0;
  }

  function handleRemoveOsc(deviceName: string) {
    removeOscDevice(deviceName);
  }
</script>

<div class="devices-view">
  <div class="tabs">
    <button
      class="tab"
      class:active={activeTab === 'midi'}
      onclick={() => activeTab = 'midi'}>
      MIDI
    </button>
    <button
      class="tab"
      class:active={activeTab === 'osc'}
      onclick={() => activeTab = 'osc'}>
      OSC
    </button>
  </div>

  <div class="devices-content">
    {#if activeTab === 'midi'}
      <div class="devices-table">
        <div class="table-header">
          <div class="col-slot">Slot</div>
          <div class="col-status">Status</div>
          <div class="col-name">Name</div>
          <div class="col-action"></div>
        </div>

        {#each $midiDevices as device}
          <div class="device-row">
            <div class="col-slot">
              {#if editingSlot === device.name}
                <input
                  type="text"
                  class="slot-input"
                  bind:value={slotEditValue}
                  onkeydown={(e) => {
                    if (e.key === 'Enter') handleSlotUpdate(device.name);
                    if (e.key === 'Escape') cancelSlotEdit();
                  }}
                  onblur={() => handleSlotUpdate(device.name)}
                  autofocus
                />
              {:else}
                <button
                  class="slot-button"
                  onclick={() => startSlotEdit(device.name, device.id)}>
                  {device.id === 0 ? '--' : device.id}
                </button>
              {/if}
            </div>
            <div class="col-status">
              <span class="status-indicator" class:connected={device.is_connected}></span>
              {device.is_connected ? 'Connected' : 'Available'}
            </div>
            <div class="col-name">{device.name}</div>
            <div class="col-action">
              <button
                class="action-button"
                class:connect={!device.is_connected}
                class:disconnect={device.is_connected}
                onclick={() => handleConnectToggle(device)}>
                {#if device.is_connected}
                  <Square size={14} />
                {:else}
                  <Play size={14} />
                {/if}
              </button>
            </div>
          </div>
        {/each}

        {#if creatingVirtualMidi}
          <div class="device-row creating">
            <div class="col-slot">--</div>
            <div class="col-status">New</div>
            <div class="col-name">
              <input
                type="text"
                class="name-input"
                bind:value={newMidiName}
                placeholder="Device name..."
                onkeydown={(e) => {
                  if (e.key === 'Enter') handleCreateVirtualMidi();
                  if (e.key === 'Escape') cancelVirtualMidiCreation();
                }}
                autofocus
              />
            </div>
            <div class="col-action">
              <button class="action-button cancel" onclick={cancelVirtualMidiCreation}>
                <X size={14} />
              </button>
            </div>
          </div>
        {:else}
          <div class="add-row">
            <button class="add-button" onclick={startVirtualMidiCreation}>
              + Add Virtual MIDI
            </button>
          </div>
        {/if}
      </div>
    {:else}
      <div class="devices-table">
        <div class="table-header">
          <div class="col-slot">Slot</div>
          <div class="col-status">Status</div>
          <div class="col-name">Name</div>
          <div class="col-address">Address</div>
          <div class="col-action"></div>
        </div>

        {#each $oscDevices as device}
          <div class="device-row">
            <div class="col-slot">
              {#if editingSlot === device.name}
                <input
                  type="text"
                  class="slot-input"
                  bind:value={slotEditValue}
                  onkeydown={(e) => {
                    if (e.key === 'Enter') handleSlotUpdate(device.name);
                    if (e.key === 'Escape') cancelSlotEdit();
                  }}
                  onblur={() => handleSlotUpdate(device.name)}
                  autofocus
                />
              {:else}
                <button
                  class="slot-button"
                  onclick={() => startSlotEdit(device.name, device.id)}>
                  {device.id === 0 ? '--' : device.id}
                </button>
              {/if}
            </div>
            <div class="col-status">
              <span class="status-indicator active"></span>
              Active
            </div>
            <div class="col-name">{device.name}</div>
            <div class="col-address">{device.address || ''}</div>
            <div class="col-action">
              <button
                class="action-button remove"
                onclick={() => handleRemoveOsc(device.name)}>
                <Trash2 size={14} />
              </button>
            </div>
          </div>
        {/each}

        {#if creatingOsc}
          <div class="device-row creating">
            <div class="col-slot">--</div>
            <div class="col-status">New</div>
            <div class="col-name">
              {#if oscStep === 0}
                <input
                  type="text"
                  class="name-input"
                  bind:value={oscName}
                  placeholder="Device name..."
                  onkeydown={(e) => {
                    if (e.key === 'Enter') handleOscNext();
                    if (e.key === 'Escape') cancelOscCreation();
                  }}
                  autofocus
                />
                <span class="step-indicator">Step 1/3: Name</span>
              {:else if oscStep === 1}
                <input
                  type="text"
                  class="name-input"
                  bind:value={oscIp}
                  placeholder="IP address..."
                  onkeydown={(e) => {
                    if (e.key === 'Enter') handleOscNext();
                    if (e.key === 'Escape') cancelOscCreation();
                  }}
                  autofocus
                />
                <span class="step-indicator">Step 2/3: IP Address</span>
              {:else}
                <input
                  type="text"
                  class="name-input"
                  bind:value={oscPort}
                  placeholder="Port..."
                  onkeydown={(e) => {
                    if (e.key === 'Enter') handleOscNext();
                    if (e.key === 'Escape') cancelOscCreation();
                  }}
                  autofocus
                />
                <span class="step-indicator">Step 3/3: Port</span>
              {/if}
            </div>
            <div class="col-address"></div>
            <div class="col-action">
              <button class="action-button cancel" onclick={cancelOscCreation}>
                <X size={14} />
              </button>
            </div>
          </div>
        {:else}
          <div class="add-row">
            <button class="add-button" onclick={startOscCreation}>
              + Add OSC Output
            </button>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .devices-view {
    --grid-columns: 60px 120px 1fr 150px 60px;
    --color-success: #4ade80;
    --color-danger: #f87171;
    --color-info: #60a5fa;
    --color-inactive: #666;

    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--colors-background);
  }

  .tabs {
    display: flex;
    gap: 4px;
    padding: 16px 16px 0 16px;
    border-bottom: 1px solid var(--colors-border, #333);
  }

  .tab {
    background: none;
    border: none;
    color: var(--colors-text-secondary, #888);
    font-family: monospace;
    font-size: 13px;
    font-weight: 600;
    padding: 8px 16px;
    cursor: pointer;
    border-bottom: 2px solid transparent;
  }

  .tab.active {
    color: var(--colors-text, #fff);
    border-bottom-color: var(--colors-accent, #0e639c);
  }

  .devices-content {
    flex: 1;
    overflow: auto;
    padding: 16px;
  }

  .devices-table {
    font-family: monospace;
    font-size: 13px;
  }

  .table-header {
    display: grid;
    grid-template-columns: var(--grid-columns);
    padding: 8px 12px;
    color: var(--colors-text-secondary, #888);
    border-bottom: 1px solid var(--colors-border, #333);
    font-weight: 600;
  }

  .device-row {
    display: grid;
    grid-template-columns: var(--grid-columns);
    padding: 8px 12px;
    border-bottom: 1px solid var(--colors-border, #333);
    align-items: center;
  }

  .device-row:hover {
    background-color: var(--colors-surface, #2d2d2d);
  }

  .device-row.creating {
    background-color: var(--colors-surface, #2d2d2d);
  }

  .col-address {
    color: var(--colors-text-secondary, #888);
  }

  .slot-button {
    background: none;
    border: 1px solid var(--colors-border, #333);
    color: var(--colors-text-secondary, #888);
    font-family: monospace;
    font-size: 13px;
    padding: 2px 8px;
    cursor: pointer;
    min-width: 40px;
  }

  .slot-button:hover {
    border-color: var(--colors-accent, #0e639c);
    color: var(--colors-text, #fff);
  }

  .slot-input {
    background: var(--colors-background);
    border: 1px solid var(--colors-accent, #0e639c);
    color: var(--colors-text, #fff);
    font-family: monospace;
    font-size: 13px;
    padding: 2px 8px;
    width: 40px;
    outline: none;
  }

  .status-indicator {
    display: inline-block;
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background-color: var(--color-inactive);
    margin-right: 8px;
  }

  .status-indicator.connected {
    background-color: var(--color-success);
  }

  .status-indicator.active {
    background-color: var(--color-info);
  }

  .action-button {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: 1px solid var(--colors-border, #333);
    color: var(--colors-text-secondary, #888);
    font-family: monospace;
    font-size: 14px;
    padding: 4px 12px;
    cursor: pointer;
  }

  .action-button.connect {
    color: var(--color-success);
  }

  .action-button.disconnect {
    color: var(--color-danger);
  }

  .action-button.remove {
    color: var(--color-danger);
  }

  .action-button.connect:hover,
  .action-button.disconnect:hover,
  .action-button.remove:hover {
    border-color: currentColor;
  }

  .action-button.cancel:hover {
    border-color: var(--colors-accent, #0e639c);
  }

  .name-input {
    background: var(--colors-background);
    border: 1px solid var(--colors-accent, #0e639c);
    color: var(--colors-text, #fff);
    font-family: monospace;
    font-size: 13px;
    padding: 4px 8px;
    width: 100%;
    outline: none;
  }

  .step-indicator {
    display: block;
    color: var(--colors-text-secondary, #888);
    font-size: 11px;
    margin-top: 4px;
  }

  .add-row {
    padding: 16px 12px;
  }

  .add-button {
    background: none;
    border: 1px dashed var(--colors-border, #333);
    color: var(--colors-text-secondary, #888);
    font-family: monospace;
    font-size: 13px;
    padding: 8px 16px;
    cursor: pointer;
    width: 100%;
  }

  .add-button:hover {
    border-color: var(--colors-accent, #0e639c);
    color: var(--colors-text, #fff);
  }
</style>
