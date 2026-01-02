<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import type { Snippet } from "svelte";
    import {
        Rows3,
        Columns3,
        ArrowLeftRight,
        ArrowUpDown,
        ZoomIn,
        ZoomOut,
        RotateCcw,
    } from "lucide-svelte";
    import SplitPane from "./SplitPane.svelte";
    import Timeline from "./scene/Timeline.svelte";
    import FrameEditor from "./scene/FrameEditor.svelte";
    import { snapGranularity, SNAP_OPTIONS } from "$lib/stores/snapGranularity";
    import {
        editingFrame,
        currentEditingFrame,
        editingFrameKey,
        openEditor,
        closeEditor,
        initEditingFrameListeners,
        cleanupEditingFrameListeners,
    } from "$lib/stores/editingFrame";
    import { isEditorPaneOpen } from "$lib/stores/paneState";

    const TIMELINE_ORIENTATION_KEY = "sova-timeline-orientation";

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

    function saveTimelineOrientation(
        orientation: "horizontal" | "vertical",
    ): void {
        try {
            localStorage.setItem(TIMELINE_ORIENTATION_KEY, orientation);
        } catch {
            // Storage unavailable
        }
    }

    interface Props {
        registerToolbar?: (_snippet: Snippet | null) => void;
    }

    let { registerToolbar }: Props = $props();

    // Zoom constraints
    const MIN_ZOOM = 0.25;
    const MAX_ZOOM = 4.0;
    const ZOOM_FACTOR = 1.05;

    // Viewport state
    let viewport = $state({
        zoom: 1.0,
        orientation: loadTimelineOrientation(),
    });

    // Layout state - responsive split direction
    let containerEl = $state<HTMLDivElement | null>(null);
    let containerSize = $state({ width: 0, height: 0 });
    let userOverride = $state(false);
    let userOrientation = $state<"horizontal" | "vertical">("vertical");

    const optimalOrientation = $derived(
        containerSize.width > containerSize.height ? "vertical" : "horizontal",
    );

    const splitOrientation = $derived(
        userOverride ? userOrientation : optimalOrientation,
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
        const newOrientation =
            viewport.orientation === "horizontal" ? "vertical" : "horizontal";
        viewport.orientation = newOrientation;
        saveTimelineOrientation(newOrientation);
    }

    function toggleSplitOrientation() {
        userOverride = true;
        userOrientation =
            userOrientation === "horizontal" ? "vertical" : "horizontal";
    }

    function handleOpenEditor(lineIdx: number, frameIdx: number) {
        openEditor(lineIdx, frameIdx);
    }

    function handleCloseEditor() {
        closeEditor();
        userOverride = false;
    }

    // Handle zoom command from command palette
    function handleSetZoom(event: Event) {
        const zoom = (event as CustomEvent<number>).detail;
        if (zoom >= MIN_ZOOM && zoom <= MAX_ZOOM) {
            viewport.zoom = zoom;
        }
    }

    // Register/unregister toolbar
    $effect(() => {
        registerToolbar?.(toolbarSnippet);
        return () => registerToolbar?.(null);
    });

    // ResizeObserver for container dimensions
    $effect(() => {
        if (!containerEl) return;
        const observer = new ResizeObserver((entries) => {
            const entry = entries[0];
            if (entry) {
                containerSize = {
                    width: entry.contentRect.width,
                    height: entry.contentRect.height,
                };
            }
        });
        observer.observe(containerEl);
        return () => observer.disconnect();
    });

    // Window event listeners
    $effect(() => {
        window.addEventListener("command:set-zoom", handleSetZoom);
        return () => {
            window.removeEventListener("command:set-zoom", handleSetZoom);
        };
    });

    onMount(() => {
        initEditingFrameListeners();
    });

    onDestroy(() => {
        cleanupEditingFrameListeners();
    });
</script>

{#snippet toolbarSnippet()}
    <div class="toolbar-controls">
        <div class="zoom-controls">
            <button
                class="toolbar-btn"
                data-help-id="scene-zoom-out"
                onclick={zoomOut}
                title="Zoom out"
                disabled={viewport.zoom <= MIN_ZOOM}
            >
                <ZoomOut size={14} />
            </button>
            <span class="zoom-level">{Math.round(viewport.zoom * 100)}%</span>
            <button
                class="toolbar-btn"
                data-help-id="scene-zoom-in"
                onclick={zoomIn}
                title="Zoom in"
                disabled={viewport.zoom >= MAX_ZOOM}
            >
                <ZoomIn size={14} />
            </button>
            {#if viewport.zoom !== 1.0}
                <button
                    class="toolbar-btn"
                    data-help-id="scene-zoom-reset"
                    onclick={resetZoom}
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
                    onclick={() => snapGranularity.set(opt.value)}
                    title="Snap to {opt.label} beat"
                >
                    {opt.label}
                </button>
            {/each}
        </div>
        <button
            class="toolbar-btn"
            data-help-id="scene-timeline-orientation"
            onclick={toggleTimelineOrientation}
            title="Toggle timeline orientation"
        >
            {#if viewport.orientation === "horizontal"}
                <Columns3 size={14} />
            {:else}
                <Rows3 size={14} />
            {/if}
        </button>
        {#if !$isEditorPaneOpen}
            <button
                class="toolbar-btn"
                data-help-id="scene-split-orientation"
                onclick={toggleSplitOrientation}
                title="Toggle split orientation"
            >
                {#if splitOrientation === "horizontal"}
                    <ArrowUpDown size={14} />
                {:else}
                    <ArrowLeftRight size={14} />
                {/if}
            </button>
        {/if}
    </div>
{/snippet}

<div class="scene-container">
    <div class="split-container" bind:this={containerEl}>
        {#if $editingFrame && !$isEditorPaneOpen}
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
                        frame={$currentEditingFrame}
                        frameKey={$editingFrameKey}
                        lineIdx={$editingFrame.lineIdx}
                        frameIdx={$editingFrame.frameIdx}
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
