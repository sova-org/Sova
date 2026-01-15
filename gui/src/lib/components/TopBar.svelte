<script lang="ts">
	import {
		Play,
		Pause,
		LogOut,
		Users,
		User,
		HelpCircle,
		Save,
		FolderOpen,
		LayoutGrid,
	} from 'lucide-svelte';
	import { isConnected } from '$lib/stores/connectionState';
	import { isPlaying, isStarting, clockState } from '$lib/stores/transport';
	import { peerCount, peers } from '$lib/stores/collaboration';
	import { nickname as nicknameStore } from '$lib/stores/nickname';
	import { globalVariables } from '$lib/stores/globalVariables';
	import {
		startTransport,
		stopTransport,
		setTempo,
		setName,
	} from '$lib/api/client';
	import { invoke } from '@tauri-apps/api/core';
	import AboutModal from './AboutModal.svelte';
	import SaveModal from './SaveModal.svelte';
	import OpenModal from './OpenModal.svelte';
	import { isHelpModeActive, toggleHelpMode } from '$lib/stores/helpMode';
	import { filteredProjects, refreshProjects } from '$lib/stores/projects';
	import { commandPalette } from '$lib/stores/commandPalette';
	import { currentView, type ViewType } from '$lib/stores/viewState';

	const viewLabels: Record<ViewType, string> = {
		LOGIN: 'Login',
		SCENE: 'Scene',
		DEVICES: 'Devices',
		LOGS: 'Logs',
		CONFIG: 'Config',
		CHAT: 'Chat',
		PROJECTS: 'Projects',
	};

	function openViewNavigator() {
		window.dispatchEvent(new CustomEvent('command:open-view-navigator'));
	}

	const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
	const shortcutKey = isMac ? '⌘K' : 'Ctrl+K';

	let showAbout = $state(false);

	let isEditingTempo = $state(false);
	let tempTempoValue = $state('120');
	let tempoInputElement: HTMLInputElement | null = $state(null);

	let isEditingNickname = $state(false);
	let tempNicknameValue = $state('');
	let nicknameInputElement: HTMLInputElement | null = $state(null);

	let showSaveModal = $state(false);
	let showOpenModal = $state(false);

	let barProgress = $derived(
		$clockState !== null
			? (($clockState.beat % $clockState.quantum) / $clockState.quantum) * 100
			: 0
	);

	const GLOBAL_VAR_ORDER = ['A', 'B', 'C', 'D', 'W', 'X', 'Y', 'Z'];

	let blinking = $state(new Set<string>());
	let prevValuesJson: Record<string, string> = {};

	let displayVars = $derived(
		GLOBAL_VAR_ORDER.map((name) => ({
			name,
			value: $globalVariables[name] ?? null,
		}))
	);

	$effect(() => {
		const changed: string[] = [];
		for (const name of GLOBAL_VAR_ORDER) {
			const curr = $globalVariables[name];
			const currJson = JSON.stringify(curr ?? null);
			if (currJson !== prevValuesJson[name]) {
				changed.push(name);
				prevValuesJson[name] = currJson;
			}
		}
		if (changed.length > 0) {
			blinking = new Set([...blinking, ...changed]);
			setTimeout(() => {
				blinking = new Set([...blinking].filter((n) => !changed.includes(n)));
			}, 80);
		}
	});

	$effect(() => {
		function handleEditNickname() {
			if ($isConnected && $nicknameStore) {
				startEditingNickname();
			}
		}

		function handleOpenSaveModal() {
			if ($isConnected) {
				openSaveModal();
			}
		}

		function handleOpenProjectModal() {
			if ($isConnected) {
				openOpenModal();
			}
		}

		window.addEventListener('command:edit-nickname', handleEditNickname);
		window.addEventListener('command:open-save-modal', handleOpenSaveModal);
		window.addEventListener(
			'command:open-project-modal',
			handleOpenProjectModal
		);

		return () => {
			window.removeEventListener('command:edit-nickname', handleEditNickname);
			window.removeEventListener(
				'command:open-save-modal',
				handleOpenSaveModal
			);
			window.removeEventListener(
				'command:open-project-modal',
				handleOpenProjectModal
			);
		};
	});

	async function handleDisconnect() {
		try {
			await invoke('disconnect_client');
			isConnected.set(false);
		} catch {
			// Disconnect failed - connection likely already closed
		}
	}

	function startEditingTempo() {
		if ($clockState !== null) {
			tempTempoValue = Math.round($clockState.tempo).toString();
			isEditingTempo = true;
			requestAnimationFrame(() => tempoInputElement?.select());
		}
	}

	function cancelEditingTempo() {
		isEditingTempo = false;
	}

	async function saveTempoEdit() {
		const tempo = parseFloat(tempTempoValue);

		// Validate tempo (typically 30-300 BPM)
		if (isNaN(tempo) || tempo < 30 || tempo > 300) {
			cancelEditingTempo();
			return;
		}

		try {
			await setTempo(tempo);
			isEditingTempo = false;
		} catch (error) {
			console.error('Failed to set tempo:', error);
			cancelEditingTempo();
		}
	}

	function handleTempoKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();
			saveTempoEdit();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			cancelEditingTempo();
		}
	}

	function startEditingNickname() {
		tempNicknameValue = $nicknameStore;
		isEditingNickname = true;
		requestAnimationFrame(() => nicknameInputElement?.select());
	}

	function cancelEditingNickname() {
		isEditingNickname = false;
	}

	async function saveNicknameEdit() {
		const newNickname = tempNicknameValue.trim();
		if (!newNickname) {
			cancelEditingNickname();
			return;
		}

		try {
			nicknameStore.set(newNickname);

			// Send to server if connected
			if ($isConnected) {
				await setName(newNickname);
			}
			isEditingNickname = false;
		} catch (error) {
			console.error('Failed to set nickname:', error);
			cancelEditingNickname();
		}
	}

	function handleNicknameKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();
			saveNicknameEdit();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			cancelEditingNickname();
		}
	}

	async function openSaveModal() {
		await refreshProjects();
		showSaveModal = true;
	}

	async function openOpenModal() {
		await refreshProjects();
		showOpenModal = true;
	}

	function handlePlayClick() {
		if ($isPlaying || $isStarting) {
			stopTransport();
		} else {
			startTransport();
		}
	}
