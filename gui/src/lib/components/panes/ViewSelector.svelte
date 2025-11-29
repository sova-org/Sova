<script lang="ts">
	import { Monitor, Settings, Users, FileText, LogIn, MessageCircle, FolderOpen } from 'lucide-svelte';
	import { availableViews, type ViewType } from '$lib/stores/paneState';

	interface Props {
		onSelect: (view: ViewType) => void;
	}

	let { onSelect }: Props = $props();

	const viewIcons: Record<ViewType, typeof Monitor> = {
		LOGIN: LogIn,
		SCENE: Monitor,
		DEVICES: Users,
		LOGS: FileText,
		CONFIG: Settings,
		CHAT: MessageCircle,
		SNAPSHOTS: FolderOpen
	};

	const viewDescriptions: Record<ViewType, string> = {
		LOGIN: 'Connect to server',
		SCENE: 'Timeline and editor',
		DEVICES: 'MIDI and OSC devices',
		LOGS: 'System logs',
		CONFIG: 'Application settings',
		CHAT: 'Peer messages',
		SNAPSHOTS: 'Save and load projects'
	};
</script>

<div class="view-selector">
	<div class="selector-grid">
		{#each $availableViews as view}
			{@const Icon = viewIcons[view]}
			<button class="view-option" onclick={() => onSelect(view)}>
				<Icon size={24} />
				<span class="view-name">{view}</span>
				<span class="view-desc">{viewDescriptions[view]}</span>
			</button>
		{/each}
	</div>
</div>

<style>
	.view-selector {
		width: 100%;
		height: 100%;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: var(--colors-background);
	}

	.selector-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
		gap: 16px;
		max-width: 400px;
		padding: 24px;
	}

	.view-option {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		padding: 20px 16px;
		background: var(--colors-surface);
		border: 1px solid var(--colors-border);
		color: var(--colors-text);
		cursor: pointer;
		transition: all 0.2s;
	}

	.view-option:hover {
		border-color: var(--colors-accent);
		background: var(--colors-background);
	}

	.view-name {
		font-family: monospace;
		font-size: 12px;
		font-weight: 600;
	}

	.view-desc {
		font-family: monospace;
		font-size: 10px;
		color: var(--colors-text-secondary);
		text-align: center;
	}
</style>
