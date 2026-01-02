<script lang="ts">
    import { onDestroy } from "svelte";
    import FrameEditor from "./scene/FrameEditor.svelte";
    import {
        editingFrame,
        currentEditingFrame,
        editingFrameKey,
        closeEditor,
    } from "$lib/stores/editingFrame";

    onDestroy(() => {
        closeEditor();
    });
</script>

<div class="editor-view">
    <FrameEditor
        frame={$currentEditingFrame}
        frameKey={$editingFrameKey}
        lineIdx={$editingFrame?.lineIdx ?? null}
        frameIdx={$editingFrame?.frameIdx ?? null}
    />
</div>

<style>
    .editor-view {
        width: 100%;
        height: 100%;
    }
</style>
