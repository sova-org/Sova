import React, { useCallback, useRef, useState } from 'react';

interface ResizeHandleProps {
  direction: 'horizontal' | 'vertical';
  position: 'left' | 'right' | 'top' | 'bottom';
  onResize: (delta: number) => void;
  onResizeStart?: () => void;
  onResizeEnd?: () => void;
  className?: string;
}

export const ResizeHandle: React.FC<ResizeHandleProps> = ({ 
  direction, 
  position, 
  onResize,
  onResizeStart,
  onResizeEnd,
  className = '' 
}) => {
  const isDragging = useRef(false);
  const startPos = useRef(0);
  const animationFrame = useRef<number | null>(null);
  const [isHovering, setIsHovering] = useState(false);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    isDragging.current = true;
    startPos.current = direction === 'horizontal' ? e.clientX : e.clientY;
    onResizeStart?.();
    
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current) return;
      
      if (animationFrame.current) {
        cancelAnimationFrame(animationFrame.current);
      }
      
      animationFrame.current = requestAnimationFrame(() => {
        const currentPos = direction === 'horizontal' ? e.clientX : e.clientY;
        const totalDelta = currentPos - startPos.current;
        
        let adjustedDelta = totalDelta;
        if (position === 'left' || position === 'top') {
          adjustedDelta = -totalDelta;
        }
        
        onResize(adjustedDelta);
      });
    };

    const handleMouseUp = () => {
      isDragging.current = false;
      if (animationFrame.current) {
        cancelAnimationFrame(animationFrame.current);
        animationFrame.current = null;
      }
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
      onResizeEnd?.();
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = direction === 'horizontal' ? 'col-resize' : 'row-resize';
    document.body.style.userSelect = 'none';
  }, [direction, position, onResize, onResizeStart, onResizeEnd]);

  const getCursorClass = () => {
    return direction === 'horizontal' ? 'cursor-col-resize' : 'cursor-row-resize';
  };

  const getPositionClasses = () => {
    switch (position) {
      case 'left':
        return 'absolute left-0 top-0 w-1 h-full';
      case 'right':
        return 'absolute right-0 top-0 w-1 h-full';
      case 'top':
        return 'absolute top-0 left-0 w-full h-1';
      case 'bottom':
        return 'absolute bottom-0 left-0 w-full h-1';
      default:
        return '';
    }
  };

  return (
    <div
      className={`${getPositionClasses().replace('w-1', 'w-3').replace('h-1', 'h-3')} ${getCursorClass()} z-50 ${className}`}
      onMouseDown={handleMouseDown}
      style={{
        backgroundColor: 'transparent'
      }}
    />
  );
};