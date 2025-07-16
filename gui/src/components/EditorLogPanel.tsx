import React, { useState } from 'react';
import { useStore } from '@nanostores/react';
import { useColorContext } from '../context/ColorContext';
import { scriptEditorStore } from '../stores/sceneStore';
import { ChevronUp, ChevronDown, CheckCircle, XCircle, Info, Code } from 'lucide-react';
import { Dropdown } from './Dropdown';

interface EditorLogPanelProps {
  onEvaluate: (() => void) | undefined;
  currentLanguage: string | undefined;
  availableLanguages: string[] | undefined;
  onLanguageChange: ((language: string) => void) | undefined;
}

export const EditorLogPanel: React.FC<EditorLogPanelProps> = ({ 
  onEvaluate,
  currentLanguage,
  availableLanguages = undefined,
  onLanguageChange
}) => {
  const { palette } = useColorContext();
  const scriptEditor = useStore(scriptEditorStore);
  const [isExpanded, setIsExpanded] = useState(false);

  const hasError = scriptEditor.compilationError !== null;
  const hasRecentSuccess = scriptEditor.lastCompiled > 0 && !hasError && (Date.now() - scriptEditor.lastCompiled < 5000);

  const getStatusIcon = () => {
    if (hasError) {
      return <XCircle size={14} style={{ color: palette.error }} />;
    } else if (hasRecentSuccess) {
      return <CheckCircle size={14} style={{ color: palette.success }} />;
    } else {
      return <Info size={14} style={{ color: palette.muted }} />;
    }
  };

  const getStatusText = () => {
    if (hasError) {
      return 'Compilation Error';
    } else if (hasRecentSuccess) {
      return 'Compiled Successfully';
    } else {
      return 'Ready';
    }
  };

  const getStatusColor = () => {
    if (hasError) {
      return palette.error;
    } else if (hasRecentSuccess) {
      return palette.success;
    } else {
      return palette.muted;
    }
  };

  return (
    <div 
      className="absolute bottom-0 left-0 right-0 border-t"
      style={{
        backgroundColor: palette.surface,
        borderColor: palette.border,
        zIndex: 10,
      }}
    >
      {/* Status Bar */}
      <div
        className="flex items-center justify-between px-3 py-2 cursor-pointer hover:opacity-80 transition-opacity"
        onClick={() => setIsExpanded(!isExpanded)}
        style={{
          fontSize: '12px',
          color: palette.text,
        }}
      >
        <div className="flex items-center space-x-2">
          {getStatusIcon()}
          <span style={{ color: getStatusColor() }}>
            {getStatusText()}
          </span>
          
          {/* Language Selector */}
          {currentLanguage && availableLanguages && availableLanguages.length > 0 && onLanguageChange && (
            <>
              <span style={{ color: palette.muted }}>•</span>
              <div onClick={(e) => e.stopPropagation()}>
                <Dropdown
                  value={currentLanguage}
                  options={availableLanguages.map(lang => ({
                    value: lang,
                    label: lang.charAt(0).toUpperCase() + lang.slice(1)
                  }))}
                  onChange={(value) => {
                    onLanguageChange(value);
                  }}
                  size="sm"
                  icon={<Code size={12} />}
                  title="Select script language"
                  dropDirection="up"
                />
              </div>
            </>
          )}
          
          {scriptEditor.selectedFrame && (
            <>
              <span style={{ color: palette.muted }}>•</span>
              <span style={{ color: palette.muted }}>
                Line {scriptEditor.selectedFrame.line_idx}, Frame {scriptEditor.selectedFrame.frame_idx}
              </span>
            </>
          )}
        </div>
        <div className="flex items-center space-x-2">
          {onEvaluate && (
            <button
              onClick={(e) => {
                e.stopPropagation(); // Prevent triggering the panel expand/collapse
                onEvaluate();
              }}
              className="px-2 py-1 text-xs font-medium transition-colors hover:opacity-80"
              style={{
                backgroundColor: palette.primary,
                color: palette.background,
                border: 'none',
                borderRadius: '0', // Square corners
              }}
              title="Evaluate and update script (Cmd/Ctrl+S or Cmd/Ctrl+Enter)"
            >
              Evaluate
            </button>
          )}
          <span style={{ color: palette.muted }}>
            {isExpanded ? 'Hide' : 'Show'} Details
          </span>
          {isExpanded ? (
            <ChevronDown size={14} style={{ color: palette.muted }} />
          ) : (
            <ChevronUp size={14} style={{ color: palette.muted }} />
          )}
        </div>
      </div>

      {/* Expandable Details */}
      {isExpanded && scriptEditor.compilationError && (
        <div
          className="px-3 py-2 border-t"
          style={{
            backgroundColor: palette.background,
            borderColor: palette.border,
            fontSize: '11px',
            fontFamily: 'inherit',
            color: palette.error,
            maxHeight: '150px',
            overflowY: 'auto',
          }}
        >
          <div className="mb-2" style={{ color: palette.text }}>
            <strong>{scriptEditor.compilationError.lang} Error:</strong> Position {scriptEditor.compilationError.from}-{scriptEditor.compilationError.to}
          </div>
          <pre className="whitespace-pre-wrap">{scriptEditor.compilationError.info}</pre>
        </div>
      )}
    </div>
  );
};