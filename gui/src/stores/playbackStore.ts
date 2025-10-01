import { map } from 'nanostores';
import type { ServerMessage } from '../types';

export interface PlaybackState {
  isPlaying: boolean;
  currentFramePositions: [number, number, number][]; // [line, frame, progression]
  clockState: [number, number, number, number]; // [tempo, beat, micros, quantum]
}

export const playbackStore = map<PlaybackState>({
  isPlaying: false,
  currentFramePositions: [],
  clockState: [0, 0, 0, 0],
});

// Playback message handlers
export const handlePlaybackMessage = (message: ServerMessage) => {
  // Handle string messages
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
  
  // Handle object messages
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

// Helper functions
export const isPlaying = () => playbackStore.get().isPlaying;
export const getCurrentFramePositions = () => playbackStore.get().currentFramePositions;
export const getClockState = () => playbackStore.get().clockState;