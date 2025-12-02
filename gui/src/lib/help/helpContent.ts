export interface HelpEntry {
  title: string;
  description: string;
}

export const helpContent: Record<string, HelpEntry> = {
  // TopBar - Left section
  "app-name": {
    title: "Sova",
    description: "Click to open the about dialog with version info and links.",
  },
  "play-button": {
    title: "Play / Pause",
    description:
      "Start or stop the transport. When playing, scheduled events will be triggered.",
  },
  "beat-display": {
    title: "Beat Counter",
    description: "Shows the current beat position in the transport.",
  },
  "tempo-display": {
    title: "Tempo",
    description: "Current tempo in BPM. Click to edit the value.",
  },
  "nickname-display": {
    title: "Your Nickname",
    description:
      "Your display name shown to other collaborators. Click to edit.",
  },
  "peer-count": {
    title: "Collaborators",
    description: "Number of other users currently connected to this session.",
  },

  // TopBar - Right section
  "help-button": {
    title: "Help Mode",
    description:
      "You are here! Hover over elements to learn what they do. Press Escape or click again to exit.",
  },
  "add-pane": {
    title: "Add Pane",
    description:
      "Add a new pane to the layout. Panes can display different views of your project.",
  },
  "disconnect-button": {
    title: "Disconnect",
    description:
      "Disconnect from the current session and return to the login screen.",
  },

  // Pane zones
  "zone-scene": {
    title: "Scene View",
    description:
      "Visual timeline showing lines and frames. Click frames to edit their code.",
  },
  "zone-logs": {
    title: "Log View",
    description: "Displays system messages, errors, and output from your code.",
  },
  "logs-filters": {
    title: "Log Level Filters",
    description:
      "Toggle which log levels to display: Fatal, Error, Warn, Info, Debug.",
  },
  "logs-auto-scroll": {
    title: "Auto-scroll",
    description: "When enabled, automatically scrolls to show new log entries.",
  },
  "logs-clear": {
    title: "Clear Logs",
    description: "Remove all log entries from the view.",
  },
  "zone-config": {
    title: "Configuration",
    description: "Edit the project configuration file (TOML format).",
  },
  "config-save": {
    title: "Save Configuration",
    description: "Save changes to the config file. Shortcut: Cmd/Ctrl+S.",
  },
  "zone-chat": {
    title: "Chat",
    description: "Send messages to other collaborators in the session.",
  },
  "chat-input": {
    title: "Message Input",
    description: "Type your message here. Press Enter to send.",
  },
  "chat-send": {
    title: "Send Message",
    description: "Send your message to other collaborators.",
  },
  "zone-devices": {
    title: "Devices",
    description: "Manage MIDI and OSC output devices for your project.",
  },
  "devices-tab-midi": {
    title: "MIDI Devices",
    description:
      "View and manage MIDI output devices. Connect to hardware or create virtual outputs.",
  },
  "devices-tab-osc": {
    title: "OSC Devices",
    description:
      "View and manage OSC (Open Sound Control) output devices for network communication.",
  },
  "devices-add-midi": {
    title: "Add Virtual MIDI",
    description:
      "Create a virtual MIDI output that other applications can receive from.",
  },
  "devices-add-osc": {
    title: "Add OSC Output",
    description:
      "Create an OSC output to send messages to a specific IP address and port.",
  },
  "devices-slot": {
    title: "Device Slot",
    description:
      "Assign a slot number (1-16) to reference this device in your code. Click to edit.",
  },
  "devices-status": {
    title: "Connection Status",
    description:
      "Shows whether the device is connected and ready to receive messages.",
  },
  "devices-connect": {
    title: "Connect / Disconnect",
    description: "Toggle the connection to this MIDI device.",
  },
  "devices-remove": {
    title: "Remove Device",
    description: "Delete this OSC output from the server devices.",
  },
  "zone-snapshots": {
    title: "Snapshots",
    description: "Save and load project snapshots. Manage your saved states.",
  },
  "snapshots-search": {
    title: "Search Projects",
    description: "Filter the project list by name.",
  },
  "snapshots-import": {
    title: "Import Project",
    description: "Import a project file from disk into the current session.",
  },
  "snapshots-refresh": {
    title: "Refresh List",
    description: "Reload the project list from the server.",
  },
  "snapshots-folder": {
    title: "Open Folder",
    description: "Open the projects folder in your file manager.",
  },
  "snapshots-name": {
    title: "Project Name",
    description: "Enter a name for the new snapshot. Press Enter to save.",
  },
  "snapshots-save": {
    title: "Save Snapshot",
    description: "Save the current session state as a new project snapshot.",
  },
  "snapshots-load-now": {
    title: "Load Now",
    description:
      "Load this project immediately, replacing the current session.",
  },
  "snapshots-load-end": {
    title: "Load at End of Line",
    description:
      "Queue this project to load when the current line finishes playing.",
  },
  "snapshots-delete": {
    title: "Delete Project",
    description: "Permanently delete this project snapshot.",
  },
  "zone-login": {
    title: "Login",
    description: "Connect to a Sova server to start or join a session.",
  },

  // Login form fields
  "login-ip": {
    title: "Server IP",
    description: "IP address of the Sova server to connect to.",
  },
  "login-port": {
    title: "Server Port",
    description: "Port number the server is listening on (default: 8080).",
  },
  "login-nickname": {
    title: "Nickname",
    description: "Your display name shown to other collaborators.",
  },
  "login-connect": {
    title: "Connect",
    description: "Connect to the server with the provided credentials.",
  },

  // SceneView toolbar
  "scene-zoom-out": {
    title: "Zoom Out",
    description: "Decrease the timeline zoom level.",
  },
  "scene-zoom-in": {
    title: "Zoom In",
    description: "Increase the timeline zoom level.",
  },
  "scene-zoom-reset": {
    title: "Reset Zoom",
    description: "Reset zoom to 100%.",
  },
  "scene-timeline-orientation": {
    title: "Timeline Orientation",
    description: "Toggle between horizontal and vertical timeline layout.",
  },
  "scene-split-orientation": {
    title: "Split Orientation",
    description: "Toggle how the editor panel splits from the timeline.",
  },

  // Pane header controls
  "pane-view-selector": {
    title: "View Selector",
    description: "Choose what to display in this pane.",
  },
  "pane-change-view": {
    title: "Change View",
    description: "Switch what this pane displays.",
  },
  "pane-split-vertical": {
    title: "Split Vertical",
    description: "Split this pane into two side-by-side panes.",
  },
  "pane-split-horizontal": {
    title: "Split Horizontal",
    description: "Split this pane into two stacked panes.",
  },
  "pane-toggle-direction": {
    title: "Toggle Direction",
    description:
      "Change the split orientation between horizontal and vertical.",
  },
  "pane-close": {
    title: "Close Pane",
    description: "Close this pane. The layout will adjust automatically.",
  },

  // Frame editor controls
  "frame-enabled": {
    title: "Enabled",
    description: "Toggle whether this frame is active and will be played.",
  },
  "frame-fetch": {
    title: "Fetch",
    description: "Discard local changes and reload content from the server.",
  },
  "frame-evaluate": {
    title: "Evaluate",
    description:
      "Send the code to the server for compilation. Shortcut: Cmd/Ctrl+Enter.",
  },
  "frame-close": {
    title: "Close Editor",
    description: "Close the frame editor panel.",
  },
};
