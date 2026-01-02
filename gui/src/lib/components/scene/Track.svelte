<script lang="ts">
    import { X, Plus } from "lucide-svelte";
    import type { Frame, Line } from "$lib/types/protocol";
    import Clip from "./Clip.svelte";
    import { getTimelineContext } from "./context.svelte";

    interface Props {
        line: Line;
        lineIdx: number;
        visibleBeatMarkers: number[];
        trackWidth: number;
        onRemoveTrack: (_e: MouseEvent) => void;
        onAddClip: () => void;
        onClipClick: (_frameIdx: number, _e: MouseEvent) => void;
        onClipDoubleClick: (_frameIdx: number) => void;
        onLineResizeStart: (_e: MouseEvent) => void;
        isFrameSelected: (_frameIdx: number) => boolean;
        playingFrameIdx: number | null;
        onSolo: () => void;
        onMute: () => void;
        isSolo: boolean;
        isMuted: boolean;
        onToggleEnabled: (_frameIdx: number) => void;
    }

    let {
        line,
        lineIdx,
        visibleBeatMarkers,
        trackWidth,
        onRemoveTrack,
        onAddClip,
        onClipClick,
        onClipDoubleClick,
        onLineResizeStart,
        isFrameSelected,
        playingFrameIdx,
        onSolo,
        onMute,
        isSolo,
        isMuted,
        onToggleEnabled,
    }: Props = $props();

    const ctx = getTimelineContext();

    const displayStartFrame = $derived(
        line.start_frame != null ? (line.start_frame + 1).toString() : "Start",
    );
    const displayEndFrame = $derived(
        line.end_frame != null ? (line.end_frame + 1).toString() : "End",
    );

    const isEditingStartFrame = $derived(
        ctx.editing?.field === "startFrame" &&
            ctx.editing?.lineIdx === lineIdx &&
            ctx.editing?.frameIdx === -1,
    );
    const isEditingEndFrame = $derived(
        ctx.editing?.field === "endFrame" &&
            ctx.editing?.lineIdx === lineIdx &&
            ctx.editing?.frameIdx === -1,
    );

    function focusOnMount(node: HTMLInputElement) {
        node.focus();
        node.select();
    }

    function handleLineEditStart(
        field: "startFrame" | "endFrame",
        e: MouseEvent,
    ) {
        e.stopPropagation();
        ctx.startLineEdit(field, lineIdx);
    }

    function handleLineEditInput(e: Event) {
        const field = ctx.editing?.field;
        if (field === "startFrame" || field === "endFrame") {
            ctx.updateEditValue(field, (e.target as HTMLInputElement).value);
        }
    }

    function handleLineEditKeydown(
        field: "startFrame" | "endFrame",
        e: KeyboardEvent,
    ) {
        if (e.key === "Enter") {
            e.preventDefault();
            e.stopPropagation();
            ctx.commitLineEdit(field);
        } else if (e.key === "Escape") {
            e.stopPropagation();
            ctx.cancelEdit();
        }
    }

    function getFrameExtent(frame: Frame, frameIdx: number): number {
        const previewDuration = ctx.getPreviewDuration(lineIdx, frameIdx);
        const d =
            previewDuration !== null
                ? previewDuration
                : typeof frame.duration === "number" &&
                    !isNaN(frame.duration) &&
                    frame.duration > 0
                  ? frame.duration
                  : 1;
        const r =
            typeof frame.repetitions === "number" &&
            !isNaN(frame.repetitions) &&
            frame.repetitions >= 1
                ? frame.repetitions
                : 1;
        return d * r * ctx.pixelsPerBeat;
    }

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

    const totalLength = $derived(
        clipPositions.length > 0
            ? clipPositions[clipPositions.length - 1].offset +
                  clipPositions[clipPositions.length - 1].extent
            : 0,
    );

    const trackStyle = $derived(
        ctx.isVertical ? `width: ${trackWidth}px` : `height: ${trackWidth}px`,
    );

    const addClipStyle = $derived.by(() => {
        const clipSize = trackWidth - 8;
        return ctx.isVertical
            ? `top: ${totalLength}px; left: 4px; width: ${clipSize}px`
            : `left: ${totalLength}px; top: 4px; height: ${clipSize}px`;
    });

    const dropIndicatorIdx = $derived(ctx.getDropIndicatorIdx(lineIdx));

    const dropIndicatorStyle = $derived.by(() => {
        if (dropIndicatorIdx === null) return null;
        const indicatorPos =
            dropIndicatorIdx < clipPositions.length
                ? clipPositions[dropIndicatorIdx].offset
                : totalLength;
        return ctx.isVertical
            ? `top: ${indicatorPos}px; left: 4px; right: 4px;`
            : `left: ${indicatorPos}px; top: 4px; bottom: 4px;`;
    });

    function getMarkerStyle(beat: number): string {
        const pos = beat * ctx.pixelsPerBeat;
        return ctx.isVertical ? `top: ${pos}px` : `left: ${pos}px`;
    }
