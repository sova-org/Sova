// Single source of truth for all Tauri event names
// Used by both Rust (emit) and TypeScript (listen)

export const SERVER_EVENTS = {
	// Connection
	HELLO: 'server:hello',
	CONNECTION_REFUSED: 'server:connection-refused',

	// Status
	SUCCESS: 'server:success',
	ERROR: 'server:error',
	LOG: 'server:log',

	// Scene
	SCENE: 'server:scene',
	SNAPSHOT: 'server:snapshot',

	// Lines
	LINE_VALUES: 'server:line-values',
	LINE_CONFIGURATIONS: 'server:line-configurations',
	ADD_LINE: 'server:add-line',
	REMOVE_LINE: 'server:remove-line',

	// Frames
	FRAME_VALUES: 'server:frame-values',
	ADD_FRAME: 'server:add-frame',
	REMOVE_FRAME: 'server:remove-frame',
	FRAME_POSITION: 'server:frame-position',

	// Transport
	TRANSPORT_STARTED: 'server:transport-started',
	TRANSPORT_STOPPED: 'server:transport-stopped',
	CLOCK_STATE: 'server:clock-state',

	// Devices
	DEVICE_LIST: 'server:device-list',

	// Collaboration
	PEERS_UPDATED: 'server:peers-updated',
	CHAT: 'server:chat',
	PEER_STARTED_EDITING: 'server:peer-started-editing',
	PEER_STOPPED_EDITING: 'server:peer-stopped-editing',

	// Compilation & Variables
	GLOBAL_VARIABLES: 'server:global-variables',
	COMPILATION_UPDATE: 'server:compilation-update'
} as const;

export const CLIENT_EVENTS = {
	DISCONNECTED: 'client-disconnected',
	CONFIG_UPDATE: 'config-update'
} as const;

// Type-safe event name
export type ServerEvent = typeof SERVER_EVENTS[keyof typeof SERVER_EVENTS];
export type ClientEvent = typeof CLIENT_EVENTS[keyof typeof CLIENT_EVENTS];
export type EventName = ServerEvent | ClientEvent;
