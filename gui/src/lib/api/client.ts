import { invoke } from "@tauri-apps/api/core";
import type {
  ClientMessage,
  ActionTiming,
  Scene,
  Line,
  Frame,
} from "$lib/types/protocol";

export const ActionTiming = {
  immediate: (): ActionTiming => "Immediate",
  endOfLine: (lineId: number): ActionTiming => ({ EndOfLine: lineId }),
  atBeat: (beat: number): ActionTiming => ({ AtBeat: beat }),
  atNextBeat: (): ActionTiming => "AtNextBeat",
};

// Core send function
async function sendMessage(message: ClientMessage): Promise<void> {
  await invoke("send_client_message", { message });
}

// Transport controls
export async function startTransport(
  timing: ActionTiming = ActionTiming.immediate(),
): Promise<void> {
  await sendMessage({ TransportStart: timing });
}

export async function stopTransport(
  timing: ActionTiming = ActionTiming.immediate(),
): Promise<void> {
  await sendMessage({ TransportStop: timing });
}

export async function setTempo(
  tempo: number,
  timing: ActionTiming = ActionTiming.immediate(),
): Promise<void> {
  await sendMessage({ SetTempo: [tempo, timing] });
}

// Scene operations
export async function getScene(): Promise<void> {
  await sendMessage("GetScene");
}

export async function setScene(
  scene: Scene,
  timing: ActionTiming = ActionTiming.immediate(),
): Promise<void> {
  await sendMessage({ SetScene: [scene, timing] });
}

// Line operations
export async function getLine(lineId: number): Promise<void> {
  await sendMessage({ GetLine: lineId });
}

export async function setLines(
  lines: [number, Line][],
  timing: ActionTiming = ActionTiming.immediate(),
): Promise<void> {
  await sendMessage({ SetLines: [lines, timing] });
}

export async function configureLines(
  lines: [number, Line][],
  timing: ActionTiming = ActionTiming.immediate(),
): Promise<void> {
  await sendMessage({ ConfigureLines: [lines, timing] });
}

export async function addLine(
  index: number,
  line: Line,
  timing: ActionTiming = ActionTiming.atNextBeat(),
): Promise<void> {
  await sendMessage({ AddLine: [index, line, timing] });
}

export async function removeLine(
  index: number,
  timing: ActionTiming = ActionTiming.atNextBeat(),
): Promise<void> {
  await sendMessage({ RemoveLine: [index, timing] });
}

// Frame operations
export async function getFrame(lineId: number, frameId: number): Promise<void> {
  await sendMessage({ GetFrame: [lineId, frameId] });
}

function stripCompiledFromFrame(frame: Frame): Frame {
  const { compiled: _, ...scriptWithoutCompiled } = frame.script;
  return { ...frame, script: scriptWithoutCompiled as Frame["script"] };
}

export async function setFrames(
  frames: [number, number, Frame][],
  timing: ActionTiming = ActionTiming.immediate(),
): Promise<void> {
  const cleanedFrames = frames.map(
    ([lineId, frameId, frame]) =>
      [lineId, frameId, stripCompiledFromFrame(frame)] as [
        number,
        number,
        Frame,
      ],
  );
  await sendMessage({ SetFrames: [cleanedFrames, timing] });
}

export async function addFrame(
  lineId: number,
  frameId: number,
  frame: Frame,
  timing: ActionTiming = ActionTiming.atNextBeat(),
): Promise<void> {
  await sendMessage({
    AddFrame: [lineId, frameId, stripCompiledFromFrame(frame), timing],
  });
}

export async function removeFrame(
  lineId: number,
  frameId: number,
  timing: ActionTiming = ActionTiming.atNextBeat(),
): Promise<void> {
  await sendMessage({ RemoveFrame: [lineId, frameId, timing] });
}

// Collaboration
export async function setName(name: string): Promise<void> {
  await sendMessage({ SetName: name });
}

export async function sendChat(message: string): Promise<void> {
  await sendMessage({ Chat: message });
}

export async function getPeers(): Promise<void> {
  await sendMessage("GetPeers");
}

export async function startedEditingFrame(
  lineId: number,
  frameId: number,
): Promise<void> {
  await sendMessage({ StartedEditingFrame: [lineId, frameId] });
}

export async function stoppedEditingFrame(
  lineId: number,
  frameId: number,
): Promise<void> {
  await sendMessage({ StoppedEditingFrame: [lineId, frameId] });
}

// Device management
export async function requestDeviceList(): Promise<void> {
  await sendMessage("RequestDeviceList");
}

export async function connectMidiDevice(name: string): Promise<void> {
  await sendMessage({ ConnectMidiDeviceByName: name });
}

export async function disconnectMidiDevice(name: string): Promise<void> {
  await sendMessage({ DisconnectMidiDeviceByName: name });
}

export async function createVirtualMidiOutput(name: string): Promise<void> {
  await sendMessage({ CreateVirtualMidiOutput: name });
}

export async function assignDeviceToSlot(
  slot: number,
  name: string,
): Promise<void> {
  await sendMessage({ AssignDeviceToSlot: [slot, name] });
}

export async function unassignDeviceFromSlot(slot: number): Promise<void> {
  await sendMessage({ UnassignDeviceFromSlot: slot });
}

export async function createOscDevice(
  name: string,
  host: string,
  port: number,
): Promise<void> {
  await sendMessage({ CreateOscDevice: [name, host, port] });
}

export async function removeOscDevice(name: string): Promise<void> {
  await sendMessage({ RemoveOscDevice: name });
}

// Queries
export async function getClock(): Promise<void> {
  await sendMessage("GetClock");
}

export async function getSnapshot(): Promise<void> {
  await sendMessage("GetSnapshot");
}
