<script lang="ts">
    import {
        Play,
        Pause,
        LogOut,
        Users,
        User,
        HelpCircle,
        Save,
    } from "lucide-svelte";
    import { isConnected } from "$lib/stores/connectionState";
    import { isPlaying, isStarting, clockState } from "$lib/stores/transport";
    import { peerCount } from "$lib/stores/collaboration";
    import { runtimeNickname, setRuntimeNickname } from "$lib/stores/config";
    import {
        startTransport,
        stopTransport,
        setTempo,
        setName,
    } from "$lib/api/client";
    import { invoke } from "@tauri-apps/api/core";
        import AboutModal from "./AboutModal.svelte";
    import { isHelpModeActive, toggleHelpMode } from "$lib/stores/helpMode";
    import { initiateSave, projectExists } from "$lib/stores/projects";
    import { commandPalette } from "$lib/stores/commandPalette";

    const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
    const shortcutKey = isMac ? "⌘K" : "Ctrl+K";

    let showAbout = $state(false);

    let isEditingTempo = $state(false);
    let tempTempoValue = $state("120");
    let tempoInputElement: HTMLInputElement;

    let isEditingNickname = $state(false);
    let tempNicknameValue = $state("");
    let nicknameInputElement: HTMLInputElement;

    let showSaveModal = $state(false);
    let saveNameInput = $state("");
    let showOverwriteConfirm = $state(false);
    let saveInputElement: HTMLInputElement;

    let barProgress = $derived(
        $clockState !== null
            ? (($clockState.beat % $clockState.quantum) / $clockState.quantum) *
                  100
            : 0,
    );

    $effect(() => {
        function handleEditNickname() {
            if ($isConnected && $runtimeNickname) {
                startEditingNickname();
            }
        }

        function handleOpenSaveModal() {
            if ($isConnected) {
                openSaveModal();
            }
        }

        window.addEventListener("command:edit-nickname", handleEditNickname);
        window.addEventListener("command:open-save-modal", handleOpenSaveModal);

        return () => {
            window.removeEventListener(
                "command:edit-nickname",
                handleEditNickname,
            );
            window.removeEventListener(
                "command:open-save-modal",
                handleOpenSaveModal,
            );
        };
    });

    async function handleDisconnect() {
        try {
            await invoke("disconnect_client");
            isConnected.set(false);
        } catch {
            // Disconnect failed - connection likely already closed
        }
    }

    function startEditingTempo() {
        if ($clockState !== null) {
            tempTempoValue = Math.round($clockState.tempo).toString();
            isEditingTempo = true;
            requestAnimationFrame(() => tempoInputElement?.select());
        }
    }

    function cancelEditingTempo() {
        isEditingTempo = false;
    }

    async function saveTempoEdit() {
        const tempo = parseFloat(tempTempoValue);

        // Validate tempo (typically 30-300 BPM)
        if (isNaN(tempo) || tempo < 30 || tempo > 300) {
            cancelEditingTempo();
            return;
        }

        try {
            await setTempo(tempo);
            isEditingTempo = false;
        } catch (error) {
            console.error("Failed to set tempo:", error);
            cancelEditingTempo();
        }
    }

    function handleTempoKeydown(event: KeyboardEvent) {
        if (event.key === "Enter") {
            event.preventDefault();
            saveTempoEdit();
        } else if (event.key === "Escape") {
            event.preventDefault();
            cancelEditingTempo();
        }
    }

    function startEditingNickname() {
        tempNicknameValue = $runtimeNickname;
        isEditingNickname = true;
        requestAnimationFrame(() => nicknameInputElement?.select());
    }

    function cancelEditingNickname() {
        isEditingNickname = false;
    }

    async function saveNicknameEdit() {
        const nickname = tempNicknameValue.trim();
        if (!nickname) {
            cancelEditingNickname();
            return;
        }

        try {
            // Update runtime nickname locally (does NOT write to TOML)
            setRuntimeNickname(nickname);

            // Send to server if connected
            if ($isConnected) {
                await setName(nickname);
            }
            isEditingNickname = false;
        } catch (error) {
            console.error("Failed to set nickname:", error);
            cancelEditingNickname();
        }
    }

    function handleNicknameKeydown(event: KeyboardEvent) {
        if (event.key === "Enter") {
            event.preventDefault();
            saveNicknameEdit();
        } else if (event.key === "Escape") {
            event.preventDefault();
            cancelEditingNickname();
        }
    }

    function openSaveModal() {
        saveNameInput = "";
        showSaveModal = true;
        requestAnimationFrame(() => saveInputElement?.focus());
    }

    function closeSaveModal() {
        showSaveModal = false;
        showOverwriteConfirm = false;
        saveNameInput = "";
    }

    function handleSaveSubmit() {
        if (!saveNameInput.trim()) return;
        if (projectExists(saveNameInput.trim())) {
            showOverwriteConfirm = true;
        } else {
            doSave();
        }
    }

    async function doSave() {
        await initiateSave(saveNameInput.trim());
        closeSaveModal();
    }

    function handleSaveKeydown(event: KeyboardEvent) {
        if (event.key === "Enter") {
            event.preventDefault();
            handleSaveSubmit();
        } else if (event.key === "Escape") {
            closeSaveModal();
        }
    }
