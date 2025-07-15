import { map } from 'nanostores';

export interface ClipboardFrame {
  duration: number;
  enabled: boolean;
  name: string | null;
  script: string | null;
  repetitions: number;
}

export interface ClipboardState {
  hasContent: boolean;
  frameData: ClipboardFrame | null;
  sourceLineIndex: number | null;
  sourceFrameIndex: number | null;
}

export const clipboardStore = map<ClipboardState>({
  hasContent: false,
  frameData: null,
  sourceLineIndex: null,
  sourceFrameIndex: null,
});

// Actions
export const copyFrame = (
  lineIndex: number,
  frameIndex: number,
  frameData: ClipboardFrame
) => {
  clipboardStore.setKey('hasContent', true);
  clipboardStore.setKey('frameData', frameData);
  clipboardStore.setKey('sourceLineIndex', lineIndex);
  clipboardStore.setKey('sourceFrameIndex', frameIndex);
};

export const clearClipboard = () => {
  clipboardStore.setKey('hasContent', false);
  clipboardStore.setKey('frameData', null);
  clipboardStore.setKey('sourceLineIndex', null);
  clipboardStore.setKey('sourceFrameIndex', null);
};

export const getClipboardData = () => clipboardStore.get();