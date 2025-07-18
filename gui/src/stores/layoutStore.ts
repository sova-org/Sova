import { persistentMap } from '@nanostores/persistent';

export interface LayoutState extends Record<string, string | undefined> {
  splitRatio: string;
  splitOrientation: 'horizontal' | 'vertical';
}

export const layoutStore = persistentMap<LayoutState>('layout:', {
  splitRatio: '0.5',
  splitOrientation: 'vertical'
});

export const setSplitRatio = (ratio: number) => {
  layoutStore.setKey('splitRatio', Math.max(0.1, Math.min(0.9, ratio)).toString());
};

export const getSplitRatio = () => {
  return parseFloat(layoutStore.get().splitRatio);
};

export const setSplitOrientation = (orientation: 'horizontal' | 'vertical') => {
  layoutStore.setKey('splitOrientation', orientation);
};

export const toggleSplitOrientation = () => {
  const current = layoutStore.get().splitOrientation;
  layoutStore.setKey('splitOrientation', current === 'horizontal' ? 'vertical' : 'horizontal');
};