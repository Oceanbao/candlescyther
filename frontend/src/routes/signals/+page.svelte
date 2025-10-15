<script lang="ts">
	import DataTable from './table-signals/data-table.svelte';
	import { columns } from './table-signals/signals.js';

	let { data } = $props();
</script>

<div class="flex flex-1 flex-col">
	<div class="@container/main m-1 flex flex-1 flex-col gap-2">
		<DataTable data={data.signals} {columns} />
	</div>
</div>
