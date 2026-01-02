import { writable, type Writable } from "svelte/store";
import { SERVER_EVENTS } from "$lib/events";
import type {
  Scene,
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

// DEBUG: Log all store updates
scene.subscribe(($scene) => {
  console.log('[SCENE STORE] Updated, lines:', $scene?.lines.length,
    'frames:', $scene?.lines.map(l => l.frames.length));
});

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
      (currentScene, payload: AddLinePayload) => {
        console.log('[ADD_LINE] Event received:', payload);
        const result = addLineToScene(currentScene, payload.index, payload.line);
        console.log('[ADD_LINE] Result:', result !== currentScene ? 'NEW SCENE' : 'UNCHANGED');
        return result;
      },
    ),
  );

  await listeners.add(
    createUpdateListener(
      SERVER_EVENTS.REMOVE_LINE,
      scene,
      (currentScene, lineIndex: number) => {
        console.log('[REMOVE_LINE] Event received:', lineIndex);
        const result = removeLineFromScene(currentScene, lineIndex);
        console.log('[REMOVE_LINE] Result:', result !== currentScene ? 'NEW SCENE' : 'UNCHANGED');
        return result;
      },
    ),
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
      (currentScene, payload: AddFramePayload) => {
        console.log('[ADD_FRAME] Event received:', payload);
        const result = addFrameToScene(currentScene, payload.lineId, payload.frameId, payload.frame);
        console.log('[ADD_FRAME] Result:', result !== currentScene ? 'NEW SCENE' : 'UNCHANGED');
        return result;
      },
    ),
  );

  await listeners.add(
    createUpdateListener(
      SERVER_EVENTS.REMOVE_FRAME,
      scene,
      (currentScene, payload: RemoveFramePayload) => {
        console.log('[REMOVE_FRAME] Event received:', payload);
        const result = removeFrameFromScene(currentScene, payload.lineId, payload.frameId);
        console.log('[REMOVE_FRAME] Result:', result !== currentScene ? 'NEW SCENE' : 'UNCHANGED');
        return result;
      },
    ),
  );
}

export function cleanupSceneStore(): void {
  listeners.cleanup();
  scene.set(null);
}
