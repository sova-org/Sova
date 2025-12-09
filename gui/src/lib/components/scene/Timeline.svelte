<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { Plus, RefreshCw } from "lucide-svelte";
    import { scene, framePositions, isPlaying } from "$lib/stores";
    import {
        selection,
        selectFrame,
        extendSelection,
        isFrameInSelection,
    } from "$lib/stores/selection";
    import {
        setFrames,
        addLine,
        removeLine,
        addFrame,
        removeFrame,
        ActionTiming,
    } from "$lib/api/client";
    import type { Frame, Line } from "$lib/types/protocol";
    import Track from "./Track.svelte";
    import { createTimelineContext } from "./context.svelte";
    import { timelineUI } from "$lib/stores/timelineUI";
    import { useSoloMute } from "./useSoloMute.svelte";
    import { useTimelineKeyboard } from "./useTimelineKeyboard.svelte";

    interface Props {
        viewport: { zoom: number; orientation: "horizontal" | "vertical" };
        minZoom: number;
        maxZoom: number;
        zoomFactor: number;
        onZoomChange: (_zoom: number) => void;
        onOpenEditor: (_lineIdx: number, _frameIdx: number) => void;
    }

    let {
        viewport,
        minZoom,
        maxZoom,
        zoomFactor: _zoomFactor,
        onZoomChange,
        onOpenEditor,
    }: Props = $props();

    // Constants
    const BASE_PIXELS_PER_BEAT = 60;
    const BASE_TRACK_SIZE = 72;
    const RULER_SIZE = 28;
    const LINE_WIDTH_MIN = 0.5;
    const LINE_WIDTH_MAX = 3.0;

    // Derived dimensions
    const pixelsPerBeat = $derived(BASE_PIXELS_PER_BEAT * viewport.zoom);
    const isVertical = $derived(viewport.orientation === "vertical");

    // Create reactive context
    const ctx = createTimelineContext({
        pixelsPerBeat: BASE_PIXELS_PER_BEAT * viewport.zoom,
        trackSize: BASE_TRACK_SIZE * viewport.zoom,
        isVertical: viewport.orientation === "vertical",
    });

    // Keep context in sync
    $effect(() => {
        ctx.pixelsPerBeat = BASE_PIXELS_PER_BEAT * viewport.zoom;
        ctx.trackSize = BASE_TRACK_SIZE * viewport.zoom;
        ctx.isVertical = viewport.orientation === "vertical";
    });

    // DEBUG: Log when scene changes
    $effect(() => {
        console.log('[TIMELINE] $scene.lines:', $scene?.lines.length,
            'frames:', $scene?.lines.map(l => l.frames.length));
    });

    // Use composables
    const soloMute = useSoloMute();
    const keyboard = useTimelineKeyboard({ ctx, onOpenEditor });

    // Internal state
    let timelineContainer: HTMLDivElement;
    let scrollPos = $state(0);
    let viewportSize = $state(1000);

    // Zoom throttling
    let lastZoomTime = 0;
    const ZOOM_THROTTLE_MS = 50;
    const ZOOM_SENSITIVITY = 0.012;

    // Line resize state
    let lineResizing: {
        lineIdx: number;
        startPos: number;
        startMultiplier: number;
    } | null = $state(null);

    function getLineWidth(lineIdx: number): number {
        const multiplier = $timelineUI.lineWidthMultipliers[lineIdx] ?? 1.0;
        return BASE_TRACK_SIZE * viewport.zoom * multiplier;
    }

    function handleLineResizeStart(lineIdx: number, event: MouseEvent) {
        event.stopPropagation();
        event.preventDefault();
        const multiplier = $timelineUI.lineWidthMultipliers[lineIdx] ?? 1.0;
        lineResizing = {
            lineIdx,
            startPos: isVertical ? event.clientX : event.clientY,
            startMultiplier: multiplier,
        };
        window.addEventListener("mousemove", handleLineResizeMove);
        window.addEventListener("mouseup", handleLineResizeEnd);
    }

    function handleLineResizeMove(event: MouseEvent) {
        if (!lineResizing) return;
        const currentPos = isVertical ? event.clientX : event.clientY;
        const delta = currentPos - lineResizing.startPos;
        const baseSize = BASE_TRACK_SIZE * viewport.zoom;
        const deltaMultiplier = delta / baseSize;
        const newMultiplier = Math.max(
            LINE_WIDTH_MIN,
            Math.min(LINE_WIDTH_MAX, lineResizing.startMultiplier + deltaMultiplier)
        );
        timelineUI.setLineWidth(lineResizing.lineIdx, newMultiplier);
    }

    function handleLineResizeEnd() {
        window.removeEventListener("mousemove", handleLineResizeMove);
        window.removeEventListener("mouseup", handleLineResizeEnd);
        lineResizing = null;
    }

    // Visible beat markers
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

    const timelineExtent = $derived(scrollPos + viewportSize * 2);

    function handleScroll() {
        if (!timelineContainer) return;
        scrollPos = isVertical
            ? timelineContainer.scrollTop
            : timelineContainer.scrollLeft;
        viewportSize = isVertical
            ? timelineContainer.clientHeight
            : timelineContainer.clientWidth;
    }

    function getMarkerStyle(beat: number): string {
        const pos = beat * pixelsPerBeat;
        return isVertical ? `top: ${pos}px` : `left: ${pos}px`;
    }

    function isFrameSelected(lineIdx: number, frameIdx: number): boolean {
        return isFrameInSelection($selection, $scene, lineIdx, frameIdx);
    }

    function getPlayingFrameIdx(lineIdx: number): number | null {
        return $isPlaying ? ($framePositions[lineIdx]?.[0] ?? null) : null;
    }

    // Wheel zoom handler
    function handleWheel(event: WheelEvent) {
        if (!event.ctrlKey && !event.metaKey) return;
        event.preventDefault();

        const now = Date.now();
        if (now - lastZoomTime < ZOOM_THROTTLE_MS) return;
        lastZoomTime = now;

        const delta = Math.abs(event.deltaY);
        const intensity = Math.min(delta * ZOOM_SENSITIVITY, 0.15);
        const direction = event.deltaY < 0 ? 1 : -1;

        const newZoom = Math.max(
            minZoom,
            Math.min(maxZoom, viewport.zoom * (1 + direction * intensity))
        );
        onZoomChange(newZoom);
    }

    // Clip interaction handlers
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
        const frame = await invoke<Frame>("create_default_frame");
        await addFrame(lineIdx, newFrameIdx, frame);
        selectFrame(lineIdx, newFrameIdx);
    }

    async function handleAddLine() {
        if (!$scene) return;
        const newLineIdx = $scene.lines.length;
        const line = await invoke<Line>("create_default_line");
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

    // Re-evaluate scene
    async function reEvaluateScene() {
        if (!$scene) return;
        const updates: [number, number, Frame][] = [];
        for (let l = 0; l < $scene.lines.length; l++) {
            for (let f = 0; f < $scene.lines[l].frames.length; f++) {
                const frame = $scene.lines[l].frames[f];
                updates.push([l, f, { ...frame }]);
            }
        }
        if (updates.length > 0) {
            try {
                await setFrames(updates, ActionTiming.immediate());
            } catch (error) {
                console.error("Failed to re-evaluate scene:", error);
            }
        }
    }

    // Drag handlers
    function handleDragMove(event: MouseEvent) {
        if (!ctx.dragging || !timelineContainer || !$scene) return;

        const rect = timelineContainer.getBoundingClientRect();
        const scrollX = timelineContainer.scrollLeft;
        const scrollY = timelineContainer.scrollTop;
        const x = event.clientX - rect.left + scrollX;
        const y = event.clientY - rect.top + scrollY;

        const { lineIdx, frameIdx } = calculateDropPosition(x, y);
        ctx.dragging = {
            ...ctx.dragging,
            currentLineIdx: lineIdx,
            currentFrameIdx: frameIdx,
        };
    }

    async function handleDragEnd() {
        window.removeEventListener("mousemove", handleDragMove);
        window.removeEventListener("mouseup", handleDragEnd);

        if (!ctx.dragging || !$scene) {
            ctx.dragging = null;
            return;
        }

        const {
            sourceLineIdx,
            sourceFrameIdx,
            frame,
            currentLineIdx,
            currentFrameIdx,
        } = ctx.dragging;

        const samePosition =
            sourceLineIdx === currentLineIdx &&
            (sourceFrameIdx === currentFrameIdx ||
                sourceFrameIdx === currentFrameIdx - 1);

        if (!samePosition) {
            let targetIdx = currentFrameIdx;

            if (sourceLineIdx === currentLineIdx && sourceFrameIdx < currentFrameIdx) {
                targetIdx--;
            }

            await removeFrame(sourceLineIdx, sourceFrameIdx);
            await addFrame(currentLineIdx, targetIdx, frame);
            selectFrame(currentLineIdx, targetIdx);
        }

        ctx.dragging = null;
    }

    function calculateDropPosition(mouseX: number, mouseY: number): { lineIdx: number; frameIdx: number } {
        if (!$scene || $scene.lines.length === 0) return { lineIdx: 0, frameIdx: 0 };

        const HEADER_SIZE = 70;
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
            const duration = typeof frame.duration === "number" && frame.duration > 0 ? frame.duration : 1;
            const reps = typeof frame.repetitions === "number" && frame.repetitions >= 1 ? frame.repetitions : 1;
            const framePixels = duration * reps * pixelsPerBeat;

            if (pixelPos < accumulatedPixels + framePixels / 2) {
                return f;
            }
            accumulatedPixels += framePixels;
        }

        return line.frames.length;
    }

    // Setup drag listeners when dragging starts
    $effect(() => {
        if (ctx.dragging) {
            window.addEventListener("mousemove", handleDragMove);
            window.addEventListener("mouseup", handleDragEnd);
            return () => {
                window.removeEventListener("mousemove", handleDragMove);
                window.removeEventListener("mouseup", handleDragEnd);
            };
        }
    });
