<script lang="ts">
    import type { Frame } from "$lib/types/protocol";
    import { getTimelineContext, type EditingField } from "./context.svelte";

    interface Props {
        frame: Frame;
        lineIdx: number;
        frameIdx: number;
        offset: number;
        extent: number;
        trackWidth: number;
        selected: boolean;
        playing: boolean;
        onClick: (_e: MouseEvent) => void;
        onDoubleClick: () => void;
        onToggleEnabled: () => void;
    }

    let {
        frame,
        lineIdx,
        frameIdx,
        offset,
        extent,
        trackWidth,
        selected,
        playing,
        onClick,
        onDoubleClick,
        onToggleEnabled,
    }: Props = $props();

    const ctx = getTimelineContext();

    // Drag initiation with threshold
    const DRAG_THRESHOLD = 5;
    let dragStartPos: { x: number; y: number } | null = null;

    function handleMouseDown(e: MouseEvent) {
        if (e.button !== 0) return;
        dragStartPos = { x: e.clientX, y: e.clientY };
        window.addEventListener("mousemove", handleDragCheck);
        window.addEventListener("mouseup", handleDragCancel);
    }

    function handleDragCheck(e: MouseEvent) {
        if (!dragStartPos) return;
        const dx = Math.abs(e.clientX - dragStartPos.x);
        const dy = Math.abs(e.clientY - dragStartPos.y);
        if (dx > DRAG_THRESHOLD || dy > DRAG_THRESHOLD) {
            cleanupDragListeners();
            ctx.startDrag(lineIdx, frameIdx);
        }
    }

    function handleDragCancel() {
        cleanupDragListeners();
    }

    function cleanupDragListeners() {
        dragStartPos = null;
        window.removeEventListener("mousemove", handleDragCheck);
        window.removeEventListener("mouseup", handleDragCancel);
    }

    // Cleanup drag listeners when component is destroyed
    $effect(() => {
        return () => cleanupDragListeners();
    });

    // Derived editing states from context
    const isEditingDuration = $derived(
        ctx.editing?.field === 'duration' &&
        ctx.editing?.lineIdx === lineIdx &&
        ctx.editing?.frameIdx === frameIdx
    );
    const isEditingReps = $derived(
        ctx.editing?.field === 'reps' &&
        ctx.editing?.lineIdx === lineIdx &&
        ctx.editing?.frameIdx === frameIdx
    );
    const isEditingName = $derived(
        ctx.editing?.field === 'name' &&
        ctx.editing?.lineIdx === lineIdx &&
        ctx.editing?.frameIdx === frameIdx
    );

    // Pure derived values
    const duration = $derived(
        (typeof frame.duration === "number" && !isNaN(frame.duration) && frame.duration > 0)
            ? frame.duration
            : 1
    );

    const reps = $derived(
        (typeof frame.repetitions === "number" && !isNaN(frame.repetitions) && frame.repetitions >= 1)
            ? frame.repetitions
            : 1
    );

    const clipLabel = $derived(frame.name || `F${frameIdx}`);
    const clipLang = $derived(frame.script?.lang || "bali");
    const formattedDuration = $derived(`${duration}`);
    const formattedReps = $derived(`×${reps}`);

    const clipWidth = $derived(ctx.isVertical ? trackWidth - 8 : extent);
    const isCompact = $derived(clipWidth < 80);
    const showLangCompact = $derived(clipWidth >= 50);
    const showRepsCompact = $derived(clipWidth >= 50);

    const clipStyle = $derived.by(() => {
        const clipSize = trackWidth - 8;
        if (ctx.isVertical) {
            return `top: ${offset}px; height: ${extent}px; left: 4px; width: ${clipSize}px`;
        } else {
            return `left: ${offset}px; width: ${extent}px; top: 4px; height: ${clipSize}px`;
        }
    });

    function focusOnMount(node: HTMLInputElement) {
        node.focus();
        node.select();
    }

    function handleEditStart(field: EditingField, e: MouseEvent) {
        e.stopPropagation();
        ctx.startEdit(field, lineIdx, frameIdx);
    }

    function handleEditInput(field: EditingField, e: Event) {
        ctx.updateEditValue(field, (e.target as HTMLInputElement).value);
    }

    function handleEditKeydown(field: EditingField, e: KeyboardEvent) {
        if (e.key === "Enter") {
            e.preventDefault();
            e.stopPropagation();
            ctx.commitEdit(field, e.shiftKey);
        } else if (e.key === "Escape") {
            e.stopPropagation();
            ctx.cancelEdit();
        }
    }

    function handleResizeStart(e: PointerEvent) {
        ctx.startResize(lineIdx, frameIdx, e);
    }
