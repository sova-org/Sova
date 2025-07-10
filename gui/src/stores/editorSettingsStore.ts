import { persistentAtom } from '@nanostores/persistent';

export interface EditorSettings {
  fontSize: number;
  tabSize: number;
  vimMode: boolean;
}

export const editorSettingsStore = persistentAtom<EditorSettings>('editorSettings', {
  fontSize: 14,
  tabSize: 4,
  vimMode: false,
}, {
  encode: JSON.stringify,
  decode: JSON.parse,
});

export const setFontSize = (fontSize: number) => {
  editorSettingsStore.set({ ...editorSettingsStore.get(), fontSize });
};

export const setTabSize = (tabSize: number) => {
  editorSettingsStore.set({ ...editorSettingsStore.get(), tabSize });
};

export const setVimMode = (vimMode: boolean) => {
  editorSettingsStore.set({ ...editorSettingsStore.get(), vimMode });
};

export const toggleVimMode = () => {
  const currentSettings = editorSettingsStore.get();
  editorSettingsStore.set({ ...currentSettings, vimMode: !currentSettings.vimMode });
};