import React, { useEffect, useRef, useState } from 'react';
import { Save, Trash2, Clock, Hash, RefreshCw, Download, RotateCcw, Timer, FileText } from 'lucide-react';
import { useStore } from '@nanostores/react';
import { ProjectsAPI, ProjectInfo } from '../api/projects';
import { invoke } from '@tauri-apps/api/core';
import { optionsPanelStore } from '../stores/optionsPanelStore';
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
  getFilteredProjects
} from '../stores/projectStore';

export const FilesPanel: React.FC = () => {
  const state = useStore(projectStore);
  const optionsState = useStore(optionsPanelStore);
  const searchInputRef = useRef<HTMLInputElement>(null);
  const saveInputRef = useRef<HTMLInputElement>(null);
  const [isLoading, setIsLoading] = useState(false);

  // Load projects on mount
  useEffect(() => {
    loadProjects();
  }, []);

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

  const filteredProjects = getFilteredProjects(state);

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
      
      // Request snapshot from server
      await invoke('send_message', { 
        message: { GetSnapshot: null }
      });
      
      // Wait a bit for the snapshot to be processed and then refresh
      setTimeout(async () => {
        await loadProjects();
        setStatusMessage(`Project '${projectName}' save requested - check the list for updates`);
      }, 1500);
      
      setSaving(false);
      setSaveProjectName('');
    } catch (error) {
      setStatusMessage(`Error requesting snapshot: ${error}`);
    }
  };

  const handleLoadProject = async (project: ProjectInfo) => {
    try {
      setStatusMessage(`Loading project '${project.name}' immediately...`);
      const snapshot = await ProjectsAPI.loadProject(project.name);
      
      // Set the scene first (Immediate timing)
      await invoke('send_message', {
        message: { 
          SetScene: [snapshot.scene, { Immediate: null }]
        }
      });
      
      // Then set the tempo (Immediate timing)
      await invoke('send_message', {
        message: {
          SetTempo: [snapshot.tempo, { Immediate: null }]
        }
      });
      
      // Request the scene to ensure UI updates
      setTimeout(async () => {
        await invoke('send_message', {
          message: { GetScene: null }
        });
      }, 100);
      
      setStatusMessage(`Project '${project.name}' loaded immediately - grid should update shortly`);
    } catch (error) {
      setStatusMessage(`Error loading project: ${error}`);
    }
  };

  const handleLoadProjectEndOfScene = async (project: ProjectInfo) => {
    try {
      setStatusMessage(`Loading project '${project.name}' at end of scene...`);
      const snapshot = await ProjectsAPI.loadProject(project.name);
      
      // Set the scene at end of scene
      await invoke('send_message', {
        message: { 
          SetScene: [snapshot.scene, { EndOfScene: null }]
        }
      });
      
      // Set the tempo at end of scene
      await invoke('send_message', {
        message: {
          SetTempo: [snapshot.tempo, { EndOfScene: null }]
        }
      });
      
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
      
      await invoke('send_message', { 
        message: { GetSnapshot: null }
      });
      
      // Wait a bit for the snapshot to be processed and then refresh
      setTimeout(async () => {
        await loadProjects();
        setStatusMessage(`Project '${state.projectToOverwrite}' overwrite requested - check the list for updates`);
      }, 1500);
      
      hideSaveOverwriteConfirmation();
      setSaving(false);
      setSaveProjectName('');
    } catch (error) {
      setStatusMessage(`Error requesting snapshot: ${error}`);
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
            Project Files
          </h3>
          <button
            onClick={loadProjects}
            disabled={isLoading}
            className="p-2 border hover:bg-opacity-80 transition-colors disabled:opacity-50"
            style={{ 
              borderColor: 'var(--color-border)', 
              backgroundColor: 'var(--color-primary)',
              color: 'white'
            }}
            title="Refresh project list"
          >
            <RefreshCw size={16} className={isLoading ? 'animate-spin' : ''} />
          </button>
        </div>
        
        {/* Save Current Project */}
        <div className="mb-4">
          <button
            onClick={handleSaveCurrentProject}
            className="w-full flex items-center justify-center space-x-2 px-4 py-3 border font-medium hover:bg-opacity-80 transition-colors"
            style={{ 
              borderColor: 'var(--color-border)', 
              backgroundColor: 'var(--color-primary)',
              color: 'white'
            }}
          >
            <Save size={18} />
            <span>Save Current Scene as Project</span>
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
            {filteredProjects.map((project, index) => (
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
                      onClick={() => handleLoadProjectEndOfScene(project)}
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

      {/* Status message */}
      {state.statusMessage && (
        <div className="p-3 border-t text-sm" style={{ 
          borderColor: 'var(--color-border)', 
          backgroundColor: 'var(--color-surface)',
          color: 'var(--color-muted)'
        }}>
          {state.statusMessage}
        </div>
      )}

      {/* Confirmation dialogs */}
      {state.showDeleteConfirmation && (
        <div className="absolute inset-0 bg-black bg-opacity-50 flex items-center justify-center">
          <div className="bg-white border p-6 max-w-sm mx-4" style={{ 
            backgroundColor: 'var(--color-surface)',
            borderColor: 'var(--color-border)'
          }}>
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
        <div className="absolute inset-0 bg-black bg-opacity-50 flex items-center justify-center">
          <div className="bg-white border p-6 max-w-sm mx-4" style={{ 
            backgroundColor: 'var(--color-surface)',
            borderColor: 'var(--color-border)'
          }}>
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