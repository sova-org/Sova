import { writable, derived, type Writable, type Readable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { SERVER_EVENTS } from "$lib/events";
import type {
  CompilationState,
  CompilationUpdatePayload,
  RemoveFramePayload,
} from "$lib/types/protocol";
import { ListenerGroup } from "./helpers";

// Track latest scriptId per frame position
interface FrameCompilation {
  scriptId: string; // String to avoid JS precision loss for u64
  state: CompilationState;
}

// Compilation state per frame: Map<"lineId:frameId", FrameCompilation>
export const compilationStates: Writable<Map<string, FrameCompilation>> =
  writable(new Map());

// Pure helper to create key (without scriptId for simpler lookup)
function makeKey(lineId: number, frameId: number): string {
  return `${lineId}:${frameId}`;
}

const listeners = new ListenerGroup();

export async function initializeCompilationStore(): Promise<void> {
  await listeners.add(() =>
    listen<CompilationUpdatePayload>(
      SERVER_EVENTS.COMPILATION_UPDATE,
      (event) => {
        const { lineId, frameId, scriptId, state } = event.payload;
        compilationStates.update(($states) => {
          const key = makeKey(lineId, frameId);
          const newStates = new Map($states);
          newStates.set(key, { scriptId, state });
          return newStates;
        });
      },
    ),
  );

  // Clean up compilation state when frames are removed
  await listeners.add(() =>
    listen<RemoveFramePayload>(SERVER_EVENTS.REMOVE_FRAME, (event) => {
      const { lineId, frameId } = event.payload;
      compilationStates.update(($states) => {
        const newStates = new Map<string, FrameCompilation>();
        for (const [key, value] of $states.entries()) {
          const [lid, fid] = key.split(":").map(Number);
          if (lid === lineId) {
            if (fid < frameId) {
              newStates.set(key, value);
            } else if (fid > frameId) {
              // Shift index down
              newStates.set(makeKey(lid, fid - 1), value);
            }
            // fid === frameId: deleted, skip
          } else {
            newStates.set(key, value);
          }
        }
        return newStates;
      });
    }),
  );

  // Clean up compilation state when lines are removed
  await listeners.add(() =>
    listen<number>(SERVER_EVENTS.REMOVE_LINE, (event) => {
      const removedLineId = event.payload;
      compilationStates.update(($states) => {
        const newStates = new Map<string, FrameCompilation>();
        for (const [key, value] of $states.entries()) {
          const [lid, fid] = key.split(":").map(Number);
          if (lid < removedLineId) {
            newStates.set(key, value);
          } else if (lid > removedLineId) {
            // Shift line index down
            newStates.set(makeKey(lid - 1, fid), value);
          }
          // lid === removedLineId: deleted, skip
        }
        return newStates;
      });
    }),
  );
}

export function cleanupCompilationStore(): void {
  listeners.cleanup();
  compilationStates.set(new Map());
}
