import { writable, derived, get } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { SERVER_EVENTS } from "$lib/events";
import type { ProjectInfo } from "$lib/types/projects";
import type { Snapshot, ActionTiming } from "$lib/types/protocol";
import * as projectsApi from "$lib/api/projects";
import {
  getSnapshot,
  setScene,
  setTempo,
  restoreDevices,
  ActionTiming as AT,
} from "$lib/api/client";
import { isConnected } from "./connectionState";
import { clearAllLocalEdits } from "./localEdits";
import { ListenerGroup } from "./helpers";
import {
  projectsUIState,
  setStatusMessage,
  clearStatusMessage,
  stopEditingName,
  type SortField,
  type SortDirection,
} from "./projectsUI";

// Re-export UI state for convenience
export {
  statusMessage,
  searchQuery,
  sortField,
  sortDirection,
  editingName,
  setSearchQuery,
  setSort,
  setStatusMessage,
  clearStatusMessage,
  startEditingName,
  stopEditingName,
  type SortField,
  type SortDirection,
} from "./projectsUI";

interface ProjectsDataState {
  projects: ProjectInfo[];
  pendingSaveName: string | null;
}

const initialState: ProjectsDataState = {
  projects: [],
  pendingSaveName: null,
};

const state = writable<ProjectsDataState>(initialState);

function sanitizeProjectName(name: string): string {
  return name.replace(/[<>:"/\\|?*]/g, "_").trim();
}

function parseDate(dateStr: string | null): Date | null {
  if (!dateStr) return null;
  const d = new Date(dateStr);
  return isNaN(d.getTime()) ? null : d;
}

function compareProjects(
  a: ProjectInfo,
  b: ProjectInfo,
  field: SortField,
  direction: SortDirection,
): number {
  let result = 0;

  switch (field) {
    case "name":
      result = a.name.localeCompare(b.name);
      break;
    case "tempo":
      result = (a.tempo ?? 0) - (b.tempo ?? 0);
      break;
    case "line_count":
      result = (a.line_count ?? 0) - (b.line_count ?? 0);
      break;
    case "updated_at": {
      const dateA = parseDate(a.updated_at);
      const dateB = parseDate(b.updated_at);
      if (!dateA && !dateB) result = 0;
      else if (!dateA) result = 1;
      else if (!dateB) result = -1;
      else result = dateA.getTime() - dateB.getTime();
      break;
    }
  }

  return direction === "desc" ? -result : result;
}

export const filteredProjects = derived(
  [state, projectsUIState],
  ([$state, $ui]) => {
    let filtered = $state.projects;

    if ($ui.searchQuery) {
      const query = $ui.searchQuery.toLowerCase();
      filtered = filtered.filter((p) => p.name.toLowerCase().includes(query));
    }

    return filtered.sort((a, b) =>
      compareProjects(a, b, $ui.sortField, $ui.sortDirection),
    );
  },
);

export const pendingSave = derived(state, ($state) => $state.pendingSaveName);

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
    setStatusMessage("Invalid project name");
    return;
  }

  if (!get(isConnected)) {
    setStatusMessage("Not connected to server");
    return;
  }

  state.update((s) => ({ ...s, pendingSaveName: sanitized }));
  setStatusMessage("Requesting snapshot...");
  await getSnapshot();
}

async function completeSave(snapshot: Snapshot): Promise<void> {
  const $state = get(state);
  if (!$state.pendingSaveName) return;

  const projectName = $state.pendingSaveName;
  state.update((s) => ({ ...s, pendingSaveName: null }));

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

export async function loadProjectAtEndOfLine(
  name: string,
  lineId: number = 0,
): Promise<void> {
  await loadProjectWithTiming(name, AT.endOfLine(lineId));
}

async function loadProjectWithTiming(
  name: string,
  timing: ActionTiming,
): Promise<void> {
  try {
    setStatusMessage(`Loading "${name}"...`);
    const snapshot = await projectsApi.loadProject(name);

    await setTempo(snapshot.tempo, timing);
    await setScene(snapshot.scene, timing);

    if (snapshot.devices) {
      await restoreDevices(snapshot.devices);
    }

    clearAllLocalEdits();
    window.dispatchEvent(new CustomEvent("project:loaded"));

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

export async function renameProjectByName(
  oldName: string,
  newName: string,
): Promise<void> {
  const sanitized = sanitizeProjectName(newName);
  if (!sanitized) {
    setStatusMessage("Invalid project name");
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

export async function importProject(timing: ActionTiming): Promise<void> {
  try {
    setStatusMessage("Select a snapshot to import...");
    const snapshot = await projectsApi.importProject();

    if (!snapshot) {
      clearStatusMessage();
      return;
    }

    setStatusMessage("Importing...");
    await setTempo(snapshot.tempo, timing);
    await setScene(snapshot.scene, timing);

    if (snapshot.devices) {
      await restoreDevices(snapshot.devices);
    }

    clearAllLocalEdits();
    window.dispatchEvent(new CustomEvent("project:loaded"));

    setStatusMessage("Imported snapshot");
  } catch (e) {
    setStatusMessage(`Failed to import: ${e}`);
  }
}

export function projectExists(name: string): boolean {
  const sanitized = sanitizeProjectName(name);
  const $state = get(state);
  return $state.projects.some((p) => p.name === sanitized);
}

const listeners = new ListenerGroup();

export async function initializeProjectsStore(): Promise<void> {
  await listeners.add(() =>
    listen<Snapshot>(SERVER_EVENTS.SNAPSHOT, (event) => {
      completeSave(event.payload);
    }),
  );
}

export function cleanupProjectsStore(): void {
  listeners.cleanup();
}
