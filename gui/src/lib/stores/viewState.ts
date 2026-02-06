import { writable, derived } from "svelte/store";
import { isConnected } from "./connectionState";

export type ViewType =
  | "LOGIN"
  | "SCENE"
  | "DEVICES"
  | "LOGS"
  | "CHAT"
  | "PROJECTS"
  | "CONFIG";

interface ViewState {
  currentView: ViewType;
  previousView: ViewType | null;
}

const DEFAULT_VIEW: ViewType = "LOGIN";

const { subscribe, update } = writable<ViewState>({
  currentView: DEFAULT_VIEW,
  previousView: null,
});

function createViewStore() {
  return {
    subscribe,

    navigateTo(view: ViewType): void {
      update((s) => ({
        currentView: view,
        previousView: s.currentView,
      }));
    },

    goBack(): void {
      update((s) => {
        if (s.previousView) {
          return {
            currentView: s.previousView,
            previousView: s.currentView,
          };
        }
        return s;
      });
    },

    handleDisconnect(): void {
      update((s) => {
        const requiresConnection: ViewType[] = ["SCENE", "DEVICES", "CHAT"];
        if (requiresConnection.includes(s.currentView)) {
          return { currentView: "LOGIN", previousView: null };
        }
        return s;
      });
    },
  };
}

export const viewState = createViewStore();

export const currentView = derived({ subscribe }, ($state) => $state.currentView);

export const availableViews = derived(
  isConnected,
  ($connected): ViewType[] =>
    $connected
      ? ["SCENE", "DEVICES", "CHAT", "PROJECTS", "LOGS", "CONFIG"]
      : ["PROJECTS", "LOGIN", "LOGS", "CONFIG"]
);

isConnected.subscribe(($connected) => {
  if (!$connected) {
    viewState.handleDisconnect();
  }
});
