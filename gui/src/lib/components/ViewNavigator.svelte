<script lang="ts">
	import {
		FolderOpen,
		Monitor,
		Users,
		LogIn,
		MessageCircle,
		FileText,
		Settings,
	} from 'lucide-svelte';
	import {
		viewState,
		currentView,
		availableViews,
		type ViewType,
	} from '$lib/stores/viewState';
	import { isConnected } from '$lib/stores/connectionState';
	import { commandPalette } from '$lib/stores/commandPalette';

	type GridCell = ViewType | 'LOGIN_OR_CHAT';

	const viewGrid: GridCell[][] = [
		['PROJECTS', 'SCENE', 'DEVICES'],
		['LOGIN_OR_CHAT', 'LOGS', 'CONFIG'],
	];

	const viewIcons: Record<ViewType, typeof Monitor> = {
		LOGIN: LogIn,
		SCENE: Monitor,
		DEVICES: Users,
		LOGS: FileText,
		CONFIG: Settings,
		CHAT: MessageCircle,
		PROJECTS: FolderOpen,
	};

	const viewLabels: Record<ViewType, string> = {
		LOGIN: 'Login',
		SCENE: 'Scene',
		DEVICES: 'Devices',
		LOGS: 'Logs',
		CONFIG: 'Config',
		CHAT: 'Chat',
		PROJECTS: 'Projects',
	};

	let isOpen = $state(false);
	let selectedRow = $state(0);
	let selectedCol = $state(1);

	$effect(() => {
		function handleOpenCommand() {
			openNavigator();
		}
		window.addEventListener('command:open-view-navigator', handleOpenCommand);
		return () => {
			window.removeEventListener(
				'command:open-view-navigator',
				handleOpenCommand
			);
		};
	});

	function resolveCell(cell: GridCell): ViewType | null {
		if (cell === 'LOGIN_OR_CHAT') {
			return $isConnected ? 'CHAT' : 'LOGIN';
		}
		return cell;
	}

	function isAvailable(view: ViewType): boolean {
		return $availableViews.includes(view);
	}

	function findViewPosition(
		view: ViewType
	): { row: number; col: number } | null {
		for (let row = 0; row < viewGrid.length; row++) {
			for (let col = 0; col < viewGrid[row].length; col++) {
				const resolved = resolveCell(viewGrid[row][col]);
				if (resolved === view) {
					return { row, col };
				}
			}
		}
		return null;
	}

	function openNavigator() {
		const pos = findViewPosition($currentView);
		if (pos) {
			selectedRow = pos.row;
			selectedCol = pos.col;
		} else {
			selectedRow = 0;
			selectedCol = 1;
		}
		isOpen = true;
	}

	function closeNavigator() {
		isOpen = false;
	}

	function confirmSelection() {
		const cell = viewGrid[selectedRow]?.[selectedCol];
		if (!cell) return;

		const view = resolveCell(cell);
		if (view && isAvailable(view)) {
			viewState.navigateTo(view);
		}
		closeNavigator();
	}

	function moveSelection(dRow: number, dCol: number) {
		let newRow = selectedRow + dRow;
		let newCol = selectedCol + dCol;

		newRow = Math.max(0, Math.min(viewGrid.length - 1, newRow));
		newCol = Math.max(0, Math.min(viewGrid[0].length - 1, newCol));

		const cell = viewGrid[newRow]?.[newCol];
		if (cell) {
			const view = resolveCell(cell);
			if (view && isAvailable(view)) {
				selectedRow = newRow;
				selectedCol = newCol;
			}
		}
	}

	function handleKeydown(e: KeyboardEvent) {
		if ($commandPalette.isOpen) return;

		// Shift+Tab toggles navigator (Escape only closes when open)
		if (e.key === 'Tab' && e.shiftKey) {
			e.preventDefault();
			if (isOpen) {
				closeNavigator();
			} else {
				openNavigator();
			}
			return;
		}

		if (e.key === 'Escape' && isOpen) {
			e.preventDefault();
			closeNavigator();
			return;
		}

		if (!isOpen) return;

		switch (e.key) {
			case 'ArrowUp':
				e.preventDefault();
				moveSelection(-1, 0);
				break;
			case 'ArrowDown':
				e.preventDefault();
				moveSelection(1, 0);
				break;
			case 'ArrowLeft':
				e.preventDefault();
				moveSelection(0, -1);
				break;
			case 'ArrowRight':
				e.preventDefault();
				moveSelection(0, 1);
				break;
			case 'Enter':
				e.preventDefault();
				confirmSelection();
				break;
		}
	}

	function handleCellClick(row: number, col: number) {
		const cell = viewGrid[row]?.[col];
		if (!cell) return;

		const view = resolveCell(cell);
		if (view && isAvailable(view)) {
			viewState.navigateTo(view);
			closeNavigator();
		}
	}

	function handleOverlayClick() {
		closeNavigator();
	}
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
	<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
	<div class="overlay" onclick={handleOverlayClick}>
		<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
		<div class="navigator" onclick={(e) => e.stopPropagation()}>
			<div class="grid">
				{#each viewGrid as row, rowIdx (rowIdx)}
					<div class="grid-row">
						{#each row as cell, colIdx (colIdx)}
							{@const view = resolveCell(cell)}
							{@const available = view ? isAvailable(view) : false}
							{@const isCurrent = view === $currentView}
							{@const isSelected =
								rowIdx === selectedRow && colIdx === selectedCol}
							<div
								class="cell"
								class:available
								class:unavailable={!available}
								class:current={isCurrent}
								class:selected={isSelected}
								onclick={() => handleCellClick(rowIdx, colIdx)}
							>
								{#if view}
									{@const Icon = viewIcons[view]}
									<Icon size={20} />
									<span class="label">{viewLabels[view]}</span>
								{/if}
							</div>
						{/each}
					</div>
				{/each}
			</div>
			<div class="hint">
				<span>Arrow keys to navigate</span>
				<span>Enter to select · Shift+Tab or ESC to close</span>
			</div>
		</div>
	</div>
{/if}

<style>
	.overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.6);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 9998;
	}

	.navigator {
		background: var(--colors-background);
		border: 1px solid var(--colors-border);
		padding: 16px;
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
		max-width: 320px;
	}

	.grid {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.grid-row {
		display: flex;
		gap: 4px;
	}

	.cell {
		width: 100px;
		height: 80px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		background: var(--colors-surface);
		border: 2px solid transparent;
		cursor: pointer;
		transition: all 0.1s ease;
	}

	.cell.available:hover {
		border-color: var(--colors-text-secondary);
	}

	.cell.unavailable {
		opacity: 0.3;
		cursor: not-allowed;
	}

	.cell.current {
		background: var(--colors-accent);
		color: var(--colors-background);
	}

	.cell.current .label {
		color: var(--colors-background);
	}

	.cell.selected {
		border-color: var(--colors-accent);
	}

	.cell.selected.current {
		border-color: var(--colors-text);
	}

	.label {
		font-family: monospace;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		color: var(--colors-text-secondary);
	}

	.hint {
		display: flex;
		flex-wrap: wrap;
		justify-content: center;
		gap: 8px 16px;
		margin-top: 12px;
		padding-top: 12px;
		border-top: 1px solid var(--colors-border);
	}

	.hint span {
		font-family: monospace;
		font-size: 10px;
		color: var(--colors-text-muted, #666);
	}
</style>
