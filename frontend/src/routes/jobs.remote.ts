import { query } from '$app/server';
import { computeSignals } from '$lib/server/server';

export const runComputeSignals = query(async () => {
	await computeSignals();

	return;
});
