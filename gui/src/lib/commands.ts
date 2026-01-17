import { get } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { registerCommand } from "$lib/stores/commandPalette";
import {
  startTransport,
  stopTransport,
  setTempo,
  setName,
} from "$lib/api/client";
import { nickname as nicknameStore } from "$lib/stores/nickname";
import { isPlaying, isStarting } from "$lib/stores/transport";
import { isConnected } from "$lib/stores/connectionState";
import { viewState, type ViewType } from "$lib/stores/viewState";
import { toggleHelpMode } from "$lib/stores/helpMode";
import {
  loadProjectImmediate,
  loadProjectAtEndOfLine,
} from "$lib/stores/projects";
import { editingFrameKey } from "$lib/stores/editingFrame";
import { availableLanguages } from "$lib/stores/languages";

function switchView(view: ViewType): void {
  viewState.navigateTo(view);
}

registerCommand({
  id: "play",
  name: "Play",
  description: "Start transport",
  keywords: ["start"],
  isAvailable: () => get(isConnected) && !get(isPlaying) && !get(isStarting),
  execute: () => startTransport(),
});

registerCommand({
  id: "pause",
  name: "Pause",
  description: "Stop transport",
  keywords: ["stop"],
  isAvailable: () => get(isConnected) && (get(isPlaying) || get(isStarting)),
  execute: () => stopTransport(),
});

registerCommand({
  id: "tempo",
  name: "Tempo",
  description: "Set tempo (e.g., tempo 120)",
  keywords: ["bpm", "speed"],
  isAvailable: () => get(isConnected),
  execute: (args) => {
    const value = parseFloat(args[0]);
    if (isNaN(value) || value < 30 || value > 300) return;
    setTempo(value);
  },
});

registerCommand({
  id: "scene",
  name: "Scene",
  description: "Switch to Scene view",
  keywords: ["editor", "timeline"],
  isAvailable: () => get(isConnected),
  execute: () => switchView("SCENE"),
});

registerCommand({
  id: "config",
  name: "Config",
  description: "Switch to Config view",
  keywords: ["settings"],
  execute: () => switchView("CONFIG"),
});

registerCommand({
  id: "devices",
  name: "Devices",
  description: "Switch to Devices view",
  keywords: ["midi", "osc"],
  isAvailable: () => get(isConnected),
  execute: () => switchView("DEVICES"),
});

registerCommand({
  id: "logs",
  name: "Logs",
  description: "Switch to Logs view",
  execute: () => switchView("LOGS"),
});

registerCommand({
  id: "projects",
  name: "Projects",
  description: "Switch to Projects view",
  keywords: ["snapshots", "load"],
  execute: () => switchView("PROJECTS"),
});

registerCommand({
  id: "chat",
  name: "Chat",
  description: "Switch to Chat view",
  keywords: ["messages"],
  isAvailable: () => get(isConnected),
  execute: () => switchView("CHAT"),
});

registerCommand({
  id: "login",
  name: "Login",
  description: "Switch to Login view",
  keywords: ["connect"],
  isAvailable: () => !get(isConnected),
  execute: () => switchView("LOGIN"),
});

registerCommand({
  id: "nickname",
  name: "Nickname",
  description: "Set nickname (e.g., nickname Alice)",
  keywords: ["name"],
  isAvailable: () => get(isConnected),
  execute: async (args) => {
    const newNickname = args.join(" ").trim();
    if (!newNickname) return;
    nicknameStore.set(newNickname);
    await setName(newNickname);
  },
});

registerCommand({
  id: "save",
  name: "Save",
  description: "Save current project",
  keywords: ["project"],
  isAvailable: () => get(isConnected),
  execute: () => {
    window.dispatchEvent(new CustomEvent("command:open-save-modal"));
  },
});

registerCommand({
  id: "open",
  name: "Open",
  description: "Open a saved project",
  keywords: ["project", "load"],
  isAvailable: () => get(isConnected),
  execute: () => {
    window.dispatchEvent(new CustomEvent("command:open-project-modal"));
  },
});

registerCommand({
  id: "disconnect",
  name: "Disconnect",
  description: "Disconnect from server",
  isAvailable: () => get(isConnected),
  execute: async () => {
    await invoke("disconnect_client");
    isConnected.set(false);
  },
});

registerCommand({
  id: "exit",
  name: "Exit",
  description: "Quit the application",
  keywords: ["quit", "close"],
  execute: async () => {
    await getCurrentWindow().close();
  },
});

registerCommand({
  id: "help",
  name: "Help",
  description: "Toggle help mode",
  execute: () => toggleHelpMode(),
});

registerCommand({
  id: "load",
  name: "Load",
  description: "Load project (e.g., load myproject now)",
  keywords: ["project", "open"],
  isAvailable: () => get(isConnected),
  execute: (args) => {
    const name = args[0];
    const timing = args[1] || "now";
    if (!name) return;
    if (timing === "end") {
      loadProjectAtEndOfLine(name);
    } else {
      loadProjectImmediate(name);
    }
  },
});

registerCommand({
  id: "language",
  name: "Language",
  description: "Set frame language (e.g., language bali)",
  keywords: ["lang", "script"],
  isAvailable: () => get(editingFrameKey) !== null,
  execute: (args) => {
    if (args.length > 0) {
      const target = args[0].toLowerCase();
      const langs = get(availableLanguages);
      const match = langs.find((l) => l.toLowerCase() === target);
      if (match) {
        window.dispatchEvent(
          new CustomEvent("command:set-language", { detail: match })
        );
      }
    } else {
      window.dispatchEvent(new CustomEvent("command:open-language-picker"));
    }
  },
});
