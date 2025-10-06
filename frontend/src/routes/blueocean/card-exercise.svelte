<script lang="ts">
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { type ChartConfig, ChartContainer, ChartTooltip } from '$lib/components/ui/chart';
	import { LineChart } from 'layerchart';

	const data = [
		{
			average: 400,
			today: 240,
			day: 'Monday'
		},
		{
			average: 300,
			today: 139,
			day: 'Tuesday'
		},
		{
			average: 200,
			today: 980,
			day: 'Wednesday'
		},
		{
			average: 278,
			today: 390,
			day: 'Thursday'
		},
		{
			average: 189,
			today: 480,
			day: 'Friday'
		},
		{
			average: 239,
			today: 380,
			day: 'Saturday'
		},
		{
			average: 349,
			today: 430,
			day: 'Sunday'
		}
	].map((x, i) => ({ average: x.average, today: x.today, day: x.day, idx: i + 1 }));

	const chartConfig = {
		today: {
			label: 'Today',
			color: 'var(--primary)'
		},
		average: {
			label: 'Average',
			color: 'var(--primary)'
		}
	} satisfies ChartConfig;
</script>

<Card>
	<CardHeader>
		<CardTitle>Exercise Minutes</CardTitle>
		<CardDescription>Your exercise minutes are ahead of where you normally are.</CardDescription>
	</CardHeader>
	<CardContent>
		<ChartContainer config={chartConfig} class="w-full md:h-[200px]">
			<LineChart
				{data}
				x="day"
				series={[
					{ key: 'average', color: 'var(--primary)' },
					{ key: 'today', color: 'var(--primary)' }
				]}
				points
				props={{
					yAxis: {
						format: (d) => ''
					},
					xAxis: {
						format: (d) => d.slice(0, 3)
					},
					grid: {
						x: false,
						y: true
					}
				}}
				padding={{
					top: 5,
					right: 10,
					left: 10,
					bottom: 5
				}}
			></LineChart>
		</ChartContainer>
	</CardContent>
</Card>
