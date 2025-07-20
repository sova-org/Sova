export interface Frame {
  duration: number;
  enabled: boolean;
  name: string | null;
  script: string | null;
  repetitions: number;
  lang?: string;
}

export interface FramePosition {
  lineIdx: number;
  frameIdx: number;
}

export interface DraggedFrame extends Frame {
  position: FramePosition;
}

export interface PastedFrameData {
  length: number;
  is_enabled: boolean;
  script_content: string | undefined;
  name: string | undefined;
  repetitions: number | undefined;
}

// Helper to convert Frame to PastedFrameData
export function frameTopastedData(frame: Frame): PastedFrameData {
  return {
    length: frame.duration,
    is_enabled: frame.enabled,
    script_content: frame.script || undefined,
    name: frame.name || undefined,
    repetitions: frame.repetitions || undefined,
  };
}

// Helper to convert PastedFrameData to Frame
export function pastedDataToFrame(data: PastedFrameData): Frame {
  return {
    duration: data.length,
    enabled: data.is_enabled,
    name: data.name || null,
    script: data.script_content || null,
    repetitions: data.repetitions || 1,
  };
}