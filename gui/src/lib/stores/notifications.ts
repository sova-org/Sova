import { writable, type Writable } from 'svelte/store';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { SERVER_EVENTS } from '$lib/events';

export type NotificationType = 'success' | 'error' | 'info';

export interface Notification {
	id: number;
	type: NotificationType;
	message: string;
	timestamp: number;
}

let nextId = 0;

// Active notifications
export const notifications: Writable<Notification[]> = writable([]);

// Pure function to add notification
function addNotification(
	current: Notification[],
	type: NotificationType,
	message: string
): Notification[] {
	return [...current, { id: nextId++, type, message, timestamp: Date.now() }];
}

// Pure function to remove notification
function removeNotification(current: Notification[], id: number): Notification[] {
	return current.filter((n) => n.id !== id);
}

// Helper to add notification with auto-dismiss
export function notify(type: NotificationType, message: string, duration = 5000): void {
	const id = nextId;
	notifications.update(($n) => addNotification($n, type, message));

	if (duration > 0) {
		setTimeout(() => {
			notifications.update(($n) => removeNotification($n, id));
		}, duration);
	}
}

// Helper to dismiss notification
export function dismissNotification(id: number): void {
	notifications.update(($n) => removeNotification($n, id));
}

let unlistenFunctions: UnlistenFn[] = [];

export async function initializeNotificationsStore(): Promise<void> {
	// Listen for success
	unlistenFunctions.push(
		await listen(SERVER_EVENTS.SUCCESS, () => {
			notify('success', 'Operation successful', 3000);
		})
	);

	// Listen for errors
	unlistenFunctions.push(
		await listen<string>(SERVER_EVENTS.ERROR, (event) => {
			notify('error', `Error: ${event.payload}`, 10000);
		})
	);

	// Listen for connection refused
	unlistenFunctions.push(
		await listen<string>(SERVER_EVENTS.CONNECTION_REFUSED, (event) => {
			notify('error', `Connection refused: ${event.payload}`, 10000);
		})
	);
}

export function cleanupNotificationsStore(): void {
	for (const unlisten of unlistenFunctions) {
		unlisten();
	}
	unlistenFunctions = [];
	notifications.set([]);
}

// Export for testing
export { addNotification, removeNotification };
