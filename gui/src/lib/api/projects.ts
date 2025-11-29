import { invoke } from '@tauri-apps/api/core';
import type { ProjectInfo } from '$lib/types/projects';
import type { Snapshot } from '$lib/types/protocol';

export async function listProjects(): Promise<ProjectInfo[]> {
	return invoke<ProjectInfo[]>('list_projects');
}

export async function saveProject(snapshot: Snapshot, projectName: string): Promise<void> {
	return invoke('save_project', { snapshot, projectName });
}

export async function loadProject(projectName: string): Promise<Snapshot> {
	return invoke<Snapshot>('load_project', { projectName });
}

export async function deleteProject(projectName: string): Promise<void> {
	return invoke('delete_project', { projectName });
}

export async function renameProject(oldName: string, newName: string): Promise<void> {
	return invoke('rename_project', { oldName, newName });
}

export async function openProjectsFolder(): Promise<void> {
	return invoke('open_projects_folder');
}
