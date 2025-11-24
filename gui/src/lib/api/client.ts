import { invoke } from '@tauri-apps/api/core';
import type {
	ClientMessage,
	ActionTiming,
	Scene,
	Line,
	Frame
} from '$lib/types/protocol';

// Helper to create ActionTiming variants
export const ActionTiming = {
	immediate: (): ActionTiming => ({ type: 'Immediate' }),
	endOfLine: (lineId: number): ActionTiming => ({ type: 'EndOfLine', lineId }),
	atBeat: (beat: number): ActionTiming => ({ type: 'AtBeat', beat })
};

// Core send function
async function sendMessage(message: ClientMessage): Promise<void> {
	await invoke('send_client_message', { message });
}

// Transport controls
export async function startTransport(timing: ActionTiming = ActionTiming.immediate()): Promise<void> {
	await sendMessage({ type: 'TransportStart', timing });
}

export async function stopTransport(timing: ActionTiming = ActionTiming.immediate()): Promise<void> {
	await sendMessage({ type: 'TransportStop', timing });
}

export async function setTempo(tempo: number, timing: ActionTiming = ActionTiming.immediate()): Promise<void> {
	await sendMessage({ type: 'SetTempo', tempo, timing });
}

// Scene operations
export async function getScene(): Promise<void> {
	await sendMessage({ type: 'GetScene' });
}

export async function setScene(scene: Scene, timing: ActionTiming = ActionTiming.immediate()): Promise<void> {
	await sendMessage({ type: 'SetScene', scene, timing });
}

// Line operations
export async function getLine(lineId: number): Promise<void> {
	await sendMessage({ type: 'GetLine', lineId });
}

export async function setLines(
	lines: [number, Line][],
	timing: ActionTiming = ActionTiming.immediate()
): Promise<void> {
	await sendMessage({ type: 'SetLines', lines, timing });
}

export async function configureLines(
	lines: [number, Line][],
	timing: ActionTiming = ActionTiming.immediate()
): Promise<void> {
	await sendMessage({ type: 'ConfigureLines', lines, timing });
}

export async function addLine(
	index: number,
	line: Line,
	timing: ActionTiming = ActionTiming.immediate()
): Promise<void> {
	await sendMessage({ type: 'AddLine', index, line, timing });
}

export async function removeLine(
	index: number,
	timing: ActionTiming = ActionTiming.immediate()
): Promise<void> {
	await sendMessage({ type: 'RemoveLine', index, timing });
}

// Frame operations
export async function getFrame(lineId: number, frameId: number): Promise<void> {
	await sendMessage({ type: 'GetFrame', lineId, frameId });
}

export async function setFrames(
	frames: [number, number, Frame][],
	timing: ActionTiming = ActionTiming.immediate()
): Promise<void> {
	await sendMessage({ type: 'SetFrames', frames, timing });
}

export async function addFrame(
	lineId: number,
	frameId: number,
	frame: Frame,
	timing: ActionTiming = ActionTiming.immediate()
): Promise<void> {
	await sendMessage({ type: 'AddFrame', lineId, frameId, frame, timing });
}

export async function removeFrame(
	lineId: number,
	frameId: number,
	timing: ActionTiming = ActionTiming.immediate()
): Promise<void> {
	await sendMessage({ type: 'RemoveFrame', lineId, frameId, timing });
}

// Collaboration
export async function sendChat(message: string): Promise<void> {
	await sendMessage({ type: 'Chat', message });
}

export async function getPeers(): Promise<void> {
	await sendMessage({ type: 'GetPeers' });
}

export async function startedEditingFrame(lineId: number, frameId: number): Promise<void> {
	await sendMessage({ type: 'StartedEditingFrame', lineId, frameId });
}

export async function stoppedEditingFrame(lineId: number, frameId: number): Promise<void> {
	await sendMessage({ type: 'StoppedEditingFrame', lineId, frameId });
}

// Device management
export async function requestDeviceList(): Promise<void> {
	await sendMessage({ type: 'RequestDeviceList' });
}

export async function connectMidiDevice(name: string): Promise<void> {
	await sendMessage({ type: 'ConnectMidiDeviceByName', name });
}

export async function disconnectMidiDevice(name: string): Promise<void> {
	await sendMessage({ type: 'DisconnectMidiDeviceByName', name });
}

export async function createVirtualMidiOutput(name: string): Promise<void> {
	await sendMessage({ type: 'CreateVirtualMidiOutput', name });
}

export async function assignDeviceToSlot(slot: number, name: string): Promise<void> {
	await sendMessage({ type: 'AssignDeviceToSlot', slot, name });
}

export async function unassignDeviceFromSlot(slot: number): Promise<void> {
	await sendMessage({ type: 'UnassignDeviceFromSlot', slot });
}

export async function createOscDevice(name: string, host: string, port: number): Promise<void> {
	await sendMessage({ type: 'CreateOscDevice', name, host, port });
}

export async function removeOscDevice(name: string): Promise<void> {
	await sendMessage({ type: 'RemoveOscDevice', name });
}

// Queries
export async function getClock(): Promise<void> {
	await sendMessage({ type: 'GetClock' });
}

export async function getSnapshot(): Promise<void> {
	await sendMessage({ type: 'GetSnapshot' });
}

// Scheduler control (low-level commands)
export async function goToFrame(
	lineId: number,
	frameId: number,
	timing: ActionTiming = ActionTiming.immediate()
): Promise<void> {
	await sendMessage({
		type: 'SchedulerControl',
		message: { type: 'GoToFrame', lineId, frameId, timing }
	});
}

export async function setScript(
	lineId: number,
	frameId: number,
	content: string,
	lang: string,
	timing: ActionTiming = ActionTiming.immediate()
): Promise<void> {
	await sendMessage({
		type: 'SchedulerControl',
		message: { type: 'SetScript', lineId, frameId, content, lang, timing }
	});
}

export async function shutdownScheduler(): Promise<void> {
	await sendMessage({
		type: 'SchedulerControl',
		message: { type: 'Shutdown' }
	});
}
