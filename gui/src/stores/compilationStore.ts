import { map } from 'nanostores';
import type { ServerMessage } from '../types';

export interface CompilationState {
  errors: Array<{ line_idx: number; frame_idx: number; error: string }>;
  compiledFrames: Set<string>; // "line_idx:frame_idx"
}

export const compilationStore = map<CompilationState>({
  errors: [],
  compiledFrames: new Set(),
});

// Compilation message handlers
export const handleCompilationMessage = (message: ServerMessage) => {
  if (typeof message === 'object' && message !== null) {
    switch (true) {
      case 'ScriptCompiled' in message:
        const { line_idx, frame_idx } = message.ScriptCompiled;
        const frameKey = `${line_idx}:${frame_idx}`;
        const compiled = new Set(compilationStore.get().compiledFrames);
        compiled.add(frameKey);
        compilationStore.setKey('compiledFrames', compiled);
        
        // Remove any errors for this frame
        const errors = compilationStore.get().errors.filter(
          err => !(err.line_idx === line_idx && err.frame_idx === frame_idx)
        );
        compilationStore.setKey('errors', errors);
        return true;
      
      case 'CompilationErrorOccurred' in message:
        const errorInfo = message.CompilationErrorOccurred;
        const newErrors = [...compilationStore.get().errors];
        newErrors.push({
          line_idx: 0, // Will need to parse from error info
          frame_idx: 0, // Will need to parse from error info
          error: errorInfo.info
        });
        compilationStore.setKey('errors', newErrors);
        return true;
    }
  }
  
  return false;
};

// Helper functions
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