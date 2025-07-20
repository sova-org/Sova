import type { WritableAtom, MapStore } from 'nanostores';

export function updateStore<T>(store: WritableAtom<T>, updates: Partial<T>) {
  store.set({ ...store.get(), ...updates });
}

export function batchUpdateMap<T extends object>(map: MapStore<T>, updates: Partial<T>) {
  Object.entries(updates).forEach(([key, value]) => {
    (map as any).setKey(key, value);
  });
}

export function updateMapKey<T extends object>(map: MapStore<T>, key: string, updater: (current: any) => any) {
  const current = map.get();
  (map as any).setKey(key, updater((current as any)[key]));
}

export function toggleMapKey<T extends object>(map: MapStore<T>, key: string) {
  const current = map.get();
  (map as any).setKey(key, !(current as any)[key]);
}

export function updateNestedInMap<T extends object>(map: MapStore<T>, key: string, nestedUpdates: any) {
  const current = map.get();
  (map as any).setKey(key, { ...(current as any)[key], ...nestedUpdates });
}