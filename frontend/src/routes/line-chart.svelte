<script lang="ts">
	import { type TMfSector } from '$lib/server/client';

	let { data }: { data: TMfSector[] } = $props();

	import {
		CategoryScale,
		Chart,
		LinearScale,
		LineController,
		LineElement,
		PointElement,
		Tooltip,
		Colors
	} from 'chart.js';
	import { onMount } from 'svelte';

	Chart.register([
		CategoryScale,
		LineController,
		LineElement,
		LinearScale,
		PointElement,
		Colors,
		Tooltip
	]);

	let canvas: HTMLCanvasElement;

	const tickerMap = new Map();
	const dates = new Set();

	for (const item of data) {
		const { realname, lead_value, super_value, date_time } = item;
		if (!dates.has(date_time)) dates.add(date_time);
		if (!tickerMap.has(realname)) {
			tickerMap.set(realname, []);
		}
		tickerMap.get(realname).push((lead_value + super_value) / 1_000_000);
	}

	let datasets = [];
	for (const [key, value] of tickerMap.entries()) {
		datasets.push({
			label: key,
			data: value
		});
	}

	onMount(() => {
		if (canvas) {
			const chart = new Chart(
				canvas, // TypeScript needs "as any" here
				{
					type: 'line',
					data: {
						labels: [...dates.values()],
						datasets
					},
					options: {
						responsive: true
					}
				}
			);
		}
	});
</script>

<div class="h-full rounded-sm border p-4">
	<canvas bind:this={canvas}></canvas>
</div>
