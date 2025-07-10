import React, { useState, useRef, useEffect } from 'react';
import { Line } from '../types';
import { useColorContext } from '../context/ColorContext';
import { X } from 'lucide-react';

export interface GridCellProps {
  line: Line;
  frameIndex: number;
  isSelected: boolean;
  isPlaying: boolean;
  progression?: number; // 0.0 to 1.0
  width: number;
  baseHeight: number; // Height for 1.0 beat frame
  onClick: () => void;
  onDoubleClick: () => void;
  onDelete?: () => void;
  onResize?: (newDuration: number) => void;
}

export const GridCell: React.FC<GridCellProps> = ({
  line,
  frameIndex,
  isSelected,
  isPlaying,
  progression,
  width,
  baseHeight,
  onClick,
  onDoubleClick,
  onDelete,
  onResize
}) => {
  const { palette } = useColorContext();
  const frameValue = line.frames[frameIndex];
  const isEnabled = line.enabled_frames[frameIndex];
  const frameName = line.frame_names[frameIndex];
  const repetitions = line.frame_repetitions[frameIndex] || 1;
  
  const [isResizing, setIsResizing] = useState(false);
  const [resizeStartY, setResizeStartY] = useState(0);
  const [resizeStartValue, setResizeStartValue] = useState(0);
  const [currentResizeValue, setCurrentResizeValue] = useState(frameValue);
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState(frameValue.toFixed(2));
  const cellRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  
  // Calculate actual height based on duration (proportional to baseHeight)
  // Use currentResizeValue during resizing for immediate visual feedback
  const displayValue = isResizing ? currentResizeValue : frameValue;
  // Ensure minimum height of baseHeight * 0.3 for visibility
  const actualHeight = Math.max(baseHeight * 0.3, baseHeight * displayValue);

  // Focus input when entering edit mode
  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  const getCellStyle = () => {
    if (isSelected) {
      return {
        backgroundColor: palette.primary,
        color: palette.background
      };
    }
    if (isPlaying) {
      return {
        backgroundColor: palette.warning,
        color: palette.background
      };
    }
    if (isEnabled) {
      return {
        backgroundColor: palette.success,
        color: palette.background
      };
    }
    return {
      backgroundColor: palette.surface,
      color: palette.muted
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
      
      // Update local state for visual feedback, but don't send to server yet
      setCurrentResizeValue(clampedValue);
    };
    
    const handleMouseUp = () => {
      setIsResizing(false);
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      
      // Only send to server on mouse release
      if (onResize && currentResizeValue !== startValue) {
        onResize(currentResizeValue);
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

  return (
    <div
      ref={cellRef}
      className="relative border cursor-pointer flex flex-col justify-between p-1 text-xs hover:opacity-80 transition-opacity group"
      style={{
        width: `${width}px`,
        height: `${actualHeight}px`,
        backgroundColor: cellStyle.backgroundColor,
        color: cellStyle.color,
        borderColor: palette.border,
        ...getProgressionStyle()
      }}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
    >
      {/* Top row - play marker, name, and delete button */}
      <div className="flex justify-between items-start h-4 overflow-hidden">
        <span className="text-xs opacity-60">
          {isPlaying ? '▶' : ' '}
        </span>
        <span className="text-xs font-medium truncate max-w-[40px]">
          {frameName || ''}
        </span>
        {onDelete && (
          <button
            className="opacity-0 group-hover:opacity-100 transition-opacity w-3 h-3 flex items-center justify-center hover:bg-red-500 hover:text-white rounded-sm"
            onClick={handleDeleteClick}
            title="Delete frame"
          >
            <X size={8} />
          </button>
        )}
      </div>

      {/* Bottom row - duration and repetitions */}
      <div className="flex justify-end items-end h-4">
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
            className="text-xs px-1 w-12 text-center"
            style={{
              backgroundColor: palette.background,
              color: palette.text,
              border: `1px solid ${palette.primary}`,
              borderRadius: '2px'
            }}
          />
        ) : (
          <span 
            className="text-xs px-1 cursor-pointer hover:bg-opacity-80 transition-colors"
            style={{
              backgroundColor: palette.background,
              color: palette.text,
              border: '1px solid transparent'
            }}
            onClick={handleValueClick}
            title="Click to edit value"
          >
            {repetitions > 1 ? `${displayValue.toFixed(2)} × ${repetitions}` : displayValue.toFixed(2)}
          </span>
        )}
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