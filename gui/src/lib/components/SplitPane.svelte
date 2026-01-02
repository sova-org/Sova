<script lang="ts">
    import type { Snippet } from "svelte";
    import { onMount } from "svelte";

    interface Props {
        orientation?: "horizontal" | "vertical";
        initialSize?: number;
        minSize?: number;
        first: Snippet;
        second: Snippet;
    }

    let {
        orientation = "horizontal",
        initialSize = 50,
        minSize = 20,
        first,
        second,
    }: Props = $props();

    let splitSize = $state(initialSize);
    let isDragging = $state(false);
    let dragStart: { pos: number; size: number } | null = $state(null);
    let container: HTMLDivElement;

    function handlePointerDown(e: PointerEvent) {
        isDragging = true;
        const pos = orientation === "vertical" ? e.clientX : e.clientY;
        dragStart = { pos, size: splitSize };
        (e.target as HTMLElement).setPointerCapture(e.pointerId);
    }

    function handleMouseMove(e: MouseEvent) {
        if (!isDragging || !dragStart || !container) return;

        const rect = container.getBoundingClientRect();
        const containerSize = orientation === "vertical" ? rect.width : rect.height;
        const currentPos = orientation === "vertical" ? e.clientX : e.clientY;
        const delta = currentPos - dragStart.pos;
        const deltaPct = (delta / containerSize) * 100;
        const newSize = dragStart.size + deltaPct;
        splitSize = Math.max(minSize, Math.min(100 - minSize, newSize));
    }

    function handleMouseUp() {
        isDragging = false;
        dragStart = null;
    }

    onMount(() => {
        document.addEventListener("mousemove", handleMouseMove);
        document.addEventListener("mouseup", handleMouseUp);

        return () => {
            document.removeEventListener("mousemove", handleMouseMove);
            document.removeEventListener("mouseup", handleMouseUp);
        };
    });
</script>

<div
    class="split-pane"
    class:vertical={orientation === "vertical"}
    class:horizontal={orientation === "horizontal"}
    bind:this={container}
>
    <div
        class="pane first"
        style="{orientation === 'vertical' ? 'width' : 'height'}: {splitSize}%"
    >
        {@render first()}
    </div>

    <div class="divider" role="separator" onpointerdown={handlePointerDown}></div>

    <div
        class="pane second"
        style="{orientation === 'vertical' ? 'width' : 'height'}: {100 -
            splitSize}%"
    >
        {@render second()}
    </div>
</div>

<style>
    .split-pane {
        width: 100%;
        height: 100%;
        display: flex;
        position: relative;
        overflow: hidden;
    }

    .split-pane.horizontal {
        flex-direction: column;
    }

    .split-pane.vertical {
        flex-direction: row;
    }

    .pane {
        overflow: hidden;
        position: relative;
    }

    .divider {
        background-color: var(--colors-border);
        flex-shrink: 0;
        cursor: col-resize;
        z-index: 10;
    }

    .horizontal .divider {
        height: 4px;
        width: 100%;
        cursor: row-resize;
    }

    .vertical .divider {
        width: 4px;
        height: 100%;
        cursor: col-resize;
    }

    .divider:hover {
        background-color: var(--colors-accent);
    }
</style>
