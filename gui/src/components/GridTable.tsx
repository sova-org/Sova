import React, { useEffect, useRef, useState } from 'react';
import { useStore } from '@nanostores/react';
import { GridCell } from './GridCell';
import { sceneStore, gridUIStore, updateGridSelection, playbackStore, addFrame, removeFrame, addLine, insertLineAfter, removeLine, resizeFrame, setFrameName, scriptEditorStore, setLineLength } from '../stores/sceneStore';
import { useColorContext } from '../context/ColorContext';
import { Plus, Minus } from 'lucide-react';

export interface GridTableProps {
  cellWidth: number;
  cellHeight: number;
  containerWidth: number;
  containerHeight: number;
  client?: any; // BuboClient for sending operations
  renamingCell?: [number, number] | null; // [row, col] 
  onRenameComplete?: () => void;
  onStartRename?: (row: number, col: number) => void;
}

export const GridTable: React.FC<GridTableProps> = ({
  cellWidth,
  cellHeight,
  containerWidth,
  containerHeight,
  client,
  renamingCell,
  onRenameComplete,
  onStartRename
}) => {
  const scene = useStore(sceneStore);
  const gridUI = useStore(gridUIStore);
  const playback = useStore(playbackStore);
  const { palette } = useColorContext();
  const containerRef = useRef<HTMLDivElement>(null);
  const [editingLineLength, setEditingLineLength] = useState<number | null>(null);
  const [lineLengthInput, setLineLengthInput] = useState('');

  if (!scene || scene.lines.length === 0) {
    return (
      <div 
        className="flex items-center justify-center h-full"
        style={{ color: palette.muted }}
      >
        No scene loaded
      </div>
    );
  }

  const maxFrames = Math.max(...scene.lines.map(line => line.frames.length));
  const visibleRows = Math.floor(containerHeight / cellHeight);
  const visibleCols = Math.floor(containerWidth / cellWidth);

  const handleCellClick = async (rowIndex: number, colIndex: number) => {
    updateGridSelection({
      start: [rowIndex, colIndex],
      end: [rowIndex, colIndex]
    });
    
    // Request the script for this frame
    if (client) {
      try {
        scriptEditorStore.setKey('isLoading', true);
        await client.sendMessage({ GetScript: [colIndex, rowIndex] });
      } catch (error) {
        console.error('Failed to get script:', error);
        scriptEditorStore.setKey('isLoading', false);
      }
    }
  };

  const handleCellDoubleClick = (rowIndex: number, colIndex: number) => {
    // TODO: Open frame editor
    // TODO: Open frame editor
  };

  const handleAddFrame = async (lineIndex: number) => {
    if (!client) {
      return;
    }
    
    const line = scene?.lines[lineIndex];
    if (!line) return;
    
    const frameIndex = line.frames.length; // Add at end
    const operation = addFrame(lineIndex, frameIndex);
    
    try {
      await client.sendMessage(operation);
    } catch (error) {
      console.error('Failed to add frame:', error);
    }
  };

  const handleDeleteFrame = async (lineIndex: number, frameIndex: number) => {
    if (!client) {
      return;
    }
    
    const operation = removeFrame(lineIndex, frameIndex);
    
    try {
      await client.sendMessage(operation);
    } catch (error) {
      console.error('Failed to delete frame:', error);
    }
  };

  const handleAddLine = async () => {
    if (!client) {
      return;
    }
    
    const operation = addLine();
    if (!operation) return;
    
    try {
      await client.sendMessage(operation);
    } catch (error) {
      console.error('Failed to add line:', error);
    }
  };

  const handleInsertLineAfter = async (afterIndex: number) => {
    if (!client) {
      return;
    }
    
    const operation = insertLineAfter(afterIndex);
    if (!operation) return;
    
    try {
      await client.sendMessage(operation);
    } catch (error) {
      console.error('Failed to insert line:', error);
    }
  };

  const handleDeleteLine = async (lineIndex: number) => {
    if (!client) {
      return;
    }
    
    if (scene && scene.lines.length <= 1) {
      return;
    }
    
    const operation = removeLine(lineIndex);
    if (!operation) return;
    
    try {
      await client.sendMessage(operation);
    } catch (error) {
      console.error('Failed to delete line:', error);
    }
  };

  const handleResizeFrame = async (lineIndex: number, frameIndex: number, newDuration: number) => {
    if (!client) {
      return;
    }
    
    const operation = resizeFrame(lineIndex, frameIndex, newDuration);
    if (!operation) return;
    
    try {
      await client.sendMessage(operation);
    } catch (error) {
      console.error('Failed to resize frame:', error);
    }
  };

  const handleNameChange = async (lineIndex: number, frameIndex: number, newName: string | null) => {
    if (!client) {
      return;
    }
    
    const operation = setFrameName(lineIndex, frameIndex, newName);
    
    try {
      await client.sendMessage(operation);
      if (onRenameComplete) {
        onRenameComplete();
      }
    } catch (error) {
      console.error('Failed to set frame name:', error);
    }
  };

  const isSelected = (rowIndex: number, colIndex: number): boolean => {
    const [[minRow, minCol], [maxRow, maxCol]] = [
      [Math.min(gridUI.selection.start[0], gridUI.selection.end[0]), Math.min(gridUI.selection.start[1], gridUI.selection.end[1])],
      [Math.max(gridUI.selection.start[0], gridUI.selection.end[0]), Math.max(gridUI.selection.start[1], gridUI.selection.end[1])]
    ];
    return rowIndex >= minRow && rowIndex <= maxRow && colIndex >= minCol && colIndex <= maxCol;
  };

  const isPlaying = (rowIndex: number, colIndex: number): boolean => {
    return playback.currentFramePositions.some(([line, frame]) => line === colIndex && frame === rowIndex);
  };

  const isRenaming = (rowIndex: number, colIndex: number): boolean => {
    return renamingCell !== null && renamingCell[0] === rowIndex && renamingCell[1] === colIndex;
  };

  const handleStartRename = (rowIndex: number, colIndex: number) => {
    if (onStartRename) {
      onStartRename(rowIndex, colIndex);
    }
  };

  const handleLineLengthSubmit = (lineIndex: number) => {
    if (!client) return;
    
    const newLength = lineLengthInput.trim() === '' ? null : parseFloat(lineLengthInput);
    if (newLength !== null && (isNaN(newLength) || newLength <= 0)) return;
    
    const operation = setLineLength(lineIndex, newLength);
    client.sendMessage(operation).catch(console.error);
    setEditingLineLength(null);
  };

  const startEditingLineLength = (lineIndex: number) => {
    if (!scene || lineIndex >= scene.lines.length) return;
    const line = scene.lines[lineIndex];
    setLineLengthInput(line.custom_length?.toString() || '');
    setEditingLineLength(lineIndex);
  };

  const renderGrid = () => {
    const columns = [];
    
    // Render each column (line) vertically
    for (let col = 0; col < Math.min(scene.lines.length, visibleCols); col++) {
      const line = scene.lines[col];
      const columnCells = [];
      
      // Render all frames in this column
      for (let row = 0; row < line.frames.length; row++) {
        columnCells.push(
          <GridCell
            key={`${row}-${col}`}
            line={line}
            frameIndex={row}
            isSelected={isSelected(row, col)}
            isPlaying={isPlaying(row, col)}
            isRenaming={isRenaming(row, col)}
            progression={undefined} // TODO: Connect to progression calculation
            width={cellWidth}
            baseHeight={cellHeight}
            onClick={() => handleCellClick(row, col)}
            onDoubleClick={() => handleCellDoubleClick(row, col)}
            onDelete={() => handleDeleteFrame(col, row)}
            onResize={(newDuration) => handleResizeFrame(col, row, newDuration)}
            onNameChange={(newName) => handleNameChange(col, row, newName)}
            onStartRename={() => handleStartRename(row, col)}
          />
        );
      }
      
      // Add the "add frame" button at the bottom of each column
      columnCells.push(
        <div
          key={`add-${col}`}
          className="border cursor-pointer flex items-center justify-center hover:bg-opacity-80 transition-colors"
          style={{
            width: cellWidth,
            height: cellHeight / 2, // Half height
            backgroundColor: palette.surface,
            borderColor: palette.border,
            color: palette.muted
          }}
          onClick={() => handleAddFrame(col)}
          title={`Add frame to line ${col}`}
        >
          <Plus size={16} />
        </div>
      );
      
      columns.push(
        <div key={col} className="flex flex-col">
          {columnCells}
        </div>
      );
    }
    
    return columns;
  };

  return (
    <div
      ref={containerRef}
      className="overflow-hidden"
      style={{ 
        width: containerWidth, 
        height: containerHeight,
        backgroundColor: 'var(--color-background)'
      }}
    >
      {/* Top spacer */}
      <div style={{ height: '16px', backgroundColor: 'var(--color-background)' }}></div>
      
      {/* Column headers */}
      <div 
        className="flex border-b"
        style={{
          backgroundColor: 'var(--color-surface)',
          borderColor: 'var(--color-border)'
        }}
      >
        {scene.lines.slice(0, visibleCols).map((line, index) => (
          <div
            key={index}
            className="relative flex flex-col border-r text-xs font-medium group"
            style={{ 
              width: cellWidth, 
              height: 36, // Increased height for two-line header
              color: 'var(--color-text)',
              borderColor: 'var(--color-border)'
            }}
          >
            {/* Top row: Line controls */}
            <div className="flex items-center justify-center h-4 relative">
              {/* Delete line button (left side) */}
              {scene.lines.length > 1 && (
                <button
                  className="absolute left-1 opacity-0 group-hover:opacity-100 transition-opacity w-3 h-3 flex items-center justify-center hover:bg-red-500 hover:text-white rounded-sm"
                  onClick={() => handleDeleteLine(index)}
                  title={`Delete line ${index}`}
                >
                  <Minus size={8} />
                </button>
              )}
              
              {/* Line label */}
              <span className="text-xs">Line {index}</span>
              
              {/* Add line button (right side) */}
              <button
                className="absolute right-1 opacity-0 group-hover:opacity-100 transition-opacity w-3 h-3 flex items-center justify-center hover:bg-green-500 hover:text-white rounded-sm"
                onClick={() => handleInsertLineAfter(index)}
                title={`Insert new line after Line ${index}`}
              >
                <Plus size={8} />
              </button>
            </div>

            {/* Bottom row: Line length */}
            <div className="flex items-center justify-center h-5 px-1">
              {editingLineLength === index ? (
                <input
                  type="number"
                  value={lineLengthInput}
                  onChange={(e) => setLineLengthInput(e.target.value)}
                  onBlur={() => setEditingLineLength(null)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') handleLineLengthSubmit(index);
                    if (e.key === 'Escape') setEditingLineLength(null);
                  }}
                  className="w-full px-1 text-xs text-center bg-transparent border border-current outline-none rounded"
                  style={{ 
                    color: 'var(--color-text)',
                    fontSize: '10px',
                    height: '16px'
                  }}
                  autoFocus
                  step="0.1"
                  placeholder="auto"
                />
              ) : (
                <button
                  onClick={() => startEditingLineLength(index)}
                  className="px-1 py-0 rounded hover:opacity-80 text-xs w-full"
                  style={{ 
                    backgroundColor: line.custom_length ? 'var(--color-primary)' : 'var(--color-muted)',
                    color: line.custom_length ? 'var(--color-surface)' : 'var(--color-background)',
                    fontSize: '10px',
                    height: '16px'
                  }}
                  title={`Line length: ${line.custom_length?.toFixed(1) || 'auto'} (click to edit)`}
                >
                  {line.custom_length?.toFixed(1) || 'auto'}
                </button>
              )}
            </div>
          </div>
        ))}
        
        {/* Add first line button if no lines exist */}
        {scene.lines.length === 0 && (
          <div
            className="flex items-center justify-center border-r text-xs font-medium cursor-pointer hover:bg-opacity-80"
            style={{ 
              width: cellWidth, 
              height: 36, // Match new header height
              color: 'var(--color-muted)',
              borderColor: 'var(--color-border)',
              backgroundColor: 'var(--color-surface)'
            }}
            onClick={handleAddLine}
          >
            <Plus size={12} /> Add Line
          </div>
        )}
      </div>

      {/* Grid body - columns laid out horizontally */}
      <div className="flex">
        {renderGrid()}
      </div>
    </div>
  );
};