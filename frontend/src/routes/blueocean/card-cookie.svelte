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
	import { Label } from '$lib/components/ui/label';
	import { Switch } from '$lib/components/ui/switch';
</script>

<Card>
	<CardHeader>
		<CardTitle>Cookie Settings</CardTitle>
		<CardDescription>Manage your cookie settings here.</CardDescription>
	</CardHeader>
	<CardContent class="grid gap-6">
		<div class="flex items-center justify-between gap-4">
			<Label for="necessary" class="flex flex-col items-start">
				<span>Strictly Necessary</span>
				<span class="leading-snug font-normal text-muted-foreground">
					These cookies are essential in order to use the website and use its features.
				</span>
			</Label>
			<Switch id="necessary" checked aria-label="Necessary" />
		</div>
		<div class="flex items-center justify-between gap-4">
			<Label for="functional" class="flex flex-col items-start">
				<span>Functional Cookies</span>
				<span class="leading-snug font-normal text-muted-foreground">
					These cookies allow the website to provide personalized functionality.
				</span>
			</Label>
			<Switch id="functional" aria-label="Functional" />
		</div>
	</CardContent>
	<CardFooter>
		<Button variant="outline" class="w-full bg-transparent">Save preferences</Button>
	</CardFooter>
</Card>
