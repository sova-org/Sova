import React from 'react';
import { SplitSquareHorizontal, SplitSquareVertical } from 'lucide-react';
import { toggleSplitOrientation } from '../../stores/ui/preferences';

interface SplitToggleButtonProps {
  orientation: 'horizontal' | 'vertical';
  className?: string;
  style?: React.CSSProperties;
}

export const SplitToggleButton: React.FC<SplitToggleButtonProps> = ({ 
  orientation, 
  className = '',
  style = {}
}) => {
  const handleToggle = () => {
    toggleSplitOrientation();
  };

  const Icon = orientation === 'horizontal' ? SplitSquareHorizontal : SplitSquareVertical;
  const tooltip = orientation === 'horizontal' ? 'Switch to vertical split' : 'Switch to horizontal split';

  return (
    <button
      onClick={handleToggle}
      className={`p-2 rounded-md hover:bg-gray-700/50 transition-colors ${className}`}
      style={style}
      title={tooltip}
    >
      <Icon size={16} />
    </button>
  );
};