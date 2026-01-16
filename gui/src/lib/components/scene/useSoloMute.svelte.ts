import { SvelteMap, SvelteSet } from "svelte/reactivity";
import { scene } from "$lib/stores";
import { setFrames, ActionTiming } from "$lib/api/client";
import type { Frame } from "$lib/types/protocol";
import { get } from "svelte/store";

export interface SoloMuteState {
  soloLineIdx: number | null;
  mutedLines: SvelteSet<number>;
  isSolo: (lineIdx: number) => boolean;
  isMuted: (lineIdx: number) => boolean;
  toggleSolo: (lineIdx: number) => Promise<void>;
  toggleMute: (lineIdx: number) => Promise<void>;
}

export function useSoloMute(): SoloMuteState {
  let soloLineIdx = $state<number | null>(null);
  const mutedLines = $state(new SvelteSet<number>());
  let savedEnabledStates = $state(new SvelteMap<string, boolean>());

  function saveCurrentStates() {
    const currentScene = get(scene);
    if (!currentScene || savedEnabledStates.size > 0) return;
    const newStates = new SvelteMap<string, boolean>();
    for (let l = 0; l < currentScene.lines.length; l++) {
      const line = currentScene.lines[l];
      for (let f = 0; f < line.frames.length; f++) {
        newStates.set(`${l}-${f}`, line.frames[f].enabled);
      }
    }
    savedEnabledStates = newStates;
  }

  function getSavedEnabled(lineIdx: number, frameIdx: number): boolean {
    const key = `${lineIdx}-${frameIdx}`;
    return savedEnabledStates.get(key) ?? true;
  }

  async function applyEffects() {
    const currentScene = get(scene);
    if (!currentScene) return;

    const updates: [number, number, Frame][] = [];

    for (let l = 0; l < currentScene.lines.length; l++) {
      const line = currentScene.lines[l];
      for (let f = 0; f < line.frames.length; f++) {
        const frame = line.frames[f];
        let shouldBeEnabled: boolean;

        if (soloLineIdx !== null && l !== soloLineIdx) {
          shouldBeEnabled = false;
        } else if (mutedLines.has(l)) {
          shouldBeEnabled = false;
        } else {
          shouldBeEnabled = getSavedEnabled(l, f);
        }

        if (frame.enabled !== shouldBeEnabled) {
          updates.push([l, f, { ...frame, enabled: shouldBeEnabled }]);
        }
      }
    }

    if (updates.length > 0) {
      try {
        await setFrames(updates, ActionTiming.immediate());
      } catch (error) {
        console.error("Failed to apply solo/mute effects:", error);
      }
    }
  }

  async function toggleSolo(lineIdx: number) {
    if (soloLineIdx === lineIdx) {
      soloLineIdx = null;
      if (mutedLines.size === 0) {
        await applyEffects();
        savedEnabledStates = new SvelteMap();
      } else {
        await applyEffects();
      }
    } else {
      saveCurrentStates();
      soloLineIdx = lineIdx;
      await applyEffects();
    }
  }

  async function toggleMute(lineIdx: number) {
    saveCurrentStates();
    // SvelteSet is reactive - mutate directly instead of creating a new instance
    if (mutedLines.has(lineIdx)) {
      mutedLines.delete(lineIdx);
    } else {
      mutedLines.add(lineIdx);
    }
    await applyEffects();

    if (soloLineIdx === null && mutedLines.size === 0) {
      savedEnabledStates = new SvelteMap();
    }
  }

  function isSolo(lineIdx: number): boolean {
    return soloLineIdx === lineIdx;
  }

  function isMuted(lineIdx: number): boolean {
    return mutedLines.has(lineIdx);
  }

  return {
    get soloLineIdx() { return soloLineIdx; },
    get mutedLines() { return mutedLines; },
    isSolo,
    isMuted,
    toggleSolo,
    toggleMute,
  };
}
