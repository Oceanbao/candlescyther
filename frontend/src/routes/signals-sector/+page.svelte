<script lang="ts">
	import { getSignalsSector } from '$lib/api/api.remote';
	import DataTable from '$lib/components/data-table/data-table.svelte';
	import { columns } from './table.js';
	import Spinner from '$lib/components/ui/spinner/spinner.svelte';

	let query = getSignalsSector();
</script>

<div class="flex flex-1 flex-col">
	<div class="@container/main m-1 flex flex-1 flex-col gap-2">
		{#if query.error}
			<em>ERROR</em>
		{:else if query.loading}
			<div class="grid h-1/2 place-content-center">
				<Spinner class="size-6" />
			</div>
		{:else}
			<DataTable data={query.current?.data ?? []} {columns} searchColumn="ticker" />
		{/if}
	</div>
</div>
