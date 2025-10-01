import type { ActionTiming, Scene } from '../types';
import { sceneStore } from './sceneStore';

// Frame and line operations
export const addFrame = (lineIndex: number, frameIndex: number, timing: ActionTiming = "Immediate") => {
  return {
    InsertFrame: [lineIndex, frameIndex, 1.0, timing] // Insert 1.0 beat frame
  };
};

export const removeFrame = (lineIndex: number, frameIndex: number, timing: ActionTiming = "Immediate") => {
  return {
    RemoveFrame: [lineIndex, frameIndex, timing]
  };
};

export const addLine = (timing: ActionTiming = "Immediate") => {
  // Create a new empty line at the end
  const scene = sceneStore.get();
  if (!scene || scene.lines.length === 0) return null;
  
  const newLine = {
    frames: [1.0], // Start with one frame
    enabled_frames: [true],
    scripts: [],
    frame_names: [null],
    frame_repetitions: [1],
    speed_factor: 1.0,
    index: scene.lines.length,
    start_frame: undefined,
    end_frame: undefined,
    custom_length: undefined,
  };
  
  return {
    SetScene: [{
      ...scene,
      lines: [...scene.lines, newLine]
    }, timing]
  };
};

export const insertLineAfter = (afterIndex: number, timing: ActionTiming = "Immediate") => {
  // Insert a new line after the specified index
  const scene = sceneStore.get();
  if (!scene || afterIndex >= scene.lines.length) return null;
  
  const newLine = {
    frames: [1.0], // Start with one frame
    enabled_frames: [true],
    scripts: [],
    frame_names: [null],
    frame_repetitions: [1],
    speed_factor: 1.0,
    index: afterIndex + 1, // Will be updated below
    start_frame: undefined,
    end_frame: undefined,
    custom_length: undefined,
  };
  
  // Create new lines array with inserted line
  const newLines = [
    ...scene.lines.slice(0, afterIndex + 1),
    newLine,
    ...scene.lines.slice(afterIndex + 1)
  ];
  
  // Update all indices to maintain consistency
  newLines.forEach((line, index) => {
    line.index = index;
  });
  
  return {
    SetScene: [{
      ...scene,
      lines: newLines
    }, timing]
  };
};

export const removeLine = (lineIndex: number, timing: ActionTiming = "Immediate") => {
  const scene = sceneStore.get();
  if (!scene || lineIndex >= scene.lines.length) return null;
  
  const newLines = scene.lines.filter((_, index) => index !== lineIndex);
  // Update indices
  newLines.forEach((line, index) => {
    line.index = index;
  });
  
  return {
    SetScene: [{
      ...scene,
      lines: newLines
    }, timing]
  };
};

export const resizeFrame = (lineIndex: number, frameIndex: number, newDuration: number, timing: ActionTiming = "Immediate") => {
  const scene = sceneStore.get();
  if (!scene || lineIndex >= scene.lines.length) return null;
  
  const line = scene.lines[lineIndex];
  if (!line || frameIndex >= line.frames.length) return null;
  
  // Clamp duration between 0.1 and 8.0 beats
  const clampedDuration = Math.max(0.1, Math.min(8.0, newDuration));
  
  // Create new frames array with updated duration
  const newFrames = [...line.frames].map(frame => frame.duration);
  if(newFrames[frameIndex]) {
    newFrames[frameIndex] = clampedDuration;
  }
  
  return {
    UpdateLineFrames: [lineIndex, newFrames, timing]
  };
};

export const setFrameName = (lineIndex: number, frameIndex: number, name: string | null, timing: ActionTiming = "Immediate") => {
  return {
    SetFrameName: [lineIndex, frameIndex, name, timing]
  };
};

export const setLineLength = (lineIndex: number, length: number | null, timing: ActionTiming = "Immediate") => {
  return {
    SetLineLength: [lineIndex, length, timing]
  };
};

export const setScript = (lineIndex: number, frameIndex: number, content: string, timing: ActionTiming = "Immediate") => {
  return {
    SetScript: [lineIndex, frameIndex, content, timing]
  };
};

export const enableFrames = (lineIndex: number, frameIndices: number[], timing: ActionTiming = "Immediate") => {
  return {
    EnableFrames: [lineIndex, frameIndices, timing]
  };
};

export const disableFrames = (lineIndex: number, frameIndices: number[], timing: ActionTiming = "Immediate") => {
  return {
    DisableFrames: [lineIndex, frameIndices, timing]
  };
};

export const setFrameRepetitions = (lineIndex: number, frameIndex: number, repetitions: number, timing: ActionTiming = "Immediate") => {
  return {
    SetFrameRepetitions: [lineIndex, frameIndex, repetitions, timing]
  };
};

export const setScriptLanguage = (lineIndex: number, frameIndex: number, language: string, timing: ActionTiming = "Immediate") => {
  return {
    SetScriptLanguage: [lineIndex, frameIndex, language, timing] as [number, number, string, ActionTiming]
  };
};

// Scene helpers
export const getMaxFrames = (scene: Scene | null): number => {
  if (!scene) return 0;
  return Math.max(...scene.lines.map(line => line.frames.length));
};

// Frame operation helpers
export const sendFrameOperation = async (operation: any) => {
  // This will be called from components with a client reference
  // For now, just log - we'll connect the client later
  console.log('Frame operation:', operation);
};