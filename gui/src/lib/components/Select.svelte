<script lang="ts">
    import { ChevronDown } from "lucide-svelte";

    interface Props {
        options: string[];
        value: string;
        onchange: (_value: string) => void;
        placeholder?: string;
        disabled?: boolean;
    }

    let {
        options,
        value,
        onchange,
        placeholder = "Select...",
        disabled = false,
    }: Props = $props();

    const uid = $props.id();

    let isOpen = $state(false);
    let highlightedIndex = $state(-1);
    let triggerEl: HTMLButtonElement;
    // svelte-ignore non_reactive_update
    let menuEl: HTMLDivElement;
    let optionEls: HTMLButtonElement[] = [];
    let menuPosition = $state({ top: 0, left: 0, width: 0, flipUp: false });

    let menuStyle = $derived.by(() => {
        const { top, left, width, flipUp } = menuPosition;
        if (flipUp) {
            return `bottom: ${window.innerHeight - top}px; left: ${left}px; min-width: ${width}px;`;
        }
        return `top: ${top}px; left: ${left}px; min-width: ${width}px;`;
    });

    function calculatePosition() {
        if (!triggerEl) return;
        const rect = triggerEl.getBoundingClientRect();
        const menuHeight = 200;
        const spaceBelow = window.innerHeight - rect.bottom;
        const spaceAbove = rect.top;
        const flipUp = spaceBelow < menuHeight && spaceAbove > spaceBelow;

        menuPosition = {
            top: flipUp ? rect.top : rect.bottom,
            left: rect.left,
            width: rect.width,
            flipUp,
        };
    }

    function open() {
        if (disabled) return;
        calculatePosition();
        isOpen = true;
        highlightedIndex = options.indexOf(value);
        if (highlightedIndex === -1) highlightedIndex = 0;
    }

    function close() {
        isOpen = false;
        highlightedIndex = -1;
    }

    function toggle() {
        if (isOpen) close();
        else open();
    }

    function select(option: string) {
        onchange(option);
        close();
        triggerEl?.focus();
    }

    function handleKeydown(event: KeyboardEvent) {
        if (disabled) return;

        switch (event.key) {
            case "ArrowDown":
                event.preventDefault();
                if (!isOpen) {
                    open();
                } else {
                    highlightedIndex = Math.min(
                        highlightedIndex + 1,
                        options.length - 1,
                    );
                }
                break;
            case "ArrowUp":
                event.preventDefault();
                if (isOpen) {
                    highlightedIndex = Math.max(highlightedIndex - 1, 0);
                }
                break;
            case "Enter":
            case " ":
                event.preventDefault();
                if (!isOpen) {
                    open();
                } else if (
                    highlightedIndex >= 0 &&
                    highlightedIndex < options.length
                ) {
                    select(options[highlightedIndex]);
                }
                break;
            case "Escape":
                event.preventDefault();
                close();
                break;
            case "Tab":
                close();
                break;
        }
    }

    $effect(() => {
        if (!isOpen) return;

        function handleClickOutside(event: MouseEvent) {
            const target = event.target as Node;
            if (
                triggerEl &&
                !triggerEl.contains(target) &&
                menuEl &&
                !menuEl.contains(target)
            ) {
                close();
            }
        }

        document.addEventListener("mousedown", handleClickOutside);
        return () =>
            document.removeEventListener("mousedown", handleClickOutside);
    });

    $effect(() => {
        if (isOpen && highlightedIndex >= 0) {
            optionEls[highlightedIndex]?.scrollIntoView({ block: "nearest" });
        }
    });
</script>

<button
    bind:this={triggerEl}
    class="select-trigger"
    class:open={isOpen}
    class:disabled
    type="button"
    role="combobox"
    aria-expanded={isOpen}
    aria-haspopup="listbox"
    aria-controls="{uid}-listbox"
    aria-disabled={disabled}
    onclick={toggle}
    onkeydown={handleKeydown}
>
    <span class="select-value" class:placeholder={!value}>
        {value || placeholder}
    </span>
    <ChevronDown size={14} class="select-chevron" />
</button>

{#if isOpen}
    <div
        bind:this={menuEl}
        id="{uid}-listbox"
        class="select-menu"
        class:flip-up={menuPosition.flipUp}
        style={menuStyle}
        role="listbox"
    >
        {#each options as option, i (option)}
            <button
                bind:this={optionEls[i]}
                type="button"
                class="select-option"
                class:selected={option === value}
                class:highlighted={i === highlightedIndex}
                role="option"
                aria-selected={option === value}
                onclick={() => select(option)}
                onmouseenter={() => (highlightedIndex = i)}
            >
                {option}
            </button>
        {/each}
    </div>
{/if}

<style>
    .select-trigger {
        appearance: none;
        background-color: var(--colors-background);
        border: 1px solid var(--colors-border);
        color: var(--colors-text);
        font-size: 13px;
        font-family: monospace;
        padding: 8px 28px 8px 10px;
        cursor: pointer;
        display: inline-flex;
        align-items: center;
        gap: 4px;
        position: relative;
        text-align: left;
        width: 100%;
    }

    .select-trigger:hover:not(.disabled) {
        border-color: var(--colors-accent);
    }

    .select-trigger:focus {
        outline: none;
        border-color: var(--colors-accent);
    }

    .select-trigger.disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    .select-value {
        flex: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .select-value.placeholder {
        color: var(--colors-text-secondary);
    }

    :global(.select-chevron) {
        position: absolute;
        right: 10px;
        transition: transform 0.15s;
    }

    .select-trigger.open :global(.select-chevron) {
        transform: rotate(180deg);
    }

    .select-menu {
        position: fixed;
        background-color: var(--colors-background);
        border: 1px solid var(--colors-border);
        z-index: 9999;
        max-height: 200px;
        overflow-y: auto;
        display: flex;
        flex-direction: column;
    }

    .select-option {
        appearance: none;
        background: none;
        border: none;
        padding: 8px 10px;
        cursor: pointer;
        font-size: 13px;
        font-family: monospace;
        color: var(--colors-text);
        text-align: left;
        width: 100%;
    }

    .select-option:hover,
    .select-option.highlighted {
        background-color: var(--colors-surface);
    }

    .select-option.selected {
        color: var(--colors-accent);
    }
</style>
