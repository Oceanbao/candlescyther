import { getPayment } from './table-signals/columns';

export async function load() {
	const payments = await getPayment();

	return {
		payments
	};
}
