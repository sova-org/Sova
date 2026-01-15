<script lang="ts">
	import {
		logs,
		filteredLogs,
		logFilters,
		activeLogTab,
		type LogEntry,
		type LogTab,
	} from '$lib/stores/logs';
	import type { Severity } from '$lib/types/protocol';
	import { formatTimeMs } from '$lib/utils/formatting';
	import { Trash2 } from 'lucide-svelte';

	const ITEM_HEIGHT = 28;
	const OVERSCAN = 10;

	let container = $state<HTMLDivElement | null>(null);
	let scrollTop = $state(0);
	let containerHeight = $state(400);
	let stickToBottom = $state(true);

	// Calculate visible range
	let totalHeight = $derived($filteredLogs.length * ITEM_HEIGHT);
	let startIndex = $derived(
		Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - OVERSCAN)
	);
	let endIndex = $derived(
		Math.min(
			$filteredLogs.length,
			Math.ceil((scrollTop + containerHeight) / ITEM_HEIGHT) + OVERSCAN
		)
	);
	let visibleLogs = $derived($filteredLogs.slice(startIndex, endIndex));
	let offsetY = $derived(startIndex * ITEM_HEIGHT);

	// Auto-scroll to bottom when new logs arrive
	$effect(() => {
		const count = $filteredLogs.length;
		if (stickToBottom && count > 0 && container) {
			container.scrollTop = count * ITEM_HEIGHT;
		}
	});

	function handleScroll(e: Event) {
		const target = e.target as HTMLDivElement;
		scrollTop = target.scrollTop;
		containerHeight = target.clientHeight;

		// Check if at bottom
		const distanceFromBottom =
			target.scrollHeight - target.scrollTop - target.clientHeight;
		stickToBottom = distanceFromBottom < ITEM_HEIGHT * 2;
	}

	function getSeverityClass(level: Severity | undefined): string {
		if (!level) return 'severity-info';
		return `severity-${level.toLowerCase()}`;
	}

	function getLogLevel(log: LogEntry): string {
		return log.level ? log.level.toUpperCase() : 'INFO';
	}

	function clearLogs() {
		logs.set([]);
	}

	function setTab(tab: LogTab) {
		activeLogTab.set(tab);
	}
</script>

<div class="logs-view">
	<div class="toolbar">
		<div class="log-tabs">
			<button
				class="tab-button"
				class:active={$activeLogTab === 'output'}
				onclick={() => setTab('output')}
			>
				Client
			</button>
			<button
				class="tab-button"
				class:active={$activeLogTab === 'server'}
				onclick={() => setTab('server')}
			>
				Server
			</button>
			<button
				class="tab-button"
				class:active={$activeLogTab === 'all'}
				onclick={() => setTab('all')}
			>
				All
			</button>
		</div>
		<div class="filter-group" data-help-id="logs-filters">
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
		<div class="toolbar-actions">
			<label class="auto-scroll-toggle" data-help-id="logs-auto-scroll">
				<input type="checkbox" bind:checked={stickToBottom} />
				Auto
			</label>
			<button
				class="clear-button"
				onclick={clearLogs}
				data-help-id="logs-clear"
				title="Clear logs"
			>
				<Trash2 size={14} />
			</button>
		</div>
	</div>

	<div class="logs-content" bind:this={container} onscroll={handleScroll}>
		{#if $filteredLogs.length === 0}
			<div class="empty-state">
				{$logs.length === 0 ? 'No logs yet' : 'No logs match current filters'}
			</div>
		{:else}
			<div class="virtual-container" style="height: {totalHeight}px;">
				<div class="virtual-window" style="transform: translateY({offsetY}px);">
					{#each visibleLogs as log, i (startIndex + i)}
						<div class="log-entry {getSeverityClass(log.level)}">
							<span class="log-level">[{getLogLevel(log)}]</span>
							<span class="log-timestamp">{formatTimeMs(log.timestamp)}</span>
							<span class="log-message" title={log.message}>{log.message}</span>
						</div>
					{/each}
				</div>
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
		flex-shrink: 0;
		background-color: var(--colors-surface, #252525);
		border-bottom: 1px solid var(--colors-border, #333);
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 16px;
		gap: 16px;
	}

	.log-tabs {
		display: flex;
		gap: 2px;
	}

	.tab-button {
		background: none;
		border: none;
		color: var(--colors-text-secondary, #888);
		font-family: monospace;
		font-size: 12px;
		padding: 4px 10px;
		cursor: pointer;
	}

	.tab-button:hover {
		color: var(--colors-text, #fff);
	}

	.tab-button.active {
		color: var(--colors-text, #fff);
		background-color: var(--colors-background, #1e1e1e);
	}

	.toolbar-actions {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.filter-group {
		display: flex;
		align-items: center;
		gap: 12px;
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

	.filter-toggle input[type='checkbox'],
	.auto-scroll-toggle input[type='checkbox'] {
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

	.filter-toggle input[type='checkbox']:checked,
	.auto-scroll-toggle input[type='checkbox']:checked {
		background-color: var(--colors-accent, #0e639c);
		border-color: var(--colors-accent, #0e639c);
	}

	.filter-toggle input[type='checkbox']:checked::after,
	.auto-scroll-toggle input[type='checkbox']:checked::after {
		content: '';
		position: absolute;
		left: 3px;
		top: 0px;
		width: 4px;
		height: 8px;
		border: solid var(--colors-text, #fff);
		border-width: 0 2px 2px 0;
		transform: rotate(45deg);
	}

	.filter-toggle input[type='checkbox']:hover,
	.auto-scroll-toggle input[type='checkbox']:hover {
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
		display: flex;
		align-items: center;
		justify-content: center;
		background: none;
		color: var(--colors-text-secondary, #888);
		border: none;
		padding: 4px;
		cursor: pointer;
	}

	.clear-button:hover {
		color: var(--colors-text, #fff);
	}

	.logs-content {
		flex: 1;
		overflow-y: auto;
		overflow-x: hidden;
	}

	.virtual-container {
		position: relative;
		width: 100%;
	}

	.virtual-window {
		position: absolute;
		top: 0;
		left: 0;
		width: 100%;
	}

	.empty-state {
		color: var(--colors-text-secondary, #888);
		font-family: monospace;
		font-size: 13px;
		text-align: center;
		padding: 32px;
	}

	.log-entry {
		height: 28px;
		display: flex;
		gap: 12px;
		align-items: center;
		font-family: monospace;
		font-size: 13px;
		padding: 0 16px;
		box-sizing: border-box;
	}

	.log-entry:hover {
		background-color: var(--colors-surface, #2d2d2d);
	}

	.log-level {
		flex-shrink: 0;
		width: 70px;
		font-weight: 600;
		font-size: 11px;
	}

	.log-timestamp {
		color: var(--colors-text-secondary, #888);
		flex-shrink: 0;
		width: 90px;
	}

	.log-message {
		color: var(--colors-text, #fff);
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
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
