<script lang="ts">
    import { onDestroy } from "svelte";
    import { javascript } from "@codemirror/lang-javascript";
    import { editorConfig } from "$lib/stores/config";
    import {
        createEditor,
        createEditorSubscriptions,
    } from "$lib/editor/editorFactory";
    import type { EditorView } from "@codemirror/view";

    let editorContainer: HTMLDivElement;
    let editorView: EditorView | null = null;
    let unsubscribe: (() => void) | null = null;

    $effect(() => {
        if (!editorContainer || !$editorConfig) {
            return;
        }

        if (editorView) {
            return;
        }

        editorView = createEditor(
            editorContainer,
            "// Start coding...\n",
            javascript(),
            $editorConfig,
        );

        unsubscribe = createEditorSubscriptions(editorView);
    });

    onDestroy(() => {
        if (unsubscribe) {
            unsubscribe();
        }
        editorView?.destroy();
    });
</script>

<div class="editor-wrapper">
    <div class="editor-container" bind:this={editorContainer}></div>
</div>

<style>
    .editor-wrapper {
        width: 100%;
        height: 100%;
        overflow: hidden;
    }

    .editor-container {
        width: 100%;
        height: 100%;
    }

    :global(.cm-editor) {
        height: 100%;
    }
</style>
