import { map } from 'nanostores';
import type { Frame } from '../../types/frame';
import { batchUpdateMap } from '../../utils/store-helpers';

// Drag State
export interface DraggedFrame {
  lineIndex: number;
  frameIndex: number;
  frameData: Frame;
}

export interface DropTarget {
  lineIndex: number;
  insertIndex: number;
}

export interface DragPreview {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface DragState {
  isDragging: boolean;
  draggedFrame: DraggedFrame | null;
  dropTarget: DropTarget | null;
  dragPreview: DragPreview | null;
  dragThreshold: number;
  dragStartPosition: { x: number; y: number } | null;
}

export const dragStore = map<DragState>({
  isDragging: false,
  draggedFrame: null,
  dropTarget: null,
  dragPreview: null,
  dragThreshold: 5,
  dragStartPosition: null,
});

export const startDrag = (
  lineIndex: number,
  frameIndex: number,
  frameData: DraggedFrame['frameData'],
  startPosition: { x: number; y: number }
) => {
  batchUpdateMap(dragStore, {
    isDragging: true,
    draggedFrame: {
      lineIndex,
      frameIndex,
      frameData,
    },
    dragStartPosition: startPosition,
  });
};

export const updateDragPreview = (preview: DragPreview) => {
  dragStore.setKey('dragPreview', preview);
};

export const setDropTarget = (target: DropTarget | null) => {
  dragStore.setKey('dropTarget', target);
};

export const clearDropTarget = () => {
  dragStore.setKey('dropTarget', null);
};

export const endDrag = () => {
  batchUpdateMap(dragStore, {
    isDragging: false,
    draggedFrame: null,
    dropTarget: null,
    dragPreview: null,
    dragStartPosition: null,
  });
};

export const getDragThreshold = () => dragStore.get().dragThreshold;

// Clipboard State
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
