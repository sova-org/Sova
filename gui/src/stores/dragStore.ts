import { map } from 'nanostores';

export interface DraggedFrame {
  lineIndex: number;
  frameIndex: number;
  frameData: {
    duration: number;
    enabled: boolean;
    name: string | null;
    script: any;
    repetitions: number;
  };
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
  dragThreshold: 5, // pixels
  dragStartPosition: null,
});

// Actions
export const startDrag = (
  lineIndex: number,
  frameIndex: number,
  frameData: DraggedFrame['frameData'],
  startPosition: { x: number; y: number }
) => {
  dragStore.setKey('isDragging', true);
  dragStore.setKey('draggedFrame', {
    lineIndex,
    frameIndex,
    frameData,
  });
  dragStore.setKey('dragStartPosition', startPosition);
};

export const updateDragPreview = (preview: DragPreview) => {
  dragStore.setKey('dragPreview', preview);
};

export const setDropTarget = (target: DropTarget | null) => {
  dragStore.setKey('dropTarget', target);
};

export const endDrag = () => {
  dragStore.setKey('isDragging', false);
  dragStore.setKey('draggedFrame', null);
  dragStore.setKey('dropTarget', null);
  dragStore.setKey('dragPreview', null);
  dragStore.setKey('dragStartPosition', null);
};

export const getDragThreshold = () => dragStore.get().dragThreshold;