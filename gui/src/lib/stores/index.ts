// Central store initialization and exports

import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { SERVER_EVENTS } from "$lib/events";
import type { HelloPayload } from "$lib/types/protocol";

// Export all stores
export * from "./scene";
export * from "./transport";
export * from "./devices";
export * from "./collaboration";
export * from "./globalVariables";
export * from "./compilation";
export * from "./logs";
export * from "./notifications";
export * from "./config";
export * from "./connectionState";
export * from "./languages";
export * from "./localEdits";
export * from "./projects";
export * from "./projectsUI";
export * from "./serverState";
export * from "./audioEngineState";

// Import initialization functions
import { initializeSceneStore, cleanupSceneStore, scene } from "./scene";

import {
  initializeTransportStore,
  cleanupTransportStore,
  playbackState,
  linkState,
} from "./transport";

import {
  initializeDevicesStore,
  cleanupDevicesStore,
  devices,
} from "./devices";

import {
  initializeCollaborationStore,
  cleanupCollaborationStore,
  peers,
} from "./collaboration";

import {
  initializeGlobalVariablesStore,
  cleanupGlobalVariablesStore,
} from "./globalVariables";

import {
  initializeCompilationStore,
  cleanupCompilationStore,
} from "./compilation";

import {
  initializeNotificationsStore,
  cleanupNotificationsStore,
} from "./notifications";

import { setAvailableLanguages, cleanupLanguagesStore } from "./languages";

import {
  initializeLocalEditsStore,
  cleanupLocalEditsStore,
} from "./localEdits";

import { initializeProjectsStore, cleanupProjectsStore } from "./projects";

import { initializeConfig, cleanupConfig } from "./config";

import {
  initializeConnectionListener,
  cleanupConnectionListener,
} from "./connectionState";

import { initializeLogsStore, cleanupLogsStore } from "./logs";

import {
  initializeServerStateListener,
  cleanupServerStateListener,
} from "./serverState";

import {
  initializeAudioEngineStore,
  cleanupAudioEngineStore,
  setAudioEngineStatus,
} from "./audioEngineState";

import { initializeLanguages } from "../../languages";

let helloUnlisten: UnlistenFn | null = null;
let sovaStoresInitialized = false;

// Initialize all Sova-related stores
export async function initializeSovaStores(): Promise<void> {
  if (sovaStoresInitialized) {
    return;
  }
  // Listen for Hello message to initialize state
  helloUnlisten = await listen<HelloPayload>(SERVER_EVENTS.HELLO, (event) => {
    const data = event.payload;

    // Initialize scene
    scene.set(data.scene);

    // Initialize transport
    playbackState.set(data.isPlaying ? "Playing" : "Stopped");
    linkState.set(data.linkState);

    // Initialize devices
    devices.set(data.devices);

    // Initialize collaboration
    peers.set(data.peers);

    // Initialize available languages
    setAvailableLanguages(data.availableLanguages);

    // Initialize audio engine status
    if (data.audioEngineStatus) {
      setAudioEngineStatus(data.audioEngineStatus);
    }
  });

  // Initialize event listeners for updates
  await Promise.all([
    initializeSceneStore(),
    initializeTransportStore(),
    initializeDevicesStore(),
    initializeCollaborationStore(),
    initializeGlobalVariablesStore(),
    initializeCompilationStore(),
    initializeNotificationsStore(),
    initializeLocalEditsStore(),
    initializeProjectsStore(),
    initializeAudioEngineStore(),
  ]);

  sovaStoresInitialized = true;
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
  cleanupNotificationsStore();
  cleanupLanguagesStore();
  cleanupLocalEditsStore();
  cleanupProjectsStore();
  cleanupAudioEngineStore();

  sovaStoresInitialized = false;
}

// Initialize app-level stores (config, connection, logs, server state)
export async function initializeApp(): Promise<void> {
  initializeLanguages();
  await initializeConfig();
  await initializeConnectionListener();
  await initializeServerStateListener();
  await initializeLogsStore();
}

// Cleanup app-level stores
export function cleanupApp(): void {
  cleanupConfig();
  cleanupConnectionListener();
  cleanupServerStateListener();
  cleanupLogsStore();
}
