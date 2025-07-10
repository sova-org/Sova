import React, { useEffect } from 'react';
import { useColorContext } from '../context/ColorContext';

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

  return (
    <div className="p-4 h-full overflow-y-auto" style={{ backgroundColor: 'var(--color-surface)' }}>
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-lg font-semibold" style={{ color: 'var(--color-text)' }}>
          Material Palette
        </h2>
        <div className="flex items-center space-x-2">
          <button
            onClick={toggleTheme}
            className="px-3 py-1 text-sm rounded-md border transition-colors hover:opacity-80"
            style={{ 
              borderColor: 'var(--color-border)', 
              backgroundColor: 'var(--color-surface)', 
              color: 'var(--color-text)' 
            }}
          >
            {themeMode === 'light' ? '🌙' : '☀️'} {themeMode}
          </button>
          <button
            onClick={regenerateColors}
            className="px-3 py-1 text-sm rounded-md border transition-colors hover:opacity-80"
            style={{ 
              borderColor: 'var(--color-border)', 
              backgroundColor: 'var(--color-primary)', 
              color: 'white' 
            }}
          >
            🎨 Regenerate
          </button>
        </div>
      </div>
      
      {/* Color Controls */}
      <div className="space-y-4 mb-6">
        <div className="space-y-2">
          <label className="text-sm font-medium" style={{ color: 'var(--color-muted)' }}>
            Hue Rotation: {hueRotation}°
          </label>
          <input
            type="range"
            min="0"
            max="360"
            value={hueRotation}
            onChange={(e) => setHueRotation(Number(e.target.value))}
            className="w-full h-2 rounded-lg appearance-none cursor-pointer"
            style={{ backgroundColor: 'var(--color-border)' }}
          />
        </div>
        
        <div className="space-y-2">
          <label className="text-sm font-medium" style={{ color: 'var(--color-muted)' }}>
            Saturation: {saturation}% {saturation < 30 ? '(pastel)' : saturation > 80 ? '(vibrant)' : '(balanced)'}
          </label>
          <input
            type="range"
            min="10"
            max="100"
            value={saturation}
            onChange={(e) => setSaturation(Number(e.target.value))}
            className="w-full h-2 rounded-lg appearance-none cursor-pointer"
            style={{ backgroundColor: 'var(--color-border)' }}
          />
        </div>
        
        <div className="space-y-2">
          <label className="text-sm font-medium" style={{ color: 'var(--color-muted)' }}>
            Brightness: {brightness}% {brightness < 30 ? '(dark)' : brightness > 70 ? '(bright)' : '(medium)'}
          </label>
          <input
            type="range"
            min="0"
            max="100"
            value={brightness}
            onChange={(e) => setBrightness(Number(e.target.value))}
            className="w-full h-2 rounded-lg appearance-none cursor-pointer"
            style={{ backgroundColor: 'var(--color-border)' }}
          />
        </div>
        
        <div className="text-xs" style={{ color: 'var(--color-muted)' }}>
          Base: Primary {basePrimary}° → Secondary {baseSecondary}°
        </div>
      </div>

      {/* Primary Shades */}
      <div className="mb-6">
        <h3 className="text-sm font-medium mb-3 flex items-center" style={{ color: 'var(--color-muted)' }}>
          <div className="w-4 h-4 rounded-full mr-2" style={{ backgroundColor: palette.primary }} />
          Primary ({(basePrimary + hueRotation) % 360}°)
        </h3>
        <div className="grid grid-cols-5 gap-2">
          {Object.entries(primaryShades).map(([shade, color]) => {
            const isMainColor = shade === '500';
            return (
              <div key={`primary-${shade}`} className="space-y-1">
                <div
                  className={`h-12 rounded-md border cursor-pointer hover:scale-105 transition-transform ${
                    isMainColor ? 'ring-2 ring-offset-2' : ''
                  }`}
                  style={{ 
                    backgroundColor: color, 
                    borderColor: 'var(--color-border)',
                    ringColor: isMainColor ? palette.primary : 'transparent'
                  }}
                  onClick={() => navigator.clipboard.writeText(color)}
                  title={`Click to copy ${color}`}
                />
                <div className="text-xs text-center">
                  <div className={`font-mono ${isMainColor ? 'font-bold' : ''}`} style={{ color: 'var(--color-muted)' }}>
                    {shade}{isMainColor ? ' ★' : ''}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* Secondary Shades */}
      <div className="mb-6">
        <h3 className="text-sm font-medium mb-3 flex items-center" style={{ color: 'var(--color-muted)' }}>
          <div className="w-4 h-4 rounded-full mr-2" style={{ backgroundColor: palette.secondary }} />
          Secondary ({(baseSecondary + hueRotation) % 360}°)
        </h3>
        <div className="grid grid-cols-5 gap-2">
          {Object.entries(secondaryShades).map(([shade, color]) => {
            const isMainColor = shade === '500';
            return (
              <div key={`secondary-${shade}`} className="space-y-1">
                <div
                  className={`h-12 rounded-md border cursor-pointer hover:scale-105 transition-transform ${
                    isMainColor ? 'ring-2 ring-offset-2' : ''
                  }`}
                  style={{ 
                    backgroundColor: color, 
                    borderColor: 'var(--color-border)',
                    ringColor: isMainColor ? palette.secondary : 'transparent'
                  }}
                  onClick={() => navigator.clipboard.writeText(color)}
                  title={`Click to copy ${color}`}
                />
                <div className="text-xs text-center">
                  <div className={`font-mono ${isMainColor ? 'font-bold' : ''}`} style={{ color: 'var(--color-muted)' }}>
                    {shade}{isMainColor ? ' ★' : ''}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>

      {/* UI Colors */}
      <div className="mb-6">
        <h3 className="text-sm font-medium mb-3" style={{ color: 'var(--color-muted)' }}>
          🎨 UI Colors ({themeMode} theme)
        </h3>
        <div className="grid grid-cols-3 gap-3">
          {Object.entries(uiColors).map(([name, color]) => (
            <div key={name} className="space-y-2">
              <div
                className="h-16 rounded-lg border cursor-pointer hover:scale-105 transition-transform shadow-sm"
                style={{ backgroundColor: color, borderColor: 'var(--color-border)' }}
                onClick={() => navigator.clipboard.writeText(color)}
                title={`Click to copy ${color}`}
              />
              <div className="text-xs text-center">
                <div className="font-mono text-xs" style={{ color: 'var(--color-muted)' }}>{color}</div>
                <div className="capitalize font-medium mt-1" style={{ color: 'var(--color-text)' }}>{name}</div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Semantic Colors */}
      <div>
        <h3 className="text-sm font-medium mb-3" style={{ color: 'var(--color-muted)' }}>
          ⚡ Semantic Colors (culturally consistent)
        </h3>
        <div className="grid grid-cols-2 gap-3">
          {Object.entries(semanticColors).map(([name, color]) => (
            <div key={name} className="space-y-2">
              <div
                className="h-16 rounded-lg border cursor-pointer hover:scale-105 transition-transform shadow-sm"
                style={{ backgroundColor: color, borderColor: 'var(--color-border)' }}
                onClick={() => navigator.clipboard.writeText(color)}
                title={`Click to copy ${color}`}
              />
              <div className="text-xs text-center">
                <div className="font-mono text-xs" style={{ color: 'var(--color-muted)' }}>{color}</div>
                <div className="capitalize font-medium mt-1 flex items-center justify-center" style={{ color: 'var(--color-text)' }}>
                  {name === 'success' && '✅'} 
                  {name === 'error' && '❌'} 
                  {name === 'warning' && '⚠️'} 
                  {name === 'info' && 'ℹ️'} 
                  {name}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};