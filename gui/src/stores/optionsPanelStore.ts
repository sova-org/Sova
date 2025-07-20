// import { atom } from 'nanostores';
import { persistentAtom } from '@nanostores/persistent';
import { updateStore } from '../utils/store-helpers';

export interface OptionsPanelState {
  width: number;
  height: number;
  position: 'left' | 'right' | 'bottom';
  activeTab: 'colors' | 'settings' | 'devices' | 'files' | 'server' | 'logs';
  isPinned: boolean;
}

export const optionsPanelStore = persistentAtom<OptionsPanelState>(
  'optionsPanel',
  {
    width: 360,
    height: 400,
    position: 'right',
    activeTab: 'colors',
    isPinned: false
  },
  {
    encode: JSON.stringify,
    decode: JSON.parse
  }
);

export const setOptionsPanelSize = (width: number, height: number) => {
  updateStore(optionsPanelStore, { width, height });
};

export const setOptionsPanelPosition = (position: 'left' | 'right' | 'bottom') => {
  updateStore(optionsPanelStore, { position });
};

export const setOptionsPanelActiveTab = (activeTab: 'colors' | 'settings' | 'devices' | 'files' | 'server' | 'logs') => {
  updateStore(optionsPanelStore, { activeTab });
};

export const toggleOptionsPanelPin = () => {
  const currentState = optionsPanelStore.get();
  updateStore(optionsPanelStore, { isPinned: !currentState.isPinned });
};