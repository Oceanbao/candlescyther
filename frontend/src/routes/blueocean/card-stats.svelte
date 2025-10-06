<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { type ChartConfig, ChartContainer } from '$lib/components/ui/chart';
	import { curveCatmullRom } from 'd3-shape';
	import { AreaChart, LineChart } from 'layerchart';

	const data = [
		{
			revenue: 10400,
			subscription: 40
		},
		{
			revenue: 14405,
			subscription: 90
		},
		{
			revenue: 9400,
			subscription: 200
		},
		{
			revenue: 8200,
			subscription: 278
		},
		{
			revenue: 7000,
			subscription: 89
		},
		{
			revenue: 9600,
			subscription: 239
		},
		{
			revenue: 11244,
			subscription: 78
		},
		{
			revenue: 26475,
			subscription: 89
		}
	].map((item, i) => ({ revenue: item.revenue, subscription: item.subscription, idx: i + 1 }));

	const chartConfig = {
		revenue: {
			label: 'Revenue',
			color: 'var(--primary)'
		},
		subscription: {
			label: 'Subscriptions',
			color: 'var(--primary)'
		}
	} satisfies ChartConfig;
</script>

<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-1 xl:grid-cols-2">
	<Card class="h-full w-full">
		<CardHeader>
			<CardDescription>Total Revenue</CardDescription>
			<CardTitle class="text-3xl">$15,231.89</CardTitle>
			<CardDescription>+20.1% from last month</CardDescription>
		</CardHeader>
		<CardContent class="pb-0">
			<ChartContainer config={chartConfig} class="h-[80px] w-full">
				<LineChart
					{data}
					y="revenue"
					x="idx"
					axis="y"
					points
					props={{
						yAxis: {
							tickLabelProps: {
								textAnchor: 'start',
								verticalAnchor: 'end'
							},
							tickLength: 0,
							format: (d) => ''
						},
						grid: {
							x: false,
							y: false
						}
					}}
					padding={{
						top: 5,
						right: 10,
						left: 10,
						bottom: 0
					}}
				/>
			</ChartContainer>
		</CardContent>
	</Card>
	<Card class="pb-0 lg:hidden xl:flex">
		<CardHeader class="flex flex-row items-center">
			<div class="grid gap-2">
				<CardDescription>Subscriptions</CardDescription>
				<CardTitle class="text-3xl">+2,350</CardTitle>
				<CardDescription>+180.1% from last month</CardDescription>
			</div>
			<Button variant="ghost" size="sm" class="ml-auto shadow-none">View More</Button>
		</CardHeader>
		<CardContent class="mt-auto max-h-[124px] flex-1 p-0">
			<ChartContainer config={chartConfig} class="size-full">
				<AreaChart
					{data}
					y="subscription"
					x="idx"
					axis="y"
					props={{
						yAxis: {
							tickLabelProps: {
								textAnchor: 'start',
								verticalAnchor: 'end'
							},
							tickLength: 0,
							format: (d) => ''
						},
						area: {
							curve: curveCatmullRom
						},
						grid: {
							x: false,
							y: false
						}
					}}
					padding={{
						left: 0,
						right: 0
					}}
				/>
			</ChartContainer>
		</CardContent>
	</Card>
</div>
