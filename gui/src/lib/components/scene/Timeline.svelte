<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { Plus } from 'lucide-svelte';
	import { scene, framePositions, isPlaying } from '$lib/stores';
	import {
		selection,
		selectFrame,
		extendSelection,
		isFrameInSelection,
		getSelectionBounds,
		collapseToFocus
	} from '$lib/stores/selection';
	import { copySelection, getClipboard, type ClipboardData } from '$lib/stores/clipboard';
	import { setFrames, addLine, removeLine, addFrame, removeFrame, ActionTiming } from '$lib/api/client';
	import type { Frame, Line } from '$lib/types/protocol';
	import Track from './Track.svelte';
	import { createTimelineContext } from './context.svelte';

	interface Props {
		viewport: { zoom: number; orientation: 'horizontal' | 'vertical' };
		minZoom: number;
		maxZoom: number;
		zoomFactor: number;
		onZoomChange: (zoom: number) => void;
		onOpenEditor: (lineIdx: number, frameIdx: number) => void;
	}

	let { viewport, minZoom, maxZoom, zoomFactor, onZoomChange, onOpenEditor }: Props = $props();

	// Constants
	const BASE_PIXELS_PER_BEAT = 60;
	const BASE_TRACK_SIZE = 72;
	const RULER_SIZE = 28;
	const DURATION_SNAP = 0.25;
	const DURATION_SNAP_FINE = 0.125;

	// Derived dimensions (local for use in this component)
	const pixelsPerBeat = $derived(BASE_PIXELS_PER_BEAT * viewport.zoom);
	const trackSize = $derived(BASE_TRACK_SIZE * viewport.zoom);
	const isVertical = $derived(viewport.orientation === 'vertical');

	// Create reactive context for child components (must be at top level)
	const ctx = createTimelineContext({
		pixelsPerBeat: BASE_PIXELS_PER_BEAT * viewport.zoom,
		trackSize: BASE_TRACK_SIZE * viewport.zoom,
		isVertical: viewport.orientation === 'vertical'
	});

	// Keep context in sync with viewport changes
	$effect(() => {
		ctx.pixelsPerBeat = BASE_PIXELS_PER_BEAT * viewport.zoom;
		ctx.trackSize = BASE_TRACK_SIZE * viewport.zoom;
		ctx.isVertical = viewport.orientation === 'vertical';
	});

	// Internal state
	let timelineContainer: HTMLDivElement;
	let resizing: { lineIdx: number; frameIdx: number; startPos: number; startDuration: number; previewDuration: number } | null = $state(null);
	let editingDuration: { lineIdx: number; frameIdx: number; value: string } | null = $state(null);
	let editingReps: { lineIdx: number; frameIdx: number; value: string } | null = $state(null);
	let editingName: { lineIdx: number; frameIdx: number; value: string } | null = $state(null);
	let scrollPos = $state(0);
	let viewportSize = $state(1000);

	// Drag state for frame reordering
	let dragging: {
		sourceLineIdx: number;
		sourceFrameIdx: number;
		frame: Frame;
		currentLineIdx: number;
		currentFrameIdx: number;
	} | null = $state(null);

	// Zoom throttling
	let lastZoomTime = 0;
	const ZOOM_THROTTLE_MS = 50;
	const ZOOM_SENSITIVITY = 0.012;

	// Line width multipliers for resizable tracks (local UI state only)
	let lineWidthMultipliers: Map<number, number> = $state(new Map());
	let lineResizing: { lineIdx: number; startPos: number; startMultiplier: number } | null = $state(null);

	// Solo/Mute state
	let soloLineIdx: number | null = $state(null);
	let mutedLines: Set<number> = $state(new Set());
	let savedEnabledStates: Map<string, boolean> = $state(new Map());

	const LINE_WIDTH_MIN = 0.5;
	const LINE_WIDTH_MAX = 3.0;

	function getLineWidth(lineIdx: number): number {
		const multiplier = lineWidthMultipliers.get(lineIdx) ?? 1.0;
		return BASE_TRACK_SIZE * viewport.zoom * multiplier;
	}

	function handleLineResizeStart(lineIdx: number, event: MouseEvent) {
		event.stopPropagation();
		event.preventDefault();
		const multiplier = lineWidthMultipliers.get(lineIdx) ?? 1.0;
		lineResizing = {
			lineIdx,
			startPos: isVertical ? event.clientX : event.clientY,
			startMultiplier: multiplier
		};
		window.addEventListener('mousemove', handleLineResizeMove);
		window.addEventListener('mouseup', handleLineResizeEnd);
	}

	function handleLineResizeMove(event: MouseEvent) {
		if (!lineResizing) return;
		const currentPos = isVertical ? event.clientX : event.clientY;
		const delta = currentPos - lineResizing.startPos;
		const baseSize = BASE_TRACK_SIZE * viewport.zoom;
		const deltaMultiplier = delta / baseSize;
		const newMultiplier = Math.max(LINE_WIDTH_MIN, Math.min(LINE_WIDTH_MAX, lineResizing.startMultiplier + deltaMultiplier));
		lineWidthMultipliers = new Map(lineWidthMultipliers).set(lineResizing.lineIdx, newMultiplier);
	}

	function handleLineResizeEnd() {
		window.removeEventListener('mousemove', handleLineResizeMove);
		window.removeEventListener('mouseup', handleLineResizeEnd);
		lineResizing = null;
	}

	// Solo/Mute functions
	function saveCurrentStates() {
		if (!$scene || savedEnabledStates.size > 0) return;
		const newStates = new Map<string, boolean>();
		for (let l = 0; l < $scene.lines.length; l++) {
			const line = $scene.lines[l];
			for (let f = 0; f < line.frames.length; f++) {
				newStates.set(`${l}-${f}`, line.frames[f].enabled);
			}
		}
		savedEnabledStates = newStates;
	}

	function getSavedEnabled(lineIdx: number, frameIdx: number): boolean {
		const key = `${lineIdx}-${frameIdx}`;
		return savedEnabledStates.get(key) ?? true;
	}

	async function applyEffects() {
		if (!$scene) return;

		const updates: [number, number, Frame][] = [];

		for (let l = 0; l < $scene.lines.length; l++) {
			const line = $scene.lines[l];
			for (let f = 0; f < line.frames.length; f++) {
				const frame = line.frames[f];
				let shouldBeEnabled: boolean;

				if (soloLineIdx !== null && l !== soloLineIdx) {
					shouldBeEnabled = false;
				} else if (mutedLines.has(l)) {
					shouldBeEnabled = false;
				} else {
					shouldBeEnabled = getSavedEnabled(l, f);
				}

				if (frame.enabled !== shouldBeEnabled) {
					updates.push([l, f, { ...frame, enabled: shouldBeEnabled }]);
				}
			}
		}

		if (updates.length > 0) {
			try {
				await setFrames(updates, ActionTiming.immediate());
			} catch (error) {
				console.error('Failed to apply solo/mute effects:', error);
			}
		}
	}

	async function toggleSolo(lineIdx: number) {
		if (soloLineIdx === lineIdx) {
			soloLineIdx = null;
			if (mutedLines.size === 0) {
				await applyEffects();
				savedEnabledStates = new Map();
			} else {
				await applyEffects();
			}
		} else {
			saveCurrentStates();
			soloLineIdx = lineIdx;
			await applyEffects();
		}
	}

	async function toggleMute(lineIdx: number) {
		saveCurrentStates();
		const newMuted = new Set(mutedLines);
		if (newMuted.has(lineIdx)) {
			newMuted.delete(lineIdx);
		} else {
			newMuted.add(lineIdx);
		}
		mutedLines = newMuted;
		await applyEffects();

		if (soloLineIdx === null && mutedLines.size === 0) {
			savedEnabledStates = new Map();
		}
	}

	function isSolo(lineIdx: number): boolean {
		return soloLineIdx === lineIdx;
	}

	function isMuted(lineIdx: number): boolean {
		return mutedLines.has(lineIdx);
	}

	// Visible beat markers based on scroll position (every 4 beats)
	const visibleBeatMarkers = $derived.by(() => {
		const beatSpacing = 4 * pixelsPerBeat;
		const startBeat = Math.floor(scrollPos / beatSpacing) * 4;
		const endBeat = Math.ceil((scrollPos + viewportSize) / beatSpacing) * 4 + 4;
		const markers: number[] = [];
		for (let b = startBeat; b <= endBeat; b += 4) {
			markers.push(b);
		}
		return markers;
	});

	// Timeline always extends beyond current scroll position
	const timelineExtent = $derived(scrollPos + viewportSize * 2);

	function handleScroll() {
		if (!timelineContainer) return;
		scrollPos = isVertical ? timelineContainer.scrollTop : timelineContainer.scrollLeft;
		viewportSize = isVertical ? timelineContainer.clientHeight : timelineContainer.clientWidth;
	}

	function getMarkerStyle(beat: number): string {
		const pos = beat * pixelsPerBeat;
		return isVertical ? `top: ${pos}px` : `left: ${pos}px`;
	}

	// Helper functions
	function getDuration(frame: Frame): number {
		const d = frame.duration;
		return typeof d === 'number' && !isNaN(d) && d > 0 ? d : 1;
	}

	function getReps(frame: Frame): number {
		const r = frame.repetitions;
		return typeof r === 'number' && !isNaN(r) && r >= 1 ? r : 1;
	}

	function isFrameSelected(lineIdx: number, frameIdx: number): boolean {
		return isFrameInSelection($selection, $scene, lineIdx, frameIdx);
	}

	function getPlayingFrameIdx(lineIdx: number): number | null {
		return $isPlaying ? ($framePositions[lineIdx]?.[0] ?? null) : null;
	}

	// Get preview duration for a specific clip (for reactive resize preview)
	function getPreviewDuration(lineIdx: number, frameIdx: number): number | null {
		if (resizing && resizing.lineIdx === lineIdx && resizing.frameIdx === frameIdx) {
			return resizing.previewDuration;
		}
		return null;
	}

	// Resize handlers - fully reactive, no DOM manipulation
	function startResize(lineIdx: number, frameIdx: number, event: MouseEvent) {
		event.stopPropagation();
		event.preventDefault();
		if (!$scene) return;
		const line = $scene.lines[lineIdx];
		if (!line) return;
		const frame = line.frames[frameIdx];
		if (!frame) return;
		const duration = getDuration(frame);
		resizing = {
			lineIdx,
			frameIdx,
			startPos: isVertical ? event.clientY : event.clientX,
			startDuration: duration,
			previewDuration: duration
		};
		window.addEventListener('mousemove', handleResizeMove);
		window.addEventListener('mouseup', handleResizeEnd);
	}

	function handleResizeMove(event: MouseEvent) {
		if (!resizing || !$scene) return;
		const line = $scene.lines[resizing.lineIdx];
		if (!line) return;
		const frame = line.frames[resizing.frameIdx];
		if (!frame) return;

		const snap = event.shiftKey ? DURATION_SNAP_FINE : DURATION_SNAP;
		const currentPos = isVertical ? event.clientY : event.clientX;
		const delta = currentPos - resizing.startPos;
		const reps = getReps(frame);
		const deltaDuration = delta / pixelsPerBeat / reps;
		const newDuration = Math.max(snap, Math.round((resizing.startDuration + deltaDuration) / snap) * snap);

		// Update preview reactively - this will cause Track/Clip to re-render
		resizing = { ...resizing, previewDuration: newDuration };
	}

	async function handleResizeEnd(event: MouseEvent) {
		window.removeEventListener('mousemove', handleResizeMove);
		window.removeEventListener('mouseup', handleResizeEnd);

		if (!resizing || !$scene) {
			resizing = null;
			return;
		}

		const line = $scene.lines[resizing.lineIdx];
		if (!line) {
			resizing = null;
			return;
		}
		const frame = line.frames[resizing.frameIdx];
		if (!frame) {
			resizing = null;
			return;
		}

		const newDuration = resizing.previewDuration;
		if (newDuration !== getDuration(frame)) {
			const updatedFrame = { ...frame, duration: newDuration };
			try {
				await setFrames([[resizing.lineIdx, resizing.frameIdx, updatedFrame]], ActionTiming.immediate());
			} catch (error) {
				console.error('Failed to update frame duration:', error);
			}
		}
		resizing = null;
	}

	// Wheel handler with throttle + deltaY-based intensity
	function handleWheel(event: WheelEvent) {
		if (!event.ctrlKey && !event.metaKey) return;
		event.preventDefault();

		const now = Date.now();
		if (now - lastZoomTime < ZOOM_THROTTLE_MS) return;
		lastZoomTime = now;

		const delta = Math.abs(event.deltaY);
		const intensity = Math.min(delta * ZOOM_SENSITIVITY, 0.15);
		const direction = event.deltaY < 0 ? 1 : -1;

		const newZoom = Math.max(minZoom, Math.min(maxZoom,
			viewport.zoom * (1 + direction * intensity)
		));
		onZoomChange(newZoom);
	}

	function handleClipClick(lineIdx: number, frameIdx: number, event: MouseEvent) {
		if (event.shiftKey && $selection) {
			extendSelection(lineIdx, frameIdx);
		} else {
			selectFrame(lineIdx, frameIdx);
		}
	}

	function handleClipDoubleClick(lineIdx: number, frameIdx: number) {
		selectFrame(lineIdx, frameIdx);
		onOpenEditor(lineIdx, frameIdx);
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

	async function insertFrameAfter(lineIdx: number, frameIdx: number, frame: Frame) {
		await addFrame(lineIdx, frameIdx + 1, frame);
		selectFrame(lineIdx, frameIdx + 1);
	}

	async function insertFramesAfter(lineIdx: number, frameIdx: number, data: ClipboardData) {
		if (!$scene || data.frames.length === 0) return;

		for (let l = 0; l < data.frames.length; l++) {
			const targetLine = lineIdx + l;
			if (targetLine >= $scene.lines.length) continue;

			const row = data.frames[l];
			let insertIdx = frameIdx + 1;

			for (const frame of row) {
				if (frame) {
					await addFrame(targetLine, insertIdx, structuredClone(frame));
					insertIdx++;
				}
			}
		}

		selectFrame(lineIdx, frameIdx + 1);
	}

	async function insertFramesBefore(lineIdx: number, frameIdx: number, data: ClipboardData) {
		if (!$scene || data.frames.length === 0) return;

		for (let l = 0; l < data.frames.length; l++) {
			const targetLine = lineIdx + l;
			if (targetLine >= $scene.lines.length) continue;

			const row = data.frames[l];
			let insertIdx = frameIdx;

			for (const frame of row) {
				if (frame) {
					await addFrame(targetLine, insertIdx, structuredClone(frame));
					insertIdx++;
				}
			}
		}

		selectFrame(lineIdx, frameIdx);
	}

	async function insertNewFrameBefore(lineIdx: number, frameIdx: number) {
		const frame = await invoke<Frame>('create_default_frame');
		await addFrame(lineIdx, frameIdx, frame);
		selectFrame(lineIdx, frameIdx);
	}

	async function insertNewFrameAfter(lineIdx: number, frameIdx: number) {
		const frame = await invoke<Frame>('create_default_frame');
		await addFrame(lineIdx, frameIdx + 1, frame);
		selectFrame(lineIdx, frameIdx + 1);
	}

	// Drag handlers for frame reordering
	function handleClipDragStart(lineIdx: number, frameIdx: number) {
		if (!$scene) return;
		const frame = $scene.lines[lineIdx]?.frames[frameIdx];
		if (!frame) return;

		dragging = {
			sourceLineIdx: lineIdx,
			sourceFrameIdx: frameIdx,
			frame: structuredClone(frame),
			currentLineIdx: lineIdx,
			currentFrameIdx: frameIdx
		};

		window.addEventListener('mousemove', handleDragMove);
		window.addEventListener('mouseup', handleDragEnd);
	}

	function handleDragMove(event: MouseEvent) {
		if (!dragging || !timelineContainer || !$scene) return;

		const rect = timelineContainer.getBoundingClientRect();
		const scrollX = timelineContainer.scrollLeft;
		const scrollY = timelineContainer.scrollTop;
		const x = event.clientX - rect.left + scrollX;
		const y = event.clientY - rect.top + scrollY;

		const { lineIdx, frameIdx } = calculateDropPosition(x, y);
		dragging = { ...dragging, currentLineIdx: lineIdx, currentFrameIdx: frameIdx };
	}

	async function handleDragEnd() {
		window.removeEventListener('mousemove', handleDragMove);
		window.removeEventListener('mouseup', handleDragEnd);

		if (!dragging || !$scene) {
			dragging = null;
			return;
		}

		const { sourceLineIdx, sourceFrameIdx, frame, currentLineIdx, currentFrameIdx } = dragging;

		// Check if dropped at same position (no move needed)
		const samePosition =
			sourceLineIdx === currentLineIdx &&
			(sourceFrameIdx === currentFrameIdx || sourceFrameIdx === currentFrameIdx - 1);

		if (!samePosition) {
			let targetIdx = currentFrameIdx;

			// Adjust target index if moving within same line and forward
			if (sourceLineIdx === currentLineIdx && sourceFrameIdx < currentFrameIdx) {
				targetIdx--;
			}

			await removeFrame(sourceLineIdx, sourceFrameIdx);
			await addFrame(currentLineIdx, targetIdx, frame);
			selectFrame(currentLineIdx, targetIdx);
		}

		dragging = null;
	}

	function calculateDropPosition(mouseX: number, mouseY: number): { lineIdx: number; frameIdx: number } {
		if (!$scene || $scene.lines.length === 0) return { lineIdx: 0, frameIdx: 0 };

		const HEADER_SIZE = 70;

		// Calculate line index based on mouse position
		let lineIdx = 0;
		let accumulatedSize = RULER_SIZE;

		for (let l = 0; l < $scene.lines.length; l++) {
			const size = getLineWidth(l);
			const pos = isVertical ? mouseX : mouseY;
			if (pos < accumulatedSize + size) {
				lineIdx = l;
				break;
			}
			accumulatedSize += size;
			lineIdx = l;
		}

		// Calculate frame index based on time position
		const timePos = (isVertical ? mouseY : mouseX) - HEADER_SIZE;
		const line = $scene.lines[lineIdx];
		const frameIdx = calculateFrameAtPosition(line, timePos);

		return { lineIdx, frameIdx };
	}

	function calculateFrameAtPosition(line: Line, pixelPos: number): number {
		if (!line || line.frames.length === 0) return 0;

		let accumulatedPixels = 0;
		for (let f = 0; f < line.frames.length; f++) {
			const frame = line.frames[f];
			const duration = getDuration(frame);
			const reps = getReps(frame);
			const framePixels = duration * reps * pixelsPerBeat;

			// Insert before this frame if mouse is in first half
			if (pixelPos < accumulatedPixels + framePixels / 2) {
				return f;
			}
			accumulatedPixels += framePixels;
		}

		// Insert at end
		return line.frames.length;
	}

	function getDropIndicatorIdx(lineIdx: number): number | null {
		if (!dragging || dragging.currentLineIdx !== lineIdx) return null;
		return dragging.currentFrameIdx;
	}

	async function handleRemoveSelectedFrames() {
		if (!$scene || !$selection) return;

		const bounds = getSelectionBounds($selection);
		const framesToRemove: Array<{ lineIdx: number; frameIdx: number }> = [];

		for (let l = bounds.minLine; l <= bounds.maxLine; l++) {
			const line = $scene.lines[l];
			if (!line) continue;
			for (let f = bounds.minFrame; f <= bounds.maxFrame; f++) {
				if (f < line.frames.length) {
					framesToRemove.push({ lineIdx: l, frameIdx: f });
				}
			}
		}

		// Remove in reverse order to keep indices valid
		framesToRemove.sort((a, b) => {
			if (a.lineIdx !== b.lineIdx) return b.lineIdx - a.lineIdx;
			return b.frameIdx - a.frameIdx;
		});

		for (const { lineIdx, frameIdx } of framesToRemove) {
			await removeFrame(lineIdx, frameIdx);
		}

		// Select valid position after removal
		if ($scene.lines.length === 0) {
			selection.set(null);
		} else {
			const newLineIdx = Math.min(bounds.minLine, $scene.lines.length - 1);
			const newLine = $scene.lines[newLineIdx];
			const newFrameIdx = newLine && newLine.frames.length > 0 ? Math.min(bounds.minFrame, newLine.frames.length - 1) : 0;
			if (newLine && newLine.frames.length > 0) {
				selectFrame(newLineIdx, newFrameIdx);
			} else {
				selection.set(null);
			}
		}
	}

	// Keyboard navigation
	function handleKeydown(event: KeyboardEvent) {
		if (editingDuration || editingReps || editingName) return;
		if (!$scene || $scene.lines.length === 0) return;

		const { key } = event;
		const lineIdx = $selection?.focus.lineId ?? 0;
		const frameIdx = $selection?.focus.frameId ?? 0;
		const line = $scene.lines[lineIdx];
		if (!line) return;
		const frame = line.frames[frameIdx];

		// Clipboard operations (Ctrl/Cmd + key)
		if ((event.ctrlKey || event.metaKey) && $selection) {
			switch (key.toLowerCase()) {
				case 'c':
					event.preventDefault();
					copySelection($scene, $selection);
					return;
				case 'v':
					event.preventDefault();
					const pasted = getClipboard();
					if (pasted) {
						if (event.shiftKey) {
							insertFramesBefore(lineIdx, frameIdx, pasted);
						} else {
							insertFramesAfter(lineIdx, frameIdx, pasted);
						}
					}
					return;
				case 'd':
					event.preventDefault();
					copySelection($scene, $selection);
					const duplicateData = getClipboard();
					if (duplicateData) insertFramesAfter(lineIdx, frameIdx, duplicateData);
					return;
			}
		}

		// Arrow keys with Shift extend selection
		if (event.shiftKey && key.startsWith('Arrow')) {
			event.preventDefault();
			let newLine = lineIdx;
			let newFrame = frameIdx;

			switch (key) {
				case 'ArrowUp':
					if (isVertical) {
						newFrame = Math.max(0, frameIdx - 1);
					} else {
						newLine = Math.max(0, lineIdx - 1);
					}
					break;
				case 'ArrowDown':
					if (isVertical) {
						newFrame = Math.min(line.frames.length - 1, frameIdx + 1);
					} else {
						newLine = Math.min($scene.lines.length - 1, lineIdx + 1);
					}
					break;
				case 'ArrowLeft':
					if (isVertical) {
						newLine = Math.max(0, lineIdx - 1);
					} else {
						newFrame = Math.max(0, frameIdx - 1);
					}
					break;
				case 'ArrowRight':
					if (isVertical) {
						newLine = Math.min($scene.lines.length - 1, lineIdx + 1);
					} else {
						newFrame = Math.min(line.frames.length - 1, frameIdx + 1);
					}
					break;
			}
			extendSelection(newLine, newFrame);
			return;
		}

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
			case 'Escape':
				event.preventDefault();
				collapseToFocus();
				break;
			case 'Enter':
				event.preventDefault();
				onOpenEditor(lineIdx, frameIdx);
				break;
			case 'Delete':
			case 'Backspace':
				event.preventDefault();
				handleRemoveSelectedFrames();
				break;
			case '+':
			case '=':
				event.preventDefault();
				adjustDuration(lineIdx, frameIdx, event.shiftKey ? DURATION_SNAP_FINE : DURATION_SNAP);
				break;
			case '-':
			case '_':
				event.preventDefault();
				adjustDuration(lineIdx, frameIdx, event.shiftKey ? -DURATION_SNAP_FINE : -DURATION_SNAP);
				break;
			case ' ':
				event.preventDefault();
				toggleEnabled(lineIdx, frameIdx);
				break;
			case 'Tab':
				event.preventDefault();
				cycleSelection(event.shiftKey);
				break;
			case 'i':
				event.preventDefault();
				insertNewFrameBefore(lineIdx, frameIdx);
				break;
			case 'a':
				event.preventDefault();
				insertNewFrameAfter(lineIdx, frameIdx);
				break;
		}
	}

	async function adjustDuration(lineIdx: number, frameIdx: number, delta: number) {
		if (!$scene) return;
		const frame = $scene.lines[lineIdx]?.frames[frameIdx];
		if (!frame) return;

		const minDuration = Math.abs(delta) < DURATION_SNAP ? DURATION_SNAP_FINE : DURATION_SNAP;
		const newDuration = Math.max(minDuration, getDuration(frame) + delta);
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

		const newEnabled = !frame.enabled;
		const updatedFrame = { ...frame, enabled: newEnabled };

		// Also update saved state if we have any solo/mute active
		if (savedEnabledStates.size > 0) {
			const key = `${lineIdx}-${frameIdx}`;
			savedEnabledStates = new Map(savedEnabledStates).set(key, newEnabled);
		}

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
			event.stopPropagation();
			const snap = event.shiftKey ? DURATION_SNAP_FINE : DURATION_SNAP;
			const parsed = parseFloat(editingDuration.value);
			if (!isNaN(parsed) && parsed > 0) {
				const newDuration = Math.max(snap, Math.round(parsed / snap) * snap);
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
			event.stopPropagation();
			editingDuration = null;
		}
	}

	function handleDurationBlur() {
		editingDuration = null;
	}

	// Reps editing handlers
	function startRepsEdit(lineIdx: number, frameIdx: number, event: MouseEvent) {
		event.stopPropagation();
		if (!$scene) return;
		const frame = $scene.lines[lineIdx]?.frames[frameIdx];
		if (!frame) return;
		editingReps = { lineIdx, frameIdx, value: getReps(frame).toString() };
	}

	function handleRepsInput(event: Event) {
		if (!editingReps) return;
		editingReps.value = (event.target as HTMLInputElement).value;
	}

	async function handleRepsKeydown(event: KeyboardEvent) {
		if (!editingReps || !$scene) return;

		if (event.key === 'Enter') {
			event.preventDefault();
			event.stopPropagation();
			const parsed = parseInt(editingReps.value, 10);
			if (!isNaN(parsed) && parsed >= 1) {
				const frame = $scene.lines[editingReps.lineIdx]?.frames[editingReps.frameIdx];
				if (frame) {
					const updatedFrame = { ...frame, repetitions: parsed };
					try {
						await setFrames([[editingReps.lineIdx, editingReps.frameIdx, updatedFrame]], ActionTiming.immediate());
					} catch (error) {
						console.error('Failed to update repetitions:', error);
					}
				}
			}
			editingReps = null;
		} else if (event.key === 'Escape') {
			event.stopPropagation();
			editingReps = null;
		}
	}

	function handleRepsBlur() {
		editingReps = null;
	}

	// Name editing handlers
	function startNameEdit(lineIdx: number, frameIdx: number, event: MouseEvent) {
		event.stopPropagation();
		if (!$scene) return;
		const frame = $scene.lines[lineIdx]?.frames[frameIdx];
		if (!frame) return;
		editingName = { lineIdx, frameIdx, value: frame.name || '' };
	}

	function handleNameInput(event: Event) {
		if (!editingName) return;
		editingName.value = (event.target as HTMLInputElement).value;
	}

	async function handleNameKeydown(event: KeyboardEvent) {
		if (!editingName || !$scene) return;

		if (event.key === 'Enter') {
			event.preventDefault();
			event.stopPropagation();
			const frame = $scene.lines[editingName.lineIdx]?.frames[editingName.frameIdx];
			if (frame) {
				const newName = editingName.value.trim() || null;
				const updatedFrame = { ...frame, name: newName };
				try {
					await setFrames([[editingName.lineIdx, editingName.frameIdx, updatedFrame]], ActionTiming.immediate());
				} catch (error) {
					console.error('Failed to update name:', error);
				}
			}
			editingName = null;
		} else if (event.key === 'Escape') {
			event.stopPropagation();
			editingName = null;
		}
	}

	function handleNameBlur() {
		editingName = null;
	}

	function getEditingNameForTrack(lineIdx: number): { frameIdx: number; value: string } | null {
		if (editingName && editingName.lineIdx === lineIdx) {
			return { frameIdx: editingName.frameIdx, value: editingName.value };
		}
		return null;
	}

	function cycleSelection(reverse: boolean) {
		if (!$scene || $scene.lines.length === 0) return;

		let lineIdx = $selection?.focus.lineId ?? 0;
		let frameIdx = $selection?.focus.frameId ?? 0;

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

	function getEditingDurationForTrack(lineIdx: number): { frameIdx: number; value: string } | null {
		if (editingDuration && editingDuration.lineIdx === lineIdx) {
			return { frameIdx: editingDuration.frameIdx, value: editingDuration.value };
		}
		return null;
	}

	function getEditingRepsForTrack(lineIdx: number): { frameIdx: number; value: string } | null {
		if (editingReps && editingReps.lineIdx === lineIdx) {
			return { frameIdx: editingReps.frameIdx, value: editingReps.value };
		}
		return null;
	}
</script>

<div
	class="timeline-pane"
	class:vertical={isVertical}
	bind:this={timelineContainer}
	tabindex="0"
	onkeydown={handleKeydown}
	onwheel={handleWheel}
	onscroll={handleScroll}
>
	{#if !$scene || $scene.lines.length === 0}
		<div class="empty">
			<button class="add-track-empty" onclick={handleAddLine}>
				<Plus size={16} />
				<span>Add Track</span>
			</button>
		</div>
	{:else}
		<div
			class="timeline"
			class:vertical={isVertical}
			style={isVertical ? `min-height: ${timelineExtent}px` : `min-width: ${timelineExtent}px`}
		>
			<!-- Ruler row -->
			<div class="timeline-row ruler-row" class:vertical={isVertical} style={isVertical ? `width: ${RULER_SIZE}px` : `height: ${RULER_SIZE}px`}>
				<div class="ruler-header" class:vertical={isVertical}></div>
				<div class="ruler-content">
					{#each visibleBeatMarkers as beat}
						<div class="beat-marker" class:vertical={isVertical} style={getMarkerStyle(beat)}>
							{beat}
						</div>
					{/each}
				</div>
			</div>

			<!-- Tracks -->
			{#each $scene.lines as line, lineIdx}
				<Track
					{line}
					{lineIdx}
					{visibleBeatMarkers}
					trackWidth={getLineWidth(lineIdx)}
					previewDuration={getPreviewDuration(lineIdx, resizing?.frameIdx ?? -1)}
					previewFrameIdx={resizing?.lineIdx === lineIdx ? resizing.frameIdx : null}
					onRemoveTrack={(e) => handleRemoveLine(lineIdx, e)}
					onAddClip={() => handleAddFrame(lineIdx)}
					onClipClick={(frameIdx, e) => handleClipClick(lineIdx, frameIdx, e)}
					onClipDoubleClick={(frameIdx) => handleClipDoubleClick(lineIdx, frameIdx)}
					onResizeStart={(frameIdx, e) => startResize(lineIdx, frameIdx, e)}
					onLineResizeStart={(e) => handleLineResizeStart(lineIdx, e)}
					onDurationEditStart={(frameIdx, e) => startDurationEdit(lineIdx, frameIdx, e)}
					editingDuration={getEditingDurationForTrack(lineIdx)}
					onDurationInput={handleDurationInput}
					onDurationKeydown={handleDurationKeydown}
					onDurationBlur={handleDurationBlur}
					onRepsEditStart={(frameIdx, e) => startRepsEdit(lineIdx, frameIdx, e)}
					editingReps={getEditingRepsForTrack(lineIdx)}
					onRepsInput={handleRepsInput}
					onRepsKeydown={handleRepsKeydown}
					onRepsBlur={handleRepsBlur}
					onNameEditStart={(frameIdx, e) => startNameEdit(lineIdx, frameIdx, e)}
					editingName={getEditingNameForTrack(lineIdx)}
					onNameInput={handleNameInput}
					onNameKeydown={handleNameKeydown}
					onNameBlur={handleNameBlur}
					isFrameSelected={(frameIdx) => isFrameSelected(lineIdx, frameIdx)}
					playingFrameIdx={getPlayingFrameIdx(lineIdx)}
					onSolo={() => toggleSolo(lineIdx)}
					onMute={() => toggleMute(lineIdx)}
					isSolo={isSolo(lineIdx)}
					isMuted={isMuted(lineIdx)}
					dropIndicatorIdx={getDropIndicatorIdx(lineIdx)}
					onClipDragStart={(frameIdx) => handleClipDragStart(lineIdx, frameIdx)}
				/>
			{/each}

			<!-- Add track row -->
			<div class="timeline-row add-track-row" class:vertical={isVertical}>
				<button class="add-track" class:vertical={isVertical} onclick={handleAddLine}>
					<Plus size={14} />
					<span>Add Track</span>
				</button>
			</div>
		</div>
	{/if}
</div>

<style>
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

	.timeline-row.vertical {
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

	.ruler-row.vertical {
		border-bottom: none;
		border-right: 1px solid var(--colors-border);
		top: auto;
		left: 0;
	}

	.ruler-header {
		width: 70px;
		min-width: 70px;
		border-right: 1px solid var(--colors-border);
		box-sizing: border-box;
	}

	.ruler-header.vertical {
		width: auto;
		min-width: auto;
		height: auto;
		min-height: auto;
		border-right: none;
		border-bottom: 1px solid var(--colors-border);
		box-sizing: border-box;
		padding: 8px 0;
	}

	.ruler-content {
		flex: 1;
		position: relative;
		overflow: visible;
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

	.beat-marker.vertical {
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

	/* Add track row */
	.add-track-row {
		height: 40px;
	}

	.add-track-row.vertical {
		height: auto;
		width: 40px;
		min-height: 100%;
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

	.add-track.vertical {
		width: 100%;
		height: 100%;
		min-height: 100%;
		writing-mode: vertical-rl;
		border-bottom: none;
		border-left: 1px dashed var(--colors-border);
	}

	.add-track.vertical:hover {
		border-left-color: var(--colors-accent);
	}
</style>
