import { writable, derived, type Readable } from "svelte/store";
import { type EditorConfig } from "./editorConfig";
import { themes, type Theme } from "$lib/themes";
import { transformThemeColors } from "$lib/utils/colorUtils";

const STORAGE_KEY = "sova-config";

export interface ServerConfig {
  auto_start: boolean;
  port: number;
  ip: string;
}

export interface Config {
  editor: EditorConfig;
  appearance: {
    theme: string;
    font_family: string;
    zoom: number;
    hue: number;
  };
  server: ServerConfig;
}

const DEFAULT_CONFIG: Config = {
  editor: {
    mode: "normal",
    font_size: 14,
    font_family: "monospace",
    show_line_numbers: true,
    line_wrapping: false,
    highlight_active_line: true,
    cursor_blink_rate: 1200,
    tab_size: 4,
    use_tabs: false,
    close_brackets: true,
    bracket_matching: true,
    autocomplete: true,
    rectangular_selection: true,
    fold_gutter: true,
    match_highlighting: true,
  },
  appearance: {
    theme: "monokai",
    font_family: "monospace",
    zoom: 1.0,
    hue: 0,
  },
  server: {
    auto_start: false,
    port: 8080,
    ip: "127.0.0.1",
  },
};

function loadConfig(): Config {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return { ...DEFAULT_CONFIG, ...parsed };
    }
  } catch {
    // Invalid stored config
  }
  return DEFAULT_CONFIG;
}

function saveConfig(cfg: Config): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(cfg));
  } catch {
    // Storage unavailable
  }
}

function createConfigStore() {
  const { subscribe, set, update } = writable<Config>(loadConfig());

  subscribe((cfg) => {
    saveConfig(cfg);
  });

  return {
    subscribe,
    set,
    update,
  };
}

export const config = createConfigStore();

export const editorConfig: Readable<EditorConfig> = derived(
  config,
  ($config) => $config.editor,
);

export const serverConfig: Readable<ServerConfig> = derived(
  config,
  ($config) => $config.server,
);

export const currentThemeName: Readable<string> = derived(
  config,
  ($config) => $config.appearance.theme,
);

export const currentZoom: Readable<number> = derived(
  config,
  ($config) => $config.appearance.zoom,
);

export const currentHue: Readable<number> = derived(
  config,
  ($config) => $config.appearance.hue,
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

export function initializeConfig(): void {
  // Config is already loaded from localStorage in createConfigStore
}

export function cleanupConfig(): void {
  // No cleanup needed for localStorage-based config
}
