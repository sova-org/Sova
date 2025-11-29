<script lang="ts">
	import { Columns2, Rows2, X, LayoutGrid, RotateCcw } from 'lucide-svelte';
	import { paneLayout, type ViewType } from '$lib/stores/paneState';
	import ViewSelector from './ViewSelector.svelte';
	import ConfigEditor from '../ConfigEditor.svelte';
	import Login from '../Login.svelte';
	import DevicesView from '../DevicesView.svelte';
	import LogView from '../LogView.svelte';
	import SceneView from '../SceneView.svelte';
	import ChatView from '../ChatView.svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		paneId: string;
		viewType: ViewType | null;
		isOnlyPane: boolean;
	}

	let { paneId, viewType, isOnlyPane }: Props = $props();

	let toolbarControls: Snippet | null = $state(null);

	const viewTitles: Record<ViewType, string> = {
		LOGIN: 'Login',
		SCENE: 'Scene',
		DEVICES: 'Devices',
		LOGS: 'Logs',
		CONFIG: 'Config',
		CHAT: 'Chat'
	};

	function handleSplitHorizontal() {
		paneLayout.splitPane(paneId, 'horizontal');
	}

	function handleSplitVertical() {
		paneLayout.splitPane(paneId, 'vertical');
	}

	function handleClose() {
		paneLayout.closePane(paneId);
	}

	function handleClearView() {
		paneLayout.setView(paneId, null);
	}

	function handleToggleDirection() {
		paneLayout.toggleParentDirection(paneId);
	}

	function handleViewSelect(view: ViewType) {
		paneLayout.setView(paneId, view);
	}

	function handleLoginSuccess() {
		paneLayout.setView(paneId, 'SCENE');
	}

	function registerToolbar(snippet: Snippet | null) {
		toolbarControls = snippet;
	}
</script>

<div class="pane-container">
	<div class="pane-header">
		<span class="pane-title">
			{viewType ? viewTitles[viewType] : 'Select View'}
		</span>
		<div class="pane-controls">
			{#if toolbarControls}
				{@render toolbarControls()}
				<div class="separator"></div>
			{/if}
			<div class="pane-actions">
				{#if viewType !== null}
					<button class="action-btn" onclick={handleClearView} title="Change View">
						<LayoutGrid size={14} />
					</button>
				{/if}
				<button class="action-btn" onclick={handleSplitVertical} title="Split Vertical">
					<Columns2 size={14} />
				</button>
				<button class="action-btn" onclick={handleSplitHorizontal} title="Split Horizontal">
					<Rows2 size={14} />
				</button>
				{#if !isOnlyPane}
					<button class="action-btn" onclick={handleToggleDirection} title="Toggle Split Direction">
						<RotateCcw size={14} />
					</button>
					<button class="action-btn close" onclick={handleClose} title="Close Pane">
						<X size={14} />
					</button>
				{/if}
			</div>
		</div>
	</div>

	<div class="pane-content">
		{#if viewType === null}
			<ViewSelector onSelect={handleViewSelect} />
		{:else if viewType === 'LOGIN'}
			<Login onConnected={handleLoginSuccess} />
		{:else if viewType === 'SCENE'}
			<SceneView {registerToolbar} />
		{:else if viewType === 'DEVICES'}
			<DevicesView />
		{:else if viewType === 'LOGS'}
			<LogView />
		{:else if viewType === 'CONFIG'}
			<ConfigEditor {registerToolbar} />
		{:else if viewType === 'CHAT'}
			<ChatView />
		{/if}
	</div>
</div>

<style>
	.pane-container {
		width: 100%;
		height: 100%;
		display: flex;
		flex-direction: column;
		background-color: var(--colors-background);
	}

	.pane-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: 28px;
		padding: 0 8px;
		background-color: var(--colors-surface);
		border-bottom: 1px solid var(--colors-border);
		flex-shrink: 0;
	}

	.pane-title {
		font-family: monospace;
		font-size: 11px;
		font-weight: 600;
		color: var(--colors-text-secondary);
		text-transform: uppercase;
		letter-spacing: 0.5px;
	}

	.pane-controls {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.separator {
		width: 1px;
		height: 16px;
		background-color: var(--colors-border);
	}

	.pane-actions {
		display: flex;
		gap: 2px;
	}

	.action-btn {
		background: none;
		border: none;
		color: var(--colors-text-secondary);
		padding: 4px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.action-btn:hover {
		color: var(--colors-text);
	}

	.action-btn.close:hover {
		color: var(--colors-danger, #f87171);
	}

	.pane-content {
		flex: 1;
		overflow: hidden;
	}
</style>
