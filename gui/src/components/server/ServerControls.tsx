import React, { useState } from 'react';
import { useStore } from '@nanostores/react';
import { serverManagerStore, serverManagerActions } from '../../stores/server/serverManager';
import { Play, Square, RotateCcw, Loader2 } from 'lucide-react';

interface ServerControlsProps {
  layout?: 'horizontal' | 'grid';
  size?: 'small' | 'medium';
}

export const ServerControls: React.FC<ServerControlsProps> = ({ 
  layout = 'horizontal', 
  size = 'medium' 
}) => {
  const serverState = useStore(serverManagerStore);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleStartServer = async () => {
    try {
      setIsLoading(true);
      setError(null);
      await serverManagerActions.startServer();
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsLoading(false);
    }
  };

  const handleStopServer = async () => {
    try {
      setIsLoading(true);
      setError(null);
      await serverManagerActions.stopServer();
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsLoading(false);
    }
  };

  const handleRestartServer = async () => {
    try {
      setIsLoading(true);
      setError(null);
      await serverManagerActions.restartServer();
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsLoading(false);
    }
  };


  const isRunning = serverState.status === 'Running';
  const isStarting = serverState.status === 'Starting';
  const isStopping = serverState.status === 'Stopping';
  const isServerLoading = isLoading || isStarting || isStopping;

  const buttonSize = size === 'small' ? 14 : 16;
  const buttonClass = size === 'small' ? 'px-3 py-2 text-sm' : 'px-3 py-2';
  const containerClass = layout === 'grid' ? 'grid grid-cols-3 gap-2' : 'flex gap-2';

  return (
    <div>
      {/* Error display */}
      {error && (
        <div className="p-3 bg-red-100 border border-red-200 text-red-700 text-sm mb-4">
          {error}
        </div>
      )}

      {/* Control buttons */}
      <div className={containerClass}>
        <button
          onClick={handleStartServer}
          disabled={isServerLoading || isRunning}
          className={`flex items-center ${layout === 'grid' ? 'justify-center' : ''} gap-2 ${buttonClass} text-white disabled:opacity-50`}
          style={{ backgroundColor: 'var(--color-success)' }}
        >
          {isStarting ? <Loader2 size={buttonSize} className="animate-spin" /> : <Play size={buttonSize} />}
          Start
        </button>
        
        <button
          onClick={handleStopServer}
          disabled={isServerLoading || !isRunning}
          className={`flex items-center ${layout === 'grid' ? 'justify-center' : ''} gap-2 ${buttonClass} text-white disabled:opacity-50`}
          style={{ backgroundColor: 'var(--color-error)' }}
        >
          {isStopping ? <Loader2 size={buttonSize} className="animate-spin" /> : <Square size={buttonSize} />}
          Stop
        </button>
        
        <button
          onClick={handleRestartServer}
          disabled={isServerLoading}
          className={`flex items-center ${layout === 'grid' ? 'justify-center' : ''} gap-2 ${buttonClass} text-white disabled:opacity-50`}
          style={{ backgroundColor: 'var(--color-warning)' }}
        >
          {isServerLoading ? <Loader2 size={buttonSize} className="animate-spin" /> : <RotateCcw size={buttonSize} />}
          Restart
        </button>
        
      </div>
    </div>
  );
};