import { writable, derived, type Writable, type Readable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { type EditorConfig } from "./editorConfig";
import { themes, type Theme } from "$lib/themes";
import { transformThemeColors } from "$lib/utils/colorUtils";
import { ListenerGroup } from "./helpers";

export interface ClientConfig {
  ip: string;
  port: number;
  nickname: string;
}

export interface Config {
  editor: EditorConfig;
  appearance: {
    theme: string;
    transparency: number;
    font_family: string;
    zoom: number;
    hue: number;
  };
  client: ClientConfig;
}

export const config: Writable<Config | null> = writable(null);

export const editorConfig: Readable<EditorConfig | null> = derived(
  config,
  ($config) => $config?.editor ?? null,
);

export const currentThemeName: Readable<string> = derived(
  config,
  ($config) => $config?.appearance.theme ?? "monokai",
);

export const currentTransparency: Readable<number> = derived(
  config,
  ($config) => $config?.appearance.transparency ?? 100,
);

export const currentZoom: Readable<number> = derived(
  config,
  ($config) => $config?.appearance.zoom ?? 1.0,
);

export const currentHue: Readable<number> = derived(
  config,
  ($config) => $config?.appearance.hue ?? 0,
);

export const currentTheme: Readable<Theme> = derived(
  currentThemeName,
  ($name) => {
    const theme = themes[$name];
    if (!theme) {
      throw new Error(
        `Invalid theme "${$name}" specified in config. Available themes: ${Object.keys(themes).slice(0, 10).join(", ")}...`,
      );
    }
    return theme;
  },
);

export const currentThemeTransformed: Readable<Theme> = derived(
  [currentTheme, currentHue],
  ([$theme, $hue]) => transformThemeColors($theme, $hue),
);

export const clientConfig: Readable<ClientConfig | null> = derived(
  config,
  ($config) => $config?.client ?? null,
);

// Runtime nickname - instance-specific, not persisted to TOML
// This allows multiple GUI instances to have different nicknames
export const runtimeNickname: Writable<string> = writable("");
let nicknameInitialized = false;

export function setRuntimeNickname(nickname: string): void {
  runtimeNickname.set(nickname);
}

const listeners = new ListenerGroup();

export async function initializeConfig(): Promise<void> {
  try {
    const loadedConfig = await invoke<Config>("get_config");
    config.set(loadedConfig);

    // Initialize runtime nickname from config ONLY on first load
    if (!nicknameInitialized && loadedConfig.client?.nickname) {
      runtimeNickname.set(loadedConfig.client.nickname);
      nicknameInitialized = true;
    }
  } catch {
    // Failed to load config - will use defaults
  }

  await listeners.add(() =>
    listen<Config>("config-update", (event) => {
      // Update config store but NOT runtimeNickname
      // This keeps nicknames independent across instances
      config.set(event.payload);
    }),
  );
}

export function cleanupConfig(): void {
  listeners.cleanup();
  nicknameInitialized = false;
}
