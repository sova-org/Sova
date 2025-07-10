import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { BuboClient, ClientMessage, ServerMessage } from './types';

export class BuboCoreClient implements BuboClient {
  private messageHandlers: Array<(message: ServerMessage) => void> = [];
  private unlistenFn?: () => void;

  constructor() {
    this.setupMessageListener();
  }

  private async setupMessageListener() {
    this.unlistenFn = await listen<ServerMessage>('server-message', (event) => {
      const message = event.payload;
      this.messageHandlers.forEach(handler => handler(message));
    });
  }

  async connect(ip: string, port: number): Promise<void> {
    console.log('BuboCoreClient.connect called with:', { ip, port });
    try {
      const result = await invoke('connect_to_server', { ip, port });
      console.log('Connection successful');
      return result;
    } catch (error) {
      console.error('Connection failed:', error);
      throw error;
    }
  }

  async disconnect(): Promise<void> {
    return await invoke('disconnect_from_server');
  }

  async sendMessage(message: ClientMessage): Promise<void> {
    return await invoke('send_message', { message });
  }

  async getMessages(): Promise<ServerMessage[]> {
    return await invoke('get_messages');
  }

  async isConnected(): Promise<boolean> {
    return await invoke('is_connected');
  }

  onMessage(handler: (message: ServerMessage) => void): () => void {
    this.messageHandlers.push(handler);
    return () => {
      const index = this.messageHandlers.indexOf(handler);
      if (index > -1) {
        this.messageHandlers.splice(index, 1);
      }
    };
  }

  destroy() {
    if (this.unlistenFn) {
      this.unlistenFn();
    }
  }
}

export const createBuboClient = (): BuboClient => {
  return new BuboCoreClient();
};

export type { ClientMessage, ServerMessage } from './types';