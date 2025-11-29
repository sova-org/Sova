import { writable, derived } from 'svelte/store';
import { isConnected } from './connectionState';

export type ViewType = 'CONFIG' | 'LOGIN' | 'DEVICES' | 'LOGS' | 'SCENE' | 'CHAT';

export interface LeafPane {
	type: 'leaf';
	id: string;
	viewType: ViewType | null;
}

export interface SplitPane {
	type: 'split';
	id: string;
	direction: 'horizontal' | 'vertical';
	sizes: [number, number];
	children: [PaneNode, PaneNode];
}

export type PaneNode = LeafPane | SplitPane;

export interface PaneLayout {
	root: PaneNode;
}

const STORAGE_KEY = 'sova-pane-layout';

function generateId(): string {
	return crypto.randomUUID();
}

function createDefaultLayout(): PaneLayout {
	return {
		root: {
			type: 'leaf',
			id: generateId(),
			viewType: null
		}
	};
}

function isValidNode(node: unknown): node is PaneNode {
	if (!node || typeof node !== 'object') return false;
	const n = node as PaneNode;
	if (n.type === 'leaf') {
		return typeof n.id === 'string';
	}
	if (n.type === 'split') {
		return (
			typeof n.id === 'string' &&
			(n.direction === 'horizontal' || n.direction === 'vertical') &&
			Array.isArray(n.sizes) &&
			n.sizes.length === 2 &&
			Array.isArray(n.children) &&
			n.children.length === 2 &&
			isValidNode(n.children[0]) &&
			isValidNode(n.children[1])
		);
	}
	return false;
}

function isValidLayout(obj: unknown): obj is PaneLayout {
	if (!obj || typeof obj !== 'object') return false;
	const layout = obj as PaneLayout;
	return layout.root !== undefined && isValidNode(layout.root);
}

function loadLayout(): PaneLayout {
	try {
		const stored = localStorage.getItem(STORAGE_KEY);
		if (stored) {
			const parsed = JSON.parse(stored);
			if (isValidLayout(parsed)) {
				return parsed;
			}
		}
	} catch {
		// Invalid stored layout
	}
	return createDefaultLayout();
}

function saveLayout(layout: PaneLayout): void {
	try {
		localStorage.setItem(STORAGE_KEY, JSON.stringify(layout));
	} catch {
		// Storage unavailable
	}
}

function findNode(node: PaneNode, id: string): PaneNode | null {
	if (node.id === id) return node;
	if (node.type === 'split') {
		return findNode(node.children[0], id) || findNode(node.children[1], id);
	}
	return null;
}

function findFirstLeaf(node: PaneNode): LeafPane | null {
	if (node.type === 'leaf') return node;
	return findFirstLeaf(node.children[0]) || findFirstLeaf(node.children[1]);
}

function splitNodeInLayout(
	layout: PaneLayout,
	paneId: string,
	direction: 'horizontal' | 'vertical'
): PaneLayout {
	const newLayout = structuredClone(layout);

	function replaceNode(
		node: PaneNode,
		parent: SplitPane | null,
		index: 0 | 1 | null
	): boolean {
		if (node.id === paneId && node.type === 'leaf') {
			const newSplit: SplitPane = {
				type: 'split',
				id: generateId(),
				direction,
				sizes: [50, 50],
				children: [
					{ ...node, id: generateId() },
					{ type: 'leaf', id: generateId(), viewType: null }
				]
			};

			if (parent === null) {
				newLayout.root = newSplit;
			} else {
				parent.children[index!] = newSplit;
			}
			return true;
		}

		if (node.type === 'split') {
			if (replaceNode(node.children[0], node, 0)) return true;
			if (replaceNode(node.children[1], node, 1)) return true;
		}
		return false;
	}

	replaceNode(newLayout.root, null, null);
	return newLayout;
}

