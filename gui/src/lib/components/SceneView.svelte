<script lang="ts">
	import { onDestroy } from 'svelte';
	import { ArrowLeftRight, ArrowUpDown } from 'lucide-svelte';
	import { scene, framePositions, isPlaying } from '$lib/stores';
	import { editorConfig, currentTheme } from '$lib/stores/config';
	import { selection, selectFrame } from '$lib/stores/selection';
	import { createEditor, createEditorSubscriptions } from '$lib/editor/editorFactory';
	import { javascript } from '@codemirror/lang-javascript';
	import type { Frame } from '$lib/types/protocol';
	import type { EditorView } from '@codemirror/view';
	import SplitPane from './SplitPane.svelte';

	const PIXELS_PER_BEAT = 40;
	const MIN_FRAME_WIDTH = 60;

	let splitOrientation = $state<'horizontal' | 'vertical'>('horizontal');
	let editorContainer: HTMLDivElement;
	let editorView: EditorView | null = null;
	let unsubscribe: (() => void) | null = null;

	function calculateFrameWidth(frame: Frame): number {
		return Math.max(frame.duration * frame.repetitions * PIXELS_PER_BEAT, MIN_FRAME_WIDTH);
	}

	function isFrameSelected(lineIdx: number, frameIdx: number): boolean {
		return $selection?.lineId === lineIdx && $selection?.frameId === frameIdx;
	}

	function isFramePlaying(lineIdx: number, frameIdx: number): boolean {
		return $isPlaying && $framePositions[lineIdx]?.[1] === frameIdx;
	}

	function toggleSplitOrientation() {
		splitOrientation = splitOrientation === 'horizontal' ? 'vertical' : 'horizontal';
	}

	function getSelectedFrame(): Frame | null {
		if (!$selection || !$scene) return null;
		const line = $scene.lines[$selection.lineId];
		if (!line) return null;
		return line.frames[$selection.frameId] ?? null;
	}

	$effect(() => {
		if (!editorContainer || !$editorConfig) return;

		if (!editorView) {
			editorView = createEditor(
				editorContainer,
				'// Select a frame to edit its script\n',
				javascript(),
				$editorConfig,
				$currentTheme
			);
			unsubscribe = createEditorSubscriptions(editorView);
		}

		const selectedFrame = getSelectedFrame();
		if (selectedFrame && editorView) {
			const currentContent = editorView.state.doc.toString();
			const newContent = selectedFrame.script.content || '// Empty script\n';

			if (currentContent !== newContent) {
				editorView.dispatch({
					changes: {
						from: 0,
						to: editorView.state.doc.length,
						insert: newContent
					}
				});
			}
		}
	});

	onDestroy(() => {
		if (unsubscribe) {
			unsubscribe();
		}
		editorView?.destroy();
	});
</script>

