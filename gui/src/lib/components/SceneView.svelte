<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import {
        Rows3,
        Columns3,
        ZoomIn,
        ZoomOut,
        RotateCcw,
        ChevronDown,
        ChevronUp,
    } from "lucide-svelte";
    import Timeline from "./scene/Timeline.svelte";
    import FrameEditor from "./scene/FrameEditor.svelte";
    import { snapGranularity, SNAP_OPTIONS } from "$lib/stores/snapGranularity";
    import {
        editingFrame,
        currentEditingFrame,
        editingFrameKey,
        openEditor,
        initEditingFrameListeners,
        cleanupEditingFrameListeners,
    } from "$lib/stores/editingFrame";

    const TIMELINE_ORIENTATION_KEY = "sova-timeline-orientation";
    const TIMELINE_EXPANDED_KEY = "sova-timeline-expanded";
    const TIMELINE_SIZE_KEY = "sova-timeline-size";

    const MIN_TIMELINE_SIZE = 100;
    const MAX_TIMELINE_SIZE = 600;
    const DEFAULT_TIMELINE_SIZE = 250;

    function loadTimelineOrientation(): "horizontal" | "vertical" {
        try {
            const stored = localStorage.getItem(TIMELINE_ORIENTATION_KEY);
            if (stored === "horizontal" || stored === "vertical") {
                return stored;
            }
        } catch {
            // Storage unavailable
        }
        return "horizontal";
    }

    function saveTimelineOrientation(orientation: "horizontal" | "vertical"): void {
        try {
            localStorage.setItem(TIMELINE_ORIENTATION_KEY, orientation);
        } catch {
            // Storage unavailable
        }
    }

    function loadTimelineExpanded(): boolean {
        try {
            const stored = localStorage.getItem(TIMELINE_EXPANDED_KEY);
            return stored === "true";
        } catch {
            // Storage unavailable
        }
        return false;
    }

    function saveTimelineExpanded(expanded: boolean): void {
        try {
            localStorage.setItem(TIMELINE_EXPANDED_KEY, expanded.toString());
        } catch {
            // Storage unavailable
        }
    }

    function loadTimelineSize(): number {
        try {
            const stored = localStorage.getItem(TIMELINE_SIZE_KEY);
            if (stored) {
                const size = parseInt(stored, 10);
                if (!isNaN(size) && size >= MIN_TIMELINE_SIZE && size <= MAX_TIMELINE_SIZE) {
                    return size;
                }
            }
        } catch {
            // Storage unavailable
        }
        return DEFAULT_TIMELINE_SIZE;
    }

    function saveTimelineSize(size: number): void {
        try {
            localStorage.setItem(TIMELINE_SIZE_KEY, size.toString());
        } catch {
            // Storage unavailable
        }
    }

    const MIN_ZOOM = 0.25;
    const MAX_ZOOM = 4.0;
    const ZOOM_FACTOR = 1.05;

    let viewport = $state({
        zoom: 1.0,
        orientation: loadTimelineOrientation(),
    });

    let timelineExpanded = $state(loadTimelineExpanded());
    let timelineSize = $state(loadTimelineSize());
    let isResizing = $state(false);
    let resizeStart = 0;
    let resizeStartSize = 0;

    const isVertical = $derived(viewport.orientation === "vertical");

    function toggleTimeline() {
        timelineExpanded = !timelineExpanded;
        saveTimelineExpanded(timelineExpanded);
    }

    function startResize(e: MouseEvent) {
        isResizing = true;
        resizeStart = isVertical ? e.clientX : e.clientY;
        resizeStartSize = timelineSize;
        document.addEventListener("mousemove", handleResize);
        document.addEventListener("mouseup", stopResize);
        document.body.style.cursor = isVertical ? "col-resize" : "row-resize";
        document.body.style.userSelect = "none";
    }

    function handleResize(e: MouseEvent) {
        if (!isResizing) return;
        const current = isVertical ? e.clientX : e.clientY;
        const delta = current - resizeStart;
        const newSize = Math.max(MIN_TIMELINE_SIZE, Math.min(MAX_TIMELINE_SIZE, resizeStartSize + delta));
        timelineSize = newSize;
    }

    function stopResize() {
        isResizing = false;
        document.removeEventListener("mousemove", handleResize);
        document.removeEventListener("mouseup", stopResize);
        document.body.style.cursor = "";
        document.body.style.userSelect = "";
        saveTimelineSize(timelineSize);
    }

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
        const newOrientation =
            viewport.orientation === "horizontal" ? "vertical" : "horizontal";
        viewport.orientation = newOrientation;
        saveTimelineOrientation(newOrientation);
    }

    function handleOpenEditor(lineIdx: number, frameIdx: number) {
        openEditor(lineIdx, frameIdx);
    }

    function handleSetZoom(event: Event) {
        const zoom = (event as CustomEvent<number>).detail;
        if (zoom >= MIN_ZOOM && zoom <= MAX_ZOOM) {
            viewport.zoom = zoom;
        }
    }

    function handleKeydown(e: KeyboardEvent) {
        if (e.key === "Tab" && !e.ctrlKey && !e.metaKey && !e.altKey) {
            // Only toggle if not in an input/textarea
            const target = e.target as HTMLElement;
            if (target.tagName !== "INPUT" && target.tagName !== "TEXTAREA" && !target.isContentEditable) {
                e.preventDefault();
                toggleTimeline();
            }
        }
    }

    $effect(() => {
        window.addEventListener("command:set-zoom", handleSetZoom);
        window.addEventListener("keydown", handleKeydown);
        return () => {
            window.removeEventListener("command:set-zoom", handleSetZoom);
            window.removeEventListener("keydown", handleKeydown);
        };
    });

    onMount(() => {
        initEditingFrameListeners();
    });

    onDestroy(() => {
        cleanupEditingFrameListeners();
    });
