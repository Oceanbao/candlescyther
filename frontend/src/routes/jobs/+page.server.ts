import { server } from '$lib/server/client';

export async function load() {
	const { data, error } = await server.GET('/api/jobs', {});

	return {
		jobs: data ?? []
	};
}
