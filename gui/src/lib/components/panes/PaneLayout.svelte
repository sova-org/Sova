<script lang="ts">
    import {
        paneLayout,
        paneDragState,
        type ViewType,
    } from "$lib/stores/paneState";
    import PaneNodeRenderer from "./PaneNodeRenderer.svelte";

    let isOnlyPane = $derived($paneLayout.root.type === "leaf");

    let mouseX = $state(0);
    let mouseY = $state(0);

    const viewTitles: Record<ViewType, string> = {
        LOGIN: "Login",
        SCENE: "Scene",
        DEVICES: "Devices",
        LOGS: "Logs",
        CONFIG: "Config",
        CHAT: "Chat",
        SNAPSHOTS: "Snapshots",
    };

    function handleDragMove(event: CustomEvent<{ x: number; y: number }>) {
        mouseX = event.detail.x;
        mouseY = event.detail.y;
    }

    $effect(() => {
        window.addEventListener(
            "pane-drag-move",
            handleDragMove as EventListener,
        );
        return () => {
            window.removeEventListener(
                "pane-drag-move",
                handleDragMove as EventListener,
            );
        };
    });
</script>

<div class="pane-layout">
    <PaneNodeRenderer node={$paneLayout.root} {isOnlyPane} />
</div>

{#if $paneDragState}
    <div
        class="drag-preview"
        style="left: {mouseX + 12}px; top: {mouseY + 12}px;"
    >
        {$paneDragState.viewType
            ? viewTitles[$paneDragState.viewType]
            : "Empty"}
    </div>
{/if}

<style>
    .pane-layout {
        width: 100%;
        height: 100%;
        overflow: hidden;
    }

    .drag-preview {
        position: fixed;
        pointer-events: none;
        z-index: 1000;
        padding: 4px 8px;
        background-color: var(--colors-surface);
        border: 1px solid var(--colors-accent);
        font-family: monospace;
        font-size: 11px;
        font-weight: 600;
        color: var(--colors-text);
        text-transform: uppercase;
        letter-spacing: 0.5px;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    }

    .pane-layout :global(.splitpanes) {
        background-color: var(--colors-background);
    }

    .pane-layout :global(.splitpanes__splitter) {
        background-color: var(--colors-border);
        position: relative;
    }

    .pane-layout :global(.splitpanes__splitter:before) {
        content: "";
        position: absolute;
        left: 0;
        top: 0;
        transition: opacity 0.2s;
        background-color: var(--colors-accent);
        opacity: 0;
        z-index: 1;
    }

    .pane-layout :global(.splitpanes__splitter:hover:before) {
        opacity: 0.5;
    }

    .pane-layout :global(.splitpanes--vertical > .splitpanes__splitter) {
        width: 4px;
        min-width: 4px;
    }

    .pane-layout :global(.splitpanes--vertical > .splitpanes__splitter:before) {
        left: -2px;
        right: -2px;
        height: 100%;
        width: auto;
    }

    .pane-layout :global(.splitpanes--horizontal > .splitpanes__splitter) {
        height: 4px;
        min-height: 4px;
    }

    .pane-layout
        :global(.splitpanes--horizontal > .splitpanes__splitter:before) {
        top: -2px;
        bottom: -2px;
        width: 100%;
        height: auto;
    }

    .pane-layout :global(.splitpanes__pane) {
        overflow: hidden;
    }
</style>
