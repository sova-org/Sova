import { atom } from 'nanostores';
import type { ServerMessage } from '../types';

export interface RemoteLogEntry {
  timestamp: Date;
  level: 'info' | 'warn' | 'error' | 'debug';
  message: string;
}

export const remoteLogsStore = atom<RemoteLogEntry[]>([]);

// Helper function to add a log entry
export const addRemoteLog = (level: 'info' | 'warn' | 'error' | 'debug', message: string) => {
  const logEntry: RemoteLogEntry = {
    timestamp: new Date(),
    level,
    message
  };
  
  const currentLogs = remoteLogsStore.get();
  const newLogs = [...currentLogs, logEntry];
  
  // Keep only the last 1000 log entries
  if (newLogs.length > 1000) {
    newLogs.splice(0, newLogs.length - 1000);
  }
  
  remoteLogsStore.set(newLogs);
};

// Generic function to convert any server message to a readable log entry
const serverMessageToLog = (message: ServerMessage): { level: 'info' | 'warn' | 'error' | 'debug', message: string } | null => {
  // Handle direct LogString messages from server
  if (typeof message === 'object' && message !== null && 'LogString' in message) {
    return { level: 'info', message: message.LogString };
  }
  
  // Handle string messages
  if (typeof message === 'string') {
    switch (message) {
      case 'TransportStarted':
        return { level: 'info', message: 'Transport started' };
      case 'TransportStopped':
        return { level: 'info', message: 'Transport stopped' };
      case 'Success':
        return { level: 'debug', message: 'Operation completed successfully' };
      default:
        return { level: 'info', message: `Server: ${message}` };
    }
  }
  
  // Handle object messages - cover ALL possible server message types
  if (typeof message === 'object' && message !== null) {
    // Connection and session messages
    if ('Hello' in message) {
      const { username, peers, is_playing } = message.Hello;
      return { 
        level: 'info', 
        message: `Connected as ${username}. Peers: ${peers.length}. Transport: ${is_playing ? 'playing' : 'stopped'}` 
      };
    }
    
    if ('ConnectionRefused' in message) {
      return { level: 'error', message: `Connection refused: ${message.ConnectionRefused}` };
    }
    
    if ('InternalError' in message) {
      return { level: 'error', message: `Server error: ${message.InternalError}` };
    }
    
    // Scene and script messages
    if ('SceneValue' in message) {
      const scene = message.SceneValue;
      return { level: 'info', message: `Scene updated: ${scene.lines.length} lines, length ${scene.length}` };
    }
    
    if ('ScriptContent' in message) {
      const { line_idx, frame_idx, content } = message.ScriptContent;
      return { level: 'debug', message: `Script loaded: line ${line_idx}, frame ${frame_idx} (${content.length} chars)` };
    }
    
    if ('ScriptCompiled' in message) {
      const { line_idx, frame_idx } = message.ScriptCompiled;
      return { level: 'info', message: `✓ Script compiled: line ${line_idx}, frame ${frame_idx}` };
    }
    
    if ('CompilationErrorOccurred' in message) {
      const error = message.CompilationErrorOccurred;
      const location = error.from !== undefined && error.to !== undefined 
        ? ` (chars ${error.from}-${error.to})` 
        : '';
      return { level: 'error', message: `✗ Compilation error in ${error.lang}: ${error.info}${location}` };
    }
    
    // Transport and timing
    if ('ClockState' in message) {
      const [tempo, beat, micros, quantum] = message.ClockState;
      return { level: 'debug', message: `Clock: ${tempo.toFixed(1)} BPM, beat ${beat.toFixed(2)}, quantum ${quantum}` };
    }
    
    if ('FramePosition' in message) {
      const positions = message.FramePosition;
      return { level: 'debug', message: `Frame positions: ${positions.length} active lines` };
    }
    
    // Device messages
    if ('DeviceList' in message) {
      const devices = message.DeviceList;
      const connected = devices.filter(d => d.is_connected).length;
      return { level: 'info', message: `Devices updated: ${connected}/${devices.length} connected` };
    }
    
    // Peer collaboration
    if ('PeersUpdated' in message) {
      const peers = message.PeersUpdated;
      return { level: 'info', message: `Peers updated: ${peers.join(', ') || 'none'}` };
    }
    
    if ('PeerGridSelectionUpdate' in message) {
      const [peer, selection] = message.PeerGridSelectionUpdate;
      return { level: 'debug', message: `${peer} selected: (${selection.start[0]},${selection.start[1]}) to (${selection.end[0]},${selection.end[1]})` };
    }
    
    if ('PeerStartedEditing' in message) {
      const [peer, line_idx, frame_idx] = message.PeerStartedEditing;
      return { level: 'debug', message: `${peer} started editing line ${line_idx}, frame ${frame_idx}` };
    }
    
    if ('PeerStoppedEditing' in message) {
      const [peer, line_idx, frame_idx] = message.PeerStoppedEditing;
      return { level: 'debug', message: `${peer} stopped editing line ${line_idx}, frame ${frame_idx}` };
    }
    
    // Chat and communication
    if ('Chat' in message) {
      return { level: 'info', message: `💬 ${message.Chat}` };
    }
    
    // Scene length and structure
    if ('SceneLength' in message) {
      return { level: 'info', message: `Scene length: ${message.SceneLength}` };
    }
    
    // Snapshots and data
    if ('Snapshot' in message) {
      const { scene, tempo, beat } = message.Snapshot;
      return { level: 'info', message: `Snapshot: ${scene.lines.length} lines, ${tempo} BPM, beat ${beat.toFixed(2)}` };
    }
    
    // Global variables
    if ('GlobalVariablesUpdate' in message) {
      const vars = message.GlobalVariablesUpdate;
      const count = Object.keys(vars).length;
      return { level: 'debug', message: `Global variables updated: ${count} variables` };
    }
    
    // Fallback for any unhandled object message
    const messageType = Object.keys(message)[0];
    return { level: 'debug', message: `Server message: ${messageType}` };
  }
  
  return null;
};

export const handleRemoteLogMessage = (message: ServerMessage) => {
  const logData = serverMessageToLog(message);
  if (logData) {
    addRemoteLog(logData.level, logData.message);
  }
};

export const clearRemoteLogs = () => {
  remoteLogsStore.set([]);
};