import { writable, derived, type Writable, type Readable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import type { ChatPayload } from "$lib/types/protocol";
import { ListenerGroup } from "./helpers";

// Peer list
export const peers: Writable<string[]> = writable([]);

// Chat messages
export interface ChatMessage {
  user: string;
  message: string;
  timestamp: number;
}

export const chatMessages: Writable<ChatMessage[]> = writable([]);

// Derived stores
export const peerCount: Readable<number> = derived(
  peers,
  ($peers) => $peers.length,
);

const listeners = new ListenerGroup();
let initialized = false;

export async function initializeCollaborationStore(): Promise<void> {
  if (initialized) return;
  initialized = true;

  // Listen for peers updates
  await listeners.add(() =>
    listen<string[]>("server:peers-updated", (event) => {
      peers.set(event.payload);
    }),
  );

  // Listen for chat messages
  await listeners.add(() =>
    listen<ChatPayload>("server:chat", (event) => {
      chatMessages.update(($messages) => [
        ...$messages,
        {
          user: event.payload.user,
          message: event.payload.message,
          timestamp: Date.now(),
        },
      ]);
    }),
  );
}

export function cleanupCollaborationStore(): void {
  listeners.cleanup();
  initialized = false;
  peers.set([]);
  chatMessages.set([]);
}
