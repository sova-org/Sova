import React, { useMemo } from 'react';
import CodeMirror from '@uiw/react-codemirror';
import { vim } from '@replit/codemirror-vim';
import { useStore } from '@nanostores/react';
import { useColorContext } from '../context/ColorContext';
import { editorSettingsStore } from '../stores/editorSettingsStore';
import { createCustomTheme } from '../themes/customTheme';
import { flashField } from './FlashField';
import { evalKeymap } from './EvalKeymap';
import { EditorLogPanel } from './EditorLogPanel';
import { getLanguageSupport } from '../languages';

interface CodeEditorProps {
  value?: string;
  onChange?: (content: string) => void;
  className?: string;
  onEvaluate?: () => void;
  showEvaluateButton?: boolean;
  language?: string;
  availableLanguages?: string[];
  onLanguageChange?: (language: string) => void;
}

export const CodeEditor: React.FC<CodeEditorProps> = ({
  value = '',
  onChange,
  className = '',
  onEvaluate,
  showEvaluateButton = false,
  language = 'bali',
  availableLanguages = [],
  onLanguageChange,
}) => {
  const { themeMode, palette } = useColorContext();
  const editorSettings = useStore(editorSettingsStore);

  // Memoize the theme to prevent unnecessary updates
  const currentTheme = useMemo(() => 
    createCustomTheme({ palette, themeMode, fontFamily: editorSettings.fontFamily }), 
    [palette.primary, palette.secondary, palette.background, palette.text, palette.surface, palette.muted, palette.border, palette.success, palette.error, palette.warning, palette.info, themeMode, editorSettings.fontFamily]
  );

  // Memoize extensions to prevent unnecessary updates
  const extensions = useMemo(() => {
    const baseExtensions = [];
    
    // Add language support
    const languageSupport = getLanguageSupport(language);
    if (languageSupport) {
      baseExtensions.push(languageSupport);
    }
    
    // Add vim mode if enabled
    if (editorSettings.vimMode) {
      baseExtensions.push(vim());
    }
    
    // Add flash field for visual feedback
    baseExtensions.push(flashField);
    
    // Add evaluation keymap if evaluate function is provided
    if (onEvaluate) {
      // Use a bright accent color with transparency for the flash
      const flashColor = `${palette.warning}40`; // 40 is hex for ~25% opacity
      baseExtensions.push(evalKeymap({ onEvaluate, flashColor }));
    }
    
    return baseExtensions;
  }, [editorSettings.vimMode, onEvaluate, palette.warning, language]);

  return (
    <div 
      className={`h-full w-full relative ${className}`}
      style={{
        backgroundColor: palette.background,
        fontSize: `${editorSettings.fontSize}px`,
        fontFamily: editorSettings.fontFamily,
      }}
    >
      <CodeMirror
        value={value}
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
          fontFamily: editorSettings.fontFamily,
        }}
      />
      
      <EditorLogPanel 
        onEvaluate={showEvaluateButton ? onEvaluate : undefined}
        currentLanguage={language}
        availableLanguages={availableLanguages}
        onLanguageChange={onLanguageChange}
      />
    </div>
  );
};