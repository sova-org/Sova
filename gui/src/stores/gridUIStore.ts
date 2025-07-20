import { map } from 'nanostores';

export interface GridUIState {
  selection: {
    start: [number, number];  // [row, col]
    end: [number, number];    // [row, col]
  };
  scrollOffset: number;
  showHelp: boolean;
}

export const gridUIStore = map<GridUIState>({
  selection: { start: [0, 0], end: [0, 0] },
  scrollOffset: 0,
  showHelp: false
});

// Grid UI helpers
export const updateGridSelection = (selection: GridUIState['selection']) => {
  gridUIStore.setKey('selection', selection);
};

export const updateGridScrollOffset = (offset: number) => {
  gridUIStore.setKey('scrollOffset', offset);
};

export const toggleGridHelp = () => {
  gridUIStore.setKey('showHelp', !gridUIStore.get().showHelp);
};

// Utility functions
export const getGridSelectionBounds = (selection: GridUIState['selection']): [[number, number], [number, number]] => {
  const [startRow, startCol] = selection.start;
  const [endRow, endCol] = selection.end;
  
  return [
    [Math.min(startRow, endRow), Math.min(startCol, endCol)],
    [Math.max(startRow, endRow), Math.max(startCol, endCol)]
  ];
};

export const isGridSelectionSingle = (selection: GridUIState['selection']): boolean => {
  return selection.start[0] === selection.end[0] && selection.start[1] === selection.end[1];
};