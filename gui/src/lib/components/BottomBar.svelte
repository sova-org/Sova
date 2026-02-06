<script lang="ts">
	import { audioEngineState } from '$lib/stores/audioEngineState';
	import { isConnected } from '$lib/stores/connectionState';
	import Scope from './Scope.svelte';
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
	<div class="middle-section">
		{#if $isConnected && $audioEngineState.running}
			<Scope />
		{/if}
	</div>
	<div class="right-section">
		{#if $isConnected && $audioEngineState.running}
			<span class="telemetry">
				CPU {($audioEngineState.cpu_load * 100).toFixed(0)}%
			</span>
			<span class="telemetry">
				{$audioEngineState.active_voices}/{$audioEngineState.max_voices} voices
			</span>
		{/if}
	</div>
</div>

<style>
	.bottombar {
		height: 40px;
		display: grid;
		grid-template-columns: auto 1fr auto;
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
		padding-right: 16px;
	}

	.middle-section {
		height: 32px;
		min-width: 0;
	}

	.right-section {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		padding-left: 16px;
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

	.telemetry {
		font-size: 11px;
		color: var(--colors-text-secondary, #888);
	}
</style>
