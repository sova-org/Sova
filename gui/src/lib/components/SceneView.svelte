<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import { invoke } from '@tauri-apps/api/core';
	import { ArrowLeftRight, ArrowUpDown, X, Plus, ZoomIn, ZoomOut, RotateCcw } from 'lucide-svelte';
	import { scene, framePositions, isPlaying } from '$lib/stores';
	import { editorConfig, currentTheme } from '$lib/stores/config';
	import { selection, selectFrame } from '$lib/stores/selection';
	import { createEditor, createEditorSubscriptions } from '$lib/editor/editorFactory';
	import { setFrames, addLine, removeLine, addFrame, removeFrame, ActionTiming } from '$lib/api/client';
	import type { Frame, Line } from '$lib/types/protocol';
	import type { EditorView } from '@codemirror/view';
	import SplitPane from './SplitPane.svelte';

	// Base dimensions (fixed)
	const BASE_PIXELS_PER_BEAT = 60;
	const BASE_TRACK_SIZE = 56;
	const TRACK_HEADER_SIZE = 60;
	const RULER_SIZE = 28;
	const CLIP_PADDING = 4;
	const DURATION_SNAP = 0.25;
	const MIN_CLIP_EXTENT = 80;

	// Zoom constraints
	const MIN_ZOOM = 0.25;
	const MAX_ZOOM = 4.0;
	const ZOOM_STEP = 0.1;

	// Viewport state
	let viewport = $state({ zoom: 1.0, orientation: 'horizontal' as 'horizontal' | 'vertical' });

	// Derived dimensions (zoom-aware)
	const pixelsPerBeat = $derived(BASE_PIXELS_PER_BEAT * viewport.zoom);
	const trackSize = $derived(BASE_TRACK_SIZE * viewport.zoom);

	// State
	let splitOrientation = $state<'horizontal' | 'vertical'>('horizontal');
	let editorContainer: HTMLDivElement;
	let timelineContainer: HTMLDivElement;
	let editorView: EditorView | null = null;
	let unsubscribe: (() => void) | null = null;
	let editorOpen = $state(false);
	let editingFrameKey = $state<string | null>(null);

	// Resize state
	let resizing: { lineIdx: number; frameIdx: number; startPos: number; startDuration: number } | null = $state(null);

	// Duration editing state
	let editingDuration: { lineIdx: number; frameIdx: number; value: string } | null = $state(null);

	// Helper functions
	function getDuration(frame: Frame): number {
		const d = frame.duration;
		return typeof d === 'number' && !isNaN(d) && d > 0 ? d : 1;
	}

	function getReps(frame: Frame): number {
		const r = frame.repetitions;
		return typeof r === 'number' && !isNaN(r) && r >= 1 ? r : 1;
	}

	function getClipLabel(frame: Frame, idx: number): string {
		return frame.name || `F${idx}`;
	}

	function getClipLang(frame: Frame): string {
		return frame.script?.lang || 'bali';
	}

	function formatDuration(duration: number): string {
		return `${duration}b`;
	}

	function formatReps(reps: number): string {
		return reps > 1 ? `×${reps}` : '';
	}

	function calculateClipExtent(frame: Frame): number {
		return getDuration(frame) * getReps(frame) * pixelsPerBeat;
	}

	function getClipOffset(line: Line, frameIdx: number): number {
		let offset = 0;
		for (let i = 0; i < frameIdx; i++) {
			offset += calculateClipExtent(line.frames[i]);
		}
		return offset;
	}

	function getTotalBeats(line: Line): number {
		return line.frames.reduce((sum, f) => sum + getDuration(f) * getReps(f), 0);
	}

	function isClipSelected(lineIdx: number, frameIdx: number): boolean {
		return $selection?.lineId === lineIdx && $selection?.frameId === frameIdx;
	}

	function isClipPlaying(lineIdx: number, frameIdx: number): boolean {
		return $isPlaying && $framePositions[lineIdx]?.[1] === frameIdx;
	}

	function toggleSplitOrientation() {
		splitOrientation = splitOrientation === 'horizontal' ? 'vertical' : 'horizontal';
	}

	function toggleTimelineOrientation() {
		viewport.orientation = viewport.orientation === 'horizontal' ? 'vertical' : 'horizontal';
	}

	// Orientation-aware style helpers
	function getClipStyle(offset: number, extent: number): string {
		const clipSize = trackSize - 8;
		if (viewport.orientation === 'horizontal') {
			return `left: ${offset}px; width: ${extent}px; top: 4px; height: ${clipSize}px`;
		} else {
			return `top: ${offset}px; height: ${extent}px; left: 4px; width: ${clipSize}px`;
		}
	}

	function getMarkerStyle(beat: number): string {
		const pos = beat * pixelsPerBeat;
		if (viewport.orientation === 'horizontal') {
			return `left: ${pos}px`;
		} else {
			return `top: ${pos}px`;
		}
	}

	function getAddClipStyle(line: Line): string {
		const offset = getClipOffset(line, line.frames.length);
		const clipSize = trackSize - 8;
		if (viewport.orientation === 'horizontal') {
			return `left: ${offset}px; top: 4px; height: ${clipSize}px`;
		} else {
			return `top: ${offset}px; left: 4px; width: ${clipSize}px`;
		}
	}

	function getSelectedFrame(): Frame | null {
		if (!$selection || !$scene) return null;
		const line = $scene.lines[$selection.lineId];
		if (!line) return null;
		return line.frames[$selection.frameId] ?? null;
	}

	function getMaxBeats(): number {
		if (!$scene || $scene.lines.length === 0) return 16;
		const max = Math.max(...$scene.lines.map(getTotalBeats));
		return Math.max(16, Math.ceil(max / 4) * 4 + 4);
	}

	function getBeatMarkers(): number[] {
		const max = getMaxBeats();
		const markers: number[] = [];
		for (let i = 0; i <= max; i += 4) {
			markers.push(i);
		}
		return markers;
	}

	// Resize handlers
	function startResize(lineIdx: number, frameIdx: number, event: MouseEvent) {
		event.stopPropagation();
		event.preventDefault();
		if (!$scene) return;
		const frame = $scene.lines[lineIdx].frames[frameIdx];
		resizing = {
			lineIdx,
			frameIdx,
			startPos: viewport.orientation === 'horizontal' ? event.clientX : event.clientY,
			startDuration: getDuration(frame)
		};
		window.addEventListener('mousemove', handleResizeMove);
		window.addEventListener('mouseup', handleResizeEnd);
	}

	function handleResizeMove(event: MouseEvent) {
		if (!resizing || !$scene) return;
		const currentPos = viewport.orientation === 'horizontal' ? event.clientX : event.clientY;
		const delta = currentPos - resizing.startPos;
		const frame = $scene.lines[resizing.lineIdx].frames[resizing.frameIdx];
		const reps = getReps(frame);
		const deltaDuration = delta / pixelsPerBeat / reps;
		const newDuration = Math.max(DURATION_SNAP, Math.round((resizing.startDuration + deltaDuration) / DURATION_SNAP) * DURATION_SNAP);

		const el = document.querySelector(`[data-clip="${resizing.lineIdx}-${resizing.frameIdx}"]`) as HTMLElement;
		if (el) {
			const extent = Math.max(newDuration * reps * pixelsPerBeat, MIN_CLIP_EXTENT);
			if (viewport.orientation === 'horizontal') {
				el.style.width = `${extent}px`;
			} else {
				el.style.height = `${extent}px`;
			}
		}
	}

	async function handleResizeEnd() {
		window.removeEventListener('mousemove', handleResizeMove);
		window.removeEventListener('mouseup', handleResizeEnd);

		if (!resizing || !$scene) {
			resizing = null;
			return;
		}

		const el = document.querySelector(`[data-clip="${resizing.lineIdx}-${resizing.frameIdx}"]`) as HTMLElement;
		if (el) {
			const currentExtent = viewport.orientation === 'horizontal'
				? parseFloat(el.style.width)
				: parseFloat(el.style.height);
			const frame = $scene.lines[resizing.lineIdx].frames[resizing.frameIdx];
			const reps = getReps(frame);
			const newDuration = Math.max(DURATION_SNAP, Math.round((currentExtent / pixelsPerBeat / reps) / DURATION_SNAP) * DURATION_SNAP);

			if (newDuration !== getDuration(frame)) {
				const updatedFrame = { ...frame, duration: newDuration };
				try {
					await setFrames([[resizing.lineIdx, resizing.frameIdx, updatedFrame]], ActionTiming.immediate());
				} catch (error) {
					console.error('Failed to update frame duration:', error);
				}
			}
		}
		resizing = null;
	}

	// Zoom handler
	function handleWheel(event: WheelEvent) {
		if (!event.ctrlKey && !event.metaKey) return;

		event.preventDefault();

		const delta = event.deltaY > 0 ? -ZOOM_STEP : ZOOM_STEP;
		const newZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, viewport.zoom + delta));

		if (newZoom !== viewport.zoom) {
			const rect = timelineContainer.getBoundingClientRect();
			const cursorPos = viewport.orientation === 'horizontal'
				? event.clientX - rect.left
				: event.clientY - rect.top;

			const scrollBefore = viewport.orientation === 'horizontal'
				? timelineContainer.scrollLeft
				: timelineContainer.scrollTop;

			const cursorBeat = (scrollBefore + cursorPos) / pixelsPerBeat;
			viewport.zoom = newZoom;

			const newScroll = cursorBeat * (BASE_PIXELS_PER_BEAT * newZoom) - cursorPos;

			if (viewport.orientation === 'horizontal') {
				timelineContainer.scrollLeft = Math.max(0, newScroll);
			} else {
				timelineContainer.scrollTop = Math.max(0, newScroll);
			}
		}
	}

	// Click handlers
	function handleClipClick(lineIdx: number, frameIdx: number) {
		selectFrame(lineIdx, frameIdx);
	}

	function handleClipDoubleClick(lineIdx: number, frameIdx: number) {
		selectFrame(lineIdx, frameIdx);
		openEditor();
	}

	function openEditor() {
		if (!$selection || !$scene) return;
		const frame = getSelectedFrame();
		if (!frame) return;

		editorOpen = true;
		editingFrameKey = `${$selection.lineId}-${$selection.frameId}`;

		if (editorView) {
			const content = frame.script?.content || '';
			editorView.dispatch({
				changes: {
					from: 0,
					to: editorView.state.doc.length,
					insert: content
				}
			});
		}
	}

	// Add/remove handlers
	async function handleAddFrame(lineIdx: number) {
		if (!$scene) return;
		const line = $scene.lines[lineIdx];
		const newFrameIdx = line.frames.length;
		const frame = await invoke<Frame>('create_default_frame');
		await addFrame(lineIdx, newFrameIdx, frame);
		selectFrame(lineIdx, newFrameIdx);
	}

	async function handleRemoveFrame(lineIdx: number, frameIdx: number, event: MouseEvent) {
		event.stopPropagation();
		if (!$scene) return;

		await removeFrame(lineIdx, frameIdx);

		const line = $scene.lines[lineIdx];
		if (!line || line.frames.length === 0) {
			if ($scene.lines.length > 0) {
				selectFrame(Math.max(0, lineIdx - 1), 0);
			} else {
				selection.set(null);
			}
		} else {
			const newFrameIdx = Math.min(frameIdx, line.frames.length - 1);
			selectFrame(lineIdx, newFrameIdx);
		}
	}

	async function handleAddLine() {
		if (!$scene) return;
		const newLineIdx = $scene.lines.length;
		const line = await invoke<Line>('create_default_line');
		await addLine(newLineIdx, line);
		selectFrame(newLineIdx, 0);
	}

	async function handleRemoveLine(lineIdx: number, event: MouseEvent) {
		event.stopPropagation();
		if (!$scene) return;

		await removeLine(lineIdx);

		if ($scene.lines.length === 0) {
			selection.set(null);
		} else {
			const newLineIdx = Math.min(lineIdx, $scene.lines.length - 1);
			selectFrame(newLineIdx, 0);
		}
	}

	// Keyboard navigation helpers
	function moveToPreviousTrack(lineIdx: number, frameIdx: number) {
		if (!$scene || lineIdx <= 0) return;
		const newFrameIdx = Math.min(frameIdx, $scene.lines[lineIdx - 1].frames.length - 1);
		selectFrame(lineIdx - 1, Math.max(0, newFrameIdx));
	}

	function moveToNextTrack(lineIdx: number, frameIdx: number) {
		if (!$scene || lineIdx >= $scene.lines.length - 1) return;
		const newFrameIdx = Math.min(frameIdx, $scene.lines[lineIdx + 1].frames.length - 1);
		selectFrame(lineIdx + 1, Math.max(0, newFrameIdx));
	}

	function moveToPreviousFrame(lineIdx: number, frameIdx: number) {
		if (frameIdx > 0) {
			selectFrame(lineIdx, frameIdx - 1);
		}
	}

	function moveToNextFrame(lineIdx: number, frameIdx: number, line: Line) {
		if (frameIdx < line.frames.length - 1) {
			selectFrame(lineIdx, frameIdx + 1);
		}
	}

	// Keyboard navigation
	function handleKeydown(event: KeyboardEvent) {
		if (!$scene || $scene.lines.length === 0) return;

		const { key } = event;
		const lineIdx = $selection?.lineId ?? 0;
		const frameIdx = $selection?.frameId ?? 0;
		const line = $scene.lines[lineIdx];
		if (!line) return;

		const isVertical = viewport.orientation === 'vertical';

		switch (key) {
			case 'ArrowUp':
				event.preventDefault();
				if (isVertical) {
					moveToPreviousFrame(lineIdx, frameIdx);
				} else {
					moveToPreviousTrack(lineIdx, frameIdx);
				}
				break;
			case 'ArrowDown':
				event.preventDefault();
				if (isVertical) {
					moveToNextFrame(lineIdx, frameIdx, line);
				} else {
					moveToNextTrack(lineIdx, frameIdx);
				}
				break;
			case 'ArrowLeft':
				event.preventDefault();
				if (isVertical) {
					moveToPreviousTrack(lineIdx, frameIdx);
				} else {
					moveToPreviousFrame(lineIdx, frameIdx);
				}
				break;
			case 'ArrowRight':
				event.preventDefault();
				if (isVertical) {
					moveToNextTrack(lineIdx, frameIdx);
				} else {
					moveToNextFrame(lineIdx, frameIdx, line);
				}
				break;
			case 'Enter':
				event.preventDefault();
				openEditor();
				break;
			case 'Delete':
			case 'Backspace':
				event.preventDefault();
				handleRemoveFrame(lineIdx, frameIdx, event as unknown as MouseEvent);
				break;
			case '+':
			case '=':
				event.preventDefault();
				adjustDuration(lineIdx, frameIdx, DURATION_SNAP);
				break;
			case '-':
			case '_':
				event.preventDefault();
				adjustDuration(lineIdx, frameIdx, -DURATION_SNAP);
				break;
			case ' ':
				event.preventDefault();
				toggleEnabled(lineIdx, frameIdx);
				break;
			case 'Tab':
				event.preventDefault();
				cycleSelection(event.shiftKey);
				break;
		}
	}

	async function adjustDuration(lineIdx: number, frameIdx: number, delta: number) {
		if (!$scene) return;
		const frame = $scene.lines[lineIdx]?.frames[frameIdx];
		if (!frame) return;

		const newDuration = Math.max(DURATION_SNAP, getDuration(frame) + delta);
		const updatedFrame = { ...frame, duration: newDuration };
		try {
			await setFrames([[lineIdx, frameIdx, updatedFrame]], ActionTiming.immediate());
		} catch (error) {
			console.error('Failed to adjust duration:', error);
		}
	}

	async function toggleEnabled(lineIdx: number, frameIdx: number) {
		if (!$scene) return;
		const frame = $scene.lines[lineIdx]?.frames[frameIdx];
		if (!frame) return;

		const updatedFrame = { ...frame, enabled: !frame.enabled };
		try {
			await setFrames([[lineIdx, frameIdx, updatedFrame]], ActionTiming.immediate());
		} catch (error) {
			console.error('Failed to toggle enabled:', error);
		}
	}

	function startDurationEdit(lineIdx: number, frameIdx: number, event: MouseEvent) {
		event.stopPropagation();
		if (!$scene) return;
		const frame = $scene.lines[lineIdx]?.frames[frameIdx];
		if (!frame) return;
		editingDuration = { lineIdx, frameIdx, value: getDuration(frame).toString() };
	}

	function handleDurationInput(event: Event) {
		if (!editingDuration) return;
		editingDuration.value = (event.target as HTMLInputElement).value;
	}

	async function handleDurationKeydown(event: KeyboardEvent) {
		if (!editingDuration || !$scene) return;

		if (event.key === 'Enter') {
			event.preventDefault();
			const parsed = parseFloat(editingDuration.value);
			if (!isNaN(parsed) && parsed > 0) {
				const newDuration = Math.max(DURATION_SNAP, Math.round(parsed / DURATION_SNAP) * DURATION_SNAP);
				const frame = $scene.lines[editingDuration.lineIdx]?.frames[editingDuration.frameIdx];
				if (frame) {
					const updatedFrame = { ...frame, duration: newDuration };
					try {
						await setFrames([[editingDuration.lineIdx, editingDuration.frameIdx, updatedFrame]], ActionTiming.immediate());
					} catch (error) {
						console.error('Failed to update duration:', error);
					}
				}
			}
			editingDuration = null;
		} else if (event.key === 'Escape') {
			editingDuration = null;
		}
	}

	function handleDurationBlur() {
		editingDuration = null;
	}

	function cycleSelection(reverse: boolean) {
		if (!$scene || $scene.lines.length === 0) return;

		let lineIdx = $selection?.lineId ?? 0;
		let frameIdx = $selection?.frameId ?? 0;

		if (reverse) {
			frameIdx--;
			if (frameIdx < 0) {
				lineIdx--;
				if (lineIdx < 0) lineIdx = $scene.lines.length - 1;
				frameIdx = $scene.lines[lineIdx].frames.length - 1;
			}
		} else {
			frameIdx++;
			if (frameIdx >= $scene.lines[lineIdx].frames.length) {
				lineIdx++;
				if (lineIdx >= $scene.lines.length) lineIdx = 0;
				frameIdx = 0;
			}
		}

		selectFrame(lineIdx, Math.max(0, frameIdx));
	}

	// Editor setup
	$effect(() => {
		if (!editorContainer || !$editorConfig) return;

		if (!editorView) {
			editorView = createEditor(
				editorContainer,
				'',
				[],
				$editorConfig,
				$currentTheme
			);
			unsubscribe = createEditorSubscriptions(editorView);
		}
	});

	onDestroy(() => {
		if (unsubscribe) {
			unsubscribe();
		}
		editorView?.destroy();
	});
