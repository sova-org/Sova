import { writable, derived, type Writable, type Readable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import type { LinkState, ClockState, FramePosition, PlaybackState } from "$lib/types/protocol";
import { ListenerGroup } from "./helpers";
import { SERVER_EVENTS } from "$lib/events";

// Transport state
export const playbackState: Writable<PlaybackState> = writable("Stopped");

// Derived: is transport playing (for backward compatibility)
export const isPlaying: Readable<boolean> = derived(
  playbackState,
  ($state) => $state === "Playing",
);

// Derived: is transport starting (waiting for beat)
export const isStarting: Readable<boolean> = derived(
  playbackState,
  ($state) => typeof $state === "object" && "Starting" in $state,
);

// Clock state
export const clockState: Writable<ClockState | null> = writable(null);

// Link state (Ableton Link)
export const linkState: Writable<LinkState | null> = writable(null);

// Frame positions (line_idx, frame_idx)
export const framePositions: Writable<FramePosition[]> = writable([]);

const listeners = new ListenerGroup();

export async function initializeTransportStore(): Promise<void> {
  // Listen for playback state changes
  await listeners.add(() =>
    listen<PlaybackState>(SERVER_EVENTS.PLAYBACK_STATE_CHANGED, (event) => {
      playbackState.set(event.payload);
    }),
  );

  // Listen for clock state updates
  await listeners.add(() =>
    listen<ClockState>(SERVER_EVENTS.CLOCK_STATE, (event) => {
      clockState.set(event.payload);
    }),
  );

  // Listen for frame position updates
  await listeners.add(() =>
    listen<FramePosition[]>(SERVER_EVENTS.FRAME_POSITION, (event) => {
      framePositions.set(event.payload);
    }),
  );
}

export function cleanupTransportStore(): void {
  listeners.cleanup();
  playbackState.set("Stopped");
  clockState.set(null);
  linkState.set(null);
  framePositions.set([]);
}
