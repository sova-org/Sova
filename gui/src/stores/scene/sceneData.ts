import { atom } from 'nanostores';
import type { Scene, ServerMessage } from '../../types';
import { updateGlobalVariables } from '../globalVariablesStore';

export const sceneStore = atom<Scene | null>(null);
export const progressionCache = atom<Map<string, number>>(new Map());

export const handleSceneMessage = (message: ServerMessage) => {
  if (typeof message === 'object' && message !== null) {
    switch (true) {
      case 'Hello' in message:
        sceneStore.set(message.Hello.scene);
        return true;

      case 'SceneValue' in message:
        sceneStore.set(message.SceneValue);
        return true;

      case 'SceneLength' in message:
        const currentScene = sceneStore.get();
        if (currentScene) {
          sceneStore.set({ ...currentScene, length: message.SceneLength });
        }
        return true;

      case 'Snapshot' in message:
        sceneStore.set(message.Snapshot.scene);
        return true;

      case 'GlobalVariablesUpdate' in message:
        updateGlobalVariables(message.GlobalVariablesUpdate);
        return true;
    }
  }

  return false;
};

export const getScene = () => sceneStore.get();
export const getSceneLength = () => sceneStore.get()?.length ?? 0;
export const getSceneLines = () => sceneStore.get()?.lines ?? [];