</script>

<div class="topbar">
	<div class="left-section">
		<button
			class="app-name"
			data-help-id="app-name"
			onclick={() => (showAbout = true)}>Sova</button
		>

		<button
			class="view-btn"
			data-help-id="view-navigator"
			onclick={openViewNavigator}
		>
			<LayoutGrid size={14} />
			{viewLabels[$currentView]} (Shift+Tab)
		</button>

		{#if $isConnected}
			<button
				class="open-btn"
				data-help-id="quick-open"
				onclick={openOpenModal}
				title="Open project"
			>
				<FolderOpen size={14} />
			</button>
			<button
				class="save-btn"
				data-help-id="quick-save"
				onclick={openSaveModal}
				title="Save snapshot"
			>
				<Save size={14} />
			</button>
		{/if}
	</div>

	<div class="middle-section">
		{#if $isConnected}
			<div class="actions">
				<div
					class="bar-progress"
					class:playing={$isPlaying || $isStarting}
					style="width: {barProgress}%"
				></div>

				<button
					class="transport-button play-button"
					data-help-id="play-button"
					onclick={handlePlayClick}
				>
					<span class="icon" class:hidden={$isPlaying || $isStarting}>
						<Play size={16} />
					</span>
					<span class="icon" class:hidden={!($isPlaying || $isStarting)}>
						<Pause size={16} />
					</span>
				</button>

				<span class="transport-info" data-help-id="beat-display">
					{#if $clockState !== null}
						{$clockState.beat.toFixed(1)}
					{:else}
						--
					{/if}
				</span>

				{#if isEditingTempo}
					<input
						bind:this={tempoInputElement}
						bind:value={tempTempoValue}
						onkeydown={handleTempoKeydown}
						onblur={saveTempoEdit}
						class="tempo-input"
						type="number"
						min="30"
						max="300"
						step="1"
					/>
					<span class="tempo-unit">BPM</span>
				{:else}
					<span
						class="transport-info tempo-clickable"
						data-help-id="tempo-display"
						onclick={startEditingTempo}
						onkeydown={(e) => e.key === 'Enter' && startEditingTempo()}
						role="button"
						tabindex="0"
					>
						{$clockState !== null
							? `${Math.round($clockState.tempo)} BPM`
							: '-- BPM'}
					</span>
				{/if}
			</div>
		{/if}

		<div class="global-vars" data-help-id="global-vars">
			{#each displayVars as { name, value }}
				<span
					class="var-item"
					class:has-value={value !== null}
					class:blink={blinking.has(name)}>{name}</span
				>
			{/each}
		</div>
	</div>

	<div class="right-section">
		{#if $isConnected}
			{#if $nicknameStore}
				{#if isEditingNickname}
					<input
						bind:this={nicknameInputElement}
						bind:value={tempNicknameValue}
						onkeydown={handleNicknameKeydown}
						onblur={saveNicknameEdit}
						class="nickname-input"
						type="text"
					/>
				{:else}
					<span
						class="nickname-display"
						data-help-id="nickname-display"
						onclick={startEditingNickname}
						onkeydown={(e) => e.key === 'Enter' && startEditingNickname()}
						role="button"
						tabindex="0"
					>
						<User size={12} />
						{$nicknameStore}
					</span>
				{/if}
			{/if}

			{#if $peerCount > 0}
				<div class="peer-count-wrapper">
					<span class="peer-count" data-help-id="peer-count">
						<Users size={12} />
						{$peerCount}
					</span>
					<div class="peer-tooltip">
						{#each $peers as peer (peer)}
							<div class="peer-name">{peer}</div>
						{/each}
					</div>
				</div>
			{/if}
		{/if}

		<button
			class="command-btn"
			data-help-id="command-button"
			onclick={() => commandPalette.open()}
			title="Command palette"
		>
			Cmd ({shortcutKey})
		</button>
		{#if $isConnected}
			<button
				class="disconnect-button"
				data-help-id="disconnect-button"
				onclick={handleDisconnect}
				title="Disconnect"
			>
				<LogOut size={16} />
				<span class="disconnect-text">Disconnect</span>
			</button>
		{/if}
		<button
			class="help-btn"
			class:active={$isHelpModeActive}
			data-help-id="help-button"
			onclick={toggleHelpMode}
			title="Help mode"
		>
			<HelpCircle size={16} />
		</button>
	</div>
</div>

<AboutModal bind:open={showAbout} />

<SaveModal
	bind:open={showSaveModal}
	projects={$filteredProjects}
	onClose={() => (showSaveModal = false)}
/>

<OpenModal
	bind:open={showOpenModal}
	projects={$filteredProjects}
	onClose={() => (showOpenModal = false)}
/>

<style>
	.topbar {
		width: 100%;
		height: 40px;
		box-sizing: border-box;
		background-color: var(--colors-background, #1e1e1e);
		border-bottom: 1px solid var(--colors-border, #333);
		display: grid;
		grid-template-columns: 1fr auto 1fr;
		align-items: center;
		padding: 0 12px;
		gap: 16px;
		overflow: hidden;
	}

	.left-section {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
		justify-content: flex-start;
	}

	.middle-section {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 12px;
	}

	.right-section {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
	}

	.app-name {
		font-family: monospace;
		font-size: 13px;
		font-weight: 700;
		color: var(--colors-text, #fff);
		letter-spacing: 0.5px;
		padding: 4px 8px;
		background: none;
		border: none;
		cursor: pointer;
		transition: color 0.2s;
	}

	.app-name:hover {
		color: var(--colors-accent, #0e639c);
	}

	.view-btn {
		display: flex;
		align-items: center;
		gap: 6px;
		font-family: monospace;
		font-size: 11px;
		font-weight: 500;
		color: var(--colors-text-secondary, #888);
		padding: 6px 10px;
		background: none;
		border: 1px solid var(--colors-border, #333);
		cursor: pointer;
		transition: all 0.2s;
	}

	.view-btn:hover {
		border-color: var(--colors-accent, #0e639c);
		color: var(--colors-text, #fff);
	}

	.help-btn {
		background: none;
		border: 1px solid var(--colors-border, #333);
		color: var(--colors-text-secondary, #888);
		padding: 6px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.2s;
	}

	.help-btn:hover {
		border-color: var(--colors-accent, #0e639c);
		color: var(--colors-text, #fff);
	}

	.help-btn.active {
		border-color: var(--colors-accent, #0e639c);
		color: var(--colors-accent, #0e639c);
		background: rgba(14, 99, 156, 0.1);
	}

	.command-btn {
		background: none;
		border: 1px solid var(--colors-border, #333);
		color: var(--colors-text-secondary, #888);
		padding: 6px 10px;
		cursor: pointer;
		font-family: monospace;
		font-size: 11px;
		transition: all 0.2s;
	}

	.command-btn:hover {
		border-color: var(--colors-accent, #0e639c);
		color: var(--colors-text, #fff);
	}

	.actions {
		display: flex;
		gap: 6px;
		align-items: center;
		position: relative;
		overflow: hidden;
		min-width: 0;
	}

	.bar-progress {
		position: absolute;
		top: 0;
		left: 0;
		height: 100%;
		background: var(--colors-surface, #2d2d2d);
		opacity: 0.3;
		pointer-events: none;
		z-index: 0;
	}

	.bar-progress.playing {
		background: var(--colors-accent, #0e639c);
		opacity: 0.35;
	}

	.transport-info {
		font-family: monospace;
		font-size: 11px;
		font-weight: 500;
		color: var(--colors-text, #ddd);
		padding: 4px 6px;
		white-space: nowrap;
		position: relative;
		z-index: 1;
	}

	.tempo-clickable {
		cursor: pointer;
		transition: color 0.2s;
	}

	.tempo-clickable:hover {
		color: var(--colors-text, #fff);
	}

	.tempo-input {
		font-family: monospace;
		font-size: 11px;
		font-weight: 500;
		color: var(--colors-text, #fff);
		background-color: var(--colors-surface, #2d2d2d);
		border: 1px solid var(--colors-accent, #0e639c);
		padding: 4px 6px;
		width: 50px;
		text-align: right;
		position: relative;
		z-index: 1;
	}

	.tempo-input:focus {
		outline: none;
		border-color: var(--colors-accent, #0e639c);
	}

	.tempo-unit {
		font-family: monospace;
		font-size: 11px;
		font-weight: 500;
		color: var(--colors-text-secondary, #888);
		padding: 4px 2px 4px 0;
		position: relative;
		z-index: 1;
	}

	.peer-count-wrapper {
		position: relative;
	}

	.peer-count {
		display: flex;
		align-items: center;
		gap: 4px;
		font-family: monospace;
		font-size: 11px;
		font-weight: 500;
		color: var(--colors-text-secondary, #888);
		padding: 4px 8px;
		position: relative;
		z-index: 1;
	}

	.peer-tooltip {
		display: none;
		position: fixed;
		top: 48px;
		right: 12px;
		background: var(--colors-background, #1e1e1e);
		border: 1px solid var(--colors-border, #333);
		padding: 8px;
		min-width: 120px;
		z-index: 9999;
	}

	.peer-count-wrapper:hover .peer-tooltip {
		display: block;
	}

	.peer-name {
		font-family: monospace;
		font-size: 11px;
		color: var(--colors-text, #fff);
		padding: 4px 0;
		white-space: nowrap;
	}

	.global-vars {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 4px 6px;
		background: var(--colors-surface, #2d2d2d);
		border: 1px solid var(--colors-border, #333);
	}

	.var-item {
		font-family: monospace;
		font-size: 13px;
		font-weight: 600;
		padding: 4px 8px;
		color: var(--colors-text-secondary, #555);
		background: transparent;
	}

	.var-item.has-value {
		color: var(--colors-text, #fff);
		background: var(--colors-accent, #0e639c);
	}

	.var-item.blink {
		background: transparent;
	}

	.nickname-display {
		display: flex;
		align-items: center;
		gap: 4px;
		font-family: monospace;
		font-size: 11px;
		font-weight: 500;
		color: var(--colors-text, #ddd);
		padding: 4px 8px;
		cursor: pointer;
		transition: color 0.2s;
	}

	.nickname-display:hover {
		color: var(--colors-accent, #0e639c);
	}

	.nickname-input {
		font-family: monospace;
		font-size: 11px;
		font-weight: 500;
		color: var(--colors-text, #fff);
		background-color: var(--colors-surface, #2d2d2d);
		border: 1px solid var(--colors-accent, #0e639c);
		padding: 4px 6px;
		width: 100px;
	}

	.nickname-input:focus {
		outline: none;
		border-color: var(--colors-accent, #0e639c);
	}

	.transport-button {
		background: none;
		border: 1px solid var(--colors-border, #333);
		color: var(--colors-text, #fff);
		padding: 6px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		transition: all 0.2s;
		position: relative;
		z-index: 1;
	}

	.transport-button:hover {
		border-color: var(--colors-accent, #0e639c);
		color: var(--colors-accent, #0e639c);
	}

	.play-button {
		border: none;
	}

	.play-button:hover {
		color: var(--colors-accent, #0e639c);
	}

	.play-button .icon {
		display: flex;
		pointer-events: none;
	}

	.play-button .icon.hidden {
		display: none;
	}

	.disconnect-button {
		background: none;
		border: 1px solid var(--colors-border, #333);
		color: var(--colors-text-secondary, #888);
		padding: 6px 10px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 6px;
		transition: all 0.2s;
		position: relative;
		z-index: 1;
	}

	.disconnect-button:hover {
		border-color: var(--colors-accent, #0e639c);
		color: var(--colors-text, #fff);
	}

	.disconnect-text {
		font-family: monospace;
		font-size: 11px;
		font-weight: 500;
	}

	.open-btn,
	.save-btn {
		background: none;
		border: none;
		color: var(--colors-text-secondary, #888);
		padding: 4px;
		cursor: pointer;
		display: flex;
		align-items: center;
		transition: color 0.2s;
	}

	.open-btn:hover,
	.save-btn:hover {
		color: var(--colors-accent, #0e639c);
	}
</style>
