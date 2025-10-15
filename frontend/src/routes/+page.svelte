<script lang="ts">
	import DataTable from './table-signals/data-table.svelte';
	import { columns } from './table-signals/columns.js';

	let { data } = $props();
</script>

<DataTable data={data.payments} {columns} />
<!-- <div class="flex flex-1 flex-col"> -->
<!-- 	<div class="@container/main flex flex-1 flex-col gap-2"> -->
<!-- 		<!-- <div class="flex flex-col gap-4 py-4 md:gap-6 md:py-6"> -->
<!-- 		<!-- 	<div>Section A</div> -->
<!-- 		<!-- 	<div class="px-4 lg:px-6"> -->
<!-- 		<!-- 		<div>Section B</div> -->
<!-- 		<!-- 	</div> -->
<!-- 		<!-- 	<div>Section C</div> -->
<!-- 		<!-- </div> -->
<!-- 	</div> -->
<!-- </div> -->
