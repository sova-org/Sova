<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Check, Save } from 'lucide-svelte';
  import type { Snippet } from 'svelte';
  import { StreamLanguage } from '@codemirror/language';
  import { toml } from '@codemirror/legacy-modes/mode/toml';
  import { keymap } from '@codemirror/view';
  import { autocompletion } from '@codemirror/autocomplete';
  import { invoke } from '@tauri-apps/api/core';
  import { editorConfig } from '$lib/stores/config';
  import { createEditor, createEditorSubscriptions } from '$lib/editor/editorFactory';
  import { tomlThemeCompletion } from '$lib/editor/tomlThemeCompletion';
  import type { EditorView } from '@codemirror/view';

  interface Props {
    registerToolbar?: (snippet: Snippet | null) => void;
  }

  let { registerToolbar }: Props = $props();

  let editorContainer: HTMLDivElement;
  let editorView: EditorView | null = null;
  let unsubscribe: (() => void) | null = null;
  let saveStatus: 'idle' | 'saving' | 'success' | 'error' = 'idle';
  let errorMessage = '';
  let initialized = false;

  async function handleSave() {
    if (!editorView) return;

    const content = editorView.state.doc.toString();
    saveStatus = 'saving';
    errorMessage = '';

    try {
      await invoke('save_config_content', { content });
      saveStatus = 'success';
      setTimeout(() => {
        saveStatus = 'idle';
      }, 2000);
    } catch (error) {
      saveStatus = 'error';
      errorMessage = String(error);
    }
  }

  const saveKeymap = keymap.of([
    {
      key: 'Mod-s',
      run: () => {
        handleSave();
        return true;
      }
    }
  ]);

  // Initialize editor when config becomes available (fixes race condition on app startup)
  $effect(() => {
    const config = $editorConfig;

    if (!config || !editorContainer || initialized) return;

    initialized = true;

    (async () => {
      try {
        const content = await invoke<string>('get_config_content');

        editorView = createEditor(
          editorContainer,
          content,
          StreamLanguage.define(toml),
          config,
          [saveKeymap, autocompletion({ override: [tomlThemeCompletion] })]
        );

        unsubscribe = createEditorSubscriptions(editorView);
      } catch (error) {
        errorMessage = `Failed to load config: ${error}`;
      }
    })();
  });

  onMount(() => {
    registerToolbar?.(toolbarSnippet);
  });

  onDestroy(() => {
    registerToolbar?.(null);
    if (unsubscribe) {
      unsubscribe();
    }
    editorView?.destroy();
  });
</script>

{#snippet toolbarSnippet()}
  <button class="toolbar-btn" onclick={handleSave} disabled={saveStatus === 'saving'} title="Save (Ctrl/Cmd+S)" data-help-id="config-save">
    {#if saveStatus === 'success'}
      <Check size={14} />
    {:else}
      <Save size={14} />
    {/if}
  </button>
{/snippet}

<div class="config-editor">
  {#if saveStatus === 'error' && errorMessage}
    <div class="error-message">
      {errorMessage}
    </div>
  {/if}

  <div class="editor-container" bind:this={editorContainer}></div>
</div>

<style>
  .config-editor {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
  }

  .toolbar-btn {
    background: none;
    border: 1px solid var(--colors-border);
    color: var(--colors-text-secondary);
    padding: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
  }

  .toolbar-btn:hover:not(:disabled) {
    border-color: var(--colors-accent);
    color: var(--colors-accent);
  }

  .toolbar-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .error-message {
    background-color: var(--colors-danger, #5a1d1d);
    color: var(--colors-text, #f48771);
    padding: 8px 16px;
    font-size: 13px;
    font-family: monospace;
    border-bottom: 1px solid var(--colors-border, #721c24);
  }

  .editor-container {
    flex: 1;
    overflow: hidden;
  }

  :global(.cm-editor) {
    height: 100%;
  }
</style>
