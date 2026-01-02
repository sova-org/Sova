<script lang="ts">
    import {
        Check,
        AlertCircle,
        Loader2,
        Send,
        RotateCcw,
        X,
    } from "lucide-svelte";
    import Select from "$lib/components/Select.svelte";
    import { EditorView, keymap } from "@codemirror/view";
    import { editorConfig } from "$lib/stores/config";
    import { availableLanguages } from "$lib/stores/languages";
    import {
        localEdits,
        getLocalEdit,
        setLocalEdit,
        clearLocalEdit,
    } from "$lib/stores/localEdits";
    import { compilationStates } from "$lib/stores/compilation";
    import {
        createEditor,
        createEditorSubscriptions,
        reconfigureLanguage,
    } from "$lib/editor/editorFactory";
    import { getLanguageSupport } from "../../../languages";
    import { setFrames, ActionTiming } from "$lib/api/client";
    import type { Frame, CompilationState } from "$lib/types/protocol";

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
    let isEvaluating = $state(false);

    // Derive effective language from localEdits (if dirty) or frame (source of truth)
    const effectiveLang = $derived(
        frameKey ? ($localEdits.get(frameKey)?.lang ?? frame?.script?.lang ?? "bali") : "bali"
    );
    let evaluationPending = false; // Debounce flag
    let previousFrameKey: string | null = null;

    // isDirty only tracks script content edits (properties sync immediately to server)
    const isDirty = $derived($localEdits.has(frameKey ?? ""));

    // Helper to update frame properties immediately to server (solo-tui pattern)
    async function updateFrameProperty(updates: Partial<Frame>) {
        if (!frame || lineIdx === null || frameIdx === null) return;
        const updatedFrame = { ...frame, ...updates };
        try {
            await setFrames([[lineIdx, frameIdx, updatedFrame]], ActionTiming.immediate());
        } catch (error) {
            console.error("Failed to update frame property:", error);
        }
    }

    // Sync editor content helper
    function syncEditorContent(content: string) {
        if (!editorView) return;
        editorView.dispatch({
            changes: {
                from: 0,
                to: editorView.state.doc.length,
                insert: content,
            },
        });
    }

    // Create update listener that saves to localEdits on every change
    function createUpdateListener() {
        return EditorView.updateListener.of((update) => {
            if (update.docChanged && frameKey) {
                const content = update.state.doc.toString();
                setLocalEdit(frameKey, content, effectiveLang);
            }
        });
    }

    // Create keymap for evaluation shortcuts (Shift+Enter and Cmd/Ctrl+Enter)
    function createEvaluateKeymap() {
        return keymap.of([
            {
                key: "Shift-Enter",
                run: () => {
                    evaluateScript();
                    return true;
                },
            },
            {
                key: "Mod-Enter",
                run: () => {
                    evaluateScript();
                    return true;
                },
            },
        ]);
    }

    // Initialize editor
    $effect(() => {
        if (!editorContainer || !$editorConfig) return;

        if (!editorView) {
            const langSupport = getLanguageSupport(effectiveLang) ?? [];
            editorView = createEditor(
                editorContainer,
                "",
                langSupport,
                $editorConfig,
                [createEvaluateKeymap(), createUpdateListener()],
            );
            unsubscribe = createEditorSubscriptions(editorView);
        }
    });

    // Sync script content when frameKey changes (properties are always from server)
    // Language is now derived via effectiveLang - no manual sync needed
    $effect(() => {
        if (!editorView || !frameKey) return;

        // Only sync when frameKey actually changes
        if (frameKey !== previousFrameKey) {
            previousFrameKey = frameKey;

            // Check for local edit first
            const localEdit = getLocalEdit(frameKey);
            if (localEdit) {
                syncEditorContent(localEdit.content);
            } else {
                // Use server state
                syncEditorContent(frame?.script?.content || "");
            }
        }
    });

    // Reconfigure language when effectiveLang changes
    $effect(() => {
        if (!editorView) return;
        const langSupport = getLanguageSupport(effectiveLang) ?? [];
        reconfigureLanguage(editorView, langSupport);
    });

    function getCompilationStatus(
        state: CompilationState | null | undefined,
    ): "none" | "compiling" | "compiled" | "error" {
        if (!state) return "none";
        if (state === "NotCompiled") return "none";
        if (state === "Compiling") return "compiling";
        if (state === "Compiled" || state === "Parsed") return "compiled";
        if (typeof state === "object" && "Error" in state) return "error";
        return "none";
    }

    function getCompilationError(
        state: CompilationState | null | undefined,
    ): string | null {
        if (state && typeof state === "object" && "Error" in state) {
            return state.Error.info;
        }
        return null;
    }

    function flashEditor() {
        if (!editorContainer) return;
        editorContainer.classList.remove("flash");
        // Force reflow to restart animation
        void editorContainer.offsetWidth;
        editorContainer.classList.add("flash");
    }

    async function evaluateScript() {
        if (
            !frame ||
            lineIdx === null ||
            frameIdx === null ||
            !editorView ||
            !frameKey
        )
            return;
        if (evaluationPending) return; // Prevent concurrent evaluations

        evaluationPending = true;
        isEvaluating = true;
        flashEditor();

        try {
            const content = editorView.state.doc.toString();
            // Only send script changes - properties are already synced immediately
            const updatedFrame: Frame = {
                ...frame,
                script: {
                    ...frame.script,
                    content,
                    lang: effectiveLang,
                },
            };

            await setFrames(
                [[lineIdx, frameIdx, updatedFrame]],
                ActionTiming.immediate(),
            );

            // Clear local edit after successful evaluation
            clearLocalEdit(frameKey);
        } catch (error) {
            console.error("Failed to evaluate script:", error);
        } finally {
            isEvaluating = false;
            evaluationPending = false;
        }
    }

    function discardChanges() {
        if (!frameKey) return;

        // Clear local script edit and re-sync from server
        // Language will automatically derive from frame via effectiveLang
        clearLocalEdit(frameKey);
        syncEditorContent(frame?.script?.content || "");
    }

    // Cleanup editor when component is destroyed
    $effect(() => {
        return () => {
            if (unsubscribe) {
                unsubscribe();
            }
            editorView?.destroy();
        };
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

<div class="editor-pane">
    {#if frame && frameKey}
        <div class="editor-header">
            <div class="header-content">
                <Select
                    options={$availableLanguages}
                    value={effectiveLang}
                    onchange={(lang) => {
                        if (frameKey && editorView) {
                            setLocalEdit(
                                frameKey,
                                editorView.state.doc.toString(),
                                lang,
                            );
                        }
                    }}
                />

                <label class="prop-field">
                    <span>dur</span>
                    <input
                        type="number"
                        class="prop-input"
                        value={frame?.duration ?? 1}
                        min="0.125"
                        step="0.25"
                        onchange={(e) => updateFrameProperty({ duration: parseFloat(e.currentTarget.value) || 1 })}
                    />
                </label>

                <label class="prop-field">
                    <span>×</span>
                    <input
                        type="number"
                        class="prop-input"
                        value={frame?.repetitions ?? 1}
                        min="1"
                        step="1"
                        onchange={(e) => updateFrameProperty({ repetitions: parseInt(e.currentTarget.value) || 1 })}
                    />
                </label>

                <label class="prop-field">
                    <span>name</span>
                    <input
                        type="text"
                        class="prop-input name"
                        value={frame?.name ?? ""}
                        placeholder="F{frameIdx}"
                        onchange={(e) => updateFrameProperty({ name: e.currentTarget.value || null })}
                    />
                </label>

                <label class="enabled-field" data-help-id="frame-enabled">
                    <input
                        type="checkbox"
                        checked={frame?.enabled ?? true}
                        onchange={(e) => updateFrameProperty({ enabled: e.currentTarget.checked })}
                    />
                    <span>Enabled</span>
                </label>

                {#if isDirty}
                    <button
                        class="action-btn"
                        data-help-id="frame-fetch"
                        onclick={discardChanges}
                        title="Discard script changes"
                    >
                        <RotateCcw size={12} />
                        Discard
                    </button>
                {/if}

                <button
                    class="action-btn"
                    data-help-id="frame-evaluate"
                    onclick={evaluateScript}
                    disabled={isEvaluating}
                    title="Evaluate script (Cmd+Enter)"
                >
                    <Send size={12} />
                    Eval
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
            class:compiled={compilationStatus === "compiled"}
            class:error={compilationStatus === "error"}
            class:compiling={compilationStatus === "compiling" || isEvaluating}
        >
            {#if compilationStatus === "compiling" || isEvaluating}
                <Loader2 size={12} class="spin" /> Compiling...
            {:else if compilationStatus === "compiled"}
                <Check size={12} /> Compiled
            {:else if compilationStatus === "error"}
                <AlertCircle size={12} />
                {compilationError || "Compilation error"}
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
        user-select: none;
        -webkit-user-select: none;
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
        user-select: text;
        -webkit-user-select: text;
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
        user-select: none;
        -webkit-user-select: none;
    }

    .status-bar.compiled {
        color: var(--colors-success, #4caf50);
    }

    .status-bar.error {
        background-color: color-mix(
            in srgb,
            var(--colors-error, #f44336) 15%,
            var(--colors-surface)
        );
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
        user-select: none;
        -webkit-user-select: none;
    }

    .editor-container {
        flex: 1;
        overflow: hidden;
    }

    :global(.editor-container .cm-editor) {
        height: 100%;
        user-select: text;
        -webkit-user-select: text;
    }

    @keyframes flash-evaluate {
        0% {
            background-color: var(--colors-accent);
        }
        100% {
            background-color: transparent;
        }
    }

    :global(.editor-container.flash .cm-editor) {
        animation: flash-evaluate 0.15s ease-out;
    }
</style>
