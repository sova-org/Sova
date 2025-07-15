import React, { useRef, useEffect } from 'react';
import { useStore } from '@nanostores/react';
import { dragStore, setDropTarget } from '../stores/dragStore';
import { useColorContext } from '../context/ColorContext';

interface DropZoneProps {
  lineIndex: number;
  insertIndex: number;
  isHorizontal?: boolean;
  width?: number;
  height?: number;
}

export const DropZone: React.FC<DropZoneProps> = ({ 
  lineIndex, 
  insertIndex, 
  isHorizontal = false,
  width = 120,
  height = 20
}) => {
  const { palette } = useColorContext();
  const dragState = useStore(dragStore);
  const dropZoneRef = useRef<HTMLDivElement>(null);

  const isActiveDropTarget = dragState.dropTarget?.lineIndex === lineIndex && 
                            dragState.dropTarget?.insertIndex === insertIndex;

  useEffect(() => {
    const element = dropZoneRef.current;
    if (!element || !dragState.isDragging) return;

    const handleMouseEnter = () => {
      setDropTarget({ lineIndex, insertIndex });
    };

    const handleMouseLeave = () => {
      // Only clear if this is the current drop target
      if (isActiveDropTarget) {
        setDropTarget(null);
      }
    };

    element.addEventListener('mouseenter', handleMouseEnter);
    element.addEventListener('mouseleave', handleMouseLeave);

    return () => {
      element.removeEventListener('mouseenter', handleMouseEnter);
      element.removeEventListener('mouseleave', handleMouseLeave);
    };
  }, [dragState.isDragging, lineIndex, insertIndex, isActiveDropTarget]);

  // Only show drop zone when dragging
  if (!dragState.isDragging) return null;

  const dropZoneStyle = {
    width: isHorizontal ? width : 4,
    height: isHorizontal ? 4 : height,
    backgroundColor: isActiveDropTarget ? palette.primary : 'transparent',
    border: isActiveDropTarget ? `2px solid ${palette.primary}` : '2px dashed rgba(255,255,255,0.3)',
    transition: 'all 0.2s ease',
    zIndex: 10,
    pointerEvents: 'auto' as const,
  };

  return (
    <div 
      ref={dropZoneRef}
      style={dropZoneStyle}
      className="absolute"
      title={`Drop here to insert at position ${insertIndex} in line ${lineIndex}`}
    />
  );
};