import { invoke } from '@tauri-apps/api/core';

export interface ProjectInfo {
  name: string;
  created_at?: string; // ISO date string
  updated_at?: string; // ISO date string
  tempo?: number;
  line_count?: number;
}

export interface Snapshot {
  scene: Scene;
  tempo: number;
  beat: number;
  micros: number;
  quantum: number;
}

export interface Scene {
  length: number;
  lines: Line[];
}

export interface Line {
  frames: number[];
  enabled_frames: boolean[];
  scripts: Script[];
  frame_names: (string | null)[];
  frame_repetitions: number[];
  speed_factor: number;
  index: number;
  start_frame?: number;
  end_frame?: number;
  custom_length?: number;
}

export interface Script {
  content: string;
  lang: string;
  index: number;
}

export class ProjectsAPI {
  static async listProjects(): Promise<ProjectInfo[]> {
    try {
      const projects = await invoke<ProjectInfo[]>('list_projects');
      // Convert date strings to Date objects for easier handling
      return projects.map(project => ({
        ...project,
        created_at: project.created_at ? project.created_at : undefined,
        updated_at: project.updated_at ? project.updated_at : undefined,
      }));
    } catch (error) {
      console.error('Failed to list projects:', error);
      throw new Error(`Failed to list projects: ${error}`);
    }
  }

  static async saveProject(snapshot: Snapshot, projectName: string): Promise<void> {
    try {
      await invoke('save_project', { snapshot, projectName });
    } catch (error) {
      console.error('Failed to save project:', error);
      throw new Error(`Failed to save project: ${error}`);
    }
  }

  static async loadProject(projectName: string): Promise<Snapshot> {
    try {
      return await invoke<Snapshot>('load_project', { projectName });
    } catch (error) {
      console.error('Failed to load project:', error);
      throw new Error(`Failed to load project: ${error}`);
    }
  }

  static async deleteProject(projectName: string): Promise<void> {
    try {
      await invoke('delete_project', { projectName });
    } catch (error) {
      console.error('Failed to delete project:', error);
      throw new Error(`Failed to delete project: ${error}`);
    }
  }
}