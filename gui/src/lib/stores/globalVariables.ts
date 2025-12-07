import { writable, type Writable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { SERVER_EVENTS } from "$lib/events";
import type { VariableValue } from "$lib/types/protocol";
import { ListenerGroup } from "./helpers";

// Global variables (A-Z single-letter variables)
export const globalVariables: Writable<Record<string, VariableValue>> =
  writable({});

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
