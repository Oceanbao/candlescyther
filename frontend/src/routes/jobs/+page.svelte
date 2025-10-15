<script lang="ts">
	import DataTable from './table-jobs/data-table.svelte';
	import { columns } from './table-jobs/jobs.js';

	let { data } = $props();
</script>

<div class="flex flex-1 flex-col">
	<div class="@container/main m-1 flex flex-1 flex-col gap-2">
		<DataTable data={data.jobs} {columns} />
	</div>
</div>
