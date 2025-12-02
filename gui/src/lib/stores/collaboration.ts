import { writable, derived, type Writable, type Readable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import type { ChatPayload, PeerEditingPayload } from "$lib/types/protocol";
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

// Frame editing locks (user -> [lineId, frameId])
export interface FrameLock {
  user: string;
  lineId: number;
  frameId: number;
}

export const frameLocks: Writable<FrameLock[]> = writable([]);

// Derived stores
export const peerCount: Readable<number> = derived(
  peers,
  ($peers) => $peers.length,
);

// Helper to check if a frame is being edited
export function isFrameBeingEdited(
  lineId: number,
  frameId: number,
): Readable<boolean> {
  return derived(frameLocks, ($locks) =>
    $locks.some((lock) => lock.lineId === lineId && lock.frameId === frameId),
  );
}

// Helper to get who is editing a frame
export function getFrameEditor(
  lineId: number,
  frameId: number,
): Readable<string | null> {
  return derived(frameLocks, ($locks) => {
    const lock = $locks.find(
      (lock) => lock.lineId === lineId && lock.frameId === frameId,
    );
    return lock?.user ?? null;
  });
}

// Helper to get all frames being edited by a user
export function getFramesEditedByUser(user: string): Readable<FrameLock[]> {
  return derived(frameLocks, ($locks) =>
    $locks.filter((lock) => lock.user === user),
  );
}

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

  // Listen for peer started editing
  await listeners.add(() =>
    listen<PeerEditingPayload>("server:peer-started-editing", (event) => {
      frameLocks.update(($locks) => {
        // Remove any existing lock for this frame
        const filtered = $locks.filter(
          (lock) =>
            !(
              lock.lineId === event.payload.lineId &&
              lock.frameId === event.payload.frameId
            ),
        );
        // Add new lock
        return [
          ...filtered,
          {
            user: event.payload.user,
            lineId: event.payload.lineId,
            frameId: event.payload.frameId,
          },
        ];
      });
    }),
  );

  // Listen for peer stopped editing
  await listeners.add(() =>
    listen<PeerEditingPayload>("server:peer-stopped-editing", (event) => {
      frameLocks.update(($locks) =>
        $locks.filter(
          (lock) =>
            !(
              lock.user === event.payload.user &&
              lock.lineId === event.payload.lineId &&
              lock.frameId === event.payload.frameId
            ),
        ),
      );
    }),
  );
}

export function cleanupCollaborationStore(): void {
  listeners.cleanup();
  initialized = false;
  peers.set([]);
  chatMessages.set([]);
  frameLocks.set([]);
}
