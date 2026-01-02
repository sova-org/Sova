<script lang="ts">
    interface Props {
        checked: boolean;
        onchange: (checked: boolean) => void;
        disabled?: boolean;
        label?: string;
    }

    let { checked, onchange, disabled = false, label }: Props = $props();

    function handleChange(event: Event) {
        const target = event.target as HTMLInputElement;
        onchange(target.checked);
    }
</script>

<label class="toggle-container" class:disabled>
    <input
        type="checkbox"
        {checked}
        {disabled}
        onchange={handleChange}
    />
    <span class="toggle-track">
        <span class="toggle-thumb"></span>
    </span>
    {#if label}
        <span class="toggle-label">{label}</span>
    {/if}
</label>

<style>
    .toggle-container {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        cursor: pointer;
        user-select: none;
    }

    .toggle-container.disabled {
        opacity: 0.5;
        cursor: not-allowed;
    }

    input {
        position: absolute;
        opacity: 0;
        width: 0;
        height: 0;
    }

    .toggle-track {
        position: relative;
        width: 32px;
        height: 16px;
        background-color: var(--colors-border, #333);
        border: 1px solid var(--colors-border, #444);
        transition: background-color 0.15s;
    }

    input:checked + .toggle-track {
        background-color: var(--colors-accent, #0e639c);
        border-color: var(--colors-accent, #0e639c);
    }

    input:focus + .toggle-track {
        border-color: var(--colors-accent, #0e639c);
    }

    .toggle-thumb {
        position: absolute;
        top: 3px;
        left: 3px;
        width: 10px;
        height: 10px;
        background-color: var(--colors-text-secondary, #888);
        transition: transform 0.15s, background-color 0.15s;
    }

    input:checked + .toggle-track .toggle-thumb {
        transform: translateX(16px);
        background-color: var(--colors-text, #fff);
    }

    .toggle-label {
        font-size: 13px;
        font-family: monospace;
        color: var(--colors-text, #fff);
    }
</style>
