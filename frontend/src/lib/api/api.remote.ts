import { command, query } from '$app/server';
import { server } from '$lib/server/client';
import { z } from 'zod/v4';

/*
  {
   data: [],
  response: Response {
    status: 200,
    statusText: 'OK',
    headers: Headers {
      'content-type': 'application/json',
      'content-length': '2',
      date: 'Mon, 20 Oct 2025 04:50:08 GMT'
    },
    body: ReadableStream { locked: true, state: 'closed', supportsBYOB: true },
    bodyUsed: true,
    ok: true,
    redirected: false,
    type: 'basic',
    url: 'http://localhost:8080/api/signals'
  }
  }
 */
export const getSignals = query(async () => {
	let { data, error: apiError } = await server.GET('/api/signals');

	return {
		data: data ?? []
	};
});

export const getSignalsUs = query(async () => {
	let { data, error: apiError } = await server.GET('/api/signals-us');

	return {
		data: data ?? []
	};
});

export const getStocks = query(async () => {
	let { data, error: apiError } = await server.GET('/api/stocks');

	return {
		data: data ?? []
	};
});

export const deleteStock = command(z.string(), async (ticker) => {
	let { data, error: apiError } = await server.DELETE('/api/stocks', {
		params: {
			query: {
				ticker
			}
		}
	});

	return {
		data: data ?? []
	};
});

export const getJobs = query(async () => {
	let { data, error: apiError } = await server.GET('/api/jobs');

	return {
		data: data ?? []
	};
});

export const getLogs = query(async () => {
	let { data, error: apiError } = await server.GET('/api/logs');

	return {
		data: data ?? []
	};
});

export const createStocks = command(z.string(), async (tickers) => {
	let res = await server.POST('/api/stocks', {
		body: {
			tickers
		}
	});
});

export const getMfSector = query(async () => {
	let { data, error } = await server.GET('/api/mf/sector');

	return {
		data: data ?? []
	};
});
