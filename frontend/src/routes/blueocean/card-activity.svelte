<script lang="ts">
	import Minus from '@lucide/svelte/icons/minus';
	import Plus from '@lucide/svelte/icons/plus';

	import Button from '$lib/components/ui/button/button.svelte';
	import {
		Card,
		CardContent,
		CardDescription,
		CardFooter,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { type ChartConfig, ChartContainer } from '$lib/components/ui/chart';
	import { BarChart } from 'layerchart';

	let data = [
		{
			goal: 400
		},
		{
			goal: 300
		},
		{
			goal: 200
		},
		{
			goal: 300
		},
		{
			goal: 200
		},
		{
			goal: 278
		},
		{
			goal: 189
		},
		{
			goal: 239
		},
		{
			goal: 300
		},
		{
			goal: 200
		},
		{
			goal: 278
		},
		{
			goal: 189
		},
		{
			goal: 349
		}
	].map((x, i) => ({ y: x.goal, x: 1 + i }));

	const chartConfig = {
		goal: {
			label: 'Goal',
			color: 'var(--primary)'
		}
	} satisfies ChartConfig;

	let goal = $state(350);

	function onClick(adjustment: number) {
		goal = Math.max(200, Math.min(400, goal + adjustment));
	}
</script>

<Card class="h-full w-full gap-5">
	<CardHeader>
		<CardTitle>Move Goal</CardTitle>
		<CardDescription>Set your dail activity goal.</CardDescription>
	</CardHeader>
	<CardContent class="flex flex-1 flex-col">
		<div class="flex items-center justify-center gap-4">
			<Button
				variant="outline"
				size="icon"
				class="size-7 rounded-full bg-transparent"
				onclick={() => onClick(-10)}
				disabled={goal <= 200}
			>
				<Minus />
				<span class="sr-only">Decrease</span>
			</Button>
			<div class="text-center">
				<div class="text-4xl font-bold tracking-tighter tabular-nums">{goal}</div>
				<div class="text-xs text-muted-foreground uppercase">Calories/day</div>
			</div>
			<Button
				variant="outline"
				size="icon"
				class="size-7 rounded-full bg-transparent"
				onclick={() => onClick(10)}
				disabled={goal >= 400}
			>
				<Plus />
				<span class="sr-only">Increase</span>
			</Button>
		</div>
		<div class="flex-1">
			<ChartContainer config={chartConfig} class="h-full w-full">
				<BarChart {data} y="y" x="x"></BarChart>
			</ChartContainer>
		</div>
	</CardContent>
	<CardFooter>
		<Button class="w-full" variant="secondary">Set Goal</Button>
	</CardFooter>
</Card>
