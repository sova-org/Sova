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
import { setRuntimeNickname } from "$lib/stores/config";
import { isPlaying, isStarting } from "$lib/stores/transport";
import { isConnected } from "$lib/stores/connectionState";
import {
  paneLayout,
  activePaneId,
  type ViewType,
  type PaneNode,
  type LeafPane,
} from "$lib/stores/paneState";
import { toggleHelpMode } from "$lib/stores/helpMode";
import {
  loadProjectImmediate,
  loadProjectAtEndOfLine,
} from "$lib/stores/projects";

function findFirstLeaf(node: PaneNode): LeafPane | null {
  if (node.type === "leaf") return node;
  return findFirstLeaf(node.children[0]) || findFirstLeaf(node.children[1]);
}

function getTargetPaneId(): string | null {
  const active = get(activePaneId);
  if (active) return active;

  const layout = get(paneLayout);
  const leaf = findFirstLeaf(layout.root);
  return leaf?.id ?? null;
}

function switchView(viewType: ViewType): void {
  const paneId = getTargetPaneId();
  if (paneId) {
    paneLayout.setView(paneId, viewType);
  }
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
    if (isNaN(value) || value < 30 || value > 300) {
      console.warn("Invalid tempo value. Use: tempo <30-300>");
      return;
    }
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
  id: "snapshots",
  name: "Snapshots",
  description: "Switch to Snapshots view",
  keywords: ["projects", "load"],
  execute: () => switchView("SNAPSHOTS"),
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
    const nickname = args.join(" ").trim();
    if (!nickname) {
      console.warn("Usage: nickname <name>");
      return;
    }
    setRuntimeNickname(nickname);
    await setName(nickname);
  },
});

registerCommand({
  id: "save",
  name: "Save",
  description: "Save current project",
  keywords: ["snapshot"],
  isAvailable: () => get(isConnected),
  execute: () => {
    window.dispatchEvent(new CustomEvent("command:open-save-modal"));
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
    if (!name) {
      console.warn("Usage: load <name> [now|end]");
      return;
    }
    if (timing === "end") {
      loadProjectAtEndOfLine(name);
    } else {
      loadProjectImmediate(name);
    }
  },
});

registerCommand({
  id: "split-horizontal",
  name: "Split Horizontal",
  description: "Split current pane horizontally",
  keywords: ["divide", "pane"],
  execute: () => {
    const paneId = getTargetPaneId();
    if (paneId) paneLayout.splitPane(paneId, "horizontal");
  },
});

registerCommand({
  id: "split-vertical",
  name: "Split Vertical",
  description: "Split current pane vertically",
  keywords: ["divide", "pane"],
  execute: () => {
    const paneId = getTargetPaneId();
    if (paneId) paneLayout.splitPane(paneId, "vertical");
  },
});

registerCommand({
  id: "close-pane",
  name: "Close Pane",
  description: "Close the current pane",
  keywords: ["remove", "delete"],
  execute: () => {
    const paneId = getTargetPaneId();
    if (paneId) paneLayout.closePane(paneId);
  },
});
