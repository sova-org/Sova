<script lang="ts">
    import { onMount } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { isConnected, connectionError } from "$lib/stores/connectionState";
    import { initializeSovaStores } from "$lib/stores";
    import { nickname as nicknameStore } from "$lib/stores/nickname";

    const STORAGE_KEY = "sova-login-fields";

    interface LoginFields {
        ip: string;
        port: number;
        nickname: string;
    }

    function loadLoginFields(): LoginFields {
        try {
            const stored = localStorage.getItem(STORAGE_KEY);
            if (stored) return JSON.parse(stored);
        } catch {
            // Invalid stored state
        }
        return { ip: "127.0.0.1", port: 8080, nickname: "" };
    }

    function saveLoginFields(fields: LoginFields): void {
        try {
            localStorage.setItem(STORAGE_KEY, JSON.stringify(fields));
        } catch {
            // Storage unavailable
        }
    }

    interface Props {
        onConnected?: () => void;
    }

    let { onConnected }: Props = $props();

    let ip = $state("");
    let port = $state(8080);
    let nicknameValue = $state("");
    let connecting = $state(false);
    let errorMsg = $state("");

    onMount(() => {
        const fields = loadLoginFields();
        ip = fields.ip;
        port = fields.port;
        nicknameValue = fields.nickname;
        connectionError.set(null);
    });

    async function handleConnect(event?: Event) {
        event?.preventDefault();

        if (!ip || !port || !nicknameValue) {
            errorMsg = "All fields are required";
            return;
        }

        connecting = true;
        errorMsg = "";

        try {
            await invoke("connect_client", { ip, port, username: nicknameValue });
            saveLoginFields({ ip, port, nickname: nicknameValue });
            nicknameStore.set(nicknameValue);

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
                    bind:value={nicknameValue}
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