</script>

<div class="topbar">
    <div class="left-section">
        <button
            class="app-name"
            data-help-id="app-name"
            onclick={() => (showAbout = true)}>Sova</button
        >

        {#if $isConnected}
            <button
                class="save-btn"
                data-help-id="quick-save"
                onclick={openSaveModal}
                title="Save snapshot"
            >
                <Save size={14} />
            </button>
        {/if}

        {#if $isConnected}
            <div class="actions">
                <div
                    class="bar-progress"
                    class:playing={$isPlaying || $isStarting}
                    style="width: {barProgress}%"
                ></div>

                <button
                    class="transport-button play-button"
                    data-help-id="play-button"
                    onclick={() =>
                        $isPlaying || $isStarting
                            ? stopTransport()
                            : startTransport()}
                >
                    {#if $isPlaying || $isStarting}
                        <Pause size={16} />
                    {:else}
                        <Play size={16} />
                    {/if}
                </button>

                <span class="transport-info" data-help-id="beat-display">
                    {#if $clockState !== null}
                        {$clockState.beat.toFixed(1)}
                    {:else}
                        --
                    {/if}
                </span>

                {#if isEditingTempo}
                    <input
                        bind:this={tempoInputElement}
                        bind:value={tempTempoValue}
                        onkeydown={handleTempoKeydown}
                        onblur={saveTempoEdit}
                        class="tempo-input"
                        type="number"
                        min="30"
                        max="300"
                        step="1"
                    />
                    <span class="tempo-unit">BPM</span>
                {:else}
                    <span
                        class="transport-info tempo-clickable"
                        data-help-id="tempo-display"
                        onclick={startEditingTempo}
                        onkeydown={(e) =>
                            e.key === "Enter" && startEditingTempo()}
                        role="button"
                        tabindex="0"
                    >
                        {$clockState !== null
                            ? `${Math.round($clockState.tempo)} BPM`
                            : "-- BPM"}
                    </span>
                {/if}
            </div>

            {#if $runtimeNickname}
                {#if isEditingNickname}
                    <input
                        bind:this={nicknameInputElement}
                        bind:value={tempNicknameValue}
                        onkeydown={handleNicknameKeydown}
                        onblur={saveNicknameEdit}
                        class="nickname-input"
                        type="text"
                    />
                {:else}
                    <span
                        class="nickname-display"
                        data-help-id="nickname-display"
                        onclick={startEditingNickname}
                        onkeydown={(e) =>
                            e.key === "Enter" && startEditingNickname()}
                        role="button"
                        tabindex="0"
                    >
                        <User size={12} />
                        {$runtimeNickname}
                    </span>
                {/if}
            {/if}

            {#if $peerCount > 0}
                <span class="peer-count" data-help-id="peer-count">
                    <Users size={12} />
                    {$peerCount}
                </span>
            {/if}
        {/if}
    </div>

    <div class="right-section">
        <button
            class="command-btn"
            data-help-id="command-button"
            onclick={() => commandPalette.open()}
            title="Command palette"
        >
            Cmd ({shortcutKey})
        </button>
        <button
            class="help-btn"
            class:active={$isHelpModeActive}
            data-help-id="help-button"
            onclick={toggleHelpMode}
            title="Help mode"
        >
            <HelpCircle size={16} />
        </button>
        {#if $isConnected}
            <button
                class="disconnect-button"
                data-help-id="disconnect-button"
                onclick={handleDisconnect}
                title="Disconnect"
            >
                <LogOut size={16} />
                <span class="disconnect-text">Disconnect</span>
            </button>
        {/if}
    </div>
</div>

<AboutModal bind:open={showAbout} />

{#if showSaveModal}
    <div
        class="modal-overlay"
        onclick={closeSaveModal}
        onkeydown={(e) => e.key === "Escape" && closeSaveModal()}
        role="presentation"
    >
        <div
            class="modal"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
        >
            {#if showOverwriteConfirm}
                <div class="modal-title">Overwrite Project?</div>
                <div class="modal-message">
                    A project named "{saveNameInput}" already exists.
                </div>
                <div class="modal-buttons">
                    <button
                        class="modal-button"
                        onclick={() => (showOverwriteConfirm = false)}
                        >Cancel</button
                    >
                    <button class="modal-button confirm" onclick={doSave}
                        >Overwrite</button
                    >
                </div>
            {:else}
                <div class="modal-title">Save Snapshot</div>
                <input
                    bind:this={saveInputElement}
                    type="text"
                    class="modal-input"
                    placeholder="Project name..."
                    bind:value={saveNameInput}
                    onkeydown={handleSaveKeydown}
                />
                <div class="modal-buttons">
                    <button class="modal-button" onclick={closeSaveModal}
                        >Cancel</button
                    >
                    <button
                        class="modal-button confirm"
                        onclick={handleSaveSubmit}>Save</button
                    >
                </div>
            {/if}
        </div>
    </div>
{/if}

<style>
    .topbar {
        width: 100%;
        height: 40px;
        box-sizing: border-box;
        background-color: var(--colors-background, #1e1e1e);
        border-bottom: 1px solid var(--colors-border, #333);
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding: 0 12px;
        gap: 8px;
        overflow: hidden;
    }

    .left-section {
        display: flex;
        align-items: center;
        gap: 8px;
        min-width: 0;
    }

    .right-section {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        flex-shrink: 0;
        gap: 8px;
    }

    .app-name {
        font-family: monospace;
        font-size: 13px;
        font-weight: 700;
        color: var(--colors-text, #fff);
        letter-spacing: 0.5px;
        padding: 4px 8px;
        background: none;
        border: none;
        cursor: pointer;
        transition: color 0.2s;
    }

    .app-name:hover {
        color: var(--colors-accent, #0e639c);
    }

    .help-btn {
        background: none;
        border: 1px solid var(--colors-border, #333);
        color: var(--colors-text-secondary, #888);
        padding: 6px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s;
    }

    .help-btn:hover {
        border-color: var(--colors-accent, #0e639c);
        color: var(--colors-text, #fff);
    }

    .help-btn.active {
        border-color: var(--colors-accent, #0e639c);
        color: var(--colors-accent, #0e639c);
        background: rgba(14, 99, 156, 0.1);
    }

    .command-btn {
        background: none;
        border: 1px solid var(--colors-border, #333);
        color: var(--colors-text-secondary, #888);
        padding: 6px 10px;
        cursor: pointer;
        font-family: monospace;
        font-size: 11px;
        transition: all 0.2s;
    }

    .command-btn:hover {
        border-color: var(--colors-accent, #0e639c);
        color: var(--colors-text, #fff);
    }

    .actions {
        display: flex;
        gap: 6px;
        align-items: center;
        position: relative;
        overflow: hidden;
        min-width: 0;
    }

    .bar-progress {
        position: absolute;
        top: 0;
        left: 0;
        height: 100%;
        background: var(--colors-surface, #2d2d2d);
        opacity: 0.3;
        pointer-events: none;
        z-index: 0;
    }

    .bar-progress.playing {
        background: var(--colors-accent, #0e639c);
        opacity: 0.35;
    }

    .transport-info {
        font-family: monospace;
        font-size: 11px;
        font-weight: 500;
        color: var(--colors-text, #ddd);
        padding: 4px 6px;
        white-space: nowrap;
        position: relative;
        z-index: 1;
    }

    .tempo-clickable {
        cursor: pointer;
        transition: color 0.2s;
    }

    .tempo-clickable:hover {
        color: var(--colors-text, #fff);
    }

    .tempo-input {
        font-family: monospace;
        font-size: 11px;
        font-weight: 500;
        color: var(--colors-text, #fff);
        background-color: var(--colors-surface, #2d2d2d);
        border: 1px solid var(--colors-accent, #0e639c);
        padding: 4px 6px;
        width: 50px;
        text-align: right;
        position: relative;
        z-index: 1;
    }

    .tempo-input:focus {
        outline: none;
        border-color: var(--colors-accent, #0e639c);
    }

    .tempo-unit {
        font-family: monospace;
        font-size: 11px;
        font-weight: 500;
        color: var(--colors-text-secondary, #888);
        padding: 4px 2px 4px 0;
        position: relative;
        z-index: 1;
    }

    .peer-count {
        display: flex;
        align-items: center;
        gap: 4px;
        font-family: monospace;
        font-size: 11px;
        font-weight: 500;
        color: var(--colors-text-secondary, #888);
        padding: 4px 8px;
        position: relative;
        z-index: 1;
    }

    .nickname-display {
        display: flex;
        align-items: center;
        gap: 4px;
        font-family: monospace;
        font-size: 11px;
        font-weight: 500;
        color: var(--colors-text, #ddd);
        padding: 4px 8px;
        cursor: pointer;
        transition: color 0.2s;
    }

    .nickname-display:hover {
        color: var(--colors-accent, #0e639c);
    }

    .nickname-input {
        font-family: monospace;
        font-size: 11px;
        font-weight: 500;
        color: var(--colors-text, #fff);
        background-color: var(--colors-surface, #2d2d2d);
        border: 1px solid var(--colors-accent, #0e639c);
        padding: 4px 6px;
        width: 100px;
    }

    .nickname-input:focus {
        outline: none;
        border-color: var(--colors-accent, #0e639c);
    }

    .transport-button {
        background: none;
        border: 1px solid var(--colors-border, #333);
        color: var(--colors-text, #fff);
        padding: 6px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        transition: all 0.2s;
        position: relative;
        z-index: 1;
    }

    .transport-button:hover {
        border-color: var(--colors-accent, #0e639c);
        color: var(--colors-accent, #0e639c);
    }

    .play-button {
        border: none;
    }

    .play-button:hover {
        color: var(--colors-accent, #0e639c);
    }

    .disconnect-button {
        background: none;
        border: 1px solid var(--colors-border, #333);
        color: var(--colors-text-secondary, #888);
        padding: 6px 10px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 6px;
        transition: all 0.2s;
        position: relative;
        z-index: 1;
    }

    .disconnect-button:hover {
        border-color: var(--colors-accent, #0e639c);
        color: var(--colors-text, #fff);
    }

    .disconnect-text {
        font-family: monospace;
        font-size: 11px;
        font-weight: 500;
    }

    .save-btn {
        background: none;
        border: none;
        color: var(--colors-text-secondary, #888);
        padding: 4px;
        cursor: pointer;
        display: flex;
        align-items: center;
        transition: color 0.2s;
    }

    .save-btn:hover {
        color: var(--colors-accent, #0e639c);
    }

    .modal-overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
    }

    .modal {
        background: var(--colors-background, #1e1e1e);
        border: 1px solid var(--colors-border, #333);
        padding: 20px;
        min-width: 300px;
    }

    .modal-title {
        font-family: monospace;
        font-size: 14px;
        font-weight: 600;
        color: var(--colors-text, #fff);
        margin-bottom: 16px;
    }

    .modal-message {
        font-family: monospace;
        font-size: 12px;
        color: var(--colors-text-secondary, #888);
        margin-bottom: 16px;
    }

    .modal-input {
        width: 100%;
        box-sizing: border-box;
        font-family: monospace;
        font-size: 13px;
        padding: 8px;
        background: var(--colors-surface, #2d2d2d);
        border: 1px solid var(--colors-border, #333);
        color: var(--colors-text, #fff);
        margin-bottom: 16px;
    }

    .modal-input:focus {
        outline: none;
        border-color: var(--colors-accent, #0e639c);
    }

    .modal-buttons {
        display: flex;
        justify-content: flex-end;
        gap: 8px;
    }

    .modal-button {
        font-family: monospace;
        font-size: 12px;
        padding: 6px 12px;
        background: none;
        border: 1px solid var(--colors-border, #333);
        color: var(--colors-text-secondary, #888);
        cursor: pointer;
    }

    .modal-button:hover {
        border-color: var(--colors-accent, #0e639c);
        color: var(--colors-text, #fff);
    }

    .modal-button.confirm {
        background: var(--colors-accent, #0e639c);
        border-color: var(--colors-accent, #0e639c);
        color: var(--colors-background, #1e1e1e);
    }

    .modal-button.confirm:hover {
        opacity: 0.9;
    }
</style>
