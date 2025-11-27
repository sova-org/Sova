// TypeScript types mirroring the Rust server protocol types

export type SyncTime = number; // u64 microseconds in Rust

// Log severity levels (matches Rust Severity enum)
export type Severity = 'Fatal' | 'Error' | 'Warn' | 'Info' | 'Debug';

// Structured log message (matches Rust LogMessage struct)
export interface LogMessage {
	level: Severity;
	event: unknown | null; // ConcreteEvent - simplified as unknown for now
	msg: string;
}

// ActionTiming for scheduling changes (matches Rust enum serialization)
export type ActionTiming =
	| 'Immediate'
	| { EndOfLine: number }
	| { AtBeat: number };

// Variable types (matches Rust enum serialization)
export type VariableValue =
	| { Integer: number }
	| { Float: number }
	| { Str: string }
	| { Bool: boolean };

export interface VariableStore {
	[key: string]: VariableValue;
}

// Compilation error (matches Rust CompilationError struct)
export interface CompilationError {
	lang: string;
	info: string;
	from: number;
	to: number;
}

// Compilation state (matches Rust enum serialization)
// Note: Compiled is a string because Program has #[serde(skip)]
export type CompilationState =
	| 'NotCompiled'
	| 'Compiling'
	| 'Compiled'
	| { Error: CompilationError };

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

// Frame position: each element is (frame_idx, rep_idx), indexed by line
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
	scriptId: string;  // String to avoid JS precision loss for u64
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

// Client message types for sending to server (matches Rust enum serialization)
export type ClientMessage =
	| { SchedulerControl: SchedulerMessage }
	| { TransportStart: ActionTiming }
	| { TransportStop: ActionTiming }
	| { SetTempo: [number, ActionTiming] }
	| 'GetScene'
	| { SetScene: [Scene, ActionTiming] }
	| { GetLine: number }
	| { SetLines: [[number, Line][], ActionTiming] }
	| { ConfigureLines: [[number, Line][], ActionTiming] }
	| { AddLine: [number, Line, ActionTiming] }
	| { RemoveLine: [number, ActionTiming] }
	| { GetFrame: [number, number] }
	| { SetFrames: [[number, number, Frame][], ActionTiming] }
	| { AddFrame: [number, number, Frame, ActionTiming] }
	| { RemoveFrame: [number, number, ActionTiming] }
	| { SetName: string }
	| 'GetPeers'
	| { Chat: string }
	| { StartedEditingFrame: [number, number] }
	| { StoppedEditingFrame: [number, number] }
	| 'RequestDeviceList'
	| { ConnectMidiDeviceByName: string }
	| { DisconnectMidiDeviceByName: string }
	| { CreateVirtualMidiOutput: string }
	| { AssignDeviceToSlot: [number, string] }
	| { UnassignDeviceFromSlot: number }
	| { CreateOscDevice: [string, string, number] }
	| { RemoveOscDevice: string }
	| 'GetClock'
	| 'GetSnapshot';
