import React, { useState } from 'react';
import { useStore } from '@nanostores/react';
import { useColorContext } from '../context/ColorContext';
import { scriptEditorStore } from '../stores/sceneStore';
import { ChevronUp, ChevronDown, CheckCircle, XCircle, Info } from 'lucide-react';

interface EditorLogPanelProps {
  onEvaluate?: () => void;
}

export const EditorLogPanel: React.FC<EditorLogPanelProps> = ({ onEvaluate }) => {
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
          {scriptEditor.selectedFrame && (
            <span style={{ color: palette.muted }}>
              Line {scriptEditor.selectedFrame.line_idx}, Frame {scriptEditor.selectedFrame.frame_idx}
            </span>
          )}
        </div>
        <div className="flex items-center space-x-2">
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
            fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace',
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