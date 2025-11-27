<script lang="ts">
	import { X, Plus } from 'lucide-svelte';
	import type { Frame, Line } from '$lib/types/protocol';
	import Clip from './Clip.svelte';
	import { getTimelineContext } from './context.svelte';

	interface Props {
		line: Line;
		lineIdx: number;
		visibleBeatMarkers: number[];
		trackWidth: number;
		previewDuration: number | null;
		previewFrameIdx: number | null;
		onRemoveTrack: (e: MouseEvent) => void;
		onAddClip: () => void;
		onClipSelect: (frameIdx: number) => void;
		onClipDoubleClick: (frameIdx: number) => void;
		onResizeStart: (frameIdx: number, e: MouseEvent) => void;
		onLineResizeStart: (e: MouseEvent) => void;
		onDurationEditStart: (frameIdx: number, e: MouseEvent) => void;
		editingDuration: { frameIdx: number; value: string } | null;
		onDurationInput: (e: Event) => void;
		onDurationKeydown: (e: KeyboardEvent) => void;
		onDurationBlur: () => void;
		onRepsEditStart: (frameIdx: number, e: MouseEvent) => void;
		editingReps: { frameIdx: number; value: string } | null;
		onRepsInput: (e: Event) => void;
		onRepsKeydown: (e: KeyboardEvent) => void;
		onRepsBlur: () => void;
		onNameEditStart: (frameIdx: number, e: MouseEvent) => void;
		editingName: { frameIdx: number; value: string } | null;
		onNameInput: (e: Event) => void;
		onNameKeydown: (e: KeyboardEvent) => void;
		onNameBlur: () => void;
		selectedFrameIdx: number | null;
		playingFrameIdx: number | null;
	}

	let {
		line,
		lineIdx,
		visibleBeatMarkers,
		trackWidth,
		previewDuration,
		previewFrameIdx,
		onRemoveTrack,
		onAddClip,
		onClipSelect,
		onClipDoubleClick,
		onResizeStart,
		onLineResizeStart,
		onDurationEditStart,
		editingDuration,
		onDurationInput,
		onDurationKeydown,
		onDurationBlur,
		onRepsEditStart,
		editingReps,
		onRepsInput,
		onRepsKeydown,
		onRepsBlur,
		onNameEditStart,
		editingName,
		onNameInput,
		onNameKeydown,
		onNameBlur,
		selectedFrameIdx,
		playingFrameIdx
	}: Props = $props();

	const ctx = getTimelineContext();

	function getFrameExtent(frame: Frame, frameIdx: number): number {
		// Use preview duration if this frame is being resized
		const d = (previewFrameIdx === frameIdx && previewDuration !== null)
			? previewDuration
			: (typeof frame.duration === 'number' && !isNaN(frame.duration) && frame.duration > 0 ? frame.duration : 1);
		const r = typeof frame.repetitions === 'number' && !isNaN(frame.repetitions) && frame.repetitions >= 1
			? frame.repetitions : 1;
		return d * r * ctx.pixelsPerBeat;
	}

	// Pre-compute all clip positions in O(n) - single pass
	const clipPositions = $derived.by(() => {
		const positions: { offset: number; extent: number }[] = [];
		let currentOffset = 0;
		for (let i = 0; i < line.frames.length; i++) {
			const frame = line.frames[i];
			const extent = getFrameExtent(frame, i);
			positions.push({ offset: currentOffset, extent });
			currentOffset += extent;
		}
		return positions;
	});

	// Total track length for add button positioning
	const totalLength = $derived(
		clipPositions.length > 0
			? clipPositions[clipPositions.length - 1].offset + clipPositions[clipPositions.length - 1].extent
			: 0
	);

	const trackStyle = $derived(
		ctx.isVertical ? `width: ${trackWidth}px` : `height: ${trackWidth}px`
	);

	const addClipStyle = $derived.by(() => {
		const clipSize = trackWidth - 8;
		return ctx.isVertical
			? `top: ${totalLength}px; left: 4px; width: ${clipSize}px`
			: `left: ${totalLength}px; top: 4px; height: ${clipSize}px`;
	});

	function getMarkerStyle(beat: number): string {
		const pos = beat * ctx.pixelsPerBeat;
		return ctx.isVertical ? `top: ${pos}px` : `left: ${pos}px`;
	}
</script>

<div
	class="track-row"
	class:vertical={ctx.isVertical}
	style={trackStyle}
