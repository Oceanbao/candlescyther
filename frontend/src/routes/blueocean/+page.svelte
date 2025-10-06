<script>
	import CardAccount from './card-account.svelte';
	import CardActivity from './card-activity.svelte';
	import CardChat from './card-chat.svelte';
	import CardCookie from './card-cookie.svelte';
	import CardExercise from './card-exercise.svelte';
	import CardForm from './card-form.svelte';
	import CardReport from './card-report.svelte';
	import CardShare from './card-share.svelte';
	import CardStats from './card-stats.svelte';
	import CardTeam from './card-team.svelte';
</script>

<main
	class="grid overflow-x-auto bg-muted p-5 md:grid-cols-2 md:gap-4 md:p-6 lg:grid-cols-10 xl:grid-cols-11"
>
	<div class="grid gap-4 lg:col-span-4 xl:col-span-6">
		<CardStats />
		<div class="grid gap-1 sm:grid-cols-[auto_1fr] md:hidden">
			<CardActivity />
			<div class="pt-3 sm:pt-0 sm:pl-2 xl:pl-4">
				<CardActivity />
			</div>
			<div class="pt-3 sm:col-span-2 xl:pl-4">
				<CardExercise />
			</div>
		</div>
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-1 xl:grid-cols-2">
			<div class="flex flex-col gap-4">
				<CardForm />
				<CardTeam />
				<CardCookie />
			</div>
			<div class="flex flex-col gap-4">
				<CardAccount />
				<CardChat />
				<div class="hidden xl:block">
					<CardReport />
				</div>
			</div>
		</div>
	</div>
	<div class="flex flex-col gap-4 lg:col-span-6 xl:col-span-5">
		<div class="hidden gap-1 sm:grid-cols-[auto_1fr] md:grid">
			<div class="pt-3 sm:col-span-2 lg:pt-0">
				<CardExercise />
			</div>
		</div>
		<div class="hidden md:block">
			<!-- <CardsPayments /> -->
			<CardShare />
		</div>
		<CardShare />
		<div class="xl:hidden">
			<CardReport />
		</div>
	</div>
</main>
