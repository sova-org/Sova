<script lang="ts">
    import { onMount } from "svelte";
    import { Play, Square, X, Trash2 } from "lucide-svelte";
    import { devices } from "$lib/stores";
    import {
        requestDeviceList,
        connectMidiDevice,
        disconnectMidiDevice,
        createVirtualMidiOutput,
        assignDeviceToSlot,
        unassignDeviceFromSlot,
        createOscDevice,
        removeOscDevice,
    } from "$lib/api/client";
    import type { DeviceInfo } from "$lib/types/protocol";

    onMount(() => {
        requestDeviceList();
    });

    let creatingVirtualMidi = $state(false);
    let newMidiName = $state("");
    let creatingOsc = $state(false);
    let oscStep = $state(0);
    let oscName = $state("");
    let oscIp = $state("127.0.0.1");
    let oscPort = $state("57120");
    let editingSlot = $state<string | null>(null);
    let slotEditValue = $state("");

    function handleConnectToggle(device: DeviceInfo) {
        if (device.is_connected) {
            disconnectMidiDevice(device.name);
        } else {
            connectMidiDevice(device.name);
        }
    }

    function startVirtualMidiCreation() {
        creatingOsc = false;
        creatingVirtualMidi = true;
        newMidiName = "";
    }

    function handleCreateVirtualMidi() {
        if (newMidiName.trim()) {
            createVirtualMidiOutput(newMidiName.trim());
            creatingVirtualMidi = false;
            newMidiName = "";
        }
    }

    function cancelVirtualMidiCreation() {
        creatingVirtualMidi = false;
        newMidiName = "";
    }

    function startSlotEdit(deviceName: string, currentSlot: number) {
        editingSlot = deviceName;
        slotEditValue = !currentSlot ? "" : String(currentSlot);
    }

    function handleSlotUpdate(deviceName: string) {
        const slotNum = parseInt(slotEditValue);

        if (slotEditValue === "" || slotNum === 0) {
            const device = $devices.find((d) => d.name === deviceName);
            if (device && device.slot_id !== 0) {
                unassignDeviceFromSlot(device.slot_id);
            }
        } else if (!isNaN(slotNum) && slotNum >= 1 && slotNum <= 16) {
            assignDeviceToSlot(slotNum, deviceName);
        }

        editingSlot = null;
        slotEditValue = "";
    }

    function cancelSlotEdit() {
        editingSlot = null;
        slotEditValue = "";
    }

    function startOscCreation() {
        creatingVirtualMidi = false;
        creatingOsc = true;
        oscStep = 0;
        oscName = "";
        oscIp = "127.0.0.1";
        oscPort = "57120";
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
    <div class="devices-content">
        <div class="devices-list">
            {#each $devices as device (device.name)}
                <div class="device-row" class:missing={device.is_missing}>
                    <div class="col-type">{device.kind === "Midi" ? "MIDI" : "OSC"}</div>
                    <div class="col-slot">
                        {#if editingSlot === device.name}
                            <!-- svelte-ignore a11y_autofocus -->
                            <input
                                type="text"
                                class="slot-input"
                                bind:value={slotEditValue}
                                onkeydown={(e) => {
                                    if (e.key === "Enter") handleSlotUpdate(device.name);
                                    if (e.key === "Escape") cancelSlotEdit();
                                }}
                                onblur={() => handleSlotUpdate(device.name)}
                                autofocus
                            />
                        {:else}
                            <button
                                class="slot-button"
                                onclick={() => startSlotEdit(device.name, device.slot_id)}
                                data-help-id="devices-slot"
                            >
                                {device.slot_id === 0 ? "" : device.slot_id}
                            </button>
                        {/if}
                    </div>
                    <div class="col-status">
                        {#if device.is_missing}
                            <span class="status-indicator missing"></span>
                            Missing
                        {:else if device.kind === "Midi"}
                            <span
                                class="status-indicator"
                                class:connected={device.is_connected}
                            ></span>
                            {device.is_connected ? "Connected" : "Available"}
                        {:else}
                            <span class="status-indicator active"></span>
                            Active
                        {/if}
                    </div>
                    <div class="col-name">{device.name}</div>
                    <div class="col-address">{device.address || ""}</div>
                    <div class="col-action">
                        {#if device.is_missing}
                            <!-- No action button for missing devices -->
                        {:else if device.kind === "Midi"}
                            <button
                                class="action-button"
                                class:connect={!device.is_connected}
                                class:disconnect={device.is_connected}
                                onclick={() => handleConnectToggle(device)}
                                data-help-id="devices-connect"
                            >
                                {#if device.is_connected}
                                    <Square size={14} />
                                {:else}
                                    <Play size={14} />
                                {/if}
                            </button>
                        {:else}
                            <button
                                class="action-button remove"
                                onclick={() => handleRemoveOsc(device.name)}
                                data-help-id="devices-remove"
                            >
                                <Trash2 size={14} />
                            </button>
                        {/if}
                    </div>
                </div>
            {/each}

            {#if creatingVirtualMidi}
                <div class="device-row creating">
                    <div class="col-type">MIDI</div>
                    <div class="col-slot"></div>
                    <div class="col-status">New</div>
                    <div class="col-name">
                        <!-- svelte-ignore a11y_autofocus -->
                        <input
                            type="text"
                            class="name-input"
                            bind:value={newMidiName}
                            placeholder="Device name..."
                            onkeydown={(e) => {
                                if (e.key === "Enter") handleCreateVirtualMidi();
                                if (e.key === "Escape") cancelVirtualMidiCreation();
                            }}
                            autofocus
                        />
                    </div>
                    <div class="col-address"></div>
                    <div class="col-action">
                        <button class="action-button cancel" onclick={cancelVirtualMidiCreation}>
                            <X size={14} />
                        </button>
                    </div>
                </div>
            {/if}

            {#if creatingOsc}
                <div class="device-row creating">
                    <div class="col-type">OSC</div>
                    <div class="col-slot"></div>
                    <div class="col-status">New</div>
                    <div class="col-name">
                        {#if oscStep === 0}
                            <!-- svelte-ignore a11y_autofocus -->
                            <input
                                type="text"
                                class="name-input"
                                bind:value={oscName}
                                placeholder="Device name..."
                                onkeydown={(e) => {
                                    if (e.key === "Enter") handleOscNext();
                                    if (e.key === "Escape") cancelOscCreation();
                                }}
                                autofocus
                            />
                            <span class="step-indicator">Step 1/3: Name</span>
                        {:else if oscStep === 1}
                            <!-- svelte-ignore a11y_autofocus -->
                            <input
                                type="text"
                                class="name-input"
                                bind:value={oscIp}
                                placeholder="IP address..."
                                onkeydown={(e) => {
                                    if (e.key === "Enter") handleOscNext();
                                    if (e.key === "Escape") cancelOscCreation();
                                }}
                                autofocus
                            />
                            <span class="step-indicator">Step 2/3: IP Address</span>
                        {:else}
                            <!-- svelte-ignore a11y_autofocus -->
                            <input
                                type="text"
                                class="name-input"
                                bind:value={oscPort}
                                placeholder="Port..."
                                onkeydown={(e) => {
                                    if (e.key === "Enter") handleOscNext();
                                    if (e.key === "Escape") cancelOscCreation();
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
            {/if}

            {#if !creatingVirtualMidi && !creatingOsc}
                <div class="add-row">
                    <button
                        class="add-button"
                        onclick={startVirtualMidiCreation}
                        data-help-id="devices-add-midi"
                    >
                        + Virtual MIDI
                    </button>
                    <button
                        class="add-button"
                        onclick={startOscCreation}
                        data-help-id="devices-add-osc"
                    >
                        + OSC Output
                    </button>
                </div>
            {/if}
        </div>
    </div>
</div>

<style>
    .devices-view {
        --color-success: var(--ansi-green, #4ade80);
        --color-danger: var(--colors-danger, #f87171);
        --color-info: var(--ansi-blue, #60a5fa);
        --color-inactive: var(--colors-text-secondary, #666);

        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        background-color: var(--colors-background);
    }

    .devices-content {
        flex: 1;
        overflow: auto;
        padding: 16px;
    }

    .devices-list {
        font-family: monospace;
        font-size: 13px;
    }

    .device-row {
        display: flex;
        flex-wrap: wrap;
        gap: 8px 16px;
        padding: 10px 12px;
        border-bottom: 1px solid var(--colors-border, #333);
        align-items: center;
    }

    .device-row:hover {
        background-color: var(--colors-surface, #2d2d2d);
    }

    .device-row.creating {
        background-color: var(--colors-surface, #2d2d2d);
    }

    .device-row.missing {
        opacity: 0.5;
    }

    .device-row.missing:hover {
        opacity: 0.7;
    }

    .col-type {
        color: var(--colors-text-secondary, #888);
        font-size: 11px;
        width: 35px;
    }

    .col-slot {
        width: 40px;
    }

    .col-status {
        display: flex;
        align-items: center;
    }

    .col-name {
        flex: 1;
        min-width: 100px;
    }

    .col-address {
        color: var(--colors-text-secondary, #888);
    }

    .col-action {
        margin-left: auto;
    }

    .slot-button {
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 40px;
        height: 22px;
        background: none;
        border: 1px solid var(--colors-border, #333);
        color: var(--colors-text-secondary, #888);
        font-family: monospace;
        font-size: 13px;
        padding: 0;
        cursor: pointer;
        box-sizing: border-box;
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
        display: flex;
        gap: 8px;
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
        flex: 1;
    }

    .add-button:hover {
        border-color: var(--colors-accent, #0e639c);
        color: var(--colors-text, #fff);
    }
</style>
