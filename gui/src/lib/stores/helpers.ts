// Helpers for reducing store boilerplate and improving testability

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Writable } from "svelte/store";

/**
 * Creates a simple event listener that sets store value
 * Pure factory function for testability
 */
export function createSetListener<T>(
  eventName: string,
  store: Writable<T>,
): () => Promise<UnlistenFn> {
  return async () => {
    return await listen<T>(eventName, (event) => {
      store.set(event.payload);
    });
  };
}

/**
 * Creates an event listener that updates store with custom handler
 * Handler function is pure and testable
 */
export function createUpdateListener<T, P>(
  eventName: string,
  store: Writable<T>,
  updateFn: (_current: T, _payload: P) => T,
): () => Promise<UnlistenFn> {
  return async () => {
    return await listen<P>(eventName, (event) => {
      store.update(($current) => updateFn($current, event.payload));
    });
  };
}

/**
 * Manages multiple listeners lifecycle
 */
export class ListenerGroup {
  private listeners: UnlistenFn[] = [];

  async add(listenerFactory: () => Promise<UnlistenFn>): Promise<void> {
    const unlisten = await listenerFactory();
    this.listeners.push(unlisten);
  }

  cleanup(): void {
    for (const unlisten of this.listeners) {
      unlisten();
    }
    this.listeners = [];
  }
}

// Pure update functions for scene operations (testable)

export function updateLinesInScene<S extends { lines: any[] }>(
  scene: S | null,
  updates: [number, any][],
): S | null {
  if (!scene || updates.length === 0) return scene;

  let newScene: S | null = null;
  for (const [idx, line] of updates) {
    if (idx >= 0 && idx < scene.lines.length) {
      if (!newScene) {
        newScene = { ...scene, lines: [...scene.lines] };
      }
      newScene.lines[idx] = line;
    }
  }
  return newScene ?? scene;
}

export function addLineToScene<S extends { lines: L[] }, L>(
  scene: S | null,
  index: number,
  line: L,
): S | null {
  if (!scene) {
    console.log('[addLineToScene] EARLY RETURN: scene is null');
    return scene;
  }
  const newScene = { ...scene, lines: [...scene.lines] };
  newScene.lines.splice(index, 0, line);
  return newScene;
}

export function removeLineFromScene<S extends { lines: any[] }>(
  scene: S | null,
  index: number,
): S | null {
  if (!scene) {
    console.log('[removeLineFromScene] EARLY RETURN: scene is null');
    return scene;
  }
  const newScene = { ...scene, lines: [...scene.lines] };
  newScene.lines.splice(index, 1);
  return newScene;
}

export function updateFramesInScene<
  S extends { lines: L[] },
  L extends { frames: F[] },
  F,
>(scene: S | null, updates: [number, number, F][]): S | null {
  if (!scene || updates.length === 0) return scene;

  const newScene = { ...scene, lines: [...scene.lines] };
  const copiedLines = new Set<number>();

  for (const [lineId, frameId, frame] of updates) {
    const line = newScene.lines[lineId];
    if (!line) continue;
    if (frameId < 0 || frameId >= line.frames.length) continue;

    if (!copiedLines.has(lineId)) {
      newScene.lines[lineId] = { ...line, frames: [...line.frames] };
      copiedLines.add(lineId);
    }
    newScene.lines[lineId].frames[frameId] = frame;
  }

  return newScene;
}

export function addFrameToScene<
  S extends { lines: L[] },
  L extends { frames: F[] },
  F,
>(scene: S | null, lineId: number, frameId: number, frame: F): S | null {
  if (!scene) {
    console.log('[addFrameToScene] EARLY RETURN: scene is null');
    return scene;
  }
  const line = scene.lines[lineId];
  if (!line) {
    console.log('[addFrameToScene] EARLY RETURN: line not found at', lineId, 'total lines:', scene.lines.length);
    return scene;
  }
  if (frameId < 0 || frameId > line.frames.length) {
    console.log('[addFrameToScene] EARLY RETURN: frameId', frameId, 'out of bounds, frames.length:', line.frames.length);
    return scene;
  }

  const newScene = { ...scene, lines: [...scene.lines] };
  newScene.lines[lineId] = {
    ...line,
    frames: [...line.frames],
  };
  newScene.lines[lineId].frames.splice(frameId, 0, frame);

  return newScene;
}

export function removeFrameFromScene<
  S extends { lines: L[] },
  L extends { frames: any[] },
>(scene: S | null, lineId: number, frameId: number): S | null {
  if (!scene) {
    console.log('[removeFrameFromScene] EARLY RETURN: scene is null');
    return scene;
  }
  const line = scene.lines[lineId];
  if (!line) {
    console.log('[removeFrameFromScene] EARLY RETURN: line not found at', lineId, 'total lines:', scene.lines.length);
    return scene;
  }
  if (frameId < 0 || frameId >= line.frames.length) {
    console.log('[removeFrameFromScene] EARLY RETURN: frameId', frameId, 'out of bounds, frames.length:', line.frames.length);
    return scene;
  }

  const newScene = { ...scene, lines: [...scene.lines] };
  newScene.lines[lineId] = {
    ...line,
    frames: [...line.frames],
  };
  newScene.lines[lineId].frames.splice(frameId, 1);

  return newScene;
}
