import { writable, derived } from "svelte/store";

export type SortField = "name" | "tempo" | "line_count" | "updated_at";
export type SortDirection = "asc" | "desc";

interface ProjectsUIState {
  searchQuery: string;
  sortField: SortField;
  sortDirection: SortDirection;
  statusMessage: string;
  editingName: string | null;
}

const initialState: ProjectsUIState = {
  searchQuery: "",
  sortField: "updated_at",
  sortDirection: "desc",
  statusMessage: "",
  editingName: null,
};

export const projectsUIState = writable<ProjectsUIState>(initialState);

export const statusMessage = derived(
  projectsUIState,
  ($state) => $state.statusMessage,
);
export const searchQuery = derived(
  projectsUIState,
  ($state) => $state.searchQuery,
);
export const sortField = derived(projectsUIState, ($state) => $state.sortField);
export const sortDirection = derived(
  projectsUIState,
  ($state) => $state.sortDirection,
);
export const editingName = derived(
  projectsUIState,
  ($state) => $state.editingName,
);

export function setSearchQuery(query: string): void {
  projectsUIState.update((s) => ({ ...s, searchQuery: query }));
}

export function setSort(field: SortField): void {
  projectsUIState.update((s) => {
    if (s.sortField === field) {
      return {
        ...s,
        sortDirection: s.sortDirection === "asc" ? "desc" : "asc",
      };
    }
    return { ...s, sortField: field, sortDirection: "desc" };
  });
}

export function setStatusMessage(message: string): void {
  projectsUIState.update((s) => ({ ...s, statusMessage: message }));
}

export function clearStatusMessage(): void {
  projectsUIState.update((s) => ({ ...s, statusMessage: "" }));
}

export function startEditingName(name: string): void {
  projectsUIState.update((s) => ({ ...s, editingName: name }));
}

export function stopEditingName(): void {
  projectsUIState.update((s) => ({ ...s, editingName: null }));
}
