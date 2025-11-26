<script lang="ts">
	import { onDestroy } from 'svelte';
	import { editorConfig, currentTheme } from '$lib/stores/config';
	import { createEditor, createEditorSubscriptions } from '$lib/editor/editorFactory';
	import type { Frame } from '$lib/types/protocol';
	import type { EditorView } from '@codemirror/view';

	interface Props {
		frame: Frame | null;
		frameKey: string | null;
	}

	let { frame, frameKey }: Props = $props();

	let editorContainer: HTMLDivElement;
	let editorView: EditorView | null = null;
	let unsubscribe: (() => void) | null = null;

	$effect(() => {
		if (!editorContainer || !$editorConfig) return;

		if (!editorView) {
			editorView = createEditor(
				editorContainer,
				'',
				[],
				$editorConfig,
				$currentTheme
			);
			unsubscribe = createEditorSubscriptions(editorView);
		}
	});

	$effect(() => {
		if (editorView && frame) {
			const content = frame.script?.content || '';
			editorView.dispatch({
				changes: {
					from: 0,
					to: editorView.state.doc.length,
					insert: content
				}
			});
		}
	});

	onDestroy(() => {
		if (unsubscribe) {
			unsubscribe();
		}
		editorView?.destroy();
	});
</script>

<div class="editor-pane">
	{#if frame && frameKey}
		<div class="editor-header">
			<span>Editing {frameKey}</span>
		</div>
	{:else}
		<div class="editor-placeholder">
			<span>Double-click a clip or press Enter to edit</span>
		</div>
	{/if}
	<div class="editor-container" bind:this={editorContainer}></div>
</div>

<style>
	.editor-pane {
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		background-color: var(--colors-background);
	}

	.editor-header {
		height: 28px;
		background-color: var(--colors-surface);
		border-bottom: 1px solid var(--colors-border);
		display: flex;
		align-items: center;
		padding: 0 12px;
		font-size: 10px;
		color: var(--colors-text-secondary);
	}

	.editor-placeholder {
		height: 28px;
		background-color: var(--colors-surface);
		border-bottom: 1px solid var(--colors-border);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 10px;
		color: var(--colors-text-secondary);
		font-style: italic;
	}

	.editor-container {
		flex: 1;
		overflow: hidden;
	}

	:global(.editor-container .cm-editor) {
		height: 100%;
	}
</style>
