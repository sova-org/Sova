import React, { useState, useEffect, useRef } from 'react';
import { BuboCoreClient } from './client';
import type { ServerMessage } from './types';

export const TestClient: React.FC = () => {
  const [connected, setConnected] = useState(false);
  const [ip, setIp] = useState('127.0.0.1');
  const [port, setPort] = useState(8080);
  const [messages, setMessages] = useState<ServerMessage[]>([]);
  const [log, setLog] = useState<string[]>([]);
  const clientRef = useRef<BuboCoreClient | null>(null);

  useEffect(() => {
    const client = new BuboCoreClient();
    clientRef.current = client;

    const unsubscribe = client.onMessage((message) => {
      setMessages(prev => [...prev, message]);
      addLog(`Received: ${JSON.stringify(message)}`);
    });

    return () => {
      unsubscribe();
      client.destroy();
    };
  }, []);

  const addLog = (message: string) => {
    setLog(prev => [...prev, `[${new Date().toLocaleTimeString()}] ${message}`]);
  };

  const connect = async () => {
    if (!clientRef.current) return;
    
    try {
      await clientRef.current.connect(ip, port);
      setConnected(true);
      addLog(`Connected to ${ip}:${port}`);
    } catch (error) {
      addLog(`Failed to connect: ${error}`);
    }
  };

  const disconnect = async () => {
    if (!clientRef.current) return;
    
    try {
      await clientRef.current.disconnect();
      setConnected(false);
      addLog('Disconnected');
    } catch (error) {
      addLog(`Failed to disconnect: ${error}`);
    }
  };

  const sendTestMessage = async (messageType: string) => {
    if (!clientRef.current) return;
    
    try {
      let message;
      switch (messageType) {
        case 'getName':
          message = { SetName: 'GUI Client' };
          break;
        case 'getScene':
          message = 'GetScene';
          break;
        case 'getClock':
          message = 'GetClock';
          break;
        case 'getPeers':
          message = 'GetPeers';
          break;
        case 'getDevices':
          message = 'RequestDeviceList';
          break;
        case 'transportStart':
          message = { TransportStart: 'Immediate' };
          break;
        case 'transportStop':
          message = { TransportStop: 'Immediate' };
          break;
        default:
          return;
      }
      
      await clientRef.current.sendMessage(message);
      addLog(`Sent: ${JSON.stringify(message)}`);
    } catch (error) {
      addLog(`Failed to send message: ${error}`);
    }
  };

  const clearLog = () => {
    setLog([]);
    setMessages([]);
  };

  return (
    <div className="p-4 max-w-4xl mx-auto">
      <h1 className="text-2xl font-bold mb-4">BuboCore Client Test</h1>
      
      {/* Connection Controls */}
      <div className="mb-4 p-4 border rounded">
        <h2 className="text-lg font-semibold mb-2">Connection</h2>
        <div className="flex gap-2 mb-2">
          <input
            type="text"
            value={ip}
            onChange={(e) => setIp(e.target.value)}
            placeholder="IP Address"
            className="px-2 py-1 border rounded"
            disabled={connected}
          />
          <input
            type="number"
            value={port}
            onChange={(e) => setPort(Number(e.target.value))}
            placeholder="Port"
            className="px-2 py-1 border rounded"
            disabled={connected}
          />
          {connected ? (
            <button
              onClick={disconnect}
              className="px-4 py-1 bg-red-500 text-white rounded hover:bg-red-600"
            >
              Disconnect
            </button>
          ) : (
            <button
              onClick={connect}
              className="px-4 py-1 bg-blue-500 text-white rounded hover:bg-blue-600"
            >
              Connect
            </button>
          )}
        </div>
        <div className={`text-sm ${connected ? 'text-green-600' : 'text-red-600'}`}>
          Status: {connected ? 'Connected' : 'Disconnected'}
        </div>
      </div>

      {/* Test Messages */}
      <div className="mb-4 p-4 border rounded">
        <h2 className="text-lg font-semibold mb-2">Test Messages</h2>
        <div className="grid grid-cols-3 gap-2">
          <button
            onClick={() => sendTestMessage('getName')}
            className="px-3 py-1 bg-gray-500 text-white rounded hover:bg-gray-600"
            disabled={!connected}
          >
            Set Name
          </button>
          <button
            onClick={() => sendTestMessage('getScene')}
            className="px-3 py-1 bg-gray-500 text-white rounded hover:bg-gray-600"
            disabled={!connected}
          >
            Get Scene
          </button>
          <button
            onClick={() => sendTestMessage('getClock')}
            className="px-3 py-1 bg-gray-500 text-white rounded hover:bg-gray-600"
            disabled={!connected}
          >
            Get Clock
          </button>
          <button
            onClick={() => sendTestMessage('getPeers')}
            className="px-3 py-1 bg-gray-500 text-white rounded hover:bg-gray-600"
            disabled={!connected}
          >
            Get Peers
          </button>
          <button
            onClick={() => sendTestMessage('getDevices')}
            className="px-3 py-1 bg-gray-500 text-white rounded hover:bg-gray-600"
            disabled={!connected}
          >
            Get Devices
          </button>
          <button
            onClick={() => sendTestMessage('transportStart')}
            className="px-3 py-1 bg-green-500 text-white rounded hover:bg-green-600"
            disabled={!connected}
          >
            Start Transport
          </button>
          <button
            onClick={() => sendTestMessage('transportStop')}
            className="px-3 py-1 bg-red-500 text-white rounded hover:bg-red-600"
            disabled={!connected}
          >
            Stop Transport
          </button>
        </div>
      </div>

      {/* Message Log */}
      <div className="mb-4 p-4 border rounded">
        <div className="flex justify-between items-center mb-2">
          <h2 className="text-lg font-semibold">Message Log</h2>
          <button
            onClick={clearLog}
            className="px-3 py-1 bg-gray-400 text-white rounded hover:bg-gray-500"
          >
            Clear
          </button>
        </div>
        <div className="bg-gray-100 p-3 rounded h-64 overflow-y-auto">
          {log.map((entry, index) => (
            <div key={index} className="text-sm font-mono mb-1">
              {entry}
            </div>
          ))}
        </div>
      </div>

      {/* Received Messages */}
      <div className="p-4 border rounded">
        <h2 className="text-lg font-semibold mb-2">Received Messages ({messages.length})</h2>
        <div className="bg-gray-100 p-3 rounded h-64 overflow-y-auto">
          {messages.map((message, index) => (
            <div key={index} className="text-sm font-mono mb-2 p-2 bg-white rounded">
              <pre>{JSON.stringify(message, null, 2)}</pre>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};