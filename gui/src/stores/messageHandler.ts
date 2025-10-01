import { handleSceneMessage } from './scene/sceneData';
import { handlePlaybackMessage, handlePeerMessage, handleCompilationMessage, handleScriptEditorMessage } from './scene/sceneUI';
import { handleProjectMessage } from './project';
import { addLog } from './logs';
import type { ServerMessage } from '../types';

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

export const handleServerMessage = (message: ServerMessage): void => {
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

  if (typeof message === 'object' && message !== null) {
    handleSceneMessage(message);
    handlePlaybackMessage(message);
    handlePeerMessage(message);
    handleCompilationMessage(message);
    handleScriptEditorMessage(message);
    handleProjectMessage(message);
    handleLogMessage(message);
  }
};
