import React, { useMemo } from 'react';
import CodeMirror from '@uiw/react-codemirror';
import { vim } from '@replit/codemirror-vim';
import { useStore } from '@nanostores/react';
import { useColorContext } from '../context/ColorContext';
import { editorSettingsStore } from '../stores/editorSettingsStore';
import { createCustomTheme } from '../themes/customTheme';

interface CodeEditorProps {
  initialContent?: string;
  onChange?: (content: string) => void;
  className?: string;
}

export const CodeEditor: React.FC<CodeEditorProps> = ({
  initialContent = '',
  onChange,
  className = '',
}) => {
  const { themeMode, palette } = useColorContext();
  const editorSettings = useStore(editorSettingsStore);

  // Memoize the theme to prevent unnecessary updates
  const currentTheme = useMemo(() => 
    createCustomTheme({ palette, themeMode }), 
    [palette.primary, palette.secondary, palette.background, palette.text, palette.surface, palette.muted, palette.border, palette.success, palette.error, palette.warning, palette.info, themeMode]
  );

  // Memoize extensions to prevent unnecessary updates
  const extensions = useMemo(() => {
    const baseExtensions = [];
    
    // Add vim mode if enabled
    if (editorSettings.vimMode) {
      baseExtensions.push(vim());
    }
    
    return baseExtensions;
  }, [editorSettings.vimMode]);

  return (
    <div 
      className={`h-full w-full ${className}`}
      style={{
        backgroundColor: palette.background,
        fontSize: `${editorSettings.fontSize}px`,
      }}
    >
      <CodeMirror
        value={initialContent}
        height="100%"
        theme={currentTheme}
        extensions={extensions}
        onChange={(value) => onChange?.(value)}
        basicSetup={{
          lineNumbers: true,
          foldGutter: true,
          dropCursor: false,
          allowMultipleSelections: false,
          indentOnInput: true,
          bracketMatching: true,
          closeBrackets: true,
          autocompletion: true,
          highlightSelectionMatches: false,
          searchKeymap: true,
        }}
        style={{
          fontSize: `${editorSettings.fontSize}px`,
          fontFamily: 'Monaco, Menlo, "Ubuntu Mono", monospace',
        }}
      />
    </div>
  );
};