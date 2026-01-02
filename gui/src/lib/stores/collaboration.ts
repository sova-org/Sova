import { writable, derived, type Writable, type Readable } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ChatPayload } from "$lib/types/protocol";

export const peers: Writable<string[]> = writable([]);

export interface ChatMessage {
  user: string;
  message: string;
  timestamp: number;
}

export const chatMessages: Writable<ChatMessage[]> = writable([]);

export const peerCount: Readable<number> = derived(peers, ($p) => $p.length);

let unlistenFns: UnlistenFn[] = [];
let initialized = false;

export async function initializeCollaborationStore(): Promise<void> {
  if (initialized) return;
  initialized = true;

  unlistenFns.push(
    await listen<string[]>("server:peers-updated", (e) => peers.set(e.payload)),
  );

  unlistenFns.push(
    await listen<ChatPayload>("server:chat", (e) => {
      chatMessages.update(($m) => [
        ...$m,
        { user: e.payload.user, message: e.payload.message, timestamp: Date.now() },
      ]);
    }),
  );
}

export function cleanupCollaborationStore(): void {
  unlistenFns.forEach((fn) => fn());
  unlistenFns = [];
  initialized = false;
  peers.set([]);
  chatMessages.set([]);
}
