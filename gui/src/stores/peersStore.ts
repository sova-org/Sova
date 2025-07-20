import { map } from 'nanostores';
import type { ServerMessage } from '../types';

export interface PeersState {
  peerList: string[];
  peerSelections: Map<string, { start: [number, number], end: [number, number] }>;
  peerEditing: Map<string, [number, number]>; // [line, frame]
}

export const peersStore = map<PeersState>({
  peerList: [],
  peerSelections: new Map(),
  peerEditing: new Map(),
});

// Peer message handlers
export const handlePeerMessage = (message: ServerMessage) => {
  if (typeof message === 'object' && message !== null) {
    switch (true) {
      case 'Hello' in message:
        peersStore.setKey('peerList', message.Hello.peers);
        return true;
      
      case 'PeersUpdated' in message:
        peersStore.setKey('peerList', message.PeersUpdated);
        return true;
      
      case 'PeerGridSelectionUpdate' in message:
        const [peerName, selection] = message.PeerGridSelectionUpdate;
        const peerSelections = new Map(peersStore.get().peerSelections);
        peerSelections.set(peerName, selection);
        peersStore.setKey('peerSelections', peerSelections);
        return true;
      
      case 'PeerStartedEditing' in message:
        const [startPeer, startLine, startFrame] = message.PeerStartedEditing;
        const peerEditingStart = new Map(peersStore.get().peerEditing);
        peerEditingStart.set(startPeer, [startLine, startFrame]);
        peersStore.setKey('peerEditing', peerEditingStart);
        return true;
      
      case 'PeerStoppedEditing' in message:
        const [stopPeer] = message.PeerStoppedEditing;
        const peerEditingStop = new Map(peersStore.get().peerEditing);
        peerEditingStop.delete(stopPeer);
        peersStore.setKey('peerEditing', peerEditingStop);
        return true;
    }
  }
  
  return false;
};

// Helper functions
export const getPeerList = () => peersStore.get().peerList;
export const getPeerSelections = () => peersStore.get().peerSelections;
export const getPeerEditing = () => peersStore.get().peerEditing;
export const isPeerEditing = (peer: string) => peersStore.get().peerEditing.has(peer);
export const getPeerEditingFrame = (peer: string) => peersStore.get().peerEditing.get(peer);