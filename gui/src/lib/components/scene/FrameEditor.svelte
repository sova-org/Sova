<script lang="ts">
	import { onDestroy } from 'svelte';
	import { Check, AlertCircle, Loader2, Send, RotateCcw, X } from 'lucide-svelte';
	import Select from '$lib/components/Select.svelte';
	import { EditorView } from '@codemirror/view';
	import { editorConfig } from '$lib/stores/config';
	import { availableLanguages } from '$lib/stores/languages';
	import {
		localEdits,
		getLocalEdit,
		setLocalEdit,
		clearLocalEdit
	} from '$lib/stores/localEdits';
	import { compilationStates } from '$lib/stores/compilation';
	import { createEditor, createEditorSubscriptions, reconfigureLanguage } from '$lib/editor/editorFactory';
	import { getLanguageSupport } from '../../../languages';
	import { setFrames, ActionTiming } from '$lib/api/client';
	import type { Frame, CompilationState } from '$lib/types/protocol';

	interface Props {
		frame: Frame | null;
		frameKey: string | null;
		lineIdx: number | null;
		frameIdx: number | null;
		onClose?: () => void;
	}

	let { frame, frameKey, lineIdx, frameIdx, onClose }: Props = $props();

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
			const langSupport = getLanguageSupport(selectedLang) ?? [];
			editorView = createEditor(
				editorContainer,
				'',
				langSupport,
				$editorConfig,
				[createUpdateListener()]
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

	// Reconfigure language when selectedLang changes
	$effect(() => {
		if (!editorView) return;
		const langSupport = getLanguageSupport(selectedLang) ?? [];
		reconfigureLanguage(editorView, langSupport);
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
		<div class="editor-header">
			<div class="header-content">
				<Select
					options={$availableLanguages}
					value={selectedLang}
					onchange={(lang) => {
						selectedLang = lang;
						if (frameKey && editorView) {
							setLocalEdit(frameKey, editorView.state.doc.toString(), lang);
						}
					}}
				/>

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

				<label class="enabled-field" data-help-id="frame-enabled">
					<input
						type="checkbox"
						bind:checked={localEnabled}
					/>
					<span>Enabled</span>
				</label>

				{#if isDirty}
					<button
						class="action-btn"
						data-help-id="frame-fetch"
						onclick={discardChanges}
						title="Discard changes"
					>
						<RotateCcw size={12} />
						Fetch
					</button>
				{/if}

				<button
					class="action-btn"
					data-help-id="frame-evaluate"
					onclick={evaluateScript}
					disabled={isEvaluating}
					title="Evaluate (Cmd+Enter)"
				>
					<Send size={12} />
					Evaluate
				</button>
			</div>

			{#if onClose}
				<button
					class="close-button"
					data-help-id="frame-close"
					onclick={onClose}
					title="Close editor"
				>
					<X size={12} />
				</button>
			{/if}
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
		background-color: var(--colors-surface);
		border-bottom: 1px solid var(--colors-border);
		display: flex;
		align-items: center;
		padding: 4px 8px;
		gap: 8px;
		font-size: 10px;
		color: var(--colors-text-secondary);
	}

	.header-content {
		flex: 1;
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 6px;
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

	.enabled-field {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 10px;
		color: var(--colors-text-secondary);
		cursor: pointer;
	}

	.enabled-field input[type="checkbox"] {
		margin: 0;
		cursor: pointer;
	}

	@keyframes spin {
		from {
			transform: rotate(0deg);
		}
		to {
			transform: rotate(360deg);
		}
	}

	.action-btn,
	.close-button {
		background: none;
		border: 1px solid var(--colors-border);
		color: var(--colors-text-secondary);
		padding: 3px 6px;
		cursor: pointer;
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 10px;
		font-family: monospace;
	}

	.action-btn:hover:not(:disabled),
	.close-button:hover {
		border-color: var(--colors-accent);
		color: var(--colors-accent);
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.status-bar {
		height: 28px;
		padding: 0 8px;
		font-size: 12px;
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
