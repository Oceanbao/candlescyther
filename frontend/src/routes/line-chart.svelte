<script>
	import { LineChart } from 'layerchart';
	import { curveNatural } from 'd3-shape';

	import { dataFlat as multiSeriesData } from './data.js';
</script>

<div class="h-[300px] rounded-sm border p-4">
	<LineChart
		data={multiSeriesData}
		x="date"
		series={[
			{ key: 'apples', color: 'red' },
			{ key: 'bananas', color: 'blue' },
			{ key: 'oranges', color: 'green' }
		]}
		props={{
			tooltip: { context: { mode: 'quadtree' } },
			spline: {
				curve: curveNatural,
				motion: 'tween'
			}
		}}
		legend
	/>
</div>
