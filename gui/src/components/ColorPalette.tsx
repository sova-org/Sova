import React, { useEffect } from 'react';
import { useColorContext } from '../context/ColorContext';
import { HarmonyType } from '../hooks/useColorPalette';

export const ColorPalette: React.FC = () => {
  const {
    hue,
    saturation,
    lightness,
    harmony,
    palette,
    setHue,
    setSaturation,
    setLightness,
    setHarmony,
    updateCSS,
  } = useColorContext();

  useEffect(() => {
    updateCSS();
  }, [updateCSS]);

  return (
    <div className="p-4 h-full" style={{ backgroundColor: 'var(--color-surface)' }}>
      <h2 className="text-lg font-semibold mb-4" style={{ color: 'var(--color-text)' }}>Color Palette</h2>
      
      {/* Controls */}
      <div className="space-y-4 mb-6">
        <div className="space-y-2">
          <label className="text-sm font-medium" style={{ color: 'var(--color-muted)' }}>
            Hue: {hue}°
          </label>
          <input
            type="range"
            min="0"
            max="360"
            value={hue}
            onChange={(e) => setHue(Number(e.target.value))}
            className="w-full h-2 rounded-lg appearance-none cursor-pointer"
            style={{ backgroundColor: 'var(--color-border)' }}
          />
        </div>
        
        <div className="space-y-2">
          <label className="text-sm font-medium" style={{ color: 'var(--color-muted)' }}>
            Saturation: {saturation}%
          </label>
          <input
            type="range"
            min="20"
            max="100"
            value={saturation}
            onChange={(e) => setSaturation(Number(e.target.value))}
            className="w-full h-2 rounded-lg appearance-none cursor-pointer"
            style={{ backgroundColor: 'var(--color-border)' }}
          />
        </div>
        
        <div className="space-y-2">
          <label className="text-sm font-medium" style={{ color: 'var(--color-muted)' }}>
            Lightness: {lightness}%
          </label>
          <input
            type="range"
            min="30"
            max="80"
            value={lightness}
            onChange={(e) => setLightness(Number(e.target.value))}
            className="w-full h-2 rounded-lg appearance-none cursor-pointer"
            style={{ backgroundColor: 'var(--color-border)' }}
          />
        </div>
        
        <div className="space-y-2">
          <label className="text-sm font-medium" style={{ color: 'var(--color-muted)' }}>Harmony</label>
          <select
            value={harmony}
            onChange={(e) => setHarmony(e.target.value as HarmonyType)}
            className="w-full p-2 border rounded-md text-sm"
            style={{ borderColor: 'var(--color-border)', backgroundColor: 'var(--color-background)', color: 'var(--color-text)' }}
          >
            <option value="analogous">Analogous</option>
            <option value="complementary">Complementary</option>
            <option value="triadic">Triadic</option>
            <option value="monochromatic">Monochromatic</option>
          </select>
        </div>
      </div>
      
      {/* Palette Display */}
      <div className="space-y-2">
        <h3 className="text-sm font-medium mb-3" style={{ color: 'var(--color-muted)' }}>Current Palette</h3>
        <div className="grid grid-cols-4 gap-2">
          {Object.entries(palette).map(([name, color]) => (
            <div key={name} className="space-y-1">
              <div
                className="h-12 rounded-md border cursor-pointer hover:scale-105 transition-transform"
                style={{ backgroundColor: color, borderColor: 'var(--color-border)' }}
                onClick={() => navigator.clipboard.writeText(color)}
                title={`Click to copy ${color}`}
              />
              <div className="text-xs text-center">
                <div className="font-mono" style={{ color: 'var(--color-muted)' }}>{color}</div>
                <div className="capitalize" style={{ color: 'var(--color-muted)', opacity: 0.8 }}>{name}</div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};