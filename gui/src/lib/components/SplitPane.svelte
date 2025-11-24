<script lang="ts">
	import type { Snippet } from 'svelte';
	import { onMount } from 'svelte';

	interface Props {
		orientation?: 'horizontal' | 'vertical';
		initialSize?: number;
		minSize?: number;
		first: Snippet;
		second: Snippet;
	}

	let {
		orientation = 'horizontal',
		initialSize = 50,
		minSize = 20,
		first,
		second
	}: Props = $props();

	let splitSize = $state(initialSize);
	let isDragging = $state(false);
	let container: HTMLDivElement;

	function handleMouseDown() {
		isDragging = true;
	}

	function handleMouseMove(e: MouseEvent) {
		if (!isDragging || !container) return;

		const rect = container.getBoundingClientRect();

		if (orientation === 'vertical') {
			const newSize = ((e.clientX - rect.left) / rect.width) * 100;
			splitSize = Math.max(minSize, Math.min(100 - minSize, newSize));
		} else {
			const newSize = ((e.clientY - rect.top) / rect.height) * 100;
			splitSize = Math.max(minSize, Math.min(100 - minSize, newSize));
		}
	}

	function handleMouseUp() {
		isDragging = false;
	}

	onMount(() => {
		document.addEventListener('mousemove', handleMouseMove);
		document.addEventListener('mouseup', handleMouseUp);

		return () => {
			document.removeEventListener('mousemove', handleMouseMove);
			document.removeEventListener('mouseup', handleMouseUp);
		};
	});
</script>

<div
	class="split-pane"
	class:vertical={orientation === 'vertical'}
	class:horizontal={orientation === 'horizontal'}
	bind:this={container}>
	<div
		class="pane first"
		style="{orientation === 'vertical' ? 'width' : 'height'}: {splitSize}%">
		{@render first()}
	</div>

	<div class="divider" role="separator" onmousedown={handleMouseDown}></div>

	<div
		class="pane second"
		style="{orientation === 'vertical' ? 'width' : 'height'}: {100 - splitSize}%">
		{@render second()}
	</div>
</div>

<style>
	.split-pane {
		width: 100%;
		height: 100%;
		display: flex;
		position: relative;
		overflow: hidden;
	}

	.split-pane.horizontal {
		flex-direction: column;
	}

	.split-pane.vertical {
		flex-direction: row;
	}

	.pane {
		overflow: hidden;
		position: relative;
	}

	.divider {
		background-color: var(--colors-border);
		flex-shrink: 0;
		cursor: col-resize;
		z-index: 10;
	}

	.horizontal .divider {
		height: 4px;
		width: 100%;
		cursor: row-resize;
	}

	.vertical .divider {
		width: 4px;
		height: 100%;
		cursor: col-resize;
	}

	.divider:hover {
		background-color: var(--colors-accent);
	}
</style>
