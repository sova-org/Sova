<script lang="ts">
  import { logs } from '$lib/stores/logs';
  import { onMount, onDestroy } from 'svelte';

  let scrollContainer: HTMLDivElement;
  let autoScroll = $state(true);

  function formatTimestamp(timestamp: number): string {
    const date = new Date(timestamp);
    const hours = String(date.getHours()).padStart(2, '0');
    const minutes = String(date.getMinutes()).padStart(2, '0');
    const seconds = String(date.getSeconds()).padStart(2, '0');
    const ms = String(date.getMilliseconds()).padStart(3, '0');
    return `${hours}:${minutes}:${seconds}.${ms}`;
  }

  function clearLogs() {
    logs.set([]);
  }

  function scrollToBottom() {
    if (autoScroll && scrollContainer) {
      scrollContainer.scrollTop = scrollContainer.scrollHeight;
    }
  }

  $effect(() => {
    $logs;
    scrollToBottom();
  });
</script>

<div class="logs-view">
  <div class="toolbar">
    <h2 class="title">LOGS</h2>
    <div class="toolbar-actions">
      <label class="auto-scroll-toggle">
        <input type="checkbox" bind:checked={autoScroll} />
        Auto-scroll
      </label>
      <button class="clear-button" onclick={clearLogs}>
        Clear
      </button>
    </div>
  </div>

  <div class="logs-content" bind:this={scrollContainer}>
    {#if $logs.length === 0}
      <div class="empty-state">No logs yet</div>
    {:else}
      <div class="logs-list">
        {#each $logs as log}
          <div class="log-entry">
            <span class="log-timestamp">{formatTimestamp(log.timestamp)}</span>
            <span class="log-message">{log.message}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>

<style>
  .logs-view {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--colors-background);
  }

  .toolbar {
    height: 40px;
    background-color: var(--colors-surface, #252525);
    border-bottom: 1px solid var(--colors-border, #333);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 16px;
  }

  .title {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
    color: var(--colors-text, #fff);
    font-family: monospace;
  }

  .toolbar-actions {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .auto-scroll-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 13px;
    color: var(--colors-text-secondary, #888);
    font-family: monospace;
    cursor: pointer;
  }

  .auto-scroll-toggle input[type="checkbox"] {
    cursor: pointer;
  }

  .clear-button {
    background-color: var(--colors-accent, #0e639c);
    color: var(--colors-text, #fff);
    border: none;
    padding: 4px 12px;
    font-size: 13px;
    cursor: pointer;
    font-family: monospace;
  }

  .clear-button:hover {
    background-color: var(--colors-accent-hover, #1177bb);
  }

  .logs-content {
    flex: 1;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 16px;
  }

  .empty-state {
    color: var(--colors-text-secondary, #888);
    font-family: monospace;
    font-size: 13px;
    text-align: center;
    padding: 32px;
  }

  .logs-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .log-entry {
    display: flex;
    gap: 12px;
    font-family: monospace;
    font-size: 13px;
    padding: 4px 8px;
    border-bottom: 1px solid var(--colors-border, #333);
  }

  .log-entry:hover {
    background-color: var(--colors-surface, #2d2d2d);
  }

  .log-timestamp {
    color: var(--colors-text-secondary, #888);
    flex-shrink: 0;
    min-width: 90px;
  }

  .log-message {
    color: var(--colors-text, #fff);
    flex: 1;
    word-break: break-word;
  }
</style>
