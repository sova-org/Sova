<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { listen, type UnlistenFn } from '@tauri-apps/api/event';
	import { scene } from '$lib/stores';
	import { SERVER_EVENTS } from '$lib/events';
	import type { Frame, RemoveFramePayload } from '$lib/types/protocol';
	import type { Snippet } from 'svelte';
	import { Rows3, Columns3, ArrowLeftRight, ArrowUpDown, ZoomIn, ZoomOut, RotateCcw } from 'lucide-svelte';
	import SplitPane from './SplitPane.svelte';
	import Timeline from './scene/Timeline.svelte';
	import FrameEditor from './scene/FrameEditor.svelte';

	const TIMELINE_ORIENTATION_KEY = 'sova-timeline-orientation';

	function loadTimelineOrientation(): 'horizontal' | 'vertical' {
		try {
			const stored = localStorage.getItem(TIMELINE_ORIENTATION_KEY);
			if (stored === 'horizontal' || stored === 'vertical') {
				return stored;
			}
		} catch {
			// Storage unavailable
		}
		return 'horizontal';
	}

	function saveTimelineOrientation(orientation: 'horizontal' | 'vertical'): void {
		try {
			localStorage.setItem(TIMELINE_ORIENTATION_KEY, orientation);
		} catch {
			// Storage unavailable
		}
	}

	interface Props {
		registerToolbar?: (snippet: Snippet | null) => void;
	}

	let { registerToolbar }: Props = $props();

	// Zoom constraints
	const MIN_ZOOM = 0.25;
	const MAX_ZOOM = 4.0;
	const ZOOM_FACTOR = 1.05;

	// Viewport state
	let viewport = $state({ zoom: 1.0, orientation: loadTimelineOrientation() });

	// Layout state - responsive split direction
	let containerEl: HTMLDivElement;
	let containerSize = $state({ width: 0, height: 0 });
	let userOverride = $state(false);
	let userOrientation = $state<'horizontal' | 'vertical'>('vertical');

	const optimalOrientation = $derived(
		containerSize.width > containerSize.height ? 'vertical' : 'horizontal'
	);

	const splitOrientation = $derived(userOverride ? userOrientation : optimalOrientation);

	// Editor state
	let editingFrame = $state<{ lineIdx: number; frameIdx: number } | null>(null);

	// Derived: get the frame being edited
	const currentFrame = $derived((): Frame | null => {
		if (!editingFrame || !$scene) return null;
		const line = $scene.lines[editingFrame.lineIdx];
		if (!line) return null;
		return line.frames[editingFrame.frameIdx] ?? null;
	});

	const frameKey = $derived(
		editingFrame ? `${editingFrame.lineIdx}-${editingFrame.frameIdx}` : null
	);

	function zoomIn() {
		viewport.zoom = Math.min(MAX_ZOOM, viewport.zoom * ZOOM_FACTOR);
	}

	function zoomOut() {
		viewport.zoom = Math.max(MIN_ZOOM, viewport.zoom / ZOOM_FACTOR);
	}

	function resetZoom() {
		viewport.zoom = 1.0;
	}

	function handleZoomChange(zoom: number) {
		viewport.zoom = zoom;
	}

	function toggleTimelineOrientation() {
		const newOrientation = viewport.orientation === 'horizontal' ? 'vertical' : 'horizontal';
		viewport.orientation = newOrientation;
		saveTimelineOrientation(newOrientation);
	}

	function toggleSplitOrientation() {
		userOverride = true;
		userOrientation = userOrientation === 'horizontal' ? 'vertical' : 'horizontal';
	}

	function handleOpenEditor(lineIdx: number, frameIdx: number) {
		editingFrame = { lineIdx, frameIdx };
	}

	function handleCloseEditor() {
		editingFrame = null;
		userOverride = false;
	}

	// Listen for frame/line removal to update editingFrame
	let unlistenFns: UnlistenFn[] = [];
	let resizeObserver: ResizeObserver;

	// Reset editor when a project is loaded
	function handleProjectLoaded() {
		editingFrame = null;
		userOverride = false;
	}

	onMount(async () => {
		registerToolbar?.(toolbarSnippet);

		resizeObserver = new ResizeObserver((entries) => {
			const entry = entries[0];
			if (entry) {
				containerSize = {
					width: entry.contentRect.width,
					height: entry.contentRect.height
				};
			}
		});
		resizeObserver.observe(containerEl);

		window.addEventListener('project:loaded', handleProjectLoaded);

		unlistenFns.push(
			await listen<RemoveFramePayload>(SERVER_EVENTS.REMOVE_FRAME, (event) => {
				if (!editingFrame) return;
				const { lineId, frameId } = event.payload;
				if (editingFrame.lineIdx === lineId) {
					if (editingFrame.frameIdx === frameId) {
						editingFrame = null;
					} else if (editingFrame.frameIdx > frameId) {
						editingFrame = { lineIdx: lineId, frameIdx: editingFrame.frameIdx - 1 };
					}
				}
			}),
			await listen<number>(SERVER_EVENTS.REMOVE_LINE, (event) => {
				if (!editingFrame) return;
				const removedLineId = event.payload;
				if (editingFrame.lineIdx === removedLineId) {
					editingFrame = null;
				} else if (editingFrame.lineIdx > removedLineId) {
					editingFrame = { ...editingFrame, lineIdx: editingFrame.lineIdx - 1 };
				}
			})
		);
	});

	onDestroy(() => {
		registerToolbar?.(null);
		unlistenFns.forEach((fn) => fn());
		resizeObserver?.disconnect();
		window.removeEventListener('project:loaded', handleProjectLoaded);
	});
