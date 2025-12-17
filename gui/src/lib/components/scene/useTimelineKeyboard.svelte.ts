import { invoke } from "@tauri-apps/api/core";
import { scene } from "$lib/stores";
import { snapGranularity } from "$lib/stores/snapGranularity";
import {
  selection,
  selectFrame,
  extendSelection,
  collapseToFocus,
  getSelectedClipIds,
  fromClipId,
} from "$lib/stores/selection";
import {
  copySelection,
  getClipboard,
  type ClipboardData,
} from "$lib/stores/clipboard";
import { setFrames, addFrame, removeFrame, ActionTiming } from "$lib/api/client";
import type { Frame, Line } from "$lib/types/protocol";
import { type TimelineContext, getDuration } from "./context.svelte";
import { get } from "svelte/store";

export interface KeyboardConfig {
  ctx: TimelineContext;
  onOpenEditor: (lineIdx: number, frameIdx: number) => void;
}

export function useTimelineKeyboard(config: KeyboardConfig) {
  const { ctx, onOpenEditor } = config;

  function moveToPreviousTrack(lineIdx: number, frameIdx: number) {
    const currentScene = get(scene);
    if (!currentScene || lineIdx <= 0) return;
    const newFrameIdx = Math.min(
      frameIdx,
      currentScene.lines[lineIdx - 1].frames.length - 1
    );
    selectFrame(lineIdx - 1, Math.max(0, newFrameIdx));
  }

  function moveToNextTrack(lineIdx: number, frameIdx: number) {
    const currentScene = get(scene);
    if (!currentScene || lineIdx >= currentScene.lines.length - 1) return;
    const newFrameIdx = Math.min(
      frameIdx,
      currentScene.lines[lineIdx + 1].frames.length - 1
    );
    selectFrame(lineIdx + 1, Math.max(0, newFrameIdx));
  }

  function moveToPreviousFrame(lineIdx: number, frameIdx: number) {
    if (frameIdx > 0) {
      selectFrame(lineIdx, frameIdx - 1);
    }
  }

  function moveToNextFrame(lineIdx: number, frameIdx: number, line: Line) {
    if (frameIdx < line.frames.length - 1) {
      selectFrame(lineIdx, frameIdx + 1);
    }
  }

  async function insertFramesAfter(
    lineIdx: number,
    frameIdx: number,
    data: ClipboardData
  ) {
    const currentScene = get(scene);
    if (!currentScene || data.frames.length === 0) return;

    for (let l = 0; l < data.frames.length; l++) {
      const targetLine = lineIdx + l;
      if (targetLine >= currentScene.lines.length) continue;

      const row = data.frames[l];
      let insertIdx = frameIdx + 1;

      for (const frame of row) {
        if (frame) {
          await addFrame(targetLine, insertIdx, structuredClone(frame));
          insertIdx++;
        }
      }
    }

    selectFrame(lineIdx, frameIdx + 1);
  }

  async function insertFramesBefore(
    lineIdx: number,
    frameIdx: number,
    data: ClipboardData
  ) {
    const currentScene = get(scene);
    if (!currentScene || data.frames.length === 0) return;

    for (let l = 0; l < data.frames.length; l++) {
      const targetLine = lineIdx + l;
      if (targetLine >= currentScene.lines.length) continue;

      const row = data.frames[l];
      let insertIdx = frameIdx;

      for (const frame of row) {
        if (frame) {
          await addFrame(targetLine, insertIdx, structuredClone(frame));
          insertIdx++;
        }
      }
    }

    selectFrame(lineIdx, frameIdx);
  }

  async function insertNewFrameBefore(lineIdx: number, frameIdx: number) {
    const frame = await invoke<Frame>("create_default_frame");
    await addFrame(lineIdx, frameIdx, frame);
    selectFrame(lineIdx, frameIdx);
  }

  async function insertNewFrameAfter(lineIdx: number, frameIdx: number) {
    const frame = await invoke<Frame>("create_default_frame");
    await addFrame(lineIdx, frameIdx + 1, frame);
    selectFrame(lineIdx, frameIdx + 1);
  }

  async function handleRemoveSelectedFrames() {
    const currentScene = get(scene);
    const currentSelection = get(selection);
    if (!currentScene || !currentSelection) return;

    const clipIds = getSelectedClipIds(currentSelection, currentScene);
    if (clipIds.length === 0) return;

    const framesToRemove = clipIds
      .map(id => fromClipId(id))
      .sort((a, b) => {
        if (a.lineIdx !== b.lineIdx) return b.lineIdx - a.lineIdx;
        return b.frameIdx - a.frameIdx;
      });

    for (const { lineIdx, frameIdx } of framesToRemove) {
      await removeFrame(lineIdx, frameIdx);
    }

    const updatedScene = get(scene);
    if (!updatedScene || updatedScene.lines.length === 0) {
      selection.set(null);
    } else {
      const minLine = Math.min(...framesToRemove.map(f => f.lineIdx));
      const minFrame = Math.min(...framesToRemove.map(f => f.frameIdx));
      const newLineIdx = Math.min(minLine, updatedScene.lines.length - 1);
      const newLine = updatedScene.lines[newLineIdx];
      if (newLine && newLine.frames.length > 0) {
        selectFrame(newLineIdx, Math.min(minFrame, newLine.frames.length - 1));
      } else {
        selection.set(null);
      }
    }
  }

  async function adjustDuration(lineIdx: number, frameIdx: number, delta: number) {
    const currentScene = get(scene);
    if (!currentScene) return;
    const frame = currentScene.lines[lineIdx]?.frames[frameIdx];
    if (!frame) return;

    const snap = get(snapGranularity);
    const minDuration = Math.abs(delta) < snap ? snap / 2 : snap;
    const newDuration = Math.max(minDuration, getDuration(frame) + delta);
    const updatedFrame = { ...frame, duration: newDuration };
    try {
      await setFrames([[lineIdx, frameIdx, updatedFrame]], ActionTiming.immediate());
    } catch (error) {
      console.error("Failed to adjust duration:", error);
    }
  }

  async function toggleEnabled(lineIdx: number, frameIdx: number) {
    const currentScene = get(scene);
    if (!currentScene) return;
    const frame = currentScene.lines[lineIdx]?.frames[frameIdx];
    if (!frame) return;

    const newEnabled = frame.enabled === false ? true : false;
    const updatedFrame = { ...frame, enabled: newEnabled };

    try {
      await setFrames([[lineIdx, frameIdx, updatedFrame]], ActionTiming.immediate());
    } catch (error) {
      console.error("Failed to toggle enabled:", error);
    }
  }

  function cycleSelection(reverse: boolean) {
    const currentScene = get(scene);
    const currentSelection = get(selection);
    if (!currentScene || currentScene.lines.length === 0) return;

    let lineIdx = currentSelection?.focus.lineId ?? 0;
    let frameIdx = currentSelection?.focus.frameId ?? 0;

    if (reverse) {
      frameIdx--;
      if (frameIdx < 0) {
        lineIdx--;
        if (lineIdx < 0) lineIdx = currentScene.lines.length - 1;
        frameIdx = currentScene.lines[lineIdx].frames.length - 1;
      }
    } else {
      frameIdx++;
      if (frameIdx >= currentScene.lines[lineIdx].frames.length) {
        lineIdx++;
        if (lineIdx >= currentScene.lines.length) lineIdx = 0;
        frameIdx = 0;
      }
    }

    selectFrame(lineIdx, Math.max(0, frameIdx));
  }

  function handleKeydown(event: KeyboardEvent) {
    if (ctx.isEditing()) return;

    const currentScene = get(scene);
    const currentSelection = get(selection);
    if (!currentScene || currentScene.lines.length === 0) return;

    const { key } = event;
    const lineIdx = currentSelection?.focus.lineId ?? 0;
    const frameIdx = currentSelection?.focus.frameId ?? 0;
    const line = currentScene.lines[lineIdx];
    if (!line) return;

    // Clipboard operations (Ctrl/Cmd + key)
    if ((event.ctrlKey || event.metaKey) && currentSelection) {
      switch (key.toLowerCase()) {
        case "c":
          event.preventDefault();
          copySelection(currentScene, currentSelection);
          return;
        case "v": {
          event.preventDefault();
          const pasted = getClipboard();
          if (pasted) {
            if (event.shiftKey) {
              insertFramesBefore(lineIdx, frameIdx, pasted);
            } else {
              insertFramesAfter(lineIdx, frameIdx, pasted);
            }
          }
          return;
        }
        case "d": {
          event.preventDefault();
          copySelection(currentScene, currentSelection);
          const duplicateData = getClipboard();
          if (duplicateData) insertFramesAfter(lineIdx, frameIdx, duplicateData);
          return;
        }
      }
    }

    // Arrow keys with Shift extend selection
    if (event.shiftKey && key.startsWith("Arrow")) {
      event.preventDefault();
      let newLine = lineIdx;
      let newFrame = frameIdx;

      switch (key) {
        case "ArrowUp":
          if (ctx.isVertical) {
            newFrame = Math.max(0, frameIdx - 1);
          } else {
            newLine = Math.max(0, lineIdx - 1);
          }
          break;
        case "ArrowDown":
          if (ctx.isVertical) {
            newFrame = Math.min(line.frames.length - 1, frameIdx + 1);
          } else {
            newLine = Math.min(currentScene.lines.length - 1, lineIdx + 1);
          }
          break;
        case "ArrowLeft":
          if (ctx.isVertical) {
            newLine = Math.max(0, lineIdx - 1);
          } else {
            newFrame = Math.max(0, frameIdx - 1);
          }
          break;
        case "ArrowRight":
          if (ctx.isVertical) {
            newLine = Math.min(currentScene.lines.length - 1, lineIdx + 1);
          } else {
            newFrame = Math.min(line.frames.length - 1, frameIdx + 1);
          }
          break;
      }
      extendSelection(newLine, newFrame);
      return;
    }

    const snap = get(snapGranularity);

    switch (key) {
      case "ArrowUp":
        event.preventDefault();
        if (ctx.isVertical) {
          moveToPreviousFrame(lineIdx, frameIdx);
        } else {
          moveToPreviousTrack(lineIdx, frameIdx);
        }
        break;
      case "ArrowDown":
        event.preventDefault();
        if (ctx.isVertical) {
          moveToNextFrame(lineIdx, frameIdx, line);
        } else {
          moveToNextTrack(lineIdx, frameIdx);
        }
        break;
      case "ArrowLeft":
        event.preventDefault();
        if (ctx.isVertical) {
          moveToPreviousTrack(lineIdx, frameIdx);
        } else {
          moveToPreviousFrame(lineIdx, frameIdx);
        }
        break;
      case "ArrowRight":
        event.preventDefault();
        if (ctx.isVertical) {
          moveToNextTrack(lineIdx, frameIdx);
        } else {
          moveToNextFrame(lineIdx, frameIdx, line);
        }
        break;
      case "Escape":
        event.preventDefault();
        collapseToFocus();
        break;
      case "Enter":
        event.preventDefault();
        onOpenEditor(lineIdx, frameIdx);
        break;
      case "Delete":
      case "Backspace":
        event.preventDefault();
        handleRemoveSelectedFrames();
        break;
      case "+":
      case "=":
        event.preventDefault();
        adjustDuration(lineIdx, frameIdx, event.shiftKey ? snap / 2 : snap);
        break;
      case "-":
      case "_":
        event.preventDefault();
        adjustDuration(lineIdx, frameIdx, event.shiftKey ? -snap / 2 : -snap);
        break;
      case " ":
        event.preventDefault();
        toggleEnabled(lineIdx, frameIdx);
        break;
      case "Tab":
        event.preventDefault();
        cycleSelection(event.shiftKey);
        break;
      case "i":
        event.preventDefault();
        insertNewFrameBefore(lineIdx, frameIdx);
        break;
      case "a":
        event.preventDefault();
        insertNewFrameAfter(lineIdx, frameIdx);
        break;
      case "e":
      case "E":
        event.preventDefault();
        toggleEnabled(lineIdx, frameIdx);
        break;
    }
  }

  return { handleKeydown, toggleEnabled };
}
