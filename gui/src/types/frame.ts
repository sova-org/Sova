import { Script } from "../types";

export interface Frame {
  duration: number;
  enabled: boolean;
  name: string | null;
  script: Script;
  repetitions: number;
}

export interface FramePosition {
  lineIdx: number;
  frameIdx: number;
}

export interface DraggedFrame extends Frame {
  position: FramePosition;
}

export function defaultFrame(): Frame {
  return {
    duration: 1.0,
    enabled: true,
    name: null,
    script: { content: '', lang: 'bali' },
    repetitions: 1
  };
}