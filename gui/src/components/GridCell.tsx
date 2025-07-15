import React, { useState, useRef, useEffect } from 'react';
import { Line } from '../types';
import { useColorContext } from '../context/ColorContext';
import { X } from 'lucide-react';
import { useStore } from '@nanostores/react';
import { dragStore, startDrag, endDrag, updateDragPreview, getDragThreshold } from '../stores/dragStore';
import { clipboardStore } from '../stores/clipboardStore';

export interface GridCellProps {
  line: Line;
  frameIndex: number;
  isSelected: boolean;
  isPlaying: boolean;
  isRenaming?: boolean;
  progression?: number; // 0.0 to 1.0
  width: number;
  baseHeight: number; // Height for 1.0 beat frame
  onClick: () => void;
  onDoubleClick: () => void;
  onDelete?: () => void;
  onResize?: (newDuration: number) => void;
  onNameChange?: (newName: string | null) => void;
  onStartRename?: () => void;
  onRepetitionsChange?: (newRepetitions: number) => void;
  lineIndex: number; // Add line index for drag operations
}

export const GridCell: React.FC<GridCellProps> = ({
  line,
  frameIndex,
  isSelected,
  isPlaying,
  isRenaming = false,
  progression,
  width,
  baseHeight,
  onClick,
  onDoubleClick,
  onDelete,
  onResize,
  onNameChange,
  onStartRename,
  onRepetitionsChange,
  lineIndex
}) => {
  const { palette } = useColorContext();
  const dragState = useStore(dragStore);
  const clipboardState = useStore(clipboardStore);
  const frameValue = line.frames[frameIndex];
  const isEnabled = line.enabled_frames[frameIndex];
  const frameName = line.frame_names[frameIndex];
  const repetitions = line.frame_repetitions[frameIndex] || 1;
  
  const [isResizing, setIsResizing] = useState(false);
  const [resizeStartY, setResizeStartY] = useState(0);
  const [resizeStartValue, setResizeStartValue] = useState(0);
  const [currentResizeValue, setCurrentResizeValue] = useState(frameValue);
  
  // Keep currentResizeValue in sync with frameValue when not resizing
  useEffect(() => {
    if (!isResizing) {
      setCurrentResizeValue(frameValue);
    }
  }, [frameValue, isResizing]);
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState(frameValue.toFixed(2));
  const [editNameValue, setEditNameValue] = useState(frameName || '');
  const [isEditingRepetitions, setIsEditingRepetitions] = useState(false);
  const [editRepetitionsValue, setEditRepetitionsValue] = useState(repetitions.toString());
  const cellRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const nameInputRef = useRef<HTMLInputElement>(null);
  const repetitionsInputRef = useRef<HTMLInputElement>(null);
  
  // Drag state - simplified for Shift+Click approach
  
  // Calculate actual height based on duration (proportional to baseHeight)
  // Use currentResizeValue during resizing for immediate visual feedback
  const displayValue = isResizing ? currentResizeValue : frameValue;
  // Ensure minimum visual height of baseHeight (size for duration 1.0) for cosmetic reasons
  const actualHeight = Math.max(baseHeight, baseHeight * displayValue);

  // Focus input when entering edit mode
  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  // Focus name input when entering name edit mode
  useEffect(() => {
    if (isRenaming && nameInputRef.current) {
      nameInputRef.current.focus();
      nameInputRef.current.select();
      setEditNameValue(frameName || '');
    }
  }, [isRenaming, frameName]);

  // Focus repetitions input when entering edit mode
  useEffect(() => {
    if (isEditingRepetitions && repetitionsInputRef.current) {
      repetitionsInputRef.current.focus();
      repetitionsInputRef.current.select();
    }
  }, [isEditingRepetitions]);


  const getCellStyle = () => {
    // Check if this cell is being dragged
    const isDragged = dragState.isDragging && 
      dragState.draggedFrame?.lineIndex === lineIndex && 
      dragState.draggedFrame?.frameIndex === frameIndex;
    
    // Check if this cell is in the clipboard
    const isCopied = clipboardState.hasContent &&
      clipboardState.sourceLineIndex === lineIndex &&
      clipboardState.sourceFrameIndex === frameIndex;
    
    if (isDragged) {
      return {
        backgroundColor: palette.surface,
        color: palette.muted,
        opacity: 0.5
      };
    }
    
    if (isSelected && isPlaying) {
      return {
        backgroundColor: palette.secondary,
        color: palette.background,
        border: isCopied ? `2px dashed ${palette.background}` : undefined
      };
    }
    if (isSelected) {
      return {
        backgroundColor: palette.info,
        color: palette.background,
        border: isCopied ? `2px dashed ${palette.background}` : undefined
      };
    }
    if (isPlaying) {
      return {
        backgroundColor: palette.warning,
        color: palette.background,
        border: isCopied ? `2px dashed ${palette.background}` : undefined
      };
    }
    if (isEnabled) {
      return {
        backgroundColor: palette.success,
        color: palette.background,
        border: isCopied ? `2px dashed ${palette.background}` : undefined
      };
    }
    
    // Default style with copied indication
    return {
      backgroundColor: palette.surface,
      color: palette.muted,
      border: isCopied ? `2px dashed ${palette.primary}` : undefined
    };
  };

  const getProgressionStyle = () => {
    if (progression !== undefined && progression > 0) {
      return {
        background: `linear-gradient(to right, 
          rgba(255, 255, 255, 0.3) 0%, 
          rgba(255, 255, 255, 0.3) ${progression * 100}%, 
          transparent ${progression * 100}%)`
      };
    }
    return {};
  };

  const cellStyle = getCellStyle();

  const handleDeleteClick = (e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent cell selection
    if (onDelete) onDelete();
  };

  const handleResizeStart = (e: React.MouseEvent) => {
    e.stopPropagation();
    e.preventDefault();
    
    const startY = e.clientY;
    const startValue = frameValue;
    let latestValue = startValue; // Track the latest value in closure
    
    setIsResizing(true);
    setResizeStartY(startY);
    setResizeStartValue(startValue);
    setCurrentResizeValue(startValue);
    
    // Add global mouse event listeners
    const handleMouseMove = (moveEvent: MouseEvent) => {
      const deltaY = moveEvent.clientY - startY; // Down = increase (natural for growing height)
      const increment = moveEvent.shiftKey ? 0.01 : 0.1;
      const pixelsPerIncrement = 10; // 10px per increment for smooth feel
      const steps = Math.round(deltaY / pixelsPerIncrement);
      const newValue = startValue + (steps * increment);
      
      // Clamp between 0.1 and 8.0
      const clampedValue = Math.max(0.1, Math.min(8.0, newValue));
      latestValue = clampedValue; // Update closure variable
      
      // Update local state for visual feedback, but don't send to server yet
      setCurrentResizeValue(clampedValue);
    };
    
    const handleMouseUp = () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      
      setIsResizing(false);
      
      // Only send to server on mouse release if value changed
      if (onResize && Math.abs(latestValue - startValue) > 0.001) {
        onResize(latestValue);
      }
    };
    
    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
  };

  const handleValueClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!isResizing) {
      setIsEditing(true);
      setEditValue(frameValue.toFixed(2));
    }
  };

  const handleInputKeyDown = (e: React.KeyboardEvent) => {
    // Prevent all input events from bubbling up to grid keyboard handlers
    e.stopPropagation();
    
    if (e.key === 'Enter') {
      handleValueSubmit();
    } else if (e.key === 'Escape') {
      setIsEditing(false);
      setEditValue(frameValue.toFixed(2));
    }
  };

  const handleValueSubmit = () => {
    const newValue = parseFloat(editValue);
    if (!isNaN(newValue) && newValue >= 0.1 && newValue <= 8.0) {
      if (onResize && newValue !== frameValue) {
        onResize(newValue);
      }
    }
    setIsEditing(false);
  };

  const handleInputBlur = () => {
    handleValueSubmit();
  };

  const handleRepetitionsClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    if (!isResizing) {
      setIsEditingRepetitions(true);
      setEditRepetitionsValue(repetitions.toString());
    }
  };

  const handleRepetitionsKeyDown = (e: React.KeyboardEvent) => {
    e.stopPropagation();
    
    if (e.key === 'Enter') {
      handleRepetitionsSubmit();
    } else if (e.key === 'Escape') {
      setIsEditingRepetitions(false);
      setEditRepetitionsValue(repetitions.toString());
    }
  };

  const handleRepetitionsSubmit = () => {
    const newRepetitions = parseInt(editRepetitionsValue);
    if (!isNaN(newRepetitions) && newRepetitions >= 1 && newRepetitions <= 16) {
      if (onRepetitionsChange && newRepetitions !== repetitions) {
        onRepetitionsChange(newRepetitions);
      }
    }
    setIsEditingRepetitions(false);
  };

  const handleRepetitionsBlur = () => {
    handleRepetitionsSubmit();
  };

  const handleNameInputKeyDown = (e: React.KeyboardEvent) => {
    e.stopPropagation();
    
    if (e.key === 'Enter') {
      handleNameSubmit();
    } else if (e.key === 'Escape') {
      // Cancel rename - just call onNameChange with current name to trigger completion
      if (onNameChange) {
        onNameChange(frameName);
      }
    }
  };

  const handleNameSubmit = () => {
    const newName = editNameValue.trim() === '' ? null : editNameValue.trim();
    if (onNameChange) {
      onNameChange(newName);
    }
  };

  const handleNameInputBlur = () => {
    handleNameSubmit();
  };

  const handleNameDoubleClick = (e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent cell double-click from firing
    if (frameName && onStartRename) {
      // First ensure the cell is selected
      if (onClick) {
        onClick();
      }
      // Then trigger rename mode
      onStartRename();
    }
  };

  // Drag detection handlers - now using Shift+Click
  const handleMouseDown = (e: React.MouseEvent) => {
    // Only start drag if Shift is held down
    if (!e.shiftKey) {
      return;
    }

    // Don't start drag if we're clicking on buttons, inputs, or resize handles
    const target = e.target as HTMLElement;
    if (target.closest('button') || target.closest('input') || target.closest('[data-resize-handle]')) {
      return;
    }

    // Don't start drag if already dragging something else
    if (dragState.isDragging) {
      return;
    }

    // Don't start drag if we're editing or renaming
    if (isEditing || isRenaming || isEditingRepetitions) {
      return;
    }

    // Prevent text selection when shift-clicking
    e.preventDefault();
    e.stopPropagation();

    const startPos = { x: e.clientX, y: e.clientY };
    
    // For drag operations, we'll get the script content during the actual move operation
    // This is just temporary data for the drag preview
    const frameData = {
      duration: frameValue,
      enabled: isEnabled,
      name: frameName,
      script: null, // Will be fetched during actual drop operation
      repetitions: repetitions,
    };

    // Start drag immediately on Shift+Click
    startDrag(lineIndex, frameIndex, frameData, startPos);
  };

  // Regular click handler
  const handleCellClick = (e: React.MouseEvent) => {
    if (dragState.isDragging) {
      e.preventDefault();
      e.stopPropagation();
      return;
    }
    onClick();
  };

  return (
    <div
      ref={cellRef}
      className="relative border cursor-pointer flex flex-col justify-between p-2 text-xs hover:opacity-80 transition-opacity group select-none"
      style={{
        width: `${width}px`,
        height: `${actualHeight}px`,
        backgroundColor: cellStyle.backgroundColor,
        color: cellStyle.color,
        borderColor: palette.border,
        opacity: cellStyle.opacity,
        userSelect: 'none',
        border: cellStyle.border || `1px solid ${palette.border}`,
        ...getProgressionStyle()
      }}
      onClick={handleCellClick}
      onMouseDown={handleMouseDown}
      onDoubleClick={onDoubleClick}
      title={`${frameName || 'Frame'} - Duration: ${frameValue.toFixed(2)}s${repetitions > 1 ? ` × ${repetitions}` : ''}\nShift+Click to drag\nCtrl+C to copy, Ctrl+V to paste`}
    >
      {/* Top row - play marker and delete button */}
      <div className="flex justify-between items-start h-4">
        <span className="text-xs opacity-60">
          {isPlaying ? '▶' : ' '}
        </span>
        {onDelete && (
          <button
            className="opacity-0 group-hover:opacity-100 transition-opacity w-4 h-4 flex items-center justify-center hover:bg-red-500 hover:text-white"
            onClick={handleDeleteClick}
            title="Delete frame"
          >
            <X size={10} />
          </button>
        )}
      </div>

      {/* Center - frame name prominently displayed */}
      <div className="flex-1 flex items-center justify-center px-1 min-h-[20px]">
        {isRenaming ? (
          <input
            ref={nameInputRef}
            type="text"
            value={editNameValue}
            onChange={(e) => setEditNameValue(e.target.value)}
            onKeyDown={handleNameInputKeyDown}
            onKeyUp={(e) => e.stopPropagation()}
            onKeyPress={(e) => e.stopPropagation()}
            onBlur={handleNameInputBlur}
            className="text-sm font-semibold text-center w-full px-1"
            style={{
              backgroundColor: palette.background,
              color: palette.text,
              border: `1px solid ${palette.primary}`
            }}
            placeholder="Frame name"
          />
        ) : (
          <span 
            className="text-sm font-semibold text-center truncate leading-tight px-1 py-1 cursor-pointer"
            title={frameName ? `${frameName} (double-click or press 'r' to rename)` : 'Select and press \'r\' to name this frame'}
            onDoubleClick={handleNameDoubleClick}
          >
            {frameName || '∅'}
          </span>
        )}
      </div>

      {/* Bottom row - duration and repetitions */}
      <div className="flex justify-between items-end h-5 gap-1">
        {/* Duration field */}
        <div className="flex-1">
          {isEditing ? (
            <input
              ref={inputRef}
              type="number"
              min="0.1"
              max="8.0"
              step="0.01"
              value={editValue}
              onChange={(e) => setEditValue(e.target.value)}
              onKeyDown={handleInputKeyDown}
              onKeyUp={(e) => e.stopPropagation()}
              onKeyPress={(e) => e.stopPropagation()}
              onBlur={handleInputBlur}
              className="text-xs px-1 w-full text-center"
              style={{
                backgroundColor: palette.background,
                color: palette.text,
                border: `1px solid ${palette.primary}`
              }}
            />
          ) : (
            <span 
              className="text-xs px-1 cursor-pointer hover:bg-opacity-80 transition-colors block text-center"
              style={{
                backgroundColor: palette.background,
                color: palette.text,
                border: '1px solid transparent'
              }}
              onClick={handleValueClick}
              title="Click to edit duration"
            >
              {displayValue.toFixed(2)}
            </span>
          )}
        </div>

        {/* Repetitions field */}
        <div className="flex-shrink-0">
          {isEditingRepetitions ? (
            <input
              ref={repetitionsInputRef}
              type="number"
              min="1"
              max="16"
              step="1"
              value={editRepetitionsValue}
              onChange={(e) => setEditRepetitionsValue(e.target.value)}
              onKeyDown={handleRepetitionsKeyDown}
              onKeyUp={(e) => e.stopPropagation()}
              onKeyPress={(e) => e.stopPropagation()}
              onBlur={handleRepetitionsBlur}
              className="text-xs px-1 w-8 text-center"
              style={{
                backgroundColor: palette.background,
                color: palette.text,
                border: `1px solid ${palette.primary}`
              }}
            />
          ) : (
            <span 
              className="text-xs px-1 cursor-pointer hover:bg-opacity-80 transition-colors"
              style={{
                backgroundColor: repetitions > 1 ? palette.warning : palette.background,
                color: repetitions > 1 ? palette.background : palette.text,
                border: '1px solid transparent',
                fontWeight: repetitions > 1 ? 'bold' : 'normal'
              }}
              onClick={handleRepetitionsClick}
              title="Click to edit repetitions"
            >
              ×{repetitions}
            </span>
          )}
        </div>
      </div>

      {/* Progress bar overlay */}
      {progression !== undefined && progression > 0 && (
        <div 
          className="absolute bottom-0 left-0 h-1"
          style={{ 
            width: `${progression * 100}%`,
            backgroundColor: palette.error
          }}
        />
      )}

      {/* Resize handle (bottom border) */}
      {onResize && (
        <div
          data-resize-handle="true"
          className="absolute bottom-0 left-0 right-0 h-1 opacity-0 group-hover:opacity-100 transition-opacity cursor-ns-resize"
          style={{
            backgroundColor: palette.primary,
            borderBottom: `2px solid ${palette.primary}`
          }}
          onMouseDown={handleResizeStart}
          title="Drag to resize frame (Shift for fine control)"
        />
      )}

      {/* Resizing indicator */}
      {isResizing && (
        <div 
          className="absolute inset-0 border-2 pointer-events-none"
          style={{
            borderColor: palette.primary,
            backgroundColor: `${palette.primary}20`
          }}
        />
      )}
    </div>
  );
};