</script>

{#snippet toolbarSnippet()}
	<div class="toolbar-controls">
		<div class="zoom-controls">
			<button class="toolbar-btn" onclick={zoomOut} title="Zoom out" disabled={viewport.zoom <= MIN_ZOOM}>
				<ZoomOut size={14} />
			</button>
			<span class="zoom-level">{Math.round(viewport.zoom * 100)}%</span>
			<button class="toolbar-btn" onclick={zoomIn} title="Zoom in" disabled={viewport.zoom >= MAX_ZOOM}>
				<ZoomIn size={14} />
			</button>
			{#if viewport.zoom !== 1.0}
				<button class="toolbar-btn" onclick={resetZoom} title="Reset zoom">
					<RotateCcw size={12} />
				</button>
			{/if}
		</div>
		<button class="toolbar-btn" onclick={toggleTimelineOrientation} title="Toggle timeline orientation">
			{#if viewport.orientation === 'horizontal'}
				<Columns3 size={14} />
			{:else}
				<Rows3 size={14} />
			{/if}
		</button>
		<button class="toolbar-btn" onclick={toggleSplitOrientation} title="Toggle split orientation">
			{#if splitOrientation === 'horizontal'}
				<ArrowUpDown size={14} />
			{:else}
				<ArrowLeftRight size={14} />
			{/if}
		</button>
	</div>
{/snippet}

<div class="scene-container">
	<div class="split-container" bind:this={containerEl}>
		{#if editingFrame}
			<SplitPane orientation={splitOrientation}>
				{#snippet first()}
					<Timeline
						{viewport}
						minZoom={MIN_ZOOM}
						maxZoom={MAX_ZOOM}
						zoomFactor={ZOOM_FACTOR}
						onZoomChange={handleZoomChange}
						onOpenEditor={handleOpenEditor}
					/>
				{/snippet}

				{#snippet second()}
					<FrameEditor
						frame={currentFrame()}
						{frameKey}
						lineIdx={editingFrame.lineIdx}
						frameIdx={editingFrame.frameIdx}
						onClose={handleCloseEditor}
					/>
				{/snippet}
			</SplitPane>
		{:else}
			<Timeline
				{viewport}
				minZoom={MIN_ZOOM}
				maxZoom={MAX_ZOOM}
				zoomFactor={ZOOM_FACTOR}
				onZoomChange={handleZoomChange}
				onOpenEditor={handleOpenEditor}
			/>
		{/if}
	</div>
</div>

<style>
	.scene-container {
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		background-color: var(--colors-background);
	}

	.split-container {
		flex: 1;
		overflow: hidden;
	}

	.toolbar-controls {
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
		font-family: monospace;
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