<div class="scene-container">
	<div class="toolbar">
		<div class="toolbar-left">
			<span class="title">SCENE</span>
			{#if $selection}
				<span class="selection-info">
					Line {$selection.lineId}, Frame {$selection.frameId}
				</span>
			{/if}
		</div>
		<button class="split-toggle" onclick={toggleSplitOrientation} title="Toggle split orientation">
			{#if splitOrientation === 'horizontal'}
				<ArrowLeftRight size={16} />
			{:else}
				<ArrowUpDown size={16} />
			{/if}
		</button>
	</div>

	<div class="split-container">
		<SplitPane orientation={splitOrientation}>
			{#snippet first()}
				<div class="timeline-pane">
					{#if !$scene || $scene.lines.length === 0}
						<div class="empty">No lines in scene</div>
					{:else}
						<div class="lines">
							{#each $scene.lines as line, lineIdx}
								<div class="line">
									<div class="line-label">
										<div class="line-name">Line {lineIdx}</div>
										<div class="line-info">{line.frames.length} frames</div>
									</div>
									<div class="frames">
										{#each line.frames as frame, frameIdx}
											<button
												class="frame"
												class:selected={isFrameSelected(lineIdx, frameIdx)}
												class:playing={isFramePlaying(lineIdx, frameIdx)}
												class:disabled={!frame.enabled}
												style="width: {calculateFrameWidth(frame)}px"
												onclick={() => selectFrame(lineIdx, frameIdx)}>
												<div class="frame-index">Frame {frameIdx}</div>
												<div class="frame-duration">{frame.duration}b</div>
												{#if frame.repetitions > 1}
													<div class="frame-reps">×{frame.repetitions}</div>
												{/if}
												<div class="frame-total"
													>{(frame.duration * frame.repetitions).toFixed(1)}b</div>
											</button>
										{/each}
									</div>
								</div>
							{/each}
						</div>
					{/if}
				</div>
			{/snippet}

			{#snippet second()}
				<div class="editor-pane">
					<div class="editor-container" bind:this={editorContainer}></div>
				</div>
			{/snippet}
		</SplitPane>
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

	.toolbar {
		height: 44px;
		background-color: var(--colors-surface);
		border-bottom: 1px solid var(--colors-border);
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 20px;
		font-family: monospace;
	}

	.toolbar-left {
		display: flex;
		align-items: center;
		gap: 20px;
	}

	.title {
		color: var(--colors-text);
		font-size: 13px;
		font-weight: 700;
		letter-spacing: 0.5px;
	}

	.selection-info {
		color: var(--colors-text-secondary);
		font-size: 11px;
		font-weight: 500;
	}

	.split-toggle {
		background: none;
		border: 1px solid var(--colors-border);
		color: var(--colors-text);
		padding: 6px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.split-toggle:hover {
		border-color: var(--colors-accent);
		color: var(--colors-accent);
	}

	.split-container {
		flex: 1;
		overflow: hidden;
	}

	.timeline-pane,
	.editor-pane {
		width: 100%;
		height: 100%;
		overflow: auto;
	}

	.editor-pane {
		background-color: var(--colors-background);
	}

	.editor-container {
		width: 100%;
		height: 100%;
	}

	:global(.editor-container .cm-editor) {
		height: 100%;
	}

	.empty {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 100%;
		color: var(--colors-text-secondary);
		font-size: 14px;
	}

	.lines {
		padding: 16px;
	}

	.line {
		display: flex;
		margin-bottom: 12px;
		background-color: var(--colors-surface);
		border: 1px solid var(--colors-border);
		min-height: 90px;
	}

	.line-label {
		width: 110px;
		padding: 14px;
		border-right: 1px solid var(--colors-border);
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 6px;
	}

	.line-name {
		color: var(--colors-text);
		font-size: 13px;
		font-weight: 600;
		letter-spacing: 0.3px;
	}

	.line-info {
		color: var(--colors-text-secondary);
		font-size: 10px;
		font-weight: 500;
	}

	.frames {
		display: flex;
		gap: 6px;
		padding: 12px;
		flex: 1;
		align-items: stretch;
	}

	.frame {
		background-color: #1a1a1a;
		border: 1px solid var(--colors-border);
		color: var(--colors-text-secondary);
		padding: 10px;
		display: flex;
		flex-direction: column;
		gap: 4px;
		font-family: monospace;
		cursor: pointer;
		position: relative;
		transition: all 0.15s ease;
	}

	.frame:hover {
		border-color: var(--colors-text-secondary);
	}

	.frame.selected {
		border: 3px solid var(--colors-accent);
		border-color: var(--colors-accent);
	}

	.frame.playing {
		background-color: var(--colors-accent);
		color: var(--colors-text);
	}

	.frame.disabled {
		opacity: 0.3;
		position: relative;
	}

	.frame.disabled::after {
		content: '';
		position: absolute;
		top: 0;
		left: 0;
		right: 0;
		bottom: 0;
		background: linear-gradient(
			to bottom right,
			transparent calc(50% - 1px),
			var(--colors-text-secondary) calc(50% - 1px),
			var(--colors-text-secondary) calc(50% + 1px),
			transparent calc(50% + 1px)
		);
		pointer-events: none;
	}

	.frame-index {
		font-size: 11px;
		font-weight: 600;
		letter-spacing: 0.3px;
	}

	.frame-duration {
		font-size: 10px;
		font-weight: 500;
	}

	.frame-reps {
		font-size: 10px;
		font-weight: 500;
	}

	.frame-total {
		font-size: 10px;
		font-weight: 700;
		margin-top: auto;
		text-align: right;
		letter-spacing: 0.2px;
	}

	.frame.playing .frame-total {
		color: var(--colors-text);
	}
</style>
