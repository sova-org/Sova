import { writable } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const isConnected = writable<boolean>(false);
export const connectionError = writable<string | null>(null);

let unlisten: UnlistenFn | null = null;

export async function initializeConnectionListener(): Promise<void> {
  unlisten = await listen<{ reason: string }>(
    "client-disconnected",
    (event) => {
      isConnected.set(false);
      connectionError.set(`Disconnected: ${event.payload.reason}`);
    },
  );
}

export function cleanupConnectionListener(): void {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
}
