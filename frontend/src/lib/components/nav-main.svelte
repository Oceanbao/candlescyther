<script lang="ts">
	import CirclePlusFilledIcon from '@tabler/icons-svelte/icons/circle-plus-filled';
	import MailIcon from '@tabler/icons-svelte/icons/mail';
	import { Button } from '$lib/components/ui/button/index.js';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import type { Icon } from '@tabler/icons-svelte';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import FolderIcon from '@tabler/icons-svelte/icons/folder';
	import Share3Icon from '@tabler/icons-svelte/icons/share-3';
	import TrashIcon from '@tabler/icons-svelte/icons/trash';

	let { items }: { items: { title: string; url: string; icon?: Icon }[] } = $props();

	const sidebar = Sidebar.useSidebar();
</script>

<Sidebar.Group>
	<Sidebar.GroupContent class="flex flex-col gap-2">
		<Sidebar.Menu>
			<Sidebar.MenuItem class="flex items-center gap-2">
				<Sidebar.MenuButton
					class="min-w-8 bg-primary text-primary-foreground duration-200 ease-linear hover:bg-primary/90 hover:text-primary-foreground active:bg-primary/90 active:text-primary-foreground"
					tooltipContent="Quick create"
				>
					<CirclePlusFilledIcon />
					<span>Quick Create</span>
				</Sidebar.MenuButton>
				<DropdownMenu.Root>
					<DropdownMenu.Trigger>
						{#snippet child({ props })}
							<Sidebar.MenuAction
								{...props}
								showOnHover
								class="rounded-sm data-[state=open]:bg-accent"
							>
								<MailIcon />
								<span class="sr-only">More</span>
							</Sidebar.MenuAction>
						{/snippet}
					</DropdownMenu.Trigger>
					<DropdownMenu.Content
						class="w-24 rounded-lg"
						side={sidebar.isMobile ? 'bottom' : 'right'}
						align={sidebar.isMobile ? 'end' : 'start'}
					>
						<DropdownMenu.Item>
							<FolderIcon />
							<span>Open</span>
						</DropdownMenu.Item>
						<DropdownMenu.Item>
							<Share3Icon />
							<span>Share</span>
						</DropdownMenu.Item>
						<DropdownMenu.Separator />
						<DropdownMenu.Item variant="destructive">
							<TrashIcon />
							<span>Delete</span>
						</DropdownMenu.Item>
					</DropdownMenu.Content>
				</DropdownMenu.Root>
				<!-- <Button -->
				<!-- 	size="icon" -->
				<!-- 	class="size-8 group-data-[collapsible=icon]:opacity-0" -->
				<!-- 	variant="outline" -->
				<!-- > -->
				<!-- 	<MailIcon /> -->
				<!-- 	<span class="sr-only">Inbox</span> -->
				<!-- </Button> -->
			</Sidebar.MenuItem>
		</Sidebar.Menu>
		<Sidebar.Menu>
			{#each items as item (item.title)}
				<a href={item.url}>
					<Sidebar.MenuItem>
						<Sidebar.MenuButton tooltipContent={item.title}>
							{#if item.icon}
								<item.icon />
							{/if}
							<span>{item.title}</span>
						</Sidebar.MenuButton>
					</Sidebar.MenuItem>
				</a>
			{/each}
		</Sidebar.Menu>
	</Sidebar.GroupContent>
</Sidebar.Group>
