<script lang="ts">
  import { viewState, type ViewType } from '$lib/stores/viewState';
  import { isConnected } from '$lib/stores/connectionState';
  import { invoke } from '@tauri-apps/api/core';

  interface Props {
    currentView: ViewType;
  }

  let { currentView }: Props = $props();

  function switchView(view: ViewType) {
    viewState.set(view);
  }

  async function handleDisconnect() {
    try {
      await invoke('disconnect_client');
      isConnected.set(false);
      viewState.set('LOGIN');
    } catch (error) {
      // Disconnect failed - connection likely already closed
    }
  }
</script>

<div class="topbar">
  <div class="tabs">
    {#if $isConnected}
      <button
        class="tab"
        class:active={currentView === 'SCENE'}
        onclick={() => switchView('SCENE')}>
        SCENE
      </button>
      <button
        class="tab"
        class:active={currentView === 'DEVICES'}
        onclick={() => switchView('DEVICES')}>
        DEVICES
      </button>
    {:else}
      <button
        class="tab"
        class:active={currentView === 'LOGIN'}
        onclick={() => switchView('LOGIN')}>
        LOGIN
      </button>
    {/if}
    <button
      class="tab"
      class:active={currentView === 'LOGS'}
      onclick={() => switchView('LOGS')}>
      LOGS
    </button>
    <button
      class="tab"
      class:active={currentView === 'CONFIG'}
      onclick={() => switchView('CONFIG')}>
      CONFIG
    </button>
  </div>

  {#if $isConnected}
    <div class="actions">
      <button class="disconnect-button" onclick={handleDisconnect}>
        Disconnect
      </button>
    </div>
  {/if}
</div>

<style>
  .topbar {
    width: 100%;
    height: 40px;
    background-color: var(--colors-background, #1e1e1e);
    border-bottom: 1px solid var(--colors-border, #333);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
  }

  .tabs {
    display: flex;
    gap: 4px;
  }

  .tab {
    background: none;
    border: none;
    color: var(--colors-text-secondary, #888);
    font-family: monospace;
    font-size: 14px;
    font-weight: 600;
    letter-spacing: 0.5px;
    padding: 8px 16px;
    cursor: pointer;
    transition: color 0.2s;
  }

  .tab:hover {
    color: var(--colors-text, #aaa);
  }

  .tab.active {
    color: var(--colors-text, #fff);
    background-color: var(--colors-surface, #2d2d2d);
  }

  .actions {
    display: flex;
    gap: 8px;
  }

  .disconnect-button {
    background: none;
    border: 1px solid var(--colors-border, #333);
    color: var(--colors-text-secondary, #888);
    font-family: monospace;
    font-size: 12px;
    padding: 4px 12px;
    cursor: pointer;
  }

  .disconnect-button:hover {
    border-color: var(--colors-accent, #0e639c);
    color: var(--colors-text, #fff);
  }
</style>
