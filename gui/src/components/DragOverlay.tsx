import React, { useEffect, useState } from 'react';
import { useStore } from '@nanostores/react';
import { dragStore, updateDragPreview } from '../stores/dragStore';
import { useColorContext } from '../context/ColorContext';

export const DragOverlay: React.FC = () => {
  const { palette } = useColorContext();
  const dragState = useStore(dragStore);
  const [mousePosition, setMousePosition] = useState({ x: 0, y: 0 });

  useEffect(() => {
    if (!dragState.isDragging) return;

    const handleMouseMove = (e: MouseEvent) => {
      const newPosition = { x: e.clientX, y: e.clientY };
      setMousePosition(newPosition);
      
      // Update drag preview position
      updateDragPreview({
        x: newPosition.x,
        y: newPosition.y,
        width: 120, // Default width, could be dynamic
        height: 60,  // Default height, could be dynamic
      });
    };

    document.addEventListener('mousemove', handleMouseMove);
    return () => document.removeEventListener('mousemove', handleMouseMove);
  }, [dragState.isDragging]);

  if (!dragState.isDragging || !dragState.draggedFrame) {
    return null;
  }

  const { frameData } = dragState.draggedFrame;
  const previewStyle = {
    position: 'fixed' as const,
    left: mousePosition.x + 10, // Offset from cursor
    top: mousePosition.y + 10,
    width: 120,
    height: Math.max(40, 60 * frameData.duration), // Scale height based on duration
    backgroundColor: frameData.enabled ? palette.success : palette.surface,
    color: frameData.enabled ? palette.background : palette.muted,
    border: `2px solid ${palette.primary}`,
    padding: '8px',
    fontSize: '12px',
    pointerEvents: 'none' as const,
    zIndex: 1000,
    opacity: 0.8,
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
  };

  return (
    <div style={previewStyle}>
      <div className="flex flex-col justify-between h-full text-xs">
        {/* Frame name or placeholder */}
        <div className="font-semibold text-center truncate">
          {frameData.name || '∅'}
        </div>
        
        {/* Duration display */}
        <div className="text-right">
          {frameData.repetitions > 1 
            ? `${frameData.duration.toFixed(2)} × ${frameData.repetitions}`
            : frameData.duration.toFixed(2)
          }
        </div>
      </div>
    </div>
  );
};