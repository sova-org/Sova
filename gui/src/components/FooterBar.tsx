import React, { useState, useRef, useEffect } from 'react';
import { Users, Wifi, WifiOff, User } from 'lucide-react';

interface FooterBarProps {
  isConnected: boolean;
  peerCount: number;
  serverAddress?: string;
  username: string;
  onUsernameChange: (username: string) => void;
}

export const FooterBar: React.FC<FooterBarProps> = ({ 
  isConnected, 
  peerCount,
  serverAddress,
  username,
  onUsernameChange
}) => {
  const [isEditing, setIsEditing] = useState(false);
  const [editValue, setEditValue] = useState(username);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  const handleSave = () => {
    if (editValue.trim()) {
      onUsernameChange(editValue.trim());
    } else {
      setEditValue(username);
    }
    setIsEditing(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSave();
    } else if (e.key === 'Escape') {
      setEditValue(username);
      setIsEditing(false);
    }
  };
  return (
    <div 
      className="h-6 border-t flex items-center justify-between px-4 text-xs"
      style={{ 
        backgroundColor: 'var(--color-surface)', 
        borderColor: 'var(--color-border)',
        color: 'var(--color-muted)'
      }}
    >
      <div className="flex items-center space-x-4">
        {/* Connection Status */}
        <div className="flex items-center space-x-1.5">
          {isConnected ? (
            <Wifi size={12} style={{ color: 'var(--color-success)' }} />
          ) : (
            <WifiOff size={12} style={{ color: 'var(--color-error)' }} />
          )}
          <span>{isConnected ? 'Connected' : 'Disconnected'}</span>
          {isConnected && serverAddress && (
            <span className="opacity-60">• {serverAddress}</span>
          )}
        </div>

        {/* Peer Count */}
        {isConnected && (
          <div className="flex items-center space-x-1.5">
            <Users size={12} />
            <span>{peerCount} {peerCount === 1 ? 'peer' : 'peers'}</span>
          </div>
        )}
      </div>

      <div className="flex items-center space-x-4">
        {/* Username */}
        <div className="flex items-center space-x-1.5">
          <User size={12} />
          {isEditing ? (
            <input
              ref={inputRef}
              type="text"
              value={editValue}
              onChange={(e) => setEditValue(e.target.value)}
              onBlur={handleSave}
              onKeyDown={handleKeyDown}
              className="bg-transparent border-b border-current outline-none px-0.5 text-xs"
              style={{ color: 'var(--color-muted)', width: `${editValue.length + 1}ch` }}
            />
          ) : (
            <span 
              onClick={() => setIsEditing(true)}
              className="cursor-pointer hover:opacity-80"
              title="Click to rename"
            >
              {username}
            </span>
          )}
        </div>
      </div>
    </div>
  );
};