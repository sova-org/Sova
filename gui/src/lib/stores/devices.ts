import { writable, derived, type Writable, type Readable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { DeviceInfo, DeviceKind } from '$lib/types/protocol';

// Main devices store
export const devices: Writable<DeviceInfo[]> = writable([]);

// Derived stores for filtering devices by type
export const midiDevices: Readable<DeviceInfo[]> = derived(devices, ($devices) =>
	$devices.filter((d) => d.kind === 'Midi')
);

export const oscDevices: Readable<DeviceInfo[]> = derived(devices, ($devices) =>
	$devices.filter((d) => d.kind === 'Osc')
);

export const connectedDevices: Readable<DeviceInfo[]> = derived(devices, ($devices) =>
	$devices.filter((d) => d.is_connected)
);

export const disconnectedDevices: Readable<DeviceInfo[]> = derived(devices, ($devices) =>
	$devices.filter((d) => !d.is_connected)
);

// Helper to get a device by ID
export function getDeviceById(deviceId: number): Readable<DeviceInfo | null> {
	return derived(devices, ($devices) => $devices.find((d) => d.id === deviceId) ?? null);
}

// Helper to get a device by name
export function getDeviceByName(name: string): Readable<DeviceInfo | null> {
	return derived(devices, ($devices) => $devices.find((d) => d.name === name) ?? null);
}

// Helper to get devices by kind
export function getDevicesByKind(kind: DeviceKind): Readable<DeviceInfo[]> {
	return derived(devices, ($devices) => $devices.filter((d) => d.kind === kind));
}

let unlistenFunctions: UnlistenFn[] = [];

export async function initializeDevicesStore(): Promise<void> {
	// Listen for device list updates
	unlistenFunctions.push(
		await listen<DeviceInfo[]>('server:device-list', (event) => {
			devices.set(event.payload);
		})
	);
}

export function cleanupDevicesStore(): void {
	for (const unlisten of unlistenFunctions) {
		unlisten();
	}
	unlistenFunctions = [];
	devices.set([]);
}
