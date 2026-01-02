import { writable, derived, type Writable, type Readable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import type { DeviceInfo } from "$lib/types/protocol";
import { ListenerGroup } from "./helpers";

// Main devices store
export const devices: Writable<DeviceInfo[]> = writable([]);

// Derived stores for filtering devices by type
export const midiDevices: Readable<DeviceInfo[]> = derived(
  devices,
  ($devices) => $devices.filter((d) => d.kind === "Midi"),
);

export const oscDevices: Readable<DeviceInfo[]> = derived(devices, ($devices) =>
  $devices.filter((d) => d.kind === "Osc"),
);

const listeners = new ListenerGroup();

export async function initializeDevicesStore(): Promise<void> {
  // Listen for device list updates
  await listeners.add(() =>
    listen<DeviceInfo[]>("server:device-list", (event) => {
      devices.set(event.payload);
    }),
  );
}

export function cleanupDevicesStore(): void {
  listeners.cleanup();
  devices.set([]);
}
