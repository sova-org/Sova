<script lang="ts">
	import SidebarTabs from './SidebarTabs.svelte';
	import ProjectsView from './ProjectsView.svelte';
	import DevicesView from './DevicesView.svelte';
	import ChatView from './ChatView.svelte';
	import LogView from './LogView.svelte';
	import SettingsPanel from './SettingsPanel.svelte';
	import {
		sidebarState,
		sidebarIsOpen,
		sidebarSide,
		sidebarWidth,
		sidebarActiveTab,
		SIDEBAR_MIN_WIDTH,
		SIDEBAR_MAX_WIDTH,
	} from '$lib/stores/sidebarState';

	let isResizing = $state(false);
	let startX = 0;
	let startWidth = 0;

	function handleMouseDown(e: MouseEvent) {
		isResizing = true;
		startX = e.clientX;
		startWidth = $sidebarWidth;
		document.body.style.cursor = 'col-resize';
		document.body.style.userSelect = 'none';
	}

	function handleMouseMove(e: MouseEvent) {
		if (!isResizing) return;
		const delta = $sidebarSide === 'left'
			? e.clientX - startX
			: startX - e.clientX;
		const newWidth = Math.min(
			SIDEBAR_MAX_WIDTH,
			Math.max(SIDEBAR_MIN_WIDTH, startWidth + delta)
		);
		sidebarState.setWidth(newWidth);
	}

	function handleMouseUp() {
		if (isResizing) {
			isResizing = false;
			document.body.style.cursor = '';
			document.body.style.userSelect = '';
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.altKey && e.key.toLowerCase() === 's') {
			e.preventDefault();
			sidebarState.toggle();
		}
		if (e.key === 'Escape' && $sidebarIsOpen) {
			sidebarState.close();
		}
	}

	$effect(() => {
		if (isResizing) {
			window.addEventListener('mousemove', handleMouseMove);
			window.addEventListener('mouseup', handleMouseUp);
			return () => {
				window.removeEventListener('mousemove', handleMouseMove);
				window.removeEventListener('mouseup', handleMouseUp);
			};
		}
	});
</script>

<svelte:window onkeydown={handleKeydown} />

<div
	class="sidebar"
	class:left={$sidebarSide === 'left'}
	class:right={$sidebarSide === 'right'}
	class:open={$sidebarIsOpen}
	style="width: {$sidebarWidth}px;"
>
	<SidebarTabs />

	<div class="sidebar-content">
		{#if $sidebarActiveTab === 'PROJECTS'}
			<ProjectsView />
		{:else if $sidebarActiveTab === 'DEVICES'}
			<DevicesView />
		{:else if $sidebarActiveTab === 'CHAT'}
			<ChatView />
		{:else if $sidebarActiveTab === 'LOGS'}
			<LogView />
		{:else if $sidebarActiveTab === 'CONFIG'}
			<SettingsPanel />
		{/if}
	</div>

	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="resize-handle"
		class:left={$sidebarSide === 'right'}
		class:right={$sidebarSide === 'left'}
		onmousedown={handleMouseDown}
	></div>
</div>

<style>
	.sidebar {
		position: fixed;
		top: 40px;
		bottom: 40px;
		z-index: 100;
		background: var(--colors-background, #1e1e1e);
		border: 1px solid var(--colors-border, #333);
		display: flex;
		flex-direction: column;
		transition: transform 150ms ease-out;
		color: var(--colors-text, #fff);
		font-family: var(--appearance-font-family);
	}

	.sidebar.left {
		left: 0;
		border-left: none;
	}

	.sidebar.right {
		right: 0;
		border-right: none;
	}

	.sidebar.left:not(.open) {
		transform: translateX(-100%);
	}

	.sidebar.right:not(.open) {
		transform: translateX(100%);
	}

	.sidebar-content {
		flex: 1;
		overflow: hidden;
		background-color: var(--colors-background, #1e1e1e);
	}

	.resize-handle {
		position: absolute;
		top: 0;
		bottom: 0;
		width: 4px;
		cursor: col-resize;
		background: transparent;
		transition: background-color 0.15s;
	}

	.resize-handle.left {
		left: 0;
	}

	.resize-handle.right {
		right: 0;
	}

	.resize-handle:hover {
		background-color: var(--colors-accent, #0e639c);
	}
</style>
