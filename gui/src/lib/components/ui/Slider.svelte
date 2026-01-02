<script lang="ts">
    interface Props {
        value: number;
        min?: number;
        max?: number;
        step?: number;
        onchange: (value: number) => void;
        disabled?: boolean;
        label?: string;
        showValue?: boolean;
    }

    let {
        value,
        min = 0,
        max = 100,
        step = 1,
        onchange,
        disabled = false,
        label,
        showValue = true,
    }: Props = $props();

    function handleInput(event: Event) {
        const target = event.target as HTMLInputElement;
        onchange(Number(target.value));
    }
</script>

<div class="slider-container" class:disabled>
    {#if label}
        <div class="slider-header">
            <span class="slider-label">{label}</span>
            {#if showValue}
                <span class="slider-value">{value}</span>
            {/if}
        </div>
    {/if}
    <input
        type="range"
        {value}
        {min}
        {max}
        {step}
        {disabled}
        oninput={handleInput}
    />
</div>

<style>
    .slider-container {
        display: flex;
        flex-direction: column;
        gap: 4px;
        width: 100%;
    }

    .slider-container.disabled {
        opacity: 0.5;
    }

    .slider-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
    }

    .slider-label {
        font-size: 13px;
        font-family: monospace;
        color: var(--colors-text, #fff);
    }

    .slider-value {
        font-size: 12px;
        font-family: monospace;
        color: var(--colors-text-secondary, #888);
    }

    input[type="range"] {
        -webkit-appearance: none;
        appearance: none;
        width: 100%;
        height: 4px;
        background: var(--colors-border, #333);
        outline: none;
        cursor: pointer;
    }

    input[type="range"]:disabled {
        cursor: not-allowed;
    }

    input[type="range"]::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 12px;
        height: 12px;
        background: var(--colors-text, #fff);
        cursor: pointer;
        border: none;
    }

    input[type="range"]::-moz-range-thumb {
        width: 12px;
        height: 12px;
        background: var(--colors-text, #fff);
        cursor: pointer;
        border: none;
    }

    input[type="range"]:focus::-webkit-slider-thumb {
        background: var(--colors-accent, #0e639c);
    }

    input[type="range"]:focus::-moz-range-thumb {
        background: var(--colors-accent, #0e639c);
    }
</style>
