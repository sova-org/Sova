import { useState, useCallback, useMemo } from 'react';

export interface ColorPalette {
  primary: string;
  secondary: string;
  accent: string;
  background: string;
  surface: string;
  text: string;
  muted: string;
  border: string;
}

export type HarmonyType = 'analogous' | 'complementary' | 'triadic' | 'monochromatic';

export function useColorPalette() {
  const [hue, setHue] = useState(220);
  const [saturation, setSaturation] = useState(70);
  const [lightness, setLightness] = useState(50);
  const [harmony, setHarmony] = useState<HarmonyType>('analogous');

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

  const generatePalette = useCallback((): ColorPalette => {
    const baseHue = hue;
    const baseSat = saturation;
    const baseLight = lightness;

    let colors: string[] = [];

    switch (harmony) {
      case 'analogous':
        colors = [
          hslToHex(baseHue, baseSat, baseLight),
          hslToHex((baseHue + 30) % 360, baseSat, baseLight + 10),
          hslToHex((baseHue + 60) % 360, baseSat, baseLight - 10),
          hslToHex(baseHue, baseSat - 20, baseLight + 30),
          hslToHex(baseHue, baseSat + 10, baseLight + 20),
          hslToHex(baseHue, baseSat, baseLight - 30),
          hslToHex(baseHue, baseSat - 30, baseLight + 40),
          hslToHex(baseHue, baseSat - 10, baseLight - 20),
        ];
        break;
      case 'complementary':
        const complement = (baseHue + 180) % 360;
        colors = [
          hslToHex(baseHue, baseSat, baseLight),
          hslToHex(complement, baseSat, baseLight),
          hslToHex(baseHue, baseSat - 20, baseLight + 20),
          hslToHex(baseHue, baseSat - 40, baseLight + 35),
          hslToHex(baseHue, baseSat + 10, baseLight + 15),
          hslToHex(baseHue, baseSat, baseLight - 25),
          hslToHex(baseHue, baseSat - 50, baseLight + 45),
          hslToHex(baseHue, baseSat - 15, baseLight - 15),
        ];
        break;
      case 'triadic':
        const tri1 = (baseHue + 120) % 360;
        const tri2 = (baseHue + 240) % 360;
        colors = [
          hslToHex(baseHue, baseSat, baseLight),
          hslToHex(tri1, baseSat, baseLight),
          hslToHex(tri2, baseSat, baseLight),
          hslToHex(baseHue, baseSat - 30, baseLight + 25),
          hslToHex(baseHue, baseSat + 10, baseLight + 15),
          hslToHex(baseHue, baseSat, baseLight - 25),
          hslToHex(baseHue, baseSat - 40, baseLight + 40),
          hslToHex(baseHue, baseSat - 10, baseLight - 15),
        ];
        break;
      case 'monochromatic':
        colors = [
          hslToHex(baseHue, baseSat, baseLight),
          hslToHex(baseHue, baseSat - 10, baseLight + 20),
          hslToHex(baseHue, baseSat + 10, baseLight - 15),
          hslToHex(baseHue, baseSat - 30, baseLight + 35),
          hslToHex(baseHue, baseSat + 5, baseLight + 10),
          hslToHex(baseHue, baseSat, baseLight - 30),
          hslToHex(baseHue, baseSat - 40, baseLight + 45),
          hslToHex(baseHue, baseSat - 5, baseLight - 20),
        ];
        break;
    }

    return {
      primary: colors[0],
      secondary: colors[1],
      accent: colors[2],
      background: colors[3],
      surface: colors[4],
      text: colors[5],
      muted: colors[6],
      border: colors[7],
    };
  }, [hue, saturation, lightness, harmony, hslToHex]);

  const palette = useMemo(() => generatePalette(), [generatePalette]);

  const updateCSS = useCallback(() => {
    const root = document.documentElement;
    Object.entries(palette).forEach(([key, value]) => {
      root.style.setProperty(`--color-${key}`, value);
    });
  }, [palette]);

  return {
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
  };
}