</script>

<div class="track-row" class:vertical={ctx.isVertical} style={trackStyle}>
    <div class="track-header" class:vertical={ctx.isVertical}>
        <span class="track-number">LINE {lineIdx}</span>
        <div class="track-controls">
            <button
                class="track-solo"
                class:active={isSolo}
                onclick={onSolo}
                title="Solo">S</button
            >
            <button
                class="track-mute"
                class:active={isMuted}
                onclick={onMute}
                title="Mute">M</button
            >
        </div>
        <button
            class="track-remove"
            onclick={onRemoveTrack}
            title="Remove track"
        >
            <X size={12} />
        </button>
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="line-resize-handle header-handle"
            class:vertical={ctx.isVertical}
            onmousedown={onLineResizeStart}
        ></div>
    </div>
    <div class="frame-range-row" class:vertical={ctx.isVertical}>
        <div class="frame-range-field">
            {#if isEditingStartFrame}
                <input
                    class="frame-range-input"
                    type="text"
                    value={ctx.editing?.value ?? ""}
                    oninput={handleLineEditInput}
                    onkeydown={(e) => handleLineEditKeydown("startFrame", e)}
                    onblur={() => ctx.cancelEdit()}
                    use:focusOnMount
                />
            {:else}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span
                    class="frame-range-value"
                    class:placeholder={line.start_frame == null}
                    ondblclick={(e) => handleLineEditStart("startFrame", e)}
                    title="Start frame (double-click to edit)"
                >
                    {displayStartFrame}
                </span>
            {/if}
        </div>
        <span class="frame-range-separator">-</span>
        <div class="frame-range-field">
            {#if isEditingEndFrame}
                <input
                    class="frame-range-input"
                    type="text"
                    value={ctx.editing?.value ?? ""}
                    oninput={handleLineEditInput}
                    onkeydown={(e) => handleLineEditKeydown("endFrame", e)}
                    onblur={() => ctx.cancelEdit()}
                    use:focusOnMount
                />
            {:else}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <span
                    class="frame-range-value"
                    class:placeholder={line.end_frame == null}
                    ondblclick={(e) => handleLineEditStart("endFrame", e)}
                    title="End frame (double-click to edit)"
                >
                    {displayEndFrame}
                </span>
            {/if}
        </div>
    </div>
    <div class="track-content">
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
            class="line-resize-handle"
            class:vertical={ctx.isVertical}
            onmousedown={onLineResizeStart}
        ></div>
        <div class="grid-lines">
            {#each visibleBeatMarkers as beat (beat)}
                <div
                    class="grid-line"
                    class:vertical={ctx.isVertical}
                    style={getMarkerStyle(beat)}
                ></div>
            {/each}
        </div>
        {#each line.frames as frame, frameIdx (frameIdx)}
            {@const pos = clipPositions[frameIdx]}
            <Clip
                {frame}
                {lineIdx}
                {frameIdx}
                offset={pos.offset}
                extent={pos.extent}
                {trackWidth}
                selected={isFrameSelected(frameIdx)}
                playing={playingFrameIdx === frameIdx}
                onClick={(e) => onClipClick(frameIdx, e)}
                onDoubleClick={() => onClipDoubleClick(frameIdx)}
                onToggleEnabled={() => onToggleEnabled(frameIdx)}
            />
        {/each}

        {#if dropIndicatorStyle}
            <div
                class="drop-indicator"
                class:vertical={ctx.isVertical}
                style={dropIndicatorStyle}
            ></div>
        {/if}

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
        user-select: none;
        -webkit-user-select: none;
    }

    .track-row.vertical {
        flex-direction: column;
        border-bottom: none;
        border-right: 1px solid var(--colors-border);
    }

    .track-header {
        position: relative;
        width: 70px;
        min-width: 70px;
        background-color: var(--colors-surface);
        border-right: 1px solid var(--colors-border);
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        gap: 6px;
        padding: 8px 4px;
        box-sizing: border-box;
    }

    .track-header.vertical {
        width: auto;
        min-width: auto;
        height: auto;
        min-height: auto;
        border-right: none;
        border-bottom: 1px solid var(--colors-border);
        flex-direction: column;
        gap: 6px;
        padding: 8px 0;
        box-sizing: border-box;
    }

    .track-controls {
        display: flex;
        flex-direction: row;
        gap: 4px;
    }

    .track-solo,
    .track-mute {
        background: none;
        border: 1px solid var(--colors-border);
        color: var(--colors-text-secondary);
        cursor: pointer;
        padding: 3px 6px;
        font-size: 9px;
        font-weight: 600;
        line-height: 1;
        opacity: 0.5;
    }

    .track-row:hover .track-solo,
    .track-row:hover .track-mute {
        opacity: 1;
    }

    .track-solo:hover {
        border-color: var(--colors-accent);
        color: var(--colors-accent);
    }

    .track-mute:hover {
        border-color: #f59e0b;
        color: #f59e0b;
    }

    .track-solo.active {
        background-color: var(--colors-accent);
        border-color: var(--colors-accent);
        color: var(--colors-background);
        opacity: 1;
    }

    .track-mute.active {
        background-color: #f59e0b;
        border-color: #f59e0b;
        color: var(--colors-background);
        opacity: 1;
    }

    .track-number {
        font-size: 10px;
        font-weight: 600;
        color: var(--colors-text);
        white-space: nowrap;
    }

    .track-remove {
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

    .track-header.vertical .track-remove {
        top: auto;
        right: 4px;
        bottom: auto;
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

    .track-row.vertical .track-content {
        overflow: hidden;
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

    .drop-indicator {
        position: absolute;
        background-color: var(--colors-accent);
        pointer-events: none;
        z-index: 100;
        width: 2px;
    }

    .drop-indicator.vertical {
        width: auto;
        height: 2px;
    }

    .frame-range-row {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 4px;
        padding: 2px 4px;
        background-color: var(--colors-surface);
        border-right: 1px solid var(--colors-border);
        font-size: 10px;
        min-width: 70px;
        box-sizing: border-box;
    }

    .frame-range-row.vertical {
        flex-direction: row;
        border-right: none;
        border-bottom: 1px solid var(--colors-border);
        min-width: auto;
        padding: 4px 8px;
    }

    .frame-range-field {
        min-width: 24px;
        text-align: center;
    }

    .frame-range-value {
        color: var(--colors-text);
        cursor: text;
        padding: 1px 4px;
    }

    .frame-range-value.placeholder {
        color: var(--colors-text-secondary);
        opacity: 0.6;
    }

    .frame-range-value:hover {
        color: var(--colors-accent);
    }

    .frame-range-input {
        width: 32px;
        font-size: 10px;
        padding: 1px 4px;
        border: 1px solid var(--colors-accent);
        background-color: var(--colors-background);
        color: var(--colors-text);
        text-align: center;
        outline: none;
    }

    .frame-range-separator {
        color: var(--colors-text-secondary);
    }
</style>