</script>

<div class="scene-container">
    <!-- Timeline header (always visible, always horizontal) -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div class="timeline-header" onclick={toggleTimeline}>
        <button class="toggle-btn" title="Toggle timeline (Tab)">
            {#if timelineExpanded}
                <ChevronUp size={14} />
            {:else}
                <ChevronDown size={14} />
            {/if}
        </button>
        <span class="header-label">Timeline</span>

        <div class="toolbar-controls">
            <div class="zoom-controls">
                <button
                    class="toolbar-btn"
                    data-help-id="scene-zoom-out"
                    onclick={(e) => { e.stopPropagation(); zoomOut(); }}
                    title="Zoom out"
                    disabled={viewport.zoom <= MIN_ZOOM}
                >
                    <ZoomOut size={14} />
                </button>
                <span class="zoom-level">{Math.round(viewport.zoom * 100)}%</span>
                <button
                    class="toolbar-btn"
                    data-help-id="scene-zoom-in"
                    onclick={(e) => { e.stopPropagation(); zoomIn(); }}
                    title="Zoom in"
                    disabled={viewport.zoom >= MAX_ZOOM}
                >
                    <ZoomIn size={14} />
                </button>
                {#if viewport.zoom !== 1.0}
                    <button
                        class="toolbar-btn"
                        data-help-id="scene-zoom-reset"
                        onclick={(e) => { e.stopPropagation(); resetZoom(); }}
                        title="Reset zoom"
                    >
                        <RotateCcw size={12} />
                    </button>
                {/if}
            </div>
            <div class="snap-controls" data-help-id="scene-snap-granularity">
                {#each SNAP_OPTIONS as opt (opt.value)}
                    <button
                        class="snap-btn"
                        class:active={$snapGranularity === opt.value}
                        onclick={(e) => { e.stopPropagation(); snapGranularity.set(opt.value); }}
                        title="Snap to {opt.label} beat"
                    >
                        {opt.label}
                    </button>
                {/each}
            </div>
            <button
                class="toolbar-btn"
                data-help-id="scene-timeline-orientation"
                onclick={(e) => { e.stopPropagation(); toggleTimelineOrientation(); }}
                title="Toggle timeline orientation"
            >
                {#if viewport.orientation === "horizontal"}
                    <Columns3 size={14} />
                {:else}
                    <Rows3 size={14} />
                {/if}
            </button>
        </div>
    </div>

    <!-- Main content area -->
    <div class="main-content" class:vertical={isVertical}>
        <!-- Timeline panel (collapsible) -->
        {#if timelineExpanded}
            <div
                class="timeline-panel"
                style="{isVertical ? 'width' : 'height'}: {timelineSize}px"
            >
                <Timeline
                    {viewport}
                    editorPosition="right"
                    minZoom={MIN_ZOOM}
                    maxZoom={MAX_ZOOM}
                    zoomFactor={ZOOM_FACTOR}
                    onZoomChange={handleZoomChange}
                    onOpenEditor={handleOpenEditor}
                />
            </div>
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div
                class="resizer"
                class:horizontal={!isVertical}
                class:vertical={isVertical}
                class:active={isResizing}
                onmousedown={startResize}
            ></div>
        {/if}

        <!-- Editor panel (always visible) -->
        <div class="editor-panel">
            <FrameEditor
                frame={$currentEditingFrame}
                frameKey={$editingFrameKey}
                lineIdx={$editingFrame?.lineIdx ?? null}
                frameIdx={$editingFrame?.frameIdx ?? null}
            />
        </div>
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

    .timeline-header {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 4px 8px;
        background-color: var(--colors-surface);
        border-bottom: 1px solid var(--colors-border);
        flex-shrink: 0;
        cursor: pointer;
        user-select: none;
    }

    .main-content {
        flex: 1;
        display: flex;
        flex-direction: column;
        min-height: 0;
        min-width: 0;
    }

    .main-content.vertical {
        flex-direction: row;
    }

    .toggle-btn {
        background: none;
        border: none;
        color: var(--colors-text-secondary);
        padding: 2px;
        cursor: pointer;
        display: flex;
        align-items: center;
    }

    .toggle-btn:hover {
        color: var(--colors-accent);
    }

    .header-label {
        font-family: monospace;
        font-size: 11px;
        font-weight: 600;
        color: var(--colors-text-secondary);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .toolbar-controls {
        display: flex;
        align-items: center;
        gap: 8px;
        margin-left: auto;
    }

    .timeline-panel {
        flex-shrink: 0;
        overflow: hidden;
    }

    .editor-panel {
        flex: 1;
        overflow: hidden;
        min-height: 0;
        min-width: 0;
    }

    .resizer {
        flex-shrink: 0;
        background: var(--colors-border);
        transition: background 0.15s;
    }

    .resizer.horizontal {
        height: 4px;
        cursor: row-resize;
    }

    .resizer.vertical {
        width: 4px;
        cursor: col-resize;
    }

    .resizer:hover,
    .resizer.active {
        background: var(--colors-accent);
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

    .snap-controls {
        display: flex;
        align-items: center;
        gap: 0;
    }

    .snap-btn {
        background: none;
        border: 1px solid var(--colors-border);
        color: var(--colors-text-secondary);
        padding: 4px 6px;
        cursor: pointer;
        font-size: 10px;
        font-family: monospace;
        min-width: 28px;
    }

    .snap-btn:not(:first-child) {
        border-left: none;
    }

    .snap-btn:hover {
        border-color: var(--colors-accent);
        color: var(--colors-accent);
    }

    .snap-btn.active {
        background-color: var(--colors-accent);
        border-color: var(--colors-accent);
        color: var(--colors-background);
    }
</style>
