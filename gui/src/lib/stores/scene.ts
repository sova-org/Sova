import { writable, derived, type Writable, type Readable } from "svelte/store";
import { SERVER_EVENTS } from "$lib/events";
import type {
  Scene,
  Line,
  Frame,
  AddLinePayload,
  AddFramePayload,
  RemoveFramePayload,
} from "$lib/types/protocol";
import {
  ListenerGroup,
  createSetListener,
  createUpdateListener,
  updateLinesInScene,
  addLineToScene,
  removeLineFromScene,
  updateFramesInScene,
  addFrameToScene,
  removeFrameFromScene,
} from "./helpers";

// Main scene store
export const scene: Writable<Scene | null> = writable(null);

// Derived stores for convenient access
export const lines: Readable<Line[]> = derived(
  scene,
  ($scene) => $scene?.lines ?? [],
);

const listeners = new ListenerGroup();

export async function initializeSceneStore(): Promise<void> {
  // Full scene updates
  await listeners.add(createSetListener(SERVER_EVENTS.SCENE, scene));

  // Line updates with pure functions
  await listeners.add(
    createUpdateListener(SERVER_EVENTS.LINE_VALUES, scene, updateLinesInScene),
  );

  await listeners.add(
    createUpdateListener(
      SERVER_EVENTS.LINE_CONFIGURATIONS,
      scene,
      (scene, updates: [number, Line][]) => {
        if (!scene) return scene;
        const newScene = { ...scene, lines: [...scene.lines] };
        for (const [idx, line] of updates) {
          // Merge configuration while preserving frames
          const existingFrames = newScene.lines[idx]?.frames ?? [];
          newScene.lines[idx] = { ...line, frames: existingFrames };
        }
        return newScene;
      },
    ),
  );

  await listeners.add(
    createUpdateListener(
      SERVER_EVENTS.ADD_LINE,
      scene,
      (scene, payload: AddLinePayload) =>
        addLineToScene(scene, payload.index, payload.line),
    ),
  );

  await listeners.add(
    createUpdateListener(SERVER_EVENTS.REMOVE_LINE, scene, removeLineFromScene),
  );

  // Frame updates
  await listeners.add(
    createUpdateListener(
      SERVER_EVENTS.FRAME_VALUES,
      scene,
      updateFramesInScene,
    ),
  );

  await listeners.add(
    createUpdateListener(
      SERVER_EVENTS.ADD_FRAME,
      scene,
      (scene, payload: AddFramePayload) =>
        addFrameToScene(scene, payload.lineId, payload.frameId, payload.frame),
    ),
  );

  await listeners.add(
    createUpdateListener(
      SERVER_EVENTS.REMOVE_FRAME,
      scene,
      (scene, payload: RemoveFramePayload) =>
        removeFrameFromScene(scene, payload.lineId, payload.frameId),
    ),
  );
}

export function cleanupSceneStore(): void {
  listeners.cleanup();
  scene.set(null);
}

// Export pure functions for testing
export {
  updateLinesInScene,
  addLineToScene,
  removeLineFromScene,
  updateFramesInScene,
  addFrameToScene,
  removeFrameFromScene,
};
