<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { Check } from 'lucide-svelte';
  import { StreamLanguage } from '@codemirror/language';
  import { toml } from '@codemirror/legacy-modes/mode/toml';
  import { keymap } from '@codemirror/view';
  import { autocompletion } from '@codemirror/autocomplete';
  import { invoke } from '@tauri-apps/api/core';
  import { editorConfig, currentTheme } from '$lib/stores/config';
  import { createEditor, createEditorSubscriptions } from '$lib/editor/editorFactory';
  import { tomlThemeCompletion } from '$lib/editor/tomlThemeCompletion';
  import type { EditorView } from '@codemirror/view';

  let editorContainer: HTMLDivElement;
  let editorView: EditorView | null = null;
  let unsubscribe: (() => void) | null = null;
  let saveStatus: 'idle' | 'saving' | 'success' | 'error' = 'idle';
  let errorMessage = '';

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

  onMount(async () => {
    const config = $editorConfig;
    const theme = $currentTheme;

    try {
      const content = await invoke<string>('get_config_content');

      editorView = createEditor(
        editorContainer,
        content,
        StreamLanguage.define(toml),
        config,
        theme,
        [saveKeymap, autocompletion({ override: [tomlThemeCompletion] })]
      );

      unsubscribe = createEditorSubscriptions(editorView);
    } catch (error) {
      console.error('Failed to load config:', error);
      errorMessage = `Failed to load config: ${error}`;
    }
  });

  onDestroy(() => {
    if (unsubscribe) {
      unsubscribe();
    }
    editorView?.destroy();
  });
</script>

<div class="config-editor">
  <div class="toolbar">
    <button class="save-button" onclick={handleSave} disabled={saveStatus === 'saving'}>
      {#if saveStatus === 'saving'}
        Saving...
      {:else if saveStatus === 'success'}
        <span class="saved-status">
          Saved <Check size={14} />
        </span>
      {:else}
        Save
      {/if}
    </button>
    <span class="shortcut">Ctrl/Cmd+S</span>
  </div>

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

  .toolbar {
    height: 40px;
    background-color: var(--colors-surface, #252525);
    border-bottom: 1px solid var(--colors-border, #333);
    display: flex;
    align-items: center;
    padding: 0 16px;
    gap: 12px;
  }

  .save-button {
    background-color: var(--colors-accent, #0e639c);
    color: var(--colors-text, #fff);
    border: none;
    padding: 6px 16px;
    font-size: 13px;
    cursor: pointer;
    font-family: monospace;
  }

  .save-button:hover:not(:disabled) {
    background-color: var(--colors-accent-hover, #1177bb);
  }

  .save-button:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .saved-status {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }

  .shortcut {
    color: var(--colors-text-secondary, #888);
    font-size: 12px;
    font-family: monospace;
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
