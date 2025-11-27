import type { Frame } from '$lib/types/protocol';

let clipboard: Frame | null = null;

export function copy(frame: Frame): void {
	clipboard = structuredClone(frame);
}

export function paste(): Frame | null {
	return clipboard ? structuredClone(clipboard) : null;
}

export function hasContent(): boolean {
	return clipboard !== null;
}
