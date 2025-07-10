import React, { useState, useEffect, useCallback } from 'react';
import { Command } from 'cmdk';
import { useColorContext } from '../context/ColorContext';
import { BuboCoreClient } from '../client';
import { 
  Search, Settings, FileText, Palette, ToggleLeft, ToggleRight, RefreshCw,
  Play, Square, Pause, RotateCcw, Grid3X3, Code, SplitSquareHorizontal,
  Zap, Users, Wifi, WifiOff, Music, Layers, Clock, Volume2
} from 'lucide-react';

interface CommandPaletteProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  client?: BuboCoreClient;
  onViewChange?: (view: 'editor' | 'grid' | 'split') => void;
  currentView?: 'editor' | 'grid' | 'split';
  isConnected?: boolean;
  onConnect?: () => void;
  onDisconnect?: () => void;
}

interface CommandItem {
  id: string;
  label: string;
  description?: string;
  icon: React.ReactNode;
  keywords: string[];
  shortcut?: string;
  action: () => void;
  category: string;
  priority?: number;
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({ 
  open, 
  onOpenChange, 
  client,
  onViewChange,
  currentView = 'editor',
  isConnected = false,
  onConnect,
  onDisconnect
}) => {
  const [search, setSearch] = useState('');
  const { toggleTheme, themeMode, regenerateColors } = useColorContext();
  const [recentCommands, setRecentCommands] = useState<string[]>([]);

  const executeCommand = useCallback((command: CommandItem) => {
    command.action();
    onOpenChange(false);
    
    // Add to recent commands
    setRecentCommands(prev => {
      const filtered = prev.filter(id => id !== command.id);
      return [command.id, ...filtered].slice(0, 5);
    });
  }, [onOpenChange]);

  const transportCommands: CommandItem[] = [
    {
      id: 'transport-play',
      label: 'Play',
      description: 'Start playback',
      icon: <Play size={16} />,
      keywords: ['play', 'start', 'transport', 'begin'],
      shortcut: 'Space',
      action: () => client?.sendMessage({ TransportStart: "Immediate" }),
      category: 'Transport',
      priority: 1
    },
    {
      id: 'transport-stop',
      label: 'Stop',
      description: 'Stop playback',
      icon: <Square size={16} />,
      keywords: ['stop', 'halt', 'transport', 'end'],
      shortcut: 'Escape',
      action: () => client?.sendMessage({ TransportStop: "Immediate" }),
      category: 'Transport',
      priority: 1
    },
    {
      id: 'transport-pause',
      label: 'Pause',
      description: 'Pause playback',
      icon: <Pause size={16} />,
      keywords: ['pause', 'suspend', 'transport'],
      action: () => client?.sendMessage({ SchedulerControl: 'Pause' }),
      category: 'Transport',
      priority: 1
    },
    {
      id: 'transport-reset',
      label: 'Reset',
      description: 'Reset to beginning',
      icon: <RotateCcw size={16} />,
      keywords: ['reset', 'rewind', 'transport', 'beginning'],
      action: () => client?.sendMessage({ SchedulerControl: 'Reset' }),
      category: 'Transport',
      priority: 1
    },
  ];

  const viewCommands: CommandItem[] = [
    {
      id: 'view-editor',
      label: 'Editor View',
      description: 'Switch to code editor',
      icon: <Code size={16} />,
      keywords: ['editor', 'code', 'view', 'switch'],
      shortcut: '⌘1',
      action: () => onViewChange?.('editor'),
      category: 'View',
      priority: currentView === 'editor' ? 0 : 2
    },
    {
      id: 'view-grid',
      label: 'Grid View',
      description: 'Switch to grid view',
      icon: <Grid3X3 size={16} />,
      keywords: ['grid', 'table', 'view', 'switch'],
      shortcut: '⌘2',
      action: () => onViewChange?.('grid'),
      category: 'View',
      priority: currentView === 'grid' ? 0 : 2
    },
    {
      id: 'view-split',
      label: 'Split View',
      description: 'Show both editor and grid',
      icon: <SplitSquareHorizontal size={16} />,
      keywords: ['split', 'both', 'view', 'dual'],
      shortcut: '⌘3',
      action: () => onViewChange?.('split'),
      category: 'View',
      priority: currentView === 'split' ? 0 : 2
    }
  ];

  const themeCommands: CommandItem[] = [
    {
      id: 'toggle-theme',
      label: `Switch to ${themeMode === 'light' ? 'Dark' : 'Light'} Mode`,
      description: 'Toggle between light and dark themes',
      icon: themeMode === 'light' ? <ToggleLeft size={16} /> : <ToggleRight size={16} />,
      keywords: ['theme', 'dark', 'light', 'mode', 'appearance'],
      shortcut: '⌘T',
      action: toggleTheme,
      category: 'Appearance',
      priority: 2
    },
    {
      id: 'regenerate-colors',
      label: 'Regenerate Colors',
      description: 'Generate new color palette',
      icon: <RefreshCw size={16} />,
      keywords: ['colors', 'palette', 'regenerate', 'refresh', 'new'],
      action: regenerateColors,
      category: 'Appearance',
      priority: 2
    }
  ];

  const connectionCommands: CommandItem[] = [
    {
      id: 'connect',
      label: 'Connect to Server',
      description: 'Connect to BuboCore server',
      icon: <Wifi size={16} />,
      keywords: ['connect', 'server', 'join', 'network'],
      action: () => onConnect?.(),
      category: 'Connection',
      priority: isConnected ? 0 : 3
    },
    {
      id: 'disconnect',
      label: 'Disconnect',
      description: 'Disconnect from server',
      icon: <WifiOff size={16} />,
      keywords: ['disconnect', 'leave', 'exit', 'server'],
      action: () => onDisconnect?.(),
      category: 'Connection',
      priority: isConnected ? 3 : 0
    }
  ];

  const sceneCommands: CommandItem[] = [
    {
      id: 'get-scene',
      label: 'Get Scene',
      description: 'Retrieve current scene',
      icon: <Layers size={16} />,
      keywords: ['scene', 'get', 'retrieve', 'current'],
      action: () => client?.sendMessage('GetScene'),
      category: 'Scene',
      priority: 3
    },
    {
      id: 'get-clock',
      label: 'Get Clock',
      description: 'Get current clock state',
      icon: <Clock size={16} />,
      keywords: ['clock', 'time', 'tempo', 'beat'],
      action: () => client?.sendMessage('GetClock'),
      category: 'Scene',
      priority: 3
    }
  ];

  const systemCommands: CommandItem[] = [
    {
      id: 'open-settings',
      label: 'Open Settings',
      description: 'Open application settings',
      icon: <Settings size={16} />,
      keywords: ['settings', 'preferences', 'config', 'options'],
      shortcut: '⌘,',
      action: () => console.log('Open settings'),
      category: 'System',
      priority: 3
    }
  ];

  const allCommands = [
    ...transportCommands,
    ...viewCommands,
    ...themeCommands,
    ...connectionCommands,
    ...sceneCommands,
    ...systemCommands
  ].filter(cmd => {
    if (cmd.id === 'connect' && isConnected) return false;
    if (cmd.id === 'disconnect' && !isConnected) return false;
    if (cmd.category === 'Transport' && !isConnected) return false;
    if (cmd.category === 'Scene' && !isConnected) return false;
    return true;
  });

  // Group commands by category
  const commandsByCategory = allCommands.reduce((acc, cmd) => {
    if (!acc[cmd.category]) acc[cmd.category] = [];
    acc[cmd.category].push(cmd);
    return acc;
  }, {} as Record<string, CommandItem[]>);

  // Sort categories by priority
  const sortedCategories = Object.entries(commandsByCategory).sort(([, a], [, b]) => {
    const avgPriorityA = a.reduce((sum, cmd) => sum + (cmd.priority || 5), 0) / a.length;
    const avgPriorityB = b.reduce((sum, cmd) => sum + (cmd.priority || 5), 0) / b.length;
    return avgPriorityA - avgPriorityB;
  });

  const recentCommandItems = recentCommands
    .map(id => allCommands.find(cmd => cmd.id === id))
    .filter(Boolean) as CommandItem[];

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'k' && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        onOpenChange(!open);
      }
      if (e.key === 'Escape' && open && search === '') {
        onOpenChange(false);
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [open, onOpenChange, search]);

