import { StateField, StateEffect } from '@codemirror/state';
import { Decoration, DecorationSet, EditorView } from '@codemirror/view';

// State effect for adding flash with color
const addFlash = StateEffect.define<{from: number, to: number, id: string, color: string}>();

// State effect for removing flash
const removeFlash = StateEffect.define<string>();

// State field to manage flash decorations
export const flashField = StateField.define<DecorationSet>({
  create() {
    return Decoration.none;
  },
  update(decorations, tr) {
    decorations = decorations.map(tr.changes);
    
    for (let e of tr.effects) {
      if (e.is(addFlash)) {
        const mark = Decoration.mark({
          class: 'cm-flash',
          attributes: {
            style: `background-color: ${e.value.color}; transition: opacity 0.3s ease-out;`
          }
        });
        decorations = decorations.update({
          add: [mark.range(e.value.from, e.value.to)]
        });
      } else if (e.is(removeFlash)) {
        decorations = decorations.update({
          filter: (_from, _to, decoration) => !decoration.spec.class?.includes('cm-flash')
        });
      }
    }
    
    return decorations;
  },
  provide: f => EditorView.decorations.from(f)
});

// Flash function to highlight a range temporarily with custom color
export function flash(view: EditorView, _from: number, _to: number, color: string, duration: number = 300) {
  const flashId = Math.random().toString(36);
  
  // Add flash effect
  view.dispatch({
    effects: addFlash.of({ from: _from, to: _to, id: flashId, color })
  });
  
  // Remove flash after duration
  setTimeout(() => {
    view.dispatch({
      effects: removeFlash.of(flashId)
    });
  }, duration);
}

// Flash entire document with color
export function flashDocument(view: EditorView, color: string, duration?: number) {
  const doc = view.state.doc;
  flash(view, 0, doc.length, color, duration);
}