import React, { useEffect } from 'react';
import { useColorContext } from '../context/ColorContext';
import { Sun, Moon, RefreshCw } from 'lucide-react';

export const MaterialColorPalette: React.FC = () => {
  const {
    hueRotation,
    themeMode,
    saturation,
    brightness,
    palette,
    setHueRotation,
    setSaturation,
    setBrightness,
    toggleTheme,
    updateCSS,
    regenerateColors,
    basePrimary,
    baseSecondary,
  } = useColorContext();

  useEffect(() => {
    updateCSS();
  }, [updateCSS]);

  // Group shades for display
  const primaryShades = {
    50: palette.primary50,
    100: palette.primary100,
    200: palette.primary200,
    300: palette.primary300,
    400: palette.primary400,
    500: palette.primary500,
    600: palette.primary600,
    700: palette.primary700,
    800: palette.primary800,
    900: palette.primary900,
  };

  const secondaryShades = {
    50: palette.secondary50,
    100: palette.secondary100,
    200: palette.secondary200,
    300: palette.secondary300,
    400: palette.secondary400,
    500: palette.secondary500,
    600: palette.secondary600,
    700: palette.secondary700,
    800: palette.secondary800,
    900: palette.secondary900,
  };

  const uiColors = {
    primary: palette.primary,
    secondary: palette.secondary,
    background: palette.background,
    surface: palette.surface,
    text: palette.text,
    muted: palette.muted,
    border: palette.border,
  };

  const semanticColors = {
    success: palette.success,
    error: palette.error,
    warning: palette.warning,
    info: palette.info,
  };

  const allColors = [
    ...Object.values(primaryShades),
    ...Object.values(secondaryShades),
    ...Object.values(uiColors).filter(color => color !== palette.background),
    ...Object.values(semanticColors),
  ];

  return (
    <div className="p-4 h-full overflow-y-auto" style={{ backgroundColor: 'var(--color-surface)' }}>
      <div className="flex items-center justify-end mb-4 space-x-2">
        <button
          onClick={toggleTheme}
          className="px-3 py-1 text-sm border transition-colors hover:opacity-80 flex items-center space-x-1"
          style={{ 
            borderColor: 'var(--color-border)', 
            backgroundColor: 'var(--color-surface)', 
            color: 'var(--color-text)' 
          }}
          title={`Switch to ${themeMode === 'light' ? 'dark' : 'light'} mode`}
        >
          {themeMode === 'light' ? <Sun size={14} /> : <Moon size={14} />}
          <span>{themeMode}</span>
        </button>
        <button
          onClick={regenerateColors}
          className="px-3 py-1 text-sm border transition-colors hover:opacity-80 flex items-center space-x-1"
          style={{ 
            borderColor: 'var(--color-border)', 
            backgroundColor: 'var(--color-primary)', 
            color: 'white' 
          }}
          title="Regenerate color palette"
        >
          <RefreshCw size={14} />
          <span>Regenerate</span>
        </button>
      </div>
      
      {/* Color Controls */}
      <div className="space-y-3 mb-6">
        <input
          type="range"
          min="0"
          max="360"
          value={hueRotation}
          onChange={(e) => setHueRotation(Number(e.target.value))}
          className="w-full h-2 appearance-none cursor-pointer"
          style={{ backgroundColor: 'var(--color-border)' }}
        />
        <input
          type="range"
          min="10"
          max="100"
          value={saturation}
          onChange={(e) => setSaturation(Number(e.target.value))}
          className="w-full h-2 appearance-none cursor-pointer"
          style={{ backgroundColor: 'var(--color-border)' }}
        />
        <input
          type="range"
          min="0"
          max="100"
          value={brightness}
          onChange={(e) => setBrightness(Number(e.target.value))}
          className="w-full h-2 appearance-none cursor-pointer"
          style={{ backgroundColor: 'var(--color-border)' }}
        />
      </div>

      {/* Color Grid */}
      <div className="grid grid-cols-6 gap-2">
        {allColors.map((color, index) => (
          <div
            key={index}
            className="aspect-square cursor-pointer hover:scale-105 transition-transform"
            style={{ backgroundColor: color }}
            onClick={() => navigator.clipboard.writeText(color)}
          />
        ))}
      </div>
    </div>
  );
};