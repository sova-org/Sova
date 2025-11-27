<script lang="ts">
	import { onDestroy } from 'svelte';
	import { Check, AlertCircle, Loader2, Play, RotateCcw } from 'lucide-svelte';
	import { EditorView } from '@codemirror/view';
	import { editorConfig, currentTheme } from '$lib/stores/config';
	import { availableLanguages } from '$lib/stores/languages';
	import {
		localEdits,
		getLocalEdit,
		setLocalEdit,
		clearLocalEdit
	} from '$lib/stores/localEdits';
	import { compilationStates } from '$lib/stores/compilation';
	import { createEditor, createEditorSubscriptions } from '$lib/editor/editorFactory';
	import { setFrames, ActionTiming } from '$lib/api/client';
	import type { Frame, CompilationState } from '$lib/types/protocol';

	interface Props {
		frame: Frame | null;
		frameKey: string | null;
		lineIdx: number | null;
		frameIdx: number | null;
	}

	let { frame, frameKey, lineIdx, frameIdx }: Props = $props();

	let editorContainer: HTMLDivElement;
	let editorView: EditorView | null = null;
	let unsubscribe: (() => void) | null = null;
	let selectedLang = $state<string>('');
	let isEvaluating = $state(false);
	let evaluationPending = false; // Debounce flag
	let previousFrameKey: string | null = null;

	// Local state for frame properties (saved with evaluation)
	// Initialize from frame prop to avoid sync timing issues
	let localDuration = $state<number>(frame?.duration ?? 1);
	let localRepetitions = $state<number>(frame?.repetitions ?? 1);
	let localName = $state<string>(frame?.name ?? '');
	let localEnabled = $state<boolean>(frame?.enabled ?? true);

	// Track if current frame has any unsaved changes
	const isDirty = $derived.by(() => {
		if (!frame) return $localEdits.has(frameKey ?? '');
		const hasScriptEdits = $localEdits.has(frameKey ?? '');
		const hasPropertyChanges =
			localDuration !== frame.duration ||
			localRepetitions !== frame.repetitions ||
			localName !== (frame.name ?? '') ||
			localEnabled !== frame.enabled;
		return hasScriptEdits || hasPropertyChanges;
	});

	// Sync editor content helper
	function syncEditorContent(content: string) {
		if (!editorView) return;
		editorView.dispatch({
			changes: {
				from: 0,
				to: editorView.state.doc.length,
				insert: content
			}
		});
	}

	// Create update listener that saves to localEdits on every change
	function createUpdateListener() {
		return EditorView.updateListener.of((update) => {
			if (update.docChanged && frameKey) {
				const content = update.state.doc.toString();
				setLocalEdit(frameKey, content, selectedLang);
			}
		});
	}

	// Initialize editor
	$effect(() => {
		if (!editorContainer || !$editorConfig) return;

		if (!editorView) {
			editorView = createEditor(
				editorContainer,
				'',
				[createUpdateListener()],
				$editorConfig,
				$currentTheme
			);
			unsubscribe = createEditorSubscriptions(editorView);
		}
	});

	// Sync content and properties when frameKey changes
	$effect(() => {
		if (!editorView || !frameKey) return;

		// Only sync when frameKey actually changes
		if (frameKey !== previousFrameKey) {
			previousFrameKey = frameKey;

			// Check for local edit first
			const localEdit = getLocalEdit(frameKey);
			if (localEdit) {
				syncEditorContent(localEdit.content);
				selectedLang = localEdit.lang;
			} else {
				// Use server state
				syncEditorContent(frame?.script?.content || '');
				selectedLang = frame?.script?.lang || 'bali';
			}

			// Sync frame properties from server state
			localDuration = frame?.duration ?? 1;
			localRepetitions = frame?.repetitions ?? 1;
			localName = frame?.name ?? '';
			localEnabled = frame?.enabled ?? true;
		}
	});

	function getCompilationStatus(state: CompilationState | null | undefined): 'none' | 'compiling' | 'compiled' | 'error' {
		if (!state) return 'none';
		if (state === 'NotCompiled') return 'none';
		if (state === 'Compiling') return 'compiling';
		if (state === 'Compiled') return 'compiled';
		if (typeof state === 'object' && 'Error' in state) return 'error';
		return 'none';
	}

	function getCompilationError(state: CompilationState | null | undefined): string | null {
		if (state && typeof state === 'object' && 'Error' in state) {
			return state.Error.info;
		}
		return null;
	}

	async function evaluateScript() {
		if (!frame || lineIdx === null || frameIdx === null || !editorView || !frameKey) return;
		if (evaluationPending) return; // Prevent concurrent evaluations

		evaluationPending = true;
		isEvaluating = true;

		try {
			const content = editorView.state.doc.toString();
			const updatedFrame: Frame = {
				...frame,
				duration: localDuration,
				repetitions: localRepetitions,
				name: localName || null,
				enabled: localEnabled,
				script: {
					...frame.script,
					content,
					lang: selectedLang
				}
			};

			await setFrames([[lineIdx, frameIdx, updatedFrame]], ActionTiming.immediate());

			// Clear local edit after successful evaluation
			clearLocalEdit(frameKey);
		} catch (error) {
			console.error('Failed to evaluate script:', error);
		} finally {
			isEvaluating = false;
			evaluationPending = false;
		}
	}

	function discardChanges() {
		if (!frameKey) return;

		// Clear local edit
		clearLocalEdit(frameKey);

		// Re-sync from server state
		syncEditorContent(frame?.script?.content || '');
		selectedLang = frame?.script?.lang || 'bali';
		localDuration = frame?.duration ?? 1;
		localRepetitions = frame?.repetitions ?? 1;
		localName = frame?.name ?? '';
		localEnabled = frame?.enabled ?? true;
	}

	function handleKeydown(event: KeyboardEvent) {
		if ((event.metaKey || event.ctrlKey) && event.key === 'Enter') {
			event.preventDefault();
			evaluateScript();
		}
	}

	function handleLangChange(event: Event) {
		const target = event.target as HTMLSelectElement;
		selectedLang = target.value;

		// Save lang change to local edits
		if (frameKey && editorView) {
			setLocalEdit(frameKey, editorView.state.doc.toString(), selectedLang);
		}
	}

	onDestroy(() => {
		if (unsubscribe) {
			unsubscribe();
		}
		editorView?.destroy();
	});

	// Get compilation state directly from the main store (no intermediate store creation)
	const compilationState = $derived.by(() => {
		if (lineIdx === null || frameIdx === null) return null;
		const key = `${lineIdx}:${frameIdx}`;
		return $compilationStates.get(key)?.state ?? null;
	});

	const compilationStatus = $derived(getCompilationStatus(compilationState));
	const compilationError = $derived(getCompilationError(compilationState));
