import createClient from 'openapi-fetch';
import type { paths, components } from './api';

// Schema Obj
export type TJobs = components['schemas']['Job'];
export type TSignals = components['schemas']['Signal'];
// Response obj
type TResponseJobSuccess =
	paths['/api/jobs']['get']['responses']['200']['content']['application/json'];
type TResponseJobError = paths['/api/jobs']['get']['responses']['500'];

// FIX: change this to ENV in prod
export const server = createClient<paths>({
	baseUrl: 'http://localhost:8080/'
});
