import { keymap, EditorView } from '@codemirror/view';
import { flashDocument } from './FlashField';

interface EvalKeymapOptions {
  onEvaluate: () => void;
  flashColor: string;
}

export function evalKeymap({ onEvaluate, flashColor }: EvalKeymapOptions) {
  return keymap.of([
    {
      key: 'Mod-s',
      preventDefault: true,
      run: (view: EditorView) => {
        // Flash the entire document
        flashDocument(view, flashColor);
        
        // Trigger evaluation
        onEvaluate();
        
        return true;
      }
    },
    {
      key: 'Ctrl-Enter',
      mac: 'Cmd-Enter',
      preventDefault: true,
      run: (view: EditorView) => {
        // Flash the entire document
        flashDocument(view, flashColor);
        
        // Trigger evaluation
        onEvaluate();
        
        return true;
      }
    }
  ]);
}