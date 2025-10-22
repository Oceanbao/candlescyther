import createClient from 'openapi-fetch';
import type { paths, components } from './api';
import { env } from '$env/dynamic/private';

// Schema Obj
export type TLogs = components['schemas']['LogEntry'];
export type TJobs = components['schemas']['Job'];
export type TSignals = components['schemas']['Signal'];
export type TStocks = components['schemas']['Stock'];

// FIX: change this to ENV in prod
export const server = createClient<paths>({
	baseUrl: env.API_URL ?? 'http://localhost:8080/'
});
