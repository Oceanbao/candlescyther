<script lang="ts">
	import { cn } from '$lib/utils';
	import { ArrowUpIcon, CheckIcon, PlusIcon } from '@lucide/svelte/icons';

	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import { Button, buttonVariants } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import * as Command from '$lib/components/ui/command';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Input } from '$lib/components/ui/input';
	import * as Tooltip from '$lib/components/ui/tooltip';

	const users = [
		{
			name: 'Olivia Martin',
			email: 'm@example.com',
			avatar: '.'
		},
		{
			name: 'Isabella Nguyen',
			email: 'isabella.nguyen@email.com',
			avatar: '.'
		},
		{
			name: 'Emma Wilson',
			email: 'emma@example.com',
			avatar: '.'
		},
		{
			name: 'Jackson Lee',
			email: 'lee@example.com',
			avatar: '.'
		},
		{
			name: 'William Kim',
			email: 'will@email.com',
			avatar: '.'
		}
	] as const;

	type User = (typeof users)[number];

	let open = $state(false);
	let selectedUsers = $state<User[]>([]);
	let messages = $state([
		{
			role: 'agent',
			content: 'Hi, how can I help you today?'
		},
		{
			role: 'user',
			content: "Hey, I'm having trouble with my account."
		},
		{
			role: 'agent',
			content: 'What seems to be the problem?'
		},
		{
			role: 'user',
			content: "I can't log in."
		}
	]);

	let input = $state('');
	const inputLength = $derived(input.trim().length);
</script>

<Card.Root>
	<Card.Header class="flex flex-row items-center">
		<div class="flex items-center gap-4">
			<Avatar class="border">
				<AvatarImage src="." alt="Image" />
				<AvatarFallback>S</AvatarFallback>
			</Avatar>
			<div class="flex flex-col gap-0.5">
				<p class="text-sm leading-none font-medium">Sofia Davis</p>
				<p class="text-xs text-muted-foreground">m@example.com</p>
			</div>
		</div>
		<Tooltip.Provider delayDuration={0}>
			<Tooltip.Root>
				<Tooltip.Trigger
					class={buttonVariants({ variant: 'secondary' })}
					onclick={() => (open = true)}
				>
					<PlusIcon />
					<span class="sr-only">New message</span>
				</Tooltip.Trigger>
				<Tooltip.Content>New message</Tooltip.Content>
			</Tooltip.Root>
		</Tooltip.Provider>
	</Card.Header>
	<Card.Content>
		<div class="flex flex-col gap-4">
			{#each messages as message (message.content)}
				<div
					class={cn(
						'flex w-max max-w-[75%] flex-col gap-2 rounded-lg px-3 py-2 text-sm',
						message.role === 'user' ? 'ml-auto bg-primary text-primary-foreground' : 'bg-muted'
					)}
				>
					{message.content}
				</div>
			{/each}
		</div>
	</Card.Content>
	<Card.Footer>
		<form
			onsubmit={(event) => {
				event.preventDefault();
				if (inputLength === 0) return;
				messages = [
					...messages,
					{
						role: 'user',
						content: input
					}
				];
				input = '';
			}}
			class="relative w-full"
		>
			<Input
				id="message"
				placeholder="Type your message..."
				class="flex-1 pr-10"
				autocomplete="off"
				bind:value={input}
			/>
			<Button
				type="submit"
				size="icon"
				class="absolute top-1/2 right-2 size-6 -translate-y-1/2 rounded-full"
				disabled={inputLength === 0}
			>
				<ArrowUpIcon class="size-3.5" />
				<span class="sr-only">Send</span>
			</Button>
		</form>
	</Card.Footer>
</Card.Root>
<Dialog.Root {open} onOpenChange={() => (open = !open)}>
	<Dialog.Content class="gap-0 p-0 outline-none">
		<Dialog.Header class="px-4 pt-5 pb-4">
			<Dialog.Title>New message</Dialog.Title>
			<Dialog.Description
				>Invite a user to this thread. This will create a new group message.</Dialog.Description
			>
		</Dialog.Header>
		<Command.Root class="overflow-hidden rounded-t-none border-t bg-transparent">
			<Command.Input placeholder="Search user..." />
			<Command.List>
				<Command.Empty>No users found.</Command.Empty>
				<Command.Group>
					{#each users as user (user.name)}
						<Command.Item
							data-active={selectedUsers.includes(user)}
							class="data-[active=true]:opacity-50"
							onselect={() => {
								if (selectedUsers.includes(user)) {
									selectedUsers = selectedUsers.filter((selectedUser) => selectedUser !== user);
									return;
								}
								selectedUsers = [...users].filter((u) => [...selectedUsers, user].includes(u));
							}}
						>
							<Avatar class="border">
								<AvatarImage src={user.avatar || '/placeholder.svg'} alt="Image" />
								<AvatarFallback>{user.name[0]}</AvatarFallback>
							</Avatar>
							<div class="ml-2">
								<p class="text-sm leading-none font-medium">{user.name}</p>
								<p class="text-sm text-muted-foreground">{user.email}</p>
							</div>
							{#if selectedUsers.includes(user)}
								<CheckIcon class="ml-auto flex size-4 text-primary" />
								}
							{/if}
						</Command.Item>
					{/each}
				</Command.Group>
			</Command.List>
		</Command.Root>
		<Dialog.Footer class="flex items-center border-t p-4 sm:justify-between">
			{#if selectedUsers.length > 0}
				<div class="flex -space-x-2 overflow-hidden">
					{#each selectedUsers as user (user.name)}
						<Avatar class="inline-block border">
							<AvatarImage src={user.avatar || '/placeholder.svg'} />
							<AvatarFallback>{user.name[0]}</AvatarFallback>
						</Avatar>
					{/each}
				</div>
			{:else}
				<p class="text-sm text-muted-foreground">Select users to add to this thread.</p>
			{/if}
			<Button disabled={selectedUsers.length < 2} size="sm" onclick={() => (open = false)}>
				Continue
			</Button>
		</Dialog.Footer>
	</Dialog.Content>
</Dialog.Root>
