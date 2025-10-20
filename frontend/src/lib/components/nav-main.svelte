<script lang="ts">
	import MailIcon from '@tabler/icons-svelte/icons/mail';
	import * as Sidebar from '$lib/components/ui/sidebar/index.js';
	import { IconCircleCheck, type Icon } from '@tabler/icons-svelte';
	import * as DropdownMenu from '$lib/components/ui/dropdown-menu/index.js';
	import FolderIcon from '@tabler/icons-svelte/icons/folder';
	import Share3Icon from '@tabler/icons-svelte/icons/share-3';
	import TrashIcon from '@tabler/icons-svelte/icons/trash';
	import { Button, buttonVariants } from '$lib/components/ui/button/index.js';
	import * as Dialog from '$lib/components/ui/dialog/index.js';
	import * as InputOTP from '$lib/components/ui/input-otp/index.js';
	import { REGEXP_ONLY_DIGITS_AND_CHARS } from 'bits-ui';
	import { Input } from '$lib/components/ui/input/index.js';
	import { Label } from '$lib/components/ui/label/index.js';
	import { Spinner } from './ui/spinner';
	import { createStocks } from '$lib/api/api.remote';

	let { items }: { items: { title: string; url: string; icon?: Icon }[] } = $props();

	const sidebar = Sidebar.useSidebar();

	let pass = $state('');
	let passed = $state(false);

	$effect(() => {
		if (pass === '1l0veu') {
			passed = true;
		}
	});

	let fetching = $state(false);
	let fetchDone = $state(false);
	function runFetch() {
		fetching = true;
		createStocks(tickers);
		fetching = false;
		fetchDone = true;
	}

	let addTicker = $state(false);

	let tickers = $state('');
</script>

<Sidebar.Group>
	<Sidebar.GroupContent class="flex flex-col gap-2">
		<Sidebar.Menu>
			<Sidebar.MenuItem class="flex items-center gap-1">
				<Dialog.Root
					onOpenChangeComplete={(open) => {
						pass = '';
						passed = false;
						fetchDone = false;
						if (!open) addTicker = false;
					}}
				>
					{#if sidebar.open}
						<Sidebar.MenuButton
							class="min-w-8 bg-primary text-primary-foreground duration-200 ease-linear hover:bg-primary/90 hover:text-primary-foreground active:bg-primary/90 active:text-primary-foreground"
							tooltipContent="Quick create"
						>
							{#snippet child({ props })}
								<Dialog.Trigger {...props} class={buttonVariants({ variant: 'outline' })}>
									<span>Run Jobs</span>
								</Dialog.Trigger>
							{/snippet}
						</Sidebar.MenuButton>
						<Sidebar.MenuButton
							class="min-w-8 bg-primary text-primary-foreground duration-200 ease-linear hover:bg-primary/90 hover:text-primary-foreground active:bg-primary/90 active:text-primary-foreground"
							tooltipContent="Quick create"
						>
							{#snippet child({ props })}
								<Dialog.Trigger
									{...props}
									class={buttonVariants({ variant: 'outline' })}
									onclick={() => {
										addTicker = true;
									}}
								>
									<span>Add Tickers</span>
								</Dialog.Trigger>
							{/snippet}
						</Sidebar.MenuButton>
					{/if}
					<Dialog.Content class="place-content-center sm:max-w-[425px]">
						<Dialog.Header>
							<Dialog.Title class="text-center">
								<MailIcon class="inline-block" />
							</Dialog.Title>
							<Dialog.Description class="text-center">パスワード</Dialog.Description>
						</Dialog.Header>
						{#if !passed}
							<InputOTP.Root maxlength={6} pattern={REGEXP_ONLY_DIGITS_AND_CHARS} bind:value={pass}>
								{#snippet children({ cells })}
									<InputOTP.Group>
										{#each cells as cell (cell)}
											<InputOTP.Slot {cell} />
										{/each}
									</InputOTP.Group>
								{/snippet}
							</InputOTP.Root>
						{:else if addTicker}
							<div class="flex flex-1 flex-col items-center justify-center gap-2">
								<Label for="tickers" class="text-right">Tickers</Label>
								<small class="text-red-800">(comma separated, e.g. 105.APP,1.600232)</small>
								<Input id="tickers" bind:value={tickers} class="" />
								<Dialog.Footer>
									<Button type="submit" onclick={() => runFetch()}>
										{#if fetching}
											<Spinner />
										{:else if fetchDone}
											<IconCircleCheck />
										{/if}
										Submit</Button
									>
								</Dialog.Footer>
							</div>
						{:else}
							<Dialog.Footer>
								<Button type="button" onclick={() => runFetch()}>
									{#if fetching}
										<Spinner />
									{:else if fetchDone}
										<IconCircleCheck />
									{/if}
									Run</Button
								>
							</Dialog.Footer>
						{/if}
					</Dialog.Content>
				</Dialog.Root>
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
