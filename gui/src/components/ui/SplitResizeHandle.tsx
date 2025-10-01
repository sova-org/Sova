import React, { useCallback, useRef, useState } from 'react';
import { setSplitRatio } from '../../stores/ui/preferences';

interface SplitResizeHandleProps {
  orientation: 'horizontal' | 'vertical';
  containerRef: React.RefObject<HTMLDivElement>;
  className?: string;
}

export const SplitResizeHandle: React.FC<SplitResizeHandleProps> = ({ 
  orientation, 
  containerRef,
  className = '' 
}) => {
  const isDragging = useRef(false);
  const startPos = useRef(0);
  const animationFrame = useRef<number | null>(null);
  const [isResizing, setIsResizing] = useState(false);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (!containerRef.current) return;
    
    e.preventDefault();
    e.stopPropagation();
    isDragging.current = true;
    setIsResizing(true);
    startPos.current = orientation === 'horizontal' ? e.clientY : e.clientX;
    
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current || !containerRef.current) return;
      
      if (animationFrame.current) {
        cancelAnimationFrame(animationFrame.current);
      }
      
      animationFrame.current = requestAnimationFrame(() => {
        if (!containerRef.current) return;
        
        const rect = containerRef.current.getBoundingClientRect();
        const currentPos = orientation === 'horizontal' ? e.clientY : e.clientX;
        const containerStart = orientation === 'horizontal' ? rect.top : rect.left;
        const containerSize = orientation === 'horizontal' ? rect.height : rect.width;
        
        const relativePos = currentPos - containerStart;
        const newRatio = Math.max(0.1, Math.min(0.9, relativePos / containerSize));
        
        setSplitRatio(newRatio);
      });
    };

    const handleMouseUp = () => {
      isDragging.current = false;
      setIsResizing(false);
      
      if (animationFrame.current) {
        cancelAnimationFrame(animationFrame.current);
        animationFrame.current = null;
      }
      
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = orientation === 'horizontal' ? 'row-resize' : 'col-resize';
    document.body.style.userSelect = 'none';
  }, [orientation, containerRef]);

  const getCursorClass = () => {
    return orientation === 'horizontal' ? 'cursor-row-resize' : 'cursor-col-resize';
  };

  const getPositionClasses = () => {
    if (orientation === 'horizontal') {
      return 'w-full h-1 hover:bg-blue-400/30';
    } else {
      return 'h-full w-1 hover:bg-blue-400/30';
    }
  };

  const getHoverAreaClasses = () => {
    if (orientation === 'horizontal') {
      return 'w-full h-4 -my-2';
    } else {
      return 'h-full w-4 -mx-2';
    }
  };

  return (
    <div
      className={`${getHoverAreaClasses()} ${getCursorClass()} ${className}`}
      onMouseDown={handleMouseDown}
    >
      <div 
        className={`${getPositionClasses()} ${isResizing ? 'bg-blue-400/50' : 'bg-transparent'} transition-colors`}
      />
    </div>
  );
};