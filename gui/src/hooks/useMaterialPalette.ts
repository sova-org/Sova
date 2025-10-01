import { useCallback, useMemo } from 'react';
import { useStore } from '@nanostores/react';
import { 
  $colorPaletteSettings, 
  updateHueRotation, 
  updateThemeMode, 
  updateSaturation, 
  updateBrightness, 
  regenerateBaseColors, 
  toggleTheme 
} from '../stores/ui/preferences';

export interface MaterialPalette {
  // Primary shades
  primary50: string;
  primary100: string;
  primary200: string;
  primary300: string;
  primary400: string;
  primary500: string;  // Main primary
  primary600: string;
  primary700: string;
  primary800: string;
  primary900: string;
  
  // Secondary shades
  secondary50: string;
  secondary100: string;
  secondary200: string;
  secondary300: string;
  secondary400: string;
  secondary500: string;  // Main secondary
  secondary600: string;
  secondary700: string;
  secondary800: string;
  secondary900: string;
  
  // UI colors derived from primary/secondary
  primary: string;      // primary500
  secondary: string;    // secondary500
  background: string;   // primary50
  surface: string;      // primary100
  text: string;         // primary900
  muted: string;        // primary400
  border: string;       // primary200
  
  // Semantic colors (theme-independent)
  success: string;      // Green for positive actions
  error: string;        // Red for negative actions
  warning: string;      // Orange for caution
  info: string;         // Blue for information
}

export type ThemeMode = 'light' | 'dark';

