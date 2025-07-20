import { atom } from 'nanostores';
import { ProjectInfo } from '../api/projects';
import { updateStore } from '../utils/store-helpers';

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
  pendingSaveProjectName: string | undefined;
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
  pendingSaveProjectName: undefined,
};

export const projectStore = atom<ProjectState>(initialState);

// Actions
export const setProjects = (projects: ProjectInfo[]) => {
  updateStore(projectStore, { projects });
};

export const setSelectedIndex = (index: number) => {
  const state = projectStore.get();
  const maxIndex = Math.max(0, state.projects.length - 1);
  updateStore(projectStore, { selectedIndex: Math.min(index, maxIndex) });
};

export const setSearchQuery = (query: string) => {
  updateStore(projectStore, { 
    searchQuery: query,
    selectedIndex: 0 // Reset selection when searching
  });
};

export const setSearching = (isSearching: boolean) => {
  updateStore(projectStore, { isSearching });
};

export const setSaving = (isSaving: boolean) => {
  updateStore(projectStore, { 
    isSaving,
    saveProjectName: isSaving ? projectStore.get().saveProjectName : ''
  });
};

export const setSaveProjectName = (name: string) => {
  updateStore(projectStore, { saveProjectName: name });
};

export const setStatusMessage = (message: string) => {
  updateStore(projectStore, { statusMessage: message });
};

export const showDeleteConfirmation = (projectName: string) => {
  updateStore(projectStore, { 
    showDeleteConfirmation: true,
    projectToDelete: projectName
  });
};

export const hideDeleteConfirmation = () => {
  updateStore(projectStore, { 
    showDeleteConfirmation: false,
    projectToDelete: undefined
  });
};

export const showSaveOverwriteConfirmation = (projectName: string) => {
  updateStore(projectStore, { 
    showSaveOverwriteConfirmation: true,
    projectToOverwrite: projectName
  });
};

export const hideSaveOverwriteConfirmation = () => {
  updateStore(projectStore, { 
    showSaveOverwriteConfirmation: false,
    projectToOverwrite: undefined
  });
};

export const setPendingSaveProjectName = (projectName: string | undefined) => {
  updateStore(projectStore, { pendingSaveProjectName: projectName });
};

// Message handler for project-related server messages
export const handleProjectMessage = async (message: any): Promise<void> => {
  if (typeof message === 'object' && message !== null && 'Snapshot' in message) {
    const snapshot = message.Snapshot;
    const state = projectStore.get();
    
    if (state.pendingSaveProjectName) {
      try {
        setStatusMessage(`Saving project '${state.pendingSaveProjectName}'...`);
        
        // Import the ProjectsAPI dynamically to avoid circular dependencies
        const { ProjectsAPI } = await import('../api/projects');
        
        await ProjectsAPI.saveProject(snapshot, state.pendingSaveProjectName);
        
        setStatusMessage(`Project '${state.pendingSaveProjectName}' saved successfully!`);
        
        // Clear pending save state
        setPendingSaveProjectName(undefined);
        setSaving(false);
        setSaveProjectName('');
        
        // Refresh the projects list
        const projects = await ProjectsAPI.listProjects();
        setProjects(projects);
        
      } catch (error) {
        setStatusMessage(`Error saving project: ${error}`);
        setPendingSaveProjectName(undefined);
      }
    }
  }
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