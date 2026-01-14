<script lang="ts">
    import { audioEngineState } from "$lib/stores/audioEngineState";
    import { isConnected } from "$lib/stores/connectionState";
</script>

<div class="bottombar">
    <div class="left-section">
        {#if $isConnected}
            <span class="status-dot" class:running={$audioEngineState.running}></span>
            <span class="status-label">Audio</span>
            {#if $audioEngineState.running}
                <span class="status-details">
                    {$audioEngineState.sample_rate} Hz · {$audioEngineState.channels} ch
                </span>
            {:else}
                <span class="status-details dimmed">Stopped</span>
            {/if}
        {/if}
    </div>
    <div class="middle-section"></div>
    <div class="right-section"></div>
</div>

<style>
    .bottombar {
        height: 40px;
        display: grid;
        grid-template-columns: 1fr auto 1fr;
        align-items: center;
        padding: 0 12px;
        background-color: var(--colors-background);
        border-top: 1px solid var(--colors-border);
        font-family: var(--appearance-font-family);
    }

    .left-section {
        display: flex;
        align-items: center;
        gap: 8px;
    }

    .middle-section {
        display: flex;
        align-items: center;
        gap: 16px;
    }

    .right-section {
        display: flex;
        align-items: center;
        justify-content: flex-end;
        gap: 8px;
    }

    .status-dot {
        width: 8px;
        height: 8px;
        background-color: var(--colors-text-secondary, #888);
    }

    .status-dot.running {
        background-color: #4caf50;
    }

    .status-label {
        font-size: 11px;
        color: var(--colors-text-secondary, #888);
    }

    .status-details {
        font-size: 11px;
        color: var(--colors-text, #fff);
    }

    .status-details.dimmed {
        color: var(--colors-text-secondary, #888);
    }
</style>
