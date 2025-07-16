import { atom, map } from 'nanostores';
import type { Scene, ServerMessage, ActionTiming } from '../types';
import { updateGlobalVariables } from './globalVariablesStore';

// Scene store - single source of truth from server
export const sceneStore = atom<Scene | null>(null);

// Grid UI state - separate from scene data
export interface GridUIState {
  selection: {
    start: [number, number];  // [row, col]
    end: [number, number];    // [row, col]
  };
  scrollOffset: number;
  showHelp: boolean;
}

export const gridUIStore = map<GridUIState>({
  selection: { start: [0, 0], end: [0, 0] },
  scrollOffset: 0,
  showHelp: false
});

// Grid progression cache for performance
export const progressionCache = atom<Map<string, number>>(new Map());

// Additional stores for other server data
export const playbackStore = map({
  isPlaying: false,
  currentFramePositions: [] as [number, number, number][], // [line, frame, progression]
  clockState: [0, 0, 0, 0] as [number, number, number, number], // [beat, tempo, micros, quantum]
});

export const peersStore = map({
  peerList: [] as string[],
  peerSelections: new Map<string, { start: [number, number], end: [number, number] }>(),
  peerEditing: new Map<string, [number, number]>(), // [line, frame]
});

export const compilationStore = map({
  errors: [] as Array<{ line_idx: number; frame_idx: number; error: string }>,
  compiledFrames: new Set<string>(), // "line_idx:frame_idx"
});

// Script editor state  
export const scriptEditorStore = map({
  currentScript: '',
  selectedFrame: null as { line_idx: number; frame_idx: number } | null,
  isLoading: false,
  compilationError: null as any | null, // This will hold the CompilationError object
  lastCompiled: 0,
});

