<script lang="ts">
  import { Play, Pause, LogOut, Plus } from 'lucide-svelte';
  import { isConnected } from '$lib/stores/connectionState';
  import { isPlaying, clockState } from '$lib/stores/transport';
  import { startTransport, stopTransport, setTempo } from '$lib/api/client';
  import { invoke } from '@tauri-apps/api/core';
  import { paneLayout } from '$lib/stores/paneState';

  let isEditingTempo = $state(false);
  let tempTempoValue = $state('120');
  let tempoInputElement: HTMLInputElement;

  let barProgress = $derived(
    $clockState !== null
      ? (($clockState.beat % $clockState.quantum) / $clockState.quantum) * 100
      : 0
  );

  function handleAddPane() {
    paneLayout.addPane();
  }

  async function handleDisconnect() {
    try {
      await invoke('disconnect_client');
      isConnected.set(false);
    } catch (error) {
      // Disconnect failed - connection likely already closed
    }
  }

  function startEditingTempo() {
    if ($clockState !== null) {
      tempTempoValue = Math.round($clockState.tempo).toString();
      isEditingTempo = true;
      // Focus input after it's rendered
      setTimeout(() => tempoInputElement?.select(), 0);
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
      console.error('Failed to set tempo:', error);
      cancelEditingTempo();
    }
  }

  function handleTempoKeydown(event: KeyboardEvent) {
    if (event.key === 'Enter') {
      event.preventDefault();
      saveTempoEdit();
    } else if (event.key === 'Escape') {
      event.preventDefault();
      cancelEditingTempo();
    }
  }
</script>

<div class="topbar">
  <div class="left-section">
    <span class="app-name">Sova</span>

    {#if $isConnected}
      <div class="actions">
        <div class="bar-progress" class:playing={$isPlaying} style="width: {barProgress}%"></div>

        <button
          class="transport-button play-button"
          onclick={() => $isPlaying ? stopTransport() : startTransport()}>
          {#if $isPlaying}
            <Pause size={16} />
          {:else}
            <Play size={16} />
          {/if}
        </button>

        <span class="transport-info">
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
            onclick={startEditingTempo}
            onkeydown={(e) => e.key === 'Enter' && startEditingTempo()}
            role="button"
            tabindex="0">
            {$clockState !== null ? `${Math.round($clockState.tempo)} BPM` : '-- BPM'}
          </span>
        {/if}
      </div>
    {/if}
  </div>

  <div class="right-section">
    <button class="add-pane-btn" onclick={handleAddPane} title="Add new pane">
      <Plus size={16} />
    </button>
    {#if $isConnected}
      <button class="disconnect-button" onclick={handleDisconnect} title="Disconnect">
        <LogOut size={16} />
        <span class="disconnect-text">Disconnect</span>
      </button>
    {/if}
  </div>
</div>

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
    padding: 0 8px;
  }

  .add-pane-btn {
    background: none;
    border: 1px solid var(--colors-border, #333);
    color: var(--colors-text-secondary, #888);
    padding: 6px 10px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s;
  }

  .add-pane-btn:hover {
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
</style>