export function useMaterialPalette() {
  const settings = useStore($colorPaletteSettings);
  const { hueRotation, themeMode, saturation, brightness, basePrimary, baseSecondary } = settings;

  const hslToHex = useCallback((h: number, s: number, l: number): string => {
    h = h / 360;
    s = s / 100;
    l = l / 100;
    
    const hue2rgb = (p: number, q: number, t: number) => {
      if (t < 0) t += 1;
      if (t > 1) t -= 1;
      if (t < 1/6) return p + (q - p) * 6 * t;
      if (t < 1/2) return q;
      if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
      return p;
    };
    
    let r, g, b;
    
    if (s === 0) {
      r = g = b = l;
    } else {
      const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
      const p = 2 * l - q;
      r = hue2rgb(p, q, h + 1/3);
      g = hue2rgb(p, q, h);
      b = hue2rgb(p, q, h - 1/3);
    }
    
    const toHex = (c: number) => {
      const hex = Math.round(c * 255).toString(16);
      return hex.length === 1 ? '0' + hex : hex;
    };
    
    return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
  }, []);

  // Enhanced Material Design shade generation with user controls
  const generateShades = useCallback((baseHue: number) => {
    // Adjust saturation based on hue for better colors
    const satMultiplier = baseHue >= 45 && baseHue <= 75 ? 0.8 : 1.0; // Reduce yellow saturation
    const baseSat = saturation * satMultiplier;
    
    // Brightness adjustment: -50 to +50 lightness shift
    const brightnessShift = (brightness - 50) * 0.3; // Scale to -15 to +15
    
    return {
      50: hslToHex(baseHue, Math.max(5, baseSat * 0.2), Math.max(5, Math.min(98, 96 + brightnessShift))),
      100: hslToHex(baseHue, Math.max(10, baseSat * 0.3), Math.max(10, Math.min(95, 92 + brightnessShift))),
      200: hslToHex(baseHue, Math.max(15, baseSat * 0.4), Math.max(15, Math.min(90, 83 + brightnessShift))),
      300: hslToHex(baseHue, Math.max(20, baseSat * 0.5), Math.max(20, Math.min(85, 74 + brightnessShift))),
      400: hslToHex(baseHue, Math.max(25, baseSat * 0.6), Math.max(25, Math.min(80, 64 + brightnessShift))),
      500: hslToHex(baseHue, baseSat, Math.max(30, Math.min(70, 50 + brightnessShift))),  // Main color
      600: hslToHex(baseHue, Math.min(100, baseSat * 1.1), Math.max(25, Math.min(65, 42 + brightnessShift))),
      700: hslToHex(baseHue, Math.min(100, baseSat * 1.0), Math.max(20, Math.min(60, 33 + brightnessShift))),
      800: hslToHex(baseHue, Math.min(100, baseSat * 0.9), Math.max(15, Math.min(55, 24 + brightnessShift))),
      900: hslToHex(baseHue, Math.min(100, baseSat * 0.8), Math.max(10, Math.min(50, 15 + brightnessShift))),
    };
  }, [hslToHex, saturation, brightness]);

  const generatePalette = useCallback((): MaterialPalette => {
    // Apply hue rotation to both base colors
    const primaryHue = (basePrimary + hueRotation) % 360;
    const secondaryHue = (baseSecondary + hueRotation) % 360;
    
    const primaryShades = generateShades(primaryHue);
    const secondaryShades = generateShades(secondaryHue);

    // Theme-aware color mappings
    const isLight = themeMode === 'light';
    
    return {
      // Primary shades
      primary50: primaryShades[50],
      primary100: primaryShades[100],
      primary200: primaryShades[200],
      primary300: primaryShades[300],
      primary400: primaryShades[400],
      primary500: primaryShades[500],
      primary600: primaryShades[600],
      primary700: primaryShades[700],
      primary800: primaryShades[800],
      primary900: primaryShades[900],
      
      // Secondary shades
      secondary50: secondaryShades[50],
      secondary100: secondaryShades[100],
      secondary200: secondaryShades[200],
      secondary300: secondaryShades[300],
      secondary400: secondaryShades[400],
      secondary500: secondaryShades[500],
      secondary600: secondaryShades[600],
      secondary700: secondaryShades[700],
      secondary800: secondaryShades[800],
      secondary900: secondaryShades[900],
      
      // Theme-aware UI colors
      primary: primaryShades[500],
      secondary: secondaryShades[500],
      background: isLight ? primaryShades[50] : primaryShades[900],
      surface: isLight ? primaryShades[100] : primaryShades[800],
      text: isLight ? primaryShades[900] : primaryShades[50],
      muted: isLight ? primaryShades[400] : primaryShades[500],
      border: isLight ? primaryShades[200] : primaryShades[700],
      
      // Semantic colors (consistent across themes)
      success: isLight ? '#16a34a' : '#22c55e',    // Green-600 / Green-500
      error: isLight ? '#dc2626' : '#ef4444',      // Red-600 / Red-500
      warning: isLight ? '#ea580c' : '#f97316',    // Orange-600 / Orange-500
      info: isLight ? '#2563eb' : '#3b82f6',       // Blue-600 / Blue-500
    };
  }, [basePrimary, baseSecondary, hueRotation, themeMode, saturation, brightness, generateShades]);

  const palette = useMemo(() => generatePalette(), [generatePalette]);

  const updateCSS = useCallback(() => {
    const root = document.documentElement;
    
    // Set all shade variables
    Object.entries(palette).forEach(([key, value]) => {
      root.style.setProperty(`--color-${key.replace(/([0-9]+)$/, '-$1')}`, value);
    });
    
    // Set main color variables for backwards compatibility
    root.style.setProperty('--color-primary', palette.primary);
    root.style.setProperty('--color-secondary', palette.secondary);
    root.style.setProperty('--color-background', palette.background);
    root.style.setProperty('--color-surface', palette.surface);
    root.style.setProperty('--color-text', palette.text);
    root.style.setProperty('--color-muted', palette.muted);
    root.style.setProperty('--color-border', palette.border);
    
    // Set semantic colors
    root.style.setProperty('--color-success', palette.success);
    root.style.setProperty('--color-error', palette.error);
    root.style.setProperty('--color-warning', palette.warning);
    root.style.setProperty('--color-info', palette.info);
  }, [palette]);

  return {
    hueRotation,
    themeMode,
    saturation,
    brightness,
    palette,
    setHueRotation: updateHueRotation,
    setThemeMode: updateThemeMode,
    setSaturation: updateSaturation,
    setBrightness: updateBrightness,
    toggleTheme,
    updateCSS,
    regenerateColors: regenerateBaseColors,
    basePrimary,
    baseSecondary,
  };
}