</script>

<div class="scene-container">
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
					onclick={() => { viewport.zoom = Math.max(MIN_ZOOM, viewport.zoom - ZOOM_STEP); }}
					title="Zoom out"
					disabled={viewport.zoom <= MIN_ZOOM}
				>
					<ZoomOut size={14} />
				</button>
				<span class="zoom-level">{Math.round(viewport.zoom * 100)}%</span>
				<button
					class="toolbar-btn"
					onclick={() => { viewport.zoom = Math.min(MAX_ZOOM, viewport.zoom + ZOOM_STEP); }}
					title="Zoom in"
					disabled={viewport.zoom >= MAX_ZOOM}
				>
					<ZoomIn size={14} />
				</button>
				{#if viewport.zoom !== 1.0}
					<button
						class="toolbar-btn"
						onclick={() => { viewport.zoom = 1.0; }}
						title="Reset zoom"
					>
						<RotateCcw size={12} />
					</button>
				{/if}
			</div>
			<button
				class="toolbar-btn"
				onclick={toggleTimelineOrientation}
				title="Toggle timeline orientation"
			>
				{#if viewport.orientation === 'horizontal'}
					<ArrowLeftRight size={14} />
				{:else}
					<ArrowUpDown size={14} />
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
	</div>

	<div class="split-container">
		<SplitPane orientation={splitOrientation}>
			{#snippet first()}
				<div
					class="timeline-pane"
					class:vertical={viewport.orientation === 'vertical'}
					bind:this={timelineContainer}
					tabindex="0"
					onkeydown={handleKeydown}
					onwheel={handleWheel}
				>
					{#if !$scene || $scene.lines.length === 0}
						<div class="empty">
							<button class="add-track-empty" onclick={handleAddLine}>
								<Plus size={16} />
								<span>Add Track</span>
							</button>
						</div>
					{:else}
						<div class="timeline" class:vertical={viewport.orientation === 'vertical'}>
							<!-- Ruler row -->
							<div class="timeline-row ruler-row" style={viewport.orientation === 'horizontal' ? `height: ${RULER_SIZE}px` : `width: ${RULER_SIZE}px`}>
								<div class="track-header ruler-header"></div>
								<div class="track-content ruler-content">
									{#each getBeatMarkers() as beat}
										<div class="beat-marker" style={getMarkerStyle(beat)}>
											{beat}
										</div>
									{/each}
								</div>
							</div>

							<!-- Tracks -->
							{#each $scene.lines as line, lineIdx}
								<div class="timeline-row track-row" style={viewport.orientation === 'horizontal' ? `height: ${trackSize}px` : `width: ${trackSize}px`}>
									<div class="track-header">
										<span class="track-number">{lineIdx}</span>
										<button
											class="track-remove"
											onclick={(e) => handleRemoveLine(lineIdx, e)}
											title="Remove track"
										>
											<X size={12} />
										</button>
									</div>
									<div class="track-content">
										<!-- Grid lines -->
										<div class="grid-lines">
											{#each getBeatMarkers() as beat}
												<div class="grid-line major" style={getMarkerStyle(beat)}></div>
											{/each}
										</div>

										<!-- Clips -->
										{#each line.frames as frame, frameIdx}
											{@const offset = getClipOffset(line, frameIdx)}
											{@const extent = calculateClipExtent(frame)}
											<div
												class="clip"
												class:selected={isClipSelected(lineIdx, frameIdx)}
												class:playing={isClipPlaying(lineIdx, frameIdx)}
												class:disabled={!frame.enabled}
												data-clip="{lineIdx}-{frameIdx}"
												style={getClipStyle(offset, extent)}
												onclick={() => handleClipClick(lineIdx, frameIdx)}
												ondblclick={() => handleClipDoubleClick(lineIdx, frameIdx)}
												role="button"
												tabindex="-1"
											>
												<div class="clip-top">
													<span class="clip-name">{getClipLabel(frame, frameIdx)}</span>
													<span class="clip-lang">{getClipLang(frame)}</span>
												</div>
												<div class="clip-bottom">
													{#if editingDuration && editingDuration.lineIdx === lineIdx && editingDuration.frameIdx === frameIdx}
														<input
															class="duration-input"
															type="text"
															value={editingDuration.value}
															oninput={handleDurationInput}
															onkeydown={handleDurationKeydown}
															onblur={handleDurationBlur}
															autofocus
														/>
													{:else}
														<span
															class="clip-duration"
															ondblclick={(e) => startDurationEdit(lineIdx, frameIdx, e)}
															title="Double-click to edit"
														>{formatDuration(getDuration(frame))}</span>
													{/if}
													<span class="clip-reps">{formatReps(getReps(frame))}</span>
												</div>
												<button
													class="clip-remove"
													onclick={(e) => handleRemoveFrame(lineIdx, frameIdx, e)}
													title="Remove"
												>
													<X size={10} />
												</button>
												<div
													class="resize-handle"
													onmousedown={(e) => startResize(lineIdx, frameIdx, e)}
												></div>
											</div>
										{/each}

										<!-- Add clip button -->
										<button
											class="add-clip"
											style={getAddClipStyle(line)}
											onclick={() => handleAddFrame(lineIdx)}
											title="Add frame"
										>
											<Plus size={14} />
										</button>
									</div>
								</div>
							{/each}

							<!-- Add track row -->
							<div class="timeline-row add-track-row">
								<button class="add-track" onclick={handleAddLine}>
									<Plus size={14} />
									<span>Add Track</span>
								</button>
							</div>
						</div>
					{/if}
				</div>
			{/snippet}

			{#snippet second()}
				<div class="editor-pane">
					{#if editorOpen && editingFrameKey}
						<div class="editor-header">
							<span>Editing {editingFrameKey}</span>
						</div>
					{:else}
						<div class="editor-placeholder">
							<span>Double-click a clip or press Enter to edit</span>
						</div>
					{/if}
					<div class="editor-container" bind:this={editorContainer}></div>
				</div>
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

	.split-container {
		flex: 1;
		overflow: hidden;
	}

	/* Timeline pane */
	.timeline-pane {
		width: 100%;
		height: 100%;
		overflow: auto;
		outline: none;
	}

	.timeline-pane:focus {
		outline: none;
	}

	.empty {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
	}

	.add-track-empty {
		background: none;
		border: 1px dashed var(--colors-border);
		color: var(--colors-text-secondary);
		padding: 16px 32px;
		cursor: pointer;
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 13px;
	}

	.add-track-empty:hover {
		border-color: var(--colors-accent);
		color: var(--colors-accent);
	}

	.timeline {
		display: flex;
		flex-direction: column;
		min-width: 100%;
	}

	.timeline.vertical {
		flex-direction: row;
		min-width: auto;
		min-height: 100%;
	}

	.timeline-row {
		display: flex;
	}

	.timeline.vertical .timeline-row {
		flex-direction: column;
	}

	/* Ruler */
	.ruler-row {
		background-color: var(--colors-surface);
		border-bottom: 1px solid var(--colors-border);
		position: sticky;
		top: 0;
		z-index: 10;
	}

	.timeline.vertical .ruler-row {
		border-bottom: none;
		border-right: 1px solid var(--colors-border);
		top: auto;
		left: 0;
	}

	.ruler-header {
		width: 60px;
		min-width: 60px;
		border-right: 1px solid var(--colors-border);
	}

	.timeline.vertical .ruler-header {
		width: auto;
		min-width: auto;
		height: 60px;
		min-height: 60px;
		border-right: none;
		border-bottom: 1px solid var(--colors-border);
	}

	.ruler-content {
		flex: 1;
		position: relative;
	}

	.beat-marker {
		position: absolute;
		top: 0;
		height: 100%;
		display: flex;
		align-items: center;
		padding-left: 4px;
		font-size: 10px;
		color: var(--colors-text-secondary);
		border-left: 1px solid var(--colors-border);
	}

	.timeline.vertical .beat-marker {
		top: auto;
		left: 0;
		height: auto;
		width: 100%;
		padding-left: 0;
		padding-top: 4px;
		border-left: none;
		border-top: 1px solid var(--colors-border);
		writing-mode: vertical-rl;
		text-orientation: mixed;
	}

	/* Track header */
	.track-header {
		width: 60px;
		min-width: 60px;
		background-color: var(--colors-surface);
		border-right: 1px solid var(--colors-border);
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 8px;
	}

	.timeline.vertical .track-header {
		width: auto;
		min-width: auto;
		height: 60px;
		min-height: 60px;
		border-right: none;
		border-bottom: 1px solid var(--colors-border);
		flex-direction: column;
		padding: 8px 0;
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

	/* Track content */
	.track-row {
		border-bottom: 1px solid var(--colors-border);
	}

	.timeline.vertical .track-row {
		border-bottom: none;
		border-right: 1px solid var(--colors-border);
	}

	.track-content {
		flex: 1;
		position: relative;
		background-color: var(--colors-background);
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
		opacity: 0.3;
	}

	.timeline.vertical .grid-line {
		top: auto;
		bottom: auto;
		left: 0;
		right: 0;
		width: auto;
		height: 1px;
	}

	.grid-line.major {
		opacity: 0.5;
	}

	/* Clips */
	.clip {
		position: absolute;
		background-color: var(--colors-surface);
		border: 1px solid var(--colors-border);
		cursor: pointer;
		display: flex;
		flex-direction: column;
		justify-content: space-between;
		padding: 6px 8px;
		user-select: none;
		box-sizing: border-box;
	}

	.clip:hover {
		border-color: var(--colors-text-secondary);
	}

	.clip:hover .clip-remove {
		opacity: 0.5;
	}

	.clip.selected {
		border: 2px solid var(--colors-accent);
		padding: 5px 7px;
	}

	.clip.playing {
		background-color: var(--colors-accent);
		border-color: var(--colors-accent);
	}

	.clip.playing .clip-name,
	.clip.playing .clip-lang,
	.clip.playing .clip-duration,
	.clip.playing .clip-reps {
		color: var(--colors-background);
	}

	.clip.disabled {
		opacity: 0.5;
		border-style: dashed;
	}

	.clip-top,
	.clip-bottom {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
	}

	.clip-name {
		font-size: 11px;
		font-weight: 600;
		color: var(--colors-text);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 60%;
	}

	.clip-lang {
		font-size: 9px;
		color: var(--colors-text-secondary);
		text-transform: lowercase;
	}

	.clip-duration {
		font-size: 10px;
		color: var(--colors-text-secondary);
		cursor: text;
	}

	.clip-duration:hover {
		color: var(--colors-accent);
	}

	.duration-input {
		width: 32px;
		font-size: 10px;
		padding: 0 2px;
		border: 1px solid var(--colors-accent);
		background-color: var(--colors-background);
		color: var(--colors-text);
		outline: none;
	}

	.clip-reps {
		font-size: 10px;
		color: var(--colors-text-secondary);
	}

	.clip-remove {
		position: absolute;
		top: 2px;
		right: 2px;
		background: none;
		border: none;
		color: var(--colors-text-secondary);
		cursor: pointer;
		padding: 2px;
		opacity: 0;
		display: flex;
		align-items: center;
	}

	.clip-remove:hover {
		opacity: 1;
		color: var(--colors-accent);
	}

	.resize-handle {
		position: absolute;
		top: 0;
		right: 0;
		width: 6px;
		height: 100%;
		cursor: ew-resize;
		background: transparent;
	}

	.timeline.vertical .resize-handle {
		top: auto;
		bottom: 0;
		right: 0;
		left: 0;
		width: 100%;
		height: 6px;
		cursor: ns-resize;
	}

	.resize-handle:hover {
		background-color: var(--colors-accent);
		opacity: 0.5;
	}

	/* Add clip button */
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

	.timeline.vertical .add-clip {
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

	/* Add track row */
	.add-track-row {
		height: 40px;
	}

	.add-track {
		width: 100%;
		height: 100%;
		background: none;
		border: none;
		border-bottom: 1px dashed var(--colors-border);
		color: var(--colors-text-secondary);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		font-size: 11px;
		opacity: 0.5;
	}

	.add-track:hover {
		opacity: 1;
		color: var(--colors-accent);
		border-color: var(--colors-accent);
	}

	/* Editor pane */
	.editor-pane {
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		background-color: var(--colors-background);
	}

	.editor-header {
		height: 28px;
		background-color: var(--colors-surface);
		border-bottom: 1px solid var(--colors-border);
		display: flex;
		align-items: center;
		padding: 0 12px;
		font-size: 10px;
		color: var(--colors-text-secondary);
	}

	.editor-placeholder {
		height: 28px;
		background-color: var(--colors-surface);
		border-bottom: 1px solid var(--colors-border);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 10px;
		color: var(--colors-text-secondary);
		font-style: italic;
	}

	.editor-container {
		flex: 1;
		overflow: hidden;
	}

	:global(.editor-container .cm-editor) {
		height: 100%;
	}
</style>
