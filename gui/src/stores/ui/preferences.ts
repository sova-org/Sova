import { persistentAtom, persistentMap } from '@nanostores/persistent';
import { updateStore } from '../../utils/store-helpers';
import type { ThemeMode } from '../../hooks/useMaterialPalette';

// Editor Settings
export interface EditorSettings {
  fontSize: number;
  tabSize: number;
  vimMode: boolean;
  fontFamily: string;
}

export const editorSettingsStore = persistentAtom<EditorSettings>('editorSettings', {
  fontSize: 14,
  tabSize: 4,
  vimMode: false,
  fontFamily: '"JetBrains Mono", monospace',
}, {
  encode: JSON.stringify,
  decode: JSON.parse,
});

export const setFontSize = (fontSize: number) => {
  updateStore(editorSettingsStore, { fontSize });
};

export const setTabSize = (tabSize: number) => {
  updateStore(editorSettingsStore, { tabSize });
};

export const setVimMode = (vimMode: boolean) => {
  updateStore(editorSettingsStore, { vimMode });
};

export const toggleVimMode = () => {
  const currentSettings = editorSettingsStore.get();
  updateStore(editorSettingsStore, { vimMode: !currentSettings.vimMode });
};

export const setFontFamily = (fontFamily: string) => {
  updateStore(editorSettingsStore, { fontFamily });
};

// Color Palette Settings
export interface ColorPaletteSettings {
  hueRotation: number;
  themeMode: ThemeMode;
  saturation: number;
  brightness: number;
  basePrimary: number;
  baseSecondary: number;
}

const defaultColorSettings: ColorPaletteSettings = {
  hueRotation: 0,
  themeMode: 'light',
  saturation: 75,
  brightness: 50,
  basePrimary: Math.floor(Math.random() * 360),
  baseSecondary: (() => {
    const primary = Math.floor(Math.random() * 360);
    const offset = 120 + Math.floor(Math.random() * 120);
    return (primary + offset) % 360;
  })(),
};

export const $colorPaletteSettings = persistentAtom<ColorPaletteSettings>(
  'colorPaletteSettings',
  defaultColorSettings,
  {
    encode: JSON.stringify,
    decode: JSON.parse,
  }
);

export const updateHueRotation = (value: number) => {
  updateStore($colorPaletteSettings, { hueRotation: value });
};

export const updateThemeMode = (mode: ThemeMode) => {
  updateStore($colorPaletteSettings, { themeMode: mode });
};

export const updateSaturation = (value: number) => {
  updateStore($colorPaletteSettings, { saturation: value });
};

export const updateBrightness = (value: number) => {
  updateStore($colorPaletteSettings, { brightness: value });
};

export const regenerateBaseColors = () => {
  const newPrimary = Math.floor(Math.random() * 360);
  const offset = 120 + Math.floor(Math.random() * 120);
  const newSecondary = (newPrimary + offset) % 360;

  updateStore($colorPaletteSettings, {
    basePrimary: newPrimary,
    baseSecondary: newSecondary,
    hueRotation: 0,
    saturation: 75,
    brightness: 50,
  });
};

export const toggleTheme = () => {
  const current = $colorPaletteSettings.get();
  updateThemeMode(current.themeMode === 'light' ? 'dark' : 'light');
};

// Layout State
export interface LayoutState extends Record<string, string | undefined> {
  splitRatio: string;
  splitOrientation: 'horizontal' | 'vertical';
}

export const layoutStore = persistentMap<LayoutState>('layout:', {
  splitRatio: '0.5',
  splitOrientation: 'vertical'
});

export const setSplitRatio = (ratio: number) => {
  layoutStore.setKey('splitRatio', Math.max(0.1, Math.min(0.9, ratio)).toString());
};

export const getSplitRatio = () => {
  return parseFloat(layoutStore.get().splitRatio);
};

export const setSplitOrientation = (orientation: 'horizontal' | 'vertical') => {
  layoutStore.setKey('splitOrientation', orientation);
};

export const toggleSplitOrientation = () => {
  const current = layoutStore.get().splitOrientation;
  layoutStore.setKey('splitOrientation', current === 'horizontal' ? 'vertical' : 'horizontal');
};
