<script lang="ts">
    interface Props {
        languages: string[];
        current: string;
        onSelect: (lang: string) => void;
        onClose: () => void;
    }

    let { languages, current, onSelect, onClose }: Props = $props();
    let highlightedIndex = $state(languages.indexOf(current));
    let pickerEl: HTMLDivElement;

    function handleKeydown(e: KeyboardEvent) {
        if (e.key >= "1" && e.key <= "9") {
            const idx = parseInt(e.key) - 1;
            if (idx < languages.length) {
                onSelect(languages[idx]);
            }
            e.preventDefault();
            return;
        }

        switch (e.key) {
            case "ArrowDown":
            case "j":
                highlightedIndex = Math.min(highlightedIndex + 1, languages.length - 1);
                e.preventDefault();
                break;
            case "ArrowUp":
            case "k":
                highlightedIndex = Math.max(highlightedIndex - 1, 0);
                e.preventDefault();
                break;
            case "Enter":
                onSelect(languages[highlightedIndex]);
                e.preventDefault();
                break;
            case "Escape":
                onClose();
                e.preventDefault();
                break;
        }
    }

    $effect(() => {
        pickerEl?.focus();
    });
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="overlay" onclick={onClose} onkeydown={handleKeydown}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
        class="picker"
        bind:this={pickerEl}
        tabindex="-1"
        onclick={(e) => e.stopPropagation()}
        role="listbox"
        aria-label="Select language"
    >
        {#each languages as lang, i}
            <button
                class="option"
                class:highlighted={i === highlightedIndex}
                class:current={lang === current}
                onclick={() => onSelect(lang)}
                role="option"
                aria-selected={lang === current}
            >
                <span class="number">{i + 1}.</span>
                <span class="lang">{lang}</span>
                {#if lang === current}
                    <span class="marker">&#9664;</span>
                {/if}
            </button>
        {/each}
    </div>
</div>

<style>
    .overlay {
        position: fixed;
        inset: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: flex-start;
        justify-content: center;
        padding-top: 20vh;
        z-index: 9999;
    }

    .picker {
        background: var(--colors-background);
        border: 1px solid var(--colors-border);
        box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
        min-width: 160px;
        padding: 8px 0;
        outline: none;
    }

    .option {
        display: flex;
        align-items: center;
        gap: 8px;
        width: 100%;
        padding: 8px 16px;
        background: none;
        border: none;
        color: var(--colors-text);
        font-family: monospace;
        font-size: 13px;
        cursor: pointer;
        text-align: left;
    }

    .option:hover,
    .option.highlighted {
        background: var(--colors-accent);
        color: var(--colors-background);
    }

    .number {
        opacity: 0.6;
        min-width: 18px;
    }

    .lang {
        flex: 1;
    }

    .marker {
        font-size: 10px;
        opacity: 0.8;
    }
</style>
