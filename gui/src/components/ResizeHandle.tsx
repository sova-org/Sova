import React, { useCallback, useRef, useState } from 'react';

interface ResizeHandleProps {
  direction: 'horizontal' | 'vertical';
  position: 'left' | 'right' | 'top' | 'bottom';
  panelRef: React.RefObject<HTMLDivElement>;
  onResizeEnd: (newWidth: number, newHeight: number) => void;
  className?: string;
}

export const ResizeHandle: React.FC<ResizeHandleProps> = ({ 
  direction, 
  position, 
  panelRef,
  onResizeEnd,
  className = '' 
}) => {
  const isDragging = useRef(false);
  const startPos = useRef(0);
  const startSize = useRef({ width: 0, height: 0 });
  const animationFrame = useRef<number | null>(null);
  const [_isResizing, setIsResizing] = useState(false);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    if (!panelRef.current) return;
    
    e.preventDefault();
    isDragging.current = true;
    setIsResizing(true);
    startPos.current = direction === 'horizontal' ? e.clientX : e.clientY;
    
    // Get current size from DOM
    const rect = panelRef.current.getBoundingClientRect();
    startSize.current = { width: rect.width, height: rect.height };
    
    // Disable transitions during resize
    panelRef.current.style.transition = 'none';
    
    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current || !panelRef.current) return;
      
      if (animationFrame.current) {
        cancelAnimationFrame(animationFrame.current);
      }
      
      animationFrame.current = requestAnimationFrame(() => {
        const currentPos = direction === 'horizontal' ? e.clientX : e.clientY;
        const delta = currentPos - startPos.current;
        
        let adjustedDelta = delta;
        if (position === 'left' || position === 'top') {
          adjustedDelta = -delta;
        }
        
        if (direction === 'horizontal') {
          const maxWidth = window.innerWidth * 0.8;
          const newWidth = Math.max(300, Math.min(maxWidth, startSize.current.width + adjustedDelta));
          panelRef.current.style.width = `${newWidth}px`;
        } else {
          const maxHeight = window.innerHeight * 0.6;
          const newHeight = Math.max(200, Math.min(maxHeight, startSize.current.height + adjustedDelta));
          panelRef.current.style.height = `${newHeight}px`;
        }
      });
    };

    const handleMouseUp = () => {
      if (!panelRef.current) return;
      
      isDragging.current = false;
      setIsResizing(false);
      if (animationFrame.current) {
        cancelAnimationFrame(animationFrame.current);
        animationFrame.current = null;
      }
      
      // Re-enable transitions
      panelRef.current.style.transition = '';
      
      // Get final size and update store
      const rect = panelRef.current.getBoundingClientRect();
      onResizeEnd(rect.width, rect.height);
      
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    };

    document.addEventListener('mousemove', handleMouseMove);
    document.addEventListener('mouseup', handleMouseUp);
    document.body.style.cursor = direction === 'horizontal' ? 'col-resize' : 'row-resize';
    document.body.style.userSelect = 'none';
  }, [direction, position, panelRef, onResizeEnd]);

  const getCursorClass = () => {
    return direction === 'horizontal' ? 'cursor-col-resize' : 'cursor-row-resize';
  };

  // const _getPositionClasses = () => {
  //   switch (position) {
  //     case 'left':
  //       return 'absolute left-0 top-0 w-2 h-full';
  //     case 'right':
  //       return 'absolute right-0 top-0 w-2 h-full';
  //     case 'top':
  //       return 'absolute top-0 left-0 w-full h-2';
  //     case 'bottom':
  //       return 'absolute bottom-0 left-0 w-full h-2';
  //     default:
  //       return '';
  //   }
  // };

  const getHoverArea = () => {
    // Larger invisible area for easier grabbing
    switch (position) {
      case 'left':
        return 'absolute top-0 w-6 h-full';
      case 'right':
        return 'absolute top-0 w-6 h-full';
      case 'top':
        return 'absolute left-0 w-full h-6';
      case 'bottom':
        return 'absolute left-0 w-full h-6';
      default:
        return '';
    }
  };

  const getHoverAreaStyle = () => {
    switch (position) {
      case 'left':
        return { left: '-8px' };
      case 'right':
        return { right: '-8px' };
      case 'top':
        return { top: '-8px' };
      case 'bottom':
        return { bottom: '-8px' };
      default:
        return {};
    }
  };

  return (
    <div
      className={`${getHoverArea()} ${getCursorClass()} z-50 transition-colors ${className}`}
      onMouseDown={handleMouseDown}
      style={{
        backgroundColor: 'transparent',
        userSelect: 'none',
        ...getHoverAreaStyle()
      }}
    />
  );
};