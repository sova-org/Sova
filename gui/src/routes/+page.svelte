<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { confirm } from "@tauri-apps/plugin-dialog";
    import ThemeProvider from "$lib/components/ThemeProvider.svelte";
    import TopBar from "$lib/components/TopBar.svelte";
    import BottomBar from "$lib/components/BottomBar.svelte";
    import ViewContainer from "$lib/components/ViewContainer.svelte";
    import HelpMode from "$lib/components/HelpMode.svelte";
    import CommandPalette from "$lib/components/CommandPalette.svelte";
    import Sidebar from "$lib/components/Sidebar.svelte";
    import {
        initializeApp,
        cleanupApp,
        initializeSovaStores,
        cleanupSovaStores,
        syncServerStatus,
    } from "$lib/stores";
    import { isConnected } from "$lib/stores/connectionState";
    import { config } from "$lib/stores/config";
    import { get } from "svelte/store";
    import type { UnlistenFn } from "@tauri-apps/api/event";
    import "$lib/commands";

    let unlistenCloseRequest: UnlistenFn | null = null;

    onMount(async () => {
        await initializeApp();

        const connected = await invoke<boolean>("is_client_connected");
        if (connected) {
            await initializeSovaStores();
            isConnected.set(true);
            // Request current audio engine state (HELLO was already sent before page load)
            await invoke("send_client_message", { message: "GetAudioEngineState" });
        }

        const cfg = get(config);
        if (cfg.server.auto_start) {
            try {
                const alreadyRunning = await invoke<boolean>("is_server_running");
                if (!alreadyRunning) {
                    await invoke("start_server", { port: cfg.server.port });
                    await syncServerStatus();
                }
            } catch (e) {
                console.error("[sova] Failed to auto-start server:", e);
            }
        }

        unlistenCloseRequest = await getCurrentWindow().onCloseRequested(
            async (event) => {
                const serverRunning =
                    await invoke<boolean>("is_server_running");
                const clientConnected = await invoke<boolean>(
                    "is_client_connected",
                );

                if (serverRunning || clientConnected) {
                    const confirmed = await confirm(
                        "Are you sure you want to quit?",
                        {
                            title: "Quit Sova",
                            kind: "warning",
                        },
                    );
                    if (!confirmed) {
                        event.preventDefault();
                    }
                }
            },
        );
    });

    onDestroy(() => {
        if (unlistenCloseRequest) {
            unlistenCloseRequest();
        }
        cleanupApp();
        cleanupSovaStores();
    });
</script>

<ThemeProvider>
    <div class="app">
        <TopBar />
        <div class="content">
            <ViewContainer />
        </div>
        <BottomBar />
    </div>
    <HelpMode />
    <CommandPalette />
    <Sidebar />
</ThemeProvider>

<style>
    :global(html),
    :global(body) {
        margin: 0;
        padding: 0;
        overflow: hidden;
        background-color: var(--colors-background);
    }

    :global(*::-webkit-scrollbar) {
        display: none;
    }

    :global(*) {
        -ms-overflow-style: none;
        scrollbar-width: none;
    }

    :global(input),
    :global(textarea),
    :global([contenteditable="true"]) {
        user-select: text;
        -webkit-user-select: text;
        cursor: text;
    }

    .app {
        width: 100vw;
        height: 100vh;
        display: flex;
        flex-direction: column;
        background-color: var(--colors-background);
        color: var(--colors-text);
        font-family: var(--appearance-font-family);
    }

    .content {
        flex: 1;
        overflow: hidden;
    }

    :global(body.help-mode-active [data-help-id]) {
        outline: 1px dashed var(--colors-text-secondary, #666);
        outline-offset: 2px;
    }

    :global(body.help-mode-active [data-help-id]:hover) {
        outline: 2px solid var(--colors-accent, #0e639c);
        background-color: rgba(14, 99, 156, 0.1);
    }
</style>
