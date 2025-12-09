import { writable, derived, get } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { scene } from "./scene";
import { SERVER_EVENTS } from "$lib/events";
import type { Frame, RemoveFramePayload } from "$lib/types/protocol";

export interface EditingFrameState {
  lineIdx: number;
  frameIdx: number;
}

const { subscribe, set, update } = writable<EditingFrameState | null>(null);

export const editingFrame = { subscribe };

export const currentEditingFrame = derived(
  [{ subscribe }, scene],
  ([$editingFrame, $scene]): Frame | null => {
    if (!$editingFrame || !$scene) return null;
    const line = $scene.lines[$editingFrame.lineIdx];
    if (!line) return null;
    return line.frames[$editingFrame.frameIdx] ?? null;
  }
);

export const editingFrameKey = derived(
  { subscribe },
  ($editingFrame): string | null =>
    $editingFrame
      ? `${$editingFrame.lineIdx}-${$editingFrame.frameIdx}`
      : null
);

export function openEditor(lineIdx: number, frameIdx: number): void {
  set({ lineIdx, frameIdx });
}

export function closeEditor(): void {
  set(null);
}

let unlistenFns: UnlistenFn[] = [];
let initialized = false;

export async function initEditingFrameListeners(): Promise<void> {
  if (initialized) return;
  initialized = true;

  unlistenFns.push(
    await listen<RemoveFramePayload>(SERVER_EVENTS.REMOVE_FRAME, (event) => {
      const current = get({ subscribe });
      if (!current) return;
      const { lineId, frameId } = event.payload;
      if (current.lineIdx === lineId) {
        if (current.frameIdx === frameId) {
          set(null);
        } else if (current.frameIdx > frameId) {
          set({ lineIdx: lineId, frameIdx: current.frameIdx - 1 });
        }
      }
    }),
    await listen<number>(SERVER_EVENTS.REMOVE_LINE, (event) => {
      const current = get({ subscribe });
      if (!current) return;
      const removedLineId = event.payload;
      if (current.lineIdx === removedLineId) {
        set(null);
      } else if (current.lineIdx > removedLineId) {
        set({ ...current, lineIdx: current.lineIdx - 1 });
      }
    })
  );

  window.addEventListener("project:loaded", handleProjectLoaded);
}

export function cleanupEditingFrameListeners(): void {
  if (!initialized) return;
  unlistenFns.forEach((fn) => fn());
  unlistenFns = [];
  window.removeEventListener("project:loaded", handleProjectLoaded);
  initialized = false;
}

function handleProjectLoaded(): void {
  set(null);
}
