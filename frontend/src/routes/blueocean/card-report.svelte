<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import * as Card from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Select from '$lib/components/ui/select';
	import { Textarea } from '$lib/components/ui/textarea';

	let id = $state();
	let value = $state('');
	let valueAlt = $state('');
</script>

<Card.Root>
	<Card.Header>
		<Card.Title>Report an issue</Card.Title>
		<Card.Description>What area are you having problems with?</Card.Description>
	</Card.Header>
	<Card.Content class="flex flex-col gap-6">
		<div class="grid gap-4 sm:grid-cols-2">
			<div class="flex flex-col gap-3">
				<Label for={`area-${id}`}>Area</Label>
				<Select.Root type="single" bind:value>
					<Select.Trigger id={`area-${id}`} aria-label="Area" class="w-full">
						{value}
					</Select.Trigger>
					<Select.Content>
						<Select.Item value="team">Team</Select.Item>
						<Select.Item value="billing">Billing</Select.Item>
						<Select.Item value="account">Account</Select.Item>
						<Select.Item value="deployments">Deployments</Select.Item>
						<Select.Item value="support">Support</Select.Item>
					</Select.Content>
				</Select.Root>
			</div>
			<div class="flex flex-col gap-3">
				<Label for={`security-level-${id}`}>Security Level</Label>
				<Select.Root type="single" bind:value={valueAlt}>
					<Select.Trigger
						id={`security-level-${id}`}
						class="w-full [&_span]:!block [&_span]:truncate"
						aria-label="Security Level">{valueAlt}</Select.Trigger
					>
					<Select.Content>
						<Select.Item value="1">Severity 1 (Highest)</Select.Item>
						<Select.Item value="2">Severity 2</Select.Item>
						<Select.Item value="3">Severity 3</Select.Item>
						<Select.Item value="4">Severity 4 (Lowest)</Select.Item>
					</Select.Content>
				</Select.Root>
			</div>
		</div>
		<div class="flex flex-col gap-3">
			<Label for={`subject-${id}`}>Subject</Label>
			<Input id={`subject-${id}`} placeholder="I need help with..." />
		</div>
		<div class="flex flex-col gap-3">
			<Label for={`description-${id}`}>Description</Label>
			<Textarea
				id={`description-${id}`}
				placeholder="Please include all information relevant to your issue."
				class="min-h-28"
			/>
		</div>
	</Card.Content>
	<Card.Footer class="justify-end gap-2">
		<Button variant="ghost" size="sm">Cancel</Button>
		<Button size="sm">Submit</Button>
	</Card.Footer>
</Card.Root>
