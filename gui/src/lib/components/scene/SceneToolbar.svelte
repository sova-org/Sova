<script lang="ts">
	import { ArrowLeftRight, ArrowUpDown, ZoomIn, ZoomOut, RotateCcw } from 'lucide-svelte';
	import { selection } from '$lib/stores/selection';

	interface Props {
		zoom: number;
		minZoom: number;
		maxZoom: number;
		zoomFactor: number;
		orientation: 'horizontal' | 'vertical';
		splitOrientation: 'horizontal' | 'vertical';
		onZoomChange: (zoom: number) => void;
		onOrientationChange: () => void;
		onSplitOrientationChange: () => void;
	}

	let {
		zoom,
		minZoom,
		maxZoom,
		zoomFactor,
		orientation,
		splitOrientation,
		onZoomChange,
		onOrientationChange,
		onSplitOrientationChange
	}: Props = $props();

	function zoomIn() {
		onZoomChange(Math.min(maxZoom, zoom * zoomFactor));
	}

	function zoomOut() {
		onZoomChange(Math.max(minZoom, zoom / zoomFactor));
	}

	function resetZoom() {
		onZoomChange(1.0);
	}
</script>

<div class="toolbar">
	<div class="toolbar-left">
		<span class="title">SCENE</span>
		{#if $selection}
			<span class="selection-info">L{$selection.lineId} F{$selection.frameId}</span>
		{/if}
	</div>
	<div class="toolbar-right">
		<div class="zoom-controls">
			<button
				class="toolbar-btn"
				onclick={zoomOut}
				title="Zoom out"
				disabled={zoom <= minZoom}
			>
				<ZoomOut size={14} />
			</button>
			<span class="zoom-level">{Math.round(zoom * 100)}%</span>
			<button
				class="toolbar-btn"
				onclick={zoomIn}
				title="Zoom in"
				disabled={zoom >= maxZoom}
			>
				<ZoomIn size={14} />
			</button>
			{#if zoom !== 1.0}
				<button
					class="toolbar-btn"
					onclick={resetZoom}
					title="Reset zoom"
				>
					<RotateCcw size={12} />
				</button>
			{/if}
		</div>
		<button
			class="toolbar-btn"
			onclick={onOrientationChange}
			title="Toggle timeline orientation"
		>
			{#if orientation === 'horizontal'}
				<ArrowLeftRight size={14} />
			{:else}
				<ArrowUpDown size={14} />
			{/if}
		</button>
		<button
			class="toolbar-btn"
			onclick={onSplitOrientationChange}
			title="Toggle split orientation"
		>
			{#if splitOrientation === 'horizontal'}
				<ArrowUpDown size={14} />
			{:else}
				<ArrowLeftRight size={14} />
			{/if}
		</button>
	</div>
</div>

<style>
	.toolbar {
		height: 36px;
		background-color: var(--colors-surface);
		border-bottom: 1px solid var(--colors-border);
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 12px;
		flex-shrink: 0;
	}

	.toolbar-left {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.title {
		color: var(--colors-text);
		font-size: 11px;
		font-weight: 700;
		letter-spacing: 0.5px;
	}

	.selection-info {
		color: var(--colors-text-secondary);
		font-size: 10px;
	}

	.toolbar-right {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.zoom-controls {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.zoom-level {
		font-size: 10px;
		color: var(--colors-text-secondary);
		min-width: 36px;
		text-align: center;
	}

	.toolbar-btn {
		background: none;
		border: 1px solid var(--colors-border);
		color: var(--colors-text-secondary);
		padding: 4px;
		cursor: pointer;
		display: flex;
		align-items: center;
	}

	.toolbar-btn:hover:not(:disabled) {
		border-color: var(--colors-accent);
		color: var(--colors-accent);
	}

	.toolbar-btn:disabled {
		opacity: 0.3;
		cursor: not-allowed;
	}
</style>