</script>

<div
    class="clip"
    class:selected
    class:playing
    class:compact={isCompact}
    class:disabled={frame.enabled === false}
    data-clip="{lineIdx}-{frameIdx}"
    style={clipStyle}
    onclick={onClick}
    ondblclick={onDoubleClick}
    onmousedown={handleMouseDown}
    onkeydown={(e) => e.key === 'Enter' && onClick(e as unknown as MouseEvent)}
    role="button"
    tabindex="-1"
>
    {#if isCompact}
        <div class="clip-content">
            <div class="compact-header">
                <input
                    type="checkbox"
                    class="enable-checkbox"
                    checked={frame.enabled !== false}
                    onclick={(e) => {
                        e.stopPropagation();
                        onToggleEnabled();
                    }}
                    title="Enable/disable (e)"
                />
                {#if showLangCompact}
                    <span class="clip-lang">{clipLang}</span>
                {/if}
            </div>
            {#if isEditingDuration}
                <input
                    class="info-input"
                    type="text"
                    value={ctx.editing?.value ?? ""}
                    oninput={(e) => handleEditInput("duration", e)}
                    onkeydown={(e) => handleEditKeydown("duration", e)}
                    onblur={() => ctx.cancelEdit()}
                    use:focusOnMount
                />
            {:else}
                <span
                    class="clip-info"
                    ondblclick={(e) => handleEditStart("duration", e)}
                    role="button"
                    tabindex="-1"
                    title="Duration (double-click to edit)"
                    >{formattedDuration}</span
                >
            {/if}
            {#if showRepsCompact}
                {#if isEditingReps}
                    <input
                        class="info-input"
                        type="text"
                        value={ctx.editing?.value ?? ""}
                        oninput={(e) => handleEditInput("reps", e)}
                        onkeydown={(e) => handleEditKeydown("reps", e)}
                        onblur={() => ctx.cancelEdit()}
                        use:focusOnMount
                    />
                {:else}
                    <span
                        class="clip-info"
                        ondblclick={(e) => handleEditStart("reps", e)}
                        role="button"
                        tabindex="-1"
                        title="Repetitions (double-click to edit)"
                        >{formattedReps}</span
                    >
                {/if}
            {/if}
        </div>
    {:else}
        <div class="clip-top">
            <input
                type="checkbox"
                class="enable-checkbox"
                checked={frame.enabled !== false}
                onclick={(e) => {
                    e.stopPropagation();
                    onToggleEnabled();
                }}
                title="Enable/disable (e)"
            />
            <span class="clip-lang">{clipLang}</span>
        </div>
        <div class="clip-center">
            {#if isEditingName}
                <input
                    class="name-input"
                    type="text"
                    value={ctx.editing?.value ?? ""}
                    oninput={(e) => handleEditInput("name", e)}
                    onkeydown={(e) => handleEditKeydown("name", e)}
                    onblur={() => ctx.cancelEdit()}
                    onclick={(e) => e.stopPropagation()}
                    ondblclick={(e) => e.stopPropagation()}
                    placeholder="F{frameIdx}"
                    use:focusOnMount
                />
            {:else}
                <span
                    class="clip-name"
                    ondblclick={(e) => handleEditStart("name", e)}
                    role="button"
                    tabindex="-1"
                    title="Double-click to edit name">{clipLabel}</span
                >
            {/if}
        </div>
        <div class="clip-bottom">
            {#if isEditingDuration}
                <input
                    class="info-input"
                    type="text"
                    value={ctx.editing?.value ?? ""}
                    oninput={(e) => handleEditInput("duration", e)}
                    onkeydown={(e) => handleEditKeydown("duration", e)}
                    onblur={() => ctx.cancelEdit()}
                    use:focusOnMount
                />
            {:else}
                <span
                    class="clip-info"
                    ondblclick={(e) => handleEditStart("duration", e)}
                    role="button"
                    tabindex="-1"
                    title="Duration (double-click to edit)"
                    >{formattedDuration}</span
                >
            {/if}
            {#if isEditingReps}
                <input
                    class="info-input"
                    type="text"
                    value={ctx.editing?.value ?? ""}
                    oninput={(e) => handleEditInput("reps", e)}
                    onkeydown={(e) => handleEditKeydown("reps", e)}
                    onblur={() => ctx.cancelEdit()}
                    use:focusOnMount
                />
            {:else}
                <span
                    class="clip-info"
                    ondblclick={(e) => handleEditStart("reps", e)}
                    role="button"
                    tabindex="-1"
                    title="Repetitions (double-click to edit)"
                    >{formattedReps}</span
                >
            {/if}
        </div>
    {/if}
    <div
        class="resize-handle"
        class:vertical={ctx.isVertical}
        onpointerdown={handleResizeStart}
    ></div>
</div>

<style>
    .clip {
        position: absolute;
        background-color: var(--colors-surface);
        border: 1px solid var(--colors-border);
        cursor: pointer;
        display: flex;
        flex-direction: column;
        justify-content: space-between;
        padding: 6px 8px;
        box-sizing: border-box;
        overflow: hidden;
        user-select: none;
        -webkit-user-select: none;
    }

    .clip.compact {
        justify-content: center;
        align-items: center;
        padding: 4px;
    }

    .clip:hover {
        border-color: var(--colors-text-secondary);
    }

    .clip.selected {
        border: 2px solid var(--colors-accent);
        padding: 5px 7px;
    }

    .clip.compact.selected {
        padding: 3px;
    }

    .clip.playing {
        background-color: var(--colors-accent);
        border-color: var(--colors-accent);
    }

    .clip.playing .clip-name,
    .clip.playing .clip-lang {
        color: var(--colors-background);
    }

    .clip.playing .clip-info {
        background-color: var(--colors-surface);
    }

    .clip.disabled {
        filter: grayscale(50%);
    }

    .clip.disabled .clip-name {
        text-decoration: line-through;
    }

    .clip-top {
        display: flex;
        justify-content: space-between;
        align-items: center;
        width: 100%;
    }

    .clip-bottom {
        display: flex;
        justify-content: space-between;
        align-items: center;
        width: 100%;
    }

    .clip-center {
        display: flex;
        align-items: center;
        justify-content: center;
        flex: 1;
        min-height: 0;
        width: 100%;
    }

    .clip-content {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 2px;
        width: 100%;
        overflow: hidden;
    }

    .compact-header {
        display: flex;
        align-items: center;
        gap: 4px;
    }

    .enable-checkbox {
        appearance: none;
        -webkit-appearance: none;
        width: 10px;
        height: 10px;
        border: 1px solid var(--colors-border);
        background-color: var(--colors-background);
        cursor: pointer;
        position: relative;
        flex-shrink: 0;
    }

    .enable-checkbox:checked {
        background-color: var(--colors-accent);
        border-color: var(--colors-accent);
    }

    .enable-checkbox:checked::after {
        content: "";
        position: absolute;
        left: 2px;
        top: 0px;
        width: 3px;
        height: 6px;
        border: solid var(--colors-background);
        border-width: 0 1.5px 1.5px 0;
        transform: rotate(45deg);
    }

    .enable-checkbox:hover {
        border-color: var(--colors-accent);
    }

    .clip-name {
        font-size: 11px;
        font-weight: 600;
        color: var(--colors-text);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
        max-width: 100%;
        text-align: center;
        cursor: text;
    }

    .clip-name:hover {
        color: var(--colors-accent);
    }

    .name-input {
        width: 90%;
        max-width: 100px;
        font-size: 11px;
        font-weight: 600;
        padding: 2px 6px;
        border: 2px solid var(--colors-accent);
        background-color: var(--colors-background);
        color: var(--colors-text);
        text-align: center;
    }

    .clip-lang {
        font-size: 9px;
        color: var(--colors-text-secondary);
        text-transform: lowercase;
    }

    .clip-info {
        font-size: 10px;
        color: var(--colors-text);
        background-color: var(--colors-background);
        padding: 1px 4px;
        cursor: text;
    }

    .clip-info:hover {
        color: var(--colors-accent);
    }

    .info-input {
        width: 32px;
        font-size: 10px;
        padding: 1px 4px;
        border: 1px solid var(--colors-accent);
        background-color: var(--colors-background);
        color: var(--colors-text);
        outline: none;
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

    .resize-handle.vertical {
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
</style>
