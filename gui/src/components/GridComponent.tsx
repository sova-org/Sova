import React, { useRef, useState, useEffect } from 'react';
import { useStore } from '@nanostores/react';
import { GridTable } from './GridTable';
import { sceneStore, gridUIStore, updateGridSelection, getMaxFrames, addFrame, removeFrame, addLine, insertLineAfter, removeLine } from '../stores/sceneStore';
import { useColorContext } from '../context/ColorContext';

export interface GridComponentProps {
  width: number;
  height: number;
  client?: any; // BuboClient for sending operations
}

export const GridComponent: React.FC<GridComponentProps> = ({
  width,
  height,
  client
}) => {
  const scene = useStore(sceneStore);
  const gridUI = useStore(gridUIStore);
  const { palette } = useColorContext();
  const [cellWidth] = useState(140);
  const [cellHeight] = useState(80);
  const [renamingCell, setRenamingCell] = useState<[number, number] | null>(null); // [row, col]
  const containerRef = useRef<HTMLDivElement>(null);

  // Clear renaming state when selection changes
  useEffect(() => {
    if (renamingCell) {
      const [renamingRow, renamingCol] = renamingCell;
      const { selection } = gridUI;
      const [currentRow, currentCol] = selection.end;
      
      // If selection moved away from the renaming cell, cancel rename
      if (currentRow !== renamingRow || currentCol !== renamingCol) {
        setRenamingCell(null);
      }
    }
  }, [gridUI.selection, renamingCell]);

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (!scene) return;

    const { selection } = gridUI;
    const [currentRow, currentCol] = selection.end;
    const maxFrames = getMaxFrames(scene);
    const maxCols = scene.lines.length;

    let newRow = currentRow;
    let newCol = currentCol;
    let handled = false;

    switch (event.key) {
      case 'ArrowUp':
        newRow = Math.max(0, currentRow - 1);
        handled = true;
        break;
      case 'ArrowDown':
        newRow = Math.min(maxFrames - 1, currentRow + 1);
        handled = true;
        break;
      case 'ArrowLeft':
        newCol = Math.max(0, currentCol - 1);
        handled = true;
        break;
      case 'ArrowRight':
        newCol = Math.min(maxCols - 1, currentCol + 1);
        handled = true;
        break;
      case 'Escape':
        // Cancel renaming if active, otherwise reset selection to single cell
        if (renamingCell) {
          setRenamingCell(null);
        } else {
          updateGridSelection({
            start: [currentRow, currentCol],
            end: [currentRow, currentCol]
          });
        }
        handled = true;
        break;
      case 'Insert':
      case '+':
        // Add frame at current position
        if (client) {
          const operation = addFrame(currentCol, currentRow + 1);
          client.sendMessage(operation).catch(console.error);
          handled = true;
        }
        break;
      case 'Delete':
      case 'Backspace':
        // Delete frame at current position
        if (client && currentRow < scene.lines[currentCol]?.frames.length) {
          const operation = removeFrame(currentCol, currentRow);
          client.sendMessage(operation).catch(console.error);
          handled = true;
        }
        break;
      case 'l':
        // Insert line after current (when Ctrl+L)
        if (event.ctrlKey && client) {
          const operation = insertLineAfter(currentCol);
          if (operation) {
            client.sendMessage(operation).catch(console.error);
            handled = true;
          }
        }
        break;
      case 'L':
        // Delete line (when Ctrl+Shift+L)
        if (event.ctrlKey && event.shiftKey && client && scene.lines.length > 1) {
          const operation = removeLine(currentCol);
          if (operation) {
            client.sendMessage(operation).catch(console.error);
            handled = true;
          }
        }
        break;
      case 'r':
      case 'R':
        // Start renaming current frame
        if (currentRow < scene.lines[currentCol]?.frames.length) {
          setRenamingCell([currentRow, currentCol]);
          handled = true;
        }
        break;
    }

    if (handled) {
      event.preventDefault();
      
      // Clamp row to available frames in the selected column
      const line = scene.lines[newCol];
      if (line && newRow >= line.frames.length) {
        newRow = Math.max(0, line.frames.length - 1);
      }

      if (event.shiftKey) {
        // Extend selection
        updateGridSelection({
          start: selection.start,
          end: [newRow, newCol]
        });
      } else {
        // Move cursor
        updateGridSelection({
          start: [newRow, newCol],
          end: [newRow, newCol]
        });
      }
    }
  };

  if (!scene) {
    return (
      <div 
        className="flex items-center justify-center h-full"
        style={{ 
          backgroundColor: palette.background,
          color: palette.muted 
        }}
      >
        No scene loaded from server
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className="border"
      style={{ 
        width, 
        height,
        backgroundColor: palette.background,
        borderColor: palette.border
      }}
      tabIndex={0}
      onKeyDown={handleKeyDown}
    >
      {/* Grid */}
      <GridTable
        cellWidth={cellWidth}
        cellHeight={cellHeight}
        containerWidth={width}
        containerHeight={height}
        client={client}
        renamingCell={renamingCell}
        onRenameComplete={() => setRenamingCell(null)}
        onStartRename={(row, col) => setRenamingCell([row, col])}
      />
    </div>
  );
};