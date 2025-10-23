<script lang="ts">
	let { filterValue, onSelectChange, sortedUniqueValue, ...restProps } = $props();
</script>

<select name="uniqueselect" onselect={() => filterValue} onchange={onSelectChange} {...restProps}>
	<option value="">All</option>
	{#each sortedUniqueValue as value (value)}
		<option {value}>
			{value}
		</option>
	{/each}
</select>
