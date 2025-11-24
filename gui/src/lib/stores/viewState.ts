import { writable } from 'svelte/store';

export type ViewType = 'EDITOR' | 'CONFIG' | 'LOGIN' | 'DEVICES';

export const viewState = writable<ViewType>('EDITOR');
