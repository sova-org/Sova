<script lang="ts">
	import {
		FolderOpen,
		Cpu,
		MessageSquare,
		FileText,
		Settings,
		PanelLeftClose,
		PanelRightClose,
		ArrowLeftRight,
	} from 'lucide-svelte';
	import {
		sidebarState,
		sidebarActiveTab,
		sidebarSide,
		availableSidebarTabs,
		type SidebarTab,
	} from '$lib/stores/sidebarState';

	const tabIcons: Record<SidebarTab, typeof FolderOpen> = {
		PROJECTS: FolderOpen,
		DEVICES: Cpu,
		CHAT: MessageSquare,
		LOGS: FileText,
		CONFIG: Settings,
	};

	const tabLabels: Record<SidebarTab, string> = {
		PROJECTS: 'Projects',
		DEVICES: 'Devices',
		CHAT: 'Chat',
		LOGS: 'Logs',
		CONFIG: 'Config',
	};
</script>

<div class="sidebar-tabs">
	<div class="tabs">
		{#each $availableSidebarTabs as tab (tab)}
			<button
				class="tab-btn"
				class:active={$sidebarActiveTab === tab}
				onclick={() => sidebarState.setTab(tab)}
				title={tabLabels[tab]}
			>
				<svelte:component this={tabIcons[tab]} size={16} />
			</button>
		{/each}
	</div>

	<div class="actions">
		<button
			class="action-btn"
			onclick={() => sidebarState.toggleSide()}
			title="Switch side"
		>
			<ArrowLeftRight size={14} />
		</button>
		<button
			class="action-btn"
			onclick={() => sidebarState.close()}
			title="Close sidebar"
		>
			{#if $sidebarSide === 'left'}
				<PanelLeftClose size={14} />
			{:else}
				<PanelRightClose size={14} />
			{/if}
		</button>
	</div>
</div>

<style>
	.sidebar-tabs {
		height: 40px;
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 0 8px;
		background-color: var(--colors-surface, #252525);
		border-bottom: 1px solid var(--colors-border, #333);
	}

	.tabs {
		display: flex;
		gap: 2px;
	}

	.tab-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 28px;
		background: none;
		border: none;
		color: var(--colors-text-secondary, #888);
		cursor: pointer;
		transition: color 0.15s, background-color 0.15s;
	}

	.tab-btn:hover {
		color: var(--colors-text, #fff);
		background-color: var(--colors-background, #1e1e1e);
	}

	.tab-btn.active {
		color: var(--colors-text, #fff);
		background-color: var(--colors-accent, #0e639c);
	}

	.actions {
		display: flex;
		gap: 4px;
	}

	.action-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		background: none;
		border: none;
		color: var(--colors-text-secondary, #888);
		cursor: pointer;
		transition: color 0.15s;
	}

	.action-btn:hover {
		color: var(--colors-text, #fff);
	}
</style>
