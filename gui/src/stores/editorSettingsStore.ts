import { persistentAtom } from '@nanostores/persistent';
import { updateStore } from '../utils/store-helpers';

export interface EditorSettings {
  fontSize: number;
  tabSize: number;
  vimMode: boolean;
  fontFamily: string;
}

export const editorSettingsStore = persistentAtom<EditorSettings>('editorSettings', {
  fontSize: 14,
  tabSize: 4,
  vimMode: false,
  fontFamily: '"JetBrains Mono", monospace',
}, {
  encode: JSON.stringify,
  decode: JSON.parse,
});

export const setFontSize = (fontSize: number) => {
  updateStore(editorSettingsStore, { fontSize });
};

export const setTabSize = (tabSize: number) => {
  updateStore(editorSettingsStore, { tabSize });
};

export const setVimMode = (vimMode: boolean) => {
  updateStore(editorSettingsStore, { vimMode });
};

export const toggleVimMode = () => {
  const currentSettings = editorSettingsStore.get();
  updateStore(editorSettingsStore, { vimMode: !currentSettings.vimMode });
};

export const setFontFamily = (fontFamily: string) => {
  updateStore(editorSettingsStore, { fontFamily });
};