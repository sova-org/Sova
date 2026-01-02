/**
 * Rotates the hue of a hex color by the specified degrees.
 * Converts hex to HSL, shifts the hue, and converts back to hex.
 */
function rotateHue(hex: string, degrees: number): string {
  if (degrees === 0) return hex;

  const color = hex.replace("#", "");
  const r = parseInt(color.substring(0, 2), 16) / 255;
  const g = parseInt(color.substring(2, 4), 16) / 255;
  const b = parseInt(color.substring(4, 6), 16) / 255;

  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const l = (max + min) / 2;

  if (max === min) {
    return hex;
  }

  const d = max - min;
  const s = l > 0.5 ? d / (2 - max - min) : d / (max + min);

  let h: number;
  if (max === r) {
    h = ((g - b) / d + (g < b ? 6 : 0)) / 6;
  } else if (max === g) {
    h = ((b - r) / d + 2) / 6;
  } else {
    h = ((r - g) / d + 4) / 6;
  }

  h = (h + degrees / 360) % 1;
  if (h < 0) h += 1;

  const hue2rgb = (p: number, q: number, t: number): number => {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1 / 6) return p + (q - p) * 6 * t;
    if (t < 1 / 2) return q;
    if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
    return p;
  };

  const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
  const p = 2 * l - q;

  const newR = Math.round(hue2rgb(p, q, h + 1 / 3) * 255);
  const newG = Math.round(hue2rgb(p, q, h) * 255);
  const newB = Math.round(hue2rgb(p, q, h - 1 / 3) * 255);

  return `#${newR
    .toString(16)
    .padStart(
      2,
      "0",
    )}${newG.toString(16).padStart(2, "0")}${newB.toString(16).padStart(2, "0")}`;
}

/**
 * Recursively transforms all hex color strings in an object by rotating their hue.
 */
export function transformThemeColors<T>(obj: T, hue: number): T {
  if (hue === 0) return obj;
  if (typeof obj === "string" && obj.startsWith("#")) {
    return rotateHue(obj, hue) as T;
  }
  if (typeof obj === "object" && obj !== null) {
    const result: Record<string, unknown> = {};
    for (const key in obj) {
      result[key] = transformThemeColors(
        (obj as Record<string, unknown>)[key],
        hue,
      );
    }
    return result as T;
  }
  return obj;
}
