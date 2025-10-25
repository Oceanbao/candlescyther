<script lang="ts">
	let { filterValue, onSelectChange, sortedUniqueValue, ...restProps } = $props();

	const formatter = (market: number) => {
		if (market > 100 && market < 110) {
			return 'US';
		}
		return 'A/H';
	};
</script>

<select name="uniqueselect" onselect={() => filterValue} onchange={onSelectChange} {...restProps}>
	<option value="">All</option>
	{#each sortedUniqueValue as value (value)}
		<option {value}>
			{formatter(value)}
		</option>
	{/each}
</select>
