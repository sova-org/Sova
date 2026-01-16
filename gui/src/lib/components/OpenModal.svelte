<script lang="ts">
	import { Play, Clock } from "lucide-svelte";
	import { fuzzyScore } from "$lib/utils/fuzzySearch";
	import { formatRelativeTime } from "$lib/utils/formatting";
	import {
		loadProjectImmediate,
		loadProjectAtEndOfLine,
	} from "$lib/stores/projects";
	import type { ProjectInfo } from "$lib/types/projects";

	interface Props {
		open: boolean;
		projects: ProjectInfo[];
		onClose: () => void;
	}

	let { open = $bindable(), projects, onClose }: Props = $props();

	let searchQuery = $state("");
	let rawSelectedIndex = $state(0);
	let inputElement: HTMLInputElement | null = $state(null);
	let listElement: HTMLDivElement | null = $state(null);

	let filteredProjects = $derived.by(() => {
		let result = projects;

		if (searchQuery.trim()) {
			const query = searchQuery.trim();
			result = projects
				.map((p) => ({ project: p, score: fuzzyScore(query, p.name) }))
				.filter((item) => item.score > 0)
				.sort((a, b) => b.score - a.score)
				.map((item) => item.project);
		} else {
			result = projects.slice().sort((a, b) => {
				const dateA = a.updated_at ? new Date(a.updated_at).getTime() : 0;
				const dateB = b.updated_at ? new Date(b.updated_at).getTime() : 0;
				return dateB - dateA;
			});
		}

		return result;
	});

	let selectedIndex = $derived(
		Math.min(rawSelectedIndex, Math.max(0, filteredProjects.length - 1))
	);

	$effect(() => {
		if (open) {
			searchQuery = "";
			rawSelectedIndex = 0;
			requestAnimationFrame(() => inputElement?.focus());
		}
	});

	$effect(() => {
		if (selectedIndex >= 0 && listElement) {
			const item = listElement.children[selectedIndex] as HTMLElement;
			item?.scrollIntoView({ block: "nearest" });
		}
	});

	function close() {
		open = false;
		searchQuery = "";
		rawSelectedIndex = 0;
		onClose();
	}

	async function loadNow(name: string) {
		await loadProjectImmediate(name);
		close();
	}

	async function loadAtEnd(name: string) {
		await loadProjectAtEndOfLine(name);
		close();
	}

	function handleKeydown(event: KeyboardEvent) {
		const maxIndex = filteredProjects.length - 1;

		switch (event.key) {
			case "Escape":
				close();
				break;
			case "ArrowDown":
				event.preventDefault();
				rawSelectedIndex = Math.min(rawSelectedIndex + 1, maxIndex);
				break;
			case "ArrowUp":
				event.preventDefault();
				rawSelectedIndex = Math.max(rawSelectedIndex - 1, 0);
				break;
			case "Enter":
				event.preventDefault();
				if (filteredProjects[selectedIndex]) {
					if (event.shiftKey) {
						loadAtEnd(filteredProjects[selectedIndex].name);
					} else {
						loadNow(filteredProjects[selectedIndex].name);
					}
				}
				break;
		}
	}
</script>

{#if open}
	<div
		class="modal-overlay"
		onclick={close}
		onkeydown={handleKeydown}
		role="presentation"
	>
		<div
			class="modal"
			onclick={(e) => e.stopPropagation()}
			onkeydown={(e) => e.stopPropagation()}
			role="dialog"
			aria-modal="true"
			tabindex="-1"
		>
			<div class="modal-title">Open Project</div>
			<input
				bind:this={inputElement}
				type="text"
				class="modal-input"
				placeholder="Search projects..."
				bind:value={searchQuery}
				onkeydown={handleKeydown}
			/>

			{#if filteredProjects.length > 0}
				<div class="project-list" bind:this={listElement}>
					{#each filteredProjects as project, i (project.name)}
						<div
							class="project-item"
							class:selected={i === selectedIndex}
							onmouseenter={() => (rawSelectedIndex = i)}
							role="option"
							aria-selected={i === selectedIndex}
							tabindex="-1"
						>
							<span class="project-name">{project.name}</span>
							<span class="project-date"
								>{formatRelativeTime(project.updated_at)}</span
							>
							<div class="project-actions">
								<button
									class="action-btn"
									onclick={() => loadNow(project.name)}
									title="Load now"
								>
									<Play size={12} />
								</button>
								<button
									class="action-btn"
									onclick={() => loadAtEnd(project.name)}
									title="Load at end of line"
								>
									<Clock size={12} />
								</button>
							</div>
						</div>
					{/each}
				</div>
			{:else}
				<div class="empty-state">No projects found</div>
			{/if}

			<div class="modal-footer">
				<span class="hint">Enter to load, Shift+Enter for end of line</span>
				<button class="modal-button" onclick={close}>Close</button>
			</div>
		</div>
	</div>
{/if}

<style>
	.modal-overlay {
		position: fixed;
		inset: 0;
		background: rgba(0, 0, 0, 0.5);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.modal {
		background: var(--colors-background, #1e1e1e);
		border: 1px solid var(--colors-border, #333);
		padding: 20px;
		min-width: 350px;
		max-width: 450px;
	}

	.modal-title {
		font-family: monospace;
		font-size: 14px;
		font-weight: 600;
		color: var(--colors-text, #fff);
		margin-bottom: 16px;
	}

	.modal-input {
		width: 100%;
		box-sizing: border-box;
		font-family: monospace;
		font-size: 13px;
		padding: 8px;
		background: var(--colors-surface, #2d2d2d);
		border: 1px solid var(--colors-border, #333);
		color: var(--colors-text, #fff);
		margin-bottom: 12px;
	}

	.modal-input:focus {
		outline: none;
		border-color: var(--colors-accent, #0e639c);
	}

	.project-list {
		max-height: 250px;
		overflow-y: auto;
		margin-bottom: 16px;
		border: 1px solid var(--colors-border, #333);
	}

	.project-item {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 8px 10px;
		cursor: default;
	}

	.project-item:hover,
	.project-item.selected {
		background: var(--colors-surface, #2d2d2d);
	}

	.project-name {
		flex: 1;
		font-family: monospace;
		font-size: 12px;
		color: var(--colors-text, #fff);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.project-date {
		font-family: monospace;
		font-size: 11px;
		color: var(--colors-text-secondary, #888);
		white-space: nowrap;
	}

	.project-actions {
		display: flex;
		gap: 4px;
	}

	.action-btn {
		background: none;
		border: 1px solid var(--colors-border, #333);
		color: var(--colors-text-secondary, #888);
		padding: 4px;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.action-btn:hover {
		border-color: var(--colors-accent, #0e639c);
		color: var(--colors-accent, #0e639c);
	}

	.empty-state {
		font-family: monospace;
		font-size: 12px;
		color: var(--colors-text-secondary, #888);
		text-align: center;
		padding: 24px;
		margin-bottom: 16px;
	}

	.modal-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.hint {
		font-family: monospace;
		font-size: 11px;
		color: var(--colors-text-secondary, #888);
	}

	.modal-button {
		font-family: monospace;
		font-size: 12px;
		padding: 6px 12px;
		background: none;
		border: 1px solid var(--colors-border, #333);
		color: var(--colors-text-secondary, #888);
		cursor: pointer;
	}

	.modal-button:hover {
		border-color: var(--colors-accent, #0e639c);
		color: var(--colors-text, #fff);
	}
</style>
