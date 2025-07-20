import { map } from 'nanostores';
import { batchUpdateMap } from '../utils/store-helpers';
import type { ServerMessage, CompilationError } from '../types';

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

// Script editor message handlers
export const handleScriptEditorMessage = (message: ServerMessage) => {
  if (typeof message === 'object' && message !== null) {
    // Direct check for compilation messages (avoid switch statement issues)
    if ('CompilationErrorOccurred' in message) {
      const error = message.CompilationErrorOccurred;
      scriptEditorStore.setKey('compilationError', error);
      return true;
    }
    
    if ('ScriptCompiled' in message) {
      const { line_idx: compiledLine, frame_idx: compiledFrame } = message.ScriptCompiled;
      const currentFrame = scriptEditorStore.get().selectedFrame;
      
      // Only clear error and show success if this is for the currently selected frame
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
        compilationError: null, // Clear compilation error when loading new script
      });
      return true;
    }
  }
  
  return false;
};

// Helper functions
export const getCurrentScript = () => scriptEditorStore.get().currentScript;
export const getSelectedFrame = () => scriptEditorStore.get().selectedFrame;
export const isScriptLoading = () => scriptEditorStore.get().isLoading;
export const getCompilationError = () => scriptEditorStore.get().compilationError;
export const getLastCompiled = () => scriptEditorStore.get().lastCompiled;

// Actions
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