// Comprehensive server message handler
export const handleServerMessage = (message: ServerMessage) => {
  
  // Handle string messages first
  if (typeof message === 'string') {
    switch (message) {
      case 'TransportStarted':
        playbackStore.setKey('isPlaying', true);
        return;
      case 'TransportStopped':
        playbackStore.setKey('isPlaying', false);
        return;
      case 'Success':
        return;
      default:
        return;
    }
  }
  
  // Handle object messages
  if (typeof message === 'object' && message !== null) {
    // Direct check for compilation messages (avoid switch statement issues)
    if ('CompilationErrorOccurred' in message) {
      const error = message.CompilationErrorOccurred;
      scriptEditorStore.setKey('compilationError', error);
      return;
    }
    
    if ('ScriptCompiled' in message) {
      const { line_idx: compiledLine, frame_idx: compiledFrame } = message.ScriptCompiled;
      const currentFrame = scriptEditorStore.get().selectedFrame;
      
      // Only clear error and show success if this is for the currently selected frame
      if (currentFrame && currentFrame.line_idx === compiledLine && currentFrame.frame_idx === compiledFrame) {
        scriptEditorStore.setKey('compilationError', null);
        scriptEditorStore.setKey('lastCompiled', Date.now());
      }
      return;
    }
    
    switch (true) {
      // Scene data updates
      case 'Hello' in message:
        sceneStore.set(message.Hello.scene);
        peersStore.setKey('peerList', message.Hello.peers);
        playbackStore.setKey('isPlaying', message.Hello.is_playing);
        break;
    
    case 'SceneValue' in message:
      sceneStore.set(message.SceneValue);
      break;
    
    case 'SceneLength' in message:
      const currentScene = sceneStore.get();
      if (currentScene) {
        sceneStore.set({ ...currentScene, length: message.SceneLength });
      }
      break;
    
    case 'Snapshot' in message:
      sceneStore.set(message.Snapshot.scene);
      playbackStore.setKey('clockState', [
        message.Snapshot.beat,
        message.Snapshot.tempo,
        message.Snapshot.micros,
        message.Snapshot.quantum
      ]);
      break;

    // Playback state updates (object format only - strings handled above)
    case 'TransportStarted' in message:
      playbackStore.setKey('isPlaying', true);
      break;
    
    case 'TransportStopped' in message:
      playbackStore.setKey('isPlaying', false);
      break;
    
    case 'ClockState' in message:
      playbackStore.setKey('clockState', message.ClockState);
      break;
    
    case 'FramePosition' in message:
      playbackStore.setKey('currentFramePositions', message.FramePosition);
      break;

    // Script compilation updates
    case 'ScriptCompiled' in message:
      const compiledMessage = message as { ScriptCompiled: { line_idx: number; frame_idx: number } };
      const frameKey = `${compiledMessage.ScriptCompiled.line_idx}:${compiledMessage.ScriptCompiled.frame_idx}`;
      const compiled = new Set(compilationStore.get().compiledFrames);
      compiled.add(frameKey);
      compilationStore.setKey('compiledFrames', compiled);
      // Remove any errors for this frame
      const errors = compilationStore.get().errors.filter(
        err => !(err.line_idx === compiledMessage.ScriptCompiled.line_idx && err.frame_idx === compiledMessage.ScriptCompiled.frame_idx)
      );
      compilationStore.setKey('errors', errors);
      break;
    
    case 'CompilationErrorOccurred' in message:
      const errorMessage = message as { CompilationErrorOccurred: { info: string } };
      const newErrors = [...compilationStore.get().errors];
      newErrors.push({
        line_idx: 0, // Will need to parse from error info
        frame_idx: 0, // Will need to parse from error info
        error: errorMessage.CompilationErrorOccurred.info
      });
      compilationStore.setKey('errors', newErrors);
      break;

    // Peer updates
    case 'PeersUpdated' in message:
      peersStore.setKey('peerList', message.PeersUpdated);
      break;
    
    case 'PeerGridSelectionUpdate' in message:
      const [peerName, selection] = message.PeerGridSelectionUpdate;
      const peerSelections = new Map(peersStore.get().peerSelections);
      peerSelections.set(peerName, selection);
      peersStore.setKey('peerSelections', peerSelections);
      break;
    
    case 'PeerStartedEditing' in message:
      const [startPeer, startLine, startFrame] = message.PeerStartedEditing;
      const peerEditingStart = new Map(peersStore.get().peerEditing);
      peerEditingStart.set(startPeer, [startLine, startFrame]);
      peersStore.setKey('peerEditing', peerEditingStart);
      break;
    
    case 'PeerStoppedEditing' in message:
      const [stopPeer] = message.PeerStoppedEditing;
      const peerEditingStop = new Map(peersStore.get().peerEditing);
      peerEditingStop.delete(stopPeer);
      peersStore.setKey('peerEditing', peerEditingStop);
      break;

    // Script messages - following TUI pattern exactly
    case 'ScriptContent' in message:
      const { line_idx, frame_idx, content } = message.ScriptContent;
      scriptEditorStore.setKey('currentScript', content);
      scriptEditorStore.setKey('selectedFrame', { line_idx, frame_idx });
      scriptEditorStore.setKey('isLoading', false);
      // Clear compilation error when loading new script (like TUI does)
      scriptEditorStore.setKey('compilationError', null);
      break;

    // Global variables update
    case 'GlobalVariablesUpdate' in message:
      updateGlobalVariables(message.GlobalVariablesUpdate);
      break;

      // Other message types (Success, InternalError, etc.) don't affect stores
      default:
        break;
    }
  }
};

// Grid UI helpers
export const updateGridSelection = (selection: GridUIState['selection']) => {
  gridUIStore.setKey('selection', selection);
};

export const updateGridScrollOffset = (offset: number) => {
  gridUIStore.setKey('scrollOffset', offset);
};

export const toggleGridHelp = () => {
  gridUIStore.setKey('showHelp', !gridUIStore.get().showHelp);
};

// Utility functions
export const getGridSelectionBounds = (selection: GridUIState['selection']): [[number, number], [number, number]] => {
  const [startRow, startCol] = selection.start;
  const [endRow, endCol] = selection.end;
  
  return [
    [Math.min(startRow, endRow), Math.min(startCol, endCol)],
    [Math.max(startRow, endRow), Math.max(startCol, endCol)]
  ];
};

export const isGridSelectionSingle = (selection: GridUIState['selection']): boolean => {
  return selection.start[0] === selection.end[0] && selection.start[1] === selection.end[1];
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
    index: scene.lines.length
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
    index: afterIndex + 1 // Will be updated below
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
  const newFrames = [...line.frames];
  newFrames[frameIndex] = clampedDuration;
  
  return {
    UpdateLineFrames: [lineIndex, newFrames, timing]
  };
};

export const setFrameName = (lineIndex: number, frameIndex: number, name: string | null, timing: ActionTiming = "Immediate") => {
  return {
    SetFrameName: [lineIndex, frameIndex, name, timing]
  };
};

export const setSceneLength = (length: number, timing: ActionTiming = "Immediate") => {
  return {
    SetSceneLength: [length, timing]
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