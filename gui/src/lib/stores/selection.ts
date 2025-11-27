import { writable } from 'svelte/store';
import type { Scene } from '$lib/types/protocol';

export interface Selection {
	anchor: { lineId: number; frameId: number };
	focus: { lineId: number; frameId: number };
}

export const selection = writable<Selection | null>(null);

export function getSelectionBounds(sel: Selection) {
	return {
		minLine: Math.min(sel.anchor.lineId, sel.focus.lineId),
		maxLine: Math.max(sel.anchor.lineId, sel.focus.lineId),
		minFrame: Math.min(sel.anchor.frameId, sel.focus.frameId),
		maxFrame: Math.max(sel.anchor.frameId, sel.focus.frameId)
	};
}

export function isFrameInSelection(
	sel: Selection | null,
	scene: Scene | null,
	lineId: number,
	frameId: number
): boolean {
	if (!sel || !scene) return false;
	const line = scene.lines[lineId];
	if (!line || frameId >= line.frames.length) return false;

	const bounds = getSelectionBounds(sel);
	return (
		lineId >= bounds.minLine &&
		lineId <= bounds.maxLine &&
		frameId >= bounds.minFrame &&
		frameId <= bounds.maxFrame
	);
}

export function selectFrame(lineId: number, frameId: number): void {
	selection.set({
		anchor: { lineId, frameId },
		focus: { lineId, frameId }
	});
}

export function extendSelection(lineId: number, frameId: number): void {
	selection.update((s) =>
		s
			? { ...s, focus: { lineId, frameId } }
			: { anchor: { lineId, frameId }, focus: { lineId, frameId } }
	);
}

export function collapseToFocus(): void {
	selection.update((s) => (s ? { anchor: s.focus, focus: s.focus } : null));
}
