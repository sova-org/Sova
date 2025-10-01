import { map } from 'nanostores';
import { batchUpdateMap } from '../../utils/store-helpers';
import type { ServerMessage, CompilationError } from '../../types';

// Grid UI State
export interface GridUIState {
  selection: {
    start: [number, number];
    end: [number, number];
  };
  scrollOffset: number;
  showHelp: boolean;
}

export const gridUIStore = map<GridUIState>({
  selection: { start: [0, 0], end: [0, 0] },
  scrollOffset: 0,
  showHelp: false
});

export const updateGridSelection = (selection: GridUIState['selection']) => {
  gridUIStore.setKey('selection', selection);
};

export const updateGridScrollOffset = (offset: number) => {
  gridUIStore.setKey('scrollOffset', offset);
};

export const toggleGridHelp = () => {
  gridUIStore.setKey('showHelp', !gridUIStore.get().showHelp);
};

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

// Playback State
export interface PlaybackState {
  isPlaying: boolean;
  currentFramePositions: [number, number, number][];
  clockState: [number, number, number, number];
}

export const playbackStore = map<PlaybackState>({
  isPlaying: false,
  currentFramePositions: [],
  clockState: [0, 0, 0, 0],
});

export const handlePlaybackMessage = (message: ServerMessage) => {
  if (typeof message === 'string') {
    switch (message) {
      case 'TransportStarted':
        playbackStore.setKey('isPlaying', true);
        return true;
      case 'TransportStopped':
        playbackStore.setKey('isPlaying', false);
        return true;
    }
    return false;
  }

  if (typeof message === 'object' && message !== null) {
    switch (true) {
      case 'Hello' in message:
        playbackStore.setKey('isPlaying', message.Hello.is_playing);
        return true;

      case 'Snapshot' in message:
        playbackStore.setKey('clockState', [
          message.Snapshot.tempo,
          message.Snapshot.beat,
          message.Snapshot.micros,
          message.Snapshot.quantum
        ]);
        return true;

      case 'TransportStarted' in message:
        playbackStore.setKey('isPlaying', true);
        return true;

      case 'TransportStopped' in message:
        playbackStore.setKey('isPlaying', false);
        return true;

      case 'ClockState' in message:
        playbackStore.setKey('clockState', message.ClockState);
        return true;

      case 'FramePosition' in message:
        playbackStore.setKey('currentFramePositions', message.FramePosition);
        return true;
    }
  }

  return false;
};

export const isPlaying = () => playbackStore.get().isPlaying;
export const getCurrentFramePositions = () => playbackStore.get().currentFramePositions;
export const getClockState = () => playbackStore.get().clockState;

// Peers State
export interface PeersState {
  peerList: string[];
  peerSelections: Map<string, { start: [number, number], end: [number, number] }>;
  peerEditing: Map<string, [number, number]>;
}

export const peersStore = map<PeersState>({
  peerList: [],
  peerSelections: new Map(),
  peerEditing: new Map(),
});

export const handlePeerMessage = (message: ServerMessage) => {
  if (typeof message === 'object' && message !== null) {
    switch (true) {
      case 'Hello' in message:
        peersStore.setKey('peerList', message.Hello.peers);
        return true;

      case 'PeersUpdated' in message:
        peersStore.setKey('peerList', message.PeersUpdated);
        return true;

      case 'PeerGridSelectionUpdate' in message:
        const [peerName, selection] = message.PeerGridSelectionUpdate;
        const peerSelections = new Map(peersStore.get().peerSelections);
        peerSelections.set(peerName, selection);
        peersStore.setKey('peerSelections', peerSelections);
        return true;

      case 'PeerStartedEditing' in message:
        const [startPeer, startLine, startFrame] = message.PeerStartedEditing;
        const peerEditingStart = new Map(peersStore.get().peerEditing);
        peerEditingStart.set(startPeer, [startLine, startFrame]);
        peersStore.setKey('peerEditing', peerEditingStart);
        return true;

      case 'PeerStoppedEditing' in message:
        const [stopPeer] = message.PeerStoppedEditing;
        const peerEditingStop = new Map(peersStore.get().peerEditing);
        peerEditingStop.delete(stopPeer);
        peersStore.setKey('peerEditing', peerEditingStop);
        return true;
    }
  }

  return false;
};

