<script lang="ts">
    import { Columns2, Rows2, X, LayoutGrid, RotateCcw } from "lucide-svelte";
    import {
        paneLayout,
        paneDragState,
        activePaneId,
        type ViewType,
    } from "$lib/stores/paneState";
    import ViewSelector from "./ViewSelector.svelte";
    import Login from "../Login.svelte";
    import DevicesView from "../DevicesView.svelte";
    import LogView from "../LogView.svelte";
    import SceneView from "../SceneView.svelte";
    import ChatView from "../ChatView.svelte";
    import ProjectsView from "../ProjectsView.svelte";
    import EditorView from "../EditorView.svelte";
    import SettingsPanel from "../SettingsPanel.svelte";
    import type { Snippet } from "svelte";

    interface Props {
        paneId: string;
        viewType: ViewType | null;
        isOnlyPane: boolean;
    }

    let { paneId, viewType, isOnlyPane }: Props = $props();

    let toolbarControls: Snippet | null = $state(null);
    let headerElement: HTMLDivElement | null = $state(null);
    let isDropTarget = $state(false);

    // Check if this pane is being dragged
    let isDragSource = $derived($paneDragState?.paneId === paneId);

    function handlePaneClick() {
        activePaneId.set(paneId);
    }

    const viewTitles: Record<ViewType, string> = {
        LOGIN: "Login",
        SCENE: "Scene",
        DEVICES: "Devices",
        LOGS: "Logs",
        CHAT: "Chat",
        PROJECTS: "Projects",
        EDITOR: "Editor",
        CONFIG: "Config",
    };

    const viewHelpIds: Record<ViewType, string> = {
        LOGIN: "zone-login",
        SCENE: "zone-scene",
        DEVICES: "zone-devices",
        LOGS: "zone-logs",
        CHAT: "zone-chat",
        PROJECTS: "zone-projects",
        EDITOR: "zone-editor",
        CONFIG: "zone-config",
    };

    const zoneHelpId = $derived(viewType ? viewHelpIds[viewType] : null);

    function handleSplitHorizontal() {
        paneLayout.splitPane(paneId, "horizontal");
    }

    function handleSplitVertical() {
        paneLayout.splitPane(paneId, "vertical");
    }

    function handleClose() {
        paneLayout.closePane(paneId);
    }

    function handleClearView() {
        paneLayout.setView(paneId, null);
    }

    function handleToggleDirection() {
        paneLayout.toggleParentDirection(paneId);
    }

    function handleViewSelect(view: ViewType) {
        paneLayout.setView(paneId, view);
    }

    function handleLoginSuccess() {
        paneLayout.setView(paneId, "SCENE");
    }

    function registerToolbar(snippet: Snippet | null) {
        toolbarControls = snippet;
    }

    function handleDragStart(event: MouseEvent) {
        // Only start drag on left mouse button
        if (event.button !== 0) return;
        // Don't start drag if clicking on buttons
        if ((event.target as HTMLElement).closest("button")) return;

        paneDragState.start(paneId, viewType);

        window.addEventListener("mousemove", handleDragMove);
        window.addEventListener("mouseup", handleDragEnd);
    }

    function handleDragMove(event: MouseEvent) {
        // Dispatch custom event with mouse position for other panes to check
        window.dispatchEvent(
            new CustomEvent("pane-drag-move", {
                detail: { x: event.clientX, y: event.clientY },
            }),
        );
    }

    function handleDragEnd(event: MouseEvent) {
        window.removeEventListener("mousemove", handleDragMove);
        window.removeEventListener("mouseup", handleDragEnd);

        // Find which pane header we're over
        const dropTarget = document
            .elementFromPoint(event.clientX, event.clientY)
            ?.closest("[data-pane-id]") as HTMLElement | null;
        const targetPaneId = dropTarget?.dataset.paneId;

        if (targetPaneId && targetPaneId !== paneId && $paneDragState) {
            paneLayout.swapViews($paneDragState.paneId, targetPaneId);
        }

        paneDragState.clear();
        window.dispatchEvent(new CustomEvent("pane-drag-end"));
    }

    function handlePaneDragMove(event: CustomEvent<{ x: number; y: number }>) {
        if (
            !$paneDragState ||
            $paneDragState.paneId === paneId ||
            !headerElement
        ) {
            isDropTarget = false;
            return;
        }

        const rect = headerElement.getBoundingClientRect();
        const { x, y } = event.detail;
        isDropTarget =
            x >= rect.left &&
            x <= rect.right &&
            y >= rect.top &&
            y <= rect.bottom;
    }

    function handlePaneDragEnd() {
        isDropTarget = false;
    }

    $effect(() => {
        window.addEventListener(
            "pane-drag-move",
            handlePaneDragMove as EventListener,
        );
        window.addEventListener("pane-drag-end", handlePaneDragEnd);

        return () => {
            window.removeEventListener(
                "pane-drag-move",
                handlePaneDragMove as EventListener,
            );
            window.removeEventListener("pane-drag-end", handlePaneDragEnd);
        };
    });
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="pane-container" onclick={handlePaneClick}>
    <div
        class="pane-header"
        class:drag-source={isDragSource}
        class:drop-target={isDropTarget}
        bind:this={headerElement}
        data-pane-id={paneId}
        onmousedown={handleDragStart}
        role="button"
        tabindex="0"
    >
        <span class="pane-title" data-help-id={zoneHelpId}>
            {viewType ? viewTitles[viewType] : "Select View"}
        </span>
        <div class="pane-controls">
            {#if toolbarControls}
                {@render toolbarControls()}
                <div class="separator"></div>
            {/if}
            <div class="pane-actions">
                {#if viewType !== null}
                    <button
                        class="action-btn"
                        data-help-id="pane-change-view"
                        onclick={handleClearView}
                        title="Change View"
                    >
                        <LayoutGrid size={14} />
                    </button>
                {/if}
                <button
                    class="action-btn"
                    data-help-id="pane-split-vertical"
                    onclick={handleSplitVertical}
                    title="Split Vertical"
                >
                    <Columns2 size={14} />
                </button>
                <button
                    class="action-btn"
                    data-help-id="pane-split-horizontal"
                    onclick={handleSplitHorizontal}
                    title="Split Horizontal"
                >
                    <Rows2 size={14} />
                </button>
                {#if !isOnlyPane}
                    <button
                        class="action-btn"
                        data-help-id="pane-toggle-direction"
                        onclick={handleToggleDirection}
                        title="Toggle Split Direction"
                    >
                        <RotateCcw size={14} />
                    </button>
                    <button
                        class="action-btn close"
                        data-help-id="pane-close"
                        onclick={handleClose}
                        title="Close Pane"
                    >
                        <X size={14} />
                    </button>
                {/if}
            </div>
        </div>
    </div>

    <div class="pane-content">
        {#if viewType === null}
            <ViewSelector onSelect={handleViewSelect} />
        {:else if viewType === "LOGIN"}
            <Login onConnected={handleLoginSuccess} />
        {:else if viewType === "SCENE"}
            <SceneView {registerToolbar} />
        {:else if viewType === "DEVICES"}
            <DevicesView />
        {:else if viewType === "LOGS"}
            <LogView />
        {:else if viewType === "CHAT"}
            <ChatView />
        {:else if viewType === "PROJECTS"}
            <ProjectsView />
        {:else if viewType === "EDITOR"}
            <EditorView />
        {:else if viewType === "CONFIG"}
            <SettingsPanel />
        {/if}
    </div>
</div>

<style>
    .pane-container {
        width: 100%;
        height: 100%;
        display: flex;
        flex-direction: column;
        background-color: var(--colors-background);
    }

    .pane-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        height: 28px;
        padding: 0 8px;
        background-color: var(--colors-surface);
        border-bottom: 1px solid var(--colors-border);
        flex-shrink: 0;
        cursor: grab;
    }

    .pane-header:active {
        cursor: grabbing;
    }

    .pane-header.drag-source {
        opacity: 0.5;
    }

    .pane-header.drop-target {
        background-color: var(--colors-accent);
        border-bottom-color: var(--colors-accent);
    }

    .pane-title {
        font-family: monospace;
        font-size: 11px;
        font-weight: 600;
        color: var(--colors-text-secondary);
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .pane-controls {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .separator {
        width: 1px;
        height: 16px;
        background-color: var(--colors-border);
    }

    .pane-actions {
        display: flex;
        gap: 2px;
    }

    .action-btn {
        background: none;
        border: none;
        color: var(--colors-text-secondary);
        padding: 4px;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .action-btn:hover {
        color: var(--colors-text);
    }

    .action-btn.close:hover {
        color: var(--colors-danger, #f87171);
    }

    .pane-content {
        flex: 1;
        overflow: hidden;
    }
</style>
