// TypeScript types mirroring the Rust server protocol types

export type SyncTime = number; // u64 microseconds in Rust

// ActionTiming for scheduling changes
export type ActionTiming =
	| { type: 'Immediate' }
	| { type: 'EndOfLine'; lineId: number }
	| { type: 'AtBeat'; beat: number };

// Variable types
export type VariableValue =
	| { type: 'Int'; value: number }
	| { type: 'Float'; value: number }
	| { type: 'String'; value: string }
	| { type: 'Bool'; value: boolean };

export interface VariableStore {
	[key: string]: VariableValue;
}

// Compilation state
export type CompilationState =
	| { type: 'NotCompiled' }
	| { type: 'Compiling' }
	| { type: 'Compiled' }
	| { type: 'Failed'; error: string };

// Script
export interface Script {
	content: string;
	lang: string;
	compiled: CompilationState;
	args: { [key: string]: string };
}

// Frame
export interface Frame {
	duration: number; // In beats
	repetitions: number;
	enabled: boolean;
	script: Script;
	name: string | null;
	vars: VariableStore;
}

// Line
export interface Line {
	frames: Frame[];
	speed_factor: number;
	vars: VariableStore;
	start_frame: number | null;
	end_frame: number | null;
	custom_length: number | null;
}

// Scene
export interface Scene {
	lines: Line[];
}

// Device types
export type DeviceKind = 'Midi' | 'Osc' | 'Log' | 'AudioEngine' | 'Other';

export interface DeviceInfo {
	id: number;
	name: string;
	kind: DeviceKind;
	is_connected: boolean;
	address: string | null;
}

// Link state
export interface LinkState {
	tempo: number;
	beat: number;
	phase: number;
	numPeers: number;
	isEnabled: boolean;
}

// Clock state
export interface ClockState {
	tempo: number;
	beat: number;
	micros: SyncTime;
	quantum: number;
}

// Frame position (line_idx, frame_idx)
export type FramePosition = [number, number];

// Snapshot (simplified - actual structure may be more complex)
export interface Snapshot {
	scene: Scene;
	// Additional fields as needed
}

// Server event payloads
export interface HelloPayload {
	username: string;
	scene: Scene;
	devices: DeviceInfo[];
	peers: string[];
	linkState: LinkState;
	isPlaying: boolean;
	availableLanguages: string[];
	syntaxDefinitions: { [key: string]: string };
}

export interface ChatPayload {
	user: string;
	message: string;
}

export interface PeerEditingPayload {
	user: string;
	lineId: number;
	frameId: number;
}

export interface AddLinePayload {
	index: number;
	line: Line;
}

export interface AddFramePayload {
	lineId: number;
	frameId: number;
	frame: Frame;
}

export interface RemoveFramePayload {
	lineId: number;
	frameId: number;
}

export interface CompilationUpdatePayload {
	lineId: number;
	frameId: number;
	scriptId: number;
	state: CompilationState;
}

// Scheduler message types (low-level control)
export type SchedulerMessage =
	| { type: 'SetScene'; scene: Scene; timing: ActionTiming }
	| { type: 'SetLines'; lines: [number, Line][]; timing: ActionTiming }
	| { type: 'ConfigureLines'; lines: [number, Line][]; timing: ActionTiming }
	| { type: 'AddLine'; index: number; line: Line; timing: ActionTiming }
	| { type: 'RemoveLine'; index: number; timing: ActionTiming }
	| { type: 'GoToFrame'; lineId: number; frameId: number; timing: ActionTiming }
	| { type: 'SetFrames'; frames: [number, number, Frame][]; timing: ActionTiming }
	| { type: 'AddFrame'; lineId: number; frameId: number; frame: Frame; timing: ActionTiming }
	| { type: 'RemoveFrame'; lineId: number; frameId: number; timing: ActionTiming }
	| { type: 'SetScript'; lineId: number; frameId: number; content: string; lang: string; timing: ActionTiming }
	| { type: 'SetTempo'; tempo: number; timing: ActionTiming }
	| { type: 'TransportStart'; timing: ActionTiming }
	| { type: 'TransportStop'; timing: ActionTiming }
	| { type: 'CompilationUpdate'; lineId: number; frameId: number; scriptId: number; state: CompilationState }
	| { type: 'Shutdown' };

// Client message types for sending to server
export type ClientMessage =
	| { type: 'SchedulerControl'; message: SchedulerMessage }
	| { type: 'TransportStart'; timing: ActionTiming }
	| { type: 'TransportStop'; timing: ActionTiming }
	| { type: 'SetTempo'; tempo: number; timing: ActionTiming }
	| { type: 'GetScene' }
	| { type: 'SetScene'; scene: Scene; timing: ActionTiming }
	| { type: 'GetLine'; lineId: number }
	| { type: 'SetLines'; lines: [number, Line][]; timing: ActionTiming }
	| { type: 'ConfigureLines'; lines: [number, Line][]; timing: ActionTiming }
	| { type: 'AddLine'; index: number; line: Line; timing: ActionTiming }
	| { type: 'RemoveLine'; index: number; timing: ActionTiming }
	| { type: 'GetFrame'; lineId: number; frameId: number }
	| { type: 'SetFrames'; frames: [number, number, Frame][]; timing: ActionTiming }
	| { type: 'AddFrame'; lineId: number; frameId: number; frame: Frame; timing: ActionTiming }
	| { type: 'RemoveFrame'; lineId: number; frameId: number; timing: ActionTiming }
	| { type: 'SetName'; name: string }
	| { type: 'GetPeers' }
	| { type: 'Chat'; message: string }
	| { type: 'StartedEditingFrame'; lineId: number; frameId: number }
	| { type: 'StoppedEditingFrame'; lineId: number; frameId: number }
	| { type: 'RequestDeviceList' }
	| { type: 'ConnectMidiDeviceByName'; name: string }
	| { type: 'DisconnectMidiDeviceByName'; name: string }
	| { type: 'CreateVirtualMidiOutput'; name: string }
	| { type: 'AssignDeviceToSlot'; slot: number; name: string }
	| { type: 'UnassignDeviceFromSlot'; slot: number }
	| { type: 'CreateOscDevice'; name: string; host: string; port: number }
	| { type: 'RemoveOscDevice'; name: string }
	| { type: 'GetClock' }
	| { type: 'GetSnapshot' };
