<script lang="ts">
    import {
        logs,
        filteredLogs,
        logFilters,
        type LogEntry,
    } from "$lib/stores/logs";
    import type { Severity } from "$lib/types/protocol";
    import { formatTimeMs } from "$lib/utils/formatting";
    import SvelteVirtualList from "@humanspeak/svelte-virtual-list";

    let virtualList = $state<any>(null);
    let autoScroll = $state(true);

    function getSeverityClass(level: Severity | undefined): string {
        if (!level) return "severity-info";
        return `severity-${level.toLowerCase()}`;
    }

    function getLogLevel(log: LogEntry): string {
        return log.level ? log.level.toUpperCase() : "INFO";
    }

    function clearLogs() {
        logs.set([]);
    }

    $effect(() => {
        const count = $filteredLogs.length;
        if (autoScroll && virtualList && count > 0) {
            virtualList.scroll({
                index: count - 1,
                align: "auto",
                smoothScroll: false,
            });
        }
    });
</script>

<div class="logs-view">
    <div class="toolbar">
        <h2 class="title">LOGS</h2>
        <div class="toolbar-actions">
            <div class="filter-group" data-help-id="logs-filters">
                <span class="filter-label">Show:</span>
                <label class="filter-toggle">
                    <input type="checkbox" bind:checked={$logFilters.fatal} />
                    Fatal
                </label>
                <label class="filter-toggle">
                    <input type="checkbox" bind:checked={$logFilters.error} />
                    Error
                </label>
                <label class="filter-toggle">
                    <input type="checkbox" bind:checked={$logFilters.warn} />
                    Warn
                </label>
                <label class="filter-toggle">
                    <input type="checkbox" bind:checked={$logFilters.info} />
                    Info
                </label>
                <label class="filter-toggle">
                    <input type="checkbox" bind:checked={$logFilters.debug} />
                    Debug
                </label>
            </div>
            <label class="auto-scroll-toggle" data-help-id="logs-auto-scroll">
                <input type="checkbox" bind:checked={autoScroll} />
                Auto-scroll
            </label>
            <button
                class="clear-button"
                onclick={clearLogs}
                data-help-id="logs-clear"
            >
                Clear
            </button>
        </div>
    </div>

    <div class="logs-content">
        {#if $filteredLogs.length === 0}
            <div class="empty-state">
                {$logs.length === 0
                    ? "No logs yet"
                    : "No logs match current filters"}
            </div>
        {:else}
            <SvelteVirtualList
                bind:this={virtualList}
                items={$filteredLogs}
                defaultEstimatedItemHeight={30}
            >
                {#snippet renderItem(item)}
                    <div class="log-entry {getSeverityClass(item.level)}">
                        <span class="log-level">[{getLogLevel(item)}]</span>
                        <span class="log-timestamp"
                            >{formatTimeMs(item.timestamp)}</span
                        >
                        <span class="log-message">{item.message}</span>
                    </div>
                {/snippet}
            </SvelteVirtualList>
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
        gap: 16px;
    }

    .filter-group {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 4px 12px;
        background-color: var(--colors-background, #1e1e1e);
        border: 1px solid var(--colors-border, #333);
    }

    .filter-label {
        font-size: 12px;
        color: var(--colors-text-secondary, #888);
        font-family: monospace;
        font-weight: 600;
    }

    .filter-toggle {
        display: flex;
        align-items: center;
        gap: 4px;
        font-size: 12px;
        color: var(--colors-text-secondary, #888);
        font-family: monospace;
        cursor: pointer;
    }

    .filter-toggle input[type="checkbox"],
    .auto-scroll-toggle input[type="checkbox"] {
        appearance: none;
        -webkit-appearance: none;
        width: 12px;
        height: 12px;
        border: 1px solid var(--colors-border, #333);
        background-color: var(--colors-background, #1e1e1e);
        cursor: pointer;
        position: relative;
        flex-shrink: 0;
    }

    .filter-toggle input[type="checkbox"]:checked,
    .auto-scroll-toggle input[type="checkbox"]:checked {
        background-color: var(--colors-accent, #0e639c);
        border-color: var(--colors-accent, #0e639c);
    }

    .filter-toggle input[type="checkbox"]:checked::after,
    .auto-scroll-toggle input[type="checkbox"]:checked::after {
        content: "";
        position: absolute;
        left: 3px;
        top: 0px;
        width: 4px;
        height: 8px;
        border: solid var(--colors-text, #fff);
        border-width: 0 2px 2px 0;
        transform: rotate(45deg);
    }

    .filter-toggle input[type="checkbox"]:hover,
    .auto-scroll-toggle input[type="checkbox"]:hover {
        border-color: var(--colors-accent, #0e639c);
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
    }

    .empty-state {
        color: var(--colors-text-secondary, #888);
        font-family: monospace;
        font-size: 13px;
        text-align: center;
        padding: 32px;
    }

    .log-entry {
        display: flex;
        gap: 12px;
        font-family: monospace;
        font-size: 13px;
        padding: 4px 16px;
        border-bottom: 1px solid var(--colors-border, #333);
        box-sizing: border-box;
    }

    .log-entry:hover {
        background-color: var(--colors-surface, #2d2d2d);
    }

    .log-level {
        flex-shrink: 0;
        min-width: 70px;
        font-weight: 600;
        font-size: 11px;
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
        user-select: text;
        -webkit-user-select: text;
    }

    /* Severity color coding */
    .severity-fatal .log-level {
        color: var(--ansi-bright-red, #ff3366);
    }

    .severity-error .log-level {
        color: var(--ansi-red, #ff6b6b);
    }

    .severity-warn .log-level {
        color: var(--ansi-yellow, #ffa500);
    }

    .severity-info .log-level {
        color: var(--ansi-cyan, #4ecdc4);
    }

    .severity-debug .log-level {
        color: var(--ansi-bright-black, #95a5a6);
    }
</style>
