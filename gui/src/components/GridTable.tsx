import React, { useEffect, useRef } from 'react';
import { useStore } from '@nanostores/react';
import { GridCell } from './GridCell';
import { sceneStore, gridUIStore, updateGridSelection, playbackStore, addFrame, removeFrame, addLine, insertLineAfter, removeLine, resizeFrame } from '../stores/sceneStore';
import { useColorContext } from '../context/ColorContext';
import { Plus, Minus } from 'lucide-react';

export interface GridTableProps {
  cellWidth: number;
  cellHeight: number;
  containerWidth: number;
  containerHeight: number;
  client?: any; // BuboClient for sending operations
}

export const GridTable: React.FC<GridTableProps> = ({
  cellWidth,
  cellHeight,
  containerWidth,
  containerHeight,
  client
}) => {
  const scene = useStore(sceneStore);
  const gridUI = useStore(gridUIStore);
  const playback = useStore(playbackStore);
  const { palette } = useColorContext();
  const containerRef = useRef<HTMLDivElement>(null);

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

  const handleCellClick = (rowIndex: number, colIndex: number) => {
    updateGridSelection({
      start: [rowIndex, colIndex],
      end: [rowIndex, colIndex]
    });
  };

  const handleCellDoubleClick = (rowIndex: number, colIndex: number) => {
    // TODO: Open frame editor
    console.log('Edit frame:', rowIndex, colIndex);
  };

  const handleAddFrame = async (lineIndex: number) => {
    if (!client) {
      console.log('No client available for add frame operation');
      return;
    }
    
    const line = scene?.lines[lineIndex];
    if (!line) return;
    
    const frameIndex = line.frames.length; // Add at end
    const operation = addFrame(lineIndex, frameIndex);
    
    try {
      await client.sendMessage(operation);
      console.log('Add frame sent:', operation);
    } catch (error) {
      console.error('Failed to add frame:', error);
    }
  };

  const handleDeleteFrame = async (lineIndex: number, frameIndex: number) => {
    if (!client) {
      console.log('No client available for delete frame operation');
      return;
    }
    
    const operation = removeFrame(lineIndex, frameIndex);
    
    try {
      await client.sendMessage(operation);
      console.log('Delete frame sent:', operation);
    } catch (error) {
      console.error('Failed to delete frame:', error);
    }
  };

  const handleAddLine = async () => {
    if (!client) {
      console.log('No client available for add line operation');
      return;
    }
    
    const operation = addLine();
    if (!operation) return;
    
    try {
      await client.sendMessage(operation);
      console.log('Add line sent:', operation);
    } catch (error) {
      console.error('Failed to add line:', error);
    }
  };

  const handleInsertLineAfter = async (afterIndex: number) => {
    if (!client) {
      console.log('No client available for insert line operation');
      return;
    }
    
    const operation = insertLineAfter(afterIndex);
    if (!operation) return;
    
    try {
      await client.sendMessage(operation);
      console.log('Insert line after', afterIndex, 'sent:', operation);
    } catch (error) {
      console.error('Failed to insert line:', error);
    }
  };

  const handleDeleteLine = async (lineIndex: number) => {
    if (!client) {
      console.log('No client available for delete line operation');
      return;
    }
    
    if (scene && scene.lines.length <= 1) {
      console.log('Cannot delete last line');
      return;
    }
    
    const operation = removeLine(lineIndex);
    if (!operation) return;
    
    try {
      await client.sendMessage(operation);
      console.log('Delete line sent:', operation);
    } catch (error) {
      console.error('Failed to delete line:', error);
    }
  };

  const handleResizeFrame = async (lineIndex: number, frameIndex: number, newDuration: number) => {
    if (!client) {
      console.log('No client available for resize frame operation');
      return;
    }
    
    const operation = resizeFrame(lineIndex, frameIndex, newDuration);
    if (!operation) return;
    
    try {
      await client.sendMessage(operation);
      console.log('Resize frame sent:', operation);
    } catch (error) {
      console.error('Failed to resize frame:', error);
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
            progression={undefined} // TODO: Connect to progression calculation
            width={cellWidth}
            baseHeight={cellHeight}
            onClick={() => handleCellClick(row, col)}
            onDoubleClick={() => handleCellDoubleClick(row, col)}
            onDelete={() => handleDeleteFrame(col, row)}
            onResize={(newDuration) => handleResizeFrame(col, row, newDuration)}
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
      className="overflow-hidden border"
      style={{ 
        width: containerWidth, 
        height: containerHeight,
        backgroundColor: palette.background,
        borderColor: palette.border
      }}
    >
      {/* Column headers */}
      <div 
        className="flex border-b"
        style={{
          backgroundColor: palette.surface,
          borderColor: palette.border
        }}
      >
        {scene.lines.slice(0, visibleCols).map((line, index) => (
          <div
            key={index}
            className="relative flex items-center justify-center border-r text-xs font-medium group"
            style={{ 
              width: cellWidth, 
              height: 24,
              color: palette.text,
              borderColor: palette.border
            }}
          >
            {/* Delete line button (left side) */}
            {scene.lines.length > 1 && (
              <button
                className="absolute left-1 opacity-0 group-hover:opacity-100 transition-opacity w-4 h-4 flex items-center justify-center hover:bg-red-500 hover:text-white rounded-sm"
                onClick={() => handleDeleteLine(index)}
                title={`Delete line ${index}`}
              >
                <Minus size={10} />
              </button>
            )}
            
            {/* Line label */}
            <span>Line {index}</span>
            
            {/* Add line button (right side) */}
            <button
              className="absolute right-1 opacity-0 group-hover:opacity-100 transition-opacity w-4 h-4 flex items-center justify-center hover:bg-green-500 hover:text-white rounded-sm"
              onClick={() => handleInsertLineAfter(index)}
              title={`Insert new line after Line ${index}`}
            >
              <Plus size={10} />
            </button>
          </div>
        ))}
        
        {/* Add first line button if no lines exist */}
        {scene.lines.length === 0 && (
          <div
            className="flex items-center justify-center border-r text-xs font-medium cursor-pointer hover:bg-opacity-80"
            style={{ 
              width: cellWidth, 
              height: 24,
              color: palette.muted,
              borderColor: palette.border,
              backgroundColor: palette.surface
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