  useEffect(() => {
    if (open) {
      setSearch('');
    }
  }, [open]);

  if (!open) return null;

  return (
    <Command.Dialog
      open={open}
      onOpenChange={onOpenChange}
      className="fixed inset-0 z-50"
      style={{ 
        background: 'rgba(0, 0, 0, 0.4)',
        display: 'flex',
        alignItems: 'flex-start',
        justifyContent: 'center',
        paddingTop: '20vh'
      }}
    >
      <div 
        className="w-full max-w-2xl rounded-lg border shadow-2xl overflow-hidden"
        style={{
          backgroundColor: 'var(--color-surface)',
          borderColor: 'var(--color-border)',
          color: 'var(--color-text)'
        }}
      >
        <div className="flex items-center px-4 py-3 border-b" style={{ borderColor: 'var(--color-border)' }}>
          <Search size={18} className="mr-3" style={{ color: 'var(--color-muted)' }} />
          <Command.Input
            placeholder="Type a command or search..."
            className="flex-1 bg-transparent outline-none text-base placeholder:text-current"
            style={{ color: 'var(--color-text)' }}
            value={search}
            onValueChange={setSearch}
            autoFocus
          />
          <div className="text-xs px-2 py-1 rounded" style={{ 
            backgroundColor: 'var(--color-primary-100)', 
            color: 'var(--color-primary-700)' 
          }}>
            ⌘K
          </div>
        </div>
        
        <Command.List className="max-h-96 overflow-y-auto p-2" style={{ '--cmdk-list-height': '24rem' }}>
          <Command.Empty className="px-4 py-8 text-center text-sm" style={{ color: 'var(--color-muted)' }}>
            No commands found.
          </Command.Empty>
          
          {recentCommandItems.length > 0 && search === '' && (
            <Command.Group heading="Recent">
              {recentCommandItems.map((command) => (
                <Command.Item
                  key={`recent-${command.id}`}
                  value={`${command.label} ${command.keywords.join(' ')}`}
                  onSelect={() => executeCommand(command)}
                  className="flex items-center justify-between px-3 py-2 text-sm rounded-md cursor-pointer transition-colors aria-selected:bg-primary-100 aria-selected:text-primary-900"
                  style={{ color: 'var(--color-text)' }}
                >
                  <div className="flex items-center">
                    <span className="mr-3" style={{ color: 'var(--color-primary)' }}>
                      {command.icon}
                    </span>
                    <div>
                      <div className="font-medium">{command.label}</div>
                      {command.description && (
                        <div className="text-xs" style={{ color: 'var(--color-muted)' }}>
                          {command.description}
                        </div>
                      )}
                    </div>
                  </div>
                  {command.shortcut && (
                    <div className="text-xs px-2 py-1 rounded" style={{ 
                      backgroundColor: 'var(--color-border)', 
                      color: 'var(--color-muted)' 
                    }}>
                      {command.shortcut}
                    </div>
                  )}
                </Command.Item>
              ))}
            </Command.Group>
          )}
          
          {sortedCategories.map(([category, commands]) => (
            <Command.Group key={category} heading={category}>
              {commands.map((command) => (
                <Command.Item
                  key={command.id}
                  value={`${command.label} ${command.keywords.join(' ')}`}
                  keywords={command.keywords}
                  onSelect={() => executeCommand(command)}
                  className="flex items-center justify-between px-3 py-2 text-sm rounded-md cursor-pointer transition-colors aria-selected:bg-primary-100 aria-selected:text-primary-900"
                  style={{ color: 'var(--color-text)' }}
                >
                  <div className="flex items-center">
                    <span className="mr-3" style={{ color: 'var(--color-primary)' }}>
                      {command.icon}
                    </span>
                    <div>
                      <div className="font-medium">{command.label}</div>
                      {command.description && (
                        <div className="text-xs" style={{ color: 'var(--color-muted)' }}>
                          {command.description}
                        </div>
                      )}
                    </div>
                  </div>
                  {command.shortcut && (
                    <div className="text-xs px-2 py-1 rounded" style={{ 
                      backgroundColor: 'var(--color-border)', 
                      color: 'var(--color-muted)' 
                    }}>
                      {command.shortcut}
                    </div>
                  )}
                </Command.Item>
              ))}
            </Command.Group>
          ))}
        </Command.List>
      </div>
    </Command.Dialog>
  );
};