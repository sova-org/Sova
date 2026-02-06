<script lang="ts">
    interface Props {
        value: number;
        min?: number;
        max?: number;
        step?: number;
        onchange: (value: number) => void;
        disabled?: boolean;
        label?: string;
        placeholder?: string;
    }

    let {
        value,
        min,
        max,
        step = 1,
        onchange,
        disabled = false,
        label,
        placeholder,
    }: Props = $props();

    const inputId = `number-input-${Math.random().toString(36).slice(2, 9)}`;

    function handleInput(event: Event) {
        const target = event.target as HTMLInputElement;
        const numValue = Number(target.value);
        if (!isNaN(numValue)) {
            onchange(numValue);
        }
    }
</script>

<div class="number-input-container" class:disabled>
    {#if label}
        <label class="number-input-label" for={inputId}>{label}</label>
    {/if}
    <input
        id={inputId}
        type="number"
        {value}
        {min}
        {max}
        {step}
        {disabled}
        {placeholder}
        oninput={handleInput}
    />
</div>

<style>
    .number-input-container {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .number-input-container.disabled {
        opacity: 0.5;
    }

    .number-input-label {
        font-size: 13px;
        font-family: monospace;
        color: var(--colors-text, #fff);
    }

    input {
        background-color: var(--colors-background, #1e1e1e);
        color: var(--colors-text, #fff);
        border: 1px solid var(--colors-border, #333);
        padding: 8px 10px;
        font-size: 13px;
        font-family: monospace;
        width: 100%;
        box-sizing: border-box;
    }

    input:focus {
        outline: none;
        border-color: var(--colors-accent, #0e639c);
    }

    input:disabled {
        cursor: not-allowed;
    }
</style>
