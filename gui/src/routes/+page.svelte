<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import ThemeProvider from '$lib/components/ThemeProvider.svelte';
  import TopBar from '$lib/components/TopBar.svelte';
  import Editor from '$lib/components/Editor.svelte';
  import ConfigEditor from '$lib/components/ConfigEditor.svelte';
  import Login from '$lib/components/Login.svelte';
  import DevicesView from '$lib/components/DevicesView.svelte';
  import { viewState } from '$lib/stores/viewState';
  import { initializeApp, cleanupApp } from '$lib/stores/config';
  import { initializeSovaStores, cleanupSovaStores } from '$lib/stores';
  import { isConnected } from '$lib/stores/connectionState';

  let currentView = $state($viewState);

  $effect(() => {
    currentView = $viewState;
  });

  onMount(async () => {
    await initializeApp();

    const connected = await invoke<boolean>('is_client_connected');
    if (!connected) {
      viewState.set('LOGIN');
    } else {
      // Already connected - initialize Sova stores
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
    <TopBar {currentView} />
    <div class="content">
      {#if currentView === 'LOGIN'}
        <Login />
      {:else if currentView === 'EDITOR'}
        <Editor />
      {:else if currentView === 'DEVICES'}
        <DevicesView />
      {:else}
        <ConfigEditor />
      {/if}
    </div>
  </div>
</ThemeProvider>

<style>
  :global(html),
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background-color: transparent;
  }

  .app {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    background-color: var(--colors-background);
    color: var(--colors-text, #ffffff);
  }

  .content {
    flex: 1;
    overflow: hidden;
  }
</style>
