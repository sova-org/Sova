<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { config } from "$lib/stores/config";
    import { serverRunning, serverError, syncServerStatus } from "$lib/stores/serverState";
    import { themes } from "$lib/themes";
    import Toggle from "./ui/Toggle.svelte";
    import Slider from "./ui/Slider.svelte";
    import NumberInput from "./ui/NumberInput.svelte";
    import Select from "./Select.svelte";

    let serverLoading = $state(false);

    async function handleStartServer() {
        serverLoading = true;
        serverError.set(null);
        try {
            await invoke("start_server", { port: $config.server.port });
            await syncServerStatus();
        } catch (e) {
            serverError.set(String(e));
        } finally {
            serverLoading = false;
        }
    }

    async function handleStopServer() {
        serverLoading = true;
        serverError.set(null);
        try {
            await invoke("stop_server");
            await syncServerStatus();
        } catch (e) {
            serverError.set(String(e));
        } finally {
            serverLoading = false;
        }
    }

    function updateConfig<K extends keyof typeof $config>(
        section: K,
        key: keyof (typeof $config)[K],
        value: (typeof $config)[K][typeof key],
    ) {
        config.update((cfg) => ({
            ...cfg,
            [section]: {
                ...cfg[section],
                [key]: value,
            },
        }));
    }

    const themeNames = Object.keys(themes);
</script>

