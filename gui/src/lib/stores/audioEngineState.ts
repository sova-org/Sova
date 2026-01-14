import { writable } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { SERVER_EVENTS } from "$lib/events";

export interface AudioEngineStatus {
  running: boolean;
  device: string | null;
}

export const audioEngineRunning = writable<boolean>(false);
export const audioEngineDevice = writable<string | null>(null);

let unlisten: UnlistenFn | null = null;

export async function initializeAudioEngineStore(): Promise<void> {
  if (unlisten) {
    unlisten();
  }

  unlisten = await listen<AudioEngineStatus>(
    SERVER_EVENTS.AUDIO_ENGINE_STATUS,
    (event) => {
      audioEngineRunning.set(event.payload.running);
      audioEngineDevice.set(event.payload.device);
    },
  );
}

export function setAudioEngineStatus(status: AudioEngineStatus): void {
  audioEngineRunning.set(status.running);
  audioEngineDevice.set(status.device);
}

export function cleanupAudioEngineStore(): void {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  audioEngineRunning.set(false);
  audioEngineDevice.set(null);
}
