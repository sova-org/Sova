import { writable } from 'svelte/store';

export type ViewType = 'CONFIG' | 'LOGIN' | 'DEVICES' | 'LOGS' | 'SCENE';

export const viewState = writable<ViewType>('SCENE');