</script>

<div
    class="timeline-pane"
    class:vertical={isVertical}
    bind:this={timelineContainer}
    tabindex="0"
    onkeydown={keyboard.handleKeydown}
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
            style={isVertical
                ? `min-height: ${timelineExtent}px`
                : `min-width: ${timelineExtent}px`}
        >
            <!-- Ruler row -->
            <div
                class="timeline-row ruler-row"
                class:vertical={isVertical}
                style={isVertical
                    ? `width: ${RULER_SIZE}px`
                    : `height: ${RULER_SIZE}px`}
            >
                <div class="ruler-header" class:vertical={isVertical}>
                    <button
                        class="re-eval-btn"
                        onclick={reEvaluateScene}
                        title="Re-evaluate all frames"
                    >
                        <RefreshCw size={12} />
                    </button>
                </div>
                <div class="ruler-content">
                    {#each visibleBeatMarkers as beat (beat)}
                        <div
                            class="beat-marker"
                            class:vertical={isVertical}
                            style={getMarkerStyle(beat)}
                        >
                            {beat}
                        </div>
                    {/each}
                </div>
            </div>

            <!-- Tracks -->
            {#each $scene.lines as line, lineIdx (lineIdx)}
                <Track
                    {line}
                    {lineIdx}
                    {visibleBeatMarkers}
                    trackWidth={getLineWidth(lineIdx)}
                    onRemoveTrack={(e) => handleRemoveLine(lineIdx, e)}
                    onAddClip={() => handleAddFrame(lineIdx)}
                    onClipClick={(frameIdx, e) => handleClipClick(lineIdx, frameIdx, e)}
                    onClipDoubleClick={(frameIdx) => handleClipDoubleClick(lineIdx, frameIdx)}
                    onLineResizeStart={(e) => handleLineResizeStart(lineIdx, e)}
                    isFrameSelected={(frameIdx) => isFrameSelected(lineIdx, frameIdx)}
                    playingFrameIdx={getPlayingFrameIdx(lineIdx)}
                    onSolo={() => soloMute.toggleSolo(lineIdx)}
                    onMute={() => soloMute.toggleMute(lineIdx)}
                    isSolo={soloMute.isSolo(lineIdx)}
                    isMuted={soloMute.isMuted(lineIdx)}
                    onToggleEnabled={(frameIdx) => keyboard.toggleEnabled(lineIdx, frameIdx)}
                />
            {/each}

            <!-- Add track row -->
            <div class="timeline-row add-track-row" class:vertical={isVertical}>
                <button
                    class="add-track"
                    class:vertical={isVertical}
                    onclick={handleAddLine}
                >
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
        user-select: none;
        -webkit-user-select: none;
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
        display: flex;
        align-items: center;
        justify-content: center;
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

    .re-eval-btn {
        background: none;
        border: 1px solid var(--colors-border);
        color: var(--colors-text-secondary);
        padding: 4px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        opacity: 0.5;
    }

    .re-eval-btn:hover {
        opacity: 1;
        border-color: var(--colors-accent);
        color: var(--colors-accent);
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
