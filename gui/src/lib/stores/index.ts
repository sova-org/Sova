// Central store initialization and exports

import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SERVER_EVENTS } from '$lib/events';
import type { HelloPayload } from '$lib/types/protocol';

// Export all stores
export * from './scene';
export * from './transport';
export * from './devices';
export * from './collaboration';
export * from './globalVariables';
export * from './compilation';
export * from './logs';
export * from './notifications';
export * from './config';
export * from './connectionState';

// Import initialization functions
import {
	initializeSceneStore,
	cleanupSceneStore,
	scene
} from './scene';

import {
	initializeTransportStore,
	cleanupTransportStore,
	isPlaying,
	linkState
} from './transport';

import {
	initializeDevicesStore,
	cleanupDevicesStore,
	devices
} from './devices';

import {
	initializeCollaborationStore,
	cleanupCollaborationStore,
	peers
} from './collaboration';

import {
	initializeGlobalVariablesStore,
	cleanupGlobalVariablesStore
} from './globalVariables';

import {
	initializeCompilationStore,
	cleanupCompilationStore
} from './compilation';

import {
	initializeLogsStore,
	cleanupLogsStore
} from './logs';

import {
	initializeNotificationsStore,
	cleanupNotificationsStore
} from './notifications';

let helloUnlisten: UnlistenFn | null = null;

// Initialize all Sova-related stores
export async function initializeSovaStores(): Promise<void> {
	// Listen for Hello message to initialize state
	helloUnlisten = await listen<HelloPayload>(SERVER_EVENTS.HELLO, (event) => {
		const data = event.payload;

		// Initialize scene
		scene.set(data.scene);

		// Initialize transport
		isPlaying.set(data.isPlaying);
		linkState.set(data.linkState);

		// Initialize devices
		devices.set(data.devices);

		// Initialize collaboration
		peers.set(data.peers);

		console.log('Sova state initialized from Hello message');
	});

	// Initialize event listeners for updates
	await Promise.all([
		initializeSceneStore(),
		initializeTransportStore(),
		initializeDevicesStore(),
		initializeCollaborationStore(),
		initializeGlobalVariablesStore(),
		initializeCompilationStore(),
		initializeLogsStore(),
		initializeNotificationsStore()
	]);
}

// Cleanup all Sova-related stores
export function cleanupSovaStores(): void {
	if (helloUnlisten) {
		helloUnlisten();
		helloUnlisten = null;
	}

	cleanupSceneStore();
	cleanupTransportStore();
	cleanupDevicesStore();
	cleanupCollaborationStore();
	cleanupGlobalVariablesStore();
	cleanupCompilationStore();
	cleanupLogsStore();
	cleanupNotificationsStore();
}
