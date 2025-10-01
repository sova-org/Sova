import React, { useEffect, useRef, useState } from 'react';
import { useStore } from '@nanostores/react';
import { GridCell } from './GridCell';
import { DragOverlay } from './DragOverlay';
import { DropZone } from './DropZone';
import { sceneStore } from '../../stores/scene/sceneData';
import { gridUIStore, updateGridSelection, playbackStore, scriptEditorStore } from '../../stores/scene/sceneUI';
import { addFrame, removeFrame, addLine, insertLineAfter, removeLine, resizeFrame, setFrameName, setLineLength, setScript, enableFrames, disableFrames, setFrameRepetitions } from '../../stores/scene/sceneOperations';
import { dragStore, endDrag, clipboardStore, copyFrame } from '../../stores/interaction/gridInteraction';
import { useColorContext } from '../../context/ColorContext';
import { Plus, Minus } from 'lucide-react';
import { Frame } from '../../types';

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
  const dragState = useStore(dragStore);
  const clipboardState = useStore(clipboardStore);
  const { palette } = useColorContext();
  const containerRef = useRef<HTMLDivElement>(null);
  const headerRef = useRef<HTMLDivElement>(null);
  const bodyRef = useRef<HTMLDivElement>(null);
  const [editingLineLength, setEditingLineLength] = useState<number | null>(null);
  const [lineLengthInput, setLineLengthInput] = useState('');
  const [isOperationInProgress, setIsOperationInProgress] = useState(false);

  // Synchronize horizontal scrolling between header and body
  useEffect(() => {
    const handleBodyScroll = () => {
      if (headerRef.current && bodyRef.current) {
        headerRef.current.scrollLeft = bodyRef.current.scrollLeft;
      }
    };

    const handleHeaderScroll = () => {
      if (headerRef.current && bodyRef.current) {
        bodyRef.current.scrollLeft = headerRef.current.scrollLeft;
      }
    };

    const body = bodyRef.current;
    const header = headerRef.current;

    if (body) body.addEventListener('scroll', handleBodyScroll);
    if (header) header.addEventListener('scroll', handleHeaderScroll);

    return () => {
      if (body) body.removeEventListener('scroll', handleBodyScroll);
      if (header) header.removeEventListener('scroll', handleHeaderScroll);
    };
  }, []);

  // Handle global mouse up for drag operations
  useEffect(() => {
    const handleMouseUp = async () => {
      if (dragState.isDragging && dragState.draggedFrame && dragState.dropTarget) {
        const { draggedFrame, dropTarget } = dragState;
        
        try {
          // Perform the move operation
          await handleFrameMove(
            draggedFrame.lineIndex,
            draggedFrame.frameIndex,
            dropTarget.lineIndex,
            dropTarget.insertIndex
          );
        } catch (error) {
          console.error('Failed to move frame:', error);
        }
      }
      
      if (dragState.isDragging) {
        endDrag();
      }
    };

    document.addEventListener('mouseup', handleMouseUp);
    return () => document.removeEventListener('mouseup', handleMouseUp);
  }, [dragState.isDragging, dragState.draggedFrame, dragState.dropTarget]);

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Don't handle shortcuts if we're typing in an input field
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }

      const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
      const ctrlOrCmd = isMac ? e.metaKey : e.ctrlKey;

      if (e.key === 'Escape' && dragState.isDragging) {
        endDrag();
        return;
      }

      if (ctrlOrCmd && e.key === 'c' && !e.shiftKey) {
        e.preventDefault();
        if (!isOperationInProgress) {
          handleCopy();
        }
        return;
      }

      if (ctrlOrCmd && e.key === 'v' && !e.shiftKey) {
        e.preventDefault();
        if (!isOperationInProgress) {
          handlePaste();
        }
        return;
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [dragState.isDragging, scene, gridUI.selection, clipboardState.hasContent, isOperationInProgress]);

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

  // const _maxFrames = Math.max(...scene.lines.map(line => line.frames.length));
  const headerHeight = 52; // Height of column headers

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

  const handleCellDoubleClick = (_rowIndex: number, _colIndex: number) => {
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

  const handleRepetitionsChange = async (lineIndex: number, frameIndex: number, newRepetitions: number) => {
    if (!client) {
      return;
    }

    const operation = setFrameRepetitions(lineIndex, frameIndex, newRepetitions);

    try {
      await client.sendMessage(operation);
    } catch (error) {
      console.error('Failed to set frame repetitions:', error);
    }
  };

  const handleFrameMove = async (
    sourceLineIndex: number,
    sourceFrameIndex: number,
    targetLineIndex: number,
    targetInsertIndex: number
  ) => {
    if (!client || !scene) return;

    const sourceLine = scene.lines[sourceLineIndex];
    const targetLine = scene.lines[targetLineIndex];
    
    if (!sourceLine || sourceFrameIndex >= sourceLine.frames.length) return;
    if (!targetLine || targetLineIndex >= scene.lines.length) return;

    // Don't move to the same position
    if (sourceLineIndex === targetLineIndex && 
        (sourceFrameIndex === targetInsertIndex || sourceFrameIndex === targetInsertIndex - 1)) {
      return;
    }

    // Validate insert index
    if (targetInsertIndex < 0 || targetInsertIndex > targetLine.frames.length) {
      return;
    }

    try {
      // First, get the script content from the server
      await client.sendMessage({ GetScript: [sourceLineIndex, sourceFrameIndex] });
      
      // Wait for script content to be received
      await new Promise(resolve => setTimeout(resolve, 100));

      // Extract frame data
      const frameData = sourceLine.frames[sourceFrameIndex] as Frame;

      // Step 1: Insert the frame at the target position
      const insertOperation = addFrame(targetLineIndex, targetInsertIndex);
      await client.sendMessage(insertOperation);

      // Step 2: Update the inserted frame with the correct data
      const resizeOperation = resizeFrame(targetLineIndex, targetInsertIndex, frameData.duration ?? 1);
      await client.sendMessage(resizeOperation);

      // Step 3: Set frame name if it exists
      if (frameData.name) {
        const nameOperation = setFrameName(targetLineIndex, targetInsertIndex, frameData.name);
        await client.sendMessage(nameOperation);
      }

      // Step 4: Set script content if it exists
      if (frameData.script) {
        const scriptOperation = setScript(targetLineIndex, targetInsertIndex, frameData.script.content);
        await client.sendMessage(scriptOperation);
      }

      // Step 5: Set enabled/disabled state
      if (frameData?.enabled) {
        const enableOperation = enableFrames(targetLineIndex, [targetInsertIndex]);
        await client.sendMessage(enableOperation);
      } else {
        const disableOperation = disableFrames(targetLineIndex, [targetInsertIndex]);
        await client.sendMessage(disableOperation);
      }

      // Step 6: Set repetitions if not 1
      if (frameData?.repetitions !== 1) {
        const repetitionOperation = setFrameRepetitions(targetLineIndex, targetInsertIndex, frameData.repetitions);
        await client.sendMessage(repetitionOperation);
      }

      // Step 7: Remove the original frame
      // Adjust the source index if we inserted in the same line before the source
      const adjustedSourceIndex = 
        sourceLineIndex === targetLineIndex && targetInsertIndex <= sourceFrameIndex
          ? sourceFrameIndex + 1
          : sourceFrameIndex;

      const removeOperation = removeFrame(sourceLineIndex, adjustedSourceIndex);
      await client.sendMessage(removeOperation);

    } catch (error) {
      console.error('Failed to move frame:', error);
      throw error;
    }
  };

  const handleCopy = async () => {
    if (!client || !scene || isOperationInProgress) return;

    const [rowIndex, colIndex] = gridUI.selection.start;
    const line = scene.lines[colIndex];
    
    if (!line || rowIndex >= line.frames.length) return;

    try {
      setIsOperationInProgress(true);
      
      // Clear any existing script editor state
      scriptEditorStore.setKey('currentScript', '');
      
      // Get the script content from the server for the selected frame
      await client.sendMessage({ GetScript: [colIndex, rowIndex] });
      
      // Wait for the script content to be received and check multiple times
      let attempts = 0;
      let scriptContent = '';
      while (attempts < 10) {
        await new Promise(resolve => setTimeout(resolve, 50));
        const currentScript = scriptEditorStore.get().currentScript;
        if (currentScript !== '') {
          scriptContent = currentScript;
          break;
        }
        attempts++;
      }
      
      let frameData = line.frames[rowIndex] as Frame;
      frameData.script.content = scriptContent;

      console.log('Copying frame data:', frameData);
      copyFrame(colIndex, rowIndex, frameData);
    } catch (error) {
      console.error('Failed to copy frame:', error);
    } finally {
      setIsOperationInProgress(false);
    }
  };

  const handlePaste = async () => {
    if (!client || !scene || !clipboardState.hasContent || !clipboardState.frameData || isOperationInProgress) return;

    const [rowIndex, colIndex] = gridUI.selection.start;
    const line = scene.lines[colIndex];
    
    if (!line || colIndex >= scene.lines.length) return;

    // Insert after the currently selected frame
    const insertIndex = rowIndex + 1;
    const frameData = clipboardState.frameData;

    try {
      setIsOperationInProgress(true);
      
      console.log('Pasting frame data:', frameData);
      
      // Step 1: Insert the frame with the duration directly in the operation
      const insertOperation = {
        InsertFrame: [colIndex, insertIndex, frameData.duration, "Immediate"]
      };
      await client.sendMessage(insertOperation);
      
      // Wait for the frame to be created
      await new Promise(resolve => setTimeout(resolve, 100));

      // Step 2: Set frame name if it exists
      if (frameData.name) {
        const nameOperation = setFrameName(colIndex, insertIndex, frameData.name);
        await client.sendMessage(nameOperation);
        await new Promise(resolve => setTimeout(resolve, 75));
      }

      // Step 3: Set repetitions if not 1
      if (frameData.repetitions !== 1) {
        const repetitionOperation = setFrameRepetitions(colIndex, insertIndex, frameData.repetitions);
        await client.sendMessage(repetitionOperation);
        await new Promise(resolve => setTimeout(resolve, 75));
      }

      // Step 4: Set enabled/disabled state (separate operations work better)
      if (!frameData.enabled) {
        const disableOperation = disableFrames(colIndex, [insertIndex]);
        await client.sendMessage(disableOperation);
        await new Promise(resolve => setTimeout(resolve, 75));
      }

      // Step 5: Set script content if it exists (do this last as it may trigger compilation)
      if (frameData.script && frameData.script.content.trim() !== '') {
        const scriptOperation = setScript(colIndex, insertIndex, frameData.script.content);
        await client.sendMessage(scriptOperation);
        await new Promise(resolve => setTimeout(resolve, 150)); // Longer wait for script compilation
      }

      // Step 6: Select the newly pasted frame
      updateGridSelection({
        start: [insertIndex, colIndex],
        end: [insertIndex, colIndex]
      });

    } catch (error) {
      console.error('Failed to paste frame:', error);
    } finally {
      setIsOperationInProgress(false);
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
    return renamingCell !== null && renamingCell !== undefined && renamingCell[0] === rowIndex && renamingCell[1] === colIndex;
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

    const operation = setLineLength(lineIndex, newLength, { EndOfLine: 0 });
    client.sendMessage(operation).catch(console.error);
    setEditingLineLength(null);
  };

  const startEditingLineLength = (lineIndex: number) => {
    if (!scene || lineIndex >= scene.lines.length) return;
    const line = scene.lines[lineIndex];
    if (line) {
      setLineLengthInput(line.custom_length?.toString() || '');
    }
    setEditingLineLength(lineIndex);
  };

  const renderGrid = () => {
    const columns = [];

    // Render each column (line) vertically - render all columns, not just visible ones
    for (let col = 0; col < scene.lines.length; col++) {
      const line = scene.lines[col];
      if (!line) continue;
      
      const columnCells = [];

      // Add drop zone at the beginning of the column
      if (dragState.isDragging) {
        columnCells.push(
          <div key={`dropzone-${col}-0`} className="relative" style={{ height: '8px' }}>
            <DropZone
              lineIndex={col}
              insertIndex={0}
              isHorizontal={true}
              width={cellWidth}
              height={8}
            />
          </div>
        );
      }

      // Render all frames in this column
      for (let row = 0; row < line.frames.length; row++) {
        columnCells.push(
          <GridCell
            key={`${row}-${col}`}
            line={line}
            frameIndex={row}
            lineIndex={col}
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
            onRepetitionsChange={(newRepetitions) => handleRepetitionsChange(col, row, newRepetitions)}
          />
        );

        // Add drop zone after each frame (except for the dragged frame)
        if (dragState.isDragging && !(dragState.draggedFrame?.lineIndex === col && dragState.draggedFrame?.frameIndex === row)) {
          columnCells.push(
            <div key={`dropzone-${col}-${row + 1}`} className="relative" style={{ height: '8px' }}>
              <DropZone
                lineIndex={col}
                insertIndex={row + 1}
                isHorizontal={true}
                width={cellWidth}
                height={8}
              />
            </div>
          );
        }
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
      className="flex flex-col"
      style={{
        width: containerWidth,
        height: containerHeight,
        backgroundColor: 'var(--color-background)'
      }}
    >
      {/* Headers container - scrolls horizontally only */}
      <div 
        ref={headerRef}
        className="overflow-x-auto overflow-y-hidden flex-shrink-0"
        style={{
          height: headerHeight,
          backgroundColor: 'var(--color-surface)'
        }}
      >
        <div
          className="flex border-b"
          style={{
            minWidth: 'max-content',
            backgroundColor: 'var(--color-surface)',
            borderColor: 'var(--color-border)',
            height: '100%'
          }}
        >
          {scene.lines.map((line, index) => (
          <div
            key={index}
            className="relative flex flex-col border-r text-xs font-medium group"
            style={{
              width: cellWidth,
              minWidth: cellWidth,
              height: 52, // Increased height for two-line header + padding
              color: 'var(--color-text)',
              borderColor: 'var(--color-border)',
              padding: '8px',
              boxSizing: 'border-box',
              fontFamily: 'inherit'
            }}
          >
            {/* Top row: Line controls */}
            <div className="flex items-center justify-center h-4 relative">
              {/* Delete line button (left side) */}
              {scene.lines.length > 1 && (
                <button
                  className="absolute left-1 opacity-0 group-hover:opacity-100 transition-opacity w-3 h-3 flex items-center justify-center hover:bg-red-500 hover:text-white"
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
                className="absolute right-1 opacity-0 group-hover:opacity-100 transition-opacity w-3 h-3 flex items-center justify-center hover:bg-green-500 hover:text-white"
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
                    e.stopPropagation();
                    if (e.key === 'Enter') handleLineLengthSubmit(index);
                    if (e.key === 'Escape') setEditingLineLength(null);
                  }}
                  className="w-full px-1 text-xs text-center bg-transparent border border-current outline-none"
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
                  className="px-1 py-0 hover:opacity-80 text-xs w-full"
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
              minWidth: cellWidth,
              height: 52, // Match new header height
              color: 'var(--color-muted)',
              borderColor: 'var(--color-border)',
              backgroundColor: 'var(--color-surface)',
              boxSizing: 'border-box'
            }}
            onClick={handleAddLine}
          >
            <Plus size={12} /> Add Line
          </div>
        )}
        </div>
      </div>

      {/* Grid body - scrolls both horizontally and vertically */}
      <div 
        ref={bodyRef}
        className="overflow-auto flex-1"
        style={{ 
          backgroundColor: 'var(--color-background)',
          height: `calc(100% - ${headerHeight}px)`
        }}
      >
        <div className="flex" style={{ minWidth: 'max-content' }}>
          {renderGrid()}
        </div>
      </div>
      
      {/* Drag overlay */}
      <DragOverlay />
    </div>
  );
};
