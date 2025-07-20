export interface GridSelection {
  start: [number, number];
  end: [number, number];
}

export type ActionTiming = 
  | "Immediate"
  | "EndOfScene"
  | { AtBeat: number };

// Re-export frame types for convenience
export type { Frame, FramePosition, DraggedFrame, PastedFrameData } from './types/frame';
import type { PastedFrameData } from './types/frame';

export interface Script {
  content: string;
  lang: string;
  index: number;
}

export interface Line {
  frames: number[];
  enabled_frames: boolean[];
  scripts: Script[];
  frame_names: (string | null)[];
  frame_repetitions: number[];
  speed_factor: number;
  index: number;
  start_frame: number | undefined;
  end_frame: number | undefined;
  custom_length: number | undefined;
}

export interface Scene {
  length: number;
  lines: Line[];
}

export interface DeviceInfo {
  id: number;
  name: string;
  kind: "Midi" | "Osc" | "Log" | "Other";
  is_connected: boolean;
  address: string | undefined;
}

export interface CompilationError {
  lang: string;
  info: string;
  from: number | undefined;
  to: number | undefined;
}

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
  | { EnableFrames: [number, number[], ActionTiming] }
  | { DisableFrames: [number, number[], ActionTiming] }
  | { SetScript: [number, number, string, ActionTiming] }
  | { GetScript: [number, number] }
  | "GetScene"
  | { SetScene: [Scene, ActionTiming] }
  | "GetClock"
  | "GetPeers"
  | { Chat: string }
  | { UpdateLineFrames: [number, number[], ActionTiming] }
  | { InsertFrame: [number, number, number, ActionTiming] }
  | { RemoveFrame: [number, number, ActionTiming] }
  | { SetLineStartFrame: [number, number | null, ActionTiming] }
  | { SetLineEndFrame: [number, number | null, ActionTiming] }
  | "GetSnapshot"
  | { UpdateGridSelection: GridSelection }
  | { StartedEditingFrame: [number, number] }
  | { StoppedEditingFrame: [number, number] }
  | "GetSceneLength"
  | { SetSceneLength: [number, ActionTiming] }
  | { SetLineLength: [number, number | null, ActionTiming] }
  | { SetLineSpeedFactor: [number, number, ActionTiming] }
  | { TransportStart: ActionTiming }
  | { TransportStop: ActionTiming }
  | "RequestDeviceList"
  | { ConnectMidiDeviceById: number }
  | { DisconnectMidiDeviceById: number }
  | { ConnectMidiDeviceByName: string }
  | { DisconnectMidiDeviceByName: string }
  | { CreateVirtualMidiOutput: string }
  | { AssignDeviceToSlot: [number, string] }
  | { UnassignDeviceFromSlot: number }
  | { CreateOscDevice: [string, string, number] }
  | { RemoveOscDevice: string }
  | { DuplicateFrameRange: {
      src_line_idx: number;
      src_frame_start_idx: number;
      src_frame_end_idx: number;
      target_insert_idx: number;
      timing: ActionTiming;
    } }
  | { RemoveFramesMultiLine: {
      lines_and_indices: [number, number[]][];
      timing: ActionTiming;
    } }
  | { RequestDuplicationData: {
      src_top: number;
      src_left: number;
      src_bottom: number;
      src_right: number;
      target_cursor_row: number;
      target_cursor_col: number;
      insert_before: boolean;
      timing: ActionTiming;
    } }
  | { PasteDataBlock: {
      data: PastedFrameData[][];
      target_row: number;
      target_col: number;
      timing: ActionTiming;
    } }
  | { SetFrameName: [number, number, string | null, ActionTiming] }
  | { SetScriptLanguage: [number, number, string, ActionTiming] }
  | { SetFrameRepetitions: [number, number, number, ActionTiming] };

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
  | { ConnectionRefused: string }
  | "Success"
  | { InternalError: string }
  | { SceneValue: Scene }
  | { ScriptContent: {
      line_idx: number;
      frame_idx: number;
      content: string;
    } }
  | { ScriptCompiled: {
      line_idx: number;
      frame_idx: number;
    } }
  | { CompilationErrorOccurred: CompilationError }
  | { SceneLength: number }
  | "TransportStarted"
  | "TransportStopped"
  | { ClockState: [number, number, number, number] }
  | { FramePosition: [number, number, number][] }
  | { DeviceList: DeviceInfo[] }
  | { PeersUpdated: string[] }
  | { PeerGridSelectionUpdate: [string, GridSelection] }
  | { PeerStartedEditing: [string, number, number] }
  | { PeerStoppedEditing: [string, number, number] }
  | { Chat: string }
  | { LogString: string }
  | { Snapshot: Snapshot }
  | { GlobalVariablesUpdate: Record<string, VariableValue> };

export interface BuboClient {
  connect: (ip: string, port: number) => Promise<void>;
  disconnect: () => Promise<void>;
  sendMessage: (message: ClientMessage) => Promise<void>;
  getMessages: () => Promise<ServerMessage[]>;
  isConnected: () => Promise<boolean>;
  onMessage: (callback: (message: ServerMessage) => void) => void;
}