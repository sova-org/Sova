import { writable, derived, type Writable, type Readable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { type EditorConfig } from './editorConfig';
import { themes, type Theme } from '$lib/themes';
import { initializeConnectionListener, cleanupConnectionListener } from './connectionState';

export interface ClientConfig {
	ip: string;
	port: number;
	nickname: string;
}

export interface Config {
	editor: EditorConfig;
	appearance: {
		theme: string;
		transparency: number;
	};
	client: ClientConfig;
}

export const config: Writable<Config | null> = writable(null);

export const editorConfig: Readable<EditorConfig | null> = derived(
	config,
	($config) => $config?.editor ?? null
);

export const currentThemeName: Readable<string> = derived(
	config,
	($config) => $config?.appearance.theme ?? 'monokai'
);

export const currentTransparency: Readable<number> = derived(
	config,
	($config) => $config?.appearance.transparency ?? 100
);

export const currentTheme: Readable<Theme> = derived(currentThemeName, ($name) => {
	const theme = themes[$name];
	if (!theme) {
		throw new Error(
			`Invalid theme "${$name}" specified in config. Available themes: ${Object.keys(themes).slice(0, 10).join(', ')}...`
		);
	}
	return theme;
});

export const clientConfig: Readable<ClientConfig | null> = derived(
	config,
	($config) => $config?.client ?? null
);

let unlisten: UnlistenFn | null = null;

export async function initializeConfig(): Promise<void> {
	try {
		const loadedConfig = await invoke<Config>('get_config');
		config.set(loadedConfig);
	} catch (error) {
		// Failed to load config - will use defaults
	}

	if (unlisten) return;

	unlisten = await listen<Config>('config-update', (event) => {
		config.set(event.payload);
	});
}

export function cleanupConfig() {
	if (unlisten) {
		unlisten();
		unlisten = null;
	}
}

export async function initializeApp(): Promise<void> {
	await initializeConfig();
	await initializeConnectionListener();
}

export function cleanupApp(): void {
	cleanupConfig();
	cleanupConnectionListener();
}
