<script lang="ts">
	import { scopeSamples } from '$lib/stores/scope';
	import { onMount } from 'svelte';

	let canvas: HTMLCanvasElement;
	let ctx: CanvasRenderingContext2D | null = null;
	let strokeColor = '#aaa';

	function resize() {
		if (!canvas) return;
		canvas.width = canvas.clientWidth * devicePixelRatio;
		canvas.height = canvas.clientHeight * devicePixelRatio;
	}

	function draw(samples: number[]) {
		if (!ctx || !samples.length) return;
		const w = ctx.canvas.width;
		const h = ctx.canvas.height;
		if (w === 0 || h === 0) return;

		const mid = h / 2;
		const gain = 4;

		ctx.clearRect(0, 0, w, h);

		ctx.strokeStyle = strokeColor;
		ctx.lineWidth = devicePixelRatio * 1.5;
		ctx.beginPath();

		for (let i = 0; i < samples.length; i++) {
			const x = (i / samples.length) * w;
			const y = mid - Math.max(-1, Math.min(1, samples[i] * gain)) * mid;
			if (i === 0) {
				ctx.moveTo(x, y);
			} else {
				ctx.lineTo(x, y);
			}
		}
		ctx.stroke();
	}

	$effect(() => {
		if ($scopeSamples) draw($scopeSamples);
	});

	onMount(() => {
		ctx = canvas.getContext('2d');
		strokeColor =
			getComputedStyle(canvas).getPropertyValue('--scope-color').trim() ||
			'#888';
		resize();
		const observer = new ResizeObserver(resize);
		observer.observe(canvas);
		return () => observer.disconnect();
	});
</script>

<canvas bind:this={canvas}></canvas>

<style>
	canvas {
		width: 100%;
		height: 100%;
		display: block;
		--scope-color: #888;
	}
</style>
