<script lang="ts">
	import { scene } from '$lib/stores';
	import { selection } from '$lib/stores/selection';
	import type { Frame } from '$lib/types/protocol';
	import SplitPane from './SplitPane.svelte';
	import SceneToolbar from './scene/SceneToolbar.svelte';
	import Timeline from './scene/Timeline.svelte';
	import FrameEditor from './scene/FrameEditor.svelte';

	// Zoom constraints
	const MIN_ZOOM = 0.25;
	const MAX_ZOOM = 4.0;
	const ZOOM_FACTOR = 1.1;

	// Viewport state
	let viewport = $state({ zoom: 1.0, orientation: 'horizontal' as 'horizontal' | 'vertical' });

	// Layout state
	let splitOrientation = $state<'horizontal' | 'vertical'>('horizontal');

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

	function handleZoomChange(zoom: number) {
		viewport.zoom = zoom;
	}

	function toggleTimelineOrientation() {
		viewport.orientation = viewport.orientation === 'horizontal' ? 'vertical' : 'horizontal';
	}

	function toggleSplitOrientation() {
		splitOrientation = splitOrientation === 'horizontal' ? 'vertical' : 'horizontal';
	}

	function handleOpenEditor(lineIdx: number, frameIdx: number) {
		editingFrame = { lineIdx, frameIdx };
	}
</script>

<div class="scene-container">
	<SceneToolbar
		zoom={viewport.zoom}
		minZoom={MIN_ZOOM}
		maxZoom={MAX_ZOOM}
		zoomFactor={ZOOM_FACTOR}
		orientation={viewport.orientation}
		{splitOrientation}
		onZoomChange={handleZoomChange}
		onOrientationChange={toggleTimelineOrientation}
		onSplitOrientationChange={toggleSplitOrientation}
	/>

	<div class="split-container">
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
				/>
			{/snippet}
		</SplitPane>
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
</style>
