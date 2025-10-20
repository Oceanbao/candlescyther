<script lang="ts">
	import { getLogs } from '$lib/api/api.remote';
	import DataTable from './table-logs/data-table.svelte';
	import { columns } from './table-logs/logs.js';

	let query = getLogs();
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