>
	<div class="track-header" class:vertical={ctx.isVertical}>
		<span class="track-number">{lineIdx}</span>
		<button
			class="track-remove"
			onclick={onRemoveTrack}
			title="Remove track"
		>
			<X size={12} />
		</button>
		<div
			class="line-resize-handle header-handle"
			class:vertical={ctx.isVertical}
			onmousedown={onLineResizeStart}
		></div>
	</div>
	<div class="track-content">
		<!-- Line resize handle -->
		<div
			class="line-resize-handle"
			class:vertical={ctx.isVertical}
			onmousedown={onLineResizeStart}
		></div>
		<div class="grid-lines">
			{#each visibleBeatMarkers as beat}
				<div class="grid-line" class:vertical={ctx.isVertical} style={getMarkerStyle(beat)}></div>
			{/each}
		</div>
		{#each line.frames as frame, frameIdx}
			{@const pos = clipPositions[frameIdx]}
			<Clip
				{frame}
				{lineIdx}
				{frameIdx}
				offset={pos.offset}
				extent={pos.extent}
				{trackWidth}
				selected={selectedFrameIdx === frameIdx}
				playing={playingFrameIdx === frameIdx}
				editingDuration={editingDuration && editingDuration.frameIdx === frameIdx ? editingDuration : null}
				onSelect={() => onClipSelect(frameIdx)}
				onDoubleClick={() => onClipDoubleClick(frameIdx)}
				onResizeStart={(e) => onResizeStart(frameIdx, e)}
				onDurationEditStart={(e) => onDurationEditStart(frameIdx, e)}
				{onDurationInput}
				{onDurationKeydown}
				{onDurationBlur}
				editingReps={editingReps && editingReps.frameIdx === frameIdx ? editingReps : null}
				onRepsEditStart={(e) => onRepsEditStart(frameIdx, e)}
				{onRepsInput}
				{onRepsKeydown}
				{onRepsBlur}
				editingName={editingName && editingName.frameIdx === frameIdx ? editingName : null}
				onNameEditStart={(e) => onNameEditStart(frameIdx, e)}
				{onNameInput}
				{onNameKeydown}
				{onNameBlur}
			/>
		{/each}

		<button
			class="add-clip"
			class:vertical={ctx.isVertical}
			style={addClipStyle}
			onclick={onAddClip}
			title="Add frame"
		>
			<Plus size={14} />
		</button>
	</div>
</div>

<style>
	.track-row {
		display: flex;
		border-bottom: 1px solid var(--colors-border);
	}

	.track-row.vertical {
		flex-direction: column;
		border-bottom: none;
		border-right: 1px solid var(--colors-border);
	}

	.track-header {
		position: relative;
		width: 60px;
		min-width: 60px;
		background-color: var(--colors-surface);
		border-right: 1px solid var(--colors-border);
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 8px;
		box-sizing: border-box;
	}

	.track-header.vertical {
		width: auto;
		min-width: auto;
		height: 60px;
		min-height: 60px;
		border-right: none;
		border-bottom: 1px solid var(--colors-border);
		flex-direction: column;
		padding: 8px 0;
		box-sizing: border-box;
	}

	.track-number {
		font-size: 14px;
		font-weight: 600;
		color: var(--colors-text);
	}

	.track-remove {
		background: none;
		border: none;
		color: var(--colors-text-secondary);
		cursor: pointer;
		padding: 4px;
		opacity: 0;
		display: flex;
		align-items: center;
	}

	.track-row:hover .track-remove {
		opacity: 0.5;
	}

	.track-remove:hover {
		opacity: 1;
		color: var(--colors-accent);
	}

	.track-content {
		flex: 1;
		position: relative;
		overflow: visible;
	}

	.grid-lines {
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		pointer-events: none;
	}

	.grid-line {
		position: absolute;
		top: 0;
		bottom: 0;
		width: 1px;
		background-color: var(--colors-border);
		opacity: 0.5;
	}

	.grid-line.vertical {
		top: auto;
		bottom: auto;
		left: 0;
		right: 0;
		width: auto;
		height: 1px;
	}

	.add-clip {
		position: absolute;
		width: 32px;
		background: none;
		border: 1px dashed var(--colors-border);
		color: var(--colors-text-secondary);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		opacity: 0;
	}

	.add-clip.vertical {
		width: auto;
		height: 32px;
	}

	.track-row:hover .add-clip {
		opacity: 0.5;
	}

	.add-clip:hover {
		opacity: 1;
		border-color: var(--colors-accent);
		color: var(--colors-accent);
	}

	.line-resize-handle {
		position: absolute;
		bottom: -1px;
		left: 0;
		right: 0;
		height: 3px;
		cursor: ns-resize;
		z-index: 10;
	}

	.line-resize-handle.vertical {
		right: -1px;
		top: 0;
		bottom: 0;
		left: auto;
		width: 3px;
		height: auto;
		cursor: ew-resize;
	}

	.line-resize-handle:hover {
		background: var(--colors-accent);
	}

	.line-resize-handle.header-handle {
		left: 0;
		right: 0;
	}

	.line-resize-handle.header-handle.vertical {
		top: 0;
		bottom: 0;
		left: auto;
	}
</style>
