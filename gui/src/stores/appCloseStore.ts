import { atom } from 'nanostores';

export const showCloseConfirmation = atom<boolean>(false);
export const closeConfirmationCallback = atom<(() => void) | null>(null);