<script lang="ts">
	import { getSignalsUs } from '$lib/api/api.remote';
	import DataTable from './table-signals-us/data-table.svelte';
	import { columns } from './table-signals-us/signals-us.js';

	let query = getSignalsUs();
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
