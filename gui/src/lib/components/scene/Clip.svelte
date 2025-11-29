<script lang="ts">
	import type { Frame } from '$lib/types/protocol';
	import { getTimelineContext } from './context.svelte';

	interface Props {
		frame: Frame;
		lineIdx: number;
		frameIdx: number;
		offset: number;
		extent: number;
		trackWidth: number;
		selected: boolean;
		playing: boolean;
		editingDuration: { value: string } | null;
		onClick: (e: MouseEvent) => void;
		onDoubleClick: () => void;
		onResizeStart: (e: MouseEvent) => void;
		onDurationEditStart: (e: MouseEvent) => void;
		onDurationInput: (e: Event) => void;
		onDurationKeydown: (e: KeyboardEvent) => void;
		onDurationBlur: () => void;
		editingReps: { value: string } | null;
		onRepsEditStart: (e: MouseEvent) => void;
		onRepsInput: (e: Event) => void;
		onRepsKeydown: (e: KeyboardEvent) => void;
		onRepsBlur: () => void;
		editingName: { value: string } | null;
		onNameEditStart: (e: MouseEvent) => void;
		onNameInput: (e: Event) => void;
		onNameKeydown: (e: KeyboardEvent) => void;
		onNameBlur: () => void;
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
		editingDuration,
		onClick,
		onDoubleClick,
		onResizeStart,
		onDurationEditStart,
		onDurationInput,
		onDurationKeydown,
		onDurationBlur,
		editingReps,
		onRepsEditStart,
		onRepsInput,
		onRepsKeydown,
		onRepsBlur,
		editingName,
		onNameEditStart,
		onNameInput,
		onNameKeydown,
		onNameBlur
	}: Props = $props();

	const ctx = getTimelineContext();

	// Pure derived values
	const duration = $derived(
		typeof frame.duration === 'number' && !isNaN(frame.duration) && frame.duration > 0
			? frame.duration
			: 1
	);

	const reps = $derived(
		typeof frame.repetitions === 'number' && !isNaN(frame.repetitions) && frame.repetitions >= 1
			? frame.repetitions
			: 1
	);

	const clipLabel = $derived(frame.name || `F${frameIdx}`);
	const clipLang = $derived(frame.script?.lang || 'bali');
	const formattedDuration = $derived(`${duration}`);
	const formattedReps = $derived(`×${reps}`);

	// Layout mode: compact (stacked) vs normal (4-corners)
	// In vertical mode, width is trackWidth; in horizontal mode, width is extent
	const clipWidth = $derived(ctx.isVertical ? trackWidth - 8 : extent);
	const isCompact = $derived(clipWidth < 80);
	// Progressive hiding in compact mode
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

	// Focus input when it mounts
	function focusOnMount(node: HTMLInputElement) {
		node.focus();
		node.select();
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
	role="button"
	tabindex="-1"
>
	{#if isCompact}
		<!-- Compact: stacked centered layout -->
		<div class="clip-content">
			{#if showLangCompact}
				<span class="clip-lang">{clipLang}</span>
			{/if}
			{#if editingName}
				<input
					class="name-input"
					type="text"
					value={editingName.value}
					oninput={onNameInput}
					onkeydown={onNameKeydown}
					onblur={onNameBlur}
					placeholder="F{frameIdx}"
					use:focusOnMount
				/>
			{:else}
				<span
					class="clip-name"
					ondblclick={(e) => {
						e.stopPropagation();
						onNameEditStart(e);
					}}
					title="Double-click to edit name"
				>{clipLabel}</span>
			{/if}
			{#if editingDuration}
				<input
					class="info-input"
					type="text"
					value={editingDuration.value}
					oninput={onDurationInput}
					onkeydown={onDurationKeydown}
					onblur={onDurationBlur}
					use:focusOnMount
				/>
			{:else}
				<span
					class="clip-info"
					ondblclick={onDurationEditStart}
					title="Duration (double-click to edit)"
				>{formattedDuration}</span>
			{/if}
			{#if showRepsCompact}
				{#if editingReps}
					<input
						class="info-input"
						type="text"
						value={editingReps.value}
						oninput={onRepsInput}
						onkeydown={onRepsKeydown}
						onblur={onRepsBlur}
						use:focusOnMount
					/>
				{:else}
					<span
						class="clip-info"
						ondblclick={onRepsEditStart}
						title="Repetitions (double-click to edit)"
					>{formattedReps}</span>
				{/if}
			{/if}
		</div>
	{:else}
		<!-- Normal: 4-corners layout -->
		<div class="clip-top">
			<span class="clip-lang">{clipLang}</span>
		</div>
		<div class="clip-center">
			{#if editingName}
				<input
					class="name-input"
					type="text"
					value={editingName.value}
					oninput={onNameInput}
					onkeydown={onNameKeydown}
					onblur={onNameBlur}
					placeholder="F{frameIdx}"
					use:focusOnMount
				/>
			{:else}
				<span
					class="clip-name"
					ondblclick={(e) => {
						e.stopPropagation();
						onNameEditStart(e);
					}}
					title="Double-click to edit name"
				>{clipLabel}</span>
			{/if}
		</div>
		<div class="clip-bottom">
			{#if editingDuration}
				<input
					class="info-input"
					type="text"
					value={editingDuration.value}
					oninput={onDurationInput}
					onkeydown={onDurationKeydown}
					onblur={onDurationBlur}
					use:focusOnMount
				/>
			{:else}
				<span
					class="clip-info"
					ondblclick={onDurationEditStart}
					title="Duration (double-click to edit)"
				>{formattedDuration}</span>
			{/if}
			{#if editingReps}
				<input
					class="info-input"
					type="text"
					value={editingReps.value}
					oninput={onRepsInput}
					onkeydown={onRepsKeydown}
					onblur={onRepsBlur}
					use:focusOnMount
				/>
			{:else}
				<span
					class="clip-info"
					ondblclick={onRepsEditStart}
					title="Repetitions (double-click to edit)"
				>{formattedReps}</span>
			{/if}
		</div>
	{/if}
	<div
		class="resize-handle"
		class:vertical={ctx.isVertical}
		onmousedown={onResizeStart}
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
		user-select: none;
		box-sizing: border-box;
		overflow: hidden;
	}

	.clip.compact {
		justify-content: center;
		align-items: center;
		padding: 4px;
	}

	.clip * {
		user-select: none;
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

	/* Normal layout: 4-corners */
	.clip-top {
		display: flex;
		justify-content: flex-end;
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

	/* Compact layout: stacked centered */
	.clip-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 2px;
		width: 100%;
		overflow: hidden;
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