function closeNodeInLayout(layout: PaneLayout, paneId: string): PaneLayout {
	if (layout.root.type === 'leaf' && layout.root.id === paneId) {
		return layout;
	}

	const newLayout = structuredClone(layout);

	function removeAndPromote(
		parent: SplitPane,
		grandparent: SplitPane | null,
		parentIndex: 0 | 1 | null
	): boolean {
		const childIndex = parent.children.findIndex((c) => c.id === paneId);
		if (childIndex !== -1) {
			const siblingIndex = childIndex === 0 ? 1 : 0;
			const sibling = parent.children[siblingIndex];

			if (grandparent === null) {
				newLayout.root = sibling;
			} else {
				grandparent.children[parentIndex!] = sibling;
			}
			return true;
		}

		for (let i = 0; i < 2; i++) {
			const child = parent.children[i];
			if (child.type === 'split') {
				if (removeAndPromote(child, parent, i as 0 | 1)) return true;
			}
		}
		return false;
	}

	if (newLayout.root.type === 'split') {
		removeAndPromote(newLayout.root, null, null);
	}

	return newLayout;
}

function resetDisconnectedViews(node: PaneNode): void {
	if (node.type === 'leaf') {
		if (node.viewType === 'SCENE' || node.viewType === 'DEVICES' || node.viewType === 'CHAT') {
			node.viewType = 'LOGIN';
		}
	} else {
		resetDisconnectedViews(node.children[0]);
		resetDisconnectedViews(node.children[1]);
	}
}

function collectOpenViews(node: PaneNode): Set<ViewType> {
	const views = new Set<ViewType>();
	if (node.type === 'leaf') {
		if (node.viewType !== null) {
			views.add(node.viewType);
		}
	} else {
		for (const view of collectOpenViews(node.children[0])) {
			views.add(view);
		}
		for (const view of collectOpenViews(node.children[1])) {
			views.add(view);
		}
	}
	return views;
}

function findParentSplit(root: PaneNode, targetId: string): SplitPane | null {
	if (root.type === 'leaf') return null;

	if (root.children[0].id === targetId || root.children[1].id === targetId) {
		return root;
	}

	for (const child of root.children) {
		if (child.type === 'split') {
			const found = findParentSplit(child, targetId);
			if (found) return found;
		}
	}
	return null;
}

function createPaneStore() {
	const { subscribe, set, update } = writable<PaneLayout>(loadLayout());

	subscribe((layout) => {
		saveLayout(layout);
	});

	return {
		subscribe,

		setView(paneId: string, viewType: ViewType | null): void {
			update((layout) => {
				const newLayout = structuredClone(layout);
				const node = findNode(newLayout.root, paneId);
				if (node && node.type === 'leaf') {
					node.viewType = viewType;
				}
				return newLayout;
			});
		},

		splitPane(paneId: string, direction: 'horizontal' | 'vertical'): void {
			update((layout) => splitNodeInLayout(layout, paneId, direction));
		},

		closePane(paneId: string): void {
			update((layout) => closeNodeInLayout(layout, paneId));
		},

		updateSizes(splitId: string, sizes: [number, number]): void {
			update((layout) => {
				const newLayout = structuredClone(layout);
				const node = findNode(newLayout.root, splitId);
				if (node && node.type === 'split') {
					node.sizes = sizes;
				}
				return newLayout;
			});
		},

		addPane(): void {
			update((layout) => {
				const firstLeaf = findFirstLeaf(layout.root);
				if (firstLeaf) {
					return splitNodeInLayout(layout, firstLeaf.id, 'vertical');
				}
				return layout;
			});
		},

		reset(): void {
			set(createDefaultLayout());
		},

		handleDisconnect(): void {
			update((layout) => {
				const newLayout = structuredClone(layout);
				resetDisconnectedViews(newLayout.root);
				return newLayout;
			});
		},

		toggleParentDirection(paneId: string): void {
			update((layout) => {
				const newLayout = structuredClone(layout);
				const parent = findParentSplit(newLayout.root, paneId);
				if (parent) {
					parent.direction = parent.direction === 'horizontal' ? 'vertical' : 'horizontal';
				}
				return newLayout;
			});
		}
	};
}

export const paneLayout = createPaneStore();

export const availableViews = derived(
	[isConnected, paneLayout],
	([$isConnected, $paneLayout]): ViewType[] => {
		const allViews: ViewType[] = $isConnected
			? ['SCENE', 'DEVICES', 'CHAT', 'LOGS', 'CONFIG']
			: ['LOGIN', 'LOGS', 'CONFIG'];

		const openViews = collectOpenViews($paneLayout.root);
		return allViews.filter((view) => !openViews.has(view));
	}
);

isConnected.subscribe(($connected) => {
	if (!$connected) {
		paneLayout.handleDisconnect();
	}
});
