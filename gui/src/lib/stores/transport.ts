import { writable, derived, type Writable, type Readable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { LinkState, ClockState, FramePosition } from '$lib/types/protocol';

// Transport state
export const isPlaying: Writable<boolean> = writable(false);

// Clock state
export const clockState: Writable<ClockState | null> = writable(null);

// Link state (Ableton Link)
export const linkState: Writable<LinkState | null> = writable(null);

// Frame positions (line_idx, frame_idx)
export const framePositions: Writable<FramePosition[]> = writable([]);

// Derived stores
export const currentTempo: Readable<number | null> = derived(
	clockState,
	($clock) => $clock?.tempo ?? null
);

export const currentBeat: Readable<number | null> = derived(
	clockState,
	($clock) => $clock?.beat ?? null
);

export const currentQuantum: Readable<number | null> = derived(
	clockState,
	($clock) => $clock?.quantum ?? null
);

export const linkTempo: Readable<number | null> = derived(
	linkState,
	($link) => $link?.tempo ?? null
);

export const linkPeerCount: Readable<number | null> = derived(
	linkState,
	($link) => $link?.numPeers ?? null
);

export const isLinkEnabled: Readable<boolean> = derived(
	linkState,
	($link) => $link?.isEnabled ?? false
);

// Helper to get current frame for a specific line
export function getCurrentFrameForLine(lineId: number): Readable<number | null> {
	return derived(framePositions, ($positions) => {
		const position = $positions[lineId];
		return position ? position[1] : null;
	});
}

let unlistenFunctions: UnlistenFn[] = [];

export async function initializeTransportStore(): Promise<void> {
	// Listen for transport started
	unlistenFunctions.push(
		await listen('server:transport-started', () => {
			isPlaying.set(true);
		})
	);

	// Listen for transport stopped
	unlistenFunctions.push(
		await listen('server:transport-stopped', () => {
			isPlaying.set(false);
		})
	);

	// Listen for clock state updates
	unlistenFunctions.push(
		await listen<ClockState>('server:clock-state', (event) => {
			clockState.set(event.payload);
		})
	);

	// Listen for frame position updates
	unlistenFunctions.push(
		await listen<FramePosition[]>('server:frame-position', (event) => {
			framePositions.set(event.payload);
		})
	);
}

export function cleanupTransportStore(): void {
	for (const unlisten of unlistenFunctions) {
		unlisten();
	}
	unlistenFunctions = [];
	isPlaying.set(false);
	clockState.set(null);
	linkState.set(null);
	framePositions.set([]);
}
