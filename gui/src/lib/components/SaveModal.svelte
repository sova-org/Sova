<script lang="ts">
	import { fuzzyScore } from "$lib/utils/fuzzySearch";
	import { formatRelativeTime } from "$lib/utils/formatting";
	import { initiateSave, projectExists } from "$lib/stores/projects";
	import type { ProjectInfo } from "$lib/types/projects";

	interface Props {
		open: boolean;
		projects: ProjectInfo[];
		onClose: () => void;
	}

	let { open = $bindable(), projects, onClose }: Props = $props();

	let nameInput = $state("");
	let selectedIndex = $state(-1);
	let showOverwriteConfirm = $state(false);
	let inputElement: HTMLInputElement | null = $state(null);
	let listElement: HTMLDivElement | null = $state(null);

	let filteredProjects = $derived.by(() => {
		if (!nameInput.trim()) {
			return projects.slice().sort((a, b) => {
				const dateA = a.updated_at ? new Date(a.updated_at).getTime() : 0;
				const dateB = b.updated_at ? new Date(b.updated_at).getTime() : 0;
				return dateB - dateA;
			});
		}

		const query = nameInput.trim();
		return projects
			.map((p) => ({ project: p, score: fuzzyScore(query, p.name) }))
			.filter((item) => item.score > 0)
			.sort((a, b) => b.score - a.score)
			.map((item) => item.project);
	});

	$effect(() => {
		if (open) {
			nameInput = "";
			selectedIndex = -1;
			showOverwriteConfirm = false;
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
		nameInput = "";
		selectedIndex = -1;
		showOverwriteConfirm = false;
		onClose();
	}

	function selectProject(name: string) {
		nameInput = name;
		selectedIndex = -1;
	}

	function handleSubmit() {
		if (!nameInput.trim()) return;
		if (projectExists(nameInput.trim())) {
			showOverwriteConfirm = true;
		} else {
			doSave();
		}
	}

	async function doSave() {
		await initiateSave(nameInput.trim());
		close();
	}

	function handleKeydown(event: KeyboardEvent) {
		if (showOverwriteConfirm) {
			if (event.key === "Escape") {
				showOverwriteConfirm = false;
			} else if (event.key === "Enter") {
				doSave();
			}
			return;
		}

		const maxIndex = filteredProjects.length - 1;

		switch (event.key) {
			case "Escape":
				close();
				break;
			case "ArrowDown":
				event.preventDefault();
				selectedIndex = Math.min(selectedIndex + 1, maxIndex);
				break;
			case "ArrowUp":
				event.preventDefault();
				selectedIndex = Math.max(selectedIndex - 1, -1);
				break;
			case "Enter":
				event.preventDefault();
				if (selectedIndex >= 0 && filteredProjects[selectedIndex]) {
					selectProject(filteredProjects[selectedIndex].name);
				} else {
					handleSubmit();
				}
				break;
			case "Tab":
				if (selectedIndex >= 0 && filteredProjects[selectedIndex]) {
					event.preventDefault();
					selectProject(filteredProjects[selectedIndex].name);
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
			{#if showOverwriteConfirm}
				<div class="modal-title">Overwrite Project?</div>
				<div class="modal-message">
					A project named "{nameInput}" already exists.
				</div>
				<div class="modal-buttons">
					<button
						class="modal-button"
						onclick={() => (showOverwriteConfirm = false)}>Cancel</button
					>
					<button class="modal-button confirm" onclick={doSave}
						>Overwrite</button
					>
				</div>
			{:else}
				<div class="modal-title">Save Snapshot</div>
				<input
					bind:this={inputElement}
					type="text"
					class="modal-input"
					placeholder="Project name..."
					bind:value={nameInput}
					onkeydown={handleKeydown}
				/>

				{#if filteredProjects.length > 0}
					<div class="project-list" bind:this={listElement}>
						{#each filteredProjects as project, i (project.name)}
							<button
								class="project-item"
								class:selected={i === selectedIndex}
								onclick={() => selectProject(project.name)}
								onmouseenter={() => (selectedIndex = i)}
							>
								<span class="project-name">{project.name}</span>
								<span class="project-date"
									>{formatRelativeTime(project.updated_at)}</span
								>
							</button>
						{/each}
					</div>
				{/if}

				<div class="modal-buttons">
					<button class="modal-button" onclick={close}>Cancel</button>
					<button class="modal-button confirm" onclick={handleSubmit}
						>Save</button
					>
				</div>
			{/if}
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

	.modal-message {
		font-family: monospace;
		font-size: 12px;
		color: var(--colors-text-secondary, #888);
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
		max-height: 200px;
		overflow-y: auto;
		margin-bottom: 16px;
		border: 1px solid var(--colors-border, #333);
	}

	.project-item {
		display: flex;
		justify-content: space-between;
		align-items: center;
		width: 100%;
		padding: 8px 10px;
		background: none;
		border: none;
		cursor: pointer;
		font-family: monospace;
		text-align: left;
	}

	.project-item:hover,
	.project-item.selected {
		background: var(--colors-surface, #2d2d2d);
	}

	.project-name {
		font-size: 12px;
		color: var(--colors-text, #fff);
	}

	.project-date {
		font-size: 11px;
		color: var(--colors-text-secondary, #888);
	}

	.modal-buttons {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
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

	.modal-button.confirm {
		background: var(--colors-accent, #0e639c);
		border-color: var(--colors-accent, #0e639c);
		color: var(--colors-background, #1e1e1e);
	}

	.modal-button.confirm:hover {
		opacity: 0.9;
	}
</style>
