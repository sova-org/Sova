import { writable } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { SERVER_EVENTS } from "$lib/events";

export interface AudioEngineState {
  running: boolean;
  device: string | null;
  sample_rate: number;
  channels: number;
  active_voices: number;
  sample_paths: string[];
  error: string | null;
}

const defaultState: AudioEngineState = {
  running: false,
  device: null,
  sample_rate: 0,
  channels: 0,
  active_voices: 0,
  sample_paths: [],
  error: null,
};

export const audioEngineState = writable<AudioEngineState>(defaultState);

let unlisten: UnlistenFn | null = null;

export async function initializeAudioEngineStore(): Promise<void> {
  if (unlisten) {
    unlisten();
  }

  unlisten = await listen<AudioEngineState>(
    SERVER_EVENTS.AUDIO_ENGINE_STATE,
    (event) => {
      audioEngineState.set(event.payload);
    },
  );
}

export function setAudioEngineState(state: AudioEngineState): void {
  audioEngineState.set(state);
}

export function cleanupAudioEngineStore(): void {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  audioEngineState.set(defaultState);
}
