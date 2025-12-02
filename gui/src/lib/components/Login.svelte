<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { isConnected, connectionError } from "$lib/stores/connectionState";
    import {
        clientConfig,
        runtimeNickname,
        setRuntimeNickname,
    } from "$lib/stores/config";
    import { initializeSovaStores } from "$lib/stores";

    interface Props {
        onConnected?: () => void;
    }

    let { onConnected }: Props = $props();

    let ip = $state("");
    let port = $state(8080);
    let nickname = $state("");
    let connecting = $state(false);
    let errorMsg = $state("");

    // Sync form fields with config (reactive - works on load AND when config is saved)
    // Nickname uses runtimeNickname (instance-specific), falling back to config default
    $effect(() => {
        const config = $clientConfig;
        const rn = $runtimeNickname;
        if (config) {
            ip = config.ip;
            port = config.port;
            // Prefer runtime nickname if set, otherwise use config default
            nickname = rn || config.nickname;
        }
    });

    // Clear connection error on mount
    $effect(() => {
        connectionError.set(null);
    });

    async function handleConnect(event?: Event) {
        event?.preventDefault();

        if (!ip || !port || !nickname) {
            errorMsg = "All fields are required";
            return;
        }

        connecting = true;
        errorMsg = "";

        try {
            await invoke("connect_client", { ip, port, username: nickname });
            await invoke("save_client_config", { ip, port, nickname });

            // Set runtime nickname for this instance
            setRuntimeNickname(nickname);

            // Initialize Sova stores to listen for server messages
            await initializeSovaStores();

            isConnected.set(true);
            connectionError.set(null);
            onConnected?.();
        } catch (error) {
            errorMsg = String(error);
            isConnected.set(false);
        } finally {
            connecting = false;
        }
    }

    function handleKeyPress(event: KeyboardEvent) {
        if (event.key === "Enter") {
            handleConnect();
        }
    }
</script>

<div class="login-container">
    <div class="login-box">
        <h1 class="login-title">Connect to Sova Server</h1>

        {#if errorMsg}
            <div class="error-message">
                {errorMsg}
            </div>
        {/if}

        <form class="login-form" onsubmit={handleConnect}>
            <div class="form-group" data-help-id="login-ip">
                <label for="ip">Server IP</label>
                <input
                    type="text"
                    id="ip"
                    bind:value={ip}
                    placeholder="127.0.0.1"
                    disabled={connecting}
                    onkeypress={handleKeyPress}
                />
            </div>

            <div class="form-group" data-help-id="login-port">
                <label for="port">Server Port</label>
                <input
                    type="number"
                    id="port"
                    bind:value={port}
                    placeholder="8080"
                    min="1"
                    max="65535"
                    disabled={connecting}
                    onkeypress={handleKeyPress}
                />
            </div>

            <div class="form-group" data-help-id="login-nickname">
                <label for="nickname">Nickname</label>
                <input
                    type="text"
                    id="nickname"
                    bind:value={nickname}
                    placeholder="Your nickname"
                    disabled={connecting}
                    onkeypress={handleKeyPress}
                />
            </div>

            <button
                type="submit"
                class="connect-button"
                data-help-id="login-connect"
                disabled={connecting}
            >
                {#if connecting}
                    Connecting...
                {:else}
                    Connect
                {/if}
            </button>
        </form>
    </div>
</div>

<style>
    .login-container {
        width: 100%;
        height: 100%;
        display: flex;
        align-items: center;
        justify-content: center;
        background-color: var(--colors-background, #1e1e1e);
    }

    .login-box {
        width: 400px;
        padding: 32px;
        background-color: var(--colors-surface, #252525);
        border: 1px solid var(--colors-border, #333);
    }

    .login-title {
        margin: 0 0 24px 0;
        font-size: 20px;
        font-weight: 500;
        color: var(--colors-text, #fff);
        font-family: monospace;
    }

    .error-message {
        background-color: var(--colors-danger, #5a1d1d);
        color: var(--colors-text, #f48771);
        padding: 12px;
        margin-bottom: 16px;
        font-size: 13px;
        font-family: monospace;
        border: 1px solid var(--colors-border, #721c24);
    }

    .login-form {
        display: flex;
        flex-direction: column;
        gap: 16px;
    }

    .form-group {
        display: flex;
        flex-direction: column;
        gap: 8px;
    }

    label {
        font-size: 13px;
        color: var(--colors-text, #fff);
        font-family: monospace;
    }

    input {
        background-color: var(--colors-background, #1e1e1e);
        color: var(--colors-text, #fff);
        border: 1px solid var(--colors-border, #333);
        padding: 10px 12px;
        font-size: 14px;
        font-family: monospace;
    }

    input:focus {
        outline: none;
        border-color: var(--colors-accent, #0e639c);
    }

    input:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .connect-button {
        background-color: var(--colors-accent, #0e639c);
        color: var(--colors-text, #fff);
        border: none;
        padding: 12px;
        font-size: 14px;
        font-weight: 500;
        cursor: pointer;
        font-family: monospace;
        margin-top: 8px;
    }

    .connect-button:hover:not(:disabled) {
        background-color: var(--colors-accent-hover, #1177bb);
    }

    .connect-button:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }
</style>
