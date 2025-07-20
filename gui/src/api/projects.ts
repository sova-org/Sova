import { invoke } from '@tauri-apps/api/core';

export interface ProjectInfo {
  name: string;
  created_at: string | undefined; // ISO date string
  updated_at: string | undefined; // ISO date string
  tempo: number | undefined;
  line_count: number | undefined;
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
  start_frame: number | undefined;
  end_frame: number | undefined;
  custom_length: number | undefined;
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
      return projects.map((project): ProjectInfo => ({
        name: project.name,
        created_at: project.created_at ?? undefined,
        updated_at: project.updated_at ?? undefined,
        tempo: project.tempo ?? undefined,
        line_count: project.line_count ?? undefined,
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