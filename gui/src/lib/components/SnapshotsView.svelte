<script lang="ts">
  import { onMount } from 'svelte';
  import { Search, FolderOpen, RefreshCw, Play, Clock, Trash2, Save, Import } from 'lucide-svelte';
  import { isConnected } from '$lib/stores/connectionState';
  import {
    filteredProjects,
    statusMessage,
    searchQuery,
    sortField,
    sortDirection,
    editingName,
    pendingSave,
    setSearchQuery,
    setSort,
    refreshProjects,
    initiateSave,
    loadProjectImmediate,
    loadProjectAtEndOfLine,
    deleteProjectByName,
    renameProjectByName,
    startEditingName,
    stopEditingName,
    openFolder,
    projectExists,
    importProject
  } from '$lib/stores/projects';
  import { ActionTiming } from '$lib/api/client';
  import type { SortField } from '$lib/stores/projects';
  import type { ProjectInfo } from '$lib/types/projects';

  let saveNameInput = $state('');
  let showOverwriteConfirm = $state(false);
  let showDeleteConfirm = $state<string | null>(null);
  let editNameValue = $state('');

  onMount(() => {
    refreshProjects();
  });

  function formatRelativeTime(dateStr: string | null): string {
    if (!dateStr) return '--';
    const date = new Date(dateStr);
    if (isNaN(date.getTime())) return '--';

    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffSec = Math.floor(diffMs / 1000);
    const diffMin = Math.floor(diffSec / 60);
    const diffHour = Math.floor(diffMin / 60);
    const diffDay = Math.floor(diffHour / 24);

    if (diffSec < 60) return 'just now';
    if (diffMin < 60) return `${diffMin} min ago`;
    if (diffHour < 24) return `${diffHour} hour${diffHour > 1 ? 's' : ''} ago`;
    if (diffDay < 7) return `${diffDay} day${diffDay > 1 ? 's' : ''} ago`;
    return date.toLocaleDateString();
  }

  function handleSave() {
    if (!saveNameInput.trim()) return;

    if (projectExists(saveNameInput)) {
      showOverwriteConfirm = true;
    } else {
      doSave();
    }
  }

  function doSave() {
    initiateSave(saveNameInput);
    saveNameInput = '';
    showOverwriteConfirm = false;
  }

  function cancelOverwrite() {
    showOverwriteConfirm = false;
  }

  function handleDelete(name: string) {
    showDeleteConfirm = name;
  }

  function confirmDelete() {
    if (showDeleteConfirm) {
      deleteProjectByName(showDeleteConfirm);
      showDeleteConfirm = null;
    }
  }

  function cancelDelete() {
    showDeleteConfirm = null;
  }

  function handleStartRename(project: ProjectInfo) {
    startEditingName(project.name);
    editNameValue = project.name;
  }

  function handleRename(oldName: string) {
    if (editNameValue.trim() && editNameValue !== oldName) {
      renameProjectByName(oldName, editNameValue);
    } else {
      stopEditingName();
    }
  }

  function handleRenameKeydown(e: KeyboardEvent, oldName: string) {
    if (e.key === 'Enter') {
      handleRename(oldName);
    } else if (e.key === 'Escape') {
      stopEditingName();
    }
  }

  function handleSortClick(field: SortField) {
    setSort(field);
  }

  function getSortIndicator(field: SortField): string {
    if ($sortField !== field) return '';
    return $sortDirection === 'asc' ? ' ▲' : ' ▼';
  }
</script>

