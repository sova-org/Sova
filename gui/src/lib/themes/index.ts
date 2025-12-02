export type { Theme, FontOption } from "./types";
export { fontFamilies, combineWithSystemFonts } from "./types";
export { createHighlightStyle } from "./utils";

import type { Theme } from "./types";
import { themes as allThemes } from "./definitions";

export const themes: Record<string, Theme> = allThemes;

export type ThemeName = keyof typeof themes;
