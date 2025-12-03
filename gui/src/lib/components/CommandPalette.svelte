<script lang="ts">
    import { commandPalette, filteredCommands } from "$lib/stores/commandPalette";

    let inputElement: HTMLInputElement | null = $state(null);
    let query = $state("");

    const isOpen = $derived($commandPalette.isOpen);
    const selectedIndex = $derived($commandPalette.selectedIndex);
    const commands = $derived($filteredCommands);

    function handleGlobalKeydown(event: KeyboardEvent) {
        if ((event.metaKey || event.ctrlKey) && event.key === "k") {
            event.preventDefault();
            if (isOpen) {
                commandPalette.close();
            } else {
                commandPalette.open();
                query = "";
            }
        }
    }

    function handleInputKeydown(event: KeyboardEvent) {
        switch (event.key) {
            case "Escape":
                event.preventDefault();
                commandPalette.close();
                break;
            case "ArrowDown":
                event.preventDefault();
                commandPalette.selectNext(commands.length - 1);
                break;
            case "ArrowUp":
                event.preventDefault();
                commandPalette.selectPrev();
                break;
            case "Enter":
                event.preventDefault();
                commandPalette.executeSelected(commands);
                break;
        }
    }

    function handleQueryChange() {
        commandPalette.setQuery(query);
    }

    function handleOverlayClick() {
        commandPalette.close();
    }

    function handleCommandClick(index: number) {
        commandPalette.setSelectedIndex(index);
        commandPalette.executeSelected(commands);
    }

    function handleCommandHover(index: number) {
        commandPalette.setSelectedIndex(index);
    }

    $effect(() => {
        if (isOpen && inputElement) {
            inputElement.focus();
        }
    });
</script>

<svelte:window onkeydown={handleGlobalKeydown} />

{#if isOpen}
    <div
        class="overlay"
        onclick={handleOverlayClick}
        onkeydown={(e) => e.key === "Escape" && commandPalette.close()}
        role="presentation"
    >
        <div
            class="palette"
            onclick={(e) => e.stopPropagation()}
            onkeydown={(e) => e.stopPropagation()}
            role="dialog"
            aria-modal="true"
            aria-label="Command palette"
        >
            <input
                bind:this={inputElement}
                bind:value={query}
                oninput={handleQueryChange}
                onkeydown={handleInputKeydown}
                type="text"
                class="palette-input"
                placeholder="Type a command..."
                role="combobox"
                aria-expanded="true"
                aria-controls="command-list"
                aria-activedescendant={commands[selectedIndex]
                    ? `cmd-${commands[selectedIndex].id}`
                    : undefined}
                aria-autocomplete="list"
            />
            {#if commands.length > 0}
                <ul id="command-list" class="command-list" role="listbox">
                    {#each commands as cmd, i (cmd.id)}
                        <li
                            id="cmd-{cmd.id}"
                            class="command-item"
                            class:selected={i === selectedIndex}
                            role="option"
                            aria-selected={i === selectedIndex}
                            onclick={() => handleCommandClick(i)}
                            onmouseenter={() => handleCommandHover(i)}
                        >
                            <span class="command-name">{cmd.name}</span>
                            <span class="command-description"
                                >{cmd.description}</span
                            >
                        </li>
                    {/each}
                </ul>
            {:else if query.length > 0}
                <div class="no-results">No commands found</div>
            {/if}
        </div>
    </div>
{/if}

<style>
    .overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: flex-start;
        justify-content: center;
        padding-top: 15vh;
        z-index: 9999;
    }

    .palette {
        background: var(--colors-background, #1e1e1e);
        border: 1px solid var(--colors-border, #333);
        width: 100%;
        max-width: 500px;
        margin: 0 16px;
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    }

    .palette-input {
        width: 100%;
        box-sizing: border-box;
        font-family: var(--appearance-font-family);
        font-size: 14px;
        padding: 12px 16px;
        background: var(--colors-surface, #2d2d2d);
        border: none;
        border-bottom: 1px solid var(--colors-border, #333);
        color: var(--colors-text, #fff);
        outline: none;
    }

    .palette-input::placeholder {
        color: var(--colors-text-secondary, #888);
    }

    .command-list {
        list-style: none;
        margin: 0;
        padding: 0;
        max-height: 300px;
        overflow-y: auto;
    }

    .command-item {
        display: flex;
        flex-direction: column;
        gap: 2px;
        padding: 10px 16px;
        cursor: pointer;
        transition: background-color 0.1s;
    }

    .command-item:hover {
        background: var(--colors-surface, #2d2d2d);
    }

    .command-item.selected {
        background: var(--colors-accent, #0e639c);
    }

    .command-item.selected .command-description {
        color: rgba(255, 255, 255, 0.8);
    }

    .command-name {
        font-family: var(--appearance-font-family);
        font-size: 13px;
        font-weight: 500;
        color: var(--colors-text, #fff);
    }

    .command-description {
        font-family: var(--appearance-font-family);
        font-size: 11px;
        color: var(--colors-text-secondary, #888);
    }

    .no-results {
        font-family: var(--appearance-font-family);
        font-size: 12px;
        color: var(--colors-text-secondary, #888);
        padding: 16px;
        text-align: center;
    }

    @media (max-width: 600px) {
        .palette {
            max-width: 100%;
            margin: 0 8px;
        }
    }
</style>
