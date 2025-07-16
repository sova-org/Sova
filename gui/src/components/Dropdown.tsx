import React, { useState, useRef, useEffect } from 'react';
import { ChevronDown } from 'lucide-react';
import { useColorContext } from '../context/ColorContext';

interface DropdownOption {
  value: string;
  label: string;
}

interface DropdownProps {
  value: string;
  options: DropdownOption[];
  onChange: (value: string) => void;
  placeholder?: string;
  disabled?: boolean;
  className?: string;
  size?: 'sm' | 'md' | 'lg';
  icon?: React.ReactNode;
  title?: string;
  dropDirection?: 'auto' | 'up' | 'down';
}

export const Dropdown: React.FC<DropdownProps> = ({
  value,
  options,
  onChange,
  placeholder = 'Select...',
  disabled = false,
  className = '',
  size = 'md',
  icon,
  title,
  dropDirection = 'auto',
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const [shouldDropUp, setShouldDropUp] = useState(false);
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const dropdownRef = useRef<HTMLDivElement>(null);
  const { palette } = useColorContext();

  const selectedOption = options.find(opt => opt.value === value);

  const sizeStyles = {
    sm: {
      button: 'px-2 py-1 text-xs',
      dropdown: 'py-1',
      option: 'px-2 py-1 text-xs',
      icon: 12,
    },
    md: {
      button: 'px-3 py-2 text-sm',
      dropdown: 'py-1',
      option: 'px-3 py-2 text-sm',
      icon: 14,
    },
    lg: {
      button: 'px-4 py-3 text-base',
      dropdown: 'py-2',
      option: 'px-4 py-3 text-base',
      icon: 16,
    },
  };

  const currentSize = sizeStyles[size];

  // Calculate drop direction
  useEffect(() => {
    if (!isOpen || !dropdownRef.current) return;

    if (dropDirection === 'up') {
      setShouldDropUp(true);
    } else if (dropDirection === 'down') {
      setShouldDropUp(false);
    } else {
      // Auto-detect based on position
      const rect = dropdownRef.current.getBoundingClientRect();
      const spaceBelow = window.innerHeight - rect.bottom;
      const spaceAbove = rect.top;
      const dropdownHeight = Math.min(240, options.length * 40); // Estimate dropdown height
      
      setShouldDropUp(spaceBelow < dropdownHeight && spaceAbove > spaceBelow);
    }
  }, [isOpen, dropDirection, options.length]);

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        setIsOpen(false);
        setHighlightedIndex(-1);
      }
    };

    const handleEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setIsOpen(false);
        setHighlightedIndex(-1);
      }
    };

    if (isOpen) {
      document.addEventListener('mousedown', handleClickOutside);
      document.addEventListener('keydown', handleEscape);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleEscape);
    };
  }, [isOpen]);

  const handleKeyDown = (event: React.KeyboardEvent) => {
    if (disabled) return;

    switch (event.key) {
      case 'Enter':
      case ' ':
        event.preventDefault();
        if (isOpen && highlightedIndex >= 0 && highlightedIndex < options.length) {
          onChange(options[highlightedIndex]!.value);
          setIsOpen(false);
          setHighlightedIndex(-1);
        } else {
          setIsOpen(true);
        }
        break;
      case 'ArrowDown':
        event.preventDefault();
        if (!isOpen) {
          setIsOpen(true);
        } else {
          setHighlightedIndex(prev => 
            prev < options.length - 1 ? prev + 1 : 0
          );
        }
        break;
      case 'ArrowUp':
        event.preventDefault();
        if (!isOpen) {
          setIsOpen(true);
        } else {
          setHighlightedIndex(prev => 
            prev > 0 ? prev - 1 : options.length - 1
          );
        }
        break;
      case 'Escape':
        setIsOpen(false);
        setHighlightedIndex(-1);
        break;
    }
  };

  const handleOptionClick = (option: DropdownOption, event: React.MouseEvent) => {
    event.preventDefault();
    event.stopPropagation();
    onChange(option.value);
    setIsOpen(false);
    setHighlightedIndex(-1);
  };

  const buttonStyle = {
    backgroundColor: palette.surface,
    borderColor: palette.border,
    color: disabled ? palette.muted : palette.text,
    cursor: disabled ? 'not-allowed' : 'pointer',
  };

  const dropdownStyle = {
    backgroundColor: palette.surface,
    borderColor: palette.border,
    boxShadow: `0 4px 6px -1px ${palette.border}40, 0 2px 4px -1px ${palette.border}20`,
  };

  return (
    <div
      ref={dropdownRef}
      className={`relative inline-block ${className}`}
      title={title}
    >
      <button
        type="button"
        className={`
          flex items-center justify-between w-full border transition-colors
          focus:outline-none focus:ring-1 hover:opacity-80 disabled:opacity-50
          ${currentSize.button}
        `}
        style={{
          ...buttonStyle,
          borderRadius: '0', // Square corners
          focusRingColor: palette.primary,
        }}
        onClick={(e) => {
          e.preventDefault();
          e.stopPropagation();
          if (!disabled) {
            setIsOpen(!isOpen);
          }
        }}
        onKeyDown={handleKeyDown}
        disabled={disabled}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
      >
        <div className="flex items-center space-x-2 flex-1 min-w-0">
          {icon && (
            <span style={{ color: palette.muted }}>
              {icon}
            </span>
          )}
          <span className="truncate">
            {selectedOption?.label || placeholder}
          </span>
        </div>
        <ChevronDown 
          size={currentSize.icon}
          style={{ 
            color: palette.muted,
            transform: isOpen ? 'rotate(180deg)' : 'rotate(0deg)',
            transition: 'transform 0.2s ease-in-out',
          }}
        />
      </button>

      {isOpen && (
        <div
          className={`
            absolute w-full border
            max-h-60 overflow-auto
            ${currentSize.dropdown}
            ${shouldDropUp ? 'bottom-full mb-1' : 'top-full mt-1'}
          `}
          style={{
            ...dropdownStyle,
            borderRadius: '0', // Square corners
            zIndex: 9999, // Very high z-index to ensure it appears above everything
          }}
          role="listbox"
        >
          {options.map((option, index) => (
            <div
              key={option.value}
              className={`
                cursor-pointer transition-colors
                ${currentSize.option}
              `}
              style={{
                backgroundColor: highlightedIndex === index ? palette.primary : 'transparent',
                color: highlightedIndex === index ? palette.background : palette.text,
              }}
              onClick={(event) => handleOptionClick(option, event)}
              onMouseEnter={() => setHighlightedIndex(index)}
              onMouseLeave={() => setHighlightedIndex(-1)}
              role="option"
              aria-selected={option.value === value}
            >
              {option.label}
            </div>
          ))}
        </div>
      )}
    </div>
  );
};