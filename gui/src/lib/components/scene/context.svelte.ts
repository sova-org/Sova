import { getContext, setContext } from 'svelte';

const TIMELINE_CONTEXT_KEY = 'timeline';

export interface TimelineContext {
	pixelsPerBeat: number;
	trackSize: number;
	isVertical: boolean;
}

// Creates and sets a reactive context object - must be called during component init
export function createTimelineContext(initial: TimelineContext): TimelineContext {
	const ctx = $state(initial);
	setContext(TIMELINE_CONTEXT_KEY, ctx);
	return ctx;
}

export function getTimelineContext(): TimelineContext {
	return getContext(TIMELINE_CONTEXT_KEY);
}
