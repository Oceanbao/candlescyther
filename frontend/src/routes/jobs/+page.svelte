<script lang="ts">
	import { getJobs } from '$lib/api/api.remote';
	import DataTable from './table-jobs/data-table.svelte';
	import { columns } from './table-jobs/jobs.js';

	let query = getJobs();
</script>

<div class="flex flex-1 flex-col">
	<div class="@container/main m-1 flex flex-1 flex-col gap-2">
		{#if query.error}
			<em>ERROR</em>
		{:else if query.loading}
			<em>LOADING...</em>
		{:else}
			<DataTable data={query.current?.data ?? []} {columns} />
		{/if}
	</div>
</div>
