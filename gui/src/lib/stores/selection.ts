import { writable } from "svelte/store";
import type { Scene } from "$lib/types/protocol";

export type ClipId = `${number}-${number}`;

export function toClipId(lineIdx: number, frameIdx: number): ClipId {
  return `${lineIdx}-${frameIdx}`;
}

export function fromClipId(id: ClipId): { lineIdx: number; frameIdx: number } {
  const [lineIdx, frameIdx] = id.split("-").map(Number);
  return { lineIdx, frameIdx };
}

export interface Selection {
  anchor: { lineId: number; frameId: number };
  focus: { lineId: number; frameId: number };
  clips: Set<ClipId>;
}

export const selection = writable<Selection | null>(null);

export function getSelectionBounds(sel: Selection) {
  return {
    minLine: Math.min(sel.anchor.lineId, sel.focus.lineId),
    maxLine: Math.max(sel.anchor.lineId, sel.focus.lineId),
    minFrame: Math.min(sel.anchor.frameId, sel.focus.frameId),
    maxFrame: Math.max(sel.anchor.frameId, sel.focus.frameId),
  };
}

export function isFrameInSelection(
  sel: Selection | null,
  scene: Scene | null,
  lineId: number,
  frameId: number,
): boolean {
  if (!sel || !scene) return false;
  const line = scene.lines[lineId];
  if (!line || frameId >= line.frames.length) return false;

  if (sel.clips.has(toClipId(lineId, frameId))) {
    return true;
  }

  const bounds = getSelectionBounds(sel);
  return (
    lineId >= bounds.minLine &&
    lineId <= bounds.maxLine &&
    frameId >= bounds.minFrame &&
    frameId <= bounds.maxFrame
  );
}

export function selectFrame(lineId: number, frameId: number): void {
  selection.set({
    anchor: { lineId, frameId },
    focus: { lineId, frameId },
    clips: new Set(),
  });
}

export function extendSelection(lineId: number, frameId: number): void {
  selection.update((s) =>
    s
      ? { ...s, focus: { lineId, frameId }, clips: new Set() }
      : { anchor: { lineId, frameId }, focus: { lineId, frameId }, clips: new Set() },
  );
}

export function collapseToFocus(): void {
  selection.update((s) => (s ? { anchor: s.focus, focus: s.focus, clips: s.clips } : null));
}

export function setSelectedClips(clips: ClipId[]): void {
  const clipSet = new Set(clips);
  if (clips.length === 0) {
    selection.set(null);
    return;
  }
  const first = fromClipId(clips[0]);
  selection.set({
    anchor: { lineId: first.lineIdx, frameId: first.frameIdx },
    focus: { lineId: first.lineIdx, frameId: first.frameIdx },
    clips: clipSet,
  });
}

export function addSelectedClips(clips: ClipId[]): void {
  selection.update((s) => {
    if (!s) {
      return setSelectedClipsAndReturn(clips);
    }
    const newClips = new Set(s.clips);
    for (const clip of clips) {
      newClips.add(clip);
    }
    return { ...s, clips: newClips };
  });
}

function setSelectedClipsAndReturn(clips: ClipId[]): Selection | null {
  if (clips.length === 0) return null;
  const first = fromClipId(clips[0]);
  return {
    anchor: { lineId: first.lineIdx, frameId: first.frameIdx },
    focus: { lineId: first.lineIdx, frameId: first.frameIdx },
    clips: new Set(clips),
  };
}

export function getSelectedClipIds(sel: Selection, scene: Scene): ClipId[] {
  if (sel.clips.size > 0) {
    return Array.from(sel.clips);
  }

  const bounds = getSelectionBounds(sel);
  const clips: ClipId[] = [];
  for (let l = bounds.minLine; l <= bounds.maxLine; l++) {
    const line = scene.lines[l];
    if (!line) continue;
    for (let f = bounds.minFrame; f <= bounds.maxFrame; f++) {
      if (f < line.frames.length) {
        clips.push(toClipId(l, f));
      }
    }
  }
  return clips;
}