</script>

<div class="editor-pane" onkeydown={handleKeydown}>
	{#if frame && frameKey}
		<div class="editor-header" class:dirty={isDirty}>
			<div class="header-left">
				<span class="frame-label">Line {(lineIdx ?? 0) + 1} Frame {(frameIdx ?? 0) + 1}</span>
				<select
					class="lang-select"
					value={selectedLang}
					onchange={handleLangChange}
				>
					{#each $availableLanguages as lang}
						<option value={lang}>{lang}</option>
					{/each}
				</select>

				<label class="prop-field">
					<span>dur</span>
					<input
						type="number"
						class="prop-input"
						bind:value={localDuration}
						min="0.125"
						step="0.25"
					/>
				</label>

				<label class="prop-field">
					<span>×</span>
					<input
						type="number"
						class="prop-input"
						bind:value={localRepetitions}
						min="1"
						step="1"
					/>
				</label>

				<label class="prop-field">
					<span>name</span>
					<input
						type="text"
						class="prop-input name"
						bind:value={localName}
						placeholder="F{frameIdx}"
					/>
				</label>

				<button
					class="enabled-toggle"
					class:disabled={!localEnabled}
					onclick={() => localEnabled = !localEnabled}
					title={localEnabled ? 'Enabled (click to disable)' : 'Disabled (click to enable)'}
				>
					{localEnabled ? '●' : '○'}
				</button>
			</div>

			<div class="header-right">
				{#if isDirty}
					<button
						class="discard-button"
						onclick={discardChanges}
						title="Discard changes"
					>
						<RotateCcw size={12} />
					</button>
				{/if}

				{#if compilationStatus === 'compiling' || isEvaluating}
					<span class="status compiling" title="Compiling...">
						<Loader2 size={12} class="spin" />
					</span>
				{:else if compilationStatus === 'compiled'}
					<span class="status compiled" title="Compiled">
						<Check size={12} />
					</span>
				{:else if compilationStatus === 'error'}
					<span class="status error" title={compilationError || 'Compilation error'}>
						<AlertCircle size={12} />
					</span>
				{/if}

				<button
					class="eval-button"
					onclick={evaluateScript}
					disabled={isEvaluating}
					title="Evaluate (Cmd+Enter)"
				>
					<Play size={12} />
				</button>
			</div>
		</div>

		{:else}
		<div class="editor-placeholder">
			<span>Double-click a clip or press Enter to edit</span>
		</div>
	{/if}
	<div class="editor-container" bind:this={editorContainer}></div>
	{#if frame && frameKey}
		<div
			class="status-bar"
			class:compiled={compilationStatus === 'compiled'}
			class:error={compilationStatus === 'error'}
			class:compiling={compilationStatus === 'compiling' || isEvaluating}
		>
			{#if compilationStatus === 'compiling' || isEvaluating}
				<Loader2 size={12} class="spin" /> Compiling...
			{:else if compilationStatus === 'compiled'}
				<Check size={12} /> Compiled
			{:else if compilationStatus === 'error'}
				<AlertCircle size={12} /> {compilationError || 'Compilation error'}
			{:else}
				<span class="muted">Not compiled</span>
			{/if}
		</div>
	{/if}
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
		justify-content: space-between;
		padding: 0 8px;
		font-size: 10px;
		color: var(--colors-text-secondary);
		transition: background-color 0.2s;
	}

	.editor-header.dirty {
		background-color: color-mix(in srgb, var(--colors-accent) 15%, var(--colors-surface));
	}

	.header-left,
	.header-right {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.header-left {
		flex: 1;
	}

	.header-right {
		justify-content: flex-end;
	}

	.frame-label {
		color: var(--colors-text-secondary);
	}

	.lang-select {
		background-color: var(--colors-background);
		border: 1px solid var(--colors-border);
		color: var(--colors-text);
		font-size: 10px;
		font-family: monospace;
		padding: 2px 4px;
		cursor: pointer;
	}

	.lang-select:hover {
		border-color: var(--colors-accent);
	}

	.lang-select:focus {
		outline: none;
		border-color: var(--colors-accent);
	}

	.prop-field {
		display: flex;
		align-items: center;
		gap: 2px;
		font-size: 10px;
		color: var(--colors-text-secondary);
	}

	.prop-input {
		width: 40px;
		background-color: var(--colors-background);
		border: 1px solid var(--colors-border);
		color: var(--colors-text);
		font-size: 10px;
		font-family: monospace;
		padding: 2px 4px;
	}

	.prop-input.name {
		width: 60px;
	}

	.prop-input:hover,
	.prop-input:focus {
		border-color: var(--colors-accent);
		outline: none;
	}

	.enabled-toggle {
		background: none;
		border: 1px solid var(--colors-border);
		color: var(--colors-success, #4caf50);
		padding: 2px 6px;
		cursor: pointer;
		font-size: 10px;
	}

	.enabled-toggle.disabled {
		color: var(--colors-text-secondary);
		opacity: 0.5;
	}

	.enabled-toggle:hover {
		border-color: var(--colors-accent);
	}

	.status {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		height: 18px;
	}

	.status.compiling {
		color: var(--colors-text-secondary);
	}

	.status.compiled {
		color: var(--colors-success, #4caf50);
	}

	.status.error {
		color: var(--colors-error, #f44336);
	}

	:global(.status .spin) {
		animation: spin 1s linear infinite;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	.eval-button,
	.discard-button {
		background: none;
		border: 1px solid var(--colors-border);
		color: var(--colors-text-secondary);
		padding: 3px 6px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.eval-button:hover:not(:disabled),
	.discard-button:hover {
		border-color: var(--colors-accent);
		color: var(--colors-accent);
	}

	.eval-button:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.status-bar {
		height: 22px;
		padding: 0 8px;
		font-size: 10px;
		font-family: monospace;
		display: flex;
		align-items: center;
		gap: 6px;
		border-top: 1px solid var(--colors-border);
		background-color: var(--colors-surface);
		color: var(--colors-text-secondary);
	}

	.status-bar.compiled {
		color: var(--colors-success, #4caf50);
	}

	.status-bar.error {
		background-color: color-mix(in srgb, var(--colors-error, #f44336) 15%, var(--colors-surface));
		color: var(--colors-error, #f44336);
	}

	.status-bar.compiling {
		color: var(--colors-text-secondary);
	}

	.status-bar .muted {
		color: var(--colors-text-muted, #666);
		font-style: italic;
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
