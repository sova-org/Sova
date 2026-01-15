import { get, writable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { SERVER_EVENTS } from '$lib/events';
import { isConnected } from './connectionState';

export interface AudioEngineState {
	running: boolean;
	device: string | null;
	sample_rate: number;
	channels: number;
	active_voices: number;
	sample_paths: string[];
	error: string | null;
	cpu_load: number;
	peak_voices: number;
	max_voices: number;
	schedule_depth: number;
	sample_pool_mb: number;
}

const defaultState: AudioEngineState = {
	running: false,
	device: null,
	sample_rate: 0,
	channels: 0,
	active_voices: 0,
	sample_paths: [],
	error: null,
	cpu_load: 0,
	peak_voices: 0,
	max_voices: 32,
	schedule_depth: 0,
	sample_pool_mb: 0,
};

export const audioEngineState = writable<AudioEngineState>(defaultState);

let unlisten: UnlistenFn | null = null;
let pollInterval: ReturnType<typeof setInterval> | null = null;

export async function initializeAudioEngineStore(): Promise<void> {
	if (unlisten) {
		unlisten();
	}

	unlisten = await listen<AudioEngineState>(
		SERVER_EVENTS.AUDIO_ENGINE_STATE,
		(event) => {
			audioEngineState.set(event.payload);
		}
	);

	pollInterval = setInterval(async () => {
		if (!get(isConnected)) return;
		try {
			await invoke('send_client_message', { message: 'GetAudioEngineState' });
		} catch {
			// Connection may have dropped between check and invoke
		}
	}, 500);
}

export function cleanupAudioEngineStore(): void {
	if (pollInterval) {
		clearInterval(pollInterval);
		pollInterval = null;
	}
	if (unlisten) {
		unlisten();
		unlisten = null;
	}
	audioEngineState.set(defaultState);
}
