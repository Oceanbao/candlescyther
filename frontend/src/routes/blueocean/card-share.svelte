<script lang="ts">
	import { Avatar, AvatarFallback, AvatarImage } from '$lib/components/ui/avatar';
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Select from '$lib/components/ui/select';
	import { Separator } from '$lib/components/ui/separator';

	const people = [
		{
			name: 'Olivia Martin',
			email: 'm@example.com',
			avatar: '.'
		},
		{
			name: 'Isabella Nguyen',
			email: 'b@example.com',
			avatar: '.'
		},
		{
			name: 'Sofia Davis',
			email: 'p@example.com',
			avatar: '.'
		},
		{
			name: 'Ethan Thompson',
			email: 'e@example.com',
			avatar: '.'
		}
	];

	let valueSelected = $state('');
</script>

<Card.Root>
	<Card.Header>
		<Card.Title>Share this document</Card.Title>
		<Card.Description>Anyone with the link can view this document.</Card.Description>
	</Card.Header>
	<Card.Content>
		<div class="flex items-center gap-2">
			<Label for="link" class="sr-only">Link</Label>
			<Input id="link" value="http://example.com/link/to/document" class="h-8" readonly />
			<Button size="sm" variant="outline" class="bg-transparent shadow-none">Copy Link</Button>
		</div>
		<Separator class="my-4" />
		<div class="flex flex-col gap-4">
			<div class="text-sm font-medium">People with access</div>
			<div class="grid gap-6">
				{#each people as person (person.name)}
					<div class="flex items-center justify-between gap-4">
						<div class="flex items-center gap-4">
							<Avatar>
								<AvatarImage src={person.avatar || '/placeholder.svg'} alt="Image" />
								<AvatarFallback>{person.name.charAt(0)}</AvatarFallback>
							</Avatar>
							<div>
								<p class="text-sm leading-none font-medium">{person.name}</p>
								<p class="text-sm text-muted-foreground">{person.email}</p>
							</div>
						</div>
						<Select.Root type="single" bind:value={valueSelected}>
							<Select.Trigger class="ml-auto pr-2" aria-label="Edit" size="sm">
								{valueSelected}
							</Select.Trigger>
							<Select.Content align="end">
								<Select.Item value="edit">Can edit</Select.Item>
								<Select.Item value="view">Can view</Select.Item>
							</Select.Content>
						</Select.Root>
					</div>
				{/each}
			</div>
		</div>
	</Card.Content>
</Card.Root>
