import React, { createContext, useContext, ReactNode } from 'react';
import { useMaterialPalette, MaterialPalette, ThemeMode } from '../hooks/useMaterialPalette';

interface ColorContextType {
  hueRotation: number;
  themeMode: ThemeMode;
  saturation: number;
  brightness: number;
  palette: MaterialPalette;
  setHueRotation: (value: number) => void;
  setThemeMode: (mode: ThemeMode) => void;
  setSaturation: (value: number) => void;
  setBrightness: (value: number) => void;
  toggleTheme: () => void;
  updateCSS: () => void;
  regenerateColors: () => void;
  basePrimary: number;
  baseSecondary: number;
}

const ColorContext = createContext<ColorContextType | undefined>(undefined);

export const useColorContext = () => {
  const context = useContext(ColorContext);
  if (!context) {
    throw new Error('useColorContext must be used within a ColorProvider');
  }
  return context;
};

interface ColorProviderProps {
  children: ReactNode;
}

export const ColorProvider: React.FC<ColorProviderProps> = ({ children }) => {
  const materialPalette = useMaterialPalette();
  
  return (
    <ColorContext.Provider value={materialPalette}>
      {children}
    </ColorContext.Provider>
  );
};