import { useEffect, useState, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface LinkClockState {
  phase: number;
  tempo: number;
  quantum: number;
}

export const useLinkClock = (isPlaying: boolean) => {
  const [clockState, setClockState] = useState<LinkClockState>({
    phase: 0,
    tempo: 120,
    quantum: 4,
  });
  
  const intervalRef = useRef<number | null>(null);

  useEffect(() => {
    // Initial fetch of tempo and quantum
    const fetchInitialState = async () => {
      try {
        const [tempo, quantum] = await Promise.all([
          invoke<number>('get_link_tempo'),
          invoke<number>('get_link_quantum'),
        ]);
        setClockState(prev => ({ ...prev, tempo, quantum }));
      } catch (error) {
        console.error('Failed to fetch initial Link state:', error);
      }
    };

    fetchInitialState();
  }, []);

  useEffect(() => {
    if (isPlaying) {
      // Update phase at 10 FPS for precise updates without smoothing
      intervalRef.current = window.setInterval(async () => {
        try {
          const phase = await invoke<number>('get_link_phase');
          setClockState(prev => ({ ...prev, phase }));
        } catch (error) {
          console.error('Failed to get Link phase:', error);
        }
      }, 33); // 30 FPS
    } else {
      // Clear interval when not playing
      if (intervalRef.current !== null) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
      // Reset phase to 0 when stopped
      setClockState(prev => ({ ...prev, phase: 0 }));
    }

    return () => {
      if (intervalRef.current !== null) {
        clearInterval(intervalRef.current);
      }
    };
  }, [isPlaying]);

  const setTempo = async (tempo: number) => {
    try {
      await invoke('set_link_tempo', { tempo });
      setClockState(prev => ({ ...prev, tempo }));
    } catch (error) {
      console.error('Failed to set Link tempo:', error);
    }
  };

  const setQuantum = async (quantum: number) => {
    try {
      await invoke('set_link_quantum', { quantum });
      setClockState(prev => ({ ...prev, quantum }));
    } catch (error) {
      console.error('Failed to set Link quantum:', error);
    }
  };

  return {
    phase: clockState.phase,
    tempo: clockState.tempo,
    quantum: clockState.quantum,
    setTempo,
    setQuantum,
  };
};