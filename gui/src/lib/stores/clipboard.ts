import type { Frame, Scene } from '$lib/types/protocol';
import { type Selection, getSelectionBounds } from './selection';

export interface ClipboardData {
	frames: (Frame | null)[][];
}

let clipboard: ClipboardData | null = null;

export function copySelection(scene: Scene, sel: Selection): void {
	const bounds = getSelectionBounds(sel);
	const frames: (Frame | null)[][] = [];

	for (let l = bounds.minLine; l <= bounds.maxLine; l++) {
		const row: (Frame | null)[] = [];
		const line = scene.lines[l];
		if (!line) {
			frames.push([]);
			continue;
		}
		for (let f = bounds.minFrame; f <= bounds.maxFrame; f++) {
			const frame = line.frames[f];
			row.push(frame ? structuredClone(frame) : null);
		}
		frames.push(row);
	}

	clipboard = { frames };
}

export function getClipboard(): ClipboardData | null {
	return clipboard ? structuredClone(clipboard) : null;
}

export function hasClipboard(): boolean {
	return clipboard !== null && clipboard.frames.some((row) => row.some((f) => f !== null));
}
