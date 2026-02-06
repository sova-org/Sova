import { writable, derived, get } from "svelte/store";
import { fuzzyScore } from "$lib/utils/fuzzySearch";

export interface Command {
  id: string;
  name: string;
  description: string;
  execute: (_args: string[]) => void | Promise<void>;
  keywords?: string[];
  isAvailable?: () => boolean;
}

interface CommandPaletteState {
  isOpen: boolean;
  query: string;
  selectedIndex: number;
}

const commands: Command[] = [];

export function registerCommand(cmd: Command): void {
  commands.push(cmd);
}

export function getCommands(): Command[] {
  return commands;
}

function parseQuery(query: string): { search: string; args: string[] } {
  const tokens = query
    .trim()
    .split(/\s+/)
    .filter((t) => t.length > 0);
  if (tokens.length === 0) return { search: "", args: [] };
  return { search: tokens[0], args: tokens.slice(1) };
}

const { subscribe, set, update } = writable<CommandPaletteState>({
  isOpen: false,
  query: "",
  selectedIndex: 0,
});

export const commandPalette = {
  subscribe,

  open(): void {
    set({ isOpen: true, query: "", selectedIndex: 0 });
  },

  close(): void {
    update((s) => ({ ...s, isOpen: false }));
  },

  setQuery(query: string): void {
    update((s) => ({ ...s, query, selectedIndex: 0 }));
  },

  selectNext(maxIndex: number): void {
    update((s) => ({
      ...s,
      selectedIndex: Math.min(s.selectedIndex + 1, maxIndex),
    }));
  },

  selectPrev(): void {
    update((s) => ({
      ...s,
      selectedIndex: Math.max(s.selectedIndex - 1, 0),
    }));
  },

  setSelectedIndex(index: number): void {
    update((s) => ({ ...s, selectedIndex: index }));
  },

  executeSelected(filtered: Command[]): void {
    const state = get({ subscribe });
    const cmd = filtered[state.selectedIndex];
    if (!cmd) return;

    const { args } = parseQuery(state.query);
    commandPalette.close();
    cmd.execute(args);
  },
};

export const filteredCommands = derived(commandPalette, ($state) => {
  const { search } = parseQuery($state.query);

  const available = commands.filter((cmd) => !cmd.isAvailable || cmd.isAvailable());

  if (!search) return available;

  const scored = available
    .map((cmd) => ({
      cmd,
      score: fuzzyScore(search, cmd.name, cmd.keywords),
    }))
    .filter((item) => item.score > 0)
    .sort((a, b) => b.score - a.score);

  return scored.map((item) => item.cmd);
});
