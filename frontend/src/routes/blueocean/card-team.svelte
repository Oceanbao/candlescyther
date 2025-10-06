<script lang="ts">
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import { buttonVariants } from '$lib/components/ui/button';
	import {
		Card,
		CardContent,
		CardDescription,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import * as Command from '$lib/components/ui/command';
	import * as Popover from '$lib/components/ui/popover';
	import ChevronDown from '@lucide/svelte/icons/chevron-down';

	const teamMembers = [
		{
			name: 'Sofia Davis',
			email: 'm@example.com',
			avatar: '/avatars/01.png',
			role: 'Owner'
		},
		{
			name: 'Jackson Lee',
			email: 'p@example.com',
			avatar: '/avatars/02.png',
			role: 'Developer'
		},
		{
			name: 'Isabella Nguyen',
			email: 'i@example.com',
			avatar: '/avatars/03.png',
			role: 'Billing'
		}
	];

	const roles = [
		{
			name: 'Viewer',
			description: 'Can view and comment.'
		},
		{
			name: 'Developer',
			description: 'Can view, comment and edit.'
		},
		{
			name: 'Billing',
			description: 'Can view, comment and manage billing.'
		},
		{
			name: 'Owner',
			description: 'Admin-level access to all resources.'
		}
	];
</script>

<Card>
	<CardHeader>
		<CardTitle>Team Members</CardTitle>
		<CardDescription>Invite your team members to collaborate.</CardDescription>
	</CardHeader>
	<CardContent class="grid gap-6">
		{#each teamMembers as member (member.name)}
			<div class="flex items-center justify-between gap-4">
				<div class="flex items-center gap-4">
					<Avatar class="border">
						<AvatarImage src={member.avatar || '/placeholder.svg'} alt="Image" />
						<AvatarFallback>{member.name.charAt(0)}</AvatarFallback>
					</Avatar>
					<div class="flex flex-col gap-0.5">
						<p class="text-sm leading-none font-medium">{member.name}</p>
						<p class="text-xs text-muted-foreground">{member.email}</p>
					</div>
				</div>
				<Popover.Root>
					<Popover.Trigger class={buttonVariants({ variant: 'outline' })}>
						<!-- <Button variant="outline" size="sm" class="ml-auto shadow-none bg-transparent"> -->
						{member.role}
						<ChevronDown />
					</Popover.Trigger>
					<Popover.Content class="p-0" align="end">
						<Command.Root>
							<Command.Input placeholder="Select role..." />
							<Command.List>
								<Command.Empty>No roles found.</Command.Empty>
								<Command.Group>
									{#each roles as role (role)}
										<Command.Item>
											<div class="flex flex-col">
												<p class="text-sm font-medium">{role.name}</p>
												<p class="text-muted-foreground">{role.description}</p>
											</div>
										</Command.Item>
									{/each}
								</Command.Group>
							</Command.List>
						</Command.Root>
					</Popover.Content>
				</Popover.Root>
			</div>
		{/each}
	</CardContent>
</Card>