export const getPeerList = () => peersStore.get().peerList;
export const getPeerSelections = () => peersStore.get().peerSelections;
export const getPeerEditing = () => peersStore.get().peerEditing;
export const isPeerEditing = (peer: string) => peersStore.get().peerEditing.has(peer);
export const getPeerEditingFrame = (peer: string) => peersStore.get().peerEditing.get(peer);

// Compilation State
export interface CompilationState {
  errors: Array<{ line_idx: number; frame_idx: number; error: string }>;
  compiledFrames: Set<string>;
}

export const compilationStore = map<CompilationState>({
  errors: [],
  compiledFrames: new Set(),
});

export const handleCompilationMessage = (message: ServerMessage) => {
  if (typeof message === 'object' && message !== null) {
    switch (true) {
      case 'ScriptCompiled' in message:
        const { line_idx, frame_idx } = message.ScriptCompiled;
        const frameKey = `${line_idx}:${frame_idx}`;
        const compiled = new Set(compilationStore.get().compiledFrames);
        compiled.add(frameKey);
        compilationStore.setKey('compiledFrames', compiled);

        const errors = compilationStore.get().errors.filter(
          err => !(err.line_idx === line_idx && err.frame_idx === frame_idx)
        );
        compilationStore.setKey('errors', errors);
        return true;

      case 'CompilationErrorOccurred' in message:
        const errorInfo = message.CompilationErrorOccurred;
        const newErrors = [...compilationStore.get().errors];
        newErrors.push({
          line_idx: 0,
          frame_idx: 0,
          error: errorInfo.info
        });
        compilationStore.setKey('errors', newErrors);
        return true;
    }
  }

  return false;
};

export const getCompilationErrors = () => compilationStore.get().errors;
export const getCompiledFrames = () => compilationStore.get().compiledFrames;
export const isFrameCompiled = (lineIdx: number, frameIdx: number) => {
  const frameKey = `${lineIdx}:${frameIdx}`;
  return compilationStore.get().compiledFrames.has(frameKey);
};
export const getFrameErrors = (lineIdx: number, frameIdx: number) => {
  return compilationStore.get().errors.filter(
    err => err.line_idx === lineIdx && err.frame_idx === frameIdx
  );
};

// Script Editor State
export interface ScriptEditorState {
  currentScript: string;
  selectedFrame: { line_idx: number; frame_idx: number } | null;
  isLoading: boolean;
  compilationError: CompilationError | null;
  lastCompiled: number;
}

export const scriptEditorStore = map<ScriptEditorState>({
  currentScript: '',
  selectedFrame: null,
  isLoading: false,
  compilationError: null,
  lastCompiled: 0,
});

export const handleScriptEditorMessage = (message: ServerMessage) => {
  if (typeof message === 'object' && message !== null) {
    if ('CompilationErrorOccurred' in message) {
      const error = message.CompilationErrorOccurred;
      scriptEditorStore.setKey('compilationError', error);
      return true;
    }

    if ('ScriptCompiled' in message) {
      const { line_idx: compiledLine, frame_idx: compiledFrame } = message.ScriptCompiled;
      const currentFrame = scriptEditorStore.get().selectedFrame;

      if (currentFrame && currentFrame.line_idx === compiledLine && currentFrame.frame_idx === compiledFrame) {
        batchUpdateMap(scriptEditorStore, {
          compilationError: null,
          lastCompiled: Date.now(),
        });
      }
      return true;
    }

    if ('ScriptContent' in message) {
      const { line_idx, frame_idx, content } = message.ScriptContent;
      batchUpdateMap(scriptEditorStore, {
        currentScript: content,
        selectedFrame: { line_idx, frame_idx },
        isLoading: false,
        compilationError: null,
      });
      return true;
    }
  }

  return false;
};

export const getCurrentScript = () => scriptEditorStore.get().currentScript;
export const getSelectedFrame = () => scriptEditorStore.get().selectedFrame;
export const isScriptLoading = () => scriptEditorStore.get().isLoading;
export const getCompilationError = () => scriptEditorStore.get().compilationError;
export const getLastCompiled = () => scriptEditorStore.get().lastCompiled;

export const setCurrentScript = (script: string) => {
  scriptEditorStore.setKey('currentScript', script);
};

export const setSelectedFrame = (frame: { line_idx: number; frame_idx: number } | null) => {
  scriptEditorStore.setKey('selectedFrame', frame);
};

export const setScriptLoading = (loading: boolean) => {
  scriptEditorStore.setKey('isLoading', loading);
};

export const clearCompilationError = () => {
  scriptEditorStore.setKey('compilationError', null);
};
