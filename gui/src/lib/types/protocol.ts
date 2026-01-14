// TypeScript types mirroring the Rust server protocol types

export type SyncTime = number; // u64 microseconds in Rust

// Log severity levels (matches Rust Severity enum)
export type Severity = "Fatal" | "Error" | "Warn" | "Info" | "Debug";

// Structured log message (matches Rust LogMessage struct)
export interface LogMessage {
  level: Severity;
  event: unknown | null; // ConcreteEvent - simplified as unknown for now
  msg: string;
}

// ActionTiming for scheduling changes
export type ActionTiming =
  | "Immediate"
  | { EndOfLine: number }
  | { AtBeat: number }
  | "AtNextBeat"
  | "AtNextPhase";

// PlaybackState for transport state
export type PlaybackState =
  | "Stopped"
  | { Starting: number } // target beat
  | "Playing";

// Variable types - untagged in Rust, so raw primitives in JSON
export type VariableValue =
  | number
  | string
  | boolean
  | number[] // Decimal as [sign, num, den]
  | Record<string, unknown> // Map
  | unknown[]; // Vec

export interface VariableStore {
  [key: string]: VariableValue;
}

// Compilation error
export interface CompilationError {
  lang: string;
  info: string;
  from: number;
  to: number;
}

// Compilation state
// Note: Compiled is a string because Program has #[serde(skip)]
export type CompilationState =
  | "NotCompiled"
  | "Compiling"
  | "Compiled"
  | "Parsed"
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
export type DeviceKind =
  | "Midi"
  | "VirtualMidi"
  | "Osc"
  | "Log"
  | "AudioEngine"
  | "Other";

export type DeviceDirection = "Input" | "Output";

export interface DeviceInfo {
  slot_id: number | null;
  name: string;
  kind: DeviceKind;
  direction: DeviceDirection;
  is_connected: boolean;
  address: string | null;
  is_missing: boolean;
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

// Snapshot - complete server state
export interface Snapshot {
  scene: Scene;
  tempo: number;
  beat: number;
  micros: SyncTime;
  quantum: number;
  devices?: DeviceInfo[];
}

// Audio engine state
export interface AudioEngineState {
  running: boolean;
  device: string | null;
  sample_rate: number;
  channels: number;
  active_voices: number;
  sample_paths: string[];
  error: string | null;
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
  audioEngineState: AudioEngineState;
}

export interface ChatPayload {
  user: string;
  message: string;
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
  scriptId: string; // String to avoid JS precision loss for u64
  state: CompilationState;
}

// Client message types for sending to server
export type ClientMessage =
  | { TransportStart: ActionTiming }
  | { TransportStop: ActionTiming }
  | { SetTempo: [number, ActionTiming] }
  | "GetScene"
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
  | "GetPeers"
  | { Chat: string }
  | { StartedEditingFrame: [number, number] }
  | { StoppedEditingFrame: [number, number] }
  | "RequestDeviceList"
  | { ConnectMidiDeviceByName: string }
  | { DisconnectMidiDeviceByName: string }
  | { CreateVirtualMidiOutput: string }
  | { AssignDeviceToSlot: [number, string] }
  | { UnassignDeviceFromSlot: number }
  | { CreateOscDevice: [string, string, number] }
  | { RemoveOscDevice: string }
  | "GetClock"
  | "GetSnapshot"
  | { RestoreDevices: DeviceInfo[] }
  | "GetAudioEngineState";
