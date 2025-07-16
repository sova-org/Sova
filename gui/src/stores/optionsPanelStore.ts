// import { atom } from 'nanostores';
import { persistentAtom } from '@nanostores/persistent';

export interface OptionsPanelState {
  width: number;
  height: number;
  position: 'left' | 'right' | 'bottom';
  activeTab: 'colors' | 'settings' | 'devices' | 'files';
}

export const optionsPanelStore = persistentAtom<OptionsPanelState>(
  'optionsPanel',
  {
    width: 360,
    height: 400,
    position: 'right',
    activeTab: 'colors'
  },
  {
    encode: JSON.stringify,
    decode: JSON.parse
  }
);

export const setOptionsPanelSize = (width: number, height: number) => {
  optionsPanelStore.set({
    ...optionsPanelStore.get(),
    width,
    height
  });
};

export const setOptionsPanelPosition = (position: 'left' | 'right' | 'bottom') => {
  optionsPanelStore.set({
    ...optionsPanelStore.get(),
    position
  });
};

export const setOptionsPanelActiveTab = (activeTab: 'colors' | 'settings' | 'devices' | 'files') => {
  optionsPanelStore.set({
    ...optionsPanelStore.get(),
    activeTab
  });
};