import { writable } from "svelte/store";

const STORAGE_KEY = "sova-login-fields";

interface LoginFields {
  ip: string;
  port: number;
  nickname: string;
}

function loadNickname(): string {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const fields: LoginFields = JSON.parse(stored);
      return fields.nickname || "";
    }
  } catch {
    // Invalid stored state
  }
  return "";
}

function saveNickname(nickname: string): void {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    const fields: LoginFields = stored
      ? JSON.parse(stored)
      : { ip: "127.0.0.1", port: 8080, nickname: "" };
    fields.nickname = nickname;
    localStorage.setItem(STORAGE_KEY, JSON.stringify(fields));
  } catch {
    // Storage unavailable
  }
}

function createNicknameStore() {
  const { subscribe, set } = writable<string>("");

  return {
    subscribe,
    set(value: string) {
      set(value);
      saveNickname(value);
    },
    initialize() {
      const nickname = loadNickname();
      set(nickname);
      return nickname;
    },
  };
}

export const nickname = createNicknameStore();
