import type { Frame, Scene } from "$lib/types/protocol";
import { type Selection, getSelectedClipIds, fromClipId, toClipId } from "./selection";

export interface ClipboardData {
  frames: (Frame | null)[][];
}

let clipboard: ClipboardData | null = null;

export function copySelection(scene: Scene, sel: Selection): void {
  const clipIds = getSelectedClipIds(sel, scene);
  if (clipIds.length === 0) return;

  const coords = clipIds.map(id => fromClipId(id));
  const minLine = Math.min(...coords.map(c => c.lineIdx));
  const maxLine = Math.max(...coords.map(c => c.lineIdx));
  const minFrame = Math.min(...coords.map(c => c.frameIdx));
  const maxFrame = Math.max(...coords.map(c => c.frameIdx));

  const selectedSet = new Set(clipIds);
  const frames: (Frame | null)[][] = [];

  for (let l = minLine; l <= maxLine; l++) {
    const row: (Frame | null)[] = [];
    const line = scene.lines[l];
    for (let f = minFrame; f <= maxFrame; f++) {
      const clipId = toClipId(l, f);
      if (selectedSet.has(clipId) && line?.frames[f]) {
        row.push(structuredClone(line.frames[f]));
      } else {
        row.push(null);
      }
    }
    frames.push(row);
  }

  clipboard = { frames };
}

export function getClipboard(): ClipboardData | null {
  return clipboard ? structuredClone(clipboard) : null;
}
