<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import {
		Card,
		CardContent,
		CardDescription,
		CardFooter,
		CardHeader,
		CardTitle
	} from '$lib/components/ui/card';
	import { Checkbox } from '$lib/components/ui/checkbox';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as RadioGroup from '$lib/components/ui/radio-group';
	import Textarea from '$lib/components/ui/textarea/textarea.svelte';

	const plans = [
		{
			id: 'starter',
			name: 'Starter Plan',
			description: 'Perfect for small businesses.',
			price: '$10'
		},
		{
			id: 'pro',
			name: 'Pro Plan',
			description: 'More features and storage.',
			price: '$20'
		}
	] as const;
</script>

<Card>
	<CardHeader>
		<CardTitle class="text-lg">Upgrade your subscription</CardTitle>
		<CardDescription class="text-balance">
			You are currently on the free plan. Upgrade to the pro plan to get access to all features.
		</CardDescription>
	</CardHeader>
	<CardContent>
		<div class="flex flex-col gap-6">
			<div class="flex flex-col gap-3 md:flex-row">
				<div class="flex flex-1 flex-col gap-2">
					<Label for="name">Name</Label>
					<Input id="name" placeholder="Evil Rabbit" />
				</div>
				<div class="flex flex-1 flex-col gap-2">
					<Label for="email">Email</Label>
					<Input id="email" placeholder="example@acme.com" />
				</div>
			</div>
			<div class="flex flex-col gap-2">
				<Label for="card-number">Card Number</Label>
				<div class="grid grid-cols-2 gap-3 md:grid-cols-[1fr_80px_60px]">
					<Input
						id="card-number"
						placeholder="1234 1234 1234 1234"
						class="col-span-2 md:col-span-1"
					/>
					<Input id="card-number-expiry" placeholder="MM/YY" />
					<Input id="card-number-cvc" placeholder="CVC" />
				</div>
			</div>
			<fieldset class="flex flex-col gap-3">
				<legend class="text-sm font-medium">Plan</legend>
				<p class="text-sm text-muted-foreground">Select the plan that best fits your needs.</p>
				<RadioGroup.Root value="starter" class="grid gap-3 md:grid-cols-2">
					{#each plans as p (p.id)}
						<Label
							class="flex items-start gap-3 rounded-lg border p-3 has-[[data-state=checked]]:border-ring has-[[data-state=checked]]:bg-primary/10"
							for={p.name}
						>
							<RadioGroup.Item
								value={p.id}
								id={p.name}
								class="data-[state=checked]:border-primary"
							/>
							<div class="grid gap-1 font-normal">
								<div class="font-medium">{p.name}</div>
								<div class="text-xs leading-snug text-balance text-muted-foreground">
									{p.description}
								</div>
							</div>
						</Label>
					{/each}
				</RadioGroup.Root>
			</fieldset>
			<div class="flex flex-col gap-2">
				<Label for="notes">Notes</Label>
				<Textarea id="notes" placeholder="Enter notes" />
			</div>
			<div class="flex flex-col gap-3">
				<div class="flex items-center gap-2">
					<Checkbox id="terms" />
					<Label for="terms" class="font-normal">I agree to the terms and conditions</Label>
				</div>
				<div class="flex items-center gap-2">
					<Checkbox id="newsletter" checked />
					<Label for="newsletter" class="font-normal">Allow us to send you emails</Label>
				</div>
			</div>
		</div>
	</CardContent>
	<CardFooter class="flex justify-between">
		<Button variant="outline" size="sm">Cancel</Button>
		<Button size="sm">Upgrade Plan</Button>
	</CardFooter>
</Card>
