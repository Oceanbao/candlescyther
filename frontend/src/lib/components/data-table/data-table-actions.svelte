<script lang="ts">
	import EllipsisIcon from '@lucide/svelte/icons/ellipsis';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import { deleteStock } from '$lib/api/api.remote';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';

	let { id }: { id: string } = $props();

	function handleDelete() {
		deleteStock(id);
		goto(page.url.origin + '/signals');
	}
</script>

<DropdownMenu.Root>
	<DropdownMenu.Trigger>
		{#snippet child({ props })}
			<Button {...props} variant="ghost" size="icon" class="relative size-8 p-0">
				<span class="sr-only">Open menu</span>
				<EllipsisIcon />
			</Button>
		{/snippet}
	</DropdownMenu.Trigger>
	<DropdownMenu.Content>
		<DropdownMenu.Group>
			<DropdownMenu.Label>Actions</DropdownMenu.Label>
			<DropdownMenu.Item onclick={() => navigator.clipboard.writeText(id)}>
				Copy Ticker
			</DropdownMenu.Item>
			<DropdownMenu.Item onclick={() => handleDelete()} class="text-destructive">
				Delete
			</DropdownMenu.Item>
		</DropdownMenu.Group>
		<DropdownMenu.Separator />
		<DropdownMenu.Item>View chart</DropdownMenu.Item>
	</DropdownMenu.Content>
</DropdownMenu.Root>
