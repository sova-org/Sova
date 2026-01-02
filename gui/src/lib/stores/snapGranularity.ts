import { writable } from "svelte/store";

export type SnapValue = 1 | 0.5 | 0.25 | 0.125;

export const snapGranularity = writable<SnapValue>(0.25);

export const SNAP_OPTIONS: { value: SnapValue; label: string }[] = [
    { value: 1, label: "1" },
    { value: 0.5, label: "1/2" },
    { value: 0.25, label: "1/4" },
    { value: 0.125, label: "1/8" },
];