<div class="snapshots-view">
  <div class="toolbar">
    <div class="toolbar-row">
      <div class="search-container" data-help-id="snapshots-search">
        <Search size={14} />
        <input
          type="text"
          class="search-input"
          placeholder="Search projects..."
          value={$searchQuery}
          oninput={(e) => setSearchQuery(e.currentTarget.value)}
        />
      </div>
      <div class="toolbar-buttons">
        <button class="icon-button" onclick={() => importProject(ActionTiming.immediate())} title="Import" disabled={!$isConnected} data-help-id="snapshots-import">
          <Import size={14} />
        </button>
        <button class="icon-button" onclick={() => refreshProjects()} title="Refresh" data-help-id="snapshots-refresh">
          <RefreshCw size={14} />
        </button>
        <button class="icon-button" onclick={() => openFolder()} title="Open Folder" data-help-id="snapshots-folder">
          <FolderOpen size={14} />
        </button>
      </div>
    </div>
    <div class="toolbar-row">
      <input
        type="text"
        class="save-input"
        placeholder="Project name..."
        bind:value={saveNameInput}
        onkeydown={(e) => e.key === 'Enter' && handleSave()}
        disabled={!$isConnected || $pendingSave !== null}
        data-help-id="snapshots-name"
      />
      <button
        class="save-button"
        onclick={handleSave}
        disabled={!$isConnected || !saveNameInput.trim() || $pendingSave !== null}
        data-help-id="snapshots-save"
      >
        <Save size={14} />
        Save
      </button>
    </div>
  </div>

  <div class="table-container">
    <div class="table-header">
      <button class="col-name sortable" onclick={() => handleSortClick('name')}>
        Name{getSortIndicator('name')}
      </button>
      <button class="col-tempo sortable" onclick={() => handleSortClick('tempo')}>
        Tempo{getSortIndicator('tempo')}
      </button>
      <button class="col-lines sortable" onclick={() => handleSortClick('line_count')}>
        Lines{getSortIndicator('line_count')}
      </button>
      <button class="col-modified sortable" onclick={() => handleSortClick('updated_at')}>
        Modified{getSortIndicator('updated_at')}
      </button>
      <div class="col-actions">Actions</div>
    </div>

    <div class="table-body">
      {#each $filteredProjects as project}
        <div class="table-row">
          <div class="col-name">
            {#if $editingName === project.name}
              <input
                type="text"
                class="name-input"
                bind:value={editNameValue}
                onkeydown={(e) => handleRenameKeydown(e, project.name)}
                onblur={() => handleRename(project.name)}
                autofocus
              />
            {:else}
              <button class="name-button" onclick={() => handleStartRename(project)}>
                {project.name}
              </button>
            {/if}
          </div>
          <div class="col-tempo">{project.tempo ?? '--'}</div>
          <div class="col-lines">{project.line_count ?? '--'}</div>
          <div class="col-modified">{formatRelativeTime(project.updated_at)}</div>
          <div class="col-actions">
            <button
              class="action-button load-now"
              onclick={() => loadProjectImmediate(project.name)}
              title="Load Now"
              disabled={!$isConnected}
              data-help-id="snapshots-load-now"
            >
              <Play size={12} />
            </button>
            <button
              class="action-button load-end"
              onclick={() => loadProjectAtEndOfLine(project.name)}
              title="Load at End of Line"
              disabled={!$isConnected}
              data-help-id="snapshots-load-end"
            >
              <Clock size={12} />
            </button>
            <button
              class="action-button delete"
              onclick={() => handleDelete(project.name)}
              title="Delete"
              data-help-id="snapshots-delete"
            >
              <Trash2 size={12} />
            </button>
          </div>
        </div>
      {:else}
        <div class="empty-state">
          {#if $searchQuery}
            No projects match "{$searchQuery}"
          {:else}
            No projects saved yet
          {/if}
        </div>
      {/each}
    </div>
  </div>

  {#if $statusMessage}
    <div class="status-bar">
      {$statusMessage}
    </div>
  {/if}

  {#if showOverwriteConfirm}
    <div class="modal-overlay">
      <div class="modal">
        <div class="modal-title">Overwrite Project?</div>
        <div class="modal-message">
          A project named "{saveNameInput}" already exists. Overwrite it?
        </div>
        <div class="modal-buttons">
          <button class="modal-button cancel" onclick={cancelOverwrite}>Cancel</button>
          <button class="modal-button confirm" onclick={doSave}>Overwrite</button>
        </div>
      </div>
    </div>
  {/if}

  {#if showDeleteConfirm}
    <div class="modal-overlay">
      <div class="modal delete-modal">
        <div class="modal-title">Delete Project?</div>
        <div class="modal-message">
          Are you sure you want to delete "{showDeleteConfirm}"? This cannot be undone.
        </div>
        <div class="modal-buttons">
          <button class="modal-button cancel" onclick={cancelDelete}>Cancel</button>
          <button class="modal-button confirm delete" onclick={confirmDelete}>Delete</button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .snapshots-view {
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background-color: var(--colors-background);
    font-family: monospace;
    font-size: 13px;
  }

  .toolbar {
    padding: 12px;
    border-bottom: 1px solid var(--colors-border, #333);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .toolbar-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .search-container {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--colors-surface, #2d2d2d);
    border: 1px solid var(--colors-border, #333);
    padding: 6px 10px;
    color: var(--colors-text-secondary, #888);
  }

  .search-input {
    flex: 1;
    background: none;
    border: none;
    color: var(--colors-text, #fff);
    font-family: monospace;
    font-size: 13px;
    outline: none;
  }

  .search-input::placeholder {
    color: var(--colors-text-secondary, #666);
  }

  .toolbar-buttons {
    display: flex;
    gap: 4px;
  }

  .icon-button {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: 1px solid var(--colors-border, #333);
    color: var(--colors-text-secondary, #888);
    padding: 6px;
    cursor: pointer;
  }

  .icon-button:hover {
    border-color: var(--colors-accent, #0e639c);
    color: var(--colors-text, #fff);
  }

  .save-input {
    flex: 1;
    background: var(--colors-surface, #2d2d2d);
    border: 1px solid var(--colors-border, #333);
    color: var(--colors-text, #fff);
    font-family: monospace;
    font-size: 13px;
    padding: 6px 10px;
    outline: none;
  }

  .save-input:focus {
    border-color: var(--colors-accent, #0e639c);
  }

  .save-input:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .save-button {
    display: flex;
    align-items: center;
    gap: 6px;
    background: var(--colors-accent, #0e639c);
    border: none;
    color: var(--colors-text, #fff);
    font-family: monospace;
    font-size: 13px;
    padding: 6px 12px;
    cursor: pointer;
  }

  .save-button:hover:not(:disabled) {
    background: var(--colors-accent-hover, #1177bb);
  }

  .save-button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .table-container {
    flex: 1;
    overflow: auto;
  }

  .table-header {
    display: grid;
    grid-template-columns: 1fr 80px 60px 120px 120px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--colors-border, #333);
    background: var(--colors-surface, #2d2d2d);
    position: sticky;
    top: 0;
  }

  .sortable {
    background: none;
    border: none;
    color: var(--colors-text-secondary, #888);
    font-family: monospace;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    text-align: left;
    padding: 0;
  }

  .sortable:hover {
    color: var(--colors-text, #fff);
  }

  .table-body {
    display: flex;
    flex-direction: column;
  }

  .table-row {
    display: grid;
    grid-template-columns: 1fr 80px 60px 120px 120px;
    padding: 8px 12px;
    border-bottom: 1px solid var(--colors-border, #333);
    align-items: center;
  }

  .table-row:hover {
    background: var(--colors-surface, #2d2d2d);
  }

  .col-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .col-tempo,
  .col-lines {
    color: var(--colors-text-secondary, #888);
    text-align: right;
    padding-right: 16px;
  }

  .col-modified {
    color: var(--colors-text-secondary, #888);
  }

  .col-actions {
    display: flex;
    gap: 4px;
    justify-content: flex-end;
    color: var(--colors-text-secondary, #888);
  }

  .name-button {
    background: none;
    border: none;
    color: var(--colors-text, #fff);
    font-family: monospace;
    font-size: 13px;
    cursor: pointer;
    padding: 2px 0;
    text-align: left;
  }

  .name-button:hover {
    color: var(--colors-accent, #0e639c);
  }

  .name-input {
    background: var(--colors-background);
    border: 1px solid var(--colors-accent, #0e639c);
    color: var(--colors-text, #fff);
    font-family: monospace;
    font-size: 13px;
    padding: 2px 4px;
    width: 100%;
    outline: none;
  }

  .action-button {
    display: flex;
    align-items: center;
    justify-content: center;
    background: none;
    border: 1px solid var(--colors-border, #333);
    color: var(--colors-text-secondary, #888);
    padding: 4px 6px;
    cursor: pointer;
  }

  .action-button:hover:not(:disabled) {
    border-color: currentColor;
  }

  .action-button:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }


  .empty-state {
    padding: 32px;
    text-align: center;
    color: var(--colors-text-secondary, #888);
  }

  .status-bar {
    padding: 8px 12px;
    border-top: 1px solid var(--colors-border, #333);
    color: var(--colors-text-secondary, #888);
    font-size: 12px;
  }

  .modal-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal {
    background: var(--colors-background);
    border: 1px solid var(--colors-border, #333);
    padding: 20px;
    min-width: 300px;
    max-width: 400px;
  }

  .modal-title {
    font-weight: 600;
    margin-bottom: 12px;
    color: var(--colors-text, #fff);
  }

  .modal-message {
    color: var(--colors-text-secondary, #888);
    margin-bottom: 16px;
    line-height: 1.5;
  }

  .modal-buttons {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
  }

  .modal-button {
    background: none;
    border: 1px solid var(--colors-border, #333);
    color: var(--colors-text, #fff);
    font-family: monospace;
    font-size: 13px;
    padding: 6px 12px;
    cursor: pointer;
  }

  .modal-button:hover {
    border-color: var(--colors-accent, #0e639c);
  }

  .modal-button.confirm {
    background: var(--colors-accent, #0e639c);
    border-color: var(--colors-accent, #0e639c);
  }

  .modal-button.confirm:hover {
    background: var(--colors-accent-hover, #1177bb);
  }

  .modal-button.confirm.delete {
    background: var(--colors-danger, #f87171);
    border-color: var(--colors-danger, #f87171);
  }

  .modal-button.confirm.delete:hover {
    background: #ef4444;
  }

  .delete-modal .modal-title {
    color: var(--colors-danger, #f87171);
  }
</style>
