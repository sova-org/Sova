<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ThemeProvider from '$lib/components/ThemeProvider.svelte';
  import TopBar from '$lib/components/TopBar.svelte';
  import PaneLayout from '$lib/components/panes/PaneLayout.svelte';
  import HelpMode from '$lib/components/HelpMode.svelte';
  import { initializeApp, cleanupApp, initializeSovaStores, cleanupSovaStores } from '$lib/stores';
  import { isConnected } from '$lib/stores/connectionState';

  onMount(async () => {
    await initializeApp();

    const connected = await invoke<boolean>('is_client_connected');
    if (connected) {
      await initializeSovaStores();
      isConnected.set(true);
    }
  });

  onDestroy(() => {
    cleanupApp();
    cleanupSovaStores();
  });
</script>

<ThemeProvider>
  <div class="app">
    <TopBar />
    <div class="content">
      <PaneLayout />
    </div>
  </div>
</ThemeProvider>

<HelpMode />

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background-color: transparent;
  }

  :global(*::-webkit-scrollbar) {
    display: none;
  }

  :global(*) {
    -ms-overflow-style: none;
    scrollbar-width: none;
    user-select: none;
    -webkit-user-select: none;
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
