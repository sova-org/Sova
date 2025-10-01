import React, { useEffect, useRef, useState } from 'react';
import { Save, Trash2, Clock, Hash, RefreshCw, Download, RotateCcw, Timer, FileText, FolderOpen } from 'lucide-react';
import { useStore } from '@nanostores/react';
import { ProjectsAPI, ProjectInfo, Snapshot } from '../../api/projects';
import { invoke } from '@tauri-apps/api/core';
import { optionsPanelStore } from '../../stores/ui/panels';
import {
  projectStore,
  setProjects,
  setSearchQuery,
  setSaving,
  setSaveProjectName,
  setStatusMessage,
  showDeleteConfirmation,
  hideDeleteConfirmation,
  showSaveOverwriteConfirmation,
  hideSaveOverwriteConfirmation,
  getFilteredProjects,
  setPendingSaveProjectName
} from '../../stores/project';
import { ActionTiming } from '../../types';

export const FilesPanel: React.FC = () => {
  const state = useStore(projectStore);
  const optionsState = useStore(optionsPanelStore);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const saveInputRef = useRef<HTMLInputElement>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [projectsDirectory, setProjectsDirectory] = useState<string>('');

  // Load projects on mount
  useEffect(() => {
    loadProjects();
    loadProjectsDirectory();
  }, []);

  const loadProjectsDirectory = async () => {
    try {
      const dir = await invoke<string>('get_projects_directory');
      setProjectsDirectory(dir);
    } catch (error) {
      console.error('Error getting projects directory:', error);
    }
  };

  // Handle keyboard events for modals
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (state.showDeleteConfirmation) {
        if (e.key === 'Escape') {
          e.preventDefault();
          hideDeleteConfirmation();
        } else if (e.key === 'Enter') {
          e.preventDefault();
          confirmDelete();
        }
      } else if (state.showSaveOverwriteConfirmation) {
        if (e.key === 'Escape') {
          e.preventDefault();
          hideSaveOverwriteConfirmation();
        } else if (e.key === 'Enter') {
          e.preventDefault();
          confirmOverwrite();
        }
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [state.showDeleteConfirmation, state.showSaveOverwriteConfirmation, state.projectToDelete, state.projectToOverwrite]);

  // Also refresh when this component becomes visible (when Files tab is clicked)
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (!document.hidden) {
        loadProjects();
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => document.removeEventListener('visibilitychange', handleVisibilityChange);
  }, []);

  // Refresh when the Files tab becomes active
  useEffect(() => {
    if (optionsState.activeTab === 'files') {
      loadProjects();
    }
  }, [optionsState.activeTab]);

  const loadProjects = async () => {
    setIsLoading(true);
    try {
      const projects = await ProjectsAPI.listProjects();
      setProjects(projects);
      setStatusMessage(`Loaded ${projects.length} projects`);
    } catch (error) {
      setStatusMessage(`Error loading projects: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  // Apply project snapshot following TUI approach
  const applyProjectSnapshot = async (snapshot: Snapshot, timing: ActionTiming) => {
    try {
      // 1. Update local state immediately (this would be handled by stores in a real implementation)
      // Note: In the TUI this updates editor scene, tempo, and resets grid selection
      
      // 2. Send messages to server with the specified timing
      await invoke('send_message', {
        message: { SetTempo: [snapshot.tempo, timing] }
      });
      
      await invoke('send_message', {
        message: { SetScene: [snapshot.scene, timing] }
      });
      
      // 3. Reset grid selection (single cell at 0,0)
      await invoke('send_message', {
        message: { 
          UpdateGridSelection: {
            start: [0, 0],
            end: [0, 0]
          }
        }
      });
      
      // 4. Send individual scripts to ensure server has all script contents
      let scriptsCount = 0;
      for (let lineIdx = 0; lineIdx < snapshot.scene.lines.length; lineIdx++) {
        const line = snapshot.scene.lines[lineIdx];
        if (line && line.scripts) {
          for (let frameIdx = 0; frameIdx < line.scripts.length; frameIdx++) {
            const script = line.scripts[frameIdx];
            if (script && script.content && script.content.trim() !== '') {
              await invoke('send_message', {
                message: { 
                  SetScript: [lineIdx, frameIdx, script.content, timing]
                }
              });
              scriptsCount++;
            }
          }
        }
      }
      
      // 5. Request scene to ensure UI updates
      await invoke('send_message', {
        message: { GetScene: null }
      });
      
      if (scriptsCount > 0) {
        setStatusMessage(`Applied project with ${scriptsCount} scripts`);
      }
      
    } catch (error) {
      throw new Error(`Failed to apply project snapshot: ${error}`);
    }
  };

  const filteredProjects = getFilteredProjects(state);

  const handleOpenProjectFolder = async () => {
    try {
      const projectsDir = await invoke<string>('get_projects_directory');
      
      // Import the opener API dynamically
      const { openPath } = await import('@tauri-apps/plugin-opener');
      
      await openPath(projectsDir);
      setStatusMessage(`Opened projects folder: ${projectsDir}`);
    } catch (error) {
      setStatusMessage(`Error opening projects folder: ${error}`);
    }
  };

  const handleSaveCurrentProject = async () => {
    setSaving(true);
    setStatusMessage('Enter project name to save current scene...');
  };

  const handleSaveConfirm = async () => {
    const projectName = state.saveProjectName.trim();
    if (!projectName) {
      setStatusMessage('Project name cannot be empty');
      return;
    }

    const existingProject = state.projects.find(p => p.name === projectName);
    if (existingProject) {
      showSaveOverwriteConfirmation(projectName);
      setStatusMessage(`Project '${projectName}' already exists. Overwrite?`);
    } else {
      await saveNewProject(projectName);
    }
  };

  const saveNewProject = async (projectName: string) => {
    try {
      setStatusMessage(`Requesting snapshot to save as '${projectName}'...`);
      
      // Set the pending save project name so the message handler knows what to do
      setPendingSaveProjectName(projectName);
      
      // Request snapshot from server
      await invoke('send_message', { 
        message: { GetSnapshot: null }
      });
      
      // Note: The actual save will be handled by the message handler when the server responds
      
    } catch (error) {
      setStatusMessage(`Error requesting snapshot: ${error}`);
      setPendingSaveProjectName(undefined);
    }
  };

  const handleLoadProject = async (project: ProjectInfo) => {
    try {
      setStatusMessage(`Loading project '${project.name}' immediately...`);
      const snapshot = await ProjectsAPI.loadProject(project.name);
      
      // Apply the project with Immediate timing
      await applyProjectSnapshot(snapshot, "Immediate");
      
      setStatusMessage(`Project '${project.name}' loaded immediately`);
    } catch (error) {
      setStatusMessage(`Error loading project: ${error}`);
    }
  };

  const handleLoadProjectEndOfLine = async (project: ProjectInfo) => {
    try {
      setStatusMessage(`Loading project '${project.name}' at end of scene...`);
      const snapshot = await ProjectsAPI.loadProject(project.name);
      
      // Apply the project with EndOfLine timing
      await applyProjectSnapshot(snapshot, { EndOfLine: 0 });
      
      setStatusMessage(`Project '${project.name}' scheduled to load at end of current scene`);
    } catch (error) {
      setStatusMessage(`Error scheduling project load: ${error}`);
    }
  };

  const handleOverwriteProject = async (project: ProjectInfo) => {
    showSaveOverwriteConfirmation(project.name);
    setStatusMessage(`Overwrite project '${project.name}'?`);
  };

  const handleDeleteProject = (project: ProjectInfo) => {
    showDeleteConfirmation(project.name);
    setStatusMessage(`Delete project '${project.name}'?`);
  };

  const confirmDelete = async () => {
    if (!state.projectToDelete) return;
    
    try {
      setStatusMessage(`Deleting project '${state.projectToDelete}'...`);
      await ProjectsAPI.deleteProject(state.projectToDelete);
      setStatusMessage(`Project '${state.projectToDelete}' deleted successfully`);
      hideDeleteConfirmation();
      await loadProjects();
    } catch (error) {
      setStatusMessage(`Error deleting project: ${error}`);
      hideDeleteConfirmation();
    }
  };

  const confirmOverwrite = async () => {
    if (!state.projectToOverwrite) return;
    
    try {
      setStatusMessage(`Requesting snapshot to overwrite '${state.projectToOverwrite}'...`);
      
      // Set the pending save project name so the message handler knows what to do
      setPendingSaveProjectName(state.projectToOverwrite);
      
      await invoke('send_message', { 
        message: { GetSnapshot: null }
      });
      
      // Note: The actual save will be handled by the message handler when the server responds
      
      hideSaveOverwriteConfirmation();
      setSaving(false);
      setSaveProjectName('');
    } catch (error) {
      setStatusMessage(`Error requesting snapshot: ${error}`);
      setPendingSaveProjectName(undefined);
      hideSaveOverwriteConfirmation();
    }
  };

  const formatDate = (dateString?: string): string => {
    if (!dateString) return 'N/A';
    const date = new Date(dateString);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  };

  return (
    <div className="h-full flex flex-col overflow-hidden">
      {/* Header */}
      <div className="p-4 border-b" style={{ borderColor: 'var(--color-border)' }}>
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-lg font-semibold" style={{ color: 'var(--color-text)' }}>
            Projects
          </h3>
        </div>
        
        {/* Action buttons */}
        <div className="mb-4 flex items-center space-x-2">
          <button
            onClick={handleSaveCurrentProject}
            className="flex-1 flex items-center justify-center space-x-2 px-4 py-2 border font-medium hover:bg-opacity-80 transition-colors"
            style={{ 
              borderColor: 'var(--color-border)', 
              backgroundColor: 'var(--color-primary)',
              color: 'white'
            }}
            title={`Save current scene as a new project${projectsDirectory ? ` to ${projectsDirectory}` : ''}`}
          >
            <Save size={16} />
            <span>Save</span>
          </button>
          <button
            onClick={loadProjects}
            disabled={isLoading}
            className="flex items-center justify-center space-x-2 px-4 py-2 border font-medium hover:bg-opacity-80 transition-colors disabled:opacity-50"
            style={{ 
              borderColor: 'var(--color-border)', 
              backgroundColor: 'var(--color-surface)',
              color: 'var(--color-text)'
            }}
            title="Refresh project list"
          >
            <RefreshCw size={16} />
            <span>Refresh</span>
          </button>
          <button
            onClick={handleOpenProjectFolder}
            className="flex items-center justify-center space-x-2 px-4 py-2 border font-medium hover:bg-opacity-80 transition-colors"
            style={{ 
              borderColor: 'var(--color-border)', 
              backgroundColor: 'var(--color-surface)',
              color: 'var(--color-text)'
            }}
            title="Open projects folder"
          >
            <FolderOpen size={16} />
            <span>Open Projects Folder</span>
          </button>
        </div>

        {/* Search */}
        <div className="flex space-x-2">
          <div className="flex-1">
            <input
              ref={searchInputRef}
              type="text"
              value={state.searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="w-full p-2 border"
              style={{ 
                borderColor: 'var(--color-border)', 
                backgroundColor: 'var(--color-surface)', 
                color: 'var(--color-text)' 
              }}
              placeholder="Search projects..."
            />
          </div>
          <button
            onClick={() => setSearchQuery('')}
            className="px-3 py-2 border hover:bg-opacity-80 transition-colors"
            style={{ 
              borderColor: 'var(--color-border)', 
              backgroundColor: 'var(--color-surface)',
              color: 'var(--color-text)'
            }}
            title="Clear search"
          >
            <RotateCcw size={16} />
          </button>
        </div>
      </div>

      {/* Save Project Input */}
      {state.isSaving && (
        <div className="p-4 border-b" style={{ 
          borderColor: 'var(--color-border)',
          backgroundColor: 'var(--color-surface)'
        }}>
          <label className="block text-sm font-medium mb-2" style={{ color: 'var(--color-text)' }}>
            Project Name
          </label>
          <div className="flex space-x-2">
            <input
              ref={saveInputRef}
              type="text"
              value={state.saveProjectName}
              onChange={(e) => setSaveProjectName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleSaveConfirm();
                if (e.key === 'Escape') {
                  setSaving(false);
                  setSaveProjectName('');
                }
              }}
              className="flex-1 p-2 border"
              style={{ 
                borderColor: 'var(--color-border)', 
                backgroundColor: 'var(--color-surface)', 
                color: 'var(--color-text)' 
              }}
              placeholder="Enter project name..."
              autoFocus
            />
            <button
              onClick={handleSaveConfirm}
              className="px-4 py-2 border font-medium hover:bg-opacity-80 transition-colors"
              style={{ 
                borderColor: 'var(--color-border)', 
                backgroundColor: 'var(--color-primary)',
                color: 'white'
              }}
            >
              Save
            </button>
            <button
              onClick={() => {
                setSaving(false);
                setSaveProjectName('');
              }}
              className="px-4 py-2 border hover:bg-opacity-80 transition-colors"
              style={{ 
                borderColor: 'var(--color-border)', 
                backgroundColor: 'var(--color-surface)',
                color: 'var(--color-text)'
              }}
            >
              Cancel
            </button>
          </div>
        </div>
      )}

      {/* Project list */}
      <div className="flex-1 overflow-y-auto">
        {isLoading ? (
          <div className="p-8 text-center" style={{ color: 'var(--color-muted)' }}>
            <RefreshCw size={48} className="mx-auto mb-4 opacity-50 animate-spin" />
            <p>Loading projects...</p>
          </div>
        ) : filteredProjects.length === 0 ? (
          <div className="p-8 text-center" style={{ color: 'var(--color-muted)' }}>
            <FileText size={48} className="mx-auto mb-4 opacity-50" />
            <p>No projects found</p>
            {state.searchQuery && (
              <p className="text-sm mt-2">Try adjusting your search query</p>
            )}
          </div>
        ) : (
          <div className="p-2 space-y-2">
            {filteredProjects.map((project, _index) => (
              <div
                key={project.name}
                className="border p-4 hover:bg-opacity-50 transition-colors"
                style={{
                  borderColor: 'var(--color-border)',
                  backgroundColor: 'var(--color-surface)',
                }}
              >
                {/* Project Header */}
                <div className="flex items-center justify-between mb-3">
                  <h4 className="text-lg font-medium" style={{ color: 'var(--color-text)' }}>
                    {project.name}
                  </h4>
                  <div className="flex items-center space-x-2">
                    <button
                      onClick={() => handleLoadProject(project)}
                      className="p-2 hover:bg-opacity-80 transition-colors"
                      style={{ 
                        backgroundColor: 'var(--color-primary)',
                        color: 'white'
                      }}
                      title="Load project immediately"
                    >
                      <Download size={16} />
                    </button>
                    <button
                      onClick={() => handleLoadProjectEndOfLine(project)}
                      className="p-2 hover:bg-opacity-80 transition-colors"
                      style={{ 
                        backgroundColor: 'var(--color-primary)',
                        color: 'white',
                        opacity: 0.8
                      }}
                      title="Load project at end of current scene"
                    >
                      <Timer size={16} />
                    </button>
                    <button
                      onClick={() => handleOverwriteProject(project)}
                      className="p-2 hover:bg-opacity-80 transition-colors"
                      style={{ 
                        backgroundColor: 'var(--color-warning)',
                        color: 'white'
                      }}
                      title="Overwrite with current scene"
                    >
                      <Save size={16} />
                    </button>
                    <button
                      onClick={() => handleDeleteProject(project)}
                      className="p-2 hover:bg-opacity-80 transition-colors"
                      style={{ 
                        backgroundColor: 'var(--color-error)',
                        color: 'white'
                      }}
                      title="Delete project"
                    >
                      <Trash2 size={16} />
                    </button>
                  </div>
                </div>
                
                {/* Project Metadata */}
                <div className="grid grid-cols-2 gap-4 text-sm" style={{ color: 'var(--color-muted)' }}>
                  <div className="flex items-center space-x-2">
                    <Clock size={14} />
                    <span>Tempo: {project.tempo?.toFixed(1) || 'N/A'}</span>
                  </div>
                  <div className="flex items-center space-x-2">
                    <Hash size={14} />
                    <span>Lines: {project.line_count || 'N/A'}</span>
                  </div>
                </div>
                
                <div className="mt-2 text-xs" style={{ color: 'var(--color-muted)' }}>
                  {project.updated_at ? (
                    <span>Last saved: {formatDate(project.updated_at)}</span>
                  ) : (
                    <span>Created: {formatDate(project.created_at)}</span>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer */}
      <div className="border-t" style={{ borderColor: 'var(--color-border)' }}>
        {/* Status message */}
        {state.statusMessage && (
          <div className="p-3 text-sm" style={{ 
            backgroundColor: 'var(--color-surface)',
            color: 'var(--color-muted)'
          }}>
            {state.statusMessage}
          </div>
        )}
        
        {/* Projects directory */}
        {projectsDirectory && (
          <div className="px-3 py-2 text-xs" style={{ 
            backgroundColor: 'var(--color-surface)',
            color: 'var(--color-muted)',
            opacity: 0.8
          }}>
            <span className="truncate">
              <span className="opacity-60">Location:</span> {projectsDirectory}
            </span>
          </div>
        )}
      </div>

      {/* Confirmation dialogs */}
      {state.showDeleteConfirmation && (
        <div 
          className="fixed inset-0 z-50 flex items-center justify-center"
          onClick={hideDeleteConfirmation}
        >
          <div 
            className="border p-6 max-w-sm mx-4 shadow-2xl" 
            style={{ 
              backgroundColor: 'var(--color-surface)',
              borderColor: 'var(--color-border)'
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 className="text-lg font-semibold mb-4" style={{ color: 'var(--color-text)' }}>
              Confirm Delete
            </h3>
            <p className="mb-6" style={{ color: 'var(--color-muted)' }}>
              Really delete '{state.projectToDelete}'?
            </p>
            <div className="flex space-x-3">
              <button
                onClick={confirmDelete}
                className="flex-1 px-4 py-2 font-medium text-white hover:bg-opacity-80 transition-colors"
                style={{ backgroundColor: 'var(--color-error)' }}
                autoFocus
              >
                Delete
              </button>
              <button
                onClick={hideDeleteConfirmation}
                className="flex-1 px-4 py-2 border hover:bg-opacity-80 transition-colors"
                style={{ 
                  borderColor: 'var(--color-border)', 
                  backgroundColor: 'var(--color-surface)',
                  color: 'var(--color-text)'
                }}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}

      {state.showSaveOverwriteConfirmation && (
        <div 
          className="fixed inset-0 z-50 flex items-center justify-center"
          onClick={hideSaveOverwriteConfirmation}
        >
          <div 
            className="border p-6 max-w-sm mx-4 shadow-2xl" 
            style={{ 
              backgroundColor: 'var(--color-surface)',
              borderColor: 'var(--color-border)'
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 className="text-lg font-semibold mb-4" style={{ color: 'var(--color-text)' }}>
              Confirm Overwrite
            </h3>
            <p className="mb-6" style={{ color: 'var(--color-muted)' }}>
              Overwrite '{state.projectToOverwrite}' with current scene?
            </p>
            <div className="flex space-x-3">
              <button
                onClick={confirmOverwrite}
                className="flex-1 px-4 py-2 font-medium text-white hover:bg-opacity-80 transition-colors"
                style={{ backgroundColor: 'var(--color-primary)' }}
                autoFocus
              >
                Overwrite
              </button>
              <button
                onClick={hideSaveOverwriteConfirmation}
                className="flex-1 px-4 py-2 border hover:bg-opacity-80 transition-colors"
                style={{ 
                  borderColor: 'var(--color-border)', 
                  backgroundColor: 'var(--color-surface)',
                  color: 'var(--color-text)'
                }}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};