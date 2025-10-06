// src/hooks.server.ts
import type { Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
	const response = await resolve(event, {
		preload: ({ type }) => type === 'font' || type === 'js' || type === 'css' // Preload fonts, JS, and CSS
	});
	return response;
};
