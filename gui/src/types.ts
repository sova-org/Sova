export interface GridSelection {
  start: [number, number];
  end: [number, number];
}

export type ActionTiming = 
  | "Immediate"
  | { EndOfLine: number }
  | { AtBeat: number };

// Re-export frame types for convenience
export type { Frame, FramePosition, DraggedFrame } from './types/frame';
import type { Frame } from './types/frame';

export interface Script {
  content: string;
  lang?: string;
  args?: [string, string]
}

export interface Line {
  frames: Frame[];
  speed_factor: number;
  index: number;
  start_frame?: number;
  end_frame?: number;
  custom_length?: number;
}

export interface Scene {
  lines: Line[];
}

export interface DeviceInfo {
  id: number;
  name: string;
  kind: "Midi" | "Osc" | "Log" | "Other";
  is_connected: boolean;
  address?: string;
}

export interface CompilationError {
  lang: string;
  info: string;
  from?: number;
  to?: number;
}

export type CompilationState = 
  | "NotCompiled"
  | "Compiling"
  | { Compiled: string }
  | { Error: CompilationError };

export interface Snapshot {
  scene: Scene;
  tempo: number;
  beat: number;
  micros: number;
  quantum: number;
}

export type VariableValue = 
  | { Integer: number }
  | { Float: number }
  | { Bool: boolean }
  | { Str: string }
  | [number, number, number] // Decimal as tuple [sign, numerator, denominator]
  | any; // Catch-all for complex types like Dur, Func, Map

export type SchedulerMessage = 
  | "Play"
  | "Stop"
  | "Pause"
  | "Reset"
  | { Seek: number };

export type ClientMessage = 
  | { SchedulerControl: SchedulerMessage }
  | { SetTempo: [number, ActionTiming] }
  | { SetName: string }
  | "GetScene"
  | { SetScene: [Scene, ActionTiming] }
  | { GetLine: number }
  | { SetLines: [[number, Line][], ActionTiming] }
  | { ConfigureLines: [[number, Line][], ActionTiming] }
  | { AddLine: [number, Line, ActionTiming] }
  | { RemoveLine: [number, ActionTiming] }
  | { GetFrame: [number, number] }
  | { SetFrames: [[number, number, Frame][], ActionTiming] }
  | { RemoveFrame: [number, number, ActionTiming] }
  | { AddFrame: [number, number, Frame, ActionTiming] }
  | { RemoveFrame: [number, number, ActionTiming] }
  | "GetClock"
  | "GetPeers"
  | { Chat: string }
  | { SetLineStartFrame: [number, number | null, ActionTiming] }
  | { SetLineEndFrame: [number, number | null, ActionTiming] }
  | "GetSnapshot"
  | { StartedEditingFrame: [number, number] }
  | { StoppedEditingFrame: [number, number] }
  | { TransportStart: ActionTiming }
  | { TransportStop: ActionTiming }
  | "RequestDeviceList"
  | { ConnectMidiDeviceByName: string }
  | { DisconnectMidiDeviceByName: string }
  | { CreateVirtualMidiOutput: string }
  | { AssignDeviceToSlot: [number, string] }
  | { UnassignDeviceFromSlot: number }
  | { CreateOscDevice: [string, string, number] }
  | { RemoveOscDevice: string };

export type ServerMessage = 
  | { Hello: {
      username: string;
      scene: Scene;
      devices: DeviceInfo[];
      peers: string[];
      link_state: [number, number, number, number, boolean];
      is_playing: boolean;
      available_compilers: string[];
      syntax_definitions: Record<string, string>;
    } }
  | { PeersUpdated: string[] }
  | { PeerStartedEditing: [string, number, number] }
  | { PeerStoppedEditing: [string, number, number] }
  | "TransportStarted"
  | "TransportStopped"
  | { LogString: string }
  | { Chat: [string, string] }
  | "Success"
  | { InternalError: string }
  | { ConnectionRefused: string }
  | { Snapshot: Snapshot }
  | { DeviceList: DeviceInfo[] }
  | { ClockState: [number, number, number, number] }
  | { SceneValue: Scene }
  | { LineValues: [number, Line][] }
  | { LineConfigurations: [number, Line][] }
  | { AddLine: [number, Line] }
  | { RemoveLine: [number] }
  | { FrameValues: [number, number, Frame][] }
  | { AddFrame: [number, number, Frame] }
  | { RemoveFrame: [number, number] }
  | { FramePosition: [number, number][] }
  | { GlobalVariablesUpdate: Record<string, VariableValue> }
  | { CompilationUpdate: [number, number, number, CompilationState] };

export interface BuboClient {
  connect: (ip: string, port: number) => Promise<void>;
  disconnect: () => Promise<void>;
  sendMessage: (message: ClientMessage) => Promise<void>;
  getMessages: () => Promise<ServerMessage[]>;
  isConnected: () => Promise<boolean>;
  onMessage: (callback: (message: ServerMessage) => void) => void;
}