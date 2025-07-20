import { map } from 'nanostores';
import type { Frame } from '../types/frame';
import { batchUpdateMap } from '../utils/store-helpers';

export type ClipboardFrame = Frame;

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
  batchUpdateMap(clipboardStore, {
    hasContent: true,
    frameData,
    sourceLineIndex: lineIndex,
    sourceFrameIndex: frameIndex,
  });
};

export const clearClipboard = () => {
  batchUpdateMap(clipboardStore, {
    hasContent: false,
    frameData: null,
    sourceLineIndex: null,
    sourceFrameIndex: null,
  });
};

export const getClipboardData = () => clipboardStore.get();