<div class="settings-panel">
    <div class="settings-section server-section">
        <h2 class="section-title">Server</h2>
        <div class="section-content">
            <Toggle
                checked={$config.server.auto_start}
                onchange={(v) => updateConfig("server", "auto_start", v)}
                label="Auto-start on launch"
            />

            <div class="form-row">
                <div class="form-field">
                    <label for="server-ip">IP Address</label>
                    <input
                        id="server-ip"
                        type="text"
                        value={$config.server.ip}
                        oninput={(e) =>
                            updateConfig(
                                "server",
                                "ip",
                                (e.target as HTMLInputElement).value,
                            )}
                    />
                </div>
                <div class="form-field port-field">
                    <NumberInput
                        value={$config.server.port}
                        min={1024}
                        max={65535}
                        onchange={(v) => updateConfig("server", "port", v)}
                        label="Port"
                    />
                </div>
            </div>

            <div class="server-controls">
                <div class="server-status">
                    <span class="status-dot" class:running={$serverRunning}
                    ></span>
                    <span class="status-text"
                        >{$serverRunning ? "Running" : "Stopped"}</span
                    >
                </div>
                <button
                    class="server-button"
                    class:stop={$serverRunning}
                    onclick={$serverRunning ? handleStopServer : handleStartServer}
                    disabled={serverLoading}
                >
                    {#if serverLoading}
                        ...
                    {:else if $serverRunning}
                        Stop Server
                    {:else}
                        Start Server
                    {/if}
                </button>
            </div>

            {#if $serverError}
                <div class="error-message">{$serverError}</div>
            {/if}
        </div>
    </div>

    <div class="settings-section">
        <h2 class="section-title">Appearance</h2>
        <div class="section-content">
            <div class="form-field">
                <span class="field-label">Theme</span>
                <Select
                    options={themeNames}
                    value={$config.appearance.theme}
                    onchange={(v) => updateConfig("appearance", "theme", v)}
                />
            </div>

            <Slider
                value={$config.appearance.zoom}
                min={0.5}
                max={2}
                step={0.1}
                onchange={(v) => updateConfig("appearance", "zoom", v)}
                label="Zoom"
            />

            <Slider
                value={$config.appearance.hue}
                min={0}
                max={360}
                step={1}
                onchange={(v) => updateConfig("appearance", "hue", v)}
                label="Hue Rotation"
            />
        </div>
    </div>

    <div class="settings-section">
        <h2 class="section-title">Editor</h2>
        <div class="section-content">
            <div class="form-field">
                <span class="field-label">Mode</span>
                <Select
                    options={["normal", "vim", "emacs"]}
                    value={$config.editor.mode}
                    onchange={(v) => updateConfig("editor", "mode", v as "vim" | "normal" | "emacs")}
                />
            </div>

            <Slider
                value={$config.editor.font_size}
                min={8}
                max={32}
                step={1}
                onchange={(v) => updateConfig("editor", "font_size", v)}
                label="Font Size"
            />

            <Slider
                value={$config.editor.tab_size}
                min={1}
                max={8}
                step={1}
                onchange={(v) => updateConfig("editor", "tab_size", v)}
                label="Tab Size"
            />

            <div class="toggle-grid">
                <Toggle
                    checked={$config.editor.show_line_numbers}
                    onchange={(v) =>
                        updateConfig("editor", "show_line_numbers", v)}
                    label="Line numbers"
                />
                <Toggle
                    checked={$config.editor.line_wrapping}
                    onchange={(v) => updateConfig("editor", "line_wrapping", v)}
                    label="Line wrapping"
                />
                <Toggle
                    checked={$config.editor.highlight_active_line}
                    onchange={(v) =>
                        updateConfig("editor", "highlight_active_line", v)}
                    label="Highlight active line"
                />
                <Toggle
                    checked={$config.editor.bracket_matching}
                    onchange={(v) =>
                        updateConfig("editor", "bracket_matching", v)}
                    label="Bracket matching"
                />
                <Toggle
                    checked={$config.editor.autocomplete}
                    onchange={(v) => updateConfig("editor", "autocomplete", v)}
                    label="Autocomplete"
                />
                <Toggle
                    checked={$config.editor.close_brackets}
                    onchange={(v) =>
                        updateConfig("editor", "close_brackets", v)}
                    label="Auto-close brackets"
                />
                <Toggle
                    checked={$config.editor.fold_gutter}
                    onchange={(v) => updateConfig("editor", "fold_gutter", v)}
                    label="Code folding"
                />
                <Toggle
                    checked={$config.editor.match_highlighting}
                    onchange={(v) =>
                        updateConfig("editor", "match_highlighting", v)}
                    label="Match highlighting"
                />
            </div>
        </div>
    </div>
</div>

<style>
    .settings-panel {
        width: 100%;
        height: 100%;
        overflow-y: auto;
        padding: 16px;
        box-sizing: border-box;
        background-color: var(--colors-background, #1e1e1e);
    }

    .settings-section {
        margin-bottom: 24px;
        background-color: var(--colors-surface, #252525);
        border: 1px solid var(--colors-border, #333);
        padding: 16px;
    }

    .server-section {
        border-color: var(--colors-accent, #0e639c);
    }

    .section-title {
        margin: 0 0 16px 0;
        font-size: 14px;
        font-weight: 600;
        color: var(--colors-text, #fff);
        font-family: monospace;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .section-content {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .form-row {
        display: flex;
        gap: 16px;
    }

    .form-field {
        display: flex;
        flex-direction: column;
        gap: 4px;
        flex: 1;
    }

    .port-field {
        max-width: 120px;
    }

    .form-field label,
    .field-label {
        font-size: 13px;
        font-family: monospace;
        color: var(--colors-text, #fff);
    }

    input {
        background-color: var(--colors-background, #1e1e1e);
        color: var(--colors-text, #fff);
        border: 1px solid var(--colors-border, #333);
        padding: 8px 10px;
        font-size: 13px;
        font-family: monospace;
        width: 100%;
        box-sizing: border-box;
    }

    input:focus {
        outline: none;
        border-color: var(--colors-accent, #0e639c);
    }

    .server-controls {
        display: flex;
        align-items: center;
        justify-content: space-between;
        padding-top: 8px;
    }

    .server-status {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .status-dot {
        width: 8px;
        height: 8px;
        background-color: var(--colors-text-secondary, #888);
    }

    .status-dot.running {
        background-color: #4caf50;
    }

    .status-text {
        font-size: 13px;
        font-family: monospace;
        color: var(--colors-text-secondary, #888);
    }

    .server-button {
        background-color: var(--colors-accent, #0e639c);
        color: var(--colors-text, #fff);
        border: none;
        padding: 8px 16px;
        font-size: 13px;
        font-family: monospace;
        cursor: pointer;
    }

    .server-button:hover:not(:disabled) {
        background-color: var(--colors-accent-hover, #1177bb);
    }

    .server-button:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .server-button.stop {
        background-color: var(--colors-danger, #c53030);
    }

    .server-button.stop:hover:not(:disabled) {
        background-color: var(--colors-danger-hover, #e53e3e);
    }

    .error-message {
        background-color: rgba(197, 48, 48, 0.2);
        color: #f48771;
        padding: 8px;
        font-size: 12px;
        font-family: monospace;
        border: 1px solid rgba(197, 48, 48, 0.5);
    }

    .toggle-grid {
        display: grid;
        grid-template-columns: repeat(2, 1fr);
        gap: 12px;
    }
</style>
