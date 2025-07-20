import { persistentAtom } from '@nanostores/persistent';
import type { ThemeMode } from '../hooks/useMaterialPalette';
import { updateStore } from '../utils/store-helpers';

export interface ColorPaletteSettings {
  hueRotation: number;
  themeMode: ThemeMode;
  saturation: number;
  brightness: number;
  basePrimary: number;
  baseSecondary: number;
}

const defaultSettings: ColorPaletteSettings = {
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
  defaultSettings,
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