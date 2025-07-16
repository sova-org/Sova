import React, { useRef, useState, useEffect } from 'react';
import { useStore } from '@nanostores/react';
import { GridTable } from './GridTable';
import { sceneStore, gridUIStore, updateGridSelection, getMaxFrames, addFrame, removeFrame, insertLineAfter, removeLine, setSceneLength } from '../stores/sceneStore';
import { globalVariablesStore, formatVariableValue } from '../stores/globalVariablesStore';
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
  const globalVariables = useStore(globalVariablesStore);
  const { palette } = useColorContext();

  const [cellWidth] = useState(140);
  const [cellHeight] = useState(80);
  const [renamingCell, setRenamingCell] = useState<[number, number] | null>(null); // [row, col]
  const [editingSceneLength, setEditingSceneLength] = useState(false);
  const [sceneLengthInput, setSceneLengthInput] = useState('');
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

  const handleSceneLengthSubmit = () => {
    if (!client || !scene) return;
    const newLength = parseInt(sceneLengthInput);
    if (isNaN(newLength) || newLength <= 0) return;

    const operation = setSceneLength(newLength, "AtSceneEnd");
    client.sendMessage(operation).catch(console.error);
    setEditingSceneLength(false);
  };

  const startEditingSceneLength = () => {
    if (!scene) return;
    setSceneLengthInput(scene.length.toString());
    setEditingSceneLength(true);
  };

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
        if (client && scene.lines[currentCol] && currentRow < scene.lines[currentCol].frames.length) {
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
        if (scene.lines[currentCol] && currentRow < scene.lines[currentCol].frames.length) {
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
    <div className="flex flex-col h-full">
      {/* Grid */}
      <div className="flex-1">
        <div
          ref={containerRef}
          className="h-full"
          style={{
            width,
            height: height - 68, // Account for bottom bar (36px) + header height (52px) - original header (36px) = 52px total
            backgroundColor: 'var(--color-background)'
          }}
          tabIndex={0}
          onKeyDown={handleKeyDown}
        >
          <GridTable
            cellWidth={cellWidth}
            cellHeight={cellHeight}
            containerWidth={width}
            containerHeight={height - 68} // Account for bottom bar (36px) + increased header height (52px) - original (36px)
            client={client}
            renamingCell={renamingCell}
            onRenameComplete={() => setRenamingCell(null)}
            onStartRename={(row, col) => setRenamingCell([row, col])}
          />
        </div>
      </div>

      {/* Bottom Status Bar */}
      <div
        className="border-t flex items-center justify-between px-3"
        style={{
          backgroundColor: 'var(--color-surface)',
          borderColor: 'var(--color-border)',
          fontSize: '12px',
          color: 'var(--color-text)',
          height: '41px' // h-10 (40px) + 1px
        }}
      >
        <div className="flex items-center space-x-2">
          <span style={{ color: 'var(--color-muted)' }}>Scene Length:</span>
          {editingSceneLength ? (
            <input
              type="number"
              value={sceneLengthInput}
              onChange={(e) => setSceneLengthInput(e.target.value)}
              onBlur={() => setEditingSceneLength(false)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') handleSceneLengthSubmit();
                if (e.key === 'Escape') setEditingSceneLength(false);
              }}
              className="w-12 px-1 bg-transparent border-b border-current outline-none"
              style={{ color: 'var(--color-text)', fontSize: '12px' }}
              autoFocus
              min="1"
            />
          ) : (
            <button
              onClick={startEditingSceneLength}
              className="hover:opacity-80 underline"
              style={{ color: 'var(--color-text)' }}
            >
              {scene.length}
            </button>
          )}
        </div>

        {/* Global Variables Display */}
        <div className="flex items-center space-x-4">
          {['A', 'B', 'C', 'D', 'W', 'X', 'Y', 'Z'].map(varName => {
            const value = globalVariables[varName];
            return (
              <div key={varName} className="flex items-center space-x-1">
                <span style={{ color: 'var(--color-primary)', fontWeight: 'bold' }}>{varName}:</span>
                <span style={{ color: 'var(--color-text)' }}>
                  {value ? formatVariableValue(value) : 'nil'}
                </span>
              </div>
            );
          })}
        </div>

        <div style={{ color: 'var(--color-muted)' }}>
          Grid {scene.lines.length} × {getMaxFrames(scene)}
        </div>
      </div>
    </div>
  );
};
