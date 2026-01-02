import { writable } from "svelte/store";

const STORAGE_KEY = "sova-timeline-ui";

interface TimelineUIState {
  lineWidthMultipliers: Record<number, number>;
}

function loadState(): TimelineUIState {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) return JSON.parse(stored);
  } catch {
    // Invalid stored state
  }
  return { lineWidthMultipliers: {} };
}

function saveState(state: TimelineUIState): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // Storage unavailable
  }
}

const initial = loadState();
const { subscribe, update } = writable<TimelineUIState>(initial);

export const timelineUI = {
  subscribe,
  setLineWidth(lineIdx: number, multiplier: number): void {
    update((state) => {
      const newState = {
        ...state,
        lineWidthMultipliers: {
          ...state.lineWidthMultipliers,
          [lineIdx]: multiplier,
        },
      };
      saveState(newState);
      return newState;
    });
  },
};
