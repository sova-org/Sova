import { writable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export const serverRunning = writable<boolean>(false);
export const serverError = writable<string | null>(null);

let unlisten: UnlistenFn | null = null;

export async function initializeServerStateListener(): Promise<void> {
  if (unlisten) {
    unlisten();
  }

  unlisten = await listen<number | null>("server:terminated", (event) => {
    serverRunning.set(false);
    if (event.payload !== 0 && event.payload !== null) {
      serverError.set(`Server exited with code ${event.payload}`);
    }
  });

  await syncServerStatus();
}

export async function syncServerStatus(): Promise<void> {
  const running = await invoke<boolean>("is_server_running");
  serverRunning.set(running);
}

export function cleanupServerStateListener(): void {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
}
