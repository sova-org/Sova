import { writable, derived, get } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SERVER_EVENTS } from '$lib/events';
import type { ProjectInfo } from '$lib/types/projects';
import type { Snapshot, ActionTiming } from '$lib/types/protocol';
import * as projectsApi from '$lib/api/projects';
import { getSnapshot, setScene, setTempo, ActionTiming as AT } from '$lib/api/client';
import { isConnected } from './connectionState';
import { clearAllLocalEdits } from './localEdits';

export type SortField = 'name' | 'tempo' | 'line_count' | 'updated_at';
export type SortDirection = 'asc' | 'desc';

interface ProjectsState {
	projects: ProjectInfo[];
	searchQuery: string;
	sortField: SortField;
	sortDirection: SortDirection;
	pendingSave: string | null;
	statusMessage: string;
	editingName: string | null;
}

const initialState: ProjectsState = {
	projects: [],
	searchQuery: '',
	sortField: 'updated_at',
	sortDirection: 'desc',
	pendingSave: null,
	statusMessage: '',
	editingName: null
};

const state = writable<ProjectsState>(initialState);

function sanitizeProjectName(name: string): string {
	return name.replace(/[<>:"/\\|?*]/g, '_').trim();
}

function parseDate(dateStr: string | null): Date | null {
	if (!dateStr) return null;
	const d = new Date(dateStr);
	return isNaN(d.getTime()) ? null : d;
}

function compareProjects(a: ProjectInfo, b: ProjectInfo, field: SortField, direction: SortDirection): number {
	let result = 0;

	switch (field) {
		case 'name':
			result = a.name.localeCompare(b.name);
			break;
		case 'tempo':
			result = (a.tempo ?? 0) - (b.tempo ?? 0);
			break;
		case 'line_count':
			result = (a.line_count ?? 0) - (b.line_count ?? 0);
			break;
		case 'updated_at': {
			const dateA = parseDate(a.updated_at);
			const dateB = parseDate(b.updated_at);
			if (!dateA && !dateB) result = 0;
			else if (!dateA) result = 1;
			else if (!dateB) result = -1;
			else result = dateA.getTime() - dateB.getTime();
			break;
		}
	}

	return direction === 'desc' ? -result : result;
}

export const filteredProjects = derived(state, ($state) => {
	let filtered = $state.projects;

	if ($state.searchQuery) {
		const query = $state.searchQuery.toLowerCase();
		filtered = filtered.filter((p) => p.name.toLowerCase().includes(query));
	}

	return filtered.sort((a, b) => compareProjects(a, b, $state.sortField, $state.sortDirection));
});

export const statusMessage = derived(state, ($state) => $state.statusMessage);
export const searchQuery = derived(state, ($state) => $state.searchQuery);
export const sortField = derived(state, ($state) => $state.sortField);
export const sortDirection = derived(state, ($state) => $state.sortDirection);
export const pendingSave = derived(state, ($state) => $state.pendingSave);
export const editingName = derived(state, ($state) => $state.editingName);

export function setSearchQuery(query: string): void {
	state.update((s) => ({ ...s, searchQuery: query }));
}

export function setSort(field: SortField): void {
	state.update((s) => {
		if (s.sortField === field) {
			return { ...s, sortDirection: s.sortDirection === 'asc' ? 'desc' : 'asc' };
		}
		return { ...s, sortField: field, sortDirection: 'desc' };
	});
}

export function setStatusMessage(message: string): void {
	state.update((s) => ({ ...s, statusMessage: message }));
}

export function clearStatusMessage(): void {
	state.update((s) => ({ ...s, statusMessage: '' }));
}

export function startEditingName(name: string): void {
	state.update((s) => ({ ...s, editingName: name }));
}

export function stopEditingName(): void {
	state.update((s) => ({ ...s, editingName: null }));
}

export async function refreshProjects(): Promise<void> {
	try {
		const projects = await projectsApi.listProjects();
		state.update((s) => ({ ...s, projects }));
	} catch (e) {
		setStatusMessage(`Failed to load projects: ${e}`);
	}
}

export async function initiateSave(name: string): Promise<void> {
	const sanitized = sanitizeProjectName(name);
	if (!sanitized) {
		setStatusMessage('Invalid project name');
		return;
	}

	if (!get(isConnected)) {
		setStatusMessage('Not connected to server');
		return;
	}

	state.update((s) => ({ ...s, pendingSave: sanitized, statusMessage: 'Requesting snapshot...' }));
	await getSnapshot();
}

export async function completeSave(snapshot: Snapshot): Promise<void> {
	const $state = get(state);
	if (!$state.pendingSave) return;

	const projectName = $state.pendingSave;
	state.update((s) => ({ ...s, pendingSave: null }));

	try {
		await projectsApi.saveProject(snapshot, projectName);
		setStatusMessage(`Saved "${projectName}"`);
		await refreshProjects();
	} catch (e) {
		setStatusMessage(`Failed to save: ${e}`);
	}
}

export async function loadProjectImmediate(name: string): Promise<void> {
	await loadProjectWithTiming(name, AT.immediate());
}

export async function loadProjectAtEndOfLine(name: string, lineId: number = 0): Promise<void> {
	await loadProjectWithTiming(name, AT.endOfLine(lineId));
}

async function loadProjectWithTiming(name: string, timing: ActionTiming): Promise<void> {
	try {
		setStatusMessage(`Loading "${name}"...`);
		const snapshot = await projectsApi.loadProject(name);

		await setTempo(snapshot.tempo, timing);
		await setScene(snapshot.scene, timing);

		// Clear stale state from previous session
		clearAllLocalEdits();
		window.dispatchEvent(new CustomEvent('project:loaded'));

		setStatusMessage(`Loaded "${name}"`);
	} catch (e) {
		setStatusMessage(`Failed to load: ${e}`);
	}
}

export async function deleteProjectByName(name: string): Promise<void> {
	try {
		await projectsApi.deleteProject(name);
		setStatusMessage(`Deleted "${name}"`);
		await refreshProjects();
	} catch (e) {
		setStatusMessage(`Failed to delete: ${e}`);
	}
}

export async function renameProjectByName(oldName: string, newName: string): Promise<void> {
	const sanitized = sanitizeProjectName(newName);
	if (!sanitized) {
		setStatusMessage('Invalid project name');
		return;
	}

	try {
		await projectsApi.renameProject(oldName, sanitized);
		setStatusMessage(`Renamed to "${sanitized}"`);
		stopEditingName();
		await refreshProjects();
	} catch (e) {
		setStatusMessage(`Failed to rename: ${e}`);
	}
}

export async function openFolder(): Promise<void> {
	try {
		await projectsApi.openProjectsFolder();
	} catch (e) {
		setStatusMessage(`Failed to open folder: ${e}`);
	}
}

export function projectExists(name: string): boolean {
	const sanitized = sanitizeProjectName(name);
	const $state = get(state);
	return $state.projects.some((p) => p.name === sanitized);
}

let snapshotUnlisten: UnlistenFn | null = null;

export async function initializeProjectsStore(): Promise<void> {
	snapshotUnlisten = await listen<Snapshot>(SERVER_EVENTS.SNAPSHOT, (event) => {
		completeSave(event.payload);
	});
}

export function cleanupProjectsStore(): void {
	if (snapshotUnlisten) {
		snapshotUnlisten();
		snapshotUnlisten = null;
	}
}
