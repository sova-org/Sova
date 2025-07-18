// Backward compatibility facade - re-export all stores from their new locations
export { sceneStore, progressionCache, getScene, getSceneLength, getSceneLines } from './sceneDataStore';
export { gridUIStore, type GridUIState, updateGridSelection, updateGridScrollOffset, toggleGridHelp, getGridSelectionBounds, isGridSelectionSingle } from './gridUIStore';
export { playbackStore, isPlaying, getCurrentFramePositions, getClockState } from './playbackStore';
export { peersStore, getPeerList, getPeerSelections, getPeerEditing, isPeerEditing, getPeerEditingFrame } from './peersStore';
export { compilationStore, getCompilationErrors, getCompiledFrames, isFrameCompiled, getFrameErrors } from './compilationStore';
export { scriptEditorStore, getCurrentScript, getSelectedFrame, isScriptLoading, getCompilationError, getLastCompiled, setCurrentScript, setSelectedFrame, setScriptLoading, clearCompilationError } from './scriptEditorStore';
export * from './sceneOperations';

// Import all message handlers
import { handleSceneMessage } from './sceneDataStore';
import { handlePlaybackMessage } from './playbackStore';
import { handlePeerMessage } from './peersStore';
import { handleCompilationMessage } from './compilationStore';
import { handleScriptEditorMessage } from './scriptEditorStore';
import { handleProjectMessage } from './projectStore';
import { addLog } from './optimizedLogStore';
import type { ServerMessage } from '../types';

// Simple log message handler using optimized store
const handleLogMessage = (message: ServerMessage): void => {
  if (typeof message === 'string') {
    switch (message) {
      case 'TransportStarted':
        addLog('info', 'Transport started');
        break;
      case 'TransportStopped':
        addLog('info', 'Transport stopped');
        break;
      case 'Success':
        addLog('debug', 'Operation completed successfully');
        break;
      default:
        addLog('info', `Server: ${message}`);
    }
  } else if (typeof message === 'object' && message !== null) {
    // Handle key object messages
    if ('LogString' in message) {
      addLog('info', message.LogString);
    } else if ('CompilationErrorOccurred' in message) {
      const error = message.CompilationErrorOccurred;
      addLog('error', `Compilation error in ${error.lang}: ${error.info}`);
    } else if ('ScriptCompiled' in message) {
      const { line_idx, frame_idx } = message.ScriptCompiled;
      addLog('info', `Script compiled: line ${line_idx}, frame ${frame_idx}`);
    }
  }
};

// Comprehensive server message handler that delegates to specialized stores
export const handleServerMessage = (message: ServerMessage): void => {
  // Handle string messages first
  if (typeof message === 'string') {
    switch (message) {
      case 'TransportStarted':
      case 'TransportStopped':
        handlePlaybackMessage(message);
        return;
      case 'Success':
        return;
      default:
        return;
    }
  }
  
  // Handle object messages by trying each specialized handler
  if (typeof message === 'object' && message !== null) {
    // Try each handler - multiple handlers can process the same message
    handleSceneMessage(message);
    handlePlaybackMessage(message);
    handlePeerMessage(message);
    handleCompilationMessage(message);
    handleScriptEditorMessage(message);
    handleProjectMessage(message);
    handleLogMessage(message);
  }
};