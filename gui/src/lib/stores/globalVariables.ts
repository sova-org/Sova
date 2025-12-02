import { writable, derived, type Writable, type Readable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { SERVER_EVENTS } from "$lib/events";
import type { VariableValue } from "$lib/types/protocol";
import { ListenerGroup } from "./helpers";

// Global variables (A-Z single-letter variables)
export const globalVariables: Writable<Record<string, VariableValue>> =
  writable({});

// Helper to get a specific variable
export function getGlobalVariable(
  name: string,
): Readable<VariableValue | null> {
  return derived(globalVariables, ($vars) => $vars[name] ?? null);
}

// Helper to get all variable names
export const globalVariableNames: Readable<string[]> = derived(
  globalVariables,
  ($vars) => Object.keys($vars).sort(),
);

const listeners = new ListenerGroup();

export async function initializeGlobalVariablesStore(): Promise<void> {
  await listeners.add(() =>
    listen<Record<string, VariableValue>>(
      SERVER_EVENTS.GLOBAL_VARIABLES,
      (event) => {
        globalVariables.set(event.payload);
      },
    ),
  );
}

export function cleanupGlobalVariablesStore(): void {
  listeners.cleanup();
  globalVariables.set({});
}
