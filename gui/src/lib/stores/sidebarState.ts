import { writable, derived, get } from "svelte/store";
import { isConnected } from "./connectionState";

export type SidebarTab = "PROJECTS" | "DEVICES" | "CHAT" | "LOGS" | "CONFIG";
export type SidebarSide = "left" | "right";

interface SidebarState {
  isOpen: boolean;
  side: SidebarSide;
  width: number;
  activeTab: SidebarTab;
}

const STORAGE_KEY = "sova-sidebar-state";
const DEFAULT_WIDTH = 300;
const MIN_WIDTH = 200;
const MAX_WIDTH = 600;

function loadState(): SidebarState {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      return {
        isOpen: parsed.isOpen ?? false,
        side: parsed.side ?? "left",
        width: Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, parsed.width ?? DEFAULT_WIDTH)),
        activeTab: parsed.activeTab ?? "PROJECTS",
      };
    }
  } catch {
    // Ignore parse errors
  }
  return {
    isOpen: false,
    side: "left",
    width: DEFAULT_WIDTH,
    activeTab: "PROJECTS",
  };
}

function saveState(state: SidebarState): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
  } catch {
    // Ignore storage errors
  }
}

const { subscribe, set, update } = writable<SidebarState>(loadState());

subscribe(saveState);

export const sidebarState = {
  subscribe,

  toggle(): void {
    update((s) => ({ ...s, isOpen: !s.isOpen }));
  },

  open(): void {
    update((s) => ({ ...s, isOpen: true }));
  },

  close(): void {
    update((s) => ({ ...s, isOpen: false }));
  },

  setTab(tab: SidebarTab): void {
    update((s) => ({ ...s, activeTab: tab, isOpen: true }));
  },

  setSide(side: SidebarSide): void {
    update((s) => ({ ...s, side }));
  },

  toggleSide(): void {
    update((s) => ({ ...s, side: s.side === "left" ? "right" : "left" }));
  },

  setWidth(width: number): void {
    const clamped = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, width));
    update((s) => ({ ...s, width: clamped }));
  },
};

export const sidebarIsOpen = derived({ subscribe }, ($state) => $state.isOpen);
export const sidebarSide = derived({ subscribe }, ($state) => $state.side);
export const sidebarWidth = derived({ subscribe }, ($state) => $state.width);
export const sidebarActiveTab = derived({ subscribe }, ($state) => $state.activeTab);

export const availableSidebarTabs = derived(
  isConnected,
  ($connected): SidebarTab[] =>
    $connected
      ? ["PROJECTS", "DEVICES", "CHAT", "LOGS", "CONFIG"]
      : ["PROJECTS", "LOGS", "CONFIG"]
);

export const SIDEBAR_MIN_WIDTH = MIN_WIDTH;
export const SIDEBAR_MAX_WIDTH = MAX_WIDTH;
