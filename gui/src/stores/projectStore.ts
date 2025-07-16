import { atom } from 'nanostores';
import { ProjectInfo } from '../api/projects';

export interface ProjectState {
  projects: ProjectInfo[];
  selectedIndex: number;
  searchQuery: string;
  isSearching: boolean;
  isSaving: boolean;
  saveProjectName: string;
  statusMessage: string;
  showDeleteConfirmation: boolean;
  projectToDelete: string | undefined;
  showSaveOverwriteConfirmation: boolean;
  projectToOverwrite: string | undefined;
}

const initialState: ProjectState = {
  projects: [],
  selectedIndex: 0,
  searchQuery: '',
  isSearching: false,
  isSaving: false,
  saveProjectName: '',
  statusMessage: '',
  showDeleteConfirmation: false,
  projectToDelete: undefined,
  showSaveOverwriteConfirmation: false,
  projectToOverwrite: undefined,
};

export const projectStore = atom<ProjectState>(initialState);

// Actions
export const setProjects = (projects: ProjectInfo[]) => {
  projectStore.set({ ...projectStore.get(), projects });
};

export const setSelectedIndex = (index: number) => {
  const state = projectStore.get();
  const maxIndex = Math.max(0, state.projects.length - 1);
  projectStore.set({ ...state, selectedIndex: Math.min(index, maxIndex) });
};

export const setSearchQuery = (query: string) => {
  projectStore.set({ 
    ...projectStore.get(), 
    searchQuery: query,
    selectedIndex: 0 // Reset selection when searching
  });
};

export const setSearching = (isSearching: boolean) => {
  projectStore.set({ ...projectStore.get(), isSearching });
};

export const setSaving = (isSaving: boolean) => {
  projectStore.set({ 
    ...projectStore.get(), 
    isSaving,
    saveProjectName: isSaving ? projectStore.get().saveProjectName : ''
  });
};

export const setSaveProjectName = (name: string) => {
  projectStore.set({ ...projectStore.get(), saveProjectName: name });
};

export const setStatusMessage = (message: string) => {
  projectStore.set({ ...projectStore.get(), statusMessage: message });
};

export const showDeleteConfirmation = (projectName: string) => {
  projectStore.set({ 
    ...projectStore.get(), 
    showDeleteConfirmation: true,
    projectToDelete: projectName
  });
};

export const hideDeleteConfirmation = () => {
  projectStore.set({ 
    ...projectStore.get(), 
    showDeleteConfirmation: false,
    projectToDelete: undefined
  });
};

export const showSaveOverwriteConfirmation = (projectName: string) => {
  projectStore.set({ 
    ...projectStore.get(), 
    showSaveOverwriteConfirmation: true,
    projectToOverwrite: projectName
  });
};

export const hideSaveOverwriteConfirmation = () => {
  projectStore.set({ 
    ...projectStore.get(), 
    showSaveOverwriteConfirmation: false,
    projectToOverwrite: undefined
  });
};

// Utility functions
export const getFilteredProjects = (state: ProjectState): ProjectInfo[] => {
  if (!state.searchQuery) return state.projects;
  
  return state.projects.filter(project => 
    fuzzyMatch(state.searchQuery, project.name)
  );
};

const fuzzyMatch = (query: string, text: string): boolean => {
  if (!query) return true;
  
  const queryChars = query.toLowerCase().split('');
  const textLower = text.toLowerCase();
  let textIndex = 0;
  
  for (const queryChar of queryChars) {
    const foundIndex = textLower.indexOf(queryChar, textIndex);
    if (foundIndex === -1) return false;
    textIndex = foundIndex + 1;
  }
  
